use actix::{Addr, Message};
use serde::Deserialize;
use uuid::Uuid;

use crate::client_conn::ClientConn;

#[derive(Message)]
#[rtype(result = "()")]
pub struct UserGameEvent {
    pub player_id: Uuid,
    pub event: UserEvent
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UserEvent {
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
#[serde(rename_all = "camelCase")]
pub struct StartGame {
    pub public_game: bool
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Disconnect {
    pub player_id: Uuid,
    pub game_id: Option<Uuid>
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JoinPrivGame {
    pub game_id: Uuid
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerMove {
    pub x: usize,
    pub y: usize,
    pub game_id: Uuid
}