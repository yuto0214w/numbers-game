// @ts-check

import { createCheckerConnection } from "./game/connection";
import { initDraw } from "./game/draw";
import { ComponentType, MessageType } from "./game/enum";

/**
 * @typedef {import("../types").PlayerData} PlayerData
 * @typedef {import("../types").RawRoomData} RawRoomData
 * @typedef {import("../types").CreateUserData} CreateUserData
 * @typedef {import("../types").ReceivedEvent} ReceivedEvent
 * @typedef {import("../types").Position} Position
 * @typedef {import("../types").CanvasComponent} CanvasComponent
 * @typedef {import("../types").Side} Side
 */

window.addEventListener("DOMContentLoaded", async () => {
  const normalizedPath = location.pathname.replace(/\/$/, "");
  const players = {
    /** @type {Map<string, PlayerData>} */
    top: new Map(),
    /** @type {Map<string, PlayerData>} */
    bottom: new Map(),
  };
  let { roomId, boardSize, currentTurn, topPieces, bottomPieces } = await (async () => {
    const res = await fetch(normalizedPath + "/room_data");
    /** @type {RawRoomData} */
    const data = await res.json();
    data.top_players.forEach(player =>
      players.top.set(player.public_id, {
        name: player.name,
        selecting_piece: player.selecting_piece,
        is_inactive: player.is_inactive,
      })
    );
    data.bottom_players.forEach(player =>
      players.bottom.set(player.public_id, {
        name: player.name,
        selecting_piece: player.selecting_piece,
        is_inactive: player.is_inactive,
      })
    );
    return {
      roomId: data.room_id,
      boardSize: data.board_size,
      currentTurn: data.current_turn,
      topPieces: data.top_pieces,
      bottomPieces: data.bottom_pieces,
    };
  })();
  const canvas = document.createElement("canvas");
  document.body.appendChild(canvas);
  const drawObj = initDraw({ canvas, aspect: 4 / 3 });
  const calculatedValues = {
    get offsetX() {
      return canvas.width / 2 - this.offsetY * (boardSize / 2);
    },
    get offsetY() {
      return canvas.height / (boardSize + 2);
    },
    get width() {
      return this.offsetX + this.offsetY * boardSize;
    },
    get height() {
      return this.offsetY + this.offsetY * boardSize;
    },
    get lineInterval() {
      return this.offsetY;
    },
    get halfLineInterval() {
      return this.lineInterval / 2;
    },
  };
  {
    /**
     * @param {CanvasComponent[]} components
     */
    function prepareToDrawCanvasBackground(components) {
      components.push({
        type: ComponentType.Rect,
        color: "#ffeb3b",
        x: 0,
        y: 0,
        w: canvas.width,
        h: canvas.height,
      });
    }
    /**
     * @param {CanvasComponent[]} components
     */
    function prepareToDrawBoardBackground(components) {
      components.push({
        type: ComponentType.Rect,
        color: "#ffee58",
        x: calculatedValues.offsetX,
        y: calculatedValues.offsetY,
        w: calculatedValues.lineInterval * boardSize,
        h: calculatedValues.lineInterval * boardSize,
      });
    }
    /**
     * @param {CanvasComponent[]} components
     */
    function prepareToDrawBoardTiles(components) {
      for (let i = 0; i < boardSize; i++) {
        for (let j = 0; j < boardSize / 2; j++) {
          const square = j * 2 + ((i + 1) % 2);
          if (square < boardSize) {
            components.push({
              type: ComponentType.Rect,
              color: "#fff",
              x: calculatedValues.offsetX + calculatedValues.lineInterval * square,
              y: calculatedValues.offsetY + calculatedValues.lineInterval * i,
              w: calculatedValues.lineInterval,
              h: calculatedValues.lineInterval,
            });
          }
        }
      }
      const selectedSquares = Object.values(players).flatMap(playerMap =>
        [...playerMap.values()]
          .filter(player => player.selecting_piece !== null)
          .map(player => /** @type {Position} */ (player.selecting_piece))
      );
      selectedSquares.forEach(square => {
        components.push({
          type: ComponentType.Rect,
          color: "#fff",
          x: calculatedValues.offsetX + calculatedValues.lineInterval * square[0],
          y: calculatedValues.offsetY + calculatedValues.lineInterval * square[1],
          w: calculatedValues.lineInterval,
          h: calculatedValues.lineInterval,
        });
      });
    }
    /**
     * @param {CanvasComponent[]} components
     */
    function prepareToDrawSelectingPieces(components) {
      [...players.top.values()].forEach(player => {
        if (player.selecting_piece) {
          components.push({
            type: ComponentType.Rect,
            color: "#bc51007f",
            x: calculatedValues.offsetX + calculatedValues.lineInterval * player.selecting_piece[0],
            y: calculatedValues.offsetY + calculatedValues.lineInterval * player.selecting_piece[1],
            w: calculatedValues.lineInterval,
            h: calculatedValues.lineInterval,
          });
        }
      });
      [...players.bottom.values()].forEach(player => {
        if (player.selecting_piece) {
          components.push({
            type: ComponentType.Rect,
            color: "#5869ff7f",
            x: calculatedValues.offsetX + calculatedValues.lineInterval * player.selecting_piece[0],
            y: calculatedValues.offsetY + calculatedValues.lineInterval * player.selecting_piece[1],
            w: calculatedValues.lineInterval,
            h: calculatedValues.lineInterval,
          });
        }
      });
    }
    /**
     * @param {CanvasComponent[]} components
     */
    function prepareToDrawNumbers(components) {
      topPieces.forEach(piece => {
        components.push({
          type: ComponentType.Text,
          color: "#bc5100",
          text: piece.number.toString(),
          x:
            calculatedValues.offsetX +
            calculatedValues.lineInterval * piece.position[0] +
            calculatedValues.halfLineInterval,
          y:
            calculatedValues.offsetY +
            calculatedValues.lineInterval * piece.position[1] +
            calculatedValues.halfLineInterval,
          font: "2em sans-serif",
          textAlign: "center",
          textBaseline: "middle",
        });
      });
      bottomPieces.forEach(piece => {
        components.push({
          type: ComponentType.Text,
          color: "#5869ff",
          text: piece.number.toString(),
          x:
            calculatedValues.offsetX +
            calculatedValues.lineInterval * piece.position[0] +
            calculatedValues.halfLineInterval,
          y:
            calculatedValues.offsetY +
            calculatedValues.lineInterval * piece.position[1] +
            calculatedValues.halfLineInterval,
          font: "2em sans-serif",
          textAlign: "center",
          textBaseline: "middle",
        });
      });
    }
    /**
     * @param {CanvasComponent[]} components
     */
    function prepareToDrawOutline(components) {
      for (let i = 0; i <= boardSize; i++) {
        const y = calculatedValues.offsetY + calculatedValues.lineInterval * i;
        components.push({
          type: ComponentType.Line,
          color: "#000",
          startX: calculatedValues.offsetX,
          startY: y,
          endX: calculatedValues.width,
          endY: y,
          lineWidth: 2,
        });
        const x = calculatedValues.offsetX + calculatedValues.lineInterval * i;
        components.push({
          type: ComponentType.Line,
          color: "#000",
          startX: x,
          startY: calculatedValues.offsetY,
          endX: x,
          endY: calculatedValues.height,
          lineWidth: 2,
        });
      }
    }
    drawObj.addComponentProducer(
      prepareToDrawCanvasBackground,
      prepareToDrawBoardBackground,
      prepareToDrawBoardTiles,
      prepareToDrawSelectingPieces,
      prepareToDrawNumbers,
      prepareToDrawOutline
    );
  }
  const { sender, receiver, heartbeat } = createCheckerConnection();
  let isGuest = false,
    isAuthorized = false;
  {
    /**
     * @param {CanvasComponent[]} components
     */
    function prepareToDrawJoinButtons(components) {
      if (!isGuest && !isAuthorized) {
        components.push({
          type: ComponentType.Rect,
          color: "#0000007f",
          x: 0,
          y: 0,
          w: canvas.width,
          h: canvas.height,
        });
        const buttonWidth = calculatedValues.lineInterval * 4;
        components.push({
          type: ComponentType.Rect,
          color: "#ff0000",
          x: canvas.width / 2 - buttonWidth / 2,
          y: canvas.height / 2 - calculatedValues.lineInterval * 2,
          w: buttonWidth,
          h: calculatedValues.lineInterval,
        });
        components.push({
          type: ComponentType.Text,
          color: "#ffffff",
          text: "Join",
          x: canvas.width / 2,
          y: canvas.height / 2 - calculatedValues.lineInterval * 1.5,
          font: "2em sans-serif",
          textAlign: "center",
          textBaseline: "middle",
        });
        components.push({
          type: ComponentType.Rect,
          color: "#7f7f7f",
          x: canvas.width / 2 - buttonWidth / 2,
          y: canvas.height / 2 - calculatedValues.lineInterval * 0.5,
          w: buttonWidth,
          h: calculatedValues.lineInterval,
        });
        components.push({
          type: ComponentType.Text,
          color: "#ffffff",
          text: "Spectate",
          x: canvas.width / 2,
          y: canvas.height / 2,
          font: "2em sans-serif",
          textAlign: "center",
          textBaseline: "middle",
        });
        components.push({
          type: ComponentType.Rect,
          color: "#0000ff",
          x: canvas.width / 2 - buttonWidth / 2,
          y: canvas.height / 2 + calculatedValues.lineInterval,
          w: buttonWidth,
          h: calculatedValues.lineInterval,
        });
        components.push({
          type: ComponentType.Text,
          color: "#ffffff",
          text: "Join",
          x: canvas.width / 2,
          y: canvas.height / 2 + calculatedValues.lineInterval * 1.5,
          font: "2em sans-serif",
          textAlign: "center",
          textBaseline: "middle",
        });
      }
    }
    drawObj.addComponentProducer(prepareToDrawJoinButtons);
  }
  const authInfo = sessionStorage.getItem(roomId);
  /** @type {string} */
  let privateId,
    /** @type {string} */
    publicId,
    /** @type {Side} */
    playerSide;
  if (authInfo) {
    ({ privateId, publicId, playerSide } = JSON.parse(authInfo));
    sender.authorize(privateId);
  }
  function redraw() {
    drawObj.adjustCanvas();
    drawObj.drawAll();
  }
  window.addEventListener("resize", redraw);
  redraw();
  let previousX = -1,
    previousY = -1,
    isMovePieceMode = false;
  drawObj.addComponentProducer(components => {
    if (isMovePieceMode) {
      components.push({
        type: ComponentType.Text,
        color: "#000",
        text: "Selecting",
        x: canvas.width / 2,
        y: canvas.height / 2,
        font: "2em sans-serif",
        textAlign: "center",
        textBaseline: "middle",
      });
    }
    /** @type {string} */
    let text;
    if (isGuest) {
      text = currentTurn.slice(0, 1).toUpperCase() + currentTurn.slice(1) + " player's turn";
    } else {
      text = currentTurn === playerSide ? "Your turn" : "Enemy turn";
    }
    components.push({
      type: ComponentType.Text,
      color: "#000",
      text,
      x: canvas.width / 2,
      y: calculatedValues.offsetY / 2,
      font: "2em sans-serif",
      textAlign: "center",
      textBaseline: "middle",
    });
  });
  window.addEventListener("mouseup", async function f(e) {
    if (!isAuthorized) {
      let side;
      {
        const buttonWidth = calculatedValues.lineInterval * 4;
        if (e.offsetX < canvas.width / 2 - buttonWidth / 2 || e.offsetX > canvas.width / 2 + buttonWidth / 2) {
          return;
        }
        if (
          e.offsetY > canvas.height / 2 - calculatedValues.lineInterval * 2 &&
          e.offsetY < canvas.height / 2 - calculatedValues.lineInterval
        ) {
          side = "top";
        } else if (
          e.offsetY > canvas.height / 2 - calculatedValues.lineInterval * 0.5 &&
          e.offsetY < canvas.height / 2 + calculatedValues.lineInterval * 0.5
        ) {
          isGuest = true;
          window.removeEventListener("mouseup", f);
          drawObj.drawAll();
          return;
        } else if (
          e.offsetY > canvas.height / 2 + calculatedValues.lineInterval &&
          e.offsetY < canvas.height / 2 + calculatedValues.lineInterval * 2
        ) {
          side = "bottom";
        } else {
          return;
        }
      }
      /** @type {CreateUserData} */
      const res = await (await fetch(normalizedPath + "/join_" + side)).json();
      if (res.success) {
        isAuthorized = true;
        drawObj.drawAll();
        privateId = res.private_id;
        publicId = res.public_id;
        playerSide = res.side;
        if (roomId.slice(0, -1) !== "00000000-0000-0000-0000-00000000000") {
          sessionStorage.setItem(
            roomId,
            JSON.stringify({
              privateId,
              publicId,
            })
          );
        }
        players[side].set(publicId, {
          name: res.name,
          selecting_piece: null,
          is_inactive: false,
        });
        console.log("Logging in as:", res.name);
        sender.authorize(privateId);
        heartbeat.start();
        drawObj.drawAll();
      } else {
        alert(`ユーザー登録に失敗しました。\n理由: ${res.message}`);
      }
    } else {
      if (
        e.offsetX > calculatedValues.offsetX &&
        e.offsetX < calculatedValues.width &&
        e.offsetY > calculatedValues.offsetY &&
        e.offsetY < calculatedValues.height
      ) {
        const x = Math.floor((e.offsetX - calculatedValues.offsetX) / calculatedValues.lineInterval);
        const y = Math.floor((e.offsetY - calculatedValues.offsetY) / calculatedValues.lineInterval);
        if (isMovePieceMode) {
          if (x !== previousX || y !== previousY) {
            sender.movePiece([previousX, previousY], [x, y]);
          }
          isMovePieceMode = false;
        } else if (x === previousX && y === previousY && !isMovePieceMode) {
          isMovePieceMode = true;
        }
        previousX = x;
        previousY = y;
        sender.selectPiece(x, y);
      }
    }
  });
  for await (const message of receiver) {
    /** @type {ReceivedEvent} */
    const data = JSON.parse(message);
    switch (data.t) {
      case MessageType.SelectPiece: {
        if (players.top.has(data.i)) {
          /** @type {PlayerData} */ (players.top.get(data.i)).selecting_piece = data.c;
        } else if (players.bottom.has(data.i)) {
          /** @type {PlayerData} */ (players.bottom.get(data.i)).selecting_piece = data.c;
        }
        redraw();
        break;
      }
      case MessageType.MovePiece: {
        // TODO: TODO
        const res = await fetch(normalizedPath + "/room_data");
        /** @type {RawRoomData} */
        const data = await res.json();
        currentTurn = data.current_turn;
        topPieces = data.top_pieces;
        bottomPieces = data.bottom_pieces;
        redraw();
        break;
      }
      case MessageType.TopPlayerJoin: {
        players.top.set(data.i, {
          name: data.c,
          selecting_piece: null,
          is_inactive: false,
        });
        break;
      }
      case MessageType.BottomPlayerJoin: {
        players.bottom.set(data.i, {
          name: data.c,
          selecting_piece: null,
          is_inactive: false,
        });
        break;
      }
      case MessageType.TopPlayerLeave: {
        players.top.delete(data.i);
        break;
      }
      case MessageType.BottomPlayerLeave: {
        players.top.delete(data.i);
        break;
      }
      case MessageType.SessionExpired: {
        alert("セッションが期限切れになりました。");
        heartbeat.stop();
        isAuthorized = false;
        privateId = "";
        publicId = "";
        sessionStorage.removeItem("authInfo");
        redraw();
        break;
      }
    }
  }
});
