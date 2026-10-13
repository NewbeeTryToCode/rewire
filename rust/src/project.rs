//! Resolve the active project from a working directory.

use crate::brain::{absolute, expand_user};
use crate::util::slugify;
use indexmap::IndexMap;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

fn git(cwd: &Path, args: &[&str]) -> String {
    let child = Command::new("git").arg("-C").arg(cwd).args(args).stdin(std::process::Stdio::null()).output();
    let _ = Duration::from_secs(5); // git rev-parse/remote are local and fast; no timeout needed
    match child {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => String::new(),
    }
}

/// A project slug, or None when cwd is not inside a project.
pub fn detect(cwd: Option<&str>, projects: &IndexMap<String, Vec<String>>) -> Option<String> {
    let cwd = cwd.filter(|c| !c.is_empty())?;
    let path = Path::new(cwd);
    if !path.is_dir() {
        return None;
    }
    let top = git(path, &["rev-parse", "--show-toplevel"]);
    let remote = git(path, &["remote", "get-url", "origin"]);
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
