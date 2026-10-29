//! rewire: own your brain, transfer it anywhere.
//!
//! A portable memory for agent harnesses. The brain is a folder of plain files (docs/design/format.md);
//! L0 working memory belongs to the harness, L1 episodic memory lives in `sessions/`, L2 semantic memory
//! in `nodes/` + `events/` (docs/design/architecture.md).

pub mod activation;
pub mod brain;
pub mod chat_review;
pub mod cli;
pub mod config;
pub mod consolidate;
pub mod fm;
pub mod hubs;
pub mod index;
pub mod integrations;
pub mod llm;
pub mod mcp;
pub mod nodefmt;
pub mod ops;
pub mod project;
pub mod pyjson;
pub mod schema;
pub mod security;
pub mod semantic;
pub mod skills;
pub mod snapshot;
pub mod sync;
pub mod text;
pub mod transcripts;
pub mod util;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Frontmatter / metadata map; keeps insertion order like a Python dict.
pub type Meta = serde_json::Map<String, serde_json::Value>;

#[derive(Debug)]
pub enum Error {
    /// Another rewire process holds the writer lock.
    Locked(String),
    /// A user-facing operation failed (bad id, empty memory, refused secret, ...).
    Op(String),
    Schema(String),
    Llm(String),
    Sync(String),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Locked(m) | Error::Op(m) | Error::Schema(m) | Error::Llm(m) | Error::Sync(m) | Error::Other(m) => f.write_str(m),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Other(e.to_string())
    }
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Error::Other(format!("index: {e}"))
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Other(format!("json: {e}"))
    }
}

impl From<schema::SchemaError> for Error {
    fn from(e: schema::SchemaError) -> Self {
        Error::Schema(e.0)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
