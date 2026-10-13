//! The brain on disk: markdown nodes, append-only events, git history.
//! Source of truth is files; .index/ is derived and .state/ is device-local.

use crate::config::{self, Config};
use crate::util::{atomic_write, now_iso, now_ts, short_of, slugify, ulid};
use crate::{fm, nodefmt, pyjson, schema, Error, Meta, Result};
use fs4::fs_std::FileExt;
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

pub const GITATTRIBUTES: &str = "events/**/*.jsonl merge=union\n";
pub const GITIGNORE: &str = ".index/\n.state/\n.obsidian/workspace*.json\n";
const DIRS: [&str; 6] = ["nodes", "events", "sessions", "inbox", ".index", ".state"];

pub fn default_path() -> PathBuf {
    let raw = std::env::var("REWIRE_BRAIN").unwrap_or_else(|_| "~/brain".into());
    expand_user(&raw)
}

pub fn expand_user(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return Path::new(&home).join(rest);
        }
    }
    if path == "~" {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home);
        }
    }
    PathBuf::from(path)
}

pub fn absolute(path: &Path) -> PathBuf {
    let p = if path.is_absolute() { path.to_path_buf() } else { std::env::current_dir().unwrap_or_default().join(path) };
    // lexical normalization like os.path.abspath (no symlink resolution)
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

#[derive(Debug, Clone)]
pub struct Node {
    pub meta: Meta,
    pub body: String,
    pub path: PathBuf,
}

impl Node {
    pub fn id(&self) -> &str {
        self.meta.get("id").and_then(Value::as_str).unwrap_or("")
    }
    pub fn type_(&self) -> &str {
        self.meta.get("type").and_then(Value::as_str).unwrap_or("")
    }
    pub fn title(&self) -> &str {
        self.meta.get("title").and_then(Value::as_str).unwrap_or("")
    }
    pub fn active(&self) -> bool {
        !is_set(self.meta.get("retracted_at")) && !is_set(self.meta.get("valid_to"))
    }
}

pub fn is_set(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Bool(b)) => *b,
        Some(_) => true,
    }
}

/// Exclusive writer lock; released on drop.
pub struct LockGuard(fs::File);

impl Drop for LockGuard {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.0);
    }
}

pub struct Brain {
    pub root: PathBuf,
    pub config: Config,
}

impl Brain {
    pub fn open(root: Option<&Path>) -> Brain {
        let root = absolute(&root.map(|r| expand_user(&r.to_string_lossy())).unwrap_or_else(default_path));
        let config = config::load(&root.join("brain.toml"));
        Brain { root, config }
    }

    pub fn path(&self, parts: &[&str]) -> PathBuf {
        let mut p = self.root.clone();
        for part in parts {
            p.push(part);
        }
        p
    }

    pub fn exists(&self) -> bool {
        self.path(&["brain.toml"]).exists()
    }

    /// Device name for the event log; device-local (never synced).
    pub fn device(&self) -> String {
        if let Ok(env) = std::env::var("REWIRE_DEVICE") {
            if !env.is_empty() {
                return config::slug(&env);
            }
        }
        let state = self.read_state("device");
        if let Some(name) = state.get("name").and_then(Value::as_str).filter(|s| !s.is_empty()) {
            return name.to_string();
        }
        let base = if self.config.device.is_empty() { config::default_device() } else { self.config.device.clone() };
        let name = config::slug(&base);
        let mut m = Meta::new();
        m.insert("name".into(), json!(name));
        let _ = self.write_state("device", &m);
        name
    }

    // ---- lifecycle -------------------------------------------------------

    pub fn init(root: Option<&Path>, device: Option<&str>) -> Result<Brain> {
        let root = absolute(&root.map(|r| expand_user(&r.to_string_lossy())).unwrap_or_else(default_path));
        if root.join("brain.toml").exists() {
            return Err(Error::Op(format!("brain already exists at {}", root.display())));
        }
        for sub in DIRS {
            fs::create_dir_all(root.join(sub))?;
        }
        atomic_write(&root.join("brain.toml"), config::DEFAULT_TOML)?;
        atomic_write(&root.join(".gitattributes"), GITATTRIBUTES)?;
        atomic_write(&root.join(".gitignore"), GITIGNORE)?;
        let brain = Brain::open(Some(&root));
        brain.init_local_state(device)?;
        let mut index = crate::index::Index::new(&brain);
        crate::ops::write_home(&brain, &mut index)?;
        if !root.join(".git").is_dir() {
            let _ = Command::new("git").args(["init", "-q"]).arg(&root).output();
        }
        brain.commit("init brain");
        Ok(brain)
    }

