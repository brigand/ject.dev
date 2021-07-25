use crate::db::{self, Db};
use crate::ids;
use crate::state::{File, FileKind, Session, SessionMeta};
use actix_web::{get, post, web, HttpResponse};
use db::DbError;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct Save {
    session: Session,
}

#[get("/saved/{save_id}")]
pub async fn r_get_saved(info: web::Path<String>) -> Result<HttpResponse, DbError> {
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
pub async fn r_post_save(
    web::Json(Save { session }): web::Json<Save>,
) -> Result<HttpResponse, DbError> {
    let save_id = ids::make_save_id();
    let mut db = Db::open_env().await?;

    db = super::util::put_files(db, &save_id, &session).await?;

    let meta = SessionMeta {
        file_kinds: vec![FileKind::JavaScript, FileKind::Css, FileKind::Html],
    };

    db.put_saved(&save_id, meta).await?;

    Ok(HttpResponse::Ok().json(json!({ "save_id": save_id })))
}
