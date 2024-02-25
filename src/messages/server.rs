use actix::Message;
use serde::Serialize;
use uuid::Uuid;

#[derive(Message, Serialize)]
#[serde(rename_all = "camelCase")]
#[rtype(result = "()")]
pub enum ServerGameEvent {
    GameWaiting(GameWaiting),
    GameStarted(GameStarted),
    GameEnded(GameEnded),
    OpponentMove(OpponentMove)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameWaiting {
    pub game_id: Uuid
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameStarted {
    pub game_id: Uuid,
    pub your_turn: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameEnded {
    pub won: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpponentMove {
    pub x: usize,
    pub y: usize,
}
