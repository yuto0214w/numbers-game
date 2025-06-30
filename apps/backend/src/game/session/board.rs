use std::{collections::HashMap, fmt};

use numbers_comm_types::{
    BOARD_SIZE, BoardPieces, Piece, Player, Position, Side,
    ws::{RegisterRoomEvent, RoomEvent},
};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::game::INITIAL_NUMBER;

use self::config::BoardConfig;

pub mod config;

#[derive(Debug, Clone)]
pub enum BoardOperationError {
    InvalidPosition,
    InvalidTurn,
    InvalidPiece,
    InvalidMove,
}

impl fmt::Display for BoardOperationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::InvalidPosition => write!(f, "INVALID_POSITION"),
            Self::InvalidTurn => write!(f, "INVALID_TURN"),
            Self::InvalidPiece => write!(f, "INVALID_PIECE"),
            Self::InvalidMove => write!(f, "INVALID_MOVE"),
        }
    }
}

type BoardOperationResult<T> = Result<T, BoardOperationError>;

#[inline(always)]
fn check_position(position: Position) -> BoardOperationResult<Position> {
    let (x, y) = position;
    if x > BOARD_SIZE || y > BOARD_SIZE {
        return Err(BoardOperationError::InvalidPosition);
    }
    Ok(position)
}

#[derive(Debug, Clone)]
pub struct Board {
    config: BoardConfig,
    room_queue: broadcast::Sender<RegisterRoomEvent>,
    players: HashMap<Uuid, Player>,
    pieces: BoardPieces,
    current_turn: Side,
    // move_log: Vec<(Side, Position)>,
}

impl Board {
    pub fn new(config: BoardConfig, room_queue: broadcast::Sender<RegisterRoomEvent>) -> Self {
        let mut pieces: BoardPieces = Default::default();
        for i in 0..3 {
            for j in 0..(BOARD_SIZE / 2) {
                let top_square = j * 2 + ((i + 1) % 2);
                let bottom_square = j * 2 + ((BOARD_SIZE - i) % 2);
                if top_square < BOARD_SIZE {
                    pieces[i][top_square] = Some(Piece {
                        side: Side::B,
                        number: INITIAL_NUMBER,
                    });
                }
                if bottom_square < BOARD_SIZE {
                    pieces[BOARD_SIZE - (i + 1)][bottom_square] = Some(Piece {
                        side: Side::A,
                        number: INITIAL_NUMBER,
                    });
                }
            }
        }
        Self {
            current_turn: config.first_side,
            config,
            room_queue,
            players: HashMap::new(),
            pieces,
            // move_log: Vec::new(),
        }
    }

    fn send_to_queue(&self, public_id: Uuid, event: RoomEvent) {
        self.room_queue
            .send(RegisterRoomEvent { public_id, event })
            .unwrap();
    }

    pub fn create_player(&mut self, side: Side, name: String) -> Option<(Uuid, Uuid)> {
        if self.players.values().filter(|p| p.side == side).count() == self.config.team_player_limit
        {
            return None;
        }
        let private_id = Uuid::new_v4();
        let public_id = Uuid::new_v4();
        self.players.insert(
            private_id,
            Player {
                public_id,
                name: name.to_owned(),
                side,
            },
        );
        self.send_to_queue(public_id, RoomEvent::PlayerJoin(side, name));
        Some((private_id, public_id))
    }

    pub fn remove_player(&mut self, private_id: Uuid) -> Option<Uuid> {
        let result = self
            .players
            .remove(&private_id)
            .map(|previous_data| previous_data.public_id);
        if let Some(public_id) = result {
            self.send_to_queue(public_id, RoomEvent::PlayerLeave);
        }
        result
    }

    pub fn contains_player(&self, private_id: Uuid) -> bool {
        self.players.contains_key(&private_id)
    }

    pub fn get_players(&self) -> Vec<&Player> {
        self.players.values().collect()
    }

    pub fn get_pieces(&self) -> &BoardPieces {
        &self.pieces
    }

    pub fn get_current_turn(&self) -> Side {
        self.current_turn
    }

    pub fn toggle_turn(&mut self) {
        self.current_turn = self.current_turn.reverse();
    }

    fn get_pieces_mut(&mut self) -> &mut BoardPieces {
        &mut self.pieces
    }

