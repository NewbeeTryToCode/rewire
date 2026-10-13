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
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

pub const GIT_TIMEOUT: u64 = 30;

fn git(brain: &Brain, args: &[&str], timeout: u64) -> Result<Output> {
    let mut child = Command::new("git")
        .arg("-C")
        .arg(&brain.root)
        .args(["-c", "user.name=rewire", "-c", "user.email=rewire@localhost"])
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0") // never hang on a credential prompt in a hook
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let deadline = Instant::now() + Duration::from_secs(timeout);
    loop {
        if child.try_wait()?.is_some() {
            return Ok(child.wait_with_output()?);
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            return Err(Error::Sync(format!("git {} timed out", args[0])));
        }
        std::thread::sleep(Duration::from_millis(20));
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

pub fn branch(brain: &Brain) -> String {
    let b = g(brain, &["rev-parse", "--abbrev-ref", "HEAD"]).map(|o| out_str(&o)).unwrap_or_default();
    if b.is_empty() { "main".into() } else { b }
}

pub fn set_remote(brain: &mut Brain, url: &str) -> Result<Vec<String>> {
    if remote_url(brain).is_empty() {
        g(brain, &["remote", "add", "origin", url])?;
    } else {
        g(brain, &["remote", "set-url", "origin", url])?;
    }
    enable_auto(brain)?;
    let mut notes = vec![format!("origin = {url}; auto sync enabled (pull at session start, push after consolidation)")];
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
    std::fs::write(&path, text)?;
    brain.config.sync_auto = true;
    brain.commit("enable sync");
    Ok(())
}

fn stamp(meta: &Meta) -> String {
    let s = |k: &str| meta.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    std::cmp::max(s("recorded_at"), s("retracted_at"))
}

fn stage(brain: &Brain, n: u8, path: &str) -> Option<String> {
    let o = g(brain, &["show", &format!(":{n}:{path}")]).ok()?;
    if o.status.success() { Some(String::from_utf8_lossy(&o.stdout).into_owned()) } else { None }
}

/// Resolve merge conflicts in place. Returns human-readable notes.
fn resolve(brain: &Brain) -> Result<Vec<String>> {
    let mut notes = Vec::new();
    let conflicted = out_str(&g(brain, &["diff", "--name-only", "--diff-filter=U"])?);
    for path in conflicted.split_whitespace() {
        let full = brain.root.join(path);
        if path.starts_with("nodes/") {
            match (stage(brain, 2, path), stage(brain, 3, path)) {
                (Some(ours), Some(theirs)) => {
                    let (m_ours, b_ours) = nodefmt::parse(&ours).map_err(Error::Sync)?;
                    let (m_theirs, b_theirs) = nodefmt::parse(&theirs).map_err(Error::Sync)?;
                    let (winner, m_lose, b_lose) =
                        if stamp(&m_theirs) > stamp(&m_ours) { (theirs, m_ours, b_ours) } else { (ours, m_theirs, b_theirs) };
                    std::fs::write(&full, winner)?;
                    let mut item = Meta::new();
                    for k in ["type", "title", "scope", "importance", "confidence", "provenance"] {
                        item.insert(k.into(), m_lose.get(k).cloned().unwrap_or(Value::Null));
                    }
                    item.insert("action".into(), json!("UPDATE"));
                    item.insert("target".into(), m_lose.get("id").cloned().unwrap_or(Value::Null));
                    item.insert("created".into(), json!(now_iso()));
                    item.insert("reason".into(), json!("sync conflict: this version lost to a newer edit from another device"));
                    item.insert("source".into(), json!({"harness": "sync"}));
                    if let Some(kept) = brain.read_node(&full) {
                        item.insert("target_hash".into(), json!(crate::ops::content_hash(&kept)));
                    }
                    brain.write_inbox(&item, &b_lose)?;
                    notes.push(format!("{path}: kept the newest version; the other one is in the inbox"));
                }
                (ours, theirs) => {
                    // deleted on one side: keep the surviving content
                    std::fs::write(&full, ours.or(theirs).unwrap_or_default())?;
                    notes.push(format!("{path}: kept the side that was not deleted"));
                }
            }
        } else {
            g(brain, &["checkout", "--ours", "--", path])?;
            notes.push(format!("{path}: conflict, kept this device's version"));
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

pub fn pull(brain: &Brain, timeout: u64) -> Result<Vec<String>> {
    if remote_url(brain).is_empty() {
        return Ok(vec!["no remote configured (rewire remote set <url>)".into()]);
    }
    brain.commit("sync: local changes");
    let r = git(brain, &["fetch", "origin"], timeout)?;
    if !r.status.success() {
        return Err(Error::Sync(format!("fetch failed: {}", err_str(&r))));
    }
    let reference = format!("origin/{}", branch(brain));
    if !g(brain, &["rev-parse", "--verify", "-q", &reference])?.status.success() {
        return Ok(vec!["remote has no brain yet".into()]);
    }
    let behind = out_str(&g(brain, &["rev-list", "--count", &format!("HEAD..{reference}")])?);
    if behind == "0" {
        return Ok(vec!["already up to date".into()]);
    }
    let r = g(brain, &["merge", "--no-edit", "--allow-unrelated-histories", &reference])?;
    let mut notes = vec![format!("pulled {behind} commit(s)")];
    if !r.status.success() {
        notes.extend(resolve(brain)?);
    }
    Ok(notes)
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
