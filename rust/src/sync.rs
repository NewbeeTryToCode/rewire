//! Sync a brain across machines through an ordinary git remote.
//!
//! - events/: one file per device per month + `merge=union`, so they never conflict.
//! - nodes/: on a conflict the version changed most recently wins; the other version goes to the
//!   inbox so nothing is lost.
//! - anything else: keep ours, and say so.

use crate::brain::{err_str, out_str, Brain};
use crate::util::now_iso;
use crate::{nodefmt, Error, Meta, Result};
use regex::Regex;
use serde_json::{json, Value};
use std::process::Output;
use std::time::Duration;

pub const GIT_TIMEOUT: u64 = 30;

fn git(brain: &Brain, args: &[&str], timeout: u64) -> Result<Output> {
    match crate::brain::run_git(&brain.root, args, Duration::from_secs(timeout)) {
        Err(e) if e.kind() == std::io::ErrorKind::TimedOut => Err(Error::Sync(format!("git {} timed out", args[0]))),
        other => Ok(other?),
    }
}

fn g(brain: &Brain, args: &[&str]) -> Result<Output> {
    git(brain, args, GIT_TIMEOUT)
}

pub fn remote_url(brain: &Brain) -> String {
    match g(brain, &["remote", "get-url", "origin"]) {
        Ok(o) if o.status.success() => out_str(&o),
        _ => String::new(),
    }
}

/// REWIRE_REMOTE, set on an ephemeral machine (a cloud session) to fetch the brain at session start.
pub fn env_remote() -> String {
    std::env::var("REWIRE_REMOTE").map(|s| s.trim().to_string()).unwrap_or_default()
}

fn same_remote(a: &str, b: &str) -> bool {
    let norm = |u: &str| u.trim().trim_end_matches('/').trim_end_matches(".git").to_lowercase();
    !a.trim().is_empty() && norm(a) == norm(b)
}

/// The brain's origin is the one REWIRE_REMOTE names: the user chose it for this machine, so it syncs automatically.
pub fn origin_is_env_remote(brain: &Brain) -> bool {
    let env = env_remote();
    !env.is_empty() && brain.root.join(".git").exists() && same_remote(&env, &remote_url(brain))
}

/// REWIRE_REMOTE_TRUSTED=1: what comes from REWIRE_REMOTE counts as reviewed on this machine. For machines that
/// start empty every session (a cloud VM): their review state cannot outlive the session, so without this every rule
/// would wait for a review that never comes. Set by the user in the environment's settings, like REWIRE_REMOTE.
pub fn remote_trusted(brain: &Brain) -> bool {
    std::env::var("REWIRE_REMOTE_TRUSTED").as_deref() == Ok("1") && origin_is_env_remote(brain)
}

/// Why fetching `url` failed, in words the user can act on. A Claude Code cloud session reaches only the private
/// repositories attached to it: without the brain's repository attached, git asks for a user name it cannot get.
pub fn fetch_failure(url: &str, err: &str) -> String {
    let url = crate::ops::redact_url(url);
    if fetch_denied(err) {
        format!(
            "no access to {url} from this machine (in a Claude Code cloud session, add the repository to the \
             environment's repositories: a session reaches only the private repositories attached to it)"
        )
    } else {
        let first: String = err.lines().find(|l| !l.trim().is_empty()).unwrap_or("").chars().take(200).collect();
        format!("could not fetch {url}: {first}")
    }
}

/// The fetch failed for want of access (not because the repository holds no brain yet, or the network is down).
pub fn fetch_denied(err: &str) -> bool {
    let e = err.to_lowercase();
    ["could not read username", "authentication failed", "terminal prompts disabled", "repository not found", "403"]
        .iter()
        .any(|d| e.contains(d))
}

pub fn branch(brain: &Brain) -> String {
    let b = g(brain, &["rev-parse", "--abbrev-ref", "HEAD"]).map(|o| out_str(&o)).unwrap_or_default();
    if b.is_empty() {
        "main".into()
    } else {
        b
    }
}

