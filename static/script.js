const gameStatusParagraph = document.getElementById("game-status");
const turnStatusParagraph = document.getElementById("turn-status");
const gameBoard = document.getElementById("game-board");

/** @type {WebSocket} */
const websocketConnection = new WebSocket("ws://127.0.0.1:8080/api/ws");

/** @type {{id: string, status: "waiting"} | {id: string, status: "started", board: string[][], turn: boolean, isO: boolean} | null} */
let game = null;

/** @type {HTMLDivElement[][]} */
let gameBoardTiles = [];

function initializeHtmlBoard() {
  gameBoard.style.gridTemplateColumns = `repeat(${game.board.length}, auto)`;
  game.board.forEach((row, y) => {
    gameBoardTiles.push(
      row.map((_, x) => {
        const tile = document.createElement("div");
        tile.innerText = "";
        tile.addEventListener("click" ,() => {
          if(game.status === "started" && game.board[y][x] === "" && game.turn) {
            tile.innerText = game.isO ? "O" : "X";
            game.board[y][x] = game.isO ? "O" : "X";
            websocketConnection.send(JSON.stringify({
              playerMove: {
                gameId: game.id,
                x: x,
                y: y
              }
            }));
            game.turn = false;
            turnStatusParagraph.innerText = "Opponent turn";
          }
        });
        gameBoard.appendChild(tile);
        return tile;
      })
    );
  });
}

function resetHtmlBoard() {
  gameBoardTiles.forEach((row) => row.forEach((e) => (e.innerText = "")));
}

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
        joinCode: e.target.elements["id"].value,
      },
    })
  );
}

/**
 * @param {Object} v
 * @param {string} v.gameId
 * @param {string} v.joinCode
 */
function handleGameWaiting({ gameId, joinCode }) {
  game = {
    id: gameId,
    status: "waiting",
  };
  gameStatusParagraph.innerText = `Waiting. Game Code: ${joinCode}`;
}

/**
 * @param {Object} v
 * @param {string} v.gameId
 * @param {boolean} v.yourTurn
 */
function handleGameStarted({ gameId, yourTurn }) {
  game = {
    id: gameId,
    status: "started",
    board: Array.from(Array(3)).map((_) => Array(3).fill("")),
    turn: yourTurn,
    isO: yourTurn,
  };
  if (gameBoardTiles.length === 0) initializeHtmlBoard();
  else resetHtmlBoard();
  gameStatusParagraph.innerText = `Playing with ${yourTurn ? "O" : "X"}`;
  turnStatusParagraph.innerText = yourTurn ? "Your turn" : "Opponent turn";
  gameBoard.style.display = "grid";
}

/**
 * @param {Object} v
 * @param {"win" | "loss" | "opponentDisconnected"} v.result
 */
function handleGameEnded({ result }) {
  alert(`Game ended with ${result}`);
  game = null;
  gameBoard.style.display = "none";
  gameStatusParagraph.innerText = "";
  turnStatusParagraph.innerText = "";
}

/**
 * @param {Object} v
 * @param {number} v.x
 * @param {number} v.y
 */
function handleOpponentMove({ x, y }) {
  if (
    game !== null &&
    game.status === "started" &&
    y >= 0 &&
    y < game.board.length &&
    x >= 0 &&
    x < game.board[0].length
  ) {
    game.turn = true;
    turnStatusParagraph.innerText = "Your turn";
    const tileValue = game.isO ? "X" : "O";
    game.board[y][x] = tileValue;
    gameBoardTiles[y][x].innerText = tileValue;
  }
}

websocketConnection.addEventListener("open", (e) => {
  console.log("Websocket connected", e);
});

websocketConnection.addEventListener("message", (messageEvent) => {
  // TODO
  const data = JSON.parse(messageEvent.data);
  console.log(data);
  if (data.gameWaiting !== undefined) handleGameWaiting(data.gameWaiting);
  else if (data.gameStarted !== undefined) handleGameStarted(data.gameStarted);
  else if (data.gameEnded !== undefined) handleGameEnded(data.gameEnded);
  else if (data.opponentMove !== undefined)
    handleOpponentMove(data.opponentMove);

  console.log(game);
});

document
  .getElementById("start-game")
  .addEventListener("click", startPlayingInput);

document
  .getElementById("start-priv")
  .addEventListener("click", startPrivateInput);

document
  .getElementById("join-priv")
  .addEventListener("submit", joinPrivateInput);
