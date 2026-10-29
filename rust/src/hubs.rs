//! Project pages. Each project hub (`nodes/project/<slug>-<id8>.md`) carries a block rewritten from the project's
//! notes and sessions on every change: what the project decided, knows and must follow, its latest sessions, and
//! what no longer holds. Deterministic, without an LLM: the same notes always give the same block, so a sync
//! conflict on a hub is settled by rendering it again. The text above the block is the user's (or arrives through
//! review) and is never touched; the index leaves the block out, so search and the session-start snapshot see only
//! that text, as before.

use crate::brain::{is_set, Brain, Node};
use crate::util::{relpath, short_of, take};
use crate::Result;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub const BEGIN: &str = "<!-- rewire:generated (rewritten from this project's notes and sessions: edit those, not this block) -->";
pub const END: &str = "<!-- /rewire:generated -->";
/// Section headings in the order they are shown, per note type.
const SECTIONS: [(&str, &str); 6] = [
    ("constraint", "Rules"),
    ("decision", "Decisions"),
    ("fact", "Facts"),
    ("preference", "Preferences"),
    ("concept", "Concepts"),
    ("person", "People"),
];
const SESSIONS: usize = 5;
const SUPERSEDED: usize = 20;
const SUMMARY_CHARS: usize = 160;

/// The body without its generated block: what a person wrote, which search and the snapshot read.
pub fn strip(body: &str) -> String {
    let Some(start) = body.find(BEGIN) else { return body.to_string() };
    let end = body[start..].find(END).map(|e| start + e + END.len()).unwrap_or(body.len());
    format!("{}{}", &body[..start], &body[end..]).trim().to_string()
}

/// The body with `block` in place of its old generated block (at the end, after what a person wrote).
pub fn with_block(body: &str, block: &str) -> String {
    let own = strip(body);
    if own.is_empty() {
        block.to_string()
    } else {
        format!("{own}\n\n{block}")
    }
}

/// The heading of the section that says where a project stands, kept up to date through review.
pub const NOW: &str = "Now";

/// The text of a `## <name>` section in what a person wrote (heading line excluded), if there is one.
pub fn section(body: &str, name: &str) -> Option<String> {
    let own = strip(body);
    let lines: Vec<&str> = own.lines().collect();
    let start = lines.iter().position(|l| is_heading(l, name))?;
    let end = lines[start + 1..].iter().position(|l| l.starts_with("## ")).map(|e| start + 1 + e).unwrap_or(lines.len());
    Some(lines[start + 1..end].join("\n").trim().to_string())
}

fn is_heading(line: &str, name: &str) -> bool {
    line.strip_prefix("## ").is_some_and(|h| h == name || h.starts_with(&format!("{name} (")))
}

/// The body with its `## <name>` section set to `text` (headed "<name> (as of <date>)"), added after what a person
/// wrote when there is none yet. The rest of the text and the generated block are kept.
pub fn set_section(body: &str, name: &str, date: &str, text: &str) -> String {
    let own = strip(body);
    let block = body.find(BEGIN).map(|s| body[s..].to_string());
    let mut lines: Vec<String> = own.lines().map(String::from).collect();
    let new = vec![format!("## {name} (as of {date})"), String::new(), text.trim().to_string()];
    match lines.iter().position(|l| is_heading(l, name)) {
        Some(start) => {
            let end = lines[start + 1..].iter().position(|l| l.starts_with("## ")).map(|e| start + 1 + e).unwrap_or(lines.len());
            let tail = if end < lines.len() { vec![String::new()] } else { Vec::new() };
            lines.splice(start..end, new.into_iter().chain(tail));
        }
        None => {
            if !lines.is_empty() {
                lines.push(String::new());
            }
            lines.extend(new);
        }
    }
    let own = lines.join("\n").trim().to_string();
    match block {
        Some(b) => with_block(&own, b.trim()),
        None => own,
    }
}

fn link(title: &str, target: &Path, dir: &Path) -> String {
    format!("[{}]({})", title.replace('[', "(").replace(']', ")"), relpath(target, dir))
}

fn date_of(n: &Node) -> String {
    let s = |k: &str| n.meta.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let d = s("valid_from");
    take(&if d.is_empty() { s("recorded_at") } else { d }, 10)
}

/// A project's session records: (day, path, summary), newest first.
fn sessions_of(brain: &Brain, slug: &str) -> Vec<(String, PathBuf, String)> {
    let mut out = Vec::new();
    let line = format!("- project: {slug}");
    for day in std::fs::read_dir(brain.path(&["sessions"])).into_iter().flatten().flatten() {
        for f in std::fs::read_dir(day.path()).into_iter().flatten().flatten() {
            let path = f.path();
            if path.extension().is_none_or(|e| e != "md") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else { continue };
            // the latest run of the session that belongs to this project
            let Some(part) = text.split(crate::consolidate::RECORD_SEPARATOR).filter(|p| p.lines().any(|l| l.trim() == line)).last() else {
                continue;
            };
            let summary = crate::index::session_summary(&format!("\n{part}"));
            let summary = summary.strip_prefix("Summary: ").unwrap_or("").to_string();
            let day = day.file_name().to_string_lossy().into_owned();
            out.push((day, path, summary));
        }
    }
    out.sort_by(|a, b| (&b.0, &b.1).cmp(&(&a.0, &a.1)));
    out
}