pub fn set_remote(brain: &mut Brain, url: &str) -> Result<Vec<String>> {
    if remote_url(brain).is_empty() {
        g(brain, &["remote", "add", "origin", url])?;
    } else {
        g(brain, &["remote", "set-url", "origin", url])?;
    }
    enable_auto(brain)?;
    let mut notes =
        vec![format!("origin = {}; auto sync enabled (pull at session start, push after consolidation)", crate::ops::redact_url(url))];
    if Regex::new(r"github\.com|gitlab\.com|bitbucket\.org").unwrap().is_match(url) {
        notes.push("WARNING: your brain contains personal memory. Make sure this repository is PRIVATE.".into());
    }
    Ok(notes)
}

fn enable_auto(brain: &mut Brain) -> Result<()> {
    let path = brain.path(&["brain.toml"]);
    let _ = brain.device(); // pin the device name locally before the legacy shared value is removed
    let mut text = std::fs::read_to_string(&path)?;
    text = Regex::new(r"(?m)^device\s*=.*\n").unwrap().replace_all(&text, "").into_owned();
    if Regex::new(r"(?m)^\[sync\]").unwrap().is_match(&text) {
        text = Regex::new(r"(?ms)(^\[sync\][^\[]*?^auto\s*=\s*)false").unwrap().replace_all(&text, "${1}true").into_owned();
        if !Regex::new(r"(?ms)^\[sync\][^\[]*?^auto\s*=").unwrap().is_match(&text) {
            text = Regex::new(r"(?m)^\[sync\][ \t]*$").unwrap().replace_all(&text, "[sync]\nauto = true").into_owned();
        }
    } else {
        text = format!("{}\n\n[sync]\nauto = true\n", text.trim_end());
    }
    crate::util::atomic_write(&path, &text)?;
    brain.config.sync_auto = true;
    brain.commit("enable sync");
    Ok(())
}

fn stamp(meta: &Meta) -> String {
    let s = |k: &str| meta.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    std::cmp::max(s("recorded_at"), s("retracted_at"))
}

fn empty_output() -> Output {
    Output { status: crate::brain::failed_status(), stdout: Vec::new(), stderr: Vec::new() }
}

fn stage(brain: &Brain, n: u8, path: &str) -> Option<String> {
    let o = g(brain, &["show", &format!(":{n}:{path}")]).ok()?;
    if o.status.success() {
        Some(String::from_utf8_lossy(&o.stdout).into_owned())
    } else {
        None
    }
}