    fn get_movable_pieces(&self, side: Side) -> Vec<Position> {
        let mut movable_pieces = Vec::new();
        for (y, row) in self.pieces.iter().enumerate() {
            for (x, piece) in row.iter().enumerate() {
                if !piece.is_some_and(|p| p.side == side) {
                    continue;
                }
                let number = piece.unwrap().number;
                let left_up = x >= 2
                    && y >= 2
                    && self.pieces[y - 1][x - 1]
                        .is_some_and(|p| side != p.side && number > p.number)
                    && self.pieces[y - 2][x - 2].is_none();
                let left_down = x >= 2
                    && y <= BOARD_SIZE - 3
                    && self.pieces[y + 1][x - 1]
                        .is_some_and(|p| side != p.side && number > p.number)
                    && self.pieces[y + 2][x - 2].is_none();
                let right_up = x <= BOARD_SIZE - 3
                    && y >= 2
                    && self.pieces[y - 1][x + 1]
                        .is_some_and(|p| side != p.side && number > p.number)
                    && self.pieces[y - 2][x + 2].is_none();
                let right_down = x <= BOARD_SIZE - 3
                    && y <= BOARD_SIZE - 3
                    && self.pieces[y + 1][x + 1]
                        .is_some_and(|p| side != p.side && number > p.number)
                    && self.pieces[y + 2][x + 2].is_none();
                if left_up || left_down || right_up || right_down {
                    movable_pieces.push((x, y));
                }
            }
        }
        movable_pieces
    }

    pub fn move_piece(
        &mut self,
        private_id: Uuid,
        old_position: Position,
        new_position: Position,
    ) -> BoardOperationResult<()> {
        let result: BoardOperationResult<bool> = (|| {
            let ((old_x, old_y), (new_x, new_y)) =
                (check_position(old_position)?, check_position(new_position)?);
            let player_side = self.players.get(&private_id).unwrap().side;
            if player_side != self.get_current_turn() {
                return Err(BoardOperationError::InvalidTurn);
            }
            let moving_piece_number = match self.pieces[old_y][old_x] {
                Some(piece) if piece.side == player_side => piece.number,
                _ => return Err(BoardOperationError::InvalidPiece),
            };
            let destination_piece = self.pieces[new_y][new_x];
            let x_diff = old_x.abs_diff(new_x);
            let y_diff = old_y.abs_diff(new_y);
            // "相手の"駒を取る(相手の駒か確認する処理はbetween_pieceの定義あたりを参照のこと)
            if x_diff == 2 && y_diff == 2 {
                if destination_piece.is_some() {
                    return Err(BoardOperationError::InvalidMove);
                }
                let between_x = old_x + (old_x < new_x) as usize * 2 - 1;
                let between_y = old_y + (old_y < new_y) as usize * 2 - 1;
                let between_piece = self.pieces[between_y][between_x];
                if !between_piece
                    .is_some_and(|p| p.side != player_side && moving_piece_number > p.number)
                {
                    return Err(BoardOperationError::InvalidMove);
                }
                {
                    let pieces_mut = self.get_pieces_mut();
                    pieces_mut[new_y][new_x] = Some(Piece {
                        side: player_side,
                        number: ((moving_piece_number as f32) * (2.0 / 3.0)) as u8,
                    });
                    pieces_mut[between_y][between_x] = None;
                    pieces_mut[old_y][old_x] = None;
                }
                let do_toggle_turn = self.get_movable_pieces(player_side).is_empty();
                if do_toggle_turn {
                    self.toggle_turn();
                }
                Ok(do_toggle_turn)
            } else if x_diff == 1 && y_diff == 1 {
                if !self.get_movable_pieces(player_side).is_empty() {
                    return Err(BoardOperationError::InvalidMove);
                }
                match destination_piece {
                    // 自分の駒と数を共有する
                    Some(piece) if piece.side == player_side && moving_piece_number >= 2 => {
                        {
                            let pieces_mut = self.get_pieces_mut();
                            pieces_mut[new_y][new_x].as_mut().unwrap().number +=
                                moving_piece_number.div_ceil(2);
                            pieces_mut[old_y][old_x].as_mut().unwrap().number =
                                moving_piece_number / 2;
                        }
                        self.toggle_turn();
                        Ok(true)
                    }
                    // 駒を進める
                    None => {
                        {
                            let pieces_mut = self.get_pieces_mut();
                            pieces_mut[new_y][new_x] = pieces_mut[old_y][old_x];
                            pieces_mut[old_y][old_x] = None;
                        }
                        self.toggle_turn();
                        Ok(true)
                    }
                    _ => Err(BoardOperationError::InvalidMove),
                }
            } else {
                Err(BoardOperationError::InvalidMove)
            }
        })();
        match result {
            Ok(toggle_turn) => {
                self.send_to_queue(
                    self.players.get(&private_id).unwrap().public_id,
                    RoomEvent::MovePiece(toggle_turn, (old_position, new_position)),
                );
                Ok(())
            }
            Err(error) => Err(error),
        }
    }
}
