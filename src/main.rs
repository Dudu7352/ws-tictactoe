mod messages;
mod client_conn;
mod game;
mod game_service;
mod config;

use actix::{Actor, Addr};
use actix_web::{
    get, web::{self, Data}, App, Error, HttpRequest, HttpResponse, HttpServer, Scope
};
use actix_web_actors::ws;
use config::Config;
use game_service::GameService;

use crate::client_conn::ClientConn;

#[get("/ws")]
async fn connect_ws(req: HttpRequest, stream: web::Payload, data: Data<Addr<GameService>>) -> Result<HttpResponse, Error> {
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
        App::new()
            .app_data(Data::new(game_service.clone()))
            .service(Scope::new("/api").service(connect_ws))
            .service(actix_files::Files::new("/", "./static").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}