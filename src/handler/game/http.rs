use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Path, Request, WebSocketUpgrade},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse as _, Response},
    Json,
};
use uuid::Uuid;

use crate::util::{
    generate_name, log_ws, SimpleResponse, SimpleResponseWithHeaders, WebSocketAction,
};

use super::{
    session::{
        map::{get_game_session_map, get_immutable_session, get_mutable_session},
        GameSession, GameSessionConfig, Side,
    },
    structure::{CreateUserData, HttpPieceData, RoomData},
    ws::handle_socket,
};

pub async fn new_room() -> Response {
    let room_id = Uuid::new_v4();
    get_game_session_map().write().insert(
        room_id,
        GameSession::new(room_id, GameSessionConfig::default()),
    );
    let redirect_url = format!("/room/{}", room_id);
    let redirect_text = format!("Redirecting you to {}", &redirect_url);
    SimpleResponseWithHeaders {
        original_response: SimpleResponse {
            status_code: StatusCode::FOUND,
            content_type: "text/plain; charset=utf-8",
            content: redirect_text,
        },
        headers: [(header::LOCATION, redirect_url)],
    }
    .into_response()
}

pub async fn room_existence_check(Path(room_id): Path<Uuid>, req: Request, next: Next) -> Response {
    if get_game_session_map().read().get(&room_id).is_none() {
        return SimpleResponse {
            status_code: StatusCode::NOT_FOUND,
            content_type: "text/plain; charset=utf-8",
            content: "Invalid Room ID",
        }
        .into_response();
    }
    next.run(req).await
}

// これより下、部屋が存在するものとして処理を進めてOK

pub async fn room_data(Path(room_id): Path<Uuid>) -> Response {
    let session = get_immutable_session(room_id);
    let piece_data = session.get_pieces();
    SimpleResponse {
        status_code: StatusCode::OK,
        content_type: "application/json",
        content: Json(RoomData {
            room_id,
            board_size: session.get_board_size(),
            current_turn: session.get_current_turn(),
            top_players: session.get_player_data(Side::Top),
            top_pieces: HttpPieceData::from_piece_data(piece_data, Side::Top),
            bottom_players: session.get_player_data(Side::Bottom),
            bottom_pieces: HttpPieceData::from_piece_data(piece_data, Side::Bottom),
        }),
    }
    .into_response()
}

#[inline(always)]
fn try_create_player(room_id: Uuid, side: Side) -> Response {
    let mut session = get_mutable_session(room_id);
    let name = generate_name();
    match session.create_player(side, &name) {
        Some(private_id) => {
            let public_id = session.get_public_id(private_id);
            SimpleResponse {
                status_code: StatusCode::OK,
                content_type: "application/json",
                content: Json(CreateUserData {
                    success: true,
                    message: None,
                    side: Some(side),
                    private_id: Some(private_id),
                    public_id: Some(public_id),
                    name: Some(name),
                }),
            }
            .into_response()
        }
        None => SimpleResponse {
            status_code: StatusCode::BAD_REQUEST,
            content_type: "application/json",
            content: Json(CreateUserData {
                success: false,
                message: Some("PLAYER_LIMIT_EXCEEDED"),
                side: None,
                private_id: None,
                public_id: None,
                name: None,
            }),
        }
        .into_response(),
    }
}

pub async fn join_top(Path(room_id): Path<Uuid>) -> Response {
    try_create_player(room_id, Side::Top)
}

pub async fn join_bottom(Path(room_id): Path<Uuid>) -> Response {
    try_create_player(room_id, Side::Bottom)
}

pub async fn leave(Path((room_id, private_id)): Path<(Uuid, Uuid)>) -> Response {
    if !get_mutable_session(room_id).remove_player(private_id) {
        return SimpleResponse {
            status_code: StatusCode::NOT_FOUND,
            content_type: "text/plain; charset=utf-8",
            content: "Invalid Player ID",
        }
        .into_response();
    }
    SimpleResponse {
        status_code: StatusCode::OK,
        content_type: "text/plain; charset=utf-8",
        content: "Successful",
    }
    .into_response()
}

pub async fn serve_ws(
    Path(room_id): Path<Uuid>,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response {
    ws.on_upgrade(move |socket| async move {
        let ip = addr.ip();
        log_ws(ip, WebSocketAction::Connect);
        handle_socket(socket, ip, room_id).await;
        log_ws(ip, WebSocketAction::Disconnect);
    })
}
