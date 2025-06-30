/// 部屋の作成とWebSocket通信への誘導
pub mod http;

/// プレイヤーの行動の処理・配信
mod ws;

/// 盤面に関する処理やデータの保持
mod session;

const MINIMUM_SERVER_VERSION: usize = 1;

const MAX_TEAM_PLAYER_LIMIT: usize = 2;
const QUEUE_MESSAGE_LIMIT: usize = 16;
const INITIAL_NUMBER: u8 = 3;
