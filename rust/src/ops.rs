//! High-level operations shared by the CLI, the MCP server and consolidation.

use crate::brain::{is_set, out_str, Brain, Node};
use crate::index::{Hit, Index, SearchOpts};
use crate::util::{now_iso, relpath, short_of, take};
use crate::{project, schema, security, skills, text, Error, Meta, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

fn op<T>(msg: impl Into<String>) -> Result<T> {
    Err(Error::Op(msg.into()))
}

/// Scope of a note proposed without one: the project of the working directory, else global. A rule stated
/// while working in a repository is most often about that repository.
pub fn default_scope(brain: &Brain, cwd: Option<&str>) -> String {
    project::detect(cwd, &brain.config.projects).map(|p| format!("project:{p}")).unwrap_or_else(|| "global".into())
}

/// No recalled note or excerpt shares two of the query's content words (one for a one-word query): what came
/// back matched only loosely (vectors, activation), and the memory may simply not hold the answer.
pub fn weak_match(query: &str, hits: &Value) -> bool {
    let need = text::content_words(query).len().min(2);
    if need == 0 {
        return false;
    }
    let texts = hits.as_array().map(|a| a.as_slice()).unwrap_or_default().iter().map(|h| {
        ["title", "snippet", "body", "text"].iter().filter_map(|k| h.get(*k).and_then(Value::as_str)).collect::<Vec<_>>().join(" ")
    });
    !texts.into_iter().any(|t| text::overlap(query, &t) >= need)
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
    if matches.len() == 1 {
        matches.into_iter().next()
    } else {
        None
    }
}

/// Rank fusion of extra queries (standard reciprocal-rank-fusion constant).
use crate::index::RRF_K;
pub const MAX_ALSO: usize = 4;
/// Excerpt budget multiplier for counting / total / ordering questions (text::aggregate).
pub const AGGREGATE_EPISODES: usize = 2;

/// `refers_to` {from, to}: the time the question is about, resolved by the calling model ("last week",
/// "kemarin"); it boosts notes and excerpts dated inside it (a boost, not a filter).
///
/// `also`: extra queries written by the calling model, searched like `query` and fused by rank. A question
/// about a category ("desserts", "cities") is answered by memories naming instances ("ice cream", "Tampa")
/// that share no word with it (measured on LoCoMo multi-hop: experiments/eval).
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
    also: &[String],
) -> Result<Vec<Hit>> {
    let window = time_range(refers_to)?;
    // the evidence is spread over several sessions
    let episodes = if episodes > 0 && text::aggregate(query) { episodes * AGGREGATE_EPISODES } else { episodes };
    let scopes = match scope.filter(|s| !s.is_empty()) {
        Some("all") => None,
        Some(s) => Some(vec![s.to_string()]),
        None => Some(project::scopes_for(project::detect(cwd, &brain.config.projects).as_deref())),
    };
    let mut queries = vec![query.to_string()];
    queries.extend(also.iter().filter(|q| !q.trim().is_empty()).take(MAX_ALSO).cloned());
    // fused score per id, in first-found order; the first query's text and trim win
    let mut fused_hits: Vec<(String, f64)> = Vec::new();
    let mut fused_eps: Vec<(String, f64)> = Vec::new();
    let mut firsts: Vec<Hit> = Vec::new();
    for q in &queries {
        let hits = index.search(q, &SearchOpts { scopes: scopes.clone(), k, beta, window: window.clone(), ..Default::default() })?;
        // raw excerpts after the distilled facts; the facts found pull in the excerpts they came from
        let eps = if episodes > 0 { index.search_episodes(q, episodes, &hits, window.as_ref(), scopes.as_deref())? } else { Vec::new() };
        for (fused, found) in [(&mut fused_hits, hits), (&mut fused_eps, eps)] {
            for (rank, hit) in found.into_iter().enumerate() {
                let add = 1.0 / (RRF_K + rank as f64 + 1.0);
                match fused.iter_mut().find(|(id, _)| *id == hit.id) {
                    Some(entry) => entry.1 += add,
                    None => fused.push((hit.id.clone(), add)),
                }
                if !firsts.iter().any(|f| f.id == hit.id) {
                    firsts.push(hit);
                }
            }
        }
    }
    let order = |id: &str| firsts.iter().position(|f| f.id == id).unwrap_or(usize::MAX);
    let top = |mut fused: Vec<(String, f64)>, n: usize| -> Vec<Hit> {
        if queries.len() > 1 {
            // ties: first found first
            fused.sort_by(|a, b| b.1.total_cmp(&a.1).then(order(&a.0).cmp(&order(&b.0))));
            fused.truncate(n);
        }
        fused.iter().filter_map(|(id, _)| firsts.iter().find(|f| f.id == *id).cloned()).collect()
    };
    let mut hits = top(fused_hits, k);
    // rules and preferences from another device not yet reviewed here are not handed to the agent
    let unreviewed = brain.untrusted_ids();
    if !unreviewed.is_empty() {
        hits.retain(|h| !unreviewed.contains(&h.id));
    }
    let mut eps = top(fused_eps, episodes);
    for hit in hits.iter_mut() {
        // re-scanned when shown: files can be edited by hand or arrive by sync
        let why = security::blocked(&format!("{} {}", hit.title, hit.snippet));
        if !why.is_empty() {
            hit.title = security::blocked_label(why);
            hit.snippet = String::new();
        }
    }
    for ep in eps.iter_mut() {
        // excerpts are re-scanned too: sessions/ arrive by sync and quote untrusted pages
        let why = security::blocked(&format!("{} {}", ep.text, ep.summary.as_deref().unwrap_or("")));
        if !why.is_empty() {
            ep.text = format!("[BLOCKED: {why}; excerpt of {} withheld]", ep.session);
            ep.summary = None;
        }
    }
    hits.extend(eps);
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
            if brain.is_untrusted(&skill.path) {
                out.push(json!({"ref": r, "error": "from another device, not reviewed on this device yet (`rewire review`)"}));
                continue;
            }
            let why = skills::blocked(&skill);
            let (description, body) = if why.is_empty() {
                (skill.description.clone(), skill.body.clone())
            } else {
                (security::blocked_label(why), String::new())
            };
            out.push(json!({"id": format!("skill:{}", skill.name), "type": "skill", "name": skill.name,
                            "description": description, "updated": skill.updated, "body": body}));
            events.push(event("use", &format!("skill:{}", skill.name), 0.5, session, harness));
            continue;
        }
        match resolve_id(brain, r) {
            None => out.push(json!({"ref": r, "error": "not found"})),
            Some(node) if brain.is_untrusted(&node.path) => {
                out.push(json!({"ref": r, "error": "from another device, not reviewed on this device yet (`rewire review`)"}))
            }
            Some(node) => {
                // the fields the format defines, nothing a file adds on its own
                let mut m = Meta::new();
                m.insert("id".into(), json!(node.id()));
                for (k, v) in node.meta.iter().filter(|(k, _)| schema::FIELDS.contains(&k.as_str()) && k.as_str() != "id") {
                    m.insert(k.clone(), v.clone());
                }
                m.insert("body".into(), json!(node.body));
                // every returned value is scanned as the text it is (a JSON dump would turn newlines into "\n")
                let why =
                    string_values(&Value::Object(m.clone())).iter().map(|t| security::blocked(t)).find(|w| !w.is_empty()).unwrap_or("");
                if !why.is_empty() {
                    m = Meta::new();
                    m.insert("id".into(), json!(node.id()));
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

/// The current hub (page) of a project, if it has one.
pub fn project_hub(brain: &Brain, slug: &str) -> Option<Node> {
    let scope = format!("project:{slug}");
    let mut paths: Vec<_> = fs::read_dir(brain.path(&["nodes", "project"]))
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .collect();
    paths.sort();
    paths
        .into_iter()
        .filter_map(|p| brain.read_node(&p))
        .find(|n| n.meta.get("scope").and_then(Value::as_str) == Some(scope.as_str()) && n.active())
}

pub fn ensure_project_hub(brain: &Brain, slug: &str) -> Result<Node> {
    let scope = format!("project:{slug}");
    if let Some(hub) = project_hub(brain, slug) {
        return Ok(hub);
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
    if !["NEW", "SAME", "UPDATE", "SUPERSEDE", "CONTRADICT"].contains(&action.as_str()) {
        return op(format!("unknown action {action:?}"));
    }
    if action == "SAME" {
        // already in memory: approving it confirms the existing note instead of writing a duplicate
        return resolve_id(brain, &s("target")).ok_or_else(|| Error::Op(format!("target not found: {}", s("target"))));
    }
    let text = s("text").trim().to_string();
    let title = take(&if truthy(change.get("title")) { s("title") } else { text.clone() }, 160);
    // every write path, reviewed or not: a secret never becomes a note
    if security::scan(&format!("{title} {text}")).iter().any(|p| p == "secret") {
        return op("refused: text looks like it contains a secret");
    }
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
                .filter(|e| e.get("rel").and_then(Value::as_str).is_some_and(|r| schema::RELATIONS.contains(&r)) && truthy(e.get("to")))
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
        // the new note first: a crash in between leaves both current, never neither
        let node = brain.write_node(&meta, &text, None)?;
        let mut old = target.meta.clone();
        old.insert("retracted_at".into(), json!(now_iso()));
        if action == "SUPERSEDE" {
            old.insert("valid_to".into(), json!(now_iso()[..10].to_string()));
            old.insert("superseded_by".into(), json!(stem_link(&node)));
        }
        brain.write_node(&old, &target.body, Some(&target.path))?;
        return Ok(node);
    }
    brain.write_node(&meta, &text, None)
}

/// `[[<file stem>]]` of a note, as relation keys write it.
fn stem_link(node: &Node) -> String {
    format!("[[{}]]", node.path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default())
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
/// The file of an inbox item. Ids are ULIDs: anything else (a path such as "../nodes/x") is refused, so an id
/// can never name a file outside the inbox.
fn inbox_path(brain: &Brain, inbox_id: &str) -> Result<std::path::PathBuf> {
    let id = inbox_id.to_uppercase();
    let ok = (6..=26).contains(&id.len()) && id.chars().all(|c| c.is_ascii_digit() || c.is_ascii_uppercase());
    if !ok {
        return op(format!(
            "not an inbox id: {inbox_id:?} (ids look like 01M5CR22VG093P636AQMVY10G3; or a unique start of 6 or more characters)"
        ));
    }
    if id.len() == 26 {
        return Ok(brain.path(&["inbox", &format!("{id}.md")]));
    }
    // a unique prefix, as `forget` takes a short id
    let matches: Vec<_> = brain
        .iter_inbox()
        .into_iter()
        .map(|i| i.path)
        .filter(|p| p.file_name().is_some_and(|n| n.to_string_lossy().starts_with(&id)))
        .collect();
    match matches.as_slice() {
        [one] => Ok(one.clone()),
        [] => op(format!("inbox item not found: {inbox_id}")),
        _ => op(format!("{inbox_id} matches {} inbox items: give more of the id", matches.len())),
    }
}

pub fn approve(brain: &Brain, inbox_id: &str, force: bool) -> Result<String> {
    let path = inbox_path(brain, inbox_id)?;
    let Some(item) = brain.iter_inbox().into_iter().find(|i| i.path == path) else {
        return op(format!("inbox item not found: {inbox_id}"));
    };
    if item.meta.get("kind").and_then(Value::as_str) == Some("hub_section") {
        // a project page's section, as the inbox item shows it
        let Some(hub) = item.meta.get("target").and_then(Value::as_str).and_then(|t| resolve_id(brain, t)).filter(Node::active) else {
            return op(format!("{inbox_id}: the project page it updates is gone"));
        };
        let why = security::scan(&item.body);
        if !why.is_empty() && !force {
            return op(format!("{inbox_id}: flagged ({}); check it, then approve with --force or reject", why.join(", ")));
        }
        let name = item.meta.get("section").and_then(Value::as_str).unwrap_or(crate::hubs::NOW);
        let date = item.meta.get("valid_from").and_then(Value::as_str).map(String::from).unwrap_or_else(|| now_iso()[..10].to_string());
        let body = crate::hubs::set_section(&hub.body, name, &date, &item.body);
        brain.write_node(&hub.meta, &body, Some(&hub.path))?;
        fs::remove_file(&path)?;
        return Ok(format!("{name} of [{}] {}", short_of(hub.id()), hub.title()));
    }
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
    // scanned again now: the stored flags are whatever the file says, and inbox files can arrive by sync
    let why = security::scan(&format!("{} {}", item.meta.get("title").and_then(Value::as_str).unwrap_or(""), item.body));
    if !why.is_empty() && !force {
        return op(format!("{inbox_id}: flagged ({}); check it, then approve with --force or reject", why.join(", ")));
    }
    let mut change = item.meta.clone();
    change.insert("text".into(), json!(item.body));
    change.insert("confidence".into(), json!("explicit")); // a human approved it
    let node = apply_change(brain, &change, item.meta.get("source"))?;
    fs::remove_file(&path)?;
    if let Some(key) = brain.review_key(&path) {
        trust(brain, &key)?;
    }
    Ok(format!("[{}] {}", crate::util::short_of(node.id()), node.title()))
}

pub fn reject(brain: &Brain, inbox_id: &str) -> Result<()> {
    let path = inbox_path(brain, inbox_id)?;
    if !path.exists() {
        return op(format!("inbox item not found: {inbox_id}"));
    }
    fs::remove_file(path)?;
    Ok(())
}

/// What a review did to one item: done, left alone on purpose, or not done.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    Done(String),
    Skipped(String),
    Failed(String),
}

/// Approve or reject inbox items, and rules, preferences and skills from other devices, by id or `all`; then
/// rewrite the home page and commit. The one path for `rewire review` in a terminal and for the review commands
/// the user sends in the chat (chat_review): the caller holds the brain lock and has made sure the user asked.
pub fn review(brain: &Brain, action: &str, ids: &[String], force: bool) -> Result<Vec<Outcome>> {
    use crate::cli::{py_str, python_list};
    let mut out = Vec::new();
    // rules, preferences and skills from other devices: approving shows them here, rejecting is not offered
    // (it would delete them on every device; edit or delete the file instead)
    let all = ids.len() == 1 && ids[0] == "all";
    let mut inbox_ids = Vec::new();
    for id in ids {
        if let Some(key) = (!all).then(|| unreviewed_key(brain, id)).flatten() {
            if action == "approve" {
                trust(brain, &key)?;
                out.push(Outcome::Done(format!("approved {key} (from another device)")));
            } else if key.starts_with("nodes/") {
                // rejecting a rule that came from another device forgets it, on every device once synced
                out.push(match forget_unreviewed(brain, &key, "rejected on review") {
                    Ok(msg) => Outcome::Done(format!("rejected {key}: {msg}")),
                    Err(e) => Outcome::Failed(format!("! {key}: {e}")),
                });
            } else {
                out.push(Outcome::Failed(format!("! {key} is a skill from another device: delete its folder to reject it everywhere")));
            }
        } else {
            inbox_ids.push(id.clone());
        }
    }
    if all && action == "approve" {
        for (key, _, why) in unreviewed(brain) {
            if why.is_empty() || force {
                trust(brain, &key)?;
                out.push(Outcome::Done(format!("approved {key} (from another device)")));
            } else {
                out.push(Outcome::Skipped(format!("skipped flagged item {key} (['{why}']): check it, then approve it by name")));
            }
        }
    }
    let ids: Vec<String> = if all {
        // "all" never approves items the scan flagged (injection, secret): those need a look and their id
        // nor proposals that arrived from another device: what they say about themselves is not checked
        let (flagged, clean): (Vec<_>, Vec<_>) = brain.iter_inbox().into_iter().partition(|i| {
            action == "approve"
                && !force
                && (i.meta.get("flags").and_then(Value::as_array).is_some_and(|f| !f.is_empty())
                    || !security::scan(&i.body).is_empty()
                    || brain.is_untrusted(&i.path))
        });
        for item in &flagged {
            out.push(Outcome::Skipped(format!(
                "skipped {} ({}): check it, then approve it by id",
                item.meta.get("inbox_id").map(py_str).unwrap_or_default(),
                if brain.is_untrusted(&item.path) {
                    "from another device".to_string()
                } else {
                    format!("flagged {}", item.meta.get("flags").map(python_list).unwrap_or_else(|| "by a new scan".into()))
                }
            )));
        }
        clean.iter().map(|i| i.meta.get("inbox_id").map(py_str).unwrap_or_default()).collect()
    } else {
        inbox_ids
    };
    for id in &ids {
        let r = if action == "approve" {
            approve(brain, id, force).map(|what| format!("approved {id} → {what}"))
        } else {
            reject(brain, id).map(|_| format!("rejected {id}"))
        };
        match r {
            Ok(done) => out.push(Outcome::Done(done)),
            Err(Error::Op(e)) | Err(Error::Schema(e)) => out.push(Outcome::Failed(format!("! {e}"))),
            Err(e) => return Err(e),
        }
    }
    write_home(brain, &mut Index::new(brain))?;
    brain.commit(&format!("review {action} {} item(s)", ids.len()));
    Ok(out)
}

/// Fold notes that say the same thing into one: each of `drops` becomes superseded by `keep` (kept in history,
/// out of recall and the snapshot, pointing to `keep`), and `keep` takes over their search keywords and, when the
/// user gives one, a merged `text` that says what all of them said. Notes only, of one type: two
/// proposals in the inbox are settled by approving one and rejecting the other. The caller holds the brain lock
/// and has made sure the user asked.
pub fn merge(brain: &Brain, keep: &str, drops: &[String], text: Option<&str>) -> Result<Vec<Outcome>> {
    let Some(kept) = resolve_id(brain, keep).filter(Node::active) else {
        return Ok(vec![Outcome::Failed(format!("! {keep}: no current note with this id"))]);
    };
    // the merged text the user gave, checked like any note before anything changes
    let text = text.map(str::trim).filter(|t| !t.is_empty());
    if let Some(t) = text {
        let why = security::scan(t);
        if !why.is_empty() {
            return Ok(vec![Outcome::Failed(format!("! the merged text is flagged ({}): nothing merged", why.join(", ")))]);
        }
        if let Err(e) = schema::validate(&kept.meta, t) {
            return Ok(vec![Outcome::Failed(format!("! the merged text: {e}; nothing merged"))]);
        }
    }
    let type_of = |n: &Node| n.meta.get("type").and_then(Value::as_str).unwrap_or("").to_string();
    let mut out = Vec::new();
    let mut meta = kept.meta.clone();
    let mut keywords: Vec<Value> = meta.get("keywords").and_then(Value::as_array).cloned().unwrap_or_default();
    let mut merged = 0;
    for id in drops {
        let Some(old) = resolve_id(brain, id).filter(Node::active) else {
            out.push(Outcome::Failed(format!("! {id}: no current note with this id")));
            continue;
        };
        if old.id() == kept.id() || type_of(&old) != type_of(&kept) || type_of(&old) == "project" {
            out.push(Outcome::Failed(format!("! {id}: only another note of the same type ({}) can be merged into it", type_of(&kept))));
            continue;
        }
        let mut gone = old.meta.clone();
        gone.insert("retracted_at".into(), json!(now_iso()));
        gone.insert("valid_to".into(), json!(now_iso()[..10].to_string()));
        gone.insert("superseded_by".into(), json!(stem_link(&kept)));
        brain.write_node(&gone, &old.body, Some(&old.path))?;
        if let Some(Value::Array(e)) = meta.get_mut("edges") {
            e.push(json!({"rel": "supersedes", "to": old.id()}));
        }
        for k in old.meta.get("keywords").and_then(Value::as_array).into_iter().flatten() {
            if !keywords.contains(k) && keywords.len() < 24 {
                keywords.push(k.clone());
            }
        }
        out.push(Outcome::Done(format!("merged [{}] {} into [{}] {}", short_of(old.id()), old.title(), short_of(kept.id()), kept.title())));
        merged += 1;
    }
    if merged > 0 {
        meta.insert("keywords".into(), Value::Array(keywords));
        let body = match text {
            Some(t) => {
                meta.insert("recorded_at".into(), json!(now_iso()));
                out.push(Outcome::Done(format!("[{}] now says: {t}", short_of(kept.id()))));
                t.to_string()
            }
            None => kept.body.clone(),
        };
        brain.write_node(&meta, &body, Some(&kept.path))?;
        write_home(brain, &mut Index::new(brain))?;
        brain.commit(&format!("merge {merged} note(s) into {}", short_of(kept.id())));
    }
    Ok(out)
}

/// Pairs of current notes (same type and scope) whose words mostly agree, or whose keywords do, likeliest first:
/// candidates for `merge`, shown by `rewire review`. Consolidation writes keywords in English and in the user's
/// language, so the same claim written in two languages is found through them.
pub fn duplicates(brain: &Brain) -> Vec<(Node, Node)> {
    let notes: Vec<Node> =
        brain.iter_nodes().into_iter().filter(|n| n.active() && n.meta.get("type").and_then(Value::as_str) != Some("project")).collect();
    let text = |n: &Node| {
        let kw = n.meta.get("keywords").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(" "));
        format!("{} {} {}", n.title(), n.body, kw.unwrap_or_default())
    };
    // each note's words worked out once: the pairs are many
    let words: Vec<(Vec<String>, std::collections::HashSet<String>)> =
        notes.iter().map(|n| (text::content_words(&text(n)), text::tokens(&text(n)))).collect();
    let keywords: Vec<std::collections::HashSet<String>> = notes
        .iter()
        .map(|n| {
            n.meta
                .get("keywords")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(|k| k.trim().to_lowercase())
                .collect()
        })
        .collect();
    let mut pairs = Vec::new();
    for i in 0..notes.len().min(1000) {
        for j in i + 1..notes.len().min(1000) {
            let same = |k: &str| notes[i].meta.get(k) == notes[j].meta.get(k);
            let ((wa, ta), (wb, tb)) = (&words[i], &words[j]);
            if !same("type") || !same("scope") || wa.len() < 3 || wb.len() < 3 {
                continue;
            }
            // the same search terms (8-15 a note, in both languages): the same claim, whatever language its text is in
            let (ka, kb) = (&keywords[i], &keywords[j]);
            if ka.len() >= 4 && kb.len() >= 4 {
                let shared = ka.intersection(kb).count() as f64 / ka.len().min(kb.len()) as f64;
                if shared >= 0.6 {
                    pairs.push((shared, i, j));
                    continue;
                }
            }
            // most pairs share almost no word: skip them before the stem-aware count
            if wa.iter().filter(|w| tb.contains(*w)).count() * 4 < wa.len().min(wb.len()) {
                continue;
            }
            let score = (text::overlap_of(wa, tb) as f64 / wa.len() as f64).min(text::overlap_of(wb, ta) as f64 / wb.len() as f64);
            if score >= 0.5 {
                pairs.push((score, i, j));
            }
        }
    }
    pairs.sort_by(|x, y| y.0.partial_cmp(&x.0).unwrap_or(std::cmp::Ordering::Equal));
    pairs.into_iter().take(20).map(|(_, i, j)| (notes[i].clone(), notes[j].clone())).collect()
}

/// Take one session back out of memory, whether its changes were applied at consolidation or approved later from
/// the inbox: notes it created are removed, notes it updated return to their content before it, notes it
/// superseded or contradicted become current again, and its pending proposals and session record are dropped.
/// Works on the files (and git history for earlier versions), so later sessions' changes stay.
pub fn undo(brain: &Brain, session: &str) -> Result<String> {
    let of_session = |v: Option<&Value>| crate::nodefmt::parse_source(v).is_some_and(|(_, s, _)| s == session);
    let mut dropped = 0;
    for item in brain.iter_inbox() {
        if of_session(item.meta.get("source")) {
            fs::remove_file(&item.path)?;
            dropped += 1;
        }
    }
    let nodes = brain.iter_nodes();
    let mine: Vec<&Node> = nodes.iter().filter(|n| of_session(n.meta.get("source"))).collect();
    let ids: std::collections::HashSet<&str> = mine.iter().map(|n| n.id()).collect();
    let slug = crate::util::slugify(session, 40);
    let records: Vec<std::path::PathBuf> = fs::read_dir(brain.path(&["sessions"]))
        .map(|days| {
            days.flatten()
                .flat_map(|d| fs::read_dir(d.path()).map(|f| f.flatten().map(|e| e.path()).collect::<Vec<_>>()).unwrap_or_default())
                .filter(|p| p.file_name().is_some_and(|n| n.to_string_lossy().ends_with(&format!("-{slug}.md"))))
                .collect()
        })
        .unwrap_or_default();
    // notes this session wrote that a later session has changed since: left as they are, and reported
    let marker = format!(" · {session} · turn");
    let touched = out_str(&brain.git(&["log", "--format=", "--name-only", &format!("-G{}", regex::escape(&marker)), "--", "nodes"]));
    let mut later: Vec<String> = touched
        .lines()
        .filter(|l| !l.is_empty())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .filter_map(|rel| brain.read_node(&brain.root.join(rel)))
        .filter(|n| !of_session(n.meta.get("source")))
        .map(|n| format!("[{}] {}", short_of(n.id()), n.title()))
        .collect();
    later.dedup();
    if mine.is_empty() && dropped == 0 && records.is_empty() && later.is_empty() {
        return op(if sessions(brain).iter().any(|(_, _, s, _)| s == session) {
            format!("nothing in memory comes from session {session}")
        } else {
            format!("no session {session} (`rewire sessions` lists them)")
        });
    }
    if mine.is_empty() && dropped == 0 && records.iter().all(|p| fs::read_to_string(p).is_ok_and(|t| t.contains("\n## Undone\n"))) {
        return op(format!("session {session} was already undone"));
    }
    let (mut removed, mut restored, mut reopened) = (0, 0, 0);
    for node in &mine {
        for (rel, to) in crate::nodefmt::edges_of(&node.meta) {
            if rel != "supersedes" && rel != "contradicts" {
                continue;
            }
            let Some(target) = resolve_id(brain, &to) else { continue };
            // still replaced by a note from another session: leave it retired
            let replaced = nodes.iter().any(|n| {
                !ids.contains(n.id())
                    && n.id() != target.id()
                    && !is_set(n.meta.get("retracted_at"))
                    && crate::nodefmt::edges_of(&n.meta).iter().any(|(r, t)| {
                        (r == "supersedes" || r == "contradicts") && resolve_id(brain, t).is_some_and(|x| x.id() == target.id())
                    })
            });
            if ids.contains(target.id()) || replaced {
                continue;
            }
            let mut meta = target.meta.clone();
            meta.insert("retracted_at".into(), Value::Null);
            meta.insert("valid_to".into(), Value::Null);
            brain.write_node(&meta, &target.body, Some(&target.path))?;
            reopened += 1;
        }
        match version_before(brain, &node.path, session) {
            Some(text) => {
                crate::util::atomic_write(&node.path, &text)?;
                restored += 1;
            }
            None => {
                fs::remove_file(&node.path)?;
                removed += 1;
            }
        }
    }
    for path in &records {
        // the record stays as the audit trail of what the session did and when it was undone; its transcript and
        // questions go, so recall no longer finds the session
        let text = fs::read_to_string(path)?;
        let mut kept = Vec::new();
        let mut skipping = false;
        for line in text.lines() {
            if line.starts_with("## ") {
                skipping = line == crate::consolidate::TRANSCRIPT_HEADING || line == crate::consolidate::QUESTIONS_HEADING;
            }
            if !skipping {
                kept.push(line);
            }
        }
        let text = format!("{}\n\n## Undone\n\n{} (transcript removed)\n", kept.join("\n").trim_end(), now_iso());
        crate::util::atomic_write(path, &text)?;
    }
    write_home(brain, &mut Index::new(brain))?;
    brain.commit(&format!("undo session {session}"));
    let mut summary = format!(
        "{removed} note(s) removed, {restored} restored to their earlier version, {reopened} made current again, \
         {dropped} proposal(s) dropped, {} transcript(s) taken out of recall",
        records.len()
    );
    if !later.is_empty() {
        summary.push_str(&format!(
            "\nleft as they are, changed by a later session since (check them; `rewire undo` that session too if needed):\n  {}",
            later.join("\n  ")
        ));
    }
    Ok(summary)
}

/// The record file of a session, if one was written.
pub fn session_record(brain: &Brain, harness: &str, session: &str) -> Option<std::path::PathBuf> {
    // by the id in the record's first line: two ids can share a file name ("a.b" and "a-b")
    let name = format!("{harness}-{}.md", crate::util::slugify(session, 40));
    let header = crate::consolidate::record_header(harness, session);
    fs::read_dir(brain.path(&["sessions"]))
        .ok()?
        .flatten()
        .map(|d| d.path().join(&name))
        .find(|p| fs::read_to_string(p).is_ok_and(|t| t.lines().any(|l| l == header)))
}

/// Whether a session's record is consolidated (Some(true)) or transcript only (Some(false)); None without a record.
pub fn session_consolidated(brain: &Brain, harness: &str, session: &str) -> Option<bool> {
    let path = session_record(brain, harness, session)?;
    let text = fs::read_to_string(path).ok()?;
    let header = crate::consolidate::record_header(harness, session);
    let mine: Vec<&str> = text.split(crate::consolidate::RECORD_SEPARATOR).filter(|b| b.lines().next() == Some(header.as_str())).collect();
    Some(mine.iter().any(|b| !b.lines().any(|l| l == crate::consolidate::TRANSCRIPT_ONLY)))
}

/// Session records, newest first: (date, harness, session id, state: "", "undone" or "transcript only").
pub fn sessions(brain: &Brain) -> Vec<(String, String, String, String)> {
    let mut out = Vec::new();
    for day in fs::read_dir(brain.path(&["sessions"])).into_iter().flatten().flatten() {
        for file in fs::read_dir(day.path()).into_iter().flatten().flatten() {
            let Ok(text) = fs::read_to_string(file.path()) else { continue };
            // "# <harness> session <id>"
            let Some((harness, session)) = text.lines().next().and_then(|l| l.strip_prefix("# ")).and_then(|l| l.split_once(" session "))
            else {
                continue;
            };
            let state = if text.contains("\n## Undone\n") {
                "undone"
            } else if text.lines().any(|l| l == crate::consolidate::TRANSCRIPT_ONLY) {
                "transcript only"
            } else {
                ""
            };
            out.push((day.file_name().to_string_lossy().into_owned(), harness.to_string(), session.trim().to_string(), state.to_string()));
        }
    }
    out.sort_by(|a, b| b.cmp(a));
    out
}

/// Every string inside a JSON value, as written.
fn string_values(v: &Value) -> Vec<String> {
    match v {
        Value::String(s) => vec![s.clone()],
        Value::Array(a) => a.iter().flat_map(string_values).collect(),
        Value::Object(o) => o.iter().flat_map(|(k, v)| std::iter::once(k.clone()).chain(string_values(v))).collect(),
        _ => Vec::new(),
    }
}

/// Retract a note: it stays in its file and in git history, marked retracted, and no longer comes back in recall
/// or the snapshot.
pub fn forget(brain: &Brain, reference: &str, reason: &str) -> Result<String> {
    let Some(node) = resolve_id(brain, reference) else { return op(format!("{reference}: no such note")) };
    forget_node(brain, &node, reference, reason)
}

/// Retract exactly the unreviewed note under a review key (the file that was reviewed, never another note that
/// happens to share its id).
pub fn forget_unreviewed(brain: &Brain, key: &str, reason: &str) -> Result<String> {
    let Some(node) = brain.untrusted_nodes().remove(key) else { return op(format!("{key}: no such unreviewed note")) };
    forget_node(brain, &node, key, reason)
}

fn forget_node(brain: &Brain, node: &Node, reference: &str, reason: &str) -> Result<String> {
    if is_set(node.meta.get("retracted_at")) {
        return op(format!("{reference}: already forgotten"));
    }
    let mut meta = node.meta.clone();
    meta.insert("retracted_at".into(), json!(now_iso()));
    if !reason.is_empty() {
        meta.insert("retracted_reason".into(), json!(reason));
    }
    // an unreviewed key stays: if the retraction is ever reverted, the rule comes back unreviewed (the listing skips
    // retracted notes meanwhile)
    brain.write_node(&meta, &node.body, Some(&node.path))?;
    write_home(brain, &mut Index::new(brain))?;
    brain.commit(&format!("forget {}", short_of(node.id())));
    Ok(format!("forgot [{}] {}", short_of(node.id()), node.title()))
}

/// A note file's content just before `session` first wrote it (from git), or None when the session created it.
fn version_before(brain: &Brain, path: &Path, session: &str) -> Option<String> {
    let rel = relpath(path, &brain.root);
    let log = out_str(&brain.git(&["log", "--reverse", "--format=%H", "--", &rel]));
    for sha in log.lines() {
        let text = out_str(&brain.git(&["show", &format!("{sha}:{rel}")]));
        let Ok((meta, _)) = crate::nodefmt::parse(&text) else { continue };
        if crate::nodefmt::parse_source(meta.get("source")).is_some_and(|(_, s, _)| s == session) {
            let before = brain.git(&["show", &format!("{sha}^:{rel}")]);
            return before.status.success().then(|| String::from_utf8_lossy(&before.stdout).into_owned());
        }
    }
    None
}

/// Rules, preferences and skills from another device not yet reviewed on this one: (key, what it says, why the scan
/// blocks it or "").
pub fn unreviewed(brain: &Brain) -> Vec<(String, String, &'static str)> {
    let mut out = Vec::new();
    let nodes = brain.untrusted_nodes();
    for key in brain.untrusted().keys().filter(|k| !k.starts_with("inbox/")) {
        let (text, why) = if let Some(name) = key.strip_prefix("skills/") {
            match skills::read(brain, name.trim_end_matches('/')) {
                Some(s) => (format!("skill {}: {}", s.name, s.description), skills::blocked(&s)),
                None => ("(skill removed since)".into(), ""),
            }
        } else {
            match nodes.get(key) {
                // retracted since (here or there): nothing left to show, nothing to review
                Some(n) if is_set(n.meta.get("retracted_at")) || is_set(n.meta.get("valid_to")) => continue,
                Some(n) => {
                    let body = n.body.trim();
                    let mut text =
                        if body.is_empty() || body == n.title().trim() { n.title().to_string() } else { format!("{}\n{body}", n.title()) };
                    // fields the format does not define are shown too: nothing in the file goes unseen
                    for (k, v) in n.meta.iter().filter(|(k, _)| !schema::FIELDS.contains(&k.as_str())) {
                        text.push_str(&format!("\n[{k}] {}", crate::util::value_text(v)));
                    }
                    let why = security::blocked(&text);
                    (format!("{}: {}", n.meta.get("type").and_then(Value::as_str).unwrap_or(""), text), why)
                }
                None => ("(note removed since)".into(), ""),
            }
        };
        out.push((key.clone(), text, why));
    }
    out
}

/// The unreviewed key an id names: the key itself, `skill:<name>`, or a note's short id.
pub fn unreviewed_key(brain: &Brain, id: &str) -> Option<String> {
    let id = id.trim();
    let lower = id.to_lowercase();
    let untrusted = brain.untrusted();
    // exact first; a prefix only when it names one item (never "the first that matches")
    if untrusted.contains_key(id) {
        return Some(id.to_string());
    }
    let matches: Vec<&String> = untrusted
        .keys()
        .filter(|k| {
            *k == id
                || lower.strip_prefix("skill:").is_some_and(|n| **k == format!("skills/{n}/"))
                || (lower.len() >= 4
                    && k.starts_with("nodes/")
                    && k.rsplit_once('-').is_some_and(|(_, tail)| tail.trim_end_matches(".md").starts_with(&crate::util::short_of(&lower))))
        })
        .collect();
    match matches.as_slice() {
        [one] => Some((*one).clone()),
        _ => None,
    }
}

/// Show an unreviewed rule, preference or skill to agents on this device from now on.
pub fn trust(brain: &Brain, key: &str) -> Result<()> {
    brain.update_state("untrusted", |u| {
        u.remove(key);
    })
}

/// A remote URL without its credentials (`https://user:token@host` → `https://***@host`).
pub fn redact_url(url: &str) -> String {
    regex::Regex::new(r"(https?://)[^/@\s]*@").unwrap().replace_all(url, "$1***@").into_owned()
}

/// Whether a hosted remote can be read with no credentials at all (a public repository): `git ls-remote` over https
/// with credential helpers and prompts off. None when it cannot tell (a local path, no network, another scheme).
pub fn remote_is_public(url: &str) -> Option<bool> {
    // git@github.com:me/brain.git → https://github.com/me/brain.git
    let https = if let Some(rest) = url.strip_prefix("git@") {
        let (host, path) = rest.split_once(':')?;
        format!("https://{host}/{path}")
    } else if let Some(rest) = url.strip_prefix("ssh://git@") {
        // ssh://git@host:22/me/b → https://host/me/b (an ssh port is not an https one)
        let (host, path) = rest.split_once('/')?;
        format!("https://{}/{path}", host.split(':').next().unwrap_or(host))
    } else if url.starts_with("https://") || url.starts_with("http://") {
        url.to_string()
    } else {
        return None;
    };
    // no user name or token from the URL, and (HOME pointed at an empty folder) no ~/.netrc or global config either
    let https = regex::Regex::new(r"^(https?://)[^/@]*@").unwrap().replace(&https, "$1").into_owned();
    // a fresh folder of its own (never one that already exists: someone else could have put a .netrc there)
    let empty_home = std::env::temp_dir().join(format!("rewire-public-check-{}", crate::util::ulid()));
    fs::create_dir(&empty_home).ok()?;
    let mut cmd = std::process::Command::new("git");
    cmd.args(["-c", "credential.helper=", "ls-remote", "--heads", &https])
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_ASKPASS", "true")
        .env("HOME", &empty_home)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("SSH_ASKPASS");
    let out = crate::util::run_with_timeout(cmd, None, std::time::Duration::from_secs(5));
    let _ = fs::remove_dir_all(&empty_home);
    let out = out.ok()?;
    let err = String::from_utf8_lossy(&out.stderr).to_lowercase();
    if out.status.success() {
        Some(true)
    } else if err.contains("authentication")
        || err.contains("not found")
        || err.contains("could not read")
        || err.contains("403")
        || err.contains("401")
    {
        Some(false)
    } else {
        None // offline, unknown host: say nothing
    }
}

/// Whether a `doctor` line reports a problem (the exit status of `rewire doctor`).
pub fn is_problem(line: &str) -> bool {
    [
        "INVALID",
        "BLOCKED",
        "UNSAFE",
        "IGNORED",
        "nodes valid: NO",
        "failed consolidations",
        "consolidation:",
        "unresolved git conflicts",
        "settings:",
    ]
    .iter()
    .any(|p| line.starts_with(p))
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
        let shown = redact_url(remotes.lines().next().unwrap_or(""));
        report.push(format!(
            "sync: {} · device {device} · make sure the brain repository is PRIVATE:\n  {shown}",
            if brain.config.sync_auto { "auto" } else { "manual" },
        ));
        if let Some(url) = remotes.lines().next().and_then(|l| l.split_whitespace().nth(1)) {
            if remote_is_public(url) == Some(true) {
                report.push("UNSAFE: the brain's remote can be read without credentials: make it private".into());
            }
        }
    } else {
        report.push(format!("sync: no remote (this brain lives only on this machine) · device {device}"));
    }
    if !["command", "anthropic"].contains(&brain.config.provider.as_str()) {
        report
            .push(format!("consolidation: unknown provider {:?} in local.toml (use \"command\" or \"anthropic\")", brain.config.provider));
    }
    if brain.config.provider == "anthropic" && std::env::var("ANTHROPIC_API_KEY").map_or(true, |k| k.trim().is_empty()) {
        report.push("consolidation: provider = \"anthropic\" but ANTHROPIC_API_KEY is not set".into());
    }
    if let Some(sync) = brain.read_state("notices").get("sync") {
        // what the last pull changed, in full (here, for the user; the snapshot only says that something did)
        for n in sync.get("notes").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str) {
            let clean: String = n.chars().map(|c| if c.is_control() { ' ' } else { c }).collect();
            report.push(format!("last sync ({}): {clean}", sync.get("at").and_then(Value::as_str).unwrap_or("")));
        }
    }
    if brain.config.provider == "command" {
        let program = shlex::split(&brain.config.command).and_then(|a| a.into_iter().next()).unwrap_or_default();
        let found = !program.is_empty()
            && (Path::new(&program).is_file()
                || std::env::var_os("PATH").is_some_and(|paths| std::env::split_paths(&paths).any(|d| d.join(&program).is_file())));
        if !found {
            report.push(format!(
                "consolidation: `{program}` is not on PATH, so sessions are not consolidated (set the command in local.toml)"
            ));
        }
    }
    let settings =
        crate::brain::expand_user(&std::env::var("CLAUDE_CONFIG_DIR").unwrap_or_else(|_| "~/.claude".into())).join("settings.json");
    if !fs::read_to_string(&settings).is_ok_and(|t| t.contains(" hook session-start")) {
        report.push("claude code: hooks not installed (`rewire install claude-code --write`)".into());
    }
    if !brain.config.synced_llm_settings.is_empty() {
        report.push(format!(
            "IGNORED brain.toml [consolidation] {}: brain.toml syncs to every device, so what runs the LLM is set per \
             device in local.toml",
            brain.config.synced_llm_settings.join(", ")
        ));
    }
    if let Some(e) = &brain.config.error {
        report.push(format!("settings: could not be read ({e}): the LLM step is off and other settings use defaults until it is fixed"));
    }
    let waiting = unreviewed(brain).len();
    if waiting > 0 {
        report.push(format!("from other devices: {waiting} rule(s), preference(s) or skill(s) wait for review here (`rewire review`)"));
    }
    let found = crate::brain::unsafe_paths(brain, None);
    if !found.is_empty() {
        report.push(format!(
            "UNSAFE: git tracks {}; consolidation will not run until they are gone (git rm --cached, then check \
             local.toml)",
            found.join(", ")
        ));
    }
    let failed = brain.read_state("failed");
    if !failed.is_empty() {
        report.push(format!(
            "failed consolidations: {} (retry with `rewire consolidate --retry`):\n  {}",
            failed.len(),
            failed
                .iter()
                .map(|(path, f)| format!("{path}: {}", f.get("error").and_then(Value::as_str).unwrap_or("")))
                .collect::<Vec<_>>()
                .join("\n  ")
        ));
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
    crate::hubs::refresh(brain)?;
    index.refresh()?;
    let mut active = index.rows("active = 1 ORDER BY importance DESC, recorded_at DESC, id", &[])?;
    let unreviewed = brain.untrusted_ids(); // from another device and not reviewed here: not on the home page either
    active.retain(|r| !unreviewed.contains(&r.id));
    let mut by_type: indexmap::IndexMap<String, Vec<&crate::index::NodeRow>> = indexmap::IndexMap::new();
    for r in &active {
        by_type.entry(r.type_.clone()).or_default().push(r);
    }
    let get = |t: &str| by_type.get(t).cloned().unwrap_or_default();
    let inbox = brain.iter_inbox();
    let counts: Vec<String> = schema::TYPES.iter().filter(|t| !get(t).is_empty()).map(|t| format!("{} {t}", get(t).len())).collect();
    let mut lines: Vec<String> = vec![
        "# Brain".into(),
        "".into(),
        HOME_INTRO.into(),
        format!("**{} active memories** — {}", active.len(), if counts.is_empty() { "none yet".into() } else { counts.join(" · ") }),
    ];
    if !inbox.is_empty() {
        lines.push(format!("\n**{} proposal(s) waiting for review** — run `rewire review` (see below).", inbox.len()));
    }
    let section = |lines: &mut Vec<String>,
                   title: &str,
                   rows: &[&crate::index::NodeRow],
                   limit: Option<usize>,
                   describe: &dyn Fn(&crate::index::NodeRow) -> String| {
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
        if r.scope == "global" {
            String::new()
        } else {
            format!(" · {}", r.scope.split_once(':').map(|x| x.1).unwrap_or(""))
        }
    };
    section(&mut lines, "Rules (always applied)", &get("constraint"), None, &scope_of);
    section(&mut lines, "Preferences", &get("preference"), None, &scope_of);
    let mut hubs = get("project");
    if !hubs.is_empty() {
        lines.extend(["".into(), "## Projects".into()]);
        hubs.sort_by(|a, b| a.title.cmp(&b.title));
        for hub in hubs {
            let members: Vec<&crate::index::NodeRow> = active.iter().filter(|r| r.scope == hub.scope && r.type_ != "project").collect();
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
    let globals: Vec<&crate::index::NodeRow> =
        ["fact", "decision", "concept", "person"].iter().flat_map(|t| get(t)).filter(|r| r.scope == "global").collect();
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
            let g = |k: &str| m.get(k).map(crate::util::value_text).filter(|t| !t.is_empty()).unwrap_or_else(|| "-".into());
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
        "- `nodes/project/` — one page per project: its rules, decisions, facts and latest sessions, kept up to date.".into(),
        "- `events/` — when memories were used (the usage log; experimental ordering by use is computed from it).".into(),
        "- `sessions/` — a summary of every consolidated agent session.".into(),
        "- Superseded memories are kept (status `superseded`) so history is never lost.".into(),
        "".into(),
    ]);
    let text = lines.join("\n");
    let path = brain.path(&["README.md"]);
    if fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
        crate::util::atomic_write(&path, &text)?;
    }
    Ok(path)
}