/// Resolve merge conflicts in place. Returns human-readable notes.
fn resolve(brain: &Brain) -> Result<Vec<String>> {
    let mut notes = Vec::new();
    let listed = g(brain, &["diff", "--name-only", "-z", "--diff-filter=U"])?;
    let conflicted = String::from_utf8_lossy(&listed.stdout).into_owned();
    for path in conflicted.split('\0').filter(|p| !p.is_empty()) {
        let full = brain.root.join(path);
        if path.starts_with("nodes/") {
            match (stage(brain, 2, path), stage(brain, 3, path)) {
                (Some(ours), Some(theirs)) => {
                    let (m_ours, b_ours) = nodefmt::parse(&ours).map_err(Error::Sync)?;
                    let (m_theirs, b_theirs) = nodefmt::parse(&theirs).map_err(Error::Sync)?;
                    // a project page whose two sides differ only in the generated block: either will do, the block
                    // is rendered again from the merged notes right after the pull
                    let page = |m: &Meta| m.get("type").and_then(Value::as_str) == Some("project");
                    if page(&m_ours) && page(&m_theirs) && crate::hubs::strip(&b_ours) == crate::hubs::strip(&b_theirs) {
                        crate::util::atomic_write(&full, &ours)?;
                        g(brain, &["add", "--", path])?;
                        notes.push(format!("{path}: project page, regenerated after the merge"));
                        continue;
                    }
                    // newest edit wins: by the notes' own times, and (a hand edit changes neither) by when each side
                    // last committed the file
                    let committed =
                        |rev: &str| out_str(&g(brain, &["log", "-1", "--format=%ct", rev, "--", path]).unwrap_or_else(|_| empty_output()));
                    let own_lost = (stamp(&m_theirs), committed("MERGE_HEAD")) > (stamp(&m_ours), committed("HEAD"));
                    let (winner, m_lose, b_lose) = if own_lost { (theirs, m_ours, b_ours) } else { (ours, m_theirs, b_theirs) };
                    crate::util::atomic_write(&full, &winner)?;
                    let mut item = Meta::new();
                    for k in ["type", "title", "scope", "importance", "confidence", "provenance"] {
                        item.insert(k.into(), m_lose.get(k).cloned().unwrap_or(Value::Null));
                    }
                    item.insert("action".into(), json!("UPDATE"));
                    item.insert("target".into(), m_lose.get("id").cloned().unwrap_or(Value::Null));
                    item.insert("created".into(), json!(now_iso()));
                    item.insert(
                        "reason".into(),
                        json!(format!(
                            "sync conflict: this version lost to a newer edit (resolved on {}, where it was {})",
                            brain.device(),
                            if own_lost { "this device's own edit" } else { "another device's edit" }
                        )),
                    );
                    item.insert("source".into(), json!({"harness": "sync"}));
                    if let Some(kept) = brain.read_node(&full) {
                        item.insert("target_hash".into(), json!(crate::ops::content_hash(&kept)));
                    }
                    let id = brain.write_inbox(&item, &b_lose)?;
                    // the other device's losing text needs a look (not approve all); this device's own does not
                    if !own_lost {
                        let key = brain.review_key(&brain.root.join("inbox").join(format!("{id}.md")));
                        brain.mark_keys(&key.into_iter().collect::<Vec<_>>())?;
                    }
                    notes.push(format!("{path}: kept the newest version; the other one is in the inbox"));
                }
                (ours, theirs) => {
                    // deleted on one side: keep the surviving content
                    crate::util::atomic_write(&full, &ours.or(theirs).unwrap_or_default())?;
                    notes.push(format!("{path}: kept the side that was not deleted"));
                }
            }
        } else {
            g(brain, &["checkout", "--ours", "--", path])?;
            if path == "README.md" {
                notes.push("README.md: generated from the notes, regenerated after the merge".into());
            } else if [".gitignore", ".gitattributes", "brain.toml"].contains(&path) {
                notes.push(format!("{path}: conflict, kept this device's version"));
            } else if let Some(theirs) = stage(brain, 3, path) {
                // e.g. a skill edited on both devices: keep the other version next to it instead of dropping it
                let other = format!("{path}.from-other-device");
                crate::util::atomic_write(&brain.root.join(&other), &theirs)?;
                g(brain, &["add", "--", &other])?;
                notes.push(format!("{path}: conflict, kept this device's version; the other one is in {other}"));
            }
        }
        g(brain, &["add", "--", path])?;
    }
    g(brain, &["add", "-A", "inbox"])?;
    let r = g(brain, &["commit", "--no-edit", "-m", "sync: merge"])?;
    if !r.status.success() {
        let msg = if err_str(&r).is_empty() { out_str(&r) } else { err_str(&r) };
        return Err(Error::Sync(format!("could not complete merge: {msg}")));
    }
    Ok(notes)
}

/// Fetch and merge, all git calls within `timeout` seconds in total.
pub fn pull(brain: &Brain, timeout: u64) -> Result<Vec<String>> {
    crate::brain::within(Duration::from_secs(timeout), || pull_now(brain, timeout))
}

