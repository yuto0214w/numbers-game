use std::net::SocketAddr;

use axum::{
    Json,
    extract::{ConnectInfo, Path, Request, WebSocketUpgrade},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse as _, Response},
};
use numbers_comm_types::{
    RoomId,
    http::{CreateRoom, HttpBoardConfig, RoomData, RoomList, RoomSummary, ServerInfo},
};

use crate::util::{JsonResponse, WebSocketAction, log_ws};

use super::{
    MINIMUM_SERVER_VERSION,
    session::{
        GameSession,
        board::config::BoardConfig,
        map::{GAME_SESSION_MAP, GameSessionLock},
    },
    ws::handle_socket,
};

pub async fn server_info() -> Response {
    JsonResponse {
        status_code: StatusCode::OK,
        content: ServerInfo {
            min_version: MINIMUM_SERVER_VERSION,
        },
    }
    .into_response()
}

pub async fn new_room(Json(config): Json<HttpBoardConfig>) -> Response {
    let config = match BoardConfig::try_from(config) {
        Ok(config) => config,
        Err(message) => {
            return JsonResponse {
                status_code: StatusCode::BAD_REQUEST,
                content: CreateRoom::Err { message },
            }
            .into_response();
        }
    };
    let room_id = RoomId::new();
    GAME_SESSION_MAP
        .write()
        .insert(room_id, GameSession::new(config));
    JsonResponse {
        status_code: StatusCode::CREATED,
        content: CreateRoom::Ok { room_id },
    }
    .into_response()
}

pub async fn room_list() -> Response {
    let ids: Vec<RoomId> = GAME_SESSION_MAP.read().keys().copied().collect();
    JsonResponse {
        status_code: StatusCode::OK,
        content: RoomList(
            ids.into_iter()
                .map(|id| RoomSummary {
                    id,
                    players: GameSessionLock::new(id).with_read(|session| {
                        session
                            .get_board()
                            .get_players()
                            .iter()
                            .map(|player| player.name.to_owned())
                            .collect()
                    }),
                })
                .collect(),
        ),
    }
    .into_response()
}

pub async fn room_existence_check(
    Path(room_id): Path<RoomId>,
    req: Request,
    next: Next,
) -> Response {
    if GAME_SESSION_MAP.read().get(&room_id).is_none() {
        return JsonResponse {
            status_code: StatusCode::NOT_FOUND,
            content: None::<()>,
        }
        .into_response();
    }
    next.run(req).await
}

// vvv 部屋が存在するものとして処理を進めてOK vvv

pub async fn room_info(Path(room_id): Path<RoomId>) -> Response {
    GameSessionLock::new(room_id).with_read(|session| {
        let board = session.get_board();
        JsonResponse {
            status_code: StatusCode::OK,
            content: RoomData {
                room_id,
                current_turn: board.get_current_turn(),
                players: board.get_players(),
                pieces: board.get_pieces(),
            },
        }
        .into_response()
    })
}

pub async fn serve_ws(
    Path(room_id): Path<RoomId>,
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