/// The generated block of one hub.
pub fn block(brain: &Brain, hub: &Node, nodes: &[Node], hidden: &std::collections::HashSet<String>) -> String {
    let scope = hub.meta.get("scope").and_then(Value::as_str).unwrap_or("");
    let hub_short = short_of(hub.id());
    let dir = hub.path.parent().map(Path::to_path_buf).unwrap_or_default();
    // the project's own notes, and notes elsewhere that say they are about it (a lesson promoted to global)
    let mut members: Vec<&Node> = nodes
        .iter()
        .filter(|n| n.id() != hub.id() && n.meta.get("type").and_then(Value::as_str) != Some("project") && !hidden.contains(n.id()))
        .filter(|n| {
            n.meta.get("scope").and_then(Value::as_str) == Some(scope)
                || crate::nodefmt::edges_of(&n.meta).iter().any(|(rel, to)| rel == "about" && short_of(to) == hub_short)
        })
        .collect();
    let key = |n: &Node| (n.meta.get("recorded_at").and_then(Value::as_str).unwrap_or("").to_string(), n.id().to_string());
    members.sort_by_key(|n| std::cmp::Reverse(key(n)));
    let mut lines = vec![BEGIN.to_string()];
    for (type_, heading) in SECTIONS {
        let rows: Vec<&&Node> =
            members.iter().filter(|n| n.active() && n.meta.get("type").and_then(Value::as_str) == Some(type_)).collect();
        if rows.is_empty() {
            continue;
        }
        lines.extend([String::new(), format!("## {heading}"), String::new()]);
        for n in rows {
            lines.push(format!("- {} · {}", link(n.title(), &n.path, &dir), date_of(n)));
        }
    }
    let slug = scope.strip_prefix("project:").unwrap_or("");
    let sessions = sessions_of(brain, slug);
    if !sessions.is_empty() {
        lines.extend([String::new(), "## Recent sessions".into(), String::new()]);
        for (day, path, summary) in sessions.iter().take(SESSIONS) {
            let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            let cut = summary.chars().count() > SUMMARY_CHARS;
            let summary = crate::snapshot::printable(&summary.chars().take(SUMMARY_CHARS).collect::<String>());
            let summary = if cut { format!("{} …", summary.trim_end()) } else { summary };
            let gist = if summary.is_empty() || summary == "-" { String::new() } else { format!(": {summary}") };
            lines.push(format!("- {}{gist}", link(&format!("{day} · {stem}"), path, &dir)));
        }
        if sessions.len() > SESSIONS {
            lines.push(format!("- … {} earlier", sessions.len() - SESSIONS));
        }
    }
    // superseded, not forgotten: a retracted note without an end of validity was taken back, not replaced
    let old: Vec<&&Node> = members.iter().filter(|n| !n.active() && is_set(n.meta.get("valid_to"))).collect();
    if !old.is_empty() {
        lines.extend([String::new(), format!("<details><summary>No longer valid ({})</summary>", old.len()), String::new()]);
        for n in old.iter().take(SUPERSEDED) {
            let until = take(n.meta.get("valid_to").and_then(Value::as_str).unwrap_or(""), 10);
            lines.push(format!("- ~~{}~~ · until {until}", link(n.title(), &n.path, &dir)));
        }
        lines.extend([String::new(), "</details>".into()]);
    }
    if lines.len() == 1 {
        lines.extend([String::new(), "No notes for this project yet.".into()]);
    }
    lines.extend([String::new(), END.to_string()]);
    lines.join("\n")
}

/// Rewrite the generated block of every active project hub whose notes changed. Returns how many were written.
pub fn refresh(brain: &Brain) -> Result<usize> {
    let nodes = brain.iter_nodes();
    let hidden = brain.untrusted_ids(); // from another device and not reviewed here: not on a page either
    let mut written = 0;
    for hub in nodes.iter().filter(|n| n.meta.get("type").and_then(Value::as_str) == Some("project") && n.active()) {
        let body = with_block(&hub.body, &block(brain, hub, &nodes, &hidden));
        if body.trim() != hub.body.trim() {
            brain.write_node(&hub.meta, &body, Some(&hub.path))?;
            written += 1;
        }
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_block_is_replaced_and_what_a_person_wrote_is_kept() {
        let block = format!("{BEGIN}\n\n## Decisions\n\n- x\n\n{END}");
        assert_eq!(with_block("", &block), block);
        let body = with_block("Goal: own the memory.", &block);
        assert_eq!(body, format!("Goal: own the memory.\n\n{block}"));
        let newer = format!("{BEGIN}\n\n## Facts\n\n- y\n\n{END}");
        assert_eq!(with_block(&body, &newer), format!("Goal: own the memory.\n\n{newer}"));
        assert_eq!(strip(&body), "Goal: own the memory.");
        assert_eq!(strip(&block), "");
    }

    #[test]
    fn the_now_section_is_set_in_place_or_added() {
        let block = format!("{BEGIN}\n\n## Facts\n\n- y\n\n{END}");
        let body = set_section(&with_block("Goal: own the memory.", &block), NOW, "2026-10-29", "- Focus: hubs");
        assert_eq!(body, format!("Goal: own the memory.\n\n## Now (as of 2026-10-29)\n\n- Focus: hubs\n\n{block}"));
        assert_eq!(section(&body, NOW).as_deref(), Some("- Focus: hubs"));
        let mid = "Goal.\n\n## Now (as of 2026-10-01)\n\n- old\n\n## Brief\n\nkeep me";
        assert_eq!(set_section(mid, NOW, "2026-10-29", "- new"), "Goal.\n\n## Now (as of 2026-10-29)\n\n- new\n\n## Brief\n\nkeep me");
        assert_eq!(section("## Nowhere\n\nx", NOW), None);
    }
}
