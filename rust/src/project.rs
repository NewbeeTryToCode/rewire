//! Resolve the active project from a working directory.

use crate::brain::{absolute, expand_user};
use crate::util::slugify;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};

fn git(cwd: &Path, args: &[&str]) -> String {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(cwd).args(args);
    match crate::util::run_with_timeout(cmd, None, Duration::from_secs(5)) {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => String::new(),
    }
}

/// cwd -> (checked at, git toplevel, origin url). Two git processes cost ~3 ms, which was most of a warm MCP
/// recall; a long-running server asks for the same cwd on every call, and a repo's root and remote rarely move.
const GIT_TTL: Duration = Duration::from_secs(60);
type RepoCache = HashMap<String, (Instant, String, String)>;
static GIT_CACHE: Mutex<Option<RepoCache>> = Mutex::new(None);

fn repo(cwd: &str, path: &Path) -> (String, String) {
    let mut cache = GIT_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    let map = cache.get_or_insert_with(HashMap::new);
    if let Some((at, top, remote)) = map.get(cwd) {
        if at.elapsed() < GIT_TTL {
            return (top.clone(), remote.clone());
        }
    }
    let (top, remote) = (git(path, &["rev-parse", "--show-toplevel"]), git(path, &["remote", "get-url", "origin"]));
    map.insert(cwd.to_string(), (Instant::now(), top.clone(), remote.clone()));
    (top, remote)
}

/// A project slug, or None when cwd is not inside a project.
pub fn detect(cwd: Option<&str>, projects: &IndexMap<String, Vec<String>>) -> Option<String> {
    let cwd = cwd.filter(|c| !c.is_empty())?;
    let path = Path::new(cwd);
    if !path.is_dir() {
        return None;
    }
    let (top, remote) = repo(cwd, path);
    let abs = absolute(path).to_string_lossy().into_owned();
    let candidates: Vec<&str> = [remote.as_str(), top.as_str(), abs.as_str()].into_iter().filter(|c| !c.is_empty()).collect();
    for (slug, patterns) in projects {
        for pattern in patterns {
            let pat = expand_user(pattern).to_string_lossy().into_owned();
            if candidates.iter().any(|c| c.contains(&pat)) {
                return Some(slug.clone());
            }
        }
    }
    if !top.is_empty() {
        let base = Path::new(&top).file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        return Some(slugify(&base, 48));
    }
    None
}

pub fn scopes_for(project: Option<&str>) -> Vec<String> {
    let mut out = vec!["global".to_string()];
    if let Some(p) = project {
        out.push(format!("project:{p}"));
    }
    out
}
