/* DO NOT MODIFY -- Run the program to regenerate this file */

export type ExportedTypes =
  | {
      Side: Side;
    }
  | {
      Player: Player;
    }
  | {
      Piece: Piece;
    }
  | {
      HttpBoardConfig: HttpBoardConfig;
    }
  | {
      ServerInfo: ServerInfo;
    }
  | {
      CreateRoom: CreateRoom;
    }
  | {
      RoomData: RoomData;
    }
  | {
      RoomSummary: RoomSummary;
    }
  | {
      RoomList: RoomList;
    }
  | {
      PlayerActionWithoutAuth: PlayerActionWithoutAuth;
    }
  | {
      PlayerActionWithAuth: PlayerActionWithAuth;
    }
  | {
      PlayerAction: PlayerAction;
    }
  | {
      CreateUser: CreateUser;
    }
  | {
      Responses: Responses;
    }
  | {
      RoomEvent: RoomEvent;
    }
  | {
      RegisterRoomEvent: RegisterRoomEvent;
    };
export type Side = "a" | "b";
export type CreateRoom =
  | {
      room_id: string;
      success: true;
    }
  | {
      message: string;
      success: false;
    };
export type RoomList = RoomSummary[];
export type PlayerActionWithoutAuth =
  | {
      t: 0;
      c: Side;
    }
  | {
      t: 1;
      c: string;
    };
export type PlayerActionWithAuth =
  | {
      t: 2;
    }
  | {
      t: 3;
      /**
       * @minItems 2
       * @maxItems 2
       */
      c: [[number, number], [number, number]];
    };
export type PlayerAction = PlayerActionWithoutAuth | PlayerActionWithAuth;
export type CreateUser =
  | {
      private_id: string;
      public_id: string;
      success: true;
    }
  | {
      message: string;
      success: false;
    };
export type Responses =
  | {
      t: 101;
      c: CreateUser;
    }
  | {
      t: 102;
    }
  | {
      t: 103;
    }
  | {
      t: 104;
    };
export type RoomEvent =
  | {
      t: 3;
      /**
       * @minItems 2
       * @maxItems 2
       */
      c: [boolean, [[number, number], [number, number]]];
    }
  | {
      t: 4;
      /**
       * @minItems 2
       * @maxItems 2
       */
      c: [Side, string];
    }
  | {
      t: 5;
    };
export type RegisterRoomEvent = {
  i: string;
} & RegisterRoomEvent1;
export type RegisterRoomEvent1 =
  | {
      t: 3;
      /**
       * @minItems 2
       * @maxItems 2
       */
      c: [boolean, [[number, number], [number, number]]];
    }
  | {
      t: 4;
      /**
       * @minItems 2
       * @maxItems 2
       */
      c: [Side, string];
    }
  | {
      t: 5;
    };

export interface Player {
  public_id: string;
  name: string;
  side: Side;
}
export interface Piece {
  side: Side;
  number: number;
}
export interface HttpBoardConfig {
  team_player_limit?: number | null;
  first_side?: Side | null;
}
export interface ServerInfo {
  min_version: number;
}
export interface RoomData {
  room_id: string;
  current_turn: Side;
  players: Player[];
  /**
   * @minItems 8
   * @maxItems 8
   */
  pieces: [
    [Piece | null, Piece | null, Piece | null, Piece | null, Piece | null, Piece | null, Piece | null, Piece | null],
    [Piece | null, Piece | null, Piece | null, Piece | null, Piece | null, Piece | null, Piece | null, Piece | null],
    [Piece | null, Piece | null, Piece | null, Piece | null, Piece | null, Piece | null, Piece | null, Piece | null],
    [Piece | null, Piece | null, Piece | null, Piece | null, Piece | null, Piece | null, Piece | null, Piece | null],
    [Piece | null, Piece | null, Piece | null, Piece | null, Piece | null, Piece | null, Piece | null, Piece | null],
    [Piece | null, Piece | null, Piece | null, Piece | null, Piece | null, Piece | null, Piece | null, Piece | null],
    [Piece | null, Piece | null, Piece | null, Piece | null, Piece | null, Piece | null, Piece | null, Piece | null],
    [Piece | null, Piece | null, Piece | null, Piece | null, Piece | null, Piece | null, Piece | null, Piece | null]
  ];
}
export interface RoomSummary {
  id: string;
  players: string[];
}
