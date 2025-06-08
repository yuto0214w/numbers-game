export type Position = [number, number];
export type PlayerData = { name: string; selecting_piece: Position | null; is_inactive: boolean };
type PlayerDataWithId = PlayerData & { public_id: string };
export type PieceData = { position: Position; number: number };
export type Side = "top" | "bottom";
export type RawRoomData = {
  room_id: string;
  board_size: number;
  current_turn: Side;
  top_players: PlayerDataWithId[];
  top_pieces: PieceData[];
  bottom_players: PlayerDataWithId[];
  bottom_pieces: PieceData[];
};
export type CreateUserData =
  | { success: true; side: Side; private_id: string; public_id: string; name: string }
  | { success: false; message: string };
type PlayerAction = { t: 1; c: Position } | { t: 2; c: [Position, Position] };
type PublicEvent = (PlayerAction | { t: 3; c: string } | { t: 4; c: string } | { t: 5 } | { t: 6 }) & { i: string };
type PrivateEvent = { t: 100 } | { t: 101; c: PlayerAction } | { t: 102 } | { t: 103 } | { t: 104 };
export type ReceivedEvent = PublicEvent | PrivateEvent;
export type CanvasComponent =
  | { type: 1; color: CanvasFillStrokeStyles["fillStyle"]; x: number; y: number; w: number; h: number }
  | {
      type: 2;
      color: CanvasFillStrokeStyles["fillStyle"];
      text: string;
      x: number;
      y: number;
      font: CanvasTextDrawingStyles["font"];
      textAlign: CanvasTextAlign;
      textBaseline: CanvasTextBaseline;
    }
  | {
      type: 3;
      color: CanvasFillStrokeStyles["strokeStyle"];
      startX: number;
      startY: number;
      endX: number;
      endY: number;
      lineWidth: CanvasPathDrawingStyles["lineWidth"];
    };
export type IterResult = { value: string; done: false } | { value: undefined; done: true };
