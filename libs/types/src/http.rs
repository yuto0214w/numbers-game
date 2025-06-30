use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{BoardPieces, Player, RoomId, Side};

// receive

#[derive(Deserialize, JsonSchema)]
pub struct HttpBoardConfig {
    pub team_player_limit: Option<usize>,
    pub first_side: Option<Side>,
}

// send

#[derive(Serialize, JsonSchema)]
pub struct ServerInfo {
    pub min_version: usize,
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "success")]
pub enum CreateRoom {
    #[serde(rename = true)]
    Ok { room_id: RoomId },
    #[serde(rename = false)]
    Err { message: &'static str },
}

#[derive(Serialize, JsonSchema)]
pub struct RoomData<'a> {
    pub room_id: RoomId,
    pub current_turn: Side,
    pub players: Vec<&'a Player>,
    pub pieces: &'a BoardPieces,
}

#[derive(Serialize, JsonSchema)]
pub struct RoomSummary {
    pub id: RoomId,
    pub players: Vec<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct RoomList(pub Vec<RoomSummary>);
