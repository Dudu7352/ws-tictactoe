use std::collections::HashMap;

use actix::{Actor, Addr, Context, Handler};
use rand::{rngs::ThreadRng, thread_rng, Rng};
use uuid::Uuid;

use crate::{
    client_conn::ClientConn,
    game::{Game, GameEndResults},
    messages::{
        server::{GameEnded, GameResult, GameStarted, GameWaiting, OpponentMove, ServerGameEvent},
        user::{UserConnectionEvent, UserEvent, UserGameEvent},
    },
};

pub struct GameService {
    sessions: HashMap<Uuid, Addr<ClientConn>>,
    games: HashMap<Uuid, Game>,
    waiting_game: Option<Uuid>,
    waiting_priv_games: HashMap<String, Uuid>,
    rng: ThreadRng
}

impl GameService {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            games: HashMap::new(),
            waiting_game: None,
            waiting_priv_games: HashMap::new(),
            rng: thread_rng()
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
                Game::Waiting { player_id: _, join_code } => {
                    join_code.as_ref().map(|code| self.waiting_priv_games.remove(code));
                }
                Game::Started {
                    players,
                    board: _,
                    first_player_turn: _,
                } => self.send_to_player(
                    &players[if players[0] == player_id { 1 } else { 0 }],
                    ServerGameEvent::GameEnded(GameEnded { result: GameResult::OpponentDisconnected }),
                ),
            }
            self.games.remove(&game_id_opt.unwrap());
        }
    }

    pub fn send_to_player(&self, player_id: &Uuid, msg: ServerGameEvent) {
        if let Some(player) = self.sessions.get(player_id) {
            player.do_send(msg);
        }
    }

    pub fn join_private_game(&mut self, join_code: String, joining_player_id: &Uuid) {
        if let Some(game_id) = self.waiting_priv_games.get(&join_code) {
            self.join_game(game_id.clone(), joining_player_id)
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
                player_id: waiting_player_id, join_code
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
                join_code.as_ref().map(|code| self.waiting_priv_games.remove(code));
                self.games.insert(game_id, started_game);
            }
        }
    }

    pub fn start_game(&mut self, player_id: &Uuid, public: bool) {
        if !public || self.waiting_game.is_none() {
            let game_id = Uuid::new_v4();
            let join_code = if public { 
                self.waiting_game = Some(game_id);
                None 
            } else { 
                let code = format!("{:0>6}", self.rng.gen_range(0..=999999));
                self.waiting_priv_games.insert(code.clone(), game_id);
                Some(code)
            };

            let game = Game::Waiting {
                player_id: *player_id,
                join_code: join_code.clone()
            };

            self.games.insert(game_id, game);

            self.send_to_player(
                player_id,
                ServerGameEvent::GameWaiting(GameWaiting { game_id, join_code }),
            );
        } else {
            self.join_game(self.waiting_game.unwrap(), player_id);
            self.waiting_game = None;
        }
    }

    pub fn try_end_game(&mut self, game_id: &Uuid) -> Result<(), ()> {
        let game = self.games.get(game_id).ok_or(())?;
        let results = game.get_winner().ok_or(())?;
        match results {
            GameEndResults::Win { winner, loser } => {
                self.send_to_player(
                    &winner,
                    ServerGameEvent::GameEnded(GameEnded { result: GameResult::Win }),
                );
                self.send_to_player(
                    &loser,
                    ServerGameEvent::GameEnded(GameEnded { result: GameResult::Loss }),
                );
            },
            GameEndResults::Tie => {
                if let Game::Started { players, board: _, first_player_turn: _ } = game {
                    players.iter().for_each(|player|
                        self.send_to_player(player, 
                        ServerGameEvent::GameEnded(GameEnded { result: GameResult::Tie }))
                    );
                }    
            },
        }
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
        match msg.event {
            UserEvent::StartGame(start_game) => {
                self.start_game(&msg.player_id, start_game.public_game)
            }
            UserEvent::JoinPrivGame(join_priv_game) => {
                self.join_private_game(join_priv_game.join_code, &msg.player_id)
            }
            UserEvent::PlayerMove(player_move) => {
                if player_move.x > 2 || player_move.y > 2 {
                    return;
                }

                if let Some(game) = self.games.get_mut(&player_move.game_id) {
                    match game {
                        Game::Waiting { player_id: _, join_code: _ } => (),
                        Game::Started {
                            players,
                            board,
                            first_player_turn,
                        } => {
                            let curr_player_i = 1 - *first_player_turn as usize;
                            if players[curr_player_i] == msg.player_id {
                                board[player_move.y][player_move.x] = curr_player_i as i8;
                                let player_id = players[1 - curr_player_i].clone();
                                *first_player_turn = !(*first_player_turn);
                                self.send_to_player(
                                    &player_id,
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
