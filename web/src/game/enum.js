export const ComponentType = Object.freeze({ Rect: 1, Text: 2, Line: 3 });
export const MessageType = Object.freeze({
  SelectPiece: 1,
  MovePiece: 2,
  TopPlayerJoin: 3,
  BottomPlayerJoin: 4,
  TopPlayerLeave: 5,
  BottomPlayerLeave: 6,
  HeartbeatAck: 100,
  NotAccepted: 101,
  SessionExpired: 102,
  GotBinary: 103,
  GotInvalidData: 104,
});