fn pull_now(brain: &Brain, timeout: u64) -> Result<Vec<String>> {
    if remote_url(brain).is_empty() {
        return Ok(vec!["no remote configured (rewire remote set <url>)".into()]);
    }
    brain.commit("sync: local changes");
    let r = git(brain, &["fetch", "origin"], timeout)?;
    if !r.status.success() {
        return Err(Error::Sync(format!("fetch failed: {}", err_str(&r))));
    }
    let reference = format!("origin/{}", branch(brain));
    let resolved = g(brain, &["rev-parse", "--verify", "-q", &format!("{reference}^{{commit}}")])?;
    if !resolved.status.success() {
        return Ok(vec!["remote has no brain yet".into()]);
    }
    // the commit checked below is the commit merged, whatever a concurrent fetch does to origin/<branch>
    let reference = out_str(&resolved);
    let behind = out_str(&g(brain, &["rev-list", "--count", &format!("HEAD..{reference}")])?);
    if behind == "0" {
        return Ok(vec!["already up to date".into()]);
    }
    // git overwrites an ignored file with a tracked one on merge: a remote that tracks local.toml would choose the
    // command this device runs; a symlink would let a note or skill reach outside the brain
    let incoming_toml = g(brain, &["show", &format!("{reference}:brain.toml")])?;
    if incoming_toml.status.success() && String::from_utf8_lossy(&incoming_toml.stdout).parse::<toml::Table>().is_err() {
        return Err(Error::Sync("refused to pull: the remote's brain.toml does not parse (fix it on the device that pushed it)".into()));
    }
    let found = crate::brain::unsafe_paths(brain, Some(&reference));
    if !found.is_empty() {
        return Err(Error::Sync(format!(
            "refused to pull: the remote contains {}; a brain never syncs device-local files (local.toml chooses what \
             runs on this machine), symlinks or submodules. Remove them from the remote and check who pushed them",
            found.join(", ")
        )));
    }
    let before = crate::config::load(&brain.path(&["brain.toml"]));
    // marked before the merge: a merge interrupted after it can never leave the other side's rules trusted
    let prior: std::collections::HashSet<String> = brain.untrusted().keys().cloned().collect();
    let trusted = remote_trusted(brain);
    let incoming = mark_incoming(brain, &reference, trusted)?;
    let head = out_str(&g(brain, &["rev-parse", "HEAD"])?);
    // the merge gets the caller's budget too (a hook has seconds); on a timeout or a failed resolve the merge is
    // aborted, so no conflict markers are left for the next commit to sweep in
    let r = match git(brain, &["merge", "--no-edit", "--allow-unrelated-histories", &reference], timeout.max(8)) {
        Ok(r) => r,
        Err(e) => {
            let _ = crate::brain::outside_deadline(|| g(brain, &["merge", "--abort"]));
            return Err(e);
        }
    };
    let mut notes = vec![format!("pulled {behind} commit(s)")];
    if !r.status.success() {
        match resolve(brain) {
            Ok(n) => notes.extend(n),
            Err(e) => {
                let _ = crate::brain::outside_deadline(|| g(brain, &["merge", "--abort"]));
                return Err(e);
            }
        }
    }
    notes.extend(crate::brain::ensure_ignores(brain)?);
    mark_merged(brain, &head, &prior)?;
    if trusted {
        brain.update_state("untrusted", |u| u.retain(|k, _| prior.contains(k)))?;
    }
    notes.extend(incoming);
    notes.extend(privacy_changes(brain, &before, &crate::config::load(&brain.path(&["brain.toml"])))?);
    if notes.len() > 1 {
        // a hook pulls silently: the snapshot tells the user what needs a look
        brain.update_state("notices", |n| {
            n.insert("sync".into(), json!({"at": now_iso(), "notes": notes[1..].to_vec()}));
        })?;
    }
    if crate::ops::write_home(brain, &mut crate::index::Index::new(brain)).is_ok() {
        brain.commit("sync: regenerate home page");
    }
    Ok(notes)
}

/// The text of each path at `rev` (None when it is not there), read in one `git cat-file --batch` call.
fn blobs(brain: &Brain, rev: &str, paths: &[String]) -> Result<Vec<Option<String>>> {
    if paths.is_empty() {
        return Ok(Vec::new());
    }
    let input: String = paths.iter().map(|p| format!("{rev}:{p}\n")).collect();
    let out = crate::brain::run_git_with_input(&brain.root, &["cat-file", "--batch"], Some(input), Duration::from_secs(GIT_TIMEOUT))
        .map_err(|e| Error::Sync(format!("git cat-file: {e}")))?;
    let data = out.stdout;
    let mut texts = Vec::new();
    let mut at = 0;
    for _ in paths {
        let Some(end) = data[at..].iter().position(|b| *b == b'\n').map(|e| at + e) else { break };
        let header = String::from_utf8_lossy(&data[at..end]).into_owned();
        at = end + 1;
        match header.rsplit_once(' ').and_then(|(_, n)| n.parse::<usize>().ok()).filter(|_| header.contains(" blob ")) {
            Some(size) if at + size <= data.len() => {
                texts.push(Some(String::from_utf8_lossy(&data[at..at + size]).into_owned()));
                at += size + 1;
            }
            _ => texts.push(None), // "<rev:path> missing"
        }
    }
    texts.resize(paths.len(), None);
    Ok(texts)
}

