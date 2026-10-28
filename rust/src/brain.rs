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
use std::process::{Command, Output};
use std::time::{Duration, Instant};

pub const GITATTRIBUTES: &str = "events/**/*.jsonl merge=union\n";
/// The derived index, device-local state and device-local settings never sync. A pulled `.gitignore` that drops
/// any of them gets them back (and stops tracking what it let in). Returns notes for what was repaired.
/// Device-local paths: ignored by git, never synced. A tracked local.toml would choose the command every device
/// runs, so a commit that tracks one is refused on pull and clone (git overwrites ignored files on a merge).
pub const LOCAL_ONLY: [&str; 3] = [".index", ".state", "local.toml"];

/// Entries of a commit (`rev`) or of the git index (None) that a brain must never receive from another device,
/// as "path (why)": device-local files in any letter case (case-insensitive file systems check out `Local.toml` as
/// local.toml), symlinks (they let a note or skill read or write outside the brain), submodules and git-lfs
/// settings (they make git fetch from elsewhere).
pub fn unsafe_paths(brain: &Brain, rev: Option<&str>) -> Vec<String> {
    use unicode_normalization::UnicodeNormalization;
    if !brain.path(&[".git"]).exists() {
        return Vec::new();
    }
    let out = match rev {
        Some(r) => brain.git(&["ls-tree", "-r", "-z", "--full-tree", r]),
        None => brain.git(&["ls-files", "-s", "-z"]),
    };
    if !out.status.success() {
        // fail closed: what cannot be listed cannot be called safe
        return vec![format!("? (git could not list the files: {})", err_str(&out))];
    }
    let mut found = Vec::new();
    // a .gitignore that un-ignores device-local files ("!local.toml") would push them on the next commit
    let ignore = match rev {
        Some(r) => out_str(&brain.git(&["show", &format!("{r}:.gitignore")])),
        None => fs::read_to_string(brain.path(&[".gitignore"])).unwrap_or_default(),
    };
    if ignore.lines().any(|l| l.trim().starts_with('!')) && unignores_local(&ignore) {
        found.push(".gitignore (it un-ignores device-local files)".into());
    }
    let mut seen: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for entry in String::from_utf8_lossy(&out.stdout).split('\0').filter(|e| !e.is_empty()) {
        let Some((info, path)) = entry.split_once('\t') else { continue };
        let mode = info.split(' ').next().unwrap_or("");
        let first = path.split('/').next().unwrap_or("").nfc().collect::<String>().to_lowercase();
        // two names one file system may treat as one file: one review would cover a file the user never saw
        let folded: String = path.nfc().collect::<String>().to_lowercase();
        if let Some(other) = seen.insert(folded, path.to_string()) {
            found.push(format!("{} and {other} (names that differ only in letter case)", path.escape_debug()));
            continue;
        }
        let why = if path.chars().any(char::is_control) {
            "control characters in the name"
        } else if LOCAL_ONLY.contains(&first.as_str()) {
            "device-local"
        } else if mode == "120000" {
            "symlink"
        } else if mode == "160000" || first == ".gitmodules" {
            "submodule"
        } else if first == ".lfsconfig" {
            "git-lfs settings"
        } else {
            continue;
        };
        found.push(format!("{} ({why})", path.escape_debug()));
    }
    found
}

/// Whether a .gitignore leaves any device-local path un-ignored, asked of git itself in a scratch repository (so
/// wildcards such as `!l?cal.toml` count too). Fails closed.
fn unignores_local(gitignore: &str) -> bool {
    let dir = std::env::temp_dir().join(format!("rewire-ignore-{}-{}", std::process::id(), crate::util::ulid()));
    let check = || -> Option<bool> {
        fs::create_dir_all(&dir).ok()?;
        fs::write(dir.join(".gitignore"), gitignore).ok()?;
        run_git(&dir, &["init", "-q"], Duration::from_secs(10)).ok()?;
        let mut free = false;
        for path in ["local.toml", ".state/x", ".index/x", "Local.toml"] {
            let out = run_git(&dir, &["check-ignore", "-q", "--no-index", path], Duration::from_secs(10)).ok()?;
            free |= out.status.code() == Some(1); // 1: not ignored
        }
        Some(free)
    };
    let result = check().unwrap_or(true);
    let _ = fs::remove_dir_all(&dir);
    result
}