    /// Attach this machine to an existing brain repository.
    pub fn clone_from(url: &str, root: Option<&Path>, device: Option<&str>) -> Result<Brain> {
        let root = absolute(&root.map(|r| expand_user(&r.to_string_lossy())).unwrap_or_else(default_path));
        if root.exists() && fs::read_dir(&root)?.next().is_some() {
            return Err(Error::Op(format!("{} is not empty", root.display())));
        }
        let out = Command::new("git").args(["clone", "-q", url]).arg(&root).output()?;
        if !out.status.success() {
            return Err(Error::Op(format!("git clone failed: {}", String::from_utf8_lossy(&out.stderr).trim())));
        }
        let brain = Brain::open(Some(&root));
        if !brain.exists() {
            return Err(Error::Op(format!("{url} is not a rewire brain (no brain.toml)")));
        }
        for sub in DIRS {
            fs::create_dir_all(root.join(sub))?;
        }
        let dev = device.map(String::from).unwrap_or_else(config::default_device);
        brain.init_local_state(Some(&dev))?;
        Ok(brain)
    }

    fn init_local_state(&self, device: Option<&str>) -> Result<()> {
        // Only transcripts written from now on are consolidated automatically.
        let mut scan = Meta::new();
        scan.insert("since".into(), json!(now_iso()));
        scan.insert("since_ts".into(), json!(now_ts()));
        self.write_state("scan", &scan)?;
        if let Some(d) = device {
            let mut m = Meta::new();
            m.insert("name".into(), json!(config::slug(d)));
            self.write_state("device", &m)?;
        }
        Ok(())
    }

    // ---- nodes -----------------------------------------------------------

    pub fn read_node(&self, path: &Path) -> Option<Node> {
        let text = fs::read_to_string(path).ok()?;
        let (meta, body) = nodefmt::parse(&text).ok()?;
        if !is_set(meta.get("id")) {
            return None;
        }
        Some(Node { meta, body, path: path.to_path_buf() })
    }

    fn node_files(&self, suffix: Option<&str>) -> Vec<PathBuf> {
        let mut out = Vec::new();
        let Ok(dirs) = fs::read_dir(self.path(&["nodes"])) else { return out };
        for d in dirs.flatten() {
            if !d.path().is_dir() {
                continue;
            }
            let Ok(files) = fs::read_dir(d.path()) else { continue };
            for f in files.flatten() {
                let name = f.file_name().to_string_lossy().into_owned();
                let ok = match suffix {
                    Some(s) => name.ends_with(&format!("-{s}.md")),
                    None => name.ends_with(".md"),
                };
                if ok {
                    out.push(f.path());
                }
            }
        }
        out.sort();
        out
    }

    /// Node files whose name ends with `-<suffix>.md`.
    pub fn iter_node_files_with_suffix(&self, suffix: &str) -> Vec<PathBuf> {
        self.node_files(Some(suffix))
    }

    /// Path of the node whose id ends with short_id (file names carry the suffix).
    pub fn locate(&self, short_id: &str) -> Option<PathBuf> {
        let hits = self.node_files(Some(&short_id.to_lowercase()));
        if hits.len() == 1 { hits.into_iter().next() } else { None }
    }

    pub fn iter_nodes(&self) -> Vec<Node> {
        self.node_files(None).iter().filter_map(|p| self.read_node(p)).collect()
    }

    pub fn get_node(&self, node_id: &str) -> Option<Node> {
        for path in self.node_files(Some(&short_of(node_id))) {
            if let Some(n) = self.read_node(&path) {
                if n.id() == node_id {
                    return Some(n);
                }
            }
        }
        self.iter_nodes().into_iter().find(|n| n.id() == node_id)
    }

    pub fn new_meta(&self, type_: &str, title: &str, fields: Meta) -> Meta {
        let mut fields = fields;
        let mut take = |k: &str, default: Value| fields.shift_remove(k).unwrap_or(default);
        let mut meta = Meta::new();
        meta.insert("id".into(), json!(ulid()));
        meta.insert("type".into(), json!(type_));
        meta.insert("title".into(), json!(title.trim()));
        meta.insert("scope".into(), take("scope", json!("global")));
        let imp = take("importance", json!(3));
        meta.insert("importance".into(), json!(imp.as_i64().or_else(|| imp.as_str().and_then(|s| s.parse().ok())).unwrap_or(3)));
        meta.insert("confidence".into(), take("confidence", json!("inferred")));
        meta.insert("provenance".into(), take("provenance", json!("assistant")));
        meta.insert("valid_from".into(), take("valid_from", json!(now_iso()[..10].to_string())));
        meta.insert("valid_to".into(), Value::Null);
        meta.insert("recorded_at".into(), json!(now_iso()));
        meta.insert("retracted_at".into(), Value::Null);
        meta.insert("edges".into(), take("edges", json!([])));
        for (k, v) in fields {
            meta.insert(k, v);
        }
        meta
    }

