use crate::{env::open_sqlite_env, state::SessionMeta};
use actix_rt::blocking::BlockingError;
use actix_web::{guard::Connect, http::StatusCode, web::block, HttpResponse, ResponseError};
use owning_ref::OwningRef;
use rusqlite::{params, Connection, Result, Row};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{borrow::Cow, fmt::Debug, sync::Mutex};
use thiserror::Error;

pub type DbResult<T, E = DbError> = Result<T, E>;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Failed to open database")]
    Open { source: anyhow::Error },

    #[error("Attempted to query one row but it returned no results")]
    NotFound {
        source: rusqlite::Error,
        sql: String,
    },

    #[error("Attempted to query one row but it returnd an unexpected error")]
    QueryRowOther {
        source: rusqlite::Error,
        sql: String,
    },

    #[error("Failed to create table. SQL: {}", sql)]
    CreateTable {
        source: rusqlite::Error,
        sql: String,
    },

    #[error("Failed to deserialize file_kinds")]
    DeFileKinds { source: serde_json::Error },

    #[error(
        "Failed insert a file {} into the database for session/saved {}",
        file_name,
        session_or_saved_id
    )]
    PutFile {
        source: rusqlite::Error,
        file_name: String,
        session_or_saved_id: String,
    },

    #[error("Failed insert a Saved with id {}", saved_id)]
    PutSaved {
        source: rusqlite::Error,
        saved_id: String,
        file_kinds: String,
    },
    #[error("Failed insert a Session Index for session {}", session_id)]
    PutSessionIndex {
        source: rusqlite::Error,
        session_id: String,
    },
    #[error("Failed to update the session counter. Action: {}", action)]
    SessionCounter {
        source: rusqlite::Error,
        action: &'static str,
    },

    #[error(
        "Unable to get the file named {} in saved/session {}",
        file_name,
        session_or_saved_id
    )]
    GetFile {
        source: rusqlite::Error,
        session_or_saved_id: String,
        file_name: String,
    },

    #[error("Unable to get the saved session with id {}", saved_id)]
    GetSaved { source: Box<Self>, saved_id: String },

    #[error("Unable to get the temporary session with id {}", session_id)]
    GetSession {
        source: Box<Self>,
        session_id: String,
    },

    #[error("The blocking operation was canceled")]
    BlockCanceled {},
}

impl From<BlockingError<DbError>> for DbError {
    fn from(error: BlockingError<DbError>) -> Self {
        match error {
            BlockingError::Error(inner) => inner,
            BlockingError::Canceled => DbError::BlockCanceled {},
        }
    }
}

impl ResponseError for DbError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            DbError::NotFound { .. } => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let message = match self {
            DbError::Open { .. } => "Unable to open database".to_string(),
            _ => self.to_string(),
        };

        HttpResponse::build(self.status_code())
            .json(json!({ "code":self.code(), "message": message }))
    }
}

impl DbError {
    pub fn code(&self) -> &'static str {
        match self {
            DbError::Open { .. } => "db_open",
            DbError::CreateTable { .. } => "db_create_table",
            DbError::BlockCanceled { .. } => "db_block_canceled",
            DbError::DeFileKinds { .. } => "db_de_file_kinds",
            DbError::PutFile { .. } => "db_put_file",
            DbError::PutSaved { .. } => "db_put_saved",
            DbError::PutSessionIndex { .. } => "db_put_session_index",
            DbError::GetFile { .. } => "db_get_file",
            DbError::GetSaved { .. } => "db_get_saved",
            DbError::GetSession { .. } => "db_get_session",
            DbError::SessionCounter { .. } => "db_session_counter",
            DbError::NotFound { .. } => "db_row_not_found",
            DbError::QueryRowOther { .. } => "db_row_other_error",
        }
    }
    pub fn to_response(&self) -> HttpResponse {
        self.error_response()
    }
}

