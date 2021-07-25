use crate::db::{Db, DbResult};
use crate::state::Session;

/// Store the session/saved files in sqlite.
pub async fn put_files(mut db: Db, session_id: &str, session: &Session) -> DbResult<Db> {
    for file in &session.files {
        let file_name = file.kind.to_default_name();
        db = db.put_file(session_id, file_name, &file.contents).await?;
    }

    Ok(db)
}
