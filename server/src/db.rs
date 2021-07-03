use crate::env::open_rocksdb_env;
use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use owning_ref::OwningRef;
use rocksdb::DB;
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
        source: rocksdb::Error,
        key_debug: String,
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

    #[error("Failed to put value of key {} into the database", key_debug)]
    Put {
        source: rocksdb::Error,
        key_debug: String,
    },

    #[error("Unable to remove the key {}", key_debug)]
    UnableToRemoveKey {
        source: rocksdb::Error,
        key_debug: String,
    },
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
        }
    }
    pub fn to_response(&self) -> HttpResponse {
        self.error_response()
    }
}

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
pub struct IjDb {
    db: DB,
    session_counter_lock: Mutex<()>,
}

impl IjDb {
    /// Create/open database in $IJ_DATA_DIR, with a fallback
    // of "$(pwd)/ij_data_dir" (see [crate::env])
    pub fn open_env() -> DbResult<Self> {
        open_rocksdb_env()
            .map(|db| Self {
                db,
                session_counter_lock: Default::default(),
            })
            .map_err(|source| DbError::Open { source })
    }

    pub fn get_bytes(&self, key: &dyn KeyLike) -> DbResult<Option<Vec<u8>>> {
        self.db
            .get(key.render_key())
            .map_err(|source| DbError::GetKey {
                source,
                key_debug: format!("{:?}", key),
            })
    }

    pub fn get_bytes_pinned(
        &self,
        key: &dyn KeyLike,
    ) -> DbResult<Option<rocksdb::DBPinnableSlice<'_>>> {
        self.db
            .get_pinned(key.render_key())
            .map_err(|source| DbError::GetKey {
                source,
                key_debug: key.dbg(),
            })
    }

    /// Read the bytes for the specified key and parse it as UTF8. Pairs with [`Self::put_text`].
    pub fn get_text(&self, key: &dyn KeyLike) -> DbResult<String> {
        let bytes = self.get_bytes(key)?.ok_or_else(|| DbError::KeyNotFound {
            key_debug: key.dbg(),
        })?;
        String::from_utf8(bytes).map_err(|_source| DbError::Utf8 {
            key_debug: key.dbg(),
        })
    }

    /// Store the value in the specified key as UTF8. Pairs with [`Self::get_text`].
    pub fn put_text(&self, key: &dyn KeyLike, value: &str) -> DbResult<()> {
        self.db
            .put(key.render_key(), value)
            .map_err(|source| DbError::Put {
                key_debug: key.dbg(),
                source,
            })?;

        Ok(())
    }

    /// Zero-copy version of [`Self::get_text`]. Unclear if this is actually more efficient.
    /// You can deref or AsRef to get `&str`, e.g.
    ///
    /// ```norun
    /// if let Some(text) = db.get_text_pinned()? {
    ///   let s: &str = x.as_ref();
    /// }
    /// ```
    pub fn get_text_pinned(
        &self,
        key: &dyn KeyLike,
    ) -> DbResult<Option<OwningRef<Box<rocksdb::DBPinnableSlice<'_>>, str>>> {
        let res = self.get_bytes_pinned(key);
        match res {
            Ok(Some(slice)) => {
                if std::str::from_utf8(&*slice).is_ok() {
                    let owning = OwningRef::new(Box::new(slice))
                        .map(|slice| std::str::from_utf8(slice).unwrap());
                    Ok(Some(owning))
                } else {
                    Err(DbError::Utf8 {
                        key_debug: key.dbg(),
                    })
                }
            }
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub fn get_json<T>(&self, key: &dyn KeyLike) -> DbResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let bytes = self
            .get_bytes_pinned(key)?
            .ok_or_else(|| DbError::KeyNotFound {
                key_debug: key.dbg(),
            })?;
        serde_json::from_slice(&bytes[..]).map_err(|source| DbError::DeserializeJson {
            key_debug: key.dbg(),
            source,
        })
    }
    pub fn put_json<T>(&self, key: &dyn KeyLike, value: T) -> DbResult<()>
    where
        T: serde::Serialize,
    {
        let bytes = serde_json::to_vec(&value).map_err(|source| DbError::SerializeValue {
            key_debug: key.dbg(),
            source,
        })?;
        self.db
            .put(key.render_key(), bytes)
            .map_err(|source| DbError::Put {
                key_debug: key.dbg(),
                source,
            })?;

        Ok(())
    }

    pub fn remove_key(&self, key: &dyn KeyLike) -> DbResult<()> {
        self.db
            .delete(key.render_key())
            .map_err(|source| DbError::UnableToRemoveKey {
                key_debug: key.dbg(),
                source,
            })
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