static TABLES: &[&str] = &[
    r#"
CREATE TABLE IF NOT EXISTS kv (
    key TEXT PRIMARY KEY,
    value BLOB
)
"#,
    r#"
CREATE TABLE IF NOT EXISTS file (
    file_id TEXT PRIMARY KEY,
    session_or_saved_id TEXT,
    name TEXT NOT NULL,
    contents TEXT
)
"#,
    r#"
CREATE TABLE IF NOT EXISTS saved (
    saved_id TEXT PRIMARY KEY,
    file_kinds TEXT
)
"#,
    r#"
CREATE TABLE IF NOT EXISTS session (
    session_id TEXT PRIMARY KEY,
    file_kinds TEXT
)
"#,
    r#"
CREATE TABLE IF NOT EXISTS session_index (
    idx INTEGER PRIMARY KEY NOT NULL,
    session_id TEXT
)
"#,
    r#"
CREATE TABLE IF NOT EXISTS session_counter (
    id INTEGER PRIMARY KEY CHECK (id = 0),
    count INTEGER NON NULL
)
"#,
];

/// Represents a key in the rocksdb database. Each is serialized to JSON using serde_json.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Key<'a> {
    /// The single session index cursor
    SessionCounter,

    /// A session in the list of current sessions
    /// Value is a session_id string
    SessionIndex { index: u32 },

    /// A session (containing all relevant data). These can be recycled when SessionIndex wraps around.
    Session { session_id: Cow<'a, str> },

    /// A file in a session or saved
    File {
        session_or_saved_id: Cow<'a, str>,
        name: Cow<'a, str>,
    },

    /// A persistently saved snapshot of a session
    Saved { id: Cow<'a, str> },
    //
    // /// An IPv4 or IPv6 address of a user.
    // IpSession {ip_address:String},
}

impl<'a> Key<'a> {
    fn render(&self) -> KeyRendered {
        KeyRendered {
            json: self.render_key(),
        }
    }

    pub fn session(session_id: &'a str) -> Self {
        Self::Session {
            session_id: Cow::Borrowed(session_id),
        }
    }

    pub fn file(id: &'a str, name: &'a str) -> Self {
        Self::File {
            session_or_saved_id: Cow::Borrowed(id),
            name: Cow::Borrowed(name),
        }
    }
}

#[derive(Debug, Clone)]
pub struct KeyRendered<'a> {
    json: Cow<'a, [u8]>,
}

pub trait KeyLike: Debug {
    fn render_key(&self) -> Cow<'_, [u8]>;

    fn dbg(&self) -> String {
        format!("{:?}", String::from_utf8_lossy(&self.render_key()))
    }
}

impl KeyLike for Key<'_> {
    fn render_key(&self) -> Cow<'_, [u8]> {
        use std::io::Write;
        let mut out = vec![];

        match self {
            Key::SessionCounter => {
                write!(&mut out, "SessionCounter").unwrap();
            }
            Key::SessionIndex { index } => {
                write!(&mut out, "SessionIndex::").unwrap();
                out.write_all(&index.to_be_bytes()).unwrap();
            }
            Key::Session { session_id } => {
                write!(&mut out, "Session::{}", session_id).unwrap();
            }
            Key::File {
                session_or_saved_id,
                name,
            } => {
                write!(&mut out, "File::{}::{}", session_or_saved_id, name).unwrap();
            }
            Key::Saved { id } => {
                write!(&mut out, "Saved::{}", id).unwrap();
            }
        }

        out.into()
    }
}

impl KeyLike for KeyRendered<'_> {
    fn render_key(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(&self.json)
    }
}

/// An open rocksdb database.
#[derive(Debug)]
pub struct Db {
    db: Connection,
}

impl Db {
    /// Create/open database at $JECT_DB, with a fallback
    // of "$(pwd)/ject.db3" (see [crate::env])
    pub async fn open_env() -> DbResult<Self> {
        let db = actix_web::web::block(|| {
            open_sqlite_env()
                .map(|db| Self { db })
                .map_err(|source| DbError::Open {
                    source: source.into(),
                })
        })
        .await?;

        Ok(db)
    }