pub fn ensure_ignores(brain: &Brain) -> Result<Vec<String>> {
    let exclude = brain.path(&[".git", "info", "exclude"]);
    if brain.path(&[".git"]).is_dir() && !fs::read_to_string(&exclude).unwrap_or_default().contains("local.toml") {
        // a second copy git keeps outside the synced tree
        let current = fs::read_to_string(&exclude).unwrap_or_default();
        let _ = atomic_write(
            &exclude,
            &format!("{current}{}.index/\n.state/\nlocal.toml\n", if current.ends_with('\n') || current.is_empty() { "" } else { "\n" }),
        );
    }
    let path = brain.path(&[".gitignore"]);
    let current = fs::read_to_string(&path).unwrap_or_default();
    let missing: Vec<&str> =
        [".index/", ".state/", "local.toml"].into_iter().filter(|e| !current.lines().any(|l| l.trim() == *e)).collect();
    if missing.is_empty() {
        return Ok(Vec::new());
    }
    let sep = if current.is_empty() || current.ends_with('\n') { "" } else { "\n" };
    atomic_write(&path, &format!("{current}{sep}{}\n", missing.join("\n")))?;
    if brain.path(&[".git"]).is_dir() {
        for entry in &missing {
            brain.git(&["rm", "-r", "-q", "--cached", "--ignore-unmatch", "--", entry.trim_end_matches('/')]);
        }
        brain.commit("keep device-local files out of sync");
    }
    Ok(vec![format!(".gitignore: restored {}", missing.join(", "))])
}

/// Git settings for every git call on a brain: an identity of its own, no signing prompt (a user's
/// `commit.gpgsign` would make every commit fail silently), and maintenance in the foreground (a detached git left
/// running races the next writer and cleanup).
pub const GIT_CONFIG: [&str; 10] = [
    "-c",
    "user.name=rewire",
    "-c",
    "user.email=rewire@localhost",
    "-c",
    "commit.gpgsign=false",
    "-c",
    "gc.autoDetach=false",
    "-c",
    "maintenance.autoDetach=false",
];

