use crate::state::State;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, Scope};

#[get("/health")]
async fn r_health() -> impl Responder {
    HttpResponse::Ok()
        .header("content-type", "text/plain")
        .body("Ok")
}

pub fn service() -> Scope {
    web::scope("/api").service(r_health)
}
