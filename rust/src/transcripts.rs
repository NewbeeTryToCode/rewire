//! Harness transcript parsers -> one shared turn format, plus discovery. The only harness-specific
//! code in rewire. A turn is {n: line number, role: user|assistant|tool, text, ts}.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub const INTERNAL_MARKER: &str = "[rewire-internal]";
const TOOL_CHARS: usize = 300;

#[derive(Debug, Clone)]
pub struct Turn {
    pub n: usize,
    pub role: String,
    pub text: String,
    pub ts: String,
}

#[derive(Debug, Clone, Default)]
pub struct SessionMeta {
    pub session: Option<String>,
    pub cwd: Option<String>,
    pub date: Option<String>,
    pub outputs: Vec<Output>,
}

/// Something the session published outside the transcript (an artifact, a pull request). Its link is kept
/// whatever `transcripts` keeps: the turn that holds it is a tool result, which is cut or dropped.
#[derive(Debug, Clone, PartialEq)]
pub struct Output {
    pub n: usize,
    pub kind: String,
    pub title: String,
    pub url: String,
}

/// (kind, title) of a tool call that publishes something: an artifact publish (not a read, list or asset
/// upload), a pull request opened through the GitHub MCP tools or `gh pr create`.
fn publishing_call(block: &Value) -> Option<(&'static str, String)> {
    let name = block.get("name").and_then(Value::as_str)?;
    let input = block.get("input").cloned().unwrap_or(Value::Null);
    let text = |k: &str| input.get(k).and_then(Value::as_str).unwrap_or("").trim().to_string();
    if name == "Artifact" {
        let action = text("action");
        if !(action.is_empty() || action == "publish") || input.get("asset").and_then(Value::as_bool) == Some(true) {
            return None;
        }
        let stem = Path::new(&text("file_path")).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let title = [text("title"), text("description"), stem].into_iter().find(|t| !t.is_empty()).unwrap_or_default();
        return Some(("artifact", title));
    }
    if name.ends_with("create_pull_request") {
        return Some(("pull request", text("title")));
    }
    if name == "Bash" && text("command").contains("gh pr create") {
        static T: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        let re = T.get_or_init(|| regex::Regex::new(r#"--title[= ]+(?:"([^"]*)"|'([^']*)')"#).unwrap());
        let command = text("command");
        let title = re.captures(&command).and_then(|c| c.get(1).or(c.get(2))).map(|m| m.as_str().to_string());
        return Some(("pull request", title.unwrap_or_default()));
    }
    None
}

/// The link a publishing call's result reports, if it succeeded.
fn published_url(kind: &str, result: &Value) -> Option<String> {
    if result.get("is_error").and_then(Value::as_bool) == Some(true) {
        return None;
    }
    let text = match result.get("content") {
        Some(Value::String(s)) => s.clone(),
        other => blocks_text(other, &["text"]),
    };
    let re = if kind == "artifact" { crate::security::artifact_url() } else { crate::security::pull_url() };
    re.find(&text).map(|m| m.as_str().to_string())
}

/// Drop memory rewire itself injected (snapshot, recall results), wherever it shows up in a transcript:
/// it must never be re-extracted as something the user said. An unclosed fence (cut tool output) runs to the end.
pub fn strip_injected(text: &str) -> String {
    static R: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    R.get_or_init(|| regex::Regex::new(r"(?s)<rewire-memory>.*?(?:</rewire-memory>|\z)").unwrap()).replace_all(text, "").into_owned()
}

/// Remove <system-reminder>…</system-reminder>, <command-*>…</command-*> and <local-command-*>…</…> blocks.
fn clean(text: &str) -> String {
    let stripped = strip_injected(text);
    let mut out = String::new();
    let mut rest = stripped.as_str();
    loop {
        let Some(open) = rest.find('<') else {
            out.push_str(rest);
            break;
        };
        let after = &rest[open + 1..];
        let name_len = after.find('>').unwrap_or(0);
        let name = &after[..name_len];
        let known = name == "system-reminder"
            || name.strip_prefix("command-").is_some_and(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_lowercase() || c == '-'))
            || name.strip_prefix("local-command-").is_some_and(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_lowercase() || c == '-'));
        if name_len > 0 && known {
            let close = format!("</{name}>");
            if let Some(end) = after[name_len + 1..].find(&close) {
                out.push_str(&rest[..open]);
                rest = &after[name_len + 1 + end + close.len()..];
                continue;
            }
        }
        out.push_str(&rest[..open + 1]);
        rest = after;
    }
    out.trim().to_string()
}

