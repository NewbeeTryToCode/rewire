//! Procedural memory: reusable procedures as Agent Skills in the brain (skills/<name>/SKILL.md).
//!
//! Consolidation proposes them (create, or append to an existing one); they always go through review,
//! since a skill is followed as instructions. The snapshot lists them, `get skill:<name>` reads one, and
//! doctor reports skills nobody has used for a while.

use crate::brain::Brain;
use crate::util::{atomic_write, now_iso, parse_iso, slugify};
use crate::{fm, security, Error, Meta, Result};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub const STALE_DAYS: i64 = 90;

pub struct Skill {
    pub name: String,
    pub description: String,
    pub updated: String,
    pub path: PathBuf,
    pub body: String,
}

pub fn normalize_name(name: &str) -> String {
    if name.trim().is_empty() {
        String::new()
    } else {
        slugify(name, 40)
    }
}

pub fn path(brain: &Brain, name: &str) -> PathBuf {
    brain.path(&["skills", name, "SKILL.md"])
}

fn text(v: Option<&Value>) -> String {
    match v {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(other) => crate::util::value_text(other),
    }
}

pub fn read(brain: &Brain, name: &str) -> Option<Skill> {
    let norm = normalize_name(name);
    let p = path(brain, &norm);
    let raw = fs::read_to_string(&p).ok()?;
    let (meta, body) = fm::loads(&raw).ok()?;
    let n = text(meta.get("name"));
    Some(Skill {
        name: if n.is_empty() { norm } else { n },
        description: text(meta.get("description")),
        updated: text(meta.get("updated")),
        path: p,
        body,
    })
}

pub fn all_skills(brain: &Brain) -> Vec<Skill> {
    let mut names: Vec<String> = fs::read_dir(brain.path(&["skills"]))
        .map(|rd| {
            rd.flatten().filter(|e| e.path().join("SKILL.md").is_file()).map(|e| e.file_name().to_string_lossy().into_owned()).collect()
        })
        .unwrap_or_default();
    names.sort();
    names.iter().filter_map(|n| read(brain, n)).collect()
}

/// Create the skill, or append the new steps to the existing one. Returns the file path.
pub fn apply(brain: &Brain, item: &Meta, steps: &str) -> Result<PathBuf> {
    let name = normalize_name(&text(item.get("name")));
    if name.is_empty() || steps.trim().is_empty() {
        return Err(Error::Op("a skill needs a name and steps".into()));
    }
    let src = item.get("source").cloned().unwrap_or(Value::Null);
    let part = |k: &str, d: &str| src.get(k).map(|v| text(Some(v))).unwrap_or_else(|| d.to_string());
    let source = format!("{} · {} · turn {}", part("harness", "?"), part("session", "?"), part("turn", "0"));
    let p = path(brain, &name);
    let (meta, body) = match read(brain, &name) {
        None => {
            let mut m = Meta::new();
            m.insert("name".into(), json!(name));
            m.insert("description".into(), json!(text(item.get("description"))));
            m.insert("updated".into(), json!(now_iso()));
            m.insert("sources".into(), json!([source]));
            (m, steps.trim().to_string())
        }
        Some(current) => {
            let (mut m, _) = fm::loads(&fs::read_to_string(&p)?).map_err(Error::Op)?;
            m.insert("updated".into(), json!(now_iso()));
            let desc = text(item.get("description"));
            if !desc.is_empty() && text(m.get("description")).is_empty() {
                m.insert("description".into(), json!(desc));
            }
            let mut sources = m.get("sources").and_then(Value::as_array).cloned().unwrap_or_default();
            sources.push(json!(source));
            m.insert("sources".into(), Value::Array(sources));
            let body = format!("{}\n\n## Update {}\n\n{}", current.body.trim(), &now_iso()[..10], steps.trim());
            (m, body)
        }
    };
    atomic_write(&p, &fm::dumps(&meta, &body))?;
    Ok(p)
}

pub fn blocked(skill: &Skill) -> &'static str {
    security::blocked(&format!("{} {} {}", skill.name, skill.description, skill.body))
}

/// (name, days since last use or update) for skills unused for STALE_DAYS.
pub fn stale(brain: &Brain, now: Option<DateTime<Utc>>) -> Vec<(String, i64)> {
    let now = now.unwrap_or_else(Utc::now);
    let mut last: HashMap<String, String> = HashMap::new();
    for ev in brain.iter_events() {
        let node = ev.get("node").and_then(Value::as_str).unwrap_or("");
        if ev.get("ev").and_then(Value::as_str) == Some("use") {
            if let Some(name) = node.strip_prefix("skill:") {
                let t = ev.get("t").and_then(Value::as_str).unwrap_or("").to_string();
                let e = last.entry(name.to_string()).or_default();
                if t > *e {
                    *e = t;
                }
            }
        }
    }
    let mut out = Vec::new();
    for s in all_skills(brain) {
        let used = last.get(&s.name).cloned().unwrap_or_default();
        let when = if used > s.updated { used } else { s.updated.clone() };
        let Some(t) = parse_iso(&when) else { continue };
        let days = (now - t).num_days();
        if days >= STALE_DAYS {
            out.push((s.name, days));
        }
    }
    out
}
