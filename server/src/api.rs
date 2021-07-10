use std::borrow::Cow;
use std::sync::Arc;

use crate::cdn::cdnjs_script;
use crate::db::{self, DbResult, IjDb, Key};
use crate::http_error::{ErrorMime, HttpError};
use crate::js::compile;
use crate::parser::{parse_html, HtmlPart};
use crate::state::{FileKind, Session, SessionMeta, State};
use crate::{ids, DbData};
use actix_web::{get, post, put, web, HttpResponse, Responder, Scope};
use db::DbError;
use serde::Deserialize;
use serde_json::json;

#[cfg(debug_assert)]
const SESSION_LIMIT: u32 = 512;
#[cfg(not(debug_assert))]
const SESSION_LIMIT: u32 = 1024 * 8;

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

#[derive(Debug, Deserialize)]
struct Save {
    session: Session,
}

fn put_files(db: &IjDb, session_id: &str, session: &Session) -> DbResult<()> {
    for file in &session.files {
        let file_name = file.kind.to_default_name();
        let file_key = Key::file(session_id, file_name);
        db.put_text(&file_key, &file.contents)?;
    }

    Ok(())
}

#[post("/save")]
async fn r_post_save(
    web::Json(Save { session }): web::Json<Save>,
    db: DbData,
) -> Result<HttpResponse, DbError> {
    let save_id = ids::make_save_id();

    put_files(&db, &save_id, &session)?;

    let save_key = db::Key::Saved {
        id: Cow::Borrowed(save_id.as_str()),
    };

    let meta = SessionMeta {
        file_kinds: vec![FileKind::JavaScript, FileKind::Css, FileKind::Html],
    };

    db.put_json(&save_key, meta)?;

    Ok(HttpResponse::Ok().json(json!({ "save_id": save_id })))
}

#[post("/session/new")]
async fn r_post_session_new(
    web::Json(SessionNew { session }): web::Json<SessionNew>,
    db: DbData,
) -> Result<HttpResponse, DbError> {
    let session_id = ids::make_session_id();

    let session_index = db.incr_session_counter(SESSION_LIMIT)?;

    let si_key = db::Key::SessionIndex {
        index: session_index,
    };
    db.put_json(&si_key, &session_id)?;

    let sess_key = db::Key::Session {
        session_id: Cow::Borrowed(session_id.as_str()),
    };

    put_files(&db, &session_id, &session)?;

    let meta = SessionMeta {
        file_kinds: vec![FileKind::JavaScript, FileKind::Css, FileKind::Html],
    };
    db.put_json(&sess_key, meta)?;

    Ok(HttpResponse::Ok().json(json!({ "session_id": session_id })))
}

#[derive(Debug, Deserialize)]
struct SessionUpdate {
    session_id: String,
    session: Session,
}

#[put("/session")]
async fn r_put_session(info: web::Json<SessionUpdate>, db: DbData) -> db::DbResult<HttpResponse> {
    let SessionUpdate {
        session_id,
        session,
    } = info.0;

    let session_key = Key::session(&session_id);
    let session_meta = match db.get_json::<SessionMeta>(&session_key) {
        Err(DbError::KeyNotFound { .. }) => return Ok(HttpResponse::UnprocessableEntity().json(json!({"code": "session_not_found", "message": "Unable to find the specified session", "input_session_id": session_id}))) ,
        Err(other) => return Err(other),
        Ok(session_meta) => session_meta
    };

    let mut kinds = vec![];
    for file in &session.files {
        let file_name = file.kind.to_default_name();
        let file_key = Key::file(&session_id, file_name);
        db.put_text(&file_key, &file.contents)?;
        if !kinds.contains(&file.kind) {
            kinds.push(file.kind);
        }
    }

    if session_meta.file_kinds != kinds {
        let new_meta = SessionMeta {
            file_kinds: kinds,
            ..session_meta
        };
        db.put_json(&session_key, new_meta)?;
    }

    Ok(HttpResponse::Ok().json(json!({})))
}

fn try_get_file(
    db: &IjDb,
    session_id: &str,
    err_mime: ErrorMime,
    file_kind: FileKind,
) -> Result<String, HttpError> {
    let session_key = Key::session(&session_id);
    let _session = db
        .get_bytes_pinned(&session_key)
        .map_err(|err| HttpError::db_error(err).with_mime(err_mime))?;
    let file_key = Key::file(&session_id, file_kind.to_default_name());

    db.get_text(&file_key).map_err(|err| match err {
        DbError::KeyNotFound { .. } => HttpError::file_not_found(err_mime).with_mime(err_mime),
        _ => HttpError::db_error(err).with_mime(err_mime),
    })
}