    fn query_row<T, P, F>(&self, sql: &str, params: P, f: F) -> DbResult<T>
    where
        P: rusqlite::Params,
        F: FnOnce(&Row<'_>) -> rusqlite::Result<T>,
    {
        use rusqlite::Error::*;
        self.db
            .query_row(sql, params, f)
            .map_err(|source| match source {
                QueryReturnedNoRows => DbError::NotFound {
                    source,
                    sql: sql.to_owned(),
                },
                _ => DbError::QueryRowOther {
                    source,
                    sql: sql.to_owned(),
                },
            })
    }

    /// Creates all tables if they don't already exist
    pub async fn create_tables(self) -> DbResult<Self> {
        let self2 = block(move || {
            for create_table in TABLES.into_iter().copied() {
                self.db
                    .execute(create_table, [])
                    .map_err(|source| DbError::CreateTable {
                        sql: create_table.to_owned(),
                        source,
                    })?;
            }
            Ok(self)
        })
        .await?;

        Ok(self2)
    }

    /// Store an entry in the 'file' table, associated with a 'saved' or 'session'.
    pub async fn put_file(
        self,
        session_or_saved_id: &str,
        file_name: &str,
        contents: &str,
    ) -> DbResult<Self> {
        let session_or_saved_id = session_or_saved_id.to_owned();
        let file_name = file_name.to_owned();
        let contents = contents.to_owned();
        let self2 = block(move || {
            let file_id = format!("{}::{}", session_or_saved_id, file_name);
            self.db
                .execute(
                    r#"INSERT INTO file (file_id, session_or_saved_id, name, contents) VALUES (?1, ?2, ?3, ?4) ON CONFLICT(file_id) DO UPDATE SET contents=?4"#,
                    params![file_id, session_or_saved_id, file_name, contents],
                )
                .map(|_| self)
                .map_err(|source| DbError::PutFile {
                    source,
                    file_name,
                    session_or_saved_id,
                })
        })
        .await?;

        Ok(self2)
    }

    /// Store an entry in the 'saved' table.
    pub async fn put_saved(self, saved_id: &str, meta: SessionMeta) -> DbResult<Self> {
        let saved_id = saved_id.to_owned();
        let file_kinds =
            serde_json::to_string(&meta.file_kinds).expect("ject: SessionMeta to json");

        let self2 = block(move || {
            self.db
                .execute(
                    r#"INSERT INTO saved (saved_id, file_kinds) VALUES (?1, ?2) ON CONFLICT(saved_id) DO UPDATE SET file_kinds=?2"#,
                    params![saved_id, file_kinds],
                )
                .map(|_| self)
                .map_err(|source| DbError::PutSaved {
                    source,
                    saved_id,
                    file_kinds,
                })
        })
        .await?;

        Ok(self2)
    }

    /// Store an entry in the 'session' table.
    pub async fn put_session(self, session_id: &str, meta: SessionMeta) -> DbResult<Self> {
        let session_id = session_id.to_owned();
        let file_kinds =
            serde_json::to_string(&meta.file_kinds).expect("ject: SessionMeta to json");

        let self2 = block(move || {
            self.db
                .execute(
                    r#"INSERT INTO session (session_id, file_kinds) VALUES (?1, ?2) ON CONFLICT(session_id) DO UPDATE SET file_kinds=?2"#,
                    params![session_id, file_kinds],
                )
                .map(|_| self)
                .map_err(|source| DbError::PutSaved {
                    source,
                    saved_id: session_id,
                    file_kinds,
                })
        })
        .await?;

        Ok(self2)
    }

    /// Store an entry in the 'session_index' table.
    pub async fn put_session_index(self, index: u32, session_id: &str) -> DbResult<Self> {
        let session_id = session_id.to_owned();

        let self2 = block(move || {
            self.db
                .execute(
                    r#"INSERT INTO session_index (idx, session_id) VALUES (?1, ?2) ON CONFLICT(idx) DO UPDATE SET session_id=?2"#,
                    params![index, session_id],
                )
                .map(|_| self)
                .map_err(|source| DbError::PutSessionIndex {
                    source,
                    session_id: session_id,
                })
        })
        .await?;

        Ok(self2)
    }

    pub async fn get_file(
        self,
        session_or_saved_id: &str,
        file_name: &str,
    ) -> DbResult<(Self, String)> {
        let session_or_saved_id = session_or_saved_id.to_owned();
        let file_name = file_name.to_owned();

        let self2 = block(move || {
            self.db
                .query_row(
                    r#"SELECT contents FROM file WHERE session_or_saved_id = ? AND name = ?"#,
                    params![session_or_saved_id, file_name],
                    |row| row.get(0),
                )
                .map(|contents| (self, contents))
                .map_err(|source| DbError::GetFile {
                    source,
                    file_name,
                    session_or_saved_id,
                })
        })
        .await?;

        Ok(self2)
    }

    fn parse_meta(file_kinds: String) -> Result<SessionMeta, DbError> {
        let file_kinds =
            serde_json::from_str(&file_kinds).map_err(|source| DbError::DeFileKinds { source })?;
        Ok(SessionMeta { file_kinds })
    }

    pub async fn get_saved(self, saved_id: &str) -> DbResult<(Self, SessionMeta)> {
        let saved_id = saved_id.to_owned();

        let self2 = block(move || {
            self.query_row(
                r#"SELECT file_kinds FROM saved WHERE saved_id = ?"#,
                params![saved_id],
                |row| row.get(0),
            )
            .map_err(|source| DbError::GetSaved {
                source: Box::new(source),
                saved_id,
            })
            .and_then(Self::parse_meta)
            .map(|meta| (self, meta))
        })
        .await?;

        Ok(self2)
    }

    pub async fn get_session(self, session_id: &str) -> DbResult<(Self, SessionMeta)> {
        let session_id = session_id.to_owned();

        let self2 = block(move || {
            self.query_row(
                r#"SELECT file_kinds FROM session WHERE session_id = ?"#,
                params![session_id],
                |row| row.get(0),
            )
            .map_err(|source| DbError::GetSession {
                source: Box::new(source),
                session_id,
            })
            .and_then(Self::parse_meta)
            .map(|meta| (self, meta))
        })
        .await?;

        Ok(self2)
    }

    pub async fn incr_session_counter(self, max_value: u32) -> DbResult<(Self, u32)> {
        static SESSION_LOCK: once_cell::sync::Lazy<Mutex<()>> =
            once_cell::sync::Lazy::new(|| Default::default());

        let r =        block(move ||{
            let _lock = SESSION_LOCK.lock().expect("session_counter_lock should never be poisoned");

            let current: u32 = self.db.query_row(
                r#"SELECT count FROM session_counter LIMIT 1"#,
                [],
                |row| row.get(0),
            ).ok().unwrap_or(0);
            let next = current.wrapping_add(1) % max_value;
            println!("Session counter current: {}, next: {}", current, next);
            self.db
                .execute(
                    r#"INSERT INTO session_counter (id, count) VALUES (0, ?1) ON CONFLICT (id) DO UPDATE SET count = ?1"#,
                    params![next],
                )
                .map_err(|source| DbError::SessionCounter { source, action: "upsert" } )?;
            let deleted = self.db
                .execute(
                    r#"DELETE FROM file WHERE file_id IN (
                            SELECT file_id FROM file a INNER JOIN session_index b ON (
                                b.session_id = a.session_or_saved_id
                            ) WHERE b.idx = ?
                        )"#,
                    [next],
                )
                .map_err(|source| DbError::SessionCounter { source, action: "delete" } )?;
            println!("Deleted {} row(s) for old session with same index", deleted);
            Ok((self, next))
        }).await?;
        Ok(r)
    }
}
