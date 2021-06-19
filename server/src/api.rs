use std::sync::Arc;

use crate::state::{self, Session, State};
use actix_web::{get, post, put, web, App, HttpResponse, HttpServer, Responder, Scope};
use serde::Deserialize;
use serde_json::json;

#[get("/health")]
async fn r_health() -> impl Responder {
    HttpResponse::Ok()
        .header("content-type", "text/plain")
        .body("Ok")
}

#[derive(Debug, Deserialize)]
struct SessionNew {
    session: Session,
}

#[post("/session/new")]
async fn r_session_new(
    info: web::Json<SessionNew>,
    state: web::Data<Arc<State>>,
) -> impl Responder {
    let session_id = nanoid::nanoid!();

    state
        .sessions()
        .insert(session_id.clone(), info.0.session.clone());

    HttpResponse::Ok().json(json!({ "session_id": session_id }))
}

#[derive(Debug, Deserialize)]
struct SessionUpdate {
    session_id: String,
    session: Session,
}

#[put("/session/{session_id}")]
async fn r_session_update(
    info: web::Json<SessionUpdate>,
    state: web::Data<Arc<State>>,
) -> impl Responder {
    let SessionUpdate {
        session_id,
        session,
    } = info.0;

    let mut sessions = state.sessions();
    if let Some(existing_session) = sessions.get_mut(&session_id) {
        *existing_session = session;
        drop(sessions);
        HttpResponse::Ok().json(json!({}))
    } else {
        drop(sessions);
        HttpResponse::UnprocessableEntity().json(json!({
        "message": "Unknown session_id",
         "input_session_id": session_id,
        }))
    }
}

pub fn service() -> Scope {
    web::scope("/api")
        .service(r_health)
        .service(r_session_new)
        .service(r_session_update)
}
