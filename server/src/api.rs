use std::sync::Arc;

use crate::http_error::{ErrorMime, HttpError};
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

    if let Some(mut existing_session) = state.sessions().into_item(&session_id) {
        *existing_session = session;
        HttpResponse::Ok().json(json!({}))
    } else {
        HttpResponse::UnprocessableEntity().json(json!({
        "message": "Unknown session_id",
         "input_session_id": session_id,
        }))
    }
}

#[get("/session/{session_id}/page.js")]
async fn r_get_session_page_js(
    info: web::Path<String>,
    state: web::Data<Arc<State>>,
) -> impl Responder {
    let session_id = info.0;
    let err_mime = ErrorMime::JavaScript;
    let js = match state.sessions().into_item(&session_id) {
        None => return HttpError::session_not_found(err_mime).to_response(err_mime),
        Some(session) => match session.file(FileKind::JavaScript) {
            None => return HttpError::file_not_found(err_mime).to_response(err_mime),
            Some(css) => css.clone(),
        },
    };

    match compile(js.contents) {
        Ok(js) => HttpResponse::Ok()
            .header("content-type", "application/javascript; charset=utf-8")
            .body(js),
        Err(err) => HttpError::js_compile_fail(err).to_response(err_mime),
    }
}

#[get("/session/{session_id}/page.css")]
async fn r_get_session_page_css(
    info: web::Path<String>,
    state: web::Data<Arc<State>>,
) -> impl Responder {
    let session_id = info.0;
    let err_mime = ErrorMime::Css;
    let css = match state.sessions().into_item(&session_id) {
        None => return HttpError::session_not_found(err_mime).to_response(err_mime),
        Some(session) => match session.file(FileKind::Css) {
            None => return HttpError::file_not_found(err_mime).to_response(err_mime),
            Some(css) => css.clone(),
        },
    };

    HttpResponse::Ok()
        .header("content-type", "text/css; charset=utf-8")
        .body(css.contents)
}

#[get("/session/{session_id}/page")]
async fn r_get_session_page(info: web::Path<String>, state: web::Data<Arc<State>>) -> HttpResponse {
    let err_mime = ErrorMime::Html;
    let session_id = info.0;
    let html = match state.sessions().into_item(&session_id) {
        None => return HttpError::session_not_found(err_mime).to_response(err_mime),
        Some(session) => match session.file(FileKind::Html) {
            None => return HttpError::file_not_found(err_mime).to_response(err_mime),
            Some(html) => html.clone(),
        },
    };

    let parts = match parse_html(&html.contents) {
        Ok(parts) => parts,
        Err(err) => return HttpError::invalid_html(err).to_response(err_mime),
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
        Err(err) => HttpError::generate_html_fail(err).to_response(err_mime),
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
