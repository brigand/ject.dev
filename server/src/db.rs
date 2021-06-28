use crate::env::open_rocksdb_env;
use actix_web::HttpResponse;
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

impl DbError {
    pub fn to_response(&self) -> HttpResponse {
        match self {
            DbError::Open { .. } => HttpResponse::InternalServerError()
                .json(json!({ "code": "db_open", "message": "Unable to open database" })),
            DbError::GetKey { .. } => HttpResponse::InternalServerError()
                .json(json!({ "code": "db_get_key", "message": self.to_string() })),
            DbError::KeyNotFound { .. } => HttpResponse::NotFound()
                .json(json!({ "code": "db_key_not_found", "message": self.to_string() })),
            DbError::Utf8 { .. } => HttpResponse::InternalServerError()
                .json(json!({ "code": "db_utf8", "message": self.to_string() })),
            DbError::DeserializeJson { .. } => HttpResponse::InternalServerError()
                .json(json!({ "code": "db_deser_json", "message": self.to_string() })),
            DbError::SerializeValue { .. } => HttpResponse::InternalServerError()
                .json(json!({ "code": "db_ser_value", "message": self.to_string() })),
            DbError::Put { .. } => HttpResponse::InternalServerError()
                .json(json!({ "code": "db_put_value", "message": self.to_string() })),
            DbError::UnableToRemoveKey { .. } => HttpResponse::InternalServerError()
                .json(json!({ "code": "db_remove_key", "message": self.to_string() })),
        }
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

    /// A persistently saved snapshot of a session
    Saved { id: Cow<'a, str> },
    //
    // /// An IPv4 or IPv6 address of a user.
    // IpSession {ip_address:String},
}

impl Key<'_> {
    fn render(&self) -> KeyRendered {
        KeyRendered {
            json: self.render_key(),
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
        serde_json::to_vec(self).expect("<inject::db::Key:: as KeyLike>::render_key should always be able to serde_json::to_vec itself").into()
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

    pub fn get_bytes_string(&self, key: &dyn KeyLike) -> DbResult<String> {
        let bytes = self.get_bytes(key)?.ok_or_else(|| DbError::KeyNotFound {
            key_debug: key.dbg(),
        })?;
        String::from_utf8(bytes).map_err(|source| DbError::KeyNotFound {
            key_debug: key.dbg(),
        })
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
                let sess_key = Key::Session {
                    session_id: Cow::Owned(session_id),
                };
                let _r = self.remove_key(&existing_key);
                let _r = self.remove_key(&sess_key);
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