fn blocks_text(content: Option<&Value>, kinds: &[&str]) -> String {
    match content {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(blocks)) => blocks
            .iter()
            .filter(|b| b.get("type").and_then(Value::as_str).is_some_and(|t| kinds.contains(&t)))
            .map(|b| b.get("text").and_then(Value::as_str).unwrap_or("").to_string())
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

fn cut(text: &str) -> String {
    if text.chars().count() > TOOL_CHARS {
        format!("{} …", text.chars().take(TOOL_CHARS).collect::<String>())
    } else {
        text.to_string()
    }
}

fn tool_text(content: Option<&Value>) -> String {
    let mut out = Vec::new();
    if let Some(Value::Array(blocks)) = content {
        for b in blocks {
            if b.get("type").and_then(Value::as_str) == Some("tool_result") {
                out.push(match b.get("content") {
                    Some(Value::String(s)) => s.clone(),
                    other => blocks_text(other, &["text"]),
                });
            }
        }
    }
    cut(strip_injected(&out.join("\n")).trim())
}

fn s(v: Option<&Value>) -> Option<String> {
    v.and_then(Value::as_str).filter(|s| !s.is_empty()).map(String::from)
}

pub fn parse_claude_code(lines: &[(usize, String)]) -> (SessionMeta, Vec<Turn>) {
    let mut meta = SessionMeta::default();
    let mut turns = Vec::new();
    // tool_use id -> (kind, title) of calls that publish something, until their result arrives
    let mut publishing: std::collections::HashMap<String, (&'static str, String)> = Default::default();
    for (n, line) in lines {
        let Ok(d) = serde_json::from_str::<Value>(line) else { continue };
        if meta.session.is_none() {
            meta.session = s(d.get("sessionId"));
        }
        if meta.cwd.is_none() {
            meta.cwd = s(d.get("cwd"));
        }
        if d.get("isSidechain").and_then(Value::as_bool) == Some(true) || d.get("isMeta").and_then(Value::as_bool) == Some(true) {
            continue;
        }
        let msg = d.get("message");
        let content = msg.and_then(|m| m.get("content"));
        for b in content.and_then(Value::as_array).into_iter().flatten() {
            match b.get("type").and_then(Value::as_str) {
                Some("tool_use") => {
                    if let (Some(id), Some(call)) = (s(b.get("id")), publishing_call(b)) {
                        publishing.insert(id, call);
                    }
                }
                Some("tool_result") => {
                    let Some((kind, title)) = s(b.get("tool_use_id")).and_then(|id| publishing.remove(&id)) else { continue };
                    if let Some(url) = published_url(kind, b).filter(|u| !meta.outputs.iter().any(|o| &o.url == u)) {
                        meta.outputs.push(Output { n: *n, kind: kind.into(), title, url });
                    }
                }
                _ => {}
            }
        }
        let (text, role) = match d.get("type").and_then(Value::as_str) {
            Some("user") => {
                let is_tool = content
                    .and_then(Value::as_array)
                    .is_some_and(|a| a.iter().any(|b| b.get("type").and_then(Value::as_str) == Some("tool_result")));
                // a compaction summary is written by the model, not said by the user
                let summary = d.get("isCompactSummary").and_then(Value::as_bool) == Some(true);
                if is_tool {
                    (tool_text(content), "tool")
                } else {
                    (clean(&blocks_text(content, &["text"])), if summary { "tool" } else { "user" })
                }
            }
            Some("assistant") => (strip_injected(&blocks_text(content, &["text"])).trim().to_string(), "assistant"),
            _ => continue,
        };
        if !text.is_empty() {
            turns.push(Turn { n: *n, role: role.into(), text, ts: s(d.get("timestamp")).unwrap_or_default() });
        }
    }
    (meta, turns)
}

/// Codex rollout JSONL: {"timestamp", "type", "payload"} per line. Lenient: only message-shaped
/// response items are read.
pub fn parse_codex(lines: &[(usize, String)]) -> (SessionMeta, Vec<Turn>) {
    let mut meta = SessionMeta::default();
    let mut turns = Vec::new();
    for (n, line) in lines {
        let Ok(d) = serde_json::from_str::<Value>(line) else { continue };
        let empty = Value::Null;
        let p = d.get("payload").unwrap_or(&empty);
        match d.get("type").and_then(Value::as_str) {
            Some("session_meta") => {
                meta.session = s(p.get("id")).or(meta.session);
                meta.cwd = s(p.get("cwd")).or(meta.cwd);
                continue;
            }
            Some("response_item") => {}
            _ => continue,
        }
        let ptype = p.get("type").and_then(Value::as_str);
        let role = p.get("role").and_then(Value::as_str);
        let (text, role) = if ptype == Some("message") && matches!(role, Some("user") | Some("assistant")) {
            let text = clean(&blocks_text(p.get("content"), &["input_text", "output_text", "text"]));
            if role == Some("user") && (text.starts_with("<environment_context>") || text.starts_with("<user_instructions>")) {
                continue;
            }
            (text, role.unwrap())
        } else if matches!(ptype, Some("function_call_output") | Some("custom_tool_call_output")) {
            let text = match p.get("output") {
                Some(Value::String(s)) => s.clone(),
                Some(other) => crate::pyjson::dumps(other, crate::pyjson::Opts { ensure_ascii: true, sort_keys: false }),
                None => "null".into(),
            };
            (strip_injected(&text).trim().chars().take(TOOL_CHARS).collect(), "tool")
        } else {
            continue;
        };
        if !text.is_empty() {
            turns.push(Turn { n: *n, role: role.into(), text, ts: s(d.get("timestamp")).unwrap_or_default() });
        }
    }
    (meta, turns)
}

pub fn guess_harness(path: &Path) -> &'static str {
    if path.file_name().is_some_and(|n| n.to_string_lossy().starts_with("rollout-")) {
        "codex"
    } else {
        "claude-code"
    }
}

fn parse(harness: &str, lines: &[(usize, String)]) -> (SessionMeta, Vec<Turn>) {
    if harness == "codex" {
        parse_codex(lines)
    } else {
        parse_claude_code(lines)
    }
}

/// Parse lines after start_line. Returns (harness, meta, turns, last_line).
pub fn read(path: &Path, start_line: usize, harness: Option<&str>) -> std::io::Result<(String, SessionMeta, Vec<Turn>, usize)> {
    let harness = harness.unwrap_or_else(|| guess_harness(path)).to_string();
    let bytes = fs::read(path)?;
    let content = String::from_utf8_lossy(&bytes);
    let lines: Vec<(usize, String)> = content.split_inclusive('\n').enumerate().map(|(i, l)| (i + 1, l.to_string())).collect();
    let (head, _) = parse(&harness, &lines[..lines.len().min(50)]); // session/cwd live near the top
    let rest: Vec<(usize, String)> = lines.iter().filter(|(n, _)| *n > start_line).cloned().collect();
    let (mut meta, turns) = parse(&harness, &rest);
    if meta.session.is_none() {
        meta.session = head.session;
    }
    if meta.cwd.is_none() {
        meta.cwd = head.cwd;
    }
    let last = lines.last().map(|(n, _)| *n).unwrap_or(start_line);
    Ok((harness, meta, turns, last))
}

pub fn is_internal(meta: &SessionMeta, turns: &[Turn], brain_root: &Path) -> bool {
    let cwd = crate::brain::absolute(Path::new(meta.cwd.as_deref().unwrap_or("/")));
    if cwd == brain_root || cwd.starts_with(brain_root) {
        return true;
    }
    turns.iter().find(|t| t.role == "user").is_some_and(|t| t.text.starts_with(INTERNAL_MARKER))
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>, keep: &dyn Fn(&Path) -> bool, recursive: bool) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            if recursive {
                walk(&p, out, keep, true);
            }
        } else if keep(&p) {
            out.push(p);
        }
    }
}

