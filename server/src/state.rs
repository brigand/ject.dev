use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileKind {
    JavaScript,
    Css,
    Html,
    Text,
}

impl FileKind {
    pub fn to_default_name(self) -> &'static str {
        match self {
            FileKind::JavaScript => "page.js",
            FileKind::Css => "page.css",
            FileKind::Html => "page.html",
            FileKind::Text => "page.txt",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    pub file_kinds: Vec<FileKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub kind: FileKind,
    pub contents: String,
}

impl File {
    pub fn new(kind: FileKind, contents: String) -> Self {
        Self { kind, contents }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub files: Vec<File>,
}
