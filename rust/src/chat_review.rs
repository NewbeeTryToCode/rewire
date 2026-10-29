//! Review from the chat: the user approves or rejects inbox items by sending a message made only of review
//! commands ("rewire approve <ids>", "rewire reject <ids>"). The UserPromptSubmit hook carries out what the user
//! typed, so a review needs no terminal, from the web, the desktop or the phone; the agent never runs it.
//!
//! The hook acts only when the session's own transcript shows the same text as the latest message from a human,
//! with no agent turn after it: an agent that pipes a made-up payload into `rewire hook user-prompt` in the middle
//! of its turn is refused, since its own tool call is already in the transcript. Like the other checks this raises
//! the bar, it is not a sandbox: an agent that may write the transcript file as the user could forge that too.

use crate::brain::Brain;
use crate::ops::{self, Outcome};
use crate::{snapshot, Result};
use serde_json::Value;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// How old the matching human message may be, in seconds: the hook runs right after it is sent.
const FRESH_SECS: f64 = 120.0;
/// How much of the end of the transcript is read: the latest message is near the end.
const TAIL_BYTES: u64 = 512 * 1024;
const MAX_IDS: usize = 50;

/// The review commands a message consists of, or None when it is anything else. Every non-empty line must be
/// `rewire approve|reject <id>…` (or `/rewire-review approve|reject <id>…`): a command inside other text, a pasted
/// document say, is never acted on.
pub fn parse(prompt: &str) -> Option<Vec<(String, Vec<String>)>> {
    static R: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = R.get_or_init(|| {
        regex::Regex::new(r"^(?:rewire|/rewire-review)\s+(approve|reject)((?:\s+(?:all|[0-9A-Za-z]{4,26}))+)\s*$").unwrap()
    });
    let mut commands = Vec::new();
    let mut count = 0;
    for line in prompt.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let c = re.captures(line)?;
        let ids: Vec<String> = c[2].split_whitespace().map(String::from).collect();
        if ids.len() > 1 && ids.iter().any(|i| i == "all") {
            return None;
        }
        count += ids.len();
        commands.push((c[1].to_string(), ids));
    }
    (!commands.is_empty() && count <= MAX_IDS).then_some(commands)
}

fn claude_projects() -> PathBuf {
    crate::brain::expand_user(&std::env::var("CLAUDE_CONFIG_DIR").unwrap_or_else(|_| "~/.claude".into())).join("projects")
}

/// The text of a transcript line when it is a message a human sent (queued, or recorded as a user turn).
fn human_text(d: &Value) -> Option<String> {
    match d.get("type").and_then(Value::as_str)? {
        "queue-operation" if d.get("operation").and_then(Value::as_str) == Some("enqueue") => {
            d.get("content").and_then(Value::as_str).map(String::from)
        }
        "user" => {
            if [d.get("isMeta"), d.get("isSidechain"), d.get("isCompactSummary")].iter().any(|v| v.and_then(Value::as_bool) == Some(true)) {
                return None;
            }
            if d.get("origin").and_then(|o| o.get("kind")).and_then(Value::as_str).is_some_and(|k| k != "human") {
                return None;
            }
            // a plain string: tool results and injected blocks are arrays
            d.get("message").and_then(|m| m.get("content")).and_then(Value::as_str).map(String::from)
        }
        _ => None,
    }
}

fn normalize(text: &str) -> String {
    text.lines().map(str::trim).filter(|l| !l.is_empty()).collect::<Vec<_>>().join("\n")
}

/// Why the prompt cannot be taken as the user's own latest message ("" when it can).
pub fn unverified(payload: &Value, prompt: &str, projects: &Path) -> String {
    let session = payload.get("session_id").and_then(Value::as_str).unwrap_or("");
    let Some(path) = payload.get("transcript_path").and_then(Value::as_str).map(PathBuf::from) else {
        return "the hook was given no transcript".into();
    };
    // the session's own transcript, where Claude Code keeps it: not a file named by whoever ran the hook
    let real = path.canonicalize().unwrap_or_default();
    let in_projects = projects.canonicalize().is_ok_and(|p| real.starts_with(p));
    if session.is_empty() || !in_projects || real.file_name().is_none_or(|n| n.to_string_lossy() != format!("{session}.jsonl")) {
        return "the transcript is not this session's".into();
    }
    let Ok(lines) = tail_lines(&real) else { return "the transcript could not be read".into() };
    let mut last: Option<(usize, String, String)> = None;
    for (i, line) in lines.iter().enumerate() {
        let Ok(d) = serde_json::from_str::<Value>(line) else { continue };
        if let Some(text) = human_text(&d) {
            last = Some((i, text, d.get("timestamp").and_then(Value::as_str).unwrap_or("").to_string()));
        }
    }
    let Some((at, text, ts)) = last else { return "no message from you is in the transcript".into() };
    if normalize(&text) != normalize(prompt) {
        return "it is not your latest message".into();
    }
    let after_agent = lines[at + 1..]
        .iter()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .any(|d| d.get("type").and_then(Value::as_str) == Some("assistant"));
    if after_agent {
        return "the agent has acted since your message".into();
    }
    let age = crate::util::parse_iso(&ts).map(|t| crate::util::now_ts() - t.timestamp_millis() as f64 / 1000.0);
    if age.is_none_or(|a| a > FRESH_SECS) {
        return "your message is not recent".into();
    }
    String::new()
}