/// What a version of a note is, read with the same parser the index uses: (is a rule or preference, is active,
/// title). A file that does not parse counts as an active rule (fail closed).
fn rule_status(text: &str) -> (bool, bool, String) {
    match nodefmt::parse(text) {
        Err(_) => (true, true, String::new()),
        Ok((meta, _)) => {
            let type_ = meta.get("type").and_then(Value::as_str).unwrap_or("").trim().to_lowercase();
            let active = !crate::brain::is_set(meta.get("retracted_at")) && !crate::brain::is_set(meta.get("valid_to"));
            let title = meta.get("title").and_then(Value::as_str).unwrap_or("").to_string();
            (type_ == "constraint" || type_ == "preference", active, title)
        }
    }
}

/// Paths a diff lists (NUL-separated), for one --diff-filter.
fn diff_names(brain: &Brain, filter: &str, range: &[&str]) -> Result<Vec<String>> {
    let mut args = vec!["diff", "--name-only", "-z", filter];
    args.extend(range);
    args.extend(["--", ":(icase)nodes", ":(icase)skills", ":(icase)inbox"]); // Nodes/ is nodes/ on some file systems
    let out = g(brain, &args)?;
    if !out.status.success() {
        return Err(Error::Sync(format!("could not list changes: {}", err_str(&out))));
    }
    Ok(String::from_utf8_lossy(&out.stdout).split('\0').filter(|p| !p.is_empty()).map(String::from).collect())
}

/// Mark the rules, preferences, skills and inbox proposals among `paths` (as they are at `rev`) unreviewed on this
/// device; notes on what arrived and what was retracted.
fn mark_changed(brain: &Brain, old_rev: &str, rev: &str, paths: &[String]) -> Result<(usize, usize, Vec<String>)> {
    let notes_at: Vec<String> = paths.iter().filter(|p| p.to_lowercase().starts_with("nodes/")).cloned().collect();
    let texts = blobs(brain, rev, &notes_at)?;
    let before = blobs(brain, old_rev, &notes_at)?;
    let mut keys = Vec::new();
    let mut retracted = Vec::new();
    let trusted_before: std::collections::HashSet<String> = brain.untrusted().keys().cloned().collect();
    let mut previous: Vec<(String, String)> = Vec::new();
    for ((path, text), old) in notes_at.iter().zip(texts).zip(before) {
        // the version approved here so far, kept to show while its change waits for review
        if let (Some(old), Some(key)) = (old.as_deref(), brain.review_key(&brain.root.join(path))) {
            if matches!(rule_status(old), (true, true, _)) && !trusted_before.contains(&key) {
                previous.push((key, old.to_string()));
            }
        }
        // a reviewed rule turned into something else (a fact, say) under the same file is a change to a rule
        let was_rule = old.as_deref().is_some_and(|t| matches!(rule_status(t), (true, true, _)));
        // a changed note git could not hand over is not known to be harmless: it waits for review (fail closed)
        let Some(text) = text else {
            keys.extend(brain.review_key(&brain.root.join(path)));
            continue;
        };
        match rule_status(&text) {
            (true, true, _) => keys.extend(brain.review_key(&brain.root.join(path))),
            (true, false, title) => retracted.push(if title.is_empty() { path.clone() } else { title }),
            (false, true, _) if was_rule => keys.extend(brain.review_key(&brain.root.join(path))),
            _ => {}
        }
    }
    let mut proposals = 0;
    for p in paths.iter().filter(|p| !p.to_lowercase().starts_with("nodes/")) {
        if let Some(k) = brain.review_key(&brain.root.join(p)) {
            proposals += usize::from(k.starts_with("inbox/"));
            keys.push(k);
        }
    }
    keys.sort();
    keys.dedup();
    brain.mark_keys(&keys)?;
    for (key, old) in previous.iter().filter(|(k, _)| keys.contains(k)) {
        brain.keep_previous(key, old)?;
    }
    Ok((keys.len() - proposals, proposals, retracted))
}

