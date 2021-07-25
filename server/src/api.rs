use crate::cdn::cdnjs_script;
use crate::db::{self, Db, DbResult};
use crate::http_error::{ErrorMime, HttpError};
use crate::ids;
use crate::parser::{parse_html, HtmlPart};
use crate::state::{File, FileKind, Session, SessionMeta};
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

async fn put_files(mut db: Db, session_id: &str, session: &Session) -> DbResult<Db> {
    for file in &session.files {
        let file_name = file.kind.to_default_name();
        db = db.put_file(session_id, file_name, &file.contents).await?;
    }

    Ok(db)
}

#[get("/saved/{save_id}")]
async fn r_get_saved(info: web::Path<String>) -> Result<HttpResponse, DbError> {
    let db = Db::open_env().await?;
    let save_id = info.0.as_str();

    let (mut db, meta) = db.get_saved(&save_id).await?;

    let mut files = vec![];
    for file_kind in &meta.file_kinds {
        let file_name = file_kind.to_default_name();
        let (db2, contents) = db.get_file(&save_id, file_name).await?;
        db = db2;
        let file = File::new(*file_kind, contents);
        files.push(file);
    }

    let session = Session { files };

    Ok(HttpResponse::Ok().json(session))
}

#[post("/save")]
async fn r_post_save(
    web::Json(Save { session }): web::Json<Save>,
) -> Result<HttpResponse, DbError> {
    let save_id = ids::make_save_id();
    let mut db = Db::open_env().await?;

    db = put_files(db, &save_id, &session).await?;

    let meta = SessionMeta {
        file_kinds: vec![FileKind::JavaScript, FileKind::Css, FileKind::Html],
    };

    db.put_saved(&save_id, meta).await?;

    Ok(HttpResponse::Ok().json(json!({ "save_id": save_id })))
}

#[post("/session/new")]
async fn r_post_session_new(
    web::Json(SessionNew { session }): web::Json<SessionNew>,
) -> Result<HttpResponse, DbError> {
    let db = Db::open_env().await?;
    let session_id = ids::make_session_id();

    let (db, session_index) = db.incr_session_counter(SESSION_LIMIT).await?;

    let db = db.put_session_index(session_index, &session_id).await?;
    let db = put_files(db, &session_id, &session).await?;

    let meta = SessionMeta {
        file_kinds: vec![FileKind::JavaScript, FileKind::Css, FileKind::Html],
    };
    db.put_session(&session_id, meta).await?;

    Ok(HttpResponse::Ok().json(json!({ "session_id": session_id })))
}

#[derive(Debug, Deserialize)]
struct SessionUpdate {
    session_id: String,
    session: Session,
}

#[put("/session")]
async fn r_put_session(info: web::Json<SessionUpdate>) -> db::DbResult<HttpResponse> {
    let db = Db::open_env().await?;
    let SessionUpdate {
        session_id,
        session,
    } = info.0;

    let (mut db, session_meta) = match db.get_session(&session_id).await {
        Ok(r) => r,
        Err(err) => {
            eprintln!(
                "get_session error, session id = {}, error: {:?}",
                session_id, err
            );
            return Ok(HttpResponse::UnprocessableEntity().json(json!({
                "code": "session_not_found",
                "message": "Unable to find the specified session", "input_session_id": session_id
            })));
        }
    };

    let mut kinds = vec![];
    for file in &session.files {
        let file_name = file.kind.to_default_name();
        db = db.put_file(&session_id, file_name, &file.contents).await?;
        if !kinds.contains(&file.kind) {
            kinds.push(file.kind);
        }
    }

    if session_meta.file_kinds != kinds {
        let new_meta = SessionMeta {
            file_kinds: kinds,
            ..session_meta
        };
        db.put_session(&session_id, new_meta).await?;
    }

    Ok(HttpResponse::Ok().json(json!({})))
}

async fn try_get_file(
    db: Db,
    session_id: &str,
    err_mime: ErrorMime,
    file_kind: FileKind,
) -> Result<(Db, String), HttpError> {
    let (db, _session) = db
        .get_session(session_id)
        .await
        .map_err(|err| HttpError::db_error(err).with_mime(err_mime))?;

    let res = db
        .get_file(session_id, file_kind.to_default_name())
        .await
        .map_err(|_err| HttpError::file_not_found(err_mime).with_mime(err_mime))?;

    Ok(res)
}

#[get("/session/{session_id}/page.js")]
async fn r_get_session_page_js(info: web::Path<String>) -> Result<HttpResponse, HttpError> {
    let err_mime = ErrorMime::JavaScript;
    let db = Db::open_env()
        .await
        .map_err(|err| HttpError::db_error(err).with_mime(err_mime))?;
    let session_id = info.0;
    let (_, code) = try_get_file(db, &session_id, err_mime, FileKind::JavaScript).await?;

    Ok(HttpResponse::Ok()
        .header("content-type", "application/javascript; charset=utf-8")
        .body(code))

    // let session_id = info.0;
    // let err_mime = ErrorMime::JavaScript;
    // let code = try_get_file(&db, &session_id, err_mime, FileKind::JavaScript)?;

    // Ok(match compile(code) {
    //     Ok(js) => HttpResponse::Ok()
    //         .header("content-type", "application/javascript; charset=utf-8")
    //         .body(js),
    //     Err(err) => {
    //         println!("compile error: {:?}", err);
    //         println!("compile error root cause: {:?}", err.source());
    //         HttpError::js_compile_fail(err).to_response(err_mime)
    //     }
    // })
}

