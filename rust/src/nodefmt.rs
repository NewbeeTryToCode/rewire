//! On-disk node format (docs/design/format.md): relations are frontmatter properties holding
//! [[wikilinks]] and are mirrored as relative markdown links under "## Related". Internally a node
//! keeps meta["edges"] = [{"rel", "to": <8-char id suffix>}].

use crate::schema::RELATIONS;
use crate::util::relpath;
use crate::{fm, Meta};
use regex::Regex;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub const RELATED: &str = "## Related";
const DERIVED: [&str; 3] = ["aliases", "tags", "status"];
const ORDER: [&str; 16] = [
    "id",
    "type",
    "title",
    "status",
    "aliases",
    "tags",
    "scope",
    "importance",
    "confidence",
    "provenance",
    "valid_from",
    "valid_to",
    "recorded_at",
    "retracted_at",
    "source",
    "keywords",
];

fn stem_id() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"-([0-9a-z]{8})$").unwrap())
}

fn link() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^\[\[([^\]|#]+)(?:[|#][^\]]*)?\]\]$").unwrap())
}

/// Normalize an id, short id, file stem or [[link]] to the 8-char id suffix.
pub fn short(reference: &str) -> String {
    let mut r = reference.trim().to_string();
    if let Some(c) = link().captures(&r) {
        r = c[1].trim().to_string();
    }
    let mut r = r.rsplit('/').next().unwrap_or("").to_string();
    if let Some(stripped) = r.strip_suffix(".md") {
        r = stripped.to_string();
    }
    let lower = r.to_lowercase();
    if let Some(c) = stem_id().captures(&lower) {
        return c[1].to_string();
    }
    if r.chars().count() == 26 {
        return crate::util::short_of(&r);
    }
    lower
}

fn value_text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => {
            if *b {
                "True".into()
            } else {
                "False".into()
            }
        }
        other => other.to_string(),
    }
}

fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
        Some(Value::Number(n)) => n.as_f64() != Some(0.0),
    }
}

pub fn source_text(source: &Value) -> Option<String> {
    match source {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Object(o) if !o.is_empty() => {
            let mut parts = Vec::new();
            if truthy(o.get("harness")) {
                parts.push(value_text(&o["harness"]));
            }
            if truthy(o.get("session")) {
                parts.push(value_text(&o["session"]));
            }
            if truthy(o.get("turn")) {
                parts.push(format!("turn {}", value_text(&o["turn"])));
            }
            Some(parts.join(" · "))
        }
        _ => None,
    }
}

/// (harness, session, turn) of a claim, from either stored form; None if incomplete.
pub fn parse_source(source: Option<&Value>) -> Option<(String, String, i64)> {
    let (h, s, t) = match source? {
        Value::Object(o) => (
            o.get("harness").filter(|v| truthy(Some(v))).map(value_text)?,
            o.get("session").filter(|v| truthy(Some(v))).map(value_text)?,
            o.get("turn").filter(|v| truthy(Some(v))).map(value_text)?,
        ),
        Value::String(text) => {
            let parts: Vec<&str> = text.split(" · ").collect();
            if parts.len() != 3 || !parts[2].starts_with("turn ") {
                return None;
            }
            (parts[0].to_string(), parts[1].to_string(), parts[2][5..].to_string())
        }
        _ => return None,
    };
    if h.is_empty() || s.is_empty() || t.is_empty() {
        return None;
    }
    let turn = t.trim().parse().ok()?;
    Some((h, s, turn))
}

pub fn edges_of(meta: &Meta) -> Vec<(String, String)> {
    meta.get("edges")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|e| Some((e.get("rel")?.as_str()?.to_string(), e.get("to")?.as_str()?.to_string()))).collect())
        .unwrap_or_default()
}

pub fn parse(text: &str) -> Result<(Meta, String), String> {
    let (mut meta, body) = fm::loads(text)?;
    let mut body = body;
    let idx = if body.starts_with(&format!("{RELATED}\n")) { Some(0) } else { body.find(&format!("\n{RELATED}\n")) };
    if let Some(i) = idx {
        body.truncate(i);
    }
    let mut edges = Vec::new();
    if let Some(Value::Array(legacy)) = meta.shift_remove("edges") {
        for e in legacy {
            if let (Some(rel), Some(to)) = (e.get("rel").filter(|v| truthy(Some(v))), e.get("to").filter(|v| truthy(Some(v)))) {
                edges.push(json!({"rel": rel, "to": short(&value_text(to))}));
            }
        }
    }
    for rel in RELATIONS {
        let Some(values) = meta.shift_remove(rel) else { continue };
        if values.is_null() {
            continue;
        }
        let list = match values {
            Value::Array(a) => a,
            other => vec![other],
        };
        for v in list {
            if truthy(Some(&v)) {
                edges.push(json!({"rel": rel, "to": short(&value_text(&v))}));
            }
        }
    }
    for key in DERIVED {
        meta.shift_remove(key);
    }
    meta.insert("edges".into(), Value::Array(edges));
    let trimmed = body.trim();
    Ok((meta, if trimmed.is_empty() { String::new() } else { format!("{trimmed}\n") }))
}

