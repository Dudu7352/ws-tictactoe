const websocketConnection = new WebSocket("ws://127.0.0.1:8080/api/ws");

        function startPlaying() {
            // TODO
            console.log("Starting game...");
        }

        function startPrivate() {
            // TODO
        }

        /**
         * @param {SubmitEvent} e
         */
        function joinPriv(e) {
            e.preventDefault();
            // TODO
        }
        
        websocketConnection.addEventListener("open" , e => {
            console.log("Websocket connected", e);
        })

        websocketConnection.addEventListener("message", messageEvent => {
            // TODO
        })

        document.getElementById("start-game").addEventListener("click", startPlaying);
        document.getElementById("join-priv").addEventListener("submit", joinPriv)