use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use serde::Serialize;
use tokio::{
    sync::{broadcast, oneshot},
    time::interval,
};
use uuid::Uuid;

use crate::handler::game::{PLAYER_INACTIVE_THRESHOLD, PLAYER_KICK_THRESHOLD};

use super::{
    structure::{RoomEvent, RoomEventWithId},
    DEFAULT_BOARD_SIZE, DEFAULT_TEAM_PLAYER_LIMIT, INITIAL_NUMBER, QUEUE_MESSAGE_LIMIT,
};

pub mod map;

pub type Position = (usize, usize);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    Top,
    Bottom,
}

impl Serialize for Side {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match *self {
            Self::Top => serializer.serialize_str("top"),
            Self::Bottom => serializer.serialize_str("bottom"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerData {
    pub public_id: Uuid,
    pub name: String,
    pub selecting_piece: Option<Position>,
    pub is_inactive: bool,
    #[serde(skip)]
    pub side: Side,
    #[serde(skip)]
    pub last_heartbeat: Instant,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct PieceData {
    pub side: Side,
    pub number: u8,
}

#[derive(Debug)]
struct HeartbeatTimer {
    tx: Option<oneshot::Sender<()>>,
}

impl HeartbeatTimer {
    fn new(room_id: Uuid) -> Self {
        let (tx, mut rx) = oneshot::channel();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let mut session = map::get_mutable_session(room_id);
                        for private_id in session.get_player_ids() {
                            let player = session.get_player_mut(private_id).unwrap();
                            if player.last_heartbeat.elapsed().as_secs() > PLAYER_KICK_THRESHOLD {
                                session.remove_player(private_id);
                            } else if player.last_heartbeat.elapsed().as_secs() > PLAYER_INACTIVE_THRESHOLD {
                                player.is_inactive = true;
                            } else if player.is_inactive {
                                player.is_inactive = false;
                            }
                        }
                    }
                    _ = &mut rx => break,
                }
            }
        });
        Self { tx: Some(tx) }
    }
}

