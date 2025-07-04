use std::{net::IpAddr, sync::Arc};

use axum::{
    body::Bytes,
    extract::ws::{CloseFrame, Message, Utf8Bytes, WebSocket, close_code},
};
use futures_util::{SinkExt as _, StreamExt as _};
use numbers_comm_types::{
    RoomId,
    ws::{CreateUser, PlayerAction, PlayerActionWithAuth, PlayerActionWithoutAuth, Responses},
};
use parking_lot::Mutex;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::util::{WebSocketReceiveAction, WebSocketSendAction, generate_name, log_error, log_ws};

use super::{QUEUE_MESSAGE_LIMIT, session::map::GameSessionLock};

#[inline(always)]
pub async fn handle_socket(mut socket: WebSocket, ip: IpAddr, room_id: RoomId) {
    match socket
        .send(Message::Ping(Bytes::from_static(&[1, 2, 3])))
        .await
    {
        Ok(_) => log_ws(ip, Ok(WebSocketSendAction::SendPing)),
        Err(_) => return log_ws(ip, Err(WebSocketSendAction::SendPing)),
    }
    match socket.recv().await {
        Some(Ok(Message::Pong(_))) => log_ws(ip, WebSocketReceiveAction::GotPong),
        _ => return,
    }

    let session_lock = GameSessionLock::new(room_id);
    let mut queue_rx = session_lock.with_read(|session| session.subscribe_queue());
    let (conn_tx, mut conn_rx) = mpsc::channel(QUEUE_MESSAGE_LIMIT);
    let (mut sender, mut receiver) = socket.split();

    let mut send_task = tokio::spawn(async move {
        macro_rules! send_text_message {
            ($text:expr) => {{
                let res = sender.send(Message::Text($text.to_owned().into())).await;
                match res {
                    Ok(_) => log_ws(ip, Ok(WebSocketSendAction::SendText($text))),
                    Err(_) => log_ws(ip, Err(WebSocketSendAction::SendText($text))),
                }
                res
            }};
        }
        macro_rules! send_close_message {
            ($code:expr, $reason:literal) => {{
                let cf = CloseFrame {
                    code: $code,
                    reason: Utf8Bytes::from_static($reason),
                };
                match sender.send(Message::Close(Some(cf.clone()))).await {
                    Ok(_) => log_ws(ip, Ok(WebSocketSendAction::SendClose(cf))),
                    Err(_) => log_ws(ip, Err(WebSocketSendAction::SendClose(cf))),
                }
            }};
        }
        loop {
            tokio::select! {
                val = queue_rx.recv() => match val {
                    Ok(event) => {
                        let event_str = serde_json::to_string(&event).unwrap();
                        if send_text_message!(&event_str).is_err() {
                            break;
                        }
                    }
                    Err(error) => {
                        log_error!("queue_recv", error);
                        send_close_message!(close_code::AGAIN, "Server Lagged");
                        break;
                    }
                },
                val = conn_rx.recv() => match val.unwrap() {
                    Responses::UserCreated(_)
                    | Responses::Authorized
                    | Responses::ActionNotAccepted
                    | Responses::SessionExpired => {
                        let text = serde_json::to_string(val.as_ref().unwrap()).unwrap();
                        if send_text_message!(&text).is_err() {
                            break;
                        }
                    }
                    Responses::AuthorizedInternal(_) => unreachable!(),
                    Responses::GotInvalidData => {
                        send_close_message!(close_code::INVALID, "Invalid Data");
                        break;
                    }
                },
            }
        }
    });

    let private_id: Arc<Mutex<Option<Uuid>>> = Arc::new(Mutex::new(None));
    let mut recv_task = tokio::spawn({
        let private_id = Arc::clone(&private_id);
        async move {
            while let Some(Ok(msg)) = receiver.next().await {
                let permit = conn_tx.reserve().await.unwrap();
                match msg {
                    Message::Text(text) => {
                        log_ws(ip, WebSocketReceiveAction::GotText(&text));
                        if let Ok(action) = serde_json::from_str::<PlayerAction>(&text) {
                            let cloned_private_id = *private_id.lock();
                            if let Some(msg) = handle_game(action, room_id, cloned_private_id) {
                                if let Responses::AuthorizedInternal(authorized_private_id) = msg {
                                    *private_id.lock() = Some(authorized_private_id);
                                    permit.send(Responses::Authorized);
                                } else {
                                    permit.send(msg);
                                }
                            }
                        } else {
                            permit.send(Responses::GotInvalidData);
                        }
                    }
                    Message::Close(c) => {
                        log_ws(ip, WebSocketReceiveAction::GotClose(c));
                        break;
                    }
                    _ => permit.send(Responses::GotInvalidData),
                }
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }
    if let Some(private_id) = *private_id.lock() {
        session_lock.with_write(|session| session.get_board_mut().remove_player(private_id));
    };
}

#[inline(always)]
fn handle_game(
    action: PlayerAction,
    room_id: RoomId,
    private_id: Option<Uuid>,
) -> Option<Responses> {
    GameSessionLock::new(room_id).with_write(|session| {
        let board = session.get_board_mut();
        match action {
            PlayerAction::WithoutAuth(action) => match action {
                PlayerActionWithoutAuth::JoinTeam(side) => {
                    match board.create_player(side, generate_name()) {
                        Some((private_id, public_id)) => {
                            return Some(Responses::UserCreated(CreateUser::Ok {
                                private_id,
                                public_id,
                            }));
                        }
                        None => {
                            return Some(Responses::UserCreated(CreateUser::Err {
                                message: "PLAYER_LIMIT_EXCEEDED",
                            }));
                        }
                    }
                }
                PlayerActionWithoutAuth::Resume(private_id) => {
                    if !board.contains_player(private_id) {
                        return Some(Responses::SessionExpired);
                    }
                    return Some(Responses::AuthorizedInternal(private_id));
                }
            },
            PlayerAction::WithAuth(action) if private_id.is_some() => match action {
                PlayerActionWithAuth::LeaveTeam => todo!(),
                PlayerActionWithAuth::Move(old_position, new_position) => {
                    if board
                        .move_piece(private_id.unwrap(), old_position, new_position)
                        .is_err()
                    {
                        return Some(Responses::ActionNotAccepted);
                    }
                }
            },
            PlayerAction::WithAuth(_) => return Some(Responses::ActionNotAccepted),
        }
        None
    })
}
