mod client_conn;
mod config;
mod game;
mod game_service;
mod messages;

use actix::{Actor, Addr};
use actix_cors::Cors;
use actix_web::{
    get,
    web::{self, Data},
    App, Error, HttpRequest, HttpResponse, HttpServer, Scope,
};
use actix_web_actors::ws;
use config::Config;
use game_service::GameService;

use crate::client_conn::ClientConn;

#[get("/ws")]
async fn connect_ws(
    req: HttpRequest,
    stream: web::Payload,
    data: Data<Addr<GameService>>,
) -> Result<HttpResponse, Error> {
    ws::start(ClientConn::new(data.as_ref().clone()), &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = match Config::load() {
        Ok(val) => val,
        Err(err) => panic!("Config error: {:?}", err),
    };

    let game_service = GameService::new().start();

    HttpServer::new(move || {
        let cors = config.cors_origin.as_ref().map(|origin| {
            Cors::default()
                .allowed_origin(origin.as_str())
                .allow_any_method()
                .allow_any_header()
        });

        let mut app = App::new()
            .wrap(cors.unwrap_or(Cors::default()))
            .app_data(Data::new(game_service.clone()))
            .service(Scope::new("/api").service(connect_ws));

        if config.host_static {
            app = app.service(actix_files::Files::new("/", "./static").index_file("index.html"))
        }

        app
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}