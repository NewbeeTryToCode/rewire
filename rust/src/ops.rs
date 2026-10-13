//! High-level operations shared by the CLI, the MCP server and consolidation.

use crate::brain::{err_str, is_set, out_str, Brain, Node};
use crate::index::{Hit, Index, SearchOpts};
use crate::util::{now_iso, relpath, short_of, take};
use crate::{project, schema, security, skills, Error, Meta, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

fn op<T>(msg: impl Into<String>) -> Result<T> {
    Err(Error::Op(msg.into()))
}

pub fn resolve_id(brain: &Brain, reference: &str) -> Option<Node> {
    let r = reference.trim().trim_matches(|c| c == '[' || c == ']');
    if r.chars().count() == 26 {
        return brain.get_node(&r.to_uppercase());
    }
    let r = r.to_lowercase();
    if r.chars().count() == 8 {
        // file names end with the last 8 id chars: no full scan needed
        let hits: Vec<Node> = brain
            .iter_node_files_with_suffix(&r)
            .into_iter()
            .filter_map(|p| brain.read_node(&p))
            .filter(|n| n.id().to_lowercase().ends_with(&r))
            .collect();
        return if hits.len() == 1 { hits.into_iter().next() } else { None };
    }
    let matches: Vec<Node> = brain.iter_nodes().into_iter().filter(|n| n.id().to_lowercase().ends_with(&r)).collect();
    if matches.len() == 1 { matches.into_iter().next() } else { None }
}

#[allow(clippy::too_many_arguments)]
pub fn recall(
    brain: &Brain,
    index: &mut Index,
    query: &str,
    k: usize,
    cwd: Option<&str>,
    scope: Option<&str>,
    beta: Option<f64>,
    episodes: usize,
    refers_to: Option<&Value>,
) -> Result<Vec<Hit>> {
    // `refers_to` {from, to}: the time the question is about, resolved by the calling model ("last week",
    // "kemarin"); it boosts notes and excerpts dated inside it (a boost, not a filter).
    let window = time_range(refers_to)?;
    let scopes = match scope.filter(|s| !s.is_empty()) {
        Some("all") => None,
        Some(s) => Some(vec![s.to_string()]),
        None => Some(project::scopes_for(project::detect(cwd, &brain.config.projects).as_deref())),
    };
    let mut hits = index.search(query, &SearchOpts { scopes, k, beta, window: window.clone(), ..Default::default() })?;
    for hit in hits.iter_mut() {
        // re-scanned when shown: files can be edited by hand or arrive by sync
        let why = security::blocked(&format!("{} {}", hit.title, hit.snippet));
        if !why.is_empty() {
            hit.title = security::blocked_label(why);
            hit.snippet = String::new();
        }
    }
    if episodes > 0 {
        // raw excerpts after the distilled facts; the facts found pull in the excerpts they came from
        let eps = index.search_episodes(query, episodes, &hits, window.as_ref())?;
        hits.extend(eps);
    }
    Ok(hits)
}

pub fn time_range(refers_to: Option<&Value>) -> Result<Option<(String, String)>> {
    let Some(r) = refers_to.filter(|v| !v.is_null() && v.as_object().is_none_or(|o| !o.is_empty())) else { return Ok(None) };
    let Some(o) = r.as_object() else { return op("refers_to must be an object {from, to}") };
    let field = |k: &str| o.get(k).and_then(Value::as_str).unwrap_or("").chars().take(10).collect::<String>();
    let (mut start, mut end) = (field("from"), field("to"));
    if start.is_empty() {
        start = end.clone();
    }
    if end.is_empty() {
        end = start.clone();
    }
    let iso = |s: &str| s.len() == 10 && s.chars().enumerate().all(|(i, c)| if i == 4 || i == 7 { c == '-' } else { c.is_ascii_digit() });
    if !iso(&start) || !iso(&end) {
        return op("refers_to needs ISO dates: {\"from\": \"YYYY-MM-DD\", \"to\": \"YYYY-MM-DD\"}");
    }
    Ok(Some(if start <= end { (start, end) } else { (end, start) }))
}

pub fn event(ev: &str, node: &str, w: f64, session: &str, harness: &str) -> Meta {
    let mut m = Meta::new();
    m.insert("ev".into(), json!(ev));
    m.insert("node".into(), json!(node));
    m.insert("w".into(), json!(w));
    m.insert("session".into(), json!(session));
    m.insert("harness".into(), json!(harness));
    m
}

pub fn get(brain: &Brain, refs: &[String], session: &str, harness: &str) -> Result<Vec<Value>> {
    let mut out = Vec::new();
    let mut events = Vec::new();
    for r in refs.iter().take(10) {
        if r.trim().to_lowercase().starts_with("skill:") {
            let Some(skill) = skills::read(brain, &r.trim()[6..]) else {
                out.push(json!({"ref": r, "error": "not found"}));
                continue;
            };
            let why = skills::blocked(&skill);
            let (description, body) =
                if why.is_empty() { (skill.description.clone(), skill.body.clone()) } else { (security::blocked_label(why), String::new()) };
            out.push(json!({"id": format!("skill:{}", skill.name), "type": "skill", "name": skill.name,
                            "description": description, "updated": skill.updated, "body": body}));
            events.push(event("use", &format!("skill:{}", skill.name), 0.5, session, harness));
            continue;
        }
        match resolve_id(brain, r) {
            None => out.push(json!({"ref": r, "error": "not found"})),
            Some(node) => {
                let mut m = Meta::new();
                m.insert("id".into(), json!(node.id()));
                for (k, v) in &node.meta {
                    if k != "id" {
                        m.insert(k.clone(), v.clone());
                    }
                }
                m.insert("body".into(), json!(node.body));
                let why = security::blocked(&format!("{} {}", node.title(), node.body));
                if !why.is_empty() {
                    m.insert("title".into(), json!(security::blocked_label(why)));
                    m.insert("body".into(), json!(""));
                }
                out.push(Value::Object(m));
                events.push(event("use", node.id(), 0.5, session, harness));
            }
        }
    }
    brain.append_events(&events)?;
    Ok(out)
}

pub fn remember(brain: &Brain, text: &str, type_: &str, scope: &str, provenance: &str, source: Option<Value>) -> Result<String> {
    let text = security::strip_private(text).trim().to_string();
    if text.is_empty() {
        return op("empty memory");
    }
    if !schema::TYPES.contains(&type_) {
        return op(format!("unknown type {type_}; use one of {}", schema::TYPES.join(", ")));
    }
    if schema::CLAIM_TYPES.contains(&type_) && text.chars().count() > 600 {
        return op("a memory must be one self-contained claim (max 600 characters)");
    }
    let problems = security::scan(&text);
    if problems.iter().any(|p| p == "secret") {
        return op("refused: text looks like it contains a secret");
    }
    let mut meta = Meta::new();
    meta.insert("action".into(), json!("NEW"));
    meta.insert("type".into(), json!(type_));
    meta.insert("title".into(), json!(take(&text, 160)));
    meta.insert("scope".into(), json!(scope));
    meta.insert("importance".into(), json!(3));
    meta.insert("confidence".into(), json!(if provenance == "user" { "explicit" } else { "inferred" }));
    meta.insert("provenance".into(), json!(provenance));
    meta.insert("reason".into(), json!("remember()"));
    meta.insert("flags".into(), json!(problems));
    meta.insert("source".into(), source.unwrap_or_else(|| json!({})));
    meta.insert("created".into(), json!(now_iso()));
    brain.write_inbox(&meta, &text)
}

// ---- applying changes -----------------------------------------------------

pub fn ensure_project_hub(brain: &Brain, slug: &str) -> Result<Node> {
    let scope = format!("project:{slug}");
    if let Ok(entries) = fs::read_dir(brain.path(&["nodes", "project"])) {
        let mut paths: Vec<_> = entries.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "md")).collect();
        paths.sort();
        for p in paths {
            if let Some(n) = brain.read_node(&p) {
                if n.meta.get("scope").and_then(Value::as_str) == Some(scope.as_str()) && n.active() {
                    return Ok(n);
                }
            }
        }
    }
    let mut fields = Meta::new();
    fields.insert("scope".into(), json!(scope));
    fields.insert("importance".into(), json!(3));
    fields.insert("confidence".into(), json!("explicit"));
    fields.insert("provenance".into(), json!("user"));
    let meta = brain.new_meta("project", slug, fields);
    brain.write_node(&meta, "", None)
}