/// A note that arrives with the id of a different note here would hide it (the index keeps one note per id):
/// the pull is refused.
fn refuse_shared_ids(brain: &Brain, theirs: &str, paths: &[String]) -> Result<()> {
    let notes: Vec<String> = paths.iter().filter(|p| p.to_lowercase().starts_with("nodes/")).cloned().collect();
    let mut clashes = Vec::new();
    for (path, text) in notes.iter().zip(blobs(brain, theirs, &notes)?) {
        let Some(id) = text.and_then(|t| nodefmt::parse(&t).ok()).and_then(|(m, _)| m.get("id").and_then(Value::as_str).map(String::from))
        else {
            continue;
        };
        // a file is named after its id's last 8 characters, and notes are found by them: the name must match the id,
        // and no other local note may share the short id (in any letter case)
        let short = crate::util::short_of(&id);
        // (a hand-named file without an id suffix is fine; one whose suffix names another id is not)
        let stem = path.to_lowercase().trim_end_matches(".md").to_string();
        let suffix =
            stem.rsplit_once('-').map(|(_, s)| s.to_string()).filter(|s| s.len() == 8 && s.chars().all(|c| c.is_ascii_alphanumeric()));
        if suffix.is_some_and(|s| s != short) {
            clashes.push(format!("{path} carries another id in its name than its own ({short})"));
            continue;
        }
        for other in brain.node_files(Some(&short)) {
            let rel = other.strip_prefix(&brain.root).unwrap_or(&other).to_string_lossy().replace('\\', "/");
            if rel.to_lowercase() == path.to_lowercase() {
                continue;
            }
            // the other side moved or deleted that file (a rename): no clash once merged
            let still_there = blobs(brain, theirs, std::slice::from_ref(&rel))?.into_iter().next().flatten().is_some();
            if still_there {
                clashes.push(format!("{path} reuses the id of {rel}"));
            }
        }
    }
    if clashes.is_empty() {
        Ok(())
    } else {
        Err(Error::Sync(format!("refused to pull: {}; check who pushed it", clashes.join(", "))))
    }
}

/// Before a merge: what the other side added or changed since the common ancestor is marked unreviewed, so a merge
/// interrupted afterwards never leaves it trusted (Brain::untrusted). After the merge `mark_merged` marks what
/// actually landed (a rename can carry their change into another file).
fn mark_incoming(brain: &Brain, theirs: &str, trusted: bool) -> Result<Vec<String>> {
    let range = format!("HEAD...{theirs}");
    let incoming = diff_names(brain, "--diff-filter=AMR", &[&range])?;
    refuse_shared_ids(brain, theirs, &incoming)?;
    let (rules, proposals, retracted) = mark_changed(brain, "HEAD", theirs, &incoming)?;
    let mut notes = Vec::new();
    if rules > 0 && !trusted {
        notes.push(format!(
            "{rules} rule(s), preference(s) or skill(s) arrived from another device; they are shown to agents on this \
             device after you review them: `rewire review`"
        ));
    }
    if proposals > 0 {
        notes.push(format!("{proposals} proposal(s) arrived in the inbox from another device (approve each by id after a look)"));
    }
    if !retracted.is_empty() {
        notes.push(format!("retracted by another device: {}", retracted.join(", ")));
    }
    // a rule can be taken away too: say so, it changes every session as much as adding one (judged by the type the
    // removed version declared)
    let deleted = diff_names(brain, "--diff-filter=D", &[&range])?;
    let old = blobs(brain, "HEAD", &deleted)?;
    let gone: Vec<String> = deleted
        .iter()
        .zip(old)
        .filter(|(p, text)| p.to_lowercase().ends_with("/skill.md") || text.as_deref().is_some_and(|t| rule_status(t).0))
        .map(|(p, _)| p.clone())
        .collect();
    if !gone.is_empty() {
        notes.push(format!("removed by another device: {}", gone.join(", ")));
    }
    Ok(notes)
}

