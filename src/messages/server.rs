use actix::Message;
use serde::Serialize;
use uuid::Uuid;

#[derive(Message, Serialize)]
#[rtype(result = "()")]
pub enum ServerGameEvent {
    GameWaiting(GameWaiting),
    GameStarted(GameStarted),
    GameEnded(GameEnded),
    OpponentMove(OpponentMove)
}

#[derive(Serialize)]
pub struct GameWaiting {
    pub game_id: Uuid
}

#[derive(Serialize)]
pub struct GameStarted {
    pub game_id: Uuid,
    pub your_turn: bool,
}

#[derive(Serialize)]
pub struct GameEnded {
    pub won: bool,
}

#[derive(Serialize)]
pub struct OpponentMove {
    pub x: usize,
    pub y: usize,
}
