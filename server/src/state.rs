use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum FileKind {
    JavaScript,
    Css,
    Html,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    kind: FileKind,
    contents: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub files: Vec<File>,
}

#[derive(Debug, Default)]
pub struct State {
    sessions: Mutex<HashMap<String, Session>>,
}

impl State {
    /// Get a lock for the state's sessions. Take care not to hold this across awaits.
    pub fn sessions(&self) -> MutexGuard<HashMap<String, Session>> {
        self.sessions
            .lock()
            .expect("State.sessions mutex must not be poisioned")
    }
}