fn truthy(v: Option<&Value>) -> bool {
    is_set(v) && !matches!(v, Some(Value::Array(a)) if a.is_empty()) && !matches!(v, Some(Value::Number(n)) if n.as_f64() == Some(0.0))
}

/// Apply NEW / UPDATE / SUPERSEDE / CONTRADICT. Returns the written node.
pub fn apply_change(brain: &Brain, change: &Meta, source: Option<&Value>) -> Result<Node> {
    let s = |k: &str| change.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let action = if truthy(change.get("action")) { s("action") } else { "NEW".into() };
    let text = s("text").trim().to_string();
    let title = take(&if truthy(change.get("title")) { s("title") } else { text.clone() }, 160);
    let scope = if truthy(change.get("scope")) { s("scope") } else { "global".into() };
    let mut fields = Meta::new();
    for k in ["importance", "confidence", "provenance", "keywords", "valid_from"] {
        if truthy(change.get(k)) {
            fields.insert(k.into(), change[k].clone());
        }
    }
    let mut edges: Vec<Value> = change
        .get("edges")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter(|e| {
                    e.get("rel").and_then(Value::as_str).is_some_and(|r| schema::RELATIONS.contains(&r)) && truthy(e.get("to"))
                })
                .map(|e| json!({"rel": e["rel"], "to": e["to"]}))
                .collect()
        })
        .unwrap_or_default();
    if let Some(slug) = scope.strip_prefix("project:") {
        let hub = ensure_project_hub(brain, slug)?;
        if s("type") != "project" {
            edges.push(json!({"rel": "about", "to": hub.id()}));
        }
    }
    let source_value = source.filter(|v| truthy(Some(v))).cloned();

    if action == "UPDATE" {
        let Some(target) = resolve_id(brain, &s("target")) else {
            return op(format!("update target not found: {}", s("target")));
        };
        let mut meta = target.meta.clone();
        for (k, v) in fields {
            meta.insert(k, v);
        }
        meta.insert("title".into(), json!(title));
        meta.insert("recorded_at".into(), json!(now_iso()));
        let src = source_value.or_else(|| meta.get("source").cloned()).unwrap_or(Value::Null);
        meta.insert("source".into(), src);
        return brain.write_node(&meta, &text, Some(&target.path));
    }

    let mut f2 = Meta::new();
    f2.insert("scope".into(), json!(scope));
    f2.insert("edges".into(), Value::Array(edges));
    f2.insert("source".into(), source_value.unwrap_or_else(|| json!({})));
    for (k, v) in fields {
        f2.insert(k, v);
    }
    let mut meta = brain.new_meta(&s("type"), &title, f2);
    if action == "SUPERSEDE" || action == "CONTRADICT" {
        let Some(target) = resolve_id(brain, &s("target")) else {
            return op(format!("target not found: {}", s("target")));
        };
        let rel = if action == "SUPERSEDE" { "supersedes" } else { "contradicts" };
        if let Some(Value::Array(e)) = meta.get_mut("edges") {
            e.push(json!({"rel": rel, "to": target.id()}));
        }
        let mut old = target.meta.clone();
        old.insert("retracted_at".into(), json!(now_iso()));
        if action == "SUPERSEDE" {
            old.insert("valid_to".into(), json!(now_iso()[..10].to_string()));
        }
        brain.write_node(&old, &target.body, Some(&target.path))?;
    }
    brain.write_node(&meta, &text, None)
}

