mod api;
mod state;

use crate::state::State;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};

#[get("/")]
async fn r_index() -> impl Responder {
    let html = "Hello, world";
    HttpResponse::Ok()
        .header("content-type", "text/html")
        .body(html)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state: State = State::default();

    let bind = "0.0.0.0:1950";
    println!("Starting server on {}", bind);

    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .service(r_index)
            .service(api::service())
    })
    .bind(bind)?
    .run()
    .await
}
