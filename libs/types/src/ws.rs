use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Position, Side};

// client -> server

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "t", content = "c")]
pub enum PlayerActionWithoutAuth {
    #[serde(rename = 0)]
    JoinTeam(Side),
    #[serde(rename = 1)]
    Resume(Uuid),
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "t", content = "c")]
pub enum PlayerActionWithAuth {
    #[serde(rename = 2)]
    LeaveTeam,
    #[serde(rename = 3)]
    Move(Position, Position),
}

#[derive(Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum PlayerAction {
    WithoutAuth(PlayerActionWithoutAuth),
    WithAuth(PlayerActionWithAuth),
}

// server -> client

#[derive(Clone, Copy, Serialize, JsonSchema)]
#[serde(tag = "success")]
pub enum CreateUser {
    #[serde(rename = true)]
    Ok { private_id: Uuid, public_id: Uuid },
    #[serde(rename = false)]
    Err { message: &'static str },
}

#[derive(Clone, Copy, Serialize, JsonSchema)]
#[serde(tag = "t", content = "c")]
pub enum Responses {
    #[serde(rename = 101u8)]
    UserCreated(CreateUser),
    #[serde(rename = 102u8)]
    Authorized,
    #[serde(rename = 103u8)]
    ActionNotAccepted,
    #[serde(rename = 104u8)]
    SessionExpired,
    //
    #[serde(skip)]
    AuthorizedInternal(Uuid),
    #[serde(skip)]
    GotInvalidData,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[serde(tag = "t", content = "c")]
pub enum RoomEvent {
    #[serde(rename = 3)]
    MovePiece(bool, (Position, Position)),
    #[serde(rename = 4)]
    PlayerJoin(Side, String),
    #[serde(rename = 5)]
    PlayerLeave,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct RegisterRoomEvent {
    #[serde(rename = "i")]
    pub public_id: Uuid,
    #[serde(flatten)]
    pub event: RoomEvent,
}