pub const GITIGNORE: &str = ".index/\n.state/\nlocal.toml\n.obsidian/workspace*.json\n";
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
        let mut brain = Brain { root, config };
        if !brain.config.sync_auto && crate::sync::origin_is_env_remote(&brain) {
            brain.config.sync_auto = true;
        }
        brain
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

    /// The key under which a rule, preference or skill is reviewed on this device: "nodes/constraint/x.md",
    /// "skills/<name>/". None for other paths.
    pub fn review_key(&self, path: &Path) -> Option<String> {
        // lower case: a case-insensitive file system checks out nodes/Constraint/ as nodes/constraint/
        let rel = path.strip_prefix(&self.root).unwrap_or(path).to_string_lossy().replace('\\', "/").to_lowercase();
        if rel.starts_with("nodes/") || rel.starts_with("inbox/") {
            Some(rel) // by path, whatever folder: what decides is the type the file declares
        } else {
            rel.strip_prefix("skills/").and_then(|r| r.split('/').next()).filter(|n| !n.is_empty()).map(|n| format!("skills/{n}/"))
        }
    }

    /// Rules, preferences and skills that arrived from another device by a pull and that the user has not yet
    /// reviewed on this device: they act as instructions in every session, and a remote is not the user.
    /// Device-local (.state/untrusted.json); `rewire review` lists them, `review approve` trusts them.
    pub fn untrusted(&self) -> Meta {
        self.read_state("untrusted")
    }

    pub fn is_untrusted(&self, path: &Path) -> bool {
        self.review_key(path).is_some_and(|k| self.untrusted().contains_key(&k))
    }

    /// Ids of the unreviewed notes, for leaving them out of recall, `get` and the hooks.
    pub fn untrusted_ids(&self) -> std::collections::HashSet<String> {
        self.untrusted_nodes().into_values().map(|n| n.id().to_string()).collect()
    }

    /// The unreviewed notes by review key. Keys are lower case, files need not be: notes are found by walking them.
    pub fn untrusted_nodes(&self) -> std::collections::HashMap<String, Node> {
        let untrusted = self.untrusted();
        if !untrusted.keys().any(|k| k.starts_with("nodes/")) {
            return Default::default();
        }
        self.iter_nodes()
            .into_iter()
            .filter_map(|n| self.review_key(&n.path).filter(|k| untrusted.contains_key(k)).map(|k| (k, n)))
            .collect()
    }

    /// Mark the rules, preferences and skills among `paths` (relative to the brain) as unreviewed on this device; how
    /// many were marked.
    pub fn mark_untrusted(&self, paths: &[String]) -> Result<usize> {
        let keys: std::collections::BTreeSet<String> = paths
            .iter()
            .filter(|p| {
                if p.to_lowercase().starts_with("skills/") {
                    return true;
                }
                // the type the file declares, not its folder, and only what is still in force; a file that does not
                // parse counts (fail closed)
                p.ends_with(".md")
                    && fs::read_to_string(self.root.join(p)).is_ok_and(|text| match nodefmt::parse(&text) {
                        Err(_) => true,
                        Ok((meta, _)) => {
                            let t = meta.get("type").and_then(Value::as_str).unwrap_or("").trim().to_lowercase();
                            (t == "constraint" || t == "preference") && !is_set(meta.get("retracted_at")) && !is_set(meta.get("valid_to"))
                        }
                    })
            })
            .filter_map(|p| self.review_key(&self.root.join(p)))
            .collect();
        self.mark_keys(&keys.into_iter().collect::<Vec<_>>())
    }

    /// Mark review keys (Brain::review_key) as unreviewed on this device; how many.
    pub fn mark_keys(&self, keys: &[String]) -> Result<usize> {
        if !keys.is_empty() {
            self.update_state("untrusted", |u| {
                for k in keys {
                    // an entry already there keeps what it holds (the approved version kept to show meanwhile)
                    u.entry(k.clone()).or_insert_with(|| json!({"from": "sync", "at": now_iso()}));
                }
            })?;
        }
        Ok(keys.len())
    }

    /// Keep the version of a rule the user had approved while a change to it from another device waits for review:
    /// the snapshot shows that version meanwhile, so a rule never silently drops out of an agent's context.
    pub fn keep_previous(&self, key: &str, text: &str) -> Result<()> {
        self.update_state("untrusted", |u| {
            if let Some(Value::Object(entry)) = u.get_mut(key) {
                entry.entry("previous").or_insert_with(|| json!(text));
            }
        })
    }

    /// Writers check first: a brain deleted or moved while a background consolidation was waiting on the LLM must
    /// not be recreated as loose directories at the old path.
    pub fn present(&self) -> Result<()> {
        if self.exists() {
            Ok(())
        } else {
            Err(Error::Op(format!("no brain at {} any more (brain.toml is gone); not writing", self.root.display())))
        }
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
            let mut cmd = Command::new("git");
            cmd.args(["init", "-q"]).arg(&root);
            let _ = crate::util::run_with_timeout(cmd, None, Duration::from_secs(30));
        }
        brain.commit("init brain");
        Ok(brain)
    }

    /// Clone the brain at `url` into `root` for a machine that starts empty (REWIRE_REMOTE): into a folder of its
    /// own first, then moved into place, so a session-start hook and an MCP server starting together never see half a
    /// clone, and whichever finishes second uses the first one's.
    pub fn fetch(url: &str, root: &Path) -> Result<Brain> {
        let root = absolute(root);
        let parent = root.parent().map(Path::to_path_buf).unwrap_or_else(|| PathBuf::from("."));
        fs::create_dir_all(&parent)?;
        let name = root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "brain".into());
        let tmp = parent.join(format!(".{name}.fetch-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        let cloned = Brain::clone_from(url, Some(&tmp), None).and_then(|_| {
            if root.is_dir() && fs::read_dir(&root)?.next().is_none() {
                fs::remove_dir(&root)?;
            }
            Ok(fs::rename(&tmp, &root)?)
        });
        let _ = fs::remove_dir_all(&tmp);
        let brain = Brain::open(Some(&root));
        match cloned {
            Ok(()) => Ok(brain),
            Err(_) if brain.exists() => Ok(brain), // another process got there first
            Err(e) => Err(e),
        }
    }

    /// Attach this machine to an existing brain repository.
    pub fn clone_from(url: &str, root: Option<&Path>, device: Option<&str>) -> Result<Brain> {
        let root = absolute(&root.map(|r| expand_user(&r.to_string_lossy())).unwrap_or_else(default_path));
        if root.exists() && fs::read_dir(&root)?.next().is_some() {
            return Err(Error::Op(format!("{} is not empty", root.display())));
        }
        let mut cmd = Command::new("git");
        cmd.args(["clone", "-q", url]).arg(&root).env("GIT_TERMINAL_PROMPT", "0"); // never wait on a credential prompt
        let out =
            crate::util::run_with_timeout(cmd, None, Duration::from_secs(120)).map_err(|e| Error::Op(format!("git clone failed: {e}")))?;
        if !out.status.success() {
            return Err(Error::Op(format!("git clone failed: {}", String::from_utf8_lossy(&out.stderr).trim())));
        }
        let brain = Brain::open(Some(&root));
        if !brain.exists() {
            let _ = fs::remove_dir_all(&root);
            return Err(Error::Op(format!("{url} is not a rewire brain (no brain.toml)")));
        }
        let found = unsafe_paths(&brain, None);
        if !found.is_empty() {
            let _ = fs::remove_dir_all(&root);
            return Err(Error::Op(format!(
                "refused {url}: it contains {}; a brain never syncs device-local files (local.toml chooses what runs on \
                 this machine), symlinks or submodules. Remove them from the repository and check who added them",
                found.join(", ")
            )));
        }
        for sub in DIRS {
            fs::create_dir_all(root.join(sub))?;
        }
        // a remote is not the user: its rules, preferences and skills wait for a review on this device too
        // NUL-separated: git quotes non-ASCII paths otherwise
        let list = |dirs: &[&str]| -> Vec<String> {
            let mut args = vec!["ls-files", "-z", "--"];
            args.extend(dirs);
            String::from_utf8_lossy(&brain.git(&args).stdout).split('\0').filter(|p| !p.is_empty()).map(String::from).collect()
        };
        let paths = list(&[":(icase)nodes", ":(icase)skills"]);
        let inbox: Vec<String> = list(&[":(icase)inbox"]).iter().filter_map(|p| brain.review_key(&root.join(p))).collect();
        brain.mark_keys(&inbox)?;
        // REWIRE_REMOTE_TRUSTED: the user said this remote's notes count as reviewed on this machine
        let waiting = if crate::sync::remote_trusted(&brain) { 0 } else { brain.mark_untrusted(&paths)? };
        if waiting > 0 {
            eprintln!(
                "rewire: {waiting} rule(s), preference(s) or skill(s) came with this brain; agents on this device see them \
                 after you review them: `rewire review` (all at once: `rewire review approve all`)"
            );
        }
        let named = device.map(String::from).or_else(|| std::env::var("REWIRE_DEVICE").ok().filter(|d| !d.trim().is_empty()));
        let dev = named.clone().unwrap_or_else(config::default_device);
        if named.is_none() && root.join("events").join(config::slug(&dev)).is_dir() {
            // two machines with one name would write one event log and look like one device
            eprintln!(
                "rewire: another device already syncs as \"{}\" (this machine's host name); give this one its own name \
                 with `rewire init --from <url> --device <name>` unless it is the same machine",
                config::slug(&dev)
            );
        }
        brain.init_local_state(Some(&dev))?;
        Ok(brain)
    }

    fn init_local_state(&self, device: Option<&str>) -> Result<()> {
        // Device-local settings (which LLM runs consolidation) never sync; older brains get the ignore rule here.
        if !self.path(&["local.toml"]).exists() {
            atomic_write(&self.path(&["local.toml"]), config::LOCAL_TOML)?;
        }
        ensure_ignores(self)?;
        #[cfg(unix)]
        {
            // memory holds personal notes and transcripts: readable by this user only
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&self.root, fs::Permissions::from_mode(0o700));
        }
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

    pub fn node_files(&self, suffix: Option<&str>) -> Vec<PathBuf> {
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
        if hits.len() == 1 {
            hits.into_iter().next()
        } else {
            None
        }
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
        self.present()?;
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
        let edges: Vec<Value> =
            nodefmt::edges_of(&meta).into_iter().map(|(rel, to)| json!({"rel": rel, "to": nodefmt::short(&to)})).collect();
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
        self.present()?;
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
        self.present()?;
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

    /// Every event in every device log (torn lines from a crash are skipped).
    pub fn iter_events(&self) -> Vec<Value> {
        let mut files = Vec::new();
        if let Ok(devs) = fs::read_dir(self.path(&["events"])) {
            for d in devs.flatten() {
                if let Ok(fs_) = fs::read_dir(d.path()) {
                    files.extend(fs_.flatten().map(|f| f.path()).filter(|p| p.extension().is_some_and(|e| e == "jsonl")));
                }
            }
        }
        files.sort();
        files
            .iter()
            .filter_map(|p| fs::read_to_string(p).ok())
            .flat_map(|t| t.lines().filter_map(|l| serde_json::from_str(l.trim()).ok()).collect::<Vec<Value>>())
            .collect()
    }

    // ---- device-local state ---------------------------------------------

    pub fn read_state(&self, name: &str) -> Meta {
        fs::read_to_string(self.path(&[".state", &format!("{name}.json")]))
            .ok()
            .and_then(|t| serde_json::from_str::<Meta>(&t).ok())
            .unwrap_or_default()
    }

    pub fn write_state(&self, name: &str, data: &Meta) -> Result<()> {
        self.present()?;
        atomic_write(&self.path(&[".state", &format!("{name}.json")]), &pyjson::dumps_indent1_sorted(&Value::Object(data.clone())))?;
        Ok(())
    }

    /// Read-modify-write a state file under a short lock of its own, so a hook adding an entry and a consolidation
    /// removing others cannot overwrite each other. Independent of the writer lock (hooks must not wait on it).
    pub fn update_state(&self, name: &str, change: impl FnOnce(&mut Meta)) -> Result<()> {
        self.present()?;
        fs::create_dir_all(self.path(&[".state"]))?;
        let fh = fs::OpenOptions::new().create(true).write(true).truncate(false).open(self.path(&[".state", &format!("{name}.lock")]))?;
        let deadline = Instant::now() + Duration::from_secs(5);
        while !FileExt::try_lock_exclusive(&fh).unwrap_or(false) {
            if Instant::now() >= deadline {
                return Err(Error::Locked(format!("state {name} is busy")));
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let mut state = self.read_state(name);
        change(&mut state);
        let written = self.write_state(name, &state);
        let _ = FileExt::unlock(&fh);
        written
    }

    /// Exclusive lock for writers (consolidation, sync).
    pub fn lock(&self, timeout: Duration) -> Result<LockGuard> {
        self.present()?;
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
        run_git(&self.root, args, Duration::from_secs(120)).unwrap_or_else(|e| Output {
            status: failed_status(),
            stdout: Vec::new(),
            stderr: e.to_string().into_bytes(),
        })
    }

    /// Commit everything. A failure other than "nothing to commit" is reported on stderr (and by `doctor`, which
    /// lists uncommitted changes): memory that never reaches git is never synced and cannot be undone.
    pub fn commit(&self, message: &str) -> bool {
        if !self.path(&[".git"]).is_dir() {
            return false;
        }
        remove_stale_lock(&self.root);
        let add = self.git(&["add", "-A"]);
        if !add.status.success() {
            eprintln!("rewire: git add failed: {}", err_str(&add));
            return false;
        }
        // whatever .gitignore says: device-local files (local.toml has the LLM command) never reach a commit
        let staged = self.git(&["diff", "--cached", "--name-only", "-z", "--", ":(icase).state", ":(icase).index", ":(icase)local.toml"]);
        let staged: Vec<String> = String::from_utf8_lossy(&staged.stdout).split('\0').filter(|p| !p.is_empty()).map(String::from).collect();
        if !staged.is_empty() {
            let mut args = vec!["rm", "-r", "-q", "--cached", "--"];
            args.extend(staged.iter().map(String::as_str));
            self.git(&args);
            eprintln!("rewire: kept device-local file(s) out of the commit: {}", staged.join(", "));
        }
        let out = self.git(&["commit", "-q", "-m", message]);
        let quiet = format!("{}{}", out_str(&out), err_str(&out));
        if !out.status.success() && !quiet.contains("nothing to commit") && !quiet.contains("nothing added") {
            eprintln!("rewire: git commit failed: {}", quiet.trim());
        }
        out.status.success()
    }
}

thread_local! {
    static GIT_DEADLINE: std::cell::Cell<Option<Instant>> = const { std::cell::Cell::new(None) };
}

/// Run `f` with every git call on this thread finished within `budget` in total: a hook has seconds, and a pull
/// is a commit, a fetch and a merge.
pub fn within<T>(budget: Duration, f: impl FnOnce() -> T) -> T {
    let end = Instant::now() + budget;
    let _restore = Restore(GIT_DEADLINE.with(|d| d.replace(Some(d.get().map_or(end, |p| p.min(end))))));
    f()
}

/// Run `f` with no total deadline: clean-up (a merge abort) must run even when the budget is spent.
pub fn outside_deadline<T>(f: impl FnOnce() -> T) -> T {
    let _restore = Restore(GIT_DEADLINE.with(|d| d.replace(None)));
    f()
}

/// Puts the previous total deadline back, also when the closure panics.
struct Restore(Option<Instant>);

impl Drop for Restore {
    fn drop(&mut self) {
        GIT_DEADLINE.with(|d| d.set(self.0));
    }
}

/// git on a brain, with a deadline (shortened by `within`). A git killed at the deadline leaves `.git/index.lock`
/// behind, and every later commit would fail: a lock this call created is removed.
pub fn run_git(root: &Path, args: &[&str], timeout: Duration) -> std::io::Result<Output> {
    run_git_with_input(root, args, None, timeout)
}

/// `run_git` with text on stdin (`git cat-file --batch`).
pub fn run_git_with_input(root: &Path, args: &[&str], input: Option<String>, timeout: Duration) -> std::io::Result<Output> {
    let timeout = GIT_DEADLINE.with(|d| d.get()).map_or(timeout, |end| timeout.min(end.saturating_duration_since(Instant::now())));
    let lock = root.join(".git").join("index.lock");
    let had_lock = lock.exists();
    let started = std::time::SystemTime::now();
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(root).args(GIT_CONFIG).args(args).env("GIT_TERMINAL_PROMPT", "0"); // never hang on a credential prompt
    let out = crate::util::run_with_timeout(cmd, input, timeout);
    // the killed git's lock: it did not exist before and was made during this call (writers on a brain take the brain
    // lock, so another rewire's git is not running here; a git the user runs by hand in the brain could be)
    let ours = !had_lock && fs::metadata(&lock).and_then(|m| m.modified()).is_ok_and(|t| t >= started);
    if matches!(&out, Err(e) if e.kind() == std::io::ErrorKind::TimedOut) && ours {
        let _ = fs::remove_file(&lock);
    }
    out
}

/// Remove `.git/index.lock` left by a git that died (older than ten minutes: no git call on a brain runs that long).
fn remove_stale_lock(root: &Path) -> bool {
    let lock = root.join(".git").join("index.lock");
    let old = fs::metadata(&lock).and_then(|m| m.modified()).is_ok_and(|t| t.elapsed().is_ok_and(|age| age > Duration::from_secs(600)));
    if old && fs::remove_file(&lock).is_ok() {
        eprintln!("rewire: removed a stale {} left by a git process that died", lock.display());
        return true;
    }
    false
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
