use owning_ref::MutexGuardRefMut;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Mutex, MutexGuard};

pub type SessionMap = HashMap<String, Session>;
pub type SessionRef<'a> = MutexGuardRefMut<'a, SessionMap, Session>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileKind {
    JavaScript,
    Css,
    Html,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub kind: FileKind,
    pub contents: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub files: Vec<File>,
}

impl Session {
    pub fn file(&self, kind: FileKind) -> Option<&File> {
        self.files.iter().find(|file| file.kind == kind)
    }
}

#[derive(Debug, Default)]
pub struct State {
    sessions: Mutex<SessionMap>,
}

impl State {
    /// Get a lock for the state's sessions. Take care not to hold this across awaits.
    pub fn sessions<'a>(&'a self) -> SessionsGuard<'a> {
        SessionsGuard {
            guard: self
                .sessions
                .lock()
                .expect("State.sessions mutex must not be poisioned"),
        }
    }
}

pub struct SessionsGuard<'a> {
    guard: MutexGuard<'a, SessionMap>,
}

impl<'a> Deref for SessionsGuard<'a> {
    type Target = SessionMap;

    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<'a> DerefMut for SessionsGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

impl<'a> SessionsGuard<'a> {
    pub fn into_item(self, key: &str) -> Option<SessionRef<'a>> {
        if self.guard.contains_key(key) {
            Some(MutexGuardRefMut::new(self.guard).map_mut(|guard| guard.get_mut(key).unwrap()))
        } else {
            None
        }
    }
}
