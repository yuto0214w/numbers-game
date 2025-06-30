use numbers_comm_types::ws::RegisterRoomEvent;
use tokio::sync::broadcast;

use self::board::{Board, config::BoardConfig};

use super::QUEUE_MESSAGE_LIMIT;

pub mod board;
pub mod map;

#[derive(Debug)]
pub struct GameSession {
    board: Board,
    room_queue: broadcast::Sender<RegisterRoomEvent>,
    _room_queue_recv: broadcast::Receiver<RegisterRoomEvent>,
}

impl GameSession {
    pub fn new(config: BoardConfig) -> Self {
        let (room_queue, _room_queue_recv) = broadcast::channel(QUEUE_MESSAGE_LIMIT);
        Self {
            board: Board::new(config, room_queue.clone()),
            room_queue,
            _room_queue_recv,
        }
    }

    pub fn subscribe_queue(&self) -> broadcast::Receiver<RegisterRoomEvent> {
        self.room_queue.subscribe()
    }

    pub fn get_board(&self) -> &Board {
        &self.board
    }

    pub fn get_board_mut(&mut self) -> &mut Board {
        &mut self.board
    }
}