/// After a merge: everything the merge changed in this device's tree, as it now is. What was marked before the merge
/// but did not change here (this device's own edit won a conflict) is this device's own again.
fn mark_merged(brain: &Brain, before: &str, prior: &std::collections::HashSet<String>) -> Result<()> {
    let landed = diff_names(brain, "--diff-filter=AMR", &[before, "HEAD"])?;
    mark_changed(brain, before, "HEAD", &landed)?;
    let landed: std::collections::HashSet<String> = landed.iter().filter_map(|p| brain.review_key(&brain.root.join(p))).collect();
    brain.update_state("untrusted", |u| u.retain(|k, _| prior.contains(k) || landed.contains(k)))?;
    Ok(())
}

/// brain.toml settings another device changed that decide what this device keeps and syncs. A change this device
/// has not pinned in local.toml is not taken: the old value is pinned there, so only the user loosens privacy.
fn privacy_changes(brain: &Brain, before: &crate::config::Config, after: &crate::config::Config) -> Result<Vec<String>> {
    let mut notes = Vec::new();
    for (key, old, new, pinned) in [
        ("transcripts", &before.synced_transcripts, &after.synced_transcripts, after.local_transcripts),
        ("profile", &before.synced_profile, &after.synced_profile, after.local_profile),
    ] {
        if old == new {
            continue;
        }
        if pinned {
            notes.push(format!(
                "brain.toml from another device changed [consolidation] {key}: {old} -> {new} (this device keeps its local.toml setting)"
            ));
        } else {
            pin_local(brain, key, old)?;
            notes.push(format!(
                "brain.toml from another device changed [consolidation] {key}: {old} -> {new}; this device keeps {old} \
                 (pinned in local.toml: change it there to accept)"
            ));
        }
    }
    Ok(notes)
}

/// Set `key = "value"` in local.toml's [consolidation] table. The result must still parse, or nothing is written
/// (a broken local.toml would silently fall back to defaults, the remote's privacy settings among them).
fn pin_local(brain: &Brain, key: &str, value: &str) -> Result<()> {
    let path = brain.path(&["local.toml"]);
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let line = format!("{key} = {}", serde_json::to_string(value)?);
    let header = Regex::new(r"(?m)^[ \t]*\[[ \t]*consolidation[ \t]*\][ \t]*(#.*)?$").unwrap();
    let key_line = Regex::new(&format!(r"(?m)^[ \t]*{key}[ \t]*=.*$")).unwrap();
    let updated = if key_line.is_match(&text) {
        key_line.replace(&text, line.as_str()).into_owned()
    } else if let Some(m) = header.find(&text) {
        format!("{}\n{line}{}", &text[..m.end()], &text[m.end()..])
    } else {
        format!("{}\n[consolidation]\n{line}\n", text.trim_end())
    };
    if updated.parse::<toml::Table>().is_err() {
        return Err(Error::Sync(format!("could not pin {key} in local.toml without breaking it: set {key} = \"{value}\" there by hand")));
    }
    crate::util::atomic_write(&path, &updated)?;
    Ok(())
}

pub fn push(brain: &Brain, timeout: u64) -> Result<Vec<String>> {
    if remote_url(brain).is_empty() {
        return Ok(vec!["no remote configured (rewire remote set <url>)".into()]);
    }
    brain.commit("sync: local changes");
    let reference = format!("origin/{}", branch(brain));
    if g(brain, &["rev-parse", "--verify", "-q", &reference])?.status.success()
        && out_str(&g(brain, &["rev-list", "--count", &format!("{reference}..HEAD")])?) == "0"
    {
        return Ok(vec!["nothing to push".into()]);
    }
    let mut r = git(brain, &["push", "-u", "origin", "HEAD"], timeout)?;
    let stderr = err_str(&r);
    if !r.status.success() && (stderr.contains("rejected") || stderr.contains("fetch first")) {
        let mut notes = pull(brain, GIT_TIMEOUT)?;
        r = g(brain, &["push", "-u", "origin", "HEAD"])?;
        if r.status.success() {
            notes.push("pushed".into());
            return Ok(notes);
        }
    }
    if !r.status.success() {
        return Err(Error::Sync(format!("push failed: {}", err_str(&r))));
    }
    Ok(vec!["pushed".into()])
}

pub fn sync(brain: &Brain) -> Result<Vec<String>> {
    let mut notes = pull(brain, GIT_TIMEOUT)?;
    notes.extend(push(brain, GIT_TIMEOUT)?);
    Ok(notes)
}
