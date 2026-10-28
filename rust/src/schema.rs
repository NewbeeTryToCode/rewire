//! Ontology: 7 node types, 6 relations (docs/design/format.md).

use crate::Meta;
use serde_json::Value;

pub const CLAIM_TYPES: [&str; 4] = ["constraint", "preference", "fact", "decision"];
pub const HUB_TYPES: [&str; 3] = ["project", "person", "concept"];
/// The frontmatter fields a note may carry (docs/design/format.md); anything else a file adds is not passed on.
pub const FIELDS: [&str; 22] = [
    "id",
    "type",
    "title",
    "scope",
    "importance",
    "confidence",
    "provenance",
    "status",
    "recorded_at",
    "valid_from",
    "valid_to",
    "retracted_at",
    "retracted_reason",
    "source",
    "edges",
    "keywords",
    "aliases",
    "tags",
    "created",
    "about",
    "project",
    "person",
];
pub const TYPES: [&str; 7] = ["constraint", "preference", "fact", "decision", "project", "person", "concept"];
pub const RELATIONS: [&str; 6] = ["about", "part_of", "depends_on", "related_to", "supersedes", "contradicts"];
pub const CONFIDENCE: [&str; 2] = ["explicit", "inferred"];
pub const PROVENANCE: [&str; 3] = ["user", "assistant", "tool"];
const REQUIRED: [&str; 8] = ["id", "type", "title", "scope", "importance", "confidence", "provenance", "recorded_at"];

#[derive(Debug)]
pub struct SchemaError(pub String);

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for SchemaError {}

fn err(msg: impl Into<String>) -> Result<(), SchemaError> {
    Err(SchemaError(msg.into()))
}

fn empty(v: Option<&Value>) -> bool {
    matches!(v, None | Some(Value::Null)) || v == Some(&Value::String(String::new()))
}

pub fn validate(meta: &Meta, body: &str) -> Result<(), SchemaError> {
    for key in REQUIRED {
        if empty(meta.get(key)) {
            return err(format!("missing field: {key}"));
        }
    }
    let s = |k: &str| meta.get(k).and_then(Value::as_str).unwrap_or("");
    let ty = s("type");
    if !TYPES.contains(&ty) {
        return err(format!("unknown type: {ty}"));
    }
    if !CONFIDENCE.contains(&s("confidence")) {
        return err(format!("bad confidence: {}", s("confidence")));
    }
    if !PROVENANCE.contains(&s("provenance")) {
        return err(format!("bad provenance: {}", s("provenance")));
    }
    match meta.get("importance").and_then(Value::as_i64) {
        Some(i) if (1..=5).contains(&i) && meta["importance"].is_i64() => {}
        _ => return err("importance must be an int 1-5"),
    }
    let scope = s("scope");
    if scope != "global" && !(scope.starts_with("project:") && scope.chars().count() > 8) {
        return err(format!("bad scope: {}", crate::pyjson::dumps_utf8(&meta["scope"])));
    }
    if let Some(Value::Array(edges)) = meta.get("edges") {
        for e in edges {
            let ok = e.get("rel").and_then(Value::as_str).is_some_and(|r| RELATIONS.contains(&r))
                && e.get("to").is_some_and(|t| !empty(Some(t)));
            if !ok {
                return err(format!("bad edge: {e}"));
            }
        }
    }
    if CLAIM_TYPES.contains(&ty) && body.trim().chars().count() > 600 {
        return err("claim body too long; claims must be atomic (<= 600 chars)");
    }
    if let Some(kw) = meta.get("keywords") {
        if !kw.is_null() && !kw.as_array().is_some_and(|a| a.iter().all(Value::is_string)) {
            return err("keywords must be a list of strings");
        }
    }
    let title = s("title");
    if title.trim().is_empty() || title.chars().count() > 160 {
        return err("title must be 1-160 chars");
    }
    Ok(())
}