    pub fn write_node(&self, meta: &Meta, body: &str, path: Option<&Path>) -> Result<Node> {
        schema::validate(meta, body)?;
        let id = meta["id"].as_str().unwrap_or("").to_string();
        let path = match path {
            Some(p) => p.to_path_buf(),
            None => {
                let slug = slugify(meta["title"].as_str().unwrap_or(""), 48);
                self.path(&["nodes", meta["type"].as_str().unwrap_or("fact"), &format!("{slug}-{}.md", short_of(&id))])
            }
        };
        let mut meta = meta.clone();
        let edges: Vec<Value> = nodefmt::edges_of(&meta)
            .into_iter()
            .map(|(rel, to)| json!({"rel": rel, "to": nodefmt::short(&to)}))
            .collect();
        meta.insert("edges".into(), Value::Array(edges));
        let text = nodefmt::render(&meta, body, &path, &|s| self.locate(s));
        atomic_write(&path, &text)?;
        let trimmed = body.trim();
        Ok(Node { meta, body: if trimmed.is_empty() { String::new() } else { format!("{trimmed}\n") }, path })
    }

    // ---- inbox -----------------------------------------------------------

    pub fn iter_inbox(&self) -> Vec<Node> {
        let mut paths: Vec<PathBuf> = fs::read_dir(self.path(&["inbox"]))
            .map(|r| r.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "md")).collect())
            .unwrap_or_default();
        paths.sort();
        paths
            .into_iter()
            .filter_map(|p| {
                let text = fs::read_to_string(&p).ok()?;
                let (meta, body) = fm::loads(&text).ok()?;
                Some(Node { meta, body, path: p })
            })
            .collect()
    }

    pub fn write_inbox(&self, meta: &Meta, body: &str) -> Result<String> {
        let mut meta = meta.clone();
        let item_id = match meta.get("inbox_id").and_then(Value::as_str) {
            Some(s) => s.to_string(),
            None => {
                let id = ulid();
                meta.insert("inbox_id".into(), json!(id));
                id
            }
        };
        atomic_write(&self.path(&["inbox", &format!("{item_id}.md")]), &fm::dumps(&meta, body))?;
        Ok(item_id)
    }

    // ---- events ----------------------------------------------------------

    pub fn append_events(&self, events: &[Meta]) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }
        let month = &now_iso()[..7];
        let path = self.path(&["events", &self.device(), &format!("{month}.jsonl")]);
        fs::create_dir_all(path.parent().unwrap())?;
        let mut fh = fs::OpenOptions::new().create(true).append(true).open(&path)?;
        let mut buf = String::new();
        for ev in events {
            let mut ev = ev.clone();
            if !ev.contains_key("t") {
                ev.insert("t".into(), json!(now_iso()));
            }
            buf.push_str(&pyjson::dumps(&Value::Object(ev), pyjson::Opts { ensure_ascii: false, sort_keys: true }));
            buf.push('\n');
        }
        fh.write_all(buf.as_bytes())?;
        Ok(())
    }

    // ---- device-local state ---------------------------------------------

    pub fn read_state(&self, name: &str) -> Meta {
        fs::read_to_string(self.path(&[".state", &format!("{name}.json")]))
            .ok()
            .and_then(|t| serde_json::from_str::<Meta>(&t).ok())
            .unwrap_or_default()
    }

    pub fn write_state(&self, name: &str, data: &Meta) -> Result<()> {
        atomic_write(
            &self.path(&[".state", &format!("{name}.json")]),
            &pyjson::dumps_indent1_sorted(&Value::Object(data.clone())),
        )?;
        Ok(())
    }

    /// Exclusive lock for writers (consolidation, sync).
    pub fn lock(&self, timeout: Duration) -> Result<LockGuard> {
        fs::create_dir_all(self.path(&[".state"]))?;
        let fh = fs::OpenOptions::new().create(true).write(true).truncate(true).open(self.path(&[".state", "brain.lock"]))?;
        let deadline = Instant::now() + timeout;
        loop {
            if FileExt::try_lock_exclusive(&fh).unwrap_or(false) {
                return Ok(LockGuard(fh));
            }
            if Instant::now() >= deadline {
                return Err(Error::Locked("another rewire process is writing to the brain".into()));
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    }

    // ---- git -------------------------------------------------------------

    pub fn git(&self, args: &[&str]) -> Output {
        Command::new("git")
            .arg("-C")
            .arg(&self.root)
            .args(["-c", "user.name=rewire", "-c", "user.email=rewire@localhost"])
            .args(args)
            .stdin(Stdio::null())
            .output()
            .unwrap_or_else(|e| Output {
                status: failed_status(),
                stdout: Vec::new(),
                stderr: e.to_string().into_bytes(),
            })
    }

    pub fn commit(&self, message: &str) -> bool {
        if !self.path(&[".git"]).is_dir() {
            return false;
        }
        self.git(&["add", "-A"]);
        self.git(&["commit", "-q", "-m", message]).status.success()
    }
}

#[cfg(unix)]
pub fn failed_status() -> std::process::ExitStatus {
    use std::os::unix::process::ExitStatusExt;
    std::process::ExitStatus::from_raw(1 << 8)
}

#[cfg(not(unix))]
pub fn failed_status() -> std::process::ExitStatus {
    use std::os::windows::process::ExitStatusExt;
    std::process::ExitStatus::from_raw(1)
}

pub fn out_str(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

pub fn err_str(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).trim().to_string()
}
