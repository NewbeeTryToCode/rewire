use indexmap::IndexMap;
use std::path::Path;

pub const DEFAULT_TOML: &str = r#"# rewire brain configuration
schema = 1

[activation]
model = "actr"
d = 0.7                  # decay; tuned in experiments/phase0
recall_beta = 0.0        # weight of activation in recall ranking (0 = off); experimental, never ablated, so off
spread = "supersedes"    # recall follows links: "supersedes" (an outdated note leads to its replacement), "full"
                         # (2-hop spreading over every link; experimental, +0.0 measured) or "off"

[budgets]                # characters (~4 chars per token)
profile = 600            # "About the user": people and the most important global facts
constraints = 1500
preferences = 800
working = 2500

[consolidation]
# Which LLM consolidates (provider, command, model, effort) is set per device in local.toml, which is not
# synced: a value here would reach, and run on, every device that pulls this brain.
# profile: "coding" keeps durable working knowledge only (default);
#          "general" remembers people, events and plans (a personal second brain).
profile = "coding"
# transcripts: keep a searchable, secret-redacted transcript in sessions/ ("episodes").
#              "auto" = user turns for coding, everything for general; or "user" / "all" / "none".
transcripts = "auto"
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
    /// which links recall follows: "supersedes", "full" or "off"
    pub spread: String,
    pub budgets: Budgets,
    pub profile: String,
    /// in effect on this device: local.toml, else brain.toml
    pub transcripts: String,
    /// as brain.toml (synced) sets it; a pull that changes it is reported
    pub synced_transcripts: String,
    pub synced_profile: String,
    /// local.toml pins transcripts / profile on this device
    pub local_transcripts: bool,
    pub local_profile: bool,
    pub provider: String,
    pub command: String,
    pub model: String,
    pub effort: String,
    pub max_chunk_chars: usize,
    /// brain.toml could not be parsed: defaults are used, and `doctor` and the snapshot say so
    pub error: Option<String>,
    /// brain.toml (synced) tried to set provider/command/model/effort: ignored, reported by `doctor`
    pub synced_llm_settings: Vec<String>,
    /// slug -> match patterns
    pub projects: IndexMap<String, Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            device: String::new(),
            sync_auto: false,
            d: 0.7,
            recall_beta: 0.0,
            spread: "supersedes".into(),
            budgets: Budgets { profile: 600, constraints: 1500, preferences: 800, working: 2500, skills: 400 },
            profile: "coding".into(),
            transcripts: "auto".into(),
            synced_transcripts: "auto".into(),
            synced_profile: "coding".into(),
            local_transcripts: false,
            local_profile: false,
            provider: "command".into(),
            command: DEFAULT_COMMAND.into(),
            model: "claude-sonnet-5-5".into(),
            effort: "medium".into(),
            max_chunk_chars: 60000,
            error: None,
            synced_llm_settings: Vec::new(),
            projects: IndexMap::new(),
        }
    }
}

/// Device-local settings (not synced): which LLM consolidates. Written by `rewire init`.
pub const LOCAL_TOML: &str = r#"# rewire settings for this device only (not synced; see .gitignore)

[consolidation]
# provider: "command" runs a CLI that reads the prompt on stdin and prints text (default: Claude Code's
#           `claude -p`, with your existing login, no tools, no MCP servers and no settings or hooks)
# provider: "anthropic" calls the Anthropic Messages API (needs ANTHROPIC_API_KEY)
provider = "command"
command = "claude -p --model claude-sonnet-5-5 --output-format text --tools \"\" --strict-mcp-config --setting-sources \"\""
model = "claude-sonnet-5-5"
effort = "medium"
# transcripts: pin how much of each transcript this device keeps in sessions/ ("user" / "all" / "none"),
#              whatever the synced brain.toml says
# transcripts = "user"
"#;

/// The default consolidation command: the model sees transcripts (untrusted text), so it gets no tools,
/// no MCP servers and none of the user's settings or hooks.
pub const DEFAULT_COMMAND: &str =
    "claude -p --model claude-sonnet-5-5 --output-format text --tools \"\" --strict-mcp-config --setting-sources \"\"";

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
    if t.is_empty() {
        "default".into()
    } else {
        t.into()
    }
}

