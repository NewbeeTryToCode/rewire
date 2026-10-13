use indexmap::IndexMap;
use std::path::Path;

pub const DEFAULT_TOML: &str = r#"# rewire brain configuration
schema = 1

[activation]
model = "actr"
d = 0.7                  # decay; tuned in experiments/phase0
recall_beta = 0.3        # weight of activation in recall ranking (0 = off)

[budgets]                # characters (~4 chars per token)
profile = 600            # "About the user": people and the most important global facts
constraints = 1500
preferences = 800
working = 2500

[consolidation]
# provider: "command" runs a CLI that reads the prompt on stdin and prints text,
#           e.g. "claude -p" (uses your existing Claude Code login)
# provider: "anthropic" calls the Anthropic Messages API (needs ANTHROPIC_API_KEY)
# profile: "coding" keeps durable working knowledge only (default);
#          "general" remembers people, events and plans (a personal second brain).
profile = "coding"
# transcripts: keep a searchable, secret-redacted transcript in sessions/ ("episodes").
#              "auto" = user turns for coding, everything for general; or "user" / "all" / "none".
transcripts = "auto"
provider = "command"
command = "claude -p --output-format text"
model = "claude-opus-5-5"
effort = "medium"
max_chunk_chars = 60000

[sync]
# Set by `rewire remote set <url>`: pull at session start, push after consolidation.
auto = false

# Map repositories to project slugs. Without a match the git top-level
# directory name is used.
# [projects.myproject]
# match = ["github.com/me/myproject", "~/code/myproject"]
"#;

#[derive(Debug, Clone)]
pub struct Budgets {
    pub profile: usize,
    pub constraints: usize,
    pub preferences: usize,
    pub working: usize,
    pub skills: usize,
}

#[derive(Debug, Clone)]
pub struct Config {
    /// legacy: the device used to live in brain.toml (shared!); see Brain::device
    pub device: String,
    pub sync_auto: bool,
    pub d: f64,
    pub recall_beta: f64,
    pub budgets: Budgets,
    pub profile: String,
    pub transcripts: String,
    pub provider: String,
    pub command: String,
    pub model: String,
    pub effort: String,
    pub max_chunk_chars: usize,
    /// slug -> match patterns
    pub projects: IndexMap<String, Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            device: String::new(),
            sync_auto: false,
            d: 0.7,
            recall_beta: 0.3,
            budgets: Budgets { profile: 600, constraints: 1500, preferences: 800, working: 2500, skills: 400 },
            profile: "coding".into(),
            transcripts: "auto".into(),
            provider: "command".into(),
            command: "claude -p --output-format text".into(),
            model: "claude-opus-5-5".into(),
            effort: "medium".into(),
            max_chunk_chars: 60000,
            projects: IndexMap::new(),
        }
    }
}

/// `re.sub(r"[^a-z0-9_-]+", "-", text.lower()).strip("-") or "default"`
pub fn slug(text: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in text.to_lowercase().chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-' {
            out.push(c);
            dash = false;
        } else if !dash {
            out.push('-');
            dash = true;
        }
    }
    let t = out.trim_matches('-');
    if t.is_empty() { "default".into() } else { t.into() }
}

pub fn default_device() -> String {
    let host = std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("HOSTNAME").ok())
        .or_else(|| {
            std::process::Command::new("hostname").output().ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        })
        .unwrap_or_default();
    let first = host.split('.').next().unwrap_or("").to_string();
    if first.is_empty() { "default".into() } else { first.to_lowercase() }
}

fn num(v: Option<&toml::Value>) -> Option<f64> {
    match v? {
        toml::Value::Float(f) => Some(*f),
        toml::Value::Integer(i) => Some(*i as f64),
        toml::Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

fn string(v: Option<&toml::Value>) -> Option<String> {
    v.and_then(|v| v.as_str()).map(String::from)
}

pub fn load(path: &Path) -> Config {
    let mut cfg = Config::default();
    let Ok(text) = std::fs::read_to_string(path) else { return cfg };
    let Ok(raw) = text.parse::<toml::Table>() else { return cfg };
    let section = |name: &str| raw.get(name).and_then(|v| v.as_table()).cloned().unwrap_or_default();
    let act = section("activation");
    let con = section("consolidation");
    let budgets = section("budgets");
    cfg.device = string(raw.get("device")).unwrap_or_default();
    cfg.sync_auto = section("sync").get("auto").and_then(|v| v.as_bool()).unwrap_or(false);
    cfg.d = num(act.get("d")).unwrap_or(cfg.d);
    cfg.recall_beta = num(act.get("recall_beta")).unwrap_or(cfg.recall_beta);
    if let Some(v) = num(budgets.get("profile")) {
        cfg.budgets.profile = v as usize;
    }
    if let Some(v) = num(budgets.get("constraints")) {
        cfg.budgets.constraints = v as usize;
    }
    if let Some(v) = num(budgets.get("preferences")) {
        cfg.budgets.preferences = v as usize;
    }
    if let Some(v) = num(budgets.get("skills")) {
        cfg.budgets.skills = v as usize;
    }
    if let Some(v) = num(budgets.get("working")) {
        cfg.budgets.working = v as usize;
    }
    cfg.profile = string(con.get("profile")).unwrap_or(cfg.profile);
    cfg.transcripts = string(con.get("transcripts")).unwrap_or(cfg.transcripts);
    cfg.provider = string(con.get("provider")).unwrap_or(cfg.provider);
    cfg.command = string(con.get("command")).unwrap_or(cfg.command);
    cfg.model = string(con.get("model")).unwrap_or(cfg.model);
    cfg.effort = string(con.get("effort")).unwrap_or(cfg.effort);
    if let Some(v) = num(con.get("max_chunk_chars")) {
        cfg.max_chunk_chars = v as usize;
    }
    if let Some(projects) = raw.get("projects").and_then(|v| v.as_table()) {
        for (slug, spec) in projects {
            let patterns = spec
                .get("match")
                .and_then(|m| m.as_array())
                .map(|a| a.iter().filter_map(|p| p.as_str().map(String::from)).collect())
                .unwrap_or_default();
            cfg.projects.insert(slug.clone(), patterns);
        }
    }
    cfg
}
