/** @type {WebSocket} */
const websocketConnection = new WebSocket("ws://127.0.0.1:8080/api/ws");

/** @type {{id: string, status: "waiting"} | {id: string, status: "started", board: string[][], turn: boolean, isO: boolean} | null} */
let game = null;

function startPlayingInput() {
  // TODO
  console.log("Starting game...");
  websocketConnection.send(
    JSON.stringify({
      startGame: {
        publicGame: true,
      },
    })
  );
}

function startPrivateInput() {
  // TODO
  console.log("Starting private...");
  websocketConnection.send(
    JSON.stringify({
      startGame: {
        publicGame: false,
      },
    })
  );
}

/**
 * @param {SubmitEvent} e
 */
function joinPrivateInput(e) {
  e.preventDefault();
  // TODO
  console.log(e.target.elements["id"].value);
  websocketConnection.send(
    JSON.stringify({
      joinPrivGame: {
        gameId: e.target.elements["id"].value,
      },
    })
  );
}

/**
 * @param {string} gameId
 */
function handleGameWaiting(gameId) {
  game = {
    id: gameId,
    status: "waiting"
  };
}

/**
 * @param {string} gameId
 * @param {boolean} yourTurn 
 */
function handleGameStarted(gameId, yourTurn) {
  game = {
    id: gameId,
    status: "started",
    board: Array.from(Array(3)).map(_ => Array(3).fill("")),
    turn: yourTurn,
    isO: yourTurn
  };
}

/**
 * @param {boolean} won
 */
function handleGameEnded(won) {
  alert(won ? "You win" : "You lose");
  game = null
}

/**
 * @param {number} x
 * @param {number} y
 */
function handleOpponentMove(x, y) {
  if(
    game !== null 
    && game.status === "started"
    && y > 0 && y < game.board.length
    && x > 0 && x < game.board[0].length
    ) {
    game.turn = true;
    game.board[y][x] = game.isO ? "X" : "O";
  }
}

websocketConnection.addEventListener("open", (e) => {
  console.log("Websocket connected", e);
});

websocketConnection.addEventListener("message", (messageEvent) => {
  // TODO
  const data = messageEvent.data;
  console.log(data);
  if (data.gameWaiting !== undefined) 
    handleGameWaiting(...data.gameWaiting);
  else if (data.gameStarted !== undefined)
    handleGameStarted(...data.gameStarted);
  else if (data.gameEnded !== undefined)
    handleGameEnded(...data.gameEnded);
  else if (data.opponentMove !== undefined)
    handleOpponentMove(...data.opponentMove);

  console.log(game);
});

document.getElementById("start-game").addEventListener("click", startPlayingInput);
document.getElementById("start-priv").addEventListener("click", startPrivateInput);
document.getElementById("join-priv").addEventListener("submit", joinPrivateInput);
