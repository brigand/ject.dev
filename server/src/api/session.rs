use crate::{
    db::{
        Db, {self},
    },
    ids,
    state::{FileKind, Session, SessionMeta},
};
use actix_web::{post, put, web, HttpResponse};
use db::DbError;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct SessionNew {
    pub session: Session,
}

#[post("/session/new")]
pub async fn r_post_session_new(
    web::Json(SessionNew { session }): web::Json<SessionNew>,
) -> Result<HttpResponse, DbError> {
    let db = Db::open_env().await?;
    let session_id = ids::make_session_id();

    let (db, session_index) = db.incr_session_counter(super::SESSION_LIMIT).await?;

    let db = db.put_session_index(session_index, &session_id).await?;
    let db = super::util::put_files(db, &session_id, &session).await?;

    let meta = SessionMeta {
        file_kinds: vec![FileKind::JavaScript, FileKind::Css, FileKind::Html],
    };
    db.put_session(&session_id, meta).await?;

    Ok(HttpResponse::Ok().json(json!({ "session_id": session_id })))
}

#[derive(Debug, Deserialize)]
pub struct SessionUpdate {
    session_id: String,
    session: Session,
}

#[put("/session")]
pub async fn r_put_session(info: web::Json<SessionUpdate>) -> db::DbResult<HttpResponse> {
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