#[get("/session/{session_id}/page.js.raw")]
async fn r_get_session_page_js_raw(info: web::Path<String>) -> Result<HttpResponse, HttpError> {
    let err_mime = ErrorMime::JavaScript;
    let db = Db::open_env()
        .await
        .map_err(|err| HttpError::db_error(err).with_mime(err_mime))?;
    let session_id = info.0;
    let (_, code) = try_get_file(db, &session_id, err_mime, FileKind::JavaScript).await?;

    Ok(HttpResponse::Ok()
        .header("content-type", "application/javascript; charset=utf-8")
        .body(code))
}

#[get("/session/{session_id}/page.css")]
async fn r_get_session_page_css(info: web::Path<String>) -> Result<HttpResponse, HttpError> {
    let err_mime = ErrorMime::Css;
    let db = Db::open_env()
        .await
        .map_err(|err| HttpError::db_error(err).with_mime(err_mime))?;
    let session_id = info.0;
    let (_, code) = try_get_file(db, &session_id, err_mime, FileKind::Css).await?;

    Ok(HttpResponse::Ok()
        .header("content-type", "text/css; charset=utf-8")
        .body(code))
}

#[get("/session/{session_id}/page")]
async fn r_get_session_page_html(info: web::Path<String>) -> Result<HttpResponse, HttpError> {
    let err_mime = ErrorMime::Html;
    let session_id = info.0;
    let db = Db::open_env()
        .await
        .map_err(|err| HttpError::db_error(err).with_mime(err_mime))?;
    let (_, html) = try_get_file(db, &session_id, err_mime, FileKind::Html).await?;

    let parts = match parse_html(&html) {
        Ok(parts) => parts,
        Err(err) => return Err(HttpError::invalid_html(err).with_mime(err_mime)),
    };

    let page_url = |suffix: &str| format!("/api/session/{}/page{}", session_id, suffix);
    let public_path = |path: &str| format!("/dist/{}", path);
    let public_script = |path: &str| format!("<script src=\"{}\"></script>", public_path(path));

    // TODO: perform searches like https://api.cdnjs.com/libraries?search=jquery&limit=1 to allow arbitrary cdnjs deps
    let html = parts
        .into_iter()
        .try_fold(String::with_capacity(html.len()), |mut out, part| {
            match part {
                HtmlPart::Literal(literal) => out.push_str(literal),
                HtmlPart::IncludePath(path) => match &path[..] {
                    &["console"] => out.push_str(&public_script("console.bundle.js")),
                    &["editors", "js"] | &["editors", "js", "url"] => {
                        out.push_str(&page_url(".js"))
                    }
                    &["editors", "js", "raw"] | &["editors", "js", "raw", "url"] => {
                        out.push_str(&page_url(".js.raw"))
                    }
                    &["editors", "css"]
                    | &["editors", "css", "url"]
                    | &["editors", "css", "raw"]
                    | &["editors", "css", "url", "raw"] => out.push_str(&page_url(".css")),
                    &["deps", "react"] => {
                        out.push_str(&cdnjs_script("react/17.0.2/umd/react.development.min.js"));
                        out.push_str(&cdnjs_script(
                            "react-dom/17.0.2/umd/react-dom.development.min.js",
                        ));
                    }
                    &["deps", "jquery"] => {
                        out.push_str(&cdnjs_script("jquery/3.6.0/jquery.min.js"));
                    }
                    &["editors", other, ..] => {
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
            // Based on jsfiddle's result frame http response
            .header("content-type", "text/html; charset=utf-8")
            .header("cache-control", "max-age=0, private, must-revalidate")
            .header("referrer-policy", "strict-origin-when-cross-origin")
            // Other maybe useful headers from that response:
            // x-frame-options: ALLOWALL
            // x-xss-protection: 0
            // x-content-type-options: nosniff
            // x-download-options: noopen
            // x-permitted-cross-domain-policies: none
            // set-cookie: csrftoken={long string}; path=/
            // vary: Origin
            // X-Firefox-Spdy: h2
            .body(html)),
        Err(err) => Err(HttpError::generate_html_fail(err).with_mime(err_mime)),
    }
}

pub fn service() -> Scope {
    web::scope("/api")
        .service(r_health)
        .service(r_get_saved)
        .service(r_post_save)
        .service(r_post_session_new)
        .service(r_put_session)
        .service(r_get_session_page_js)
        .service(r_get_session_page_js_raw)
        .service(r_get_session_page_css)
        .service(r_get_session_page_html)
}