pub fn default_device() -> String {
    let host = std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("HOSTNAME").ok())
        .or_else(|| {
            crate::util::run_with_timeout(std::process::Command::new("hostname"), None, std::time::Duration::from_secs(2))
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        })
        .unwrap_or_default();
    let first = host.split('.').next().unwrap_or("").to_string();
    if first.is_empty() {
        "default".into()
    } else {
        first.to_lowercase()
    }
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
    // a brain.toml that does not parse still lets local.toml decide (and keeps no transcripts unless it says so)
    let (raw, broken) = match text.parse::<toml::Table>() {
        Ok(raw) => (raw, false),
        Err(e) => {
            cfg.error = Some(format!("brain.toml: {}", e.message()));
            (toml::Table::new(), true)
        }
    };
    let section = |name: &str| raw.get(name).and_then(|v| v.as_table()).cloned().unwrap_or_default();
    let act = section("activation");
    let con = section("consolidation");
    let budgets = section("budgets");
    cfg.device = string(raw.get("device")).unwrap_or_default();
    cfg.sync_auto = section("sync").get("auto").and_then(|v| v.as_bool()).unwrap_or(false);
    cfg.d = num(act.get("d")).unwrap_or(cfg.d);
    cfg.recall_beta = num(act.get("recall_beta")).unwrap_or(cfg.recall_beta);
    if let Some(v) = string(act.get("spread")).filter(|v| ["supersedes", "full", "off"].contains(&v.as_str())) {
        cfg.spread = v;
    }
    if let Some(v) = num(budgets.get("profile")) {
        cfg.budgets.profile = (v.max(0.0) as usize).max(200); // a synced zero must not hide the user's own notes
    }
    if let Some(v) = num(budgets.get("constraints")) {
        cfg.budgets.constraints = (v.max(0.0) as usize).max(400); // a synced zero must not hide the user's own notes
    }
    if let Some(v) = num(budgets.get("preferences")) {
        cfg.budgets.preferences = (v.max(0.0) as usize).max(200); // a synced zero must not hide the user's own notes
    }
    if let Some(v) = num(budgets.get("skills")) {
        cfg.budgets.skills = (v.max(0.0) as usize).max(200); // a synced zero must not hide the user's own notes
    }
    if let Some(v) = num(budgets.get("working")) {
        cfg.budgets.working = (v.max(0.0) as usize).max(400); // a synced zero must not hide the user's own notes
    }
    // only known values: a synced string is never passed on as is
    cfg.synced_profile = string(con.get("profile")).filter(|v| ["coding", "general"].contains(&v.as_str())).unwrap_or(cfg.synced_profile);
    cfg.synced_transcripts =
        string(con.get("transcripts")).filter(|v| ["auto", "user", "all", "none"].contains(&v.as_str())).unwrap_or(cfg.synced_transcripts);
    // what runs on this machine comes from local.toml only; brain.toml syncs from the remote
    const LEGACY_DEFAULTS: [&str; 3] = ["command", "claude -p --output-format text", "claude-opus-5-5"];
    for key in ["provider", "command", "model", "effort"] {
        if let Some(v) = string(con.get(key)) {
            if !LEGACY_DEFAULTS.contains(&v.as_str()) && !(key == "effort" && v == "medium") {
                cfg.synced_llm_settings.push(key.to_string());
            }
        }
    }
    let local_text = path.parent().and_then(|dir| std::fs::read_to_string(dir.join("local.toml")).ok());
    let local_table = local_text.map(|t| t.parse::<toml::Table>());
    if let Some(Err(e)) = &local_table {
        cfg.error = Some(format!("local.toml: {}", e.message()));
    }
    let local =
        local_table.and_then(|t| t.ok()).and_then(|t| t.get("consolidation").and_then(|v| v.as_table()).cloned()).unwrap_or_default();
    cfg.provider = string(local.get("provider")).unwrap_or(cfg.provider);
    cfg.command = string(local.get("command")).unwrap_or(cfg.command);
    cfg.model = string(local.get("model")).unwrap_or(cfg.model);
    cfg.effort = string(local.get("effort")).unwrap_or(cfg.effort);
    // how much of each transcript is kept (and synced) can be pinned per device
    cfg.local_transcripts = local.contains_key("transcripts");
    cfg.transcripts = string(local.get("transcripts"))
        .filter(|v| ["auto", "user", "all", "none"].contains(&v.as_str()))
        .unwrap_or_else(|| cfg.synced_transcripts.clone());
    cfg.local_profile = local.contains_key("profile");
    cfg.profile =
        string(local.get("profile")).filter(|v| ["coding", "general"].contains(&v.as_str())).unwrap_or_else(|| cfg.synced_profile.clone());
    if let Some(v) = num(con.get("max_chunk_chars")) {
        cfg.max_chunk_chars = v as usize;
    }
    // settings that do not parse: keep no transcripts unless a readable local.toml says otherwise
    if (broken && !cfg.local_transcripts) || cfg.error.as_deref().is_some_and(|e| e.starts_with("local.toml")) {
        cfg.transcripts = "none".into();
    }
    if let Some(projects) = raw.get("projects").and_then(|v| v.as_table()) {
        // project names are shown to agents: only plain slugs
        for (slug, spec) in projects.iter().filter(|(s, _)| !s.is_empty() && slug(s) == **s && s.len() <= 48) {
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