/// Seconds a transcript must have been left untouched before it is consolidated without a SessionEnd: by the idle
/// watcher while the session pauses, or by lazy discovery from a later hook. A session that is still going is
/// picked up when it goes quiet (or at once through its SessionEnd hook), not one fragment at a time.
/// REWIRE_IDLE_SECS overrides it.
pub fn idle_secs() -> f64 {
    std::env::var("REWIRE_IDLE_SECS").ok().and_then(|v| v.parse().ok()).filter(|v: &f64| *v >= 0.0).unwrap_or(600.0)
}

/// Transcripts changed since `since_ts` and idle for `idle_secs()`, and the oldest change time among the ones still
/// active (to look at them again next time).
pub fn discover(since_ts: f64) -> (Vec<PathBuf>, Option<f64>) {
    let claude_dir = crate::brain::expand_user(&std::env::var("CLAUDE_CONFIG_DIR").unwrap_or_else(|_| "~/.claude".into()));
    let codex_dir = crate::brain::expand_user(&std::env::var("CODEX_HOME").unwrap_or_else(|_| "~/.codex".into()));
    let mut paths = Vec::new();
    if let Ok(projects) = fs::read_dir(claude_dir.join("projects")) {
        for p in projects.flatten() {
            walk(&p.path(), &mut paths, &|f| f.extension().is_some_and(|e| e == "jsonl"), false);
        }
    }
    walk(
        &codex_dir.join("sessions"),
        &mut paths,
        &|f| {
            let n = f.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            n.starts_with("rollout-") && n.ends_with(".jsonl")
        },
        true,
    );
    let now = crate::util::now_ts();
    let (idle, mut out, mut active) = (idle_secs(), Vec::new(), None::<f64>);
    for p in paths {
        let Some(mtime) = fs::metadata(&p)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs_f64())
        else {
            continue;
        };
        if mtime < since_ts {
            continue;
        }
        if now - mtime < idle {
            active = Some(active.map_or(mtime, |a: f64| a.min(mtime)));
        } else {
            out.push(p);
        }
    }
    out.sort();
    (out, active)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_tags() {
        assert_eq!(clean("hi <system-reminder>x\ny</system-reminder> there"), "hi  there");
        assert_eq!(clean("<command-name>/x</command-name>keep <b>bold</b>"), "keep <b>bold</b>");
    }
}
