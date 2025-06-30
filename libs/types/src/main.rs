use std::{
    io::Write as _,
    path::Path,
    process::{Command, ExitCode, Stdio},
};

use numbers_comm_types::{
    Piece, Player, Side,
    http::{CreateRoom, HttpBoardConfig, RoomData, RoomList, RoomSummary, ServerInfo},
    ws::{
        CreateUser, PlayerAction, PlayerActionWithAuth, PlayerActionWithoutAuth, RegisterRoomEvent,
        Responses, RoomEvent,
    },
};
use schemars::{JsonSchema, schema_for};

#[derive(JsonSchema)]
#[allow(dead_code)]
enum ExportedTypes<'a> {
    Side(Side),
    Player(Player),
    Piece(Piece),

    // HTTP
    HttpBoardConfig(HttpBoardConfig),
    ServerInfo(ServerInfo),
    CreateRoom(CreateRoom),
    RoomData(RoomData<'a>),
    RoomSummary(RoomSummary),
    RoomList(RoomList),

    // WebSocket
    PlayerActionWithoutAuth(PlayerActionWithoutAuth),
    PlayerActionWithAuth(PlayerActionWithAuth),
    PlayerAction(PlayerAction),
    CreateUser(CreateUser),
    Responses(Responses),
    RoomEvent(RoomEvent),
    RegisterRoomEvent(RegisterRoomEvent),
}

#[cfg(windows)]
const NPX_COMMAND: &'static str = "npx.cmd";
#[cfg(not(windows))]
const NPX_COMMAND: &'static str = "npx";

const BANNER_COMMENT: &'static str =
    "/* DO NOT MODIFY -- Run the program to regenerate this file */";

fn main() -> ExitCode {
    let base_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/frontend");

    let schema = schema_for!(ExportedTypes);
    let schema_str = serde_json::to_string(&schema).unwrap();

    eprintln!("Generating...");

    let mut json2ts = Command::new(NPX_COMMAND)
        .current_dir(base_path)
        .args([
            "json2ts",
            "-o",
            "src/lib/types.ts",
            "--additionalProperties",
            "false",
            "--bannerComment",
            BANNER_COMMENT,
        ])
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    json2ts
        .stdin
        .take()
        .unwrap()
        .write_all(schema_str.as_bytes())
        .unwrap();

    if !json2ts.wait().unwrap().success() {
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
