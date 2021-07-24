use crate::{env::open_sqlite_env, state::SessionMeta};
use actix_rt::blocking::BlockingError;
use actix_web::{guard::Connect, http::StatusCode, web::block, HttpResponse, ResponseError};
use owning_ref::OwningRef;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{borrow::Cow, fmt::Debug, sync::Mutex};
use thiserror::Error;

pub type DbResult<T, E = DbError> = Result<T, E>;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Failed to open rocksdb database")]
    Open { source: anyhow::Error },

    #[error(
        "Failed to get key {}. This is different from a key not existing.",
        key_debug
    )]
    GetKey {
        source: rusqlite::Error,
        key_debug: String,
    },

    #[error("Failed to create table. SQL: {}", sql)]
    CreateTable {
        source: rusqlite::Error,
        sql: String,
    },

    #[error("The key {} does not exist", key_debug)]
    KeyNotFound { key_debug: String },

    #[error("The key {} has a value that is invalid UTF-8", key_debug)]
    Utf8 { key_debug: String },

    #[error("Failed to deserialize the value of key {}", key_debug)]
    DeserializeJson {
        key_debug: String,
        source: serde_json::Error,
    },

    #[error("Failed to serialize the value of key {}", key_debug)]
    SerializeValue {
        source: serde_json::Error,
        key_debug: String,
    },

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
        meta: String,
    },
    #[error("Failed insert a Session Index for session {}", session_id)]
    PutSessionIndex {
        source: rusqlite::Error,
        session_id: String,
    },

    #[error("Unable to remove the key {}", key_debug)]
    UnableToRemoveKey {
        source: rusqlite::Error,
        key_debug: String,
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
            DbError::KeyNotFound { .. } => StatusCode::NOT_FOUND,
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
            DbError::GetKey { .. } => "db_get_key",
            DbError::KeyNotFound { .. } => "db_key_not_found",
            DbError::Utf8 { .. } => "db_utf8",
            DbError::DeserializeJson { .. } => "db_deser_json",
            DbError::SerializeValue { .. } => "db_ser_value",
            DbError::Put { .. } => "db_put_value",
            DbError::UnableToRemoveKey { .. } => "db_remove_key",
            DbError::CreateTable { source, sql } => "db_create_table",
            DbError::BlockCanceled {} => "db_block_canceled",
        }
    }
    pub fn to_response(&self) -> HttpResponse {
        self.error_response()
    }
}

static TABLES: &[&str] = &[
    r#"
CREATE TABLE IF NOT EXISTS kv (
    key BLOB PRIMARY KEY,
    value BLOB
)
"#,
    r#"
CREATE TABLE IF NOT EXISTS file (
    file_id INTEGER PRIMARY KEY,
    session_or_saved_id BLOB,
    name TEXT NOT NULL,
    contents BLOB
)
"#,
    r#"
CREATE TABLE IF NOT EXISTS saved (
    saved_id BLOB PRIMARY KEY,
    file_types BLOB
)
"#,
    r#"
CREATE TABLE IF NOT EXISTS session (
    session_id BLOB PRIMARY KEY,
    file_types BLOB
)
"#,
    r#"
CREATE TABLE IF NOT EXISTS session_index (
    index INTEGER NON NULL,
    session_id BLOB,
)
"#,
    r#"
CREATE TABLE IF NOT EXISTS session_counter (
    count INTEGER NON NULL,
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
    session_counter_lock: Mutex<()>,
}

impl Db {
    /// Create/open database at $JECT_DB, with a fallback
    // of "$(pwd)/ject.db3" (see [crate::env])
    pub async fn open_env() -> DbResult<Self> {
        let db = actix_web::web::block(|| {
            open_sqlite_env()
                .map(|db| Self {
                    db,
                    session_counter_lock: Default::default(),
                })
                .map_err(|source| DbError::Open {
                    source: source.into(),
                })
        })
        .await?;

        Ok(db)
    }

    /// Creates all tables if they don't already exist
    pub fn create_tables(&self) -> DbResult<()> {
        for create_table in TABLES.into_iter().copied() {
            self.db
                .execute(create_table, [])
                .map_err(|source| DbError::CreateTable {
                    sql: create_table.to_owned(),
                    source,
                })?;
        }

        Ok(())
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
            self.db
                .execute(
                    r#"UPSERT INTO file (session_or_saved_id, name, contents) VALUES (?, ?, ?)"#,
                    rusqlite::params![session_or_saved_id, file_name, contents],
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
        let meta = serde_json::to_string(&meta).expect("ject: SessionMeta to json");

        let self2 = block(move || {
            self.db
                .execute(
                    r#"UPSERT INTO saved (saved_id, meta) VALUES (?, ?)"#,
                    rusqlite::params![saved_id, meta],
                )
                .map(|_| self)
                .map_err(|source| DbError::PutSaved {
                    source,
                    saved_id,
                    meta,
                })
        })
        .await?;

        Ok(self2)
    }

    /// Store an entry in the 'session' table.
    pub async fn put_session(self, session_id: &str, meta: SessionMeta) -> DbResult<Self> {
        let session_id = session_id.to_owned();
        let meta = serde_json::to_string(&meta).expect("ject: SessionMeta to json");

        let self2 = block(move || {
            self.db
                .execute(
                    r#"UPSERT INTO session (session_id, meta) VALUES (?, ?)"#,
                    rusqlite::params![session_id, meta],
                )
                .map(|_| self)
                .map_err(|source| DbError::PutSaved {
                    source,
                    saved_id: session_id,
                    meta,
                })
        })
        .await?;

        Ok(self2)
    }

    /// Store an entry in the 'session_index' table.
    pub async fn put_session_index(self, index: usize, session_id: &str) -> DbResult<Self> {
        let session_id = session_id.to_owned();

        let self2 = block(move || {
            self.db
                .execute(
                    r#"UPSERT INTO session_index (index, session_id) VALUES (?, ?)"#,
                    rusqlite::params![index, session_id],
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

    pub fn incr_session_counter(&self, max_value: u32) -> DbResult<u32> {
        let _lock = self
            .session_counter_lock
            .lock()
            .expect("session_counter_lock should never be poisoned");
        let key = Key::SessionCounter.render();
        let mut next = match self.get_json::<u32>(&key) {
            Ok(current) => current % max_value,

            Err(DbError::KeyNotFound { .. }) => 0,
            Err(err) => return Err(err),
        };

        if next < max_value {
            next += 1;
        } else {
            next = 1;
        }

        // Remove existing session at this index if any.
        let existing_key = Key::SessionIndex { index: next };
        match self.get_json::<String>(&existing_key) {
            Ok(session_id) => {
                let session_key = Key::Session {
                    session_id: Cow::Owned(session_id),
                };
                let _r = self.remove_key(&existing_key);
                let _r = self.remove_key(&session_key);
            }
            Err(DbError::DeserializeJson { .. }) => {
                let _r = self.remove_key(&existing_key);
            }
            Err(DbError::KeyNotFound { .. }) => {
                // Do nothing
            }
            Err(err) => return Err(err),
        };

        self.put_json(&key, next)?;

        Ok(next)
    }
}
