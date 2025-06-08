// game/のファイルについて、よく自分でもその内容や目的を忘れてしまうので、
// ファイルの内容とその歴史について、ここに備忘録を残しておきます。

pub mod http;
/// game::httpは、部屋・プレイヤーの作成、WebSocket通信への誘導を担っています。
//    一旦は、game::wsの事情により、このファイルがゲームの処理も請け負っていました。
//    その後、game::wsの説明で後述する理由により、
//    game::wsファイル内で受信->送信の処理ができるようになり、このファイルの機能は元通りになりました。
mod session;
/// game::sessionは、GameSessionやPlayerDataなどのゲームのセッションに関する情報を保持するstructを定義しています。
//    元々はgame::structsというファイルに定義されていて、いくつかに分断されていましたが、
//    ゲームを構成する重要なstructであることから一つのファイルとして独立しました。
//    その後、SelectPlayerという、「上のプレイヤー」と「下のプレイヤー」を、
//    「片方」と「その反対」としてアクセスできるようにするstructが作成されましたが、その関数のほとんどが使用されておらず
//    不必要と判断され、get_pieces_pair_mutだけを残して廃止されました。
mod structure;
/// game::structureは、主にgame::httpで使用するSerialize/Deserializeが可能なstructを定義しています。
//    こちらもgame::sessionと同じように、元々はgame::structsというファイルに定義されていました。
//    元々、「片方」と「その反対」や「自分」と「他人」のように、どちらかを主観とする考え方で設計を進めていたため、
//    ServerEvents(現RoomEvent)というenumに、OpponentActionとYourActionというvariantが存在しました。
//    その後、このファイルに限らず、「上」か「下」、または「一人」か「全員」のように、「主観」から「客観」になるよう再設計がなされました。
//    特にこうしたマルチプレイヤーのゲームを作る時、客観的な設計は重要なのかもしれません。
mod ws;
/// game::wsは、通信とゲームの処理の総括的な役割を担っています。
//    一旦は、「SplitSinkをスレッド間で共有することが難しい」という事情により、
//    プレイヤーの行動の処理(=ゲームの処理)をgame::httpが担っていました。
//    しかし、WebSocketMessagingというenumの誕生と、tokio::selectマクロの存在によって、
//    「受信した内容を元に送信する」ことが可能になり、このファイルの機能は元通りになりました。

const DEFAULT_BOARD_SIZE: usize = 8;
const DEFAULT_TEAM_PLAYER_LIMIT: usize = 2;
const INITIAL_NUMBER: u8 = 3;
const QUEUE_MESSAGE_LIMIT: usize = 16;
const PLAYER_INACTIVE_THRESHOLD: u64 = 30;
const PLAYER_KICK_THRESHOLD: u64 = 45;