pub fn render(meta: &Meta, body: &str, path: &Path, locate: &dyn Fn(&str) -> Option<PathBuf>) -> String {
    let mut out = Meta::new();
    let title = meta.get("title").and_then(Value::as_str).unwrap_or("").to_string();
    let status = if truthy(meta.get("valid_to")) {
        "superseded"
    } else if truthy(meta.get("retracted_at")) {
        "retracted"
    } else {
        "active"
    };
    let ty = meta.get("type").map(value_text).unwrap_or_else(|| "None".into());
    for key in ORDER {
        match key {
            "aliases" => {
                out.insert(key.into(), if title.is_empty() { json!([]) } else { json!([title]) });
            }
            "tags" => {
                out.insert(key.into(), json!([format!("rewire/{ty}")]));
            }
            "status" => {
                out.insert(key.into(), json!(status));
            }
            "source" => {
                if truthy(meta.get("source")) {
                    if let Some(s) = source_text(&meta["source"]) {
                        out.insert(key.into(), json!(s));
                    }
                }
            }
            _ => {
                if let Some(v) = meta.get(key) {
                    out.insert(key.into(), v.clone());
                }
            }
        }
    }
    let mut related: Vec<String> = Vec::new();
    let mut by_rel: indexmap::IndexMap<String, Vec<Value>> = indexmap::IndexMap::new();
    let dir = path.parent().unwrap_or(Path::new("."));
    for (rel, to) in edges_of(meta) {
        let target = locate(&short(&to));
        let stem = match &target {
            Some(t) => t.file_stem().unwrap().to_string_lossy().into_owned(),
            None => short(&to),
        };
        by_rel.entry(rel.clone()).or_default().push(json!(format!("[[{stem}]]")));
        match &target {
            Some(t) => {
                let label = title_of(t).unwrap_or_else(|| stem.clone()).replace('[', "(").replace(']', ")");
                related.push(format!("- {rel}: [{label}]({})", relpath(t, dir)));
            }
            None => related.push(format!("- {rel}: {stem} (missing)")),
        }
    }
    for (rel, links) in by_rel {
        out.insert(rel, Value::Array(links));
    }
    for (k, v) in meta {
        if !ORDER.contains(&k.as_str()) && k != "edges" && !RELATIONS.contains(&k.as_str()) {
            out.insert(k.clone(), v.clone());
        }
    }
    let mut text = body.trim().to_string();
    if !related.is_empty() {
        if !text.is_empty() {
            text.push_str("\n\n");
        }
        text.push_str(RELATED);
        text.push('\n');
        text.push_str(&related.join("\n"));
    }
    fm::dumps(&out, &text)
}

fn title_of(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(2048)]).into_owned();
    static R: OnceLock<Regex> = OnceLock::new();
    let re = R.get_or_init(|| Regex::new(r"(?m)^title:[ \t]*(.+)$").unwrap());
    let raw = re.captures(&head)?[1].trim().to_string();
    if raw.len() >= 2 && raw.starts_with('"') && raw.ends_with('"') {
        return Some(match serde_json::from_str::<Value>(&raw) {
            Ok(Value::String(s)) => s,
            _ => raw.trim_matches('"').to_string(),
        });
    }
    Some(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shorts() {
        assert_eq!(short("[[melanie-s-trip-4mps24m3]]"), "4mps24m3");
        assert_eq!(short("01M4NA819FA9T8ZE375KF00VZF"), "5kf00vzf");
        assert_eq!(short("../fact/x-abcdefgh.md"), "abcdefgh");
    }

    #[test]
    fn sources() {
        assert_eq!(
            source_text(&json!({"harness": "codex", "session": "abcdef123456", "turn": 4})).unwrap(),
            "codex · abcdef123456 · turn 4"
        );
        assert_eq!(
            parse_source(Some(&json!("locomo · conv-26-session_4 · turn 3"))),
            Some(("locomo".into(), "conv-26-session_4".into(), 3))
        );
        assert_eq!(parse_source(Some(&json!("x · y"))), None);
    }
}
