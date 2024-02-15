use actix::{Addr, Message};
use serde::Deserialize;
use uuid::Uuid;

use crate::client_conn::ClientConn;

#[derive(Message, Deserialize)]
#[serde(rename_all = "camelCase")]
#[rtype(result = "()")]
pub enum UserGameEvent {
    StartGame(StartGame),
    JoinPrivGame(JoinPrivGame),
    PlayerMove(PlayerMove),
}

#[derive(Message)]
#[rtype(result = "()")]
pub enum UserConnectionEvent {
    Connect(Connect),
    NotResponding,
    Disconnect(Disconnect)
}

pub struct Connect {
    pub player_id: Uuid,
    pub addr: Addr<ClientConn>,
}

#[derive(Deserialize)]
pub struct StartGame {
    pub player_id: Uuid,
    pub public_game: bool
}

#[derive(Deserialize)]
pub struct Disconnect {
    pub self_id: Uuid,
}

#[derive(Deserialize)]
pub struct JoinPrivGame {
    pub player_id: Uuid,
    pub game_id: Uuid
}

#[derive(Deserialize)]
pub struct PlayerMove {
    pub x: usize,
    pub y: usize,
    pub player_id: Uuid,
    pub game_id: Uuid
}