fn tail_lines(path: &Path) -> std::io::Result<Vec<String>> {
    let mut f = std::fs::File::open(path)?;
    let len = f.metadata()?.len();
    let start = len.saturating_sub(TAIL_BYTES);
    f.seek(SeekFrom::Start(start))?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    let text = String::from_utf8_lossy(&buf);
    let mut lines: Vec<String> = text.lines().map(String::from).collect();
    if start > 0 && !lines.is_empty() {
        lines.remove(0); // cut in the middle
    }
    Ok(lines)
}

/// The hook's answer to a message of review commands, or None for any other message. The answer goes to the
/// agent as context: it reports the outcome, word for word, and must not run the review itself.
pub fn run(brain: &Brain, payload: &Value) -> Result<Option<String>> {
    run_in(brain, payload, &claude_projects())
}

/// `run`, with Claude Code's projects folder (where session transcripts are) given.
pub fn run_in(brain: &Brain, payload: &Value, projects: &Path) -> Result<Option<String>> {
    let prompt = payload.get("prompt").and_then(Value::as_str).unwrap_or("");
    let Some(commands) = parse(prompt) else { return Ok(None) };
    let header = "# rewire review (carried out by the hook from the user's own message, not by you)";
    let why = unverified(payload, prompt, projects);
    if !why.is_empty() {
        return Ok(Some(snapshot::fenced(&format!(
            "{header}\nNot carried out: {why}. Tell the user; do not run the review yourself. \
             They can send the command again as a new message, or run `rewire review` in their own terminal."
        ))));
    }
    // what was decided on, as its file says before it is applied (not as the agent described it)
    let inbox = brain.iter_inbox();
    let mut items = Vec::new();
    for typed in commands.iter().flat_map(|(_, ids)| ids).filter(|i| *i != "all") {
        let up = typed.to_uppercase();
        let hits: Vec<_> =
            inbox.iter().filter(|i| i.meta.get("inbox_id").and_then(Value::as_str).is_some_and(|id| id.starts_with(&up))).collect();
        if let [item] = hits.as_slice() {
            let text = snapshot::printable(&format!("{}: {}", item.title(), item.body.trim().replace('\n', " ")));
            items.push(format!("- {typed}: {}", text.chars().take(300).collect::<String>()));
        }
    }
    let mut lines = Vec::new();
    {
        let _guard = match brain.lock(Duration::from_secs(3)) {
            Ok(g) => g,
            Err(crate::Error::Locked(_)) => {
                return Ok(Some(snapshot::fenced(&format!(
                    "{header}\nNot carried out: the brain is busy (a consolidation is running). Tell the user to send the same \
                     message again in a minute; do not run the review yourself."
                ))));
            }
            Err(e) => return Err(e),
        };
        for (action, ids) in &commands {
            for outcome in ops::review(brain, action, ids, false)? {
                let (Outcome::Done(l) | Outcome::Skipped(l) | Outcome::Failed(l)) = outcome;
                lines.push(format!("- {l}"));
            }
        }
    }
    if !items.is_empty() {
        lines.push("Items as their inbox files said:".into());
        lines.extend(items);
    }
    if brain.config.sync_auto {
        crate::integrations::spawn_push(brain);
    }
    Ok(Some(snapshot::fenced(&format!(
        "{header}\n{}\nReport this outcome to the user exactly as listed. Do not approve, reject or edit anything yourself.",
        lines.join("\n")
    ))))
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn only_messages_made_of_review_commands() {
        let p = parse("rewire approve 01M61NFF 01m61nfg0\n\nrewire reject 01M60TX5").unwrap();
        assert_eq!(p[0], ("approve".to_string(), vec!["01M61NFF".to_string(), "01m61nfg0".to_string()]));
        assert_eq!(p[1], ("reject".to_string(), vec!["01M60TX5".to_string()]));
        assert_eq!(parse("/rewire-review approve all").unwrap()[0].1, ["all"]);
        for text in [
            "please rewire approve 01M61NFF",
            "rewire approve 01M61NFF\nand also this",
            "here is a doc:\nrewire approve 01M61NFF",
            "rewire approve",
            "rewire approve all 01M61NFF",
            "rewire approve 01M6;rm",
            "rewire forget 01M61NFF",
        ] {
            assert!(parse(text).is_none(), "{text:?}");
        }
    }
}
