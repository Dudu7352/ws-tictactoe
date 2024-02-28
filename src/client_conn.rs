use std::time::{Duration, Instant};

use actix::{Actor, ActorContext, Addr, AsyncContext, Handler, StreamHandler};
use actix_web_actors::ws::{self, WebsocketContext};
use uuid::Uuid;

use crate::{game_service::GameService, messages::{server::ServerGameEvent, user::{Connect, Disconnect, UserConnectionEvent, UserEvent, UserGameEvent}}};

const PING_TIMEOUT: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(15);

pub struct ClientConn {
    pub id: Uuid,
    pub game_id: Option<Uuid>,
    pub game_service: Addr<GameService>,
    pub last_ping: Instant
}

impl Actor for ClientConn {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        self.game_service.do_send(UserConnectionEvent::Connect(Connect {
            player_id: self.id,
            addr: ctx.address()
        }));
    }

    fn stopping(&mut self, _: &mut Self::Context) -> actix::prelude::Running {
        self.game_service.do_send(UserConnectionEvent::Disconnect(Disconnect {
            player_id: self.id,
            game_id: self.game_id
        }));
        actix::Running::Stop
    }
}

impl ClientConn {
    pub fn new(game_service: Addr<GameService>) -> Self {
        Self {
            id: Uuid::new_v4(),
            game_id: None,
            game_service,
            last_ping: Instant::now()
        }
    }

    pub fn hb(&self, ctx: &mut WebsocketContext<Self>) {
        ctx.run_interval(PING_TIMEOUT,|conn, ctx| {
            ctx.ping(b"PING");
            let elapsed = conn.last_ping.elapsed();
            if elapsed > CLIENT_TIMEOUT {
                ctx.close(None);
                ctx.stop();
            }
            if elapsed > 2 * PING_TIMEOUT {
                let _ = conn.game_service.send(UserConnectionEvent::NotResponding);
            }
        });
    }
}

impl Handler<ServerGameEvent> for ClientConn {
    type Result = ();

    fn handle(&mut self, msg: ServerGameEvent, ctx: &mut Self::Context) -> Self::Result {
        match &msg {
            ServerGameEvent::GameWaiting(game_waiting) => {
                self.game_id = Some(game_waiting.game_id);
            },
            ServerGameEvent::GameEnded(_) => {
                self.game_id = None;
            },
            _ => {}
        }
        ctx.text(serde_json::to_string(&msg).unwrap());
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ClientConn {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(ws_msg) = item {
            match ws_msg {
                ws::Message::Continuation(_) => {},
                ws::Message::Text(txt) => {
                    match serde_json::from_str::<UserEvent>(&txt) {
                        Ok(event) => {
                            if let Some(_) = self.game_id {
                                match event {
                                    UserEvent::StartGame(_) => return (),
                                    UserEvent::JoinPrivGame(_) => return (),
                                    UserEvent::PlayerMove(_) => {},
                                }
                            }
                            self.game_service.do_send(UserGameEvent {
                                player_id: self.id,
                                event
                            });
                        },
                        Err(err) => {
                            // TODO: better error
                            ctx.text(err.to_string())
                        },
                    }
                },
                ws::Message::Binary(_bin) => {},
                ws::Message::Ping(msg) => {
                    ctx.pong(&msg);
                    self.last_ping = Instant::now();
                },
                ws::Message::Pong(_msg) => {
                    self.last_ping = Instant::now();
                },
                ws::Message::Close(reason) => {
                    ctx.close(reason);
                },
                ws::Message::Nop => {},
            }
        }
    }
}