#[get("/session/{session_id}/page.js")]
async fn r_get_session_page_js(
    info: web::Path<String>,
    db: DbData,
) -> Result<HttpResponse, HttpError> {
    let session_id = info.0;
    let err_mime = ErrorMime::JavaScript;
    let code = try_get_file(&db, &session_id, err_mime, FileKind::JavaScript)?;

    Ok(match compile(code) {
        Ok(js) => HttpResponse::Ok()
            .header("content-type", "application/javascript; charset=utf-8")
            .body(js),
        Err(err) => {
            println!("compile error: {:?}", err);
            println!("compile error root cause: {:?}", err.source());
            HttpError::js_compile_fail(err).to_response(err_mime)
        }
    })
}

#[get("/session/{session_id}/page.js.raw")]
async fn r_get_session_page_js_raw(
    info: web::Path<String>,
    db: DbData,
) -> Result<HttpResponse, HttpError> {
    let session_id = info.0;
    let err_mime = ErrorMime::JavaScript;
    let code = try_get_file(&db, &session_id, err_mime, FileKind::JavaScript)?;

    Ok(HttpResponse::Ok()
        .header("content-type", "application/javascript; charset=utf-8")
        .body(code))
}

#[get("/session/{session_id}/page.css")]
async fn r_get_session_page_css(
    info: web::Path<String>,
    db: DbData,
) -> Result<HttpResponse, HttpError> {
    let session_id = info.0;
    let err_mime = ErrorMime::Css;
    let code = try_get_file(&db, &session_id, err_mime, FileKind::Css)?;

    Ok(HttpResponse::Ok()
        .header("content-type", "text/css; charset=utf-8")
        .body(code))
}

#[get("/session/{session_id}/page")]
async fn r_get_session_page_html(
    info: web::Path<String>,
    db: DbData,
) -> Result<HttpResponse, HttpError> {
    let err_mime = ErrorMime::Html;
    let session_id = info.0;
    let html = try_get_file(&db, &session_id, err_mime, FileKind::Html)?;

    let parts = match parse_html(&html) {
        Ok(parts) => parts,
        Err(err) => return Err(HttpError::invalid_html(err).with_mime(err_mime)),
    };

    let page_url = |suffix: &str| format!("/api/session/{}/page{}", session_id, suffix);

    // TODO: perform searches like https://api.cdnjs.com/libraries?search=jquery&limit=1 to allow arbitrary cdnjs deps
    let html = parts
        .into_iter()
        .try_fold(String::with_capacity(html.len()), |mut out, part| {
            match part {
                HtmlPart::Literal(literal) => out.push_str(literal),
                HtmlPart::IncludePath(path) => match &path[..] {
                    &["urls", "js"] => out.push_str(&page_url(".js")),
                    &["urls", "js", "raw"] => out.push_str(&page_url(".js.raw")),
                    &["urls", "css"] => out.push_str(&page_url(".css")),
                    &["deps", "react"] => {
                        out.push_str(&cdnjs_script("react/17.0.2/umd/react.development.min.js"));
                        out.push_str(&cdnjs_script(
                            "react-dom/17.0.2/umd/react-dom.development.min.js",
                        ));
                    }
                    &["deps", "jquery"] => {
                        out.push_str(&cdnjs_script("jquery/3.6.0/jquery.min.js"));
                    }
                    &["urls", other] => {
                        anyhow::bail!("Unexpected second segment in inject(urls.{})", other)
                    }
                    &[other, ..] => anyhow::bail!("Unexpected command: inject!({}, …)", other),
                    &[] => anyhow::bail!("Unexpected empty inject!()"),
                },
            }

            Ok(out)
        });

    match html {
        Ok(html) => Ok(HttpResponse::Ok()
            .header("content-type", "text/html; charset=utf-8")
            .body(html)),
        Err(err) => Err(HttpError::generate_html_fail(err).with_mime(err_mime)),
    }
}

pub fn service() -> Scope {
    web::scope("/api")
        .service(r_health)
        .service(r_post_save)
        .service(r_post_session_new)
        .service(r_put_session)
        .service(r_get_session_page_js)
        .service(r_get_session_page_js_raw)
        .service(r_get_session_page_css)
        .service(r_get_session_page_html)
}
