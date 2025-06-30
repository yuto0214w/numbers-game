use std::{collections::HashMap, sync::LazyLock};

use numbers_comm_types::RoomId;
use parking_lot::RwLock;

use super::GameSession;

pub static GAME_SESSION_MAP: LazyLock<RwLock<HashMap<RoomId, GameSession>>> = LazyLock::new(|| {
    RwLock::new(
        #[cfg(not(debug_assertions))]
        {
            HashMap::new()
        },
        #[cfg(debug_assertions)]
        {
            let debug_room_id: RoomId = "testroom".try_into().unwrap();
            HashMap::from([(
                debug_room_id,
                GameSession::new(super::board::config::BoardConfig {
                    team_player_limit: usize::MAX,
                    first_side: numbers_comm_types::Side::A,
                }),
            )])
        },
    )
});

pub struct GameSessionLock {
    room_id: RoomId,
}

impl GameSessionLock {
    pub fn new(room_id: RoomId) -> Self {
        Self { room_id }
    }

    pub fn with_read<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&GameSession) -> T,
    {
        let lock = GAME_SESSION_MAP.read();
        let session = lock.get(&self.room_id).unwrap();
        f(session)
    }

    pub fn with_write<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut GameSession) -> T,
    {
        let mut lock = GAME_SESSION_MAP.write();
        let session = lock.get_mut(&self.room_id).unwrap();
        f(session)
    }
}