impl Drop for HeartbeatTimer {
    fn drop(&mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(());
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum GameSessionBoardStyle {
    // x x x x x
    //  x x x x
    // x x x x x
    #[default]
    Checker,
    // xxxxxxxxx
    // xxxxxxxxx
    Chess,
}

#[derive(Debug, Clone, Copy)]
pub struct GameSessionConfig {
    pub board_size: usize,
    pub board_style: GameSessionBoardStyle,
    pub team_player_limit: usize,
}

impl Default for GameSessionConfig {
    fn default() -> Self {
        Self {
            board_size: DEFAULT_BOARD_SIZE,
            board_style: Default::default(),
            team_player_limit: DEFAULT_TEAM_PLAYER_LIMIT,
        }
    }
}

#[derive(Debug)]
pub struct GameSession {
    config: GameSessionConfig,
    room_queue: broadcast::Sender<RoomEventWithId>,
    players: HashMap<Uuid, PlayerData>,
    pieces: Vec<Vec<Option<PieceData>>>,
    current_turn: Side,
    // このstructがdropした際にHeartbeatTimerをdropするため
    #[allow(dead_code)]
    heartbeat_timer: HeartbeatTimer,
}

pub struct ActionRejectedMarker;

impl GameSession {
    pub fn new(room_id: Uuid, config: GameSessionConfig) -> Self {
        if config.board_size < 7 {
            panic!("board size must be 7 or above");
        }
        let mut pieces = vec![vec![None; config.board_size]; config.board_size];
        match config.board_style {
            GameSessionBoardStyle::Checker => {
                for i in 0..3 {
                    for j in 0..(config.board_size / 2) {
                        let top_square = j * 2 + ((config.board_size - i) % 2);
                        let bottom_square = j * 2 + ((i + 1) % 2);
                        if top_square < config.board_size {
                            pieces[i][top_square] = Some(PieceData {
                                side: Side::Top,
                                number: INITIAL_NUMBER,
                            });
                        }
                        if bottom_square < config.board_size {
                            pieces[config.board_size - (i + 1)][bottom_square] = Some(PieceData {
                                side: Side::Bottom,
                                number: INITIAL_NUMBER,
                            });
                        }
                    }
                }
            }
            GameSessionBoardStyle::Chess => {
                for i in 0..2 {
                    for j in 0..config.board_size {
                        pieces[i][j] = Some(PieceData {
                            side: Side::Top,
                            number: INITIAL_NUMBER,
                        });
                        pieces[config.board_size - (i + 1)][j] = Some(PieceData {
                            side: Side::Bottom,
                            number: INITIAL_NUMBER,
                        });
                    }
                }
            }
        }
        Self {
            config,
            room_queue: broadcast::channel(QUEUE_MESSAGE_LIMIT).0,
            players: HashMap::new(),
            pieces,
            current_turn: Side::Bottom,
            heartbeat_timer: HeartbeatTimer::new(room_id),
        }
    }

    pub fn get_board_size(&self) -> usize {
        self.config.board_size
    }

    pub fn get_queue_sender(&self) -> broadcast::Sender<RoomEventWithId> {
        self.room_queue.clone()
    }

    pub fn get_player_ids(&self) -> Vec<Uuid> {
        self.players.keys().copied().collect()
    }

    pub fn get_player_data(&self, side: Side) -> Vec<PlayerData> {
        self.players
            .values()
            .filter(|data| data.side == side)
            .cloned()
            .collect()
    }

    pub fn get_pieces(&self) -> &Vec<Vec<Option<PieceData>>> {
        &self.pieces
    }

    fn get_pieces_mut(&mut self) -> &mut Vec<Vec<Option<PieceData>>> {
        &mut self.pieces
    }

    pub fn create_player<T>(&mut self, side: Side, name: T) -> Option<Uuid>
    where
        T: Into<String>,
    {
        if self.get_player_data(side).len() >= self.config.team_player_limit {
            return None;
        }
        let private_id = Uuid::new_v4();
        let public_id = Uuid::new_v4();
        let name = name.into();
        self.players.insert(
            private_id,
            PlayerData {
                public_id,
                name: name.to_owned(),
                selecting_piece: None,
                is_inactive: false,
                last_heartbeat: Instant::now(),
                side,
            },
        );
        let _ = self.room_queue.send(RoomEventWithId {
            public_id,
            event: match side {
                Side::Top => RoomEvent::TopPlayerJoin(name),
                Side::Bottom => RoomEvent::BottomPlayerJoin(name),
            },
        });
        Some(private_id)
    }

    pub fn remove_player(&mut self, private_id: Uuid) -> bool {
        match self.players.remove(&private_id) {
            Some(previous_data) => {
                let _ = self.room_queue.send(RoomEventWithId {
                    public_id: previous_data.public_id,
                    event: match previous_data.side {
                        Side::Top => RoomEvent::TopPlayerLeave,
                        Side::Bottom => RoomEvent::BottomPlayerLeave,
                    },
                });
                true
            }
            None => false,
        }
    }

    pub fn contains_player(&self, private_id: Uuid) -> bool {
        self.players.contains_key(&private_id)
    }

    fn get_player_mut(&mut self, private_id: Uuid) -> Option<&mut PlayerData> {
        self.players.get_mut(&private_id)
    }

    /// # This function will panic if ID is invalid.
    /// Double-check the argument.
    pub fn get_public_id(&self, private_id: Uuid) -> Uuid {
        self.players.get(&private_id).unwrap().public_id
    }

    /// # This function will panic if ID is invalid.
    /// Double-check the argument.
    pub fn update_heartbeat(&mut self, private_id: Uuid) {
        self.get_player_mut(private_id).unwrap().last_heartbeat = Instant::now();
    }

    pub fn get_current_turn(&self) -> Side {
        self.current_turn
    }

    fn toggle_turn(&mut self) {
        self.current_turn = match self.current_turn {
            Side::Top => Side::Bottom,
            Side::Bottom => Side::Top,
        };
    }

    /// # This function will panic if ID is invalid.
    /// Double-check the argument.
    pub fn select_piece(
        &mut self,
        private_id: Uuid,
        position: Position,
    ) -> Result<(), ActionRejectedMarker> {
        let board_size = self.get_board_size();
        let (x, y) = position;
        if x > board_size || y > board_size {
            return Err(ActionRejectedMarker);
        }
        self.get_player_mut(private_id).unwrap().selecting_piece = Some(position);
        self.get_queue_sender()
            .send(RoomEventWithId {
                public_id: self.get_public_id(private_id),
                event: RoomEvent::SelectPiece(position),
            })
            .unwrap();
        Ok(())
    }

    fn is_any_piece_still_movable(&self, side: Side) -> bool {
        let board_size = self.get_board_size();
        for (y, row) in self.pieces.iter().enumerate() {
            for (x, piece) in row.iter().enumerate() {
                if !piece.is_some_and(|p| p.side == side) {
                    continue;
                }
                let number = piece.unwrap().number;
                // x
                //   o
                //
                if x >= 2
                    && y >= 2
                    && self.pieces[y - 1][x - 1]
                        .is_some_and(|p| side != p.side && number > p.number)
                    && self.pieces[y - 2][x - 2].is_none()
                {
                    return true;
                }
                //     x
                //   o
                //
                if x <= board_size - 3
                    && y >= 2
                    && self.pieces[y - 1][x + 1]
                        .is_some_and(|p| side != p.side && number > p.number)
                    && self.pieces[y - 2][x + 2].is_none()
                {
                    return true;
                }
                //
                //   o
                // x
                if x >= 2
                    && y <= board_size - 3
                    && self.pieces[y + 1][x - 1]
                        .is_some_and(|p| side != p.side && number > p.number)
                    && self.pieces[y + 2][x - 2].is_none()
                {
                    return true;
                }
                //
                //   o
                //     x
                if x <= board_size - 3
                    && y <= board_size - 3
                    && self.pieces[y + 1][x + 1]
                        .is_some_and(|p| side != p.side && number > p.number)
                    && self.pieces[y + 2][x + 2].is_none()
                {
                    return true;
                }
            }
        }
        false
    }

    /// # This function will panic if ID is invalid.
    /// Double-check the argument.
    pub fn move_piece(
        &mut self,
        private_id: Uuid,
        old_position: Position,
        new_position: Position,
    ) -> Result<(), ActionRejectedMarker> {
        let board_size = self.get_board_size();
        let ((old_x, old_y), (new_x, new_y)) = (old_position, new_position);
        if new_x > board_size || new_y > board_size {
            return Err(ActionRejectedMarker);
        }
        let player_side = self.players.get(&private_id).unwrap().side;
        if player_side != self.get_current_turn() {
            return Err(ActionRejectedMarker);
        }
        let moving_piece_number = match self.pieces[old_y][old_x] {
            Some(piece) if piece.side == player_side => piece.number,
            _ => return Err(ActionRejectedMarker),
        };
        let destination_piece = self.pieces[new_y][new_x];
        let x_diff = old_x.abs_diff(new_x);
        let y_diff = old_y.abs_diff(new_y);
        // 敵の駒を取る
        if x_diff == 2 && y_diff == 2 {
            if destination_piece.is_some() {
                return Err(ActionRejectedMarker);
            }
            let between_x = {
                if old_x > new_x {
                    old_x - 1
                } else {
                    old_x + 1
                }
            };
            let between_y = {
                if old_y > new_y {
                    old_y - 1
                } else {
                    old_y + 1
                }
            };
            let between_piece = self.pieces[between_y][between_x];
            if !between_piece.is_some_and(|p| moving_piece_number > p.number) {
                return Err(ActionRejectedMarker);
            }
            {
                let pieces_mut = self.get_pieces_mut();
                pieces_mut[new_y][new_x] = Some(PieceData {
                    side: player_side,
                    number: ((moving_piece_number as f32) * (2.0 / 3.0)) as u8,
                });
                pieces_mut[between_y][between_x] = None;
                pieces_mut[old_y][old_x] = None;
            }
            self.get_queue_sender()
                .send(RoomEventWithId {
                    public_id: self.get_public_id(private_id),
                    event: RoomEvent::MovePiece(old_position, new_position),
                })
                .unwrap();
            if !self.is_any_piece_still_movable(player_side) {
                self.toggle_turn();
            }
            Ok(())
        } else if x_diff == 1 && y_diff == 1 {
            if self.is_any_piece_still_movable(player_side) {
                return Err(ActionRejectedMarker);
            }
            match destination_piece {
                Some(piece) if piece.side == player_side && moving_piece_number > 2 => {
                    {
                        let pieces_mut = self.get_pieces_mut();
                        pieces_mut[new_y][new_x].as_mut().unwrap().number +=
                            moving_piece_number.div_ceil(2);
                        pieces_mut[old_y][old_x].as_mut().unwrap().number = moving_piece_number / 2;
                    }
                    self.get_queue_sender()
                        .send(RoomEventWithId {
                            public_id: self.get_public_id(private_id),
                            event: RoomEvent::MovePiece(old_position, new_position),
                        })
                        .unwrap();
                    self.toggle_turn();
                    Ok(())
                }
                None => {
                    {
                        let pieces_mut = self.get_pieces_mut();
                        pieces_mut[new_y][new_x] = pieces_mut[old_y][old_x];
                        pieces_mut[old_y][old_x] = None;
                    }
                    self.get_queue_sender()
                        .send(RoomEventWithId {
                            public_id: self.get_public_id(private_id),
                            event: RoomEvent::MovePiece(old_position, new_position),
                        })
                        .unwrap();
                    self.toggle_turn();
                    Ok(())
                }
                _ => Err(ActionRejectedMarker),
            }
        } else {
            Err(ActionRejectedMarker)
        }
    }
}
