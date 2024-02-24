const websocketConnection = new WebSocket("ws://127.0.0.1:8080/api/ws");

function startPlaying() {
  // TODO
  console.log("Starting game...");
  websocketConnection.send(JSON.stringify({
    startGame: {
      publicGame: true,
    },
  }));
}

function startPrivate() {
  // TODO
  console.log("Starting private...");
  websocketConnection.send(JSON.stringify({
    startGame: {
      publicGame: false,
    },
  }));
}

/**
 * @param {SubmitEvent} e
 */
function joinPriv(e) {
  e.preventDefault();
  // TODO
  console.log(e.target.elements["id"].value);
  websocketConnection.send(JSON.stringify({
    joinPrivGame: {
      gameId: e.target.elements["id"].value,
    },
  }));
}

websocketConnection.addEventListener("open", (e) => {
  console.log("Websocket connected", e);
});

websocketConnection.addEventListener("message", (messageEvent) => {
  // TODO
  console.log(messageEvent.data);
});

document.getElementById("start-game").addEventListener("click", startPlaying);
document.getElementById("start-priv").addEventListener("click", startPrivate);
document.getElementById("join-priv").addEventListener("submit", joinPriv);
