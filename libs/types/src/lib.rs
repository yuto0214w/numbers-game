use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod http;
pub mod ws;

mod room_id;
pub use room_id::RoomId;

pub const BOARD_SIZE: usize = 8;

pub type Position = (usize, usize);
pub type BoardPieces = [[Option<Piece>; BOARD_SIZE]; BOARD_SIZE];

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    #[default]
    A,
    B,
}

impl Side {
    pub fn reverse(self) -> Self {
        match self {
            Self::A => Self::B,
            Self::B => Self::A,
        }
    }
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Player {
    pub public_id: Uuid,
    pub name: String,
    pub side: Side,
}

#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
pub struct Piece {
    pub side: Side,
    pub number: u8,
}
