use std::{borrow::Cow, net::IpAddr};

use axum::extract::ws::{close_code, CloseFrame, Message, WebSocket};
use futures_util::{SinkExt as _, StreamExt as _};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::util::{log_error, log_ws, WebSocketReceiveAction, WebSocketSendAction};

use super::{
    session::map::{get_immutable_session, get_mutable_session},
    structure::{AuthData, PlayerAction, WebSocketMessaging},
    QUEUE_MESSAGE_LIMIT,
};

// TODO: このコードには不備があります。

#[inline(always)]
pub async fn handle_socket(mut socket: WebSocket, ip: IpAddr, room_id: Uuid) {
    match socket.send(Message::Ping(vec![1, 2, 3])).await {
        Ok(_) => {
            log_ws(ip, Ok(WebSocketSendAction::SendPing));
        }
        Err(_) => {
            log_ws(ip, Err(WebSocketSendAction::SendPing));
            return;
        }
    }
    match socket.recv().await {
        Some(Ok(Message::Pong(_))) => {
            log_ws(ip, WebSocketReceiveAction::GotPong);
        }
        _ => return,
    }
    // vvv 通信関連の変数定義ここから vvv
    let mut queue_rx = get_immutable_session(room_id)
        .get_queue_sender()
        .subscribe();
    let (conn_tx, mut conn_rx) = mpsc::channel(QUEUE_MESSAGE_LIMIT);
    let (mut sender, mut receiver) = socket.split();
    // ^^^ 通信関連の変数定義ここまで ^^^
    let mut send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                val = queue_rx.recv() => match val {
                    Ok(event) => {
                        let event_str = serde_json::to_string(&event).unwrap();
                        match sender.send(Message::Text(event_str.clone())).await {
                            Ok(_) => {
                                log_ws(ip, Ok(WebSocketSendAction::SendText(&event_str)));
                            }
                            Err(error) => {
                                log_error!("socket_send", error);
                                break;
                            }
                        }
                    }
                    Err(error) => {
                        log_error!("socket_recv", error);
                        let _ = sender
                            .send(Message::Close(Some(CloseFrame {
                                code: close_code::AGAIN,
                                reason: Cow::from("Server Lagged"),
                            })))
                            .await;
                        break;
                    }
                },
                val = conn_rx.recv() => match val.unwrap() {
                    WebSocketMessaging::HeartbeatAck
                    | WebSocketMessaging::NotAccepted(_)
                    | WebSocketMessaging::SessionExpired => {
                        let text = serde_json::to_string(val.as_ref().unwrap()).unwrap();
                        if sender.send(Message::Text(text.clone())).await.is_err() {
                            log_ws(ip, Err(WebSocketSendAction::SendText(&text)));
                            break;
                        }
                        log_ws(ip, Ok(WebSocketSendAction::SendText(&text)));
                    }
                    WebSocketMessaging::GotBinary => {
                        let cf = CloseFrame {
                            code: close_code::UNSUPPORTED,
                            reason: Cow::from("Binary Not Recognized"),
                        };
                        if sender.send(Message::Close(Some(cf.clone()))).await.is_err() {
                            log_ws(ip, Err(WebSocketSendAction::SendClose(&cf)));
                        } else {
                            log_ws(ip, Ok(WebSocketSendAction::SendClose(&cf)));
                        }
                        break;
                    }
                    WebSocketMessaging::GotInvalidData => {
                        let cf = CloseFrame {
                            code: close_code::INVALID,
                            reason: Cow::from("Invalid Data"),
                        };
                        if sender.send(Message::Close(Some(cf.clone()))).await.is_err() {
                            log_ws(ip, Err(WebSocketSendAction::SendClose(&cf)));
                        } else {
                            log_ws(ip, Ok(WebSocketSendAction::SendClose(&cf)));
                        }
                        break;
                    }
                },
            }
        }
    });
    let mut recv_task = tokio::spawn(async move {
        let mut private_id = None::<Uuid>;
        while let Some(Ok(msg)) = receiver.next().await {
            let permit = conn_tx.reserve().await.unwrap();
            match msg {
                Message::Text(text) => {
                    log_ws(ip, WebSocketReceiveAction::GotText(&text));
                    match private_id {
                        Some(private_id)
                            if get_immutable_session(room_id).contains_player(private_id) =>
                        {
                            match serde_json::from_str::<PlayerAction>(&text) {
                                Ok(action) => {
                                    if let Some(msg) = handle_game(action, room_id, private_id) {
                                        permit.send(msg);
                                    }
                                }
                                _ => permit.send(WebSocketMessaging::GotInvalidData),
                            }
                        }
                        Some(_) => {
                            private_id = None;
                            permit.send(WebSocketMessaging::SessionExpired);
                        }
                        None => match serde_json::from_str::<AuthData>(&text) {
                            Ok(AuthData {
                                private_id: provided_private_id,
                            }) if get_immutable_session(room_id)
                                .contains_player(provided_private_id) =>
                            {
                                private_id = Some(provided_private_id);
                            }
                            _ => permit.send(WebSocketMessaging::GotInvalidData),
                        },
                    }
                }
                Message::Binary(_) => {
                    log_ws(ip, WebSocketReceiveAction::GotBinary);
                    permit.send(WebSocketMessaging::GotBinary);
                }
                Message::Close(c) => {
                    log_ws(ip, WebSocketReceiveAction::GotClose(&c));
                    break;
                }
                _ => permit.send(WebSocketMessaging::GotInvalidData),
            }
        }
    });
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}

#[inline(always)]
fn handle_game(
    action: PlayerAction,
    room_id: Uuid,
    private_id: Uuid,
) -> Option<WebSocketMessaging> {
    let mut session = get_mutable_session(room_id);
    match action {
        PlayerAction::Heartbeat => {
            session.update_heartbeat(private_id);
            return Some(WebSocketMessaging::HeartbeatAck);
        }
        PlayerAction::SelectPiece(position) => {
            if session.select_piece(private_id, position).is_err() {
                return Some(WebSocketMessaging::NotAccepted(action));
            }
        }
        PlayerAction::MovePiece(old_position, new_position) => {
            if session
                .move_piece(private_id, old_position, new_position)
                .is_err()
            {
                return Some(WebSocketMessaging::NotAccepted(action));
            }
        }
    }
    None
}