/// Fingerprint of a note's content, pinned on inbox items that change an existing note.
pub fn content_hash(node: &Node) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(format!("{}\n{}", node.title(), node.body).as_bytes()))[..16].to_string()
}

/// Record what the target looked like when the change was proposed (approve refuses if it moved).
pub fn pin_target(brain: &Brain, item: &mut Meta) {
    let target = item.get("target").and_then(Value::as_str).filter(|t| !t.is_empty()).map(String::from);
    if let Some(node) = target.and_then(|t| resolve_id(brain, &t)) {
        item.insert("target_hash".into(), json!(content_hash(&node)));
    }
}

/// Apply an inbox item. Returns what it became ("[id8] title" or "skill <name>").
pub fn approve(brain: &Brain, inbox_id: &str, force: bool) -> Result<String> {
    let path = brain.path(&["inbox", &format!("{inbox_id}.md")]);
    let Some(item) = brain.iter_inbox().into_iter().find(|i| i.path == path) else {
        return op(format!("inbox item not found: {inbox_id}"));
    };
    if item.meta.get("kind").and_then(Value::as_str) == Some("skill") {
        match skills::apply(brain, &item.meta, &item.body) {
            Err(Error::Op(e)) => return op(format!("{inbox_id}: {e}")),
            other => other?,
        };
        fs::remove_file(&path)?;
        let name = item.meta.get("name").and_then(Value::as_str).unwrap_or("");
        return Ok(format!("skill {}", skills::normalize_name(name)));
    }
    if let (Some(pinned), false) = (item.meta.get("target_hash").and_then(Value::as_str).filter(|h| !h.is_empty()), force) {
        let target = item.meta.get("target").and_then(Value::as_str).and_then(|t| resolve_id(brain, t));
        if target.is_some_and(|t| content_hash(&t) != pinned) {
            return op(format!(
                "{inbox_id}: the note it changes was edited after this was proposed; check it, then approve with --force or reject"
            ));
        }
    }
    let mut change = item.meta.clone();
    change.insert("text".into(), json!(item.body));
    change.insert("confidence".into(), json!("explicit")); // a human approved it
    let node = apply_change(brain, &change, item.meta.get("source"))?;
    fs::remove_file(&path)?;
    Ok(format!("[{}] {}", crate::util::short_of(node.id()), node.title()))
}

