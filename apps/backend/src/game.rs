/// 部屋の作成とWebSocket通信への誘導
pub mod http;

/// 部屋情報の保持とロジックの定義
mod session;

/// game::sessionで定義された部屋情報の操作
mod ws;

const MINIMUM_SERVER_VERSION: usize = 1;

const MAX_TEAM_PLAYER_LIMIT: usize = 2;
const QUEUE_MESSAGE_LIMIT: usize = 16;
const INITIAL_NUMBER: u8 = 3;
