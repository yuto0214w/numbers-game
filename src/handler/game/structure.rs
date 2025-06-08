use serde::{ser::SerializeStruct as _, Deserialize, Serialize};
use uuid::Uuid;

use crate::util::deser_utils;

use super::session::{PieceData, PlayerData, Position, Side};

// HTTP

#[derive(Debug, Clone, Copy, Serialize)]
pub struct HttpPieceData {
    position: Position,
    number: u8,
}

impl HttpPieceData {
    pub fn from_piece_data(piece_data: &Vec<Vec<Option<PieceData>>>, side: Side) -> Vec<Self> {
        let mut v = Vec::new();
        for (y, row) in piece_data.iter().enumerate() {
            for (x, piece) in row.iter().enumerate() {
                if piece.is_some_and(|p| p.side == side) {
                    v.push(Self {
                        position: (x, y),
                        number: piece.unwrap().number,
                    });
                }
            }
        }
        v
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RoomData {
    pub room_id: Uuid,
    pub board_size: usize,
    pub current_turn: Side,
    pub top_players: Vec<PlayerData>,
    pub top_pieces: Vec<HttpPieceData>,
    pub bottom_players: Vec<PlayerData>,
    pub bottom_pieces: Vec<HttpPieceData>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateUserData {
    pub success: bool,
    pub message: Option<&'static str>,
    pub side: Option<Side>,
    pub private_id: Option<Uuid>,
    pub public_id: Option<Uuid>,
    pub name: Option<String>,
}

// WebSocket

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct AuthData {
    #[serde(rename = "i")]
    pub private_id: Uuid,
}

#[derive(Debug, Clone, Copy)]
pub enum WebSocketMessaging {
    HeartbeatAck,
    NotAccepted(PlayerAction),
    SessionExpired,
    GotBinary,
    GotInvalidData,
}

impl Serialize for WebSocketMessaging {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state;
        if matches!(*self, Self::NotAccepted(_)) {
            state = serializer.serialize_struct("WebSocketMessaging", 2)?;
        } else {
            state = serializer.serialize_struct("WebSocketMessaging", 1)?;
        }
        match *self {
            Self::HeartbeatAck => {
                state.serialize_field("t", &100)?;
            }
            Self::NotAccepted(action) => {
                state.serialize_field("t", &101)?;
                state.serialize_field("c", &action)?;
            }
            Self::SessionExpired => {
                state.serialize_field("t", &102)?;
            }
            Self::GotBinary => {
                state.serialize_field("t", &103)?;
            }
            Self::GotInvalidData => {
                state.serialize_field("t", &104)?;
            }
        }
        state.end()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PlayerAction {
    Heartbeat,
    SelectPiece(Position),
    MovePiece(Position, Position),
}

impl Serialize for PlayerAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state;
        if matches!(*self, Self::Heartbeat) {
            state = serializer.serialize_struct("PlayerAction", 1)?;
        } else {
            state = serializer.serialize_struct("PlayerAction", 2)?;
        }
        match *self {
            Self::Heartbeat => {
                state.serialize_field("t", &99)?;
            }
            Self::SelectPiece(ref pos) => {
                state.serialize_field("t", &1)?;
                state.serialize_field("c", pos)?;
            }
            Self::MovePiece(ref pos1, ref pos2) => {
                state.serialize_field("t", &2)?;
                state.serialize_field("c", &(pos1, pos2))?;
            }
        }
        state.end()
    }
}

deser_utils::dcwt! {
    target: PlayerAction,
    tag: "t",
    content: "c",
    with_no_content: [
        Heartbeat = 99
    ],
    with_single_content: [
        Position => SelectPiece(a) = 1
    ],
    with_tuplelike_content: [
        (Position, Position) => MovePiece(a, b) = 2
    ]
}

#[derive(Debug, Clone)]
pub enum RoomEvent {
    SelectPiece(Position),
    MovePiece(Position, Position),
    TopPlayerJoin(String),
    BottomPlayerJoin(String),
    TopPlayerLeave,
    BottomPlayerLeave,
}

impl Serialize for RoomEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state;
        if matches!(*self, Self::TopPlayerLeave | Self::BottomPlayerLeave) {
            state = serializer.serialize_struct("RoomEvent", 1)?;
        } else {
            state = serializer.serialize_struct("RoomEvent", 2)?;
        }
        match *self {
            Self::SelectPiece(ref pos) => {
                state.serialize_field("t", &1)?;
                state.serialize_field("c", pos)?;
            }
            Self::MovePiece(ref pos1, ref pos2) => {
                state.serialize_field("t", &2)?;
                state.serialize_field("c", &(pos1, pos2))?;
            }
            Self::TopPlayerJoin(ref name) => {
                state.serialize_field("t", &3)?;
                state.serialize_field("c", name)?;
            }
            Self::BottomPlayerJoin(ref name) => {
                state.serialize_field("t", &4)?;
                state.serialize_field("c", name)?;
            }
            Self::TopPlayerLeave => {
                state.serialize_field("t", &5)?;
            }
            Self::BottomPlayerLeave => {
                state.serialize_field("t", &6)?;
            }
        }
        state.end()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RoomEventWithId {
    #[serde(rename = "i")]
    pub public_id: Uuid,
    #[serde(flatten)]
    pub event: RoomEvent,
}