pub fn reject(brain: &Brain, inbox_id: &str) -> Result<()> {
    let path = brain.path(&["inbox", &format!("{inbox_id}.md")]);
    if !path.exists() {
        return op(format!("inbox item not found: {inbox_id}"));
    }
    fs::remove_file(path)?;
    Ok(())
}

pub fn undo(brain: &Brain, session: &str) -> Result<String> {
    let log = out_str(&brain.git(&["log", "--format=%H %s", &format!("--grep=consolidate .*{session}")]));
    let Some(sha) = log.lines().next().and_then(|l| l.split_whitespace().next()).map(String::from) else {
        return op(format!("no consolidation commit found for session {session}"));
    };
    let r = brain.git(&["revert", "--no-edit", &sha]);
    if !r.status.success() {
        brain.git(&["revert", "--abort"]);
        return op(format!("git revert failed: {}", err_str(&r)));
    }
    Ok(sha)
}

pub fn doctor(brain: &Brain) -> Vec<String> {
    let mut report = Vec::new();
    let mut bad = 0;
    for node in brain.iter_nodes() {
        if let Err(e) = schema::validate(&node.meta, &node.body) {
            bad += 1;
            report.push(format!("INVALID {}: {}", node.path.display(), e));
        }
        let why = security::blocked(&format!("{} {}", node.title(), node.body));
        if !why.is_empty() {
            report.push(format!("BLOCKED ({why}) {}: hidden from sessions until edited", node.path.display()));
        }
    }
    for skill in skills::all_skills(brain) {
        let why = skills::blocked(&skill);
        if !why.is_empty() {
            report.push(format!("BLOCKED ({why}) {}: hidden from sessions until edited", skill.path.display()));
        }
    }
    for (name, days) in skills::stale(brain, None) {
        report.push(format!("STALE skill {name}: not used for {days} days (delete skills/{name}/ to retire it)"));
    }
    report.push(format!("inbox: {} item(s) awaiting review", brain.iter_inbox().len()));
    let remotes = out_str(&brain.git(&["remote", "-v"]));
    let device = brain.device();
    if !remotes.is_empty() {
        report.push(format!(
            "sync: {} · device {device} · make sure the brain repository is PRIVATE:\n  {}",
            if brain.config.sync_auto { "auto" } else { "manual" },
            remotes.lines().next().unwrap_or("")
        ));
    } else {
        report.push(format!("sync: no remote (this brain lives only on this machine) · device {device}"));
    }
    let failed = brain.read_state("failed");
    if !failed.is_empty() {
        report.push(format!("failed consolidations: {} (see .state/failed.json)", failed.len()));
    }
    let pending = brain.read_state("pending");
    if !pending.is_empty() {
        report.push(format!("pending transcripts: {}", pending.len()));
    }
    let conflicts = out_str(&brain.git(&["diff", "--name-only", "--diff-filter=U"]));
    if !conflicts.is_empty() {
        report.push(format!("unresolved git conflicts:\n  {}", conflicts.replace('\n', "\n  ")));
    }
    report.insert(0, format!("nodes valid: {}", if bad == 0 { "yes".to_string() } else { format!("NO ({bad} invalid)") }));
    report
}

