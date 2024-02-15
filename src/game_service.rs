use std::collections::HashMap;

use actix::{Actor, Addr, Context, Handler};
use uuid::Uuid;

use crate::{
    client_conn::ClientConn,
    game::Game,
    messages::{
        server::{GameEnded, GameStarted, GameWaiting, OpponentMove, ServerGameEvent},
        user::{UserConnectionEvent, UserGameEvent},
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
                players.iter().for_each(|player_id| {
                    self.send_to_player(
                        player_id,
                        ServerGameEvent::GameStarted(GameStarted {
                            game_id,
                            your_turn: started_game.is_current_turn(*player_id),
                        }),
                    )
                });
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

    fn handle(&mut self, msg: UserConnectionEvent, _ctx: &mut Self::Context) -> Self::Result {
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

    fn handle(&mut self, msg: UserGameEvent, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            UserGameEvent::StartGame(start_game) => {
                self.start_game(&start_game.player_id, start_game.public_game)
            }
            UserGameEvent::JoinPrivGame(join_priv_game) => {
                self.join_game(join_priv_game.game_id, &join_priv_game.player_id)
            }
            UserGameEvent::PlayerMove(player_move) => {
                if let Some(game) = self.games.get(&player_move.game_id) {
                    if player_move.x > 2 || player_move.y > 2 {
                        return;
                    }
                    match game {
                        Game::Waiting { player_id: _ } => (),
                        Game::Started {
                            players,
                            mut board,
                            first_player_turn,
                        } => {
                            let curr_player_i = 1 - *first_player_turn as usize;
                            if players[curr_player_i] == player_move.player_id {
                                board[player_move.y][player_move.x] = curr_player_i as i8;
                                self.send_to_player(
                                    &players[1 - curr_player_i],
                                    ServerGameEvent::OpponentMove(OpponentMove {
                                        x: player_move.x,
                                        y: player_move.y,
                                    }),
                                );
                                let _ = self.try_end_game(&player_move.game_id);
                            }
                        }
                    }
                }
            }
        }
    }
}
