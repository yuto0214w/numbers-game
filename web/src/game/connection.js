// @ts-check

/** @typedef {import("../../types").IterResult} IterResult */

export function createCheckerConnection() {
  const secure = location.protocol === "https:" ? "s" : "";
  const normalizedPath = location.pathname.replace(/\/$/, "");
  const ws = new WebSocket(`ws${secure}://${location.host}${normalizedPath}/ws`);
  /** @type {Parameters<WebSocket["send"]>[0][]} */
  const messageSendingRequests = [];
  /** @type {Pick<PromiseWithResolvers<IterResult>, "promise" | "resolve">[]} */
  const messageReceivingRequests = [];
  /** @type {string[]} */
  const receivedMessages = [];
  /** @type {number | undefined} */
  let heartbeatTimeout;
  const heartbeatIntervalMs = 15000;
  ws.addEventListener("open", _openEvent => {
    const cloned = [...messageSendingRequests];
    messageSendingRequests.length = 0;
    for (const req of cloned) {
      ws.send(req);
    }
  });
  ws.addEventListener("message", message => {
    const _ignored = JSON.parse(message.data);
    if (messageReceivingRequests.length) {
      messageReceivingRequests.shift()?.resolve({ value: String(message.data), done: false });
    } else {
      receivedMessages.push(message.data);
    }
  });
  ws.addEventListener("close", _closeEvent => {
    clearTimeout(heartbeatTimeout);
    const cloned = [...messageReceivingRequests];
    messageReceivingRequests.length = 0;
    for (const req of cloned) {
      req.resolve({
        value: undefined,
        done: true,
      });
    }
  });
  /**
   * @param {Parameters<WebSocket["send"]>[0]} data
   */
  function wsSend(data) {
    if (ws.readyState === WebSocket.CONNECTING) {
      messageSendingRequests.push(data);
    } else {
      ws.send(data);
    }
  }
  const msgIter = {
    [Symbol.asyncIterator]() {
      return msgIter;
    },
    /**
     * @returns {Promise<IterResult>}
     */
    async next() {
      if (receivedMessages.length) {
        return {
          value: /** @type {string} */ (receivedMessages.shift()),
          done: false,
        };
      }
      if (ws.readyState === WebSocket.CLOSING || ws.readyState === WebSocket.CLOSED) {
        return {
          value: undefined,
          done: true,
        };
      }
      const { promise, resolve } = /** @type {PromiseWithResolvers<IterResult>} */ (Promise.withResolvers());
      messageReceivingRequests.push({ promise, resolve });
      return promise;
    },
  };
  return {
    sender: {
      /**
       * @param {string} privateId
       */
      authorize(privateId) {
        wsSend(
          JSON.stringify({
            i: privateId,
          })
        );
      },
      /**
       * @param {number} x
       * @param {number} y
       */
      selectPiece(x, y) {
        wsSend(
          JSON.stringify({
            t: 1,
            c: [x, y],
          })
        );
      },
      /**
       * @param {[number, number]} _
       * @param {[number, number]} _
       */
      movePiece([x1, y1], [x2, y2]) {
        wsSend(
          JSON.stringify({
            t: 2,
            c: [
              [x1, y1],
              [x2, y2],
            ],
          })
        );
      },
    },
    receiver: msgIter,
    heartbeat: {
      start() {
        const initTime = Date.now();
        heartbeatTimeout = setTimeout(
          function f(/** @type {number} */ counter) {
            wsSend('{"t":99}');
            heartbeatTimeout = setTimeout(
              f,
              heartbeatIntervalMs - (Date.now() - (initTime + heartbeatIntervalMs * counter)),
              counter + 1
            );
          },
          heartbeatIntervalMs,
          1
        );
      },
      stop() {
        clearTimeout(heartbeatTimeout);
      }
    },
  };
}
