use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::OnceLock,
};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use uuid::{uuid, Uuid};

use super::{GameSession, GameSessionBoardStyle, GameSessionConfig};

pub fn get_game_session_map() -> &'static RwLock<HashMap<Uuid, GameSession>> {
    static GAME_SESSION_MAP: OnceLock<RwLock<HashMap<Uuid, GameSession>>> = OnceLock::new();
    GAME_SESSION_MAP.get_or_init(|| {
        RwLock::new(
            #[cfg(not(debug_assertions))]
            {
                HashMap::new()
            },
            #[cfg(debug_assertions)]
            {
                let debug_room_id_1 = uuid!("00000000-0000-0000-0000-000000000000");
                let debug_room_id_2 = uuid!("00000000-0000-0000-0000-000000000001");
                HashMap::from([
                    (
                        debug_room_id_1,
                        GameSession::new(
                            debug_room_id_1,
                            GameSessionConfig {
                                team_player_limit: usize::MAX,
                                ..Default::default()
                            },
                        ),
                    ),
                    (
                        debug_room_id_2,
                        GameSession::new(
                            debug_room_id_2,
                            GameSessionConfig {
                                board_style: GameSessionBoardStyle::Chess,
                                team_player_limit: usize::MAX,
                                ..Default::default()
                            },
                        ),
                    ),
                ])
            },
        )
    })
}

pub struct ReadLockedSession<'a> {
    lock: RwLockReadGuard<'a, HashMap<Uuid, GameSession>>,
    room_id: Uuid,
}

impl Deref for ReadLockedSession<'_> {
    type Target = GameSession;

    fn deref(&self) -> &Self::Target {
        self.lock.get(&self.room_id).unwrap()
    }
}

pub struct WriteLockedSession<'a> {
    lock: RwLockWriteGuard<'a, HashMap<Uuid, GameSession>>,
    room_id: Uuid,
}

impl Deref for WriteLockedSession<'_> {
    type Target = GameSession;

    fn deref(&self) -> &Self::Target {
        self.lock.get(&self.room_id).unwrap()
    }
}

impl DerefMut for WriteLockedSession<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.lock.get_mut(&self.room_id).unwrap()
    }
}

/// # The returned value will panic if room doesn't exist.
/// Make sure room is created before this function.
#[inline(always)]
pub fn get_immutable_session(room_id: Uuid) -> ReadLockedSession<'static> {
    ReadLockedSession {
        lock: get_game_session_map().read(),
        room_id,
    }
}

/// # The returned value will panic if room doesn't exist.
/// Make sure room is created before this function.
#[inline(always)]
pub fn get_mutable_session(room_id: Uuid) -> WriteLockedSession<'static> {
    WriteLockedSession {
        lock: get_game_session_map().write(),
        room_id,
    }
}