// ---- human-readable home page --------------------------------------------

const HOME_INTRO: &str = "> Generated by [rewire](https://github.com/NewbeeTryToCode/rewire) from the notes in `nodes/`.\n> Edit those notes (or use `rewire review`), not this page: it is rewritten after every change.\n";

fn md_link(brain: &Brain, path: &str, title: &str) -> String {
    let rel = relpath(Path::new(path), &brain.root);
    format!("[{}]({rel})", title.replace('[', "(").replace(']', ")"))
}

/// Write README.md: an overview a person can read on GitHub or in Obsidian. Deterministic (no
/// timestamps, no activation-based order) so it only changes when the memory itself changes.
pub fn write_home(brain: &Brain, index: &mut Index) -> Result<std::path::PathBuf> {
    index.refresh()?;
    let active = index.rows("active = 1 ORDER BY importance DESC, recorded_at DESC, id", &[])?;
    let mut by_type: indexmap::IndexMap<String, Vec<&crate::index::NodeRow>> = indexmap::IndexMap::new();
    for r in &active {
        by_type.entry(r.type_.clone()).or_default().push(r);
    }
    let get = |t: &str| by_type.get(t).cloned().unwrap_or_default();
    let inbox = brain.iter_inbox();
    let counts: Vec<String> =
        schema::TYPES.iter().filter(|t| !get(t).is_empty()).map(|t| format!("{} {t}", get(t).len())).collect();
    let mut lines: Vec<String> = vec![
        "# Brain".into(),
        "".into(),
        HOME_INTRO.into(),
        format!("**{} active memories** — {}", active.len(), if counts.is_empty() { "none yet".into() } else { counts.join(" · ") }),
    ];
    if !inbox.is_empty() {
        lines.push(format!("\n**{} proposal(s) waiting for review** — run `rewire review` (see below).", inbox.len()));
    }
    let section = |lines: &mut Vec<String>, title: &str, rows: &[&crate::index::NodeRow], limit: Option<usize>, describe: &dyn Fn(&crate::index::NodeRow) -> String| {
        if rows.is_empty() {
            return;
        }
        lines.extend(["".into(), format!("## {title}"), "".into()]);
        let n = limit.unwrap_or(rows.len()).min(rows.len());
        for r in &rows[..n] {
            lines.push(format!("- {}{}", md_link(brain, &r.path, &r.title), describe(r)));
        }
        if let Some(l) = limit {
            if rows.len() > l {
                lines.push(format!("- … {} more in `nodes/`", rows.len() - l));
            }
        }
    };
    let scope_of = |r: &crate::index::NodeRow| {
        if r.scope == "global" { String::new() } else { format!(" · {}", r.scope.split_once(':').map(|x| x.1).unwrap_or("")) }
    };
    section(&mut lines, "Rules (always applied)", &get("constraint"), None, &scope_of);
    section(&mut lines, "Preferences", &get("preference"), None, &scope_of);
    let mut hubs = get("project");
    if !hubs.is_empty() {
        lines.extend(["".into(), "## Projects".into()]);
        hubs.sort_by(|a, b| a.title.cmp(&b.title));
        for hub in hubs {
            let members: Vec<&crate::index::NodeRow> =
                active.iter().filter(|r| r.scope == hub.scope && r.type_ != "project").collect();
            lines.extend(["".into(), format!("### {} — {} memories", md_link(brain, &hub.path, &hub.title), members.len()), "".into()]);
            for r in members.iter().take(10) {
                lines.push(format!("- {} · {}", md_link(brain, &r.path, &r.title), r.type_));
            }
            if members.len() > 10 {
                lines.push(format!("- … {} more", members.len() - 10));
            }
        }
    }
    let mut recent: Vec<&crate::index::NodeRow> = active.iter().filter(|r| r.type_ != "project").collect();
    recent.sort_by(|a, b| (b.recorded_at.as_str(), b.id.as_str()).cmp(&(a.recorded_at.as_str(), a.id.as_str())));
    section(&mut lines, "Recently learned", &recent, Some(10), &|r| format!(" · {} · {}", r.type_, take(&r.recorded_at, 10)));
    let globals: Vec<&crate::index::NodeRow> = ["fact", "decision", "concept", "person"]
        .iter()
        .flat_map(|t| get(t))
        .filter(|r| r.scope == "global")
        .collect();
    section(&mut lines, "General knowledge", &globals, Some(20), &|r| format!(" · {}", r.type_));

    let mut sessions: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(days) = fs::read_dir(brain.path(&["sessions"])) {
        for d in days.flatten() {
            if let Ok(files) = fs::read_dir(d.path()) {
                sessions.extend(files.flatten().map(|f| f.path()).filter(|p| p.extension().is_some_and(|e| e == "md")));
            }
        }
    }
    sessions.sort();
    sessions.reverse();
    if !sessions.is_empty() {
        lines.extend(["".into(), "## Recent sessions".into(), "".into()]);
        for p in sessions.iter().take(10) {
            let day = p.parent().and_then(|d| d.file_name()).map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            let stem = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            lines.push(format!("- [{day} · {stem}]({})", relpath(p, &brain.root)));
        }
    }
    if !inbox.is_empty() {
        lines.extend(["".into(), "## Waiting for review".into(), "".into()]);
        for item in inbox.iter().take(20) {
            let m = &item.meta;
            let g = |k: &str| m.get(k).map(crate::cli::py_str).unwrap_or_else(|| "None".into());
            let target = match m.get("target").and_then(Value::as_str).filter(|s| !s.is_empty()) {
                Some(t) => format!(" → `{}`", short_of(t)),
                None => String::new(),
            };
            let action = m.get("action").and_then(Value::as_str).unwrap_or("NEW");
            lines.push(format!("- `{}` {action}{target} · {}: {}", g("inbox_id"), g("type"), g("title")));
        }
        lines.push("\nApprove with `rewire review approve <id>` or reject with `rewire review reject <id>`.".into());
    }
    lines.extend([
        "".into(),
        "## How this brain works".into(),
        "".into(),
        "- `nodes/<type>/` — one memory per file; relations are links you can click.".into(),
        "- `events/` — when memories were used; how strongly each memory is recalled is computed from this.".into(),
        "- `sessions/` — a summary of every consolidated agent session.".into(),
        "- Superseded memories are kept (status `superseded`) so history is never lost.".into(),
        "".into(),
    ]);
    let text = lines.join("\n");
    let path = brain.path(&["README.md"]);
    if fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
        fs::write(&path, text)?;
    }
    Ok(path)
}
