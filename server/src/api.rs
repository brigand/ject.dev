use std::sync::Arc;

use crate::js::compile;
use crate::parser::{parse_html, HtmlPart};
use crate::state::{FileKind, Session, State};
use actix_web::{get, post, put, web, HttpResponse, Responder, Scope};
use serde::Deserialize;
use serde_json::json;

#[get("/health")]
async fn r_health() -> impl Responder {
    HttpResponse::Ok()
        .header("content-type", "text/plain; charset=utf-8")
        .body("Ok")
}

#[derive(Debug, Deserialize)]
struct SessionNew {
    session: Session,
}

#[post("/session/new")]
async fn r_post_session_new(
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

#[put("/session")]
async fn r_put_session(
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

#[get("/session/{session_id}/page.js")]
async fn r_get_session_page_js(
    info: web::Path<(String,)>,
    state: web::Data<Arc<State>>,
) -> impl Responder {
    let session_id = info.0 .0;
    let js_file = match state
        .sessions()
        .get(&session_id)
        .and_then(|session| session.file(FileKind::JavaScript).cloned())
    {
        Some(session) => session,
        None => {
            return HttpResponse::NotFound()
                .header("content-type", "text/html; charset=utf-8")
                .body("<h2>Unknown session or file</h2><p>Please try reloading the page</p>")
        }
    };

    let js = match compile(js_file.contents) {
        Ok(js) => js,
        Err(err) => {
            return HttpResponse::NotFound()
                .header("content-type", "text/plain; charset=utf-8")
                .body(format!("Failed to compile JS with SWC\n\n{:?}", err));
        }
    };

    HttpResponse::Ok()
        .header("content-type", "application/javascript; charset=utf-8")
        .body(js)
}

#[get("/session/{session_id}/page.css")]
async fn r_get_session_page_css(
    info: web::Path<(String,)>,
    state: web::Data<Arc<State>>,
) -> impl Responder {
    let session_id = info.0 .0;
    let css_file = match state
        .sessions()
        .get(&session_id)
        .and_then(|session| session.file(FileKind::Css).cloned())
    {
        Some(session) => session,
        None => {
            return HttpResponse::NotFound()
                .header("content-type", "text/html; charset=utf-8")
                .body("<h2>Unknown session or file</h2><p>Please try reloading the page</p>")
        }
    };

    HttpResponse::Ok()
        .header("content-type", "text/css; charset=utf-8")
        .body(css_file.contents)
}

#[get("/session/{session_id}/page")]
async fn r_get_session_page(
    info: web::Path<(String,)>,
    state: web::Data<Arc<State>>,
) -> HttpResponse {
    let session_id = info.0 .0;
    let html = {
        let sessions = state.sessions();
        let session = match sessions.get(&session_id).clone() {
            Some(session) => session,
            None => {
                return HttpResponse::NotFound()
                    .header("content-type", "text/html; charset=utf-8")
                    .body("<h2>Unknown session</h2><p>Please try reloading the page</p>")
            }
        };

        match session.file(FileKind::Html) {
            Some(html) => html.clone(),
            None => return HttpResponse::UnprocessableEntity().header("content-type", "text/html; charset=utf-8").body("<h2>No HTML file found</h2><p>The session appears to be missing an essential file</p>")
        }
    };

    let parts = match parse_html(&html.contents) {
        Ok(parts) => parts,
        Err(err) => {
            return HttpResponse::UnprocessableEntity()
                .header("content-type", "text/plain; charset=utf-8")
                .body(format!("Invalid HTML Provided\n\nReason:\n{}", err));
        }
    };

    let page_url = |suffix: &str| format!("/api/session/{}/page{}", session_id, suffix);

    let html = parts.into_iter().try_fold(
        String::with_capacity(html.contents.len()),
        |mut out, part| {
            match part {
                HtmlPart::Literal(literal) => out.push_str(literal),
                HtmlPart::IncludePath(path) => match &path[..] {
                    &["urls", "js"] => out.push_str(&page_url(".js")),
                    &["urls", "css"] => out.push_str(&page_url(".css")),
                    &["deps", "react"] => out.push_str(r#"<script src="https://cdnjs.cloudflare.com/ajax/libs/react/17.0.2/umd/react.development.min.js" crossorigin="anonymous" referrerpolicy="no-referrer"></script><script src="https://cdnjs.cloudflare.com/ajax/libs/react-dom/18.0.0-alpha-568dc3532/umd/react-dom.development.min.js" crossorigin="anonymous" referrerpolicy="no-referrer"></script>"#),
                    &["urls", other] => {
                        anyhow::bail!("Unexpected second segment in inject(urls.{})", other)
                    }
                    &[other, ..] => anyhow::bail!("Unexpected command: inject!({}, …)", other),
                    &[] => anyhow::bail!("Unexpected empty inject!()"),
                },
            }

            Ok(out)
        },
    );

    match html {
        Ok(html) => HttpResponse::Ok()
            .header("content-type", "text/html; charset=utf-8")
            .body(html),
        Err(err) => HttpResponse::UnprocessableEntity()
            .header("content-type", "text/plain; charset=utf-8")
            .body(format!(
                "Unable to generate html for the page\n\nReason:\n{}",
                err
            )),
    }
}

pub fn service() -> Scope {
    web::scope("/api")
        .service(r_health)
        .service(r_post_session_new)
        .service(r_put_session)
        .service(r_get_session_page_js)
        .service(r_get_session_page_css)
        .service(r_get_session_page)
}
