//! rewire: own your brain, transfer it anywhere.
//!
//! A portable memory for agent harnesses. The brain is a folder of plain files (docs/design/format.md);
//! L0 working memory belongs to the harness, L1 episodic memory lives in `sessions/`, L2 semantic memory
//! in `nodes/` + `events/` (docs/design/architecture.md).

pub mod activation;
pub mod brain;
pub mod cli;
pub mod config;
pub mod consolidate;
pub mod excerpt;
pub mod fm;
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
    /// A filesystem or network failure, kept typed so a caller can tell it from a bad argument.
    Io(std::io::Error),
    /// The SQLite index failed.
    Db(rusqlite::Error),
    /// A file, an LLM reply or a command line held something other than the schema expects.
    Json(serde_json::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Locked(m) | Error::Op(m) | Error::Schema(m) | Error::Llm(m) | Error::Sync(m) | Error::Other(m) => f.write_str(m),
            Error::Io(e) => e.fmt(f),
            Error::Db(e) => write!(f, "index: {e}"),
            Error::Json(e) => write!(f, "json: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Db(e) => Some(e),
            Error::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Error::Db(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(e)
    }
}

impl From<schema::SchemaError> for Error {
    fn from(e: schema::SchemaError) -> Self {
        Error::Schema(e.0)
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod error_tests {
    use super::Error;

    /// The messages a user sees did not change when the variants gained types, and the cause is reachable.
    #[test]
    fn a_typed_error_keeps_its_old_message_and_exposes_its_cause() {
        let io = Error::from(std::io::Error::new(std::io::ErrorKind::NotFound, "no such brain"));
        assert_eq!(io.to_string(), "no such brain");
        assert!(std::error::Error::source(&io).is_some());

        let db = Error::from(rusqlite::Error::QueryReturnedNoRows);
        assert_eq!(db.to_string(), "index: Query returned no rows");
        assert!(std::error::Error::source(&db).is_some());

        let bad = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let expected = format!("json: {bad}");
        let json = Error::from(bad);
        assert_eq!(json.to_string(), expected);
        assert!(std::error::Error::source(&json).is_some());
    }

    /// A message-only error has no cause to show, so callers must not print an empty chain.
    #[test]
    fn a_message_error_has_no_cause() {
        let op = Error::Op("give inbox ids, or `all`".into());
        assert_eq!(op.to_string(), "give inbox ids, or `all`");
        assert!(std::error::Error::source(&op).is_none());
    }
}
