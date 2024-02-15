use std::collections::HashMap;

use actix::{Actor, Addr, Context, Handler};
use uuid::Uuid;

use crate::{
    client_conn::ClientConn,
    game::Game,
    messages::{
        server::{GameEnded, GameWaiting, ServerGameEvent},
        user::{Disconnect, UserConnectionEvent, UserGameEvent},
    },
};

pub struct GameService {
    sessions: HashMap<Uuid, Addr<ClientConn>>,
    games: HashMap<Uuid, Game>,
    waiting_game: Option<Uuid>,
}

impl GameService {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            games: HashMap::new(),
            waiting_game: None,
        }
    }

    pub fn register_player(&mut self, player_id: Uuid, player_addr: Addr<ClientConn>) {
        self.sessions.insert(player_id, player_addr);
    }

    pub fn deregister_player(&mut self, player_id: Uuid, game_id_opt: Option<Uuid>) {
        self.sessions.remove(&player_id);
        if game_id_opt.is_none() {
            return;
        }
        if let Some(game) = self.games.get(&game_id_opt.unwrap()) {
            match game {
                Game::Waiting { player_id: _ } => {}
                Game::Started {
                    players,
                    board: _,
                    first_player_turn: _,
                } => self.send_to_player(
                    &players[if players[0] == player_id { 1 } else { 0 }],
                    ServerGameEvent::GameEnded(GameEnded { won: true }),
                ),
            }
            self.games.remove(&game_id_opt.unwrap());
        }
    }

    pub fn send_to_player(&self, player_id: &Uuid, msg: ServerGameEvent) {
        if let Some(player) = self.sessions.get(player_id) {
            let _ = player.send(msg);
        }
    }

    pub fn join_game(&mut self, game_id: Uuid, joining_player_id: &Uuid) {
        let game = self.games.get(&game_id);
        if game.is_none() {
            return;
        }
        match game.unwrap() {
            Game::Started {
                players: _,
                board: _,
                first_player_turn: _,
            } => {}
            Game::Waiting {
                player_id: waiting_player_id,
            } => {
                let players = [*waiting_player_id, *joining_player_id];
                let started_game = Game::new_started(players);
                self.games.insert(game_id, started_game);
            }
        }
    }

    pub fn start_game(&mut self, player_id: &Uuid, public: bool) {
        if !public || self.waiting_game.is_none() {
            let game_id = Uuid::new_v4();
            let game = Game::Waiting {
                player_id: *player_id,
            };
            self.games.insert(game_id, game);
            if public {
                self.waiting_game = Some(game_id);
            }
            self.send_to_player(
                player_id,
                ServerGameEvent::GameWaiting(GameWaiting { game_id }),
            );
        } else {
            self.join_game(self.waiting_game.unwrap(), player_id)
        }
    }

    pub fn try_end_game(&mut self, game_id: &Uuid) -> Result<(), ()> {
        let game = self.games.get(game_id).ok_or(())?;
        let results = game.get_winner().ok_or(())?;

        self.send_to_player(
            &results.winner,
            ServerGameEvent::GameEnded(GameEnded { won: true }),
        );
        self.send_to_player(
            &results.loser,
            ServerGameEvent::GameEnded(GameEnded { won: false }),
        );
        self.games.remove(game_id);

        Ok(())
    }
}

impl Actor for GameService {
    type Context = Context<Self>;
}

impl Handler<UserConnectionEvent> for GameService {
    type Result = ();

    fn handle(&mut self, msg: UserConnectionEvent, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            UserConnectionEvent::Connect(connect) => {
                self.register_player(connect.player_id, connect.addr)
            }
            UserConnectionEvent::NotResponding => {}
            UserConnectionEvent::Disconnect(disconnect) => {
                self.deregister_player(disconnect.player_id, disconnect.game_id);
            }
        }
    }
}

impl Handler<UserGameEvent> for GameService {
    type Result = ();

    fn handle(&mut self, msg: UserGameEvent, ctx: &mut Self::Context) -> Self::Result {
        todo!()
    }
}
