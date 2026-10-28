//! rewire command line.

use crate::brain::{default_path, Brain};
use crate::consolidate::Consolidator;
use crate::index::Index;
use crate::{integrations, ops, pyjson, snapshot, sync, Error, Meta, Result, VERSION};
use clap::{Parser, Subcommand};
use serde_json::{json, Value};
use std::io::{IsTerminal, Read};
use std::path::PathBuf;
use std::time::Duration;

/// Python `str(value)` for JSON values (strings unquoted, null -> None).
pub fn py_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "None".into(),
        Value::Bool(b) => {
            if *b {
                "True".into()
            } else {
                "False".into()
            }
        }
        other => pyjson::dumps_utf8(other),
    }
}

#[derive(Parser)]
#[command(name = "rewire", version = VERSION, about = "Own your brain, transfer it anywhere.")]
pub struct Cli {
    /// brain directory (default: $REWIRE_BRAIN or ~/brain)
    #[arg(long, global = true)]
    pub brain: Option<PathBuf>,
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand)]
pub enum Cmd {
    /// create a new brain, or attach to an existing one with --from
    Init {
        /// clone an existing brain repository
        #[arg(long = "from", value_name = "URL")]
        from_url: Option<String>,
        /// name of this machine in the event log (default: hostname)
        #[arg(long)]
        device: Option<String>,
    },
    /// print the session-start memory snapshot
    Prime {
        #[arg(long)]
        project: Option<String>,
        #[arg(long)]
        cwd: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// search memory
    Recall {
        #[arg(required = true)]
        query: Vec<String>,
        #[arg(short = 'k', default_value_t = 8)]
        k: usize,
        /// raw transcript excerpts to include (0 = none)
        #[arg(long, default_value_t = 3)]
        episodes: usize,
        /// global | project:<slug> | all
        #[arg(long)]
        scope: Option<String>,
        /// the question is about this period: boost it
        #[arg(long = "from", value_name = "YYYY-MM-DD")]
        since: Option<String>,
        #[arg(long = "to", value_name = "YYYY-MM-DD")]
        until: Option<String>,
        /// an extra query searched too and merged by rank (repeatable, up to 4)
        #[arg(long, value_name = "QUERY")]
        also: Vec<String>,
        #[arg(long)]
        json: bool,
        /// with --json: each memory's whole body, valid_from, recorded_at and source (read without marking it used)
        #[arg(long)]
        full: bool,
    },
    /// read memory notes by id
    Get {
        #[arg(required = true)]
        ids: Vec<String>,
        #[arg(long)]
        session: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// propose a memory (queued for review)
    Remember {
        #[arg(required = true)]
        text: Vec<String>,
        #[arg(long = "type", default_value = "fact", value_parser = ["constraint", "preference", "fact", "decision", "person", "concept"])]
        type_: String,
        /// global | project:<slug> (default: the project of the current directory, else global)
        #[arg(long)]
        scope: Option<String>,
        #[arg(long, default_value_t = 3)]
        importance: i64,
        /// save immediately (interactive terminal only)
        #[arg(long)]
        apply: bool,
    },
    /// consolidate transcripts into memory
    Consolidate {
        files: Vec<PathBuf>,
        /// process pending + newly discovered transcripts
        #[arg(long)]
        pending: bool,
        /// consolidate again the transcripts that failed before (listed by `doctor`)
        #[arg(long)]
        retry: bool,
        #[arg(long, value_parser = ["claude-code", "codex"])]
        harness: Option<String>,
        #[arg(long)]
        quiet: bool,
        #[arg(long)]
        json: bool,
    },
    /// review the inbox
    Review {
        #[arg(default_value = "list", value_parser = ["list", "approve", "reject"])]
        action: String,
        /// approve even if the target note changed since
        #[arg(long)]
        force: bool,
        ids: Vec<String>,
    },
    /// import sessions from JSON lines (transcripts only, or consolidated with --llm); one JSON record per session on stdout
    Import {
        /// JSON lines, `-` for stdin: {"harness", "session", "date", "cwd", "turns": [{"n", "role", "text", "ts"}]}
        file: PathBuf,
        /// consolidate each session through the configured LLM provider (otherwise: transcripts only, no LLM)
        #[arg(long)]
        llm: bool,
    },
    /// rebuild the index
    Rebuild,
    /// health check
    Doctor,
    /// optional semantic search: install the model files, or show whether it is on
    Semantic {
        #[arg(value_parser = ["status", "install"], default_value = "status")]
        action: String,
        /// directory holding the model files (default: the local `wordllama` Python package)
        #[arg(long)]
        from: Option<String>,
    },
    /// rewrite all nodes in the current file format
    Upgrade,
    /// revert a session's consolidation (`rewire sessions` lists them)
    Undo { session: String },
    /// list consolidated and imported sessions, newest first
    Sessions {
        #[arg(short = 'n', default_value_t = 20)]
        limit: usize,
    },
    /// retract a note (by id, short id or title); it stays in git history
    Forget {
        id: String,
        #[arg(long, default_value = "")]
        reason: String,
    },
    /// show or set the git remote used to sync this brain
    Remote {
        #[arg(default_value = "show", value_parser = ["show", "set"])]
        action: String,
        url: Option<String>,
    },
    /// pull and push the brain
    Sync {
        #[arg(long, value_parser = ["pull", "push"])]
        only: Option<String>,
    },
    /// run the MCP server on stdio
    Mcp,
    /// harness lifecycle hook (reads hook JSON on stdin)
    Hook {
        #[arg(value_parser = ["session-start", "user-prompt", "session-end", "pre-compact"])]
        event: String,
        #[arg(long, default_value = "claude-code")]
        harness: String,
    },
    /// connect a harness
    Install {
        /// cloud: Claude Code in a cloud VM that starts empty every session (hooks, MCP server, the brain's remote)
        #[arg(value_parser = ["claude-code", "codex", "cloud"])]
        harness: String,
        /// write Claude Code settings (backs up first)
        #[arg(long)]
        write: bool,
        /// cloud: the brain's git repository, fetched at every session start (an empty new repository becomes one)
        #[arg(long)]
        remote: Option<String>,
        /// cloud: rules, preferences and skills from the remote wait for a review in the VM (which ends with it)
        #[arg(long)]
        review: bool,
        /// Agent Skills directory for codex (default ~/.agents/skills)
        #[arg(long)]
        skills_dir: Option<String>,
    },
}

/// Groups `get` calls from one shell into one session: the parent process id where there is one.
fn shell_id() -> u32 {
    #[cfg(unix)]
    {
        std::os::unix::process::parent_id()
    }
    #[cfg(not(unix))]
    {
        std::process::id()
    }
}

/// What shows this command runs inside a coding agent's shell, if anything: an environment variable the agent
/// sets, or (when that was removed with `env -u`) a Claude Code or Codex process among this process's ancestors.
/// Neither stops an agent that detaches itself first; they keep honest agents and simple tricks out.
fn agent_shell() -> Option<String> {
    let var = ["CLAUDECODE", "AI_AGENT", "CODEX_SANDBOX", "CODEX_SANDBOX_NETWORK_DISABLED", "REWIRE_INTERNAL"]
        .into_iter()
        .find(|k| std::env::var(k).is_ok_and(|v| !v.is_empty()));
    // test suites and benchmark harnesses started from inside an agent session say so (an agent could too: this is
    // a speed bump, not a sandbox)
    let harness = std::env::var("REWIRE_NOT_AN_AGENT").is_ok_and(|v| v == "1");
    var.map(|k| format!("{k} is set")).or_else(|| if harness { None } else { agent_ancestor().map(|p| format!("it runs under {p}")) })
}

fn is_agent_program(argv: &[String]) -> Option<String> {
    let base = |s: &str| s.rsplit('/').next().unwrap_or("").to_string();
    let first = argv.first().map(|a| base(a)).unwrap_or_default();
    if ["claude", "codex"].contains(&first.as_str()) {
        return Some(first);
    }
    // node or another runtime running the agent's script
    argv.iter().take(3).find(|a| a.contains("@anthropic-ai/claude-code") || a.contains("@openai/codex")).map(|a| base(a))
}

#[cfg(target_os = "linux")]
fn agent_ancestor() -> Option<String> {
    let mut pid = std::process::id();
    for _ in 0..32 {
        let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
        // the command name sits in parentheses and may contain spaces: the parent pid is the 2nd field after it
        let ppid: u32 = stat.rsplit_once(')')?.1.split_whitespace().nth(1)?.parse().ok()?;
        if ppid <= 1 {
            return None;
        }
        let cmdline = std::fs::read(format!("/proc/{ppid}/cmdline")).unwrap_or_default();
        let argv: Vec<String> =
            cmdline.split(|b| *b == 0).filter(|a| !a.is_empty()).map(|a| String::from_utf8_lossy(a).into_owned()).collect();
        if let Some(name) = is_agent_program(&argv) {
            return Some(name);
        }
        pid = ppid;
    }
    None
}

#[cfg(all(unix, not(target_os = "linux")))]
fn agent_ancestor() -> Option<String> {
    let mut pid = std::process::id();
    for _ in 0..32 {
        let mut cmd = std::process::Command::new("ps");
        cmd.args(["-o", "ppid=", "-p", &pid.to_string()]);
        let out = crate::util::run_with_timeout(cmd, None, Duration::from_secs(2)).ok()?;
        let ppid: u32 = String::from_utf8_lossy(&out.stdout).trim().parse().ok()?;
        if ppid <= 1 {
            return None;
        }
        let mut cmd = std::process::Command::new("ps");
        cmd.args(["-o", "command=", "-p", &ppid.to_string()]);
        let out = crate::util::run_with_timeout(cmd, None, Duration::from_secs(2)).ok()?;
        let argv: Vec<String> = String::from_utf8_lossy(&out.stdout).split_whitespace().map(String::from).collect();
        if let Some(name) = is_agent_program(&argv) {
            return Some(name);
        }
        pid = ppid;
    }
    None
}

#[cfg(not(unix))]
fn agent_ancestor() -> Option<String> {
    None
}

/// Ask on the controlling terminal (not stdin, which a caller can fill) and read the answer from it.
fn confirm_on_tty(question: &str) -> bool {
    use std::io::{BufRead, Write};
    let Ok(tty) = std::fs::OpenOptions::new().read(true).write(true).open("/dev/tty") else { return false };
    let mut out = &tty;
    if out.write_all(question.as_bytes()).is_err() || out.flush().is_err() {
        return false;
    }
    let mut answer = String::new();
    std::io::BufReader::new(&tty).read_line(&mut answer).is_ok() && matches!(answer.trim().to_lowercase().as_str(), "y" | "yes")
}

fn fail(msg: impl std::fmt::Display) -> ! {
    eprintln!("{msg}");
    std::process::exit(1)
}

/// Writer lock for commands that change files and commit (`git add -A`): without it a background
/// consolidation could sweep their half-done changes into its own commit.
fn writing(brain: &Brain) -> crate::brain::LockGuard {
    match brain.lock(Duration::from_secs(120)) {
        Ok(g) => g,
        Err(e) => fail(format!("{e}; try again in a moment")),
    }
}

fn open_brain(path: &Option<PathBuf>) -> Brain {
    let brain = Brain::open(path.as_deref());
    if !brain.exists() {
        fail(format!("no brain at {} — run `rewire init` (or set REWIRE_BRAIN)", brain.root.display()));
    }
    brain
}

fn print_json(v: &Value) {
    println!("{}", pyjson::dumps_indent1(v));
}

fn cwd() -> String {
    std::env::current_dir().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default()
}

/// Why an import record cannot be imported, if it cannot.
fn import_problem(s: &Value) -> Option<String> {
    let text = |k: &str| s.get(k).and_then(Value::as_str);
    if !s.is_object() {
        return Some("not a JSON object".into());
    }
    if text("session").is_none_or(|v| v.trim().is_empty()) {
        return Some("\"session\" (a non-empty string) is missing".into());
    }
    let Some(turns) = s.get("turns").and_then(Value::as_array).filter(|t| !t.is_empty()) else {
        return Some("\"turns\" must be a non-empty list".into());
    };
    for (i, t) in turns.iter().enumerate() {
        let role = t.get("role").and_then(Value::as_str).unwrap_or("user");
        if !["user", "assistant", "tool", "system"].contains(&role) {
            return Some(format!("turn {}: role must be user, assistant, tool or system, not {role:?}", i + 1));
        }
        if !t.get("text").is_some_and(Value::is_string) {
            return Some(format!("turn {}: \"text\" (a string) is missing", i + 1));
        }
    }
    if let Some(h) = text("harness").filter(|h| crate::util::slugify(h, 40) != *h) {
        return Some(format!("\"harness\" must be a short name of lower-case letters, digits and dashes, not {h:?}"));
    }
    if let Some(d) = text("date").filter(|d| !d.is_empty() && crate::util::valid_day(d).is_none()) {
        return Some(format!("\"date\" must start with YYYY-MM-DD, not {d:?}"));
    }
    None
}

fn print_items(items: &[Value]) {
    println!("{}", crate::snapshot::FENCE_OPEN);
    if items.is_empty() {
        println!("(no matches)");
    }
    for item in items {
        let s = |k: &str| crate::snapshot::escape(&crate::snapshot::printable(&item.get(k).map(py_str).unwrap_or_default()));
        if item.get("error").is_some() {
            println!("! {}: {}", s("ref"), s("error"));
        } else if s("type") == "episode" {
            let excerpt: String = s("text").lines().collect::<Vec<_>>().join(" / ").chars().take(240).collect();
            println!("  ~ {} {}: {}", s("date"), s("session"), excerpt);
        } else if s("type") == "skill" {
            println!("[{}] {}\n{}", s("id"), s("description"), s("body"));
        } else if item.get("body").is_some() {
            println!("[{}] {} · {} · importance {}", crate::util::short_of(&s("id")), s("type"), s("scope"), s("importance"));
            if s("body").trim() == s("title").trim() || s("body").trim().is_empty() {
                println!("  {}\n", s("title"));
            } else {
                println!("  {}\n  {}\n", s("title"), s("body").trim().replace('\n', "\n  "));
            }
        } else {
            let score = item.get("score").and_then(Value::as_f64).unwrap_or(0.0);
            let date = if s("date").is_empty() { "----------".to_string() } else { s("date") };
            let status = match item.get("status").and_then(Value::as_str) {
                Some(st) if st != "active" => format!(" ({st})"),
                _ => String::new(),
            };
            println!("[{}] {:.2} {:<10} {date} {}{status}", crate::util::short_of(&s("id")), score, s("type"), s("title"));
        }
    }
    println!("{}", crate::snapshot::FENCE_CLOSE);
}

pub fn run(cli: Cli) -> Result<()> {
    let bp = &cli.brain;
    match cli.cmd {
        Cmd::Init { from_url, device } => {
            if let Some(url) = from_url {
                let brain = Brain::clone_from(&url, bp.as_deref(), device.as_deref())?;
                println!("brain cloned from {url} to {} (device: {})", brain.root.display(), brain.device());
                if !brain.config.sync_auto {
                    println!("auto sync is off; enable with `rewire remote set <url>`");
                }
            } else {
                let brain = Brain::init(bp.as_deref(), device.as_deref())?;
                println!("brain created at {} (device: {})", brain.root.display(), brain.device());
            }
            println!("next: `rewire install claude-code` and/or `rewire install codex`");
        }
        Cmd::Prime { project, cwd: dir, json: as_json } => {
            let brain = open_brain(bp);
            let mut index = Index::new(&brain);
            index.refresh()?;
            let dir = dir.unwrap_or_else(cwd);
            let snap = snapshot::build(&brain, &mut index, Some(&dir), project.as_deref())?;
            if as_json {
                print_json(&json!({"project": snap.project, "text": snap.text, "ids": snap.ids, "empty": snap.empty}));
            } else {
                println!("{}", snap.text);
            }
        }
        Cmd::Recall { query, k, episodes, scope, since, until, also, json: as_json, full } => {
            let brain = open_brain(bp);
            let mut index = Index::new(&brain);
            index.refresh()?;
            let here = cwd();
            let refers_to = (since.is_some() || until.is_some()).then(|| json!({"from": since, "to": until}));
            let hits = match ops::recall(
                &brain,
                &mut index,
                &query.join(" "),
                k,
                Some(&here),
                scope.as_deref(),
                None,
                episodes,
                refers_to.as_ref(),
                &also,
            ) {
                Err(Error::Op(e)) => fail(&e),
                other => other?,
            };
            let mut items: Vec<Value> = hits.iter().map(|h| h.to_json()).collect();
            if full {
                for item in items.iter_mut().filter(|i| i["type"] != "episode") {
                    let Some(node) = item["id"].as_str().and_then(|id| ops::resolve_id(&brain, id)) else { continue };
                    let blocked = !crate::security::blocked(&format!("{} {}", node.title(), node.body)).is_empty();
                    item["body"] = json!(if blocked { String::new() } else { node.body.clone() });
                    for key in ["valid_from", "recorded_at", "source"] {
                        item[key] = node.meta.get(key).cloned().unwrap_or(Value::Null);
                    }
                }
            }
            if as_json {
                print_json(&Value::Array(items))
            } else {
                print_items(&items)
            }
        }
        Cmd::Import { file, llm: use_llm } => {
            let brain = open_brain(bp);
            let text = if file.as_os_str() == "-" {
                let mut t = String::new();
                std::io::stdin().read_to_string(&mut t)?;
                t
            } else {
                std::fs::read_to_string(&file)?
            };
            // every line is checked before anything is written: a bad line never leaves half an import behind
            let mut sessions = Vec::new();
            let mut problems = Vec::new();
            for (i, line) in text.lines().enumerate().filter(|(_, l)| !l.trim().is_empty()) {
                let at = format!("{}: line {}", file.display(), i + 1);
                match serde_json::from_str::<Value>(line) {
                    Err(e) => problems.push(format!("{at}: not JSON: {e}")),
                    Ok(s) => match import_problem(&s) {
                        Some(p) => problems.push(format!("{at}: {p}")),
                        None => sessions.push(s),
                    },
                }
            }
            if !problems.is_empty() {
                fail(format!("nothing imported; fix these records first:\n  {}", problems.join("\n  ")));
            }
            let usage = crate::llm::Usage::default();
            let provider: Box<dyn crate::llm::Provider> =
                if use_llm { crate::llm::make(&brain.config, &brain.root)? } else { Box::new(crate::llm::NoProvider) };
            let mut con =
                Consolidator::new(&brain, Some(Box::new(crate::llm::Counting { inner: provider, usage: &usage })), Box::new(|_| {}))?;
            con.review_all = true; // an import file says who spoke: what it claims the user said is reviewed
            let _g = writing(&brain);
            let mut written = 0;
            for s in sessions {
                let harness_slug = crate::util::slugify(s.get("harness").and_then(Value::as_str).unwrap_or("import"), 40);
                let session_id = s.get("session").and_then(Value::as_str).unwrap_or("");
                // a transcript-only record (the LLM step failed before) is consolidated again, not skipped
                let done = ops::session_consolidated(&brain, &harness_slug, session_id);
                if done == Some(true) || (!use_llm && done.is_some()) {
                    println!("{}", pyjson::dumps_utf8(&json!({"session": session_id, "ok": true, "skipped": "already imported"})));
                    continue;
                }
                let field = |k: &str| s.get(k).and_then(|v| v.as_str()).map(String::from);
                let harness = field("harness").unwrap_or_else(|| "import".into());
                let session = field("session").unwrap_or_else(|| "unknown".into());
                let turns: Vec<crate::transcripts::Turn> = s["turns"]
                    .as_array()
                    .map(|a| a.as_slice())
                    .unwrap_or_default()
                    .iter()
                    .enumerate()
                    .map(|(i, t)| crate::transcripts::Turn {
                        n: t["n"].as_u64().map(|n| n as usize).unwrap_or(i + 1),
                        role: t["role"].as_str().unwrap_or("user").into(),
                        text: t["text"].as_str().unwrap_or("").into(),
                        ts: t["ts"].as_str().unwrap_or("").into(),
                    })
                    .collect();
                let meta = crate::transcripts::SessionMeta { session: Some(session.clone()), cwd: field("cwd"), date: field("date") };
                let before = (usage.calls.get(), usage.prompt_chars.get(), usage.completion_chars.get(), usage.seconds.get());
                let start = std::time::Instant::now();
                let mut rec = json!({"session": session, "turns": turns.len()});
                let outcome = if use_llm {
                    let r = con.consolidate(&harness, &meta, &turns).map(|r| {
                        for key in ["applied", "queued", "rejected"] {
                            rec[key] = json!(r.get(key).and_then(|v| v.as_array()).map_or(0, |a| a.len()));
                        }
                    });
                    if r.is_err() {
                        // the transcript is kept and searchable even when the LLM part failed (rerun the import to retry)
                        let date = meta.date.clone().unwrap_or_default();
                        let transcript = con.transcript(&turns);
                        let fallback = [crate::consolidate::FALLBACK_SUMMARY.to_string()];
                        let project = con.project_of(meta.cwd.as_deref());
                        con.write_session(
                            &harness,
                            &session,
                            project.as_deref(),
                            &fallback,
                            &[],
                            &[],
                            &[],
                            0,
                            &Default::default(),
                            &date,
                            &transcript,
                            &[],
                        )?;
                    }
                    r
                } else {
                    let date = meta.date.clone().unwrap_or_default();
                    let project = con.project_of(meta.cwd.as_deref());
                    con.write_session(
                        &harness,
                        &session,
                        project.as_deref(),
                        &[],
                        &[],
                        &[],
                        &[],
                        0,
                        &Default::default(),
                        &date,
                        &con.transcript(&turns),
                        &[],
                    )
                };
                rec["ok"] = json!(outcome.is_ok());
                rec["seconds"] = json!(crate::util::round_to(start.elapsed().as_secs_f64(), 2));
                rec["llm_calls"] = json!(usage.calls.get() - before.0);
                rec["prompt_chars"] = json!(usage.prompt_chars.get() - before.1);
                rec["completion_chars"] = json!(usage.completion_chars.get() - before.2);
                rec["llm_seconds"] = json!(crate::util::round_to(usage.seconds.get() - before.3, 2));
                if let Err(e) = &outcome {
                    rec["error"] = json!(e.to_string().chars().take(300).collect::<String>());
                }
                println!("{}", pyjson::dumps_utf8(&rec));
                if outcome.is_err() {
                    if !use_llm && written > 0 {
                        brain.commit(&format!("import {written} session(s)"));
                    }
                    std::process::exit(1); // sessions build on each other: stop at the first failure, a rerun resumes
                }
                written += 1;
            }
            if !use_llm && written > 0 {
                brain.commit(&format!("import {written} session(s)"));
            }
        }
        Cmd::Get { ids, session, json: as_json } => {
            let brain = open_brain(bp);
            let session = session.unwrap_or_else(|| format!("cli-{}", shell_id()));
            let items = ops::get(&brain, &ids, &session, "cli")?;
            let missing = items.iter().any(|i| i.get("error").is_some());
            if as_json {
                print_json(&Value::Array(items))
            } else {
                print_items(&items)
            }
            if missing {
                std::process::exit(1);
            }
        }
        Cmd::Remember { text, type_, scope, importance, apply } => {
            let brain = open_brain(bp);
            let scope = scope.unwrap_or_else(|| ops::default_scope(&brain, Some(&cwd())));
            let text = text.join(" ");
            // typed by a person at their own terminal (not an agent's shell): the user's words, still reviewed
            let provenance = if apply || (std::io::stdin().is_terminal() && agent_shell().is_none()) { "user" } else { "assistant" };
            let result = (|| -> Result<()> {
                if apply {
                    // --apply writes an explicit user memory with no review: only a person at a terminal may do it.
                    // An agent's shell (even one inside a pseudo-terminal) is refused, and the person confirms on
                    // the terminal itself.
                    if let Some(marker) = agent_shell() {
                        fail(format!("--apply is refused inside an agent session ({marker}): agents propose with `rewire remember`, the user approves with `rewire review`"));
                    }
                    if !std::io::stdin().is_terminal()
                        || !confirm_on_tty(&format!("Save as your own explicit {type_}, with no review?\n  {text}\n[y/N] "))
                    {
                        fail("--apply needs a person at the terminal to confirm (agents use the review queue)");
                    }
                    let why = crate::security::scan(&text);
                    if !why.is_empty() {
                        fail(format!("refused: text looks like it contains {}", why.join(" and ")));
                    }
                    let mut change = Meta::new();
                    for (k, v) in [
                        ("action", json!("NEW")),
                        ("type", json!(type_)),
                        ("text", json!(text)),
                        ("scope", json!(scope)),
                        ("importance", json!(importance)),
                        ("confidence", json!("explicit")),
                        ("provenance", json!(provenance)),
                    ] {
                        change.insert(k.into(), v);
                    }
                    let _g = writing(&brain);
                    let node = ops::apply_change(&brain, &change, None)?;
                    ops::write_home(&brain, &mut Index::new(&brain))?;
                    brain.commit(&format!("remember {}", node.id()));
                    println!("saved [{}] {}", crate::util::short_of(node.id()), node.title());
                } else {
                    let id = ops::remember(&brain, &text, &type_, &scope, provenance, Some(json!({"harness": "cli"})))?;
                    println!(
                        "queued for review as {scope}: {id} (approve with `rewire review approve {id}`; --scope global for every project)"
                    );
                }
                Ok(())
            })();
            if let Err(Error::Op(e)) | Err(Error::Schema(e)) = &result {
                fail(e);
            }
            result?;
        }
        Cmd::Consolidate { mut files, pending, retry, harness, quiet, json: as_json } => {
            let brain = open_brain(bp);
            let mut failures = 0;
            if retry {
                // transcripts that are gone cannot be retried: forget their failures
                for gone in brain.read_state("failed").keys().filter(|p| !std::path::Path::new(p.as_str()).is_file()) {
                    eprintln!("skipped {gone}: the transcript file is gone (what was kept of it stays searchable)");
                }
                brain.update_state("failed", |f| f.retain(|p, _| std::path::Path::new(p).is_file()))?;
                files.extend(brain.read_state("failed").keys().map(PathBuf::from));
                if files.is_empty() {
                    println!("no failed consolidations");
                    return Ok(());
                }
            }
            let log: Box<dyn Fn(&str)> = if quiet { Box::new(|_| {}) } else { Box::new(|m: &str| eprintln!("{m}")) };
            let results = (|| -> Result<Vec<Meta>> {
                let mut con = Consolidator::new(&brain, None, log)?;
                let r = if pending || files.is_empty() {
                    con.run_pending(&files)
                } else {
                    // every file is tried; failures are counted and reported, not a reason to stop
                    let mut done = Vec::new();
                    for f in &files {
                        match con.run_file(&crate::brain::absolute(f), harness.as_deref()) {
                            Ok(r) => done.push(r),
                            Err(e @ Error::Locked(_)) => return Err(e),
                            Err(e) => {
                                eprintln!("{}: {e}", f.display());
                                con.failures += 1;
                            }
                        }
                    }
                    Ok(done)
                };
                failures = con.failures;
                r
            })();
            let results = match results {
                Err(Error::Locked(e)) => {
                    eprintln!("skipped: {e}");
                    return Ok(());
                }
                other => other?,
            };
            if brain.config.sync_auto && (pending || retry) {
                match brain.lock(Duration::from_secs(120)) {
                    Ok(_g) => match sync::push(&brain, sync::GIT_TIMEOUT) {
                        Ok(notes) if !quiet => eprintln!("sync: {}", notes.join("; ")),
                        Err(e) if !quiet => eprintln!("sync: push skipped: {e}"),
                        _ => {}
                    },
                    Err(e) if !quiet => eprintln!("sync: push skipped: {e}"),
                    _ => {}
                }
            }
            if as_json {
                print_json(&Value::Array(results.into_iter().map(Value::Object).collect()));
                if failures > 0 {
                    std::process::exit(1);
                }
                return Ok(());
            }
            for r in &results {
                if r.get("status").and_then(Value::as_str) != Some("consolidated") {
                    continue;
                }
                let s = |k: &str| r.get(k).map(py_str).unwrap_or_default();
                let project = r.get("project").and_then(Value::as_str).unwrap_or("-");
                println!("{} {} → project {project}", s("harness"), s("session"));
                for a in r.get("applied").and_then(Value::as_array).cloned().unwrap_or_default() {
                    println!("  + {:<10} {}", py_str(&a["action"]), py_str(&a["title"]));
                }
                for q in r.get("queued").and_then(Value::as_array).cloned().unwrap_or_default() {
                    println!("  ? {:<10} {}  (inbox {})", py_str(&q["action"]), py_str(&q["title"]), py_str(&q["inbox_id"]));
                }
                for x in r.get("rejected").and_then(Value::as_array).cloned().unwrap_or_default() {
                    println!("  ✗ {} ({})", py_str(&x["title"]), py_str(&x["why"]));
                }
            }
            if failures > 0 {
                eprintln!("{failures} transcript(s) could not be consolidated (retry with `rewire consolidate --retry`)");
                std::process::exit(1);
            }
        }
        Cmd::Review { action, force, ids } => {
            let brain = open_brain(bp);
            if action == "list" {
                let items = brain.iter_inbox();
                if items.is_empty() {
                    println!("inbox empty");
                }
                for item in items {
                    let m = &item.meta;
                    let g = |k: &str| crate::snapshot::printable(&m.get(k).map(py_str).unwrap_or_default());
                    let target = m
                        .get("target")
                        .filter(|t| crate::brain::is_set(Some(t)))
                        .map(|t| format!(" → {}", crate::snapshot::printable(&py_str(t))))
                        .unwrap_or_default();
                    let action = crate::snapshot::printable(&m.get("action").map(py_str).unwrap_or_else(|| "NEW".into()));
                    let flags = m.get("flags").filter(|f| f.as_array().is_some_and(|a| !a.is_empty()));
                    let remote =
                        if brain.is_untrusted(&item.path) { "  FROM ANOTHER DEVICE (its reason and labels are its own)" } else { "" };
                    println!(
                        "{}  {action}{target}  [{} · {} · {}/{}]{remote}\n    {}\n    reason: {}{}",
                        g("inbox_id"),
                        g("type"),
                        g("scope"),
                        g("confidence"),
                        g("provenance"),
                        crate::snapshot::printable(item.body.trim()).replace('\n', "\n    "),
                        g("reason"),
                        flags.map(|f| format!("  flags: {}", crate::snapshot::printable(&python_list(f)))).unwrap_or_default()
                    );
                }
                let waiting = ops::unreviewed(&brain);
                if !waiting.is_empty() {
                    println!("\nFrom other devices (shown to agents on this device once you approve them):");
                    for (key, text, why) in waiting {
                        let flag = if why.is_empty() { String::new() } else { format!("  flags: ['{why}']") };
                        // the id to approve or reject it by: a note's short id, a skill's name
                        let id = key
                            .strip_prefix("skills/")
                            .map(|n| format!("skill:{}", n.trim_end_matches('/')))
                            .or_else(|| key.rsplit_once('-').map(|(_, r)| r.trim_end_matches(".md").to_string()))
                            .unwrap_or_else(|| key.clone());
                        // remote text: no terminal escape codes that could hide part of it
                        // in full: what is approved is exactly what was shown
                        let shown = crate::snapshot::printable(&text).replace('\n', "\n    ");
                        println!("{}  ({}){flag}\n    {shown}", crate::snapshot::printable(&id), crate::snapshot::printable(&key));
                    }
                    println!("\napprove: `rewire review approve <id>` · reject (forget it on every device): `rewire review reject <id>`");
                }
                return Ok(());
            }
            if ids.is_empty() {
                fail("give inbox ids, or `all`");
            }
            // approving makes a proposal part of every future session, rejecting drops the user's own proposals:
            // the user's call, not the agent's
            {
                // refused outright: a prompt on the terminal an agent shares with the user could take a keystroke
                // meant for something else
                if let Some(marker) = agent_shell() {
                    fail(format!(
                        "review {action} is refused inside an agent session ({marker}): run `rewire review` in your own terminal"
                    ));
                }
            }
            let _g = writing(&brain);
            let mut errors = 0;
            // rules, preferences and skills from other devices: approving shows them here, rejecting is not offered
            // (it would delete them on every device; edit or delete the file instead)
            let all = ids == ["all"];
            let mut inbox_ids = Vec::new();
            for id in &ids {
                if let Some(key) = (!all).then(|| ops::unreviewed_key(&brain, id)).flatten() {
                    if action == "approve" {
                        ops::trust(&brain, &key)?;
                        println!("approved {key} (from another device)");
                    } else if key.starts_with("nodes/") {
                        // rejecting a rule that came from another device forgets it, on every device once synced
                        match ops::forget_unreviewed(&brain, &key, "rejected on review") {
                            Ok(msg) => println!("rejected {key}: {msg}"),
                            Err(e) => {
                                eprintln!("! {key}: {e}");
                                errors += 1;
                            }
                        }
                    } else {
                        eprintln!("! {key} is a skill from another device: delete its folder to reject it everywhere");
                        errors += 1;
                    }
                } else {
                    inbox_ids.push(id.clone());
                }
            }
            if all && action == "approve" {
                for (key, _, why) in ops::unreviewed(&brain) {
                    if why.is_empty() || force {
                        ops::trust(&brain, &key)?;
                        println!("approved {key} (from another device)");
                    } else {
                        eprintln!("skipped flagged item {key} (['{why}']): check it, then approve it by name");
                    }
                }
            }
            let ids = inbox_ids;
            let ids: Vec<String> = if all {
                // "all" never approves items the scan flagged (injection, secret): those need a look and their id
                // nor proposals that arrived from another device: what they say about themselves is not checked
                let (flagged, clean): (Vec<_>, Vec<_>) = brain.iter_inbox().into_iter().partition(|i| {
                    action == "approve"
                        && !force
                        && (i.meta.get("flags").and_then(Value::as_array).is_some_and(|f| !f.is_empty())
                            || !crate::security::scan(&i.body).is_empty()
                            || brain.is_untrusted(&i.path))
                });
                for item in &flagged {
                    eprintln!(
                        "skipped {} ({}): check it, then approve it by id",
                        item.meta.get("inbox_id").map(py_str).unwrap_or_default(),
                        if brain.is_untrusted(&item.path) {
                            "from another device".to_string()
                        } else {
                            format!("flagged {}", item.meta.get("flags").map(python_list).unwrap_or_else(|| "by a new scan".into()))
                        }
                    );
                }
                clean.iter().map(|i| i.meta.get("inbox_id").map(py_str).unwrap_or_default()).collect()
            } else {
                ids
            };
            for id in &ids {
                let r = if action == "approve" {
                    ops::approve(&brain, id, force).map(|what| println!("approved {id} → {what}"))
                } else {
                    ops::reject(&brain, id).map(|_| println!("rejected {id}"))
                };
                match r {
                    Err(Error::Op(e)) | Err(Error::Schema(e)) => {
                        eprintln!("! {e}");
                        errors += 1;
                    }
                    other => other?,
                }
            }
            ops::write_home(&brain, &mut Index::new(&brain))?;
            brain.commit(&format!("review {action} {} item(s)", ids.len()));
            if errors > 0 {
                std::process::exit(1);
            }
        }
        Cmd::Rebuild => {
            let brain = open_brain(bp);
            println!("indexed {} nodes", Index::new(&brain).rebuild()?);
        }
        Cmd::Doctor => {
            let report = ops::doctor(&open_brain(bp));
            println!("{}", crate::snapshot::printable(&report.join("\n")));
            if report.iter().any(|l| ops::is_problem(l)) {
                std::process::exit(1);
            }
        }
        Cmd::Semantic { action, from } => {
            if action == "install" {
                match crate::semantic::install(from.as_deref()) {
                    Err(Error::Op(e)) => fail(&e),
                    other => println!("{}", other?),
                }
            } else if crate::semantic::available() {
                println!("semantic search: on ({})", crate::semantic::model_dir().display());
            } else if cfg!(feature = "semantic") {
                println!("semantic search: off (no model in {}; run `rewire semantic install`)", crate::semantic::model_dir().display());
            } else {
                println!("semantic search: not built in (build with the `semantic` feature)");
            }
        }
        Cmd::Upgrade => {
            let brain = open_brain(bp);
            let mut count = 0;
            let _g = writing(&brain);
            for node in brain.iter_nodes() {
                brain.write_node(&node.meta, &node.body, Some(&node.path))?;
                count += 1;
            }
            let mut index = Index::new(&brain);
            index.refresh()?;
            ops::write_home(&brain, &mut index)?;
            brain.commit("upgrade node format");
            println!("rewrote {count} nodes in the current format");
        }
        Cmd::Sessions { limit } => {
            let brain = open_brain(bp);
            let list = ops::sessions(&brain);
            if list.is_empty() {
                println!("no sessions yet");
            }
            for (date, harness, session, state) in list.iter().take(limit) {
                println!("{date}  {harness:<12} {session}{}", if state.is_empty() { String::new() } else { format!("  ({state})") });
            }
            if list.len() > limit {
                println!("… {} more (-n {})", list.len() - limit, list.len());
            }
        }
        Cmd::Forget { id, reason } => {
            let brain = open_brain(bp);
            // forgetting a rule changes every future session as much as adding one: the user's call
            if let Some(marker) = agent_shell() {
                fail(format!("forget is refused inside an agent session ({marker}): run it in your own terminal"));
            }
            let _g = writing(&brain);
            match ops::forget(&brain, &id, &reason) {
                Ok(msg) => println!("{msg}"),
                Err(Error::Op(e)) => fail(e),
                Err(e) => return Err(e),
            }
        }
        Cmd::Undo { session } => {
            let brain = open_brain(bp);
            // undo removes notes, approved ones too: the user's call, like forget
            if let Some(marker) = agent_shell() {
                fail(format!("undo is refused inside an agent session ({marker}): run it in your own terminal"));
            }
            let undone = {
                let _g = writing(&brain);
                ops::undo(&brain, &session)
            };
            match undone {
                Ok(summary) => println!("{summary}"),
                Err(e) => fail(e),
            }
        }
        Cmd::Remote { action, url } => {
            let mut brain = open_brain(bp);
            if action == "set" {
                // where the whole brain is pushed to: the user's call
                if let Some(marker) = agent_shell() {
                    fail(format!("remote set is refused inside an agent session ({marker}): run it in your own terminal"));
                }
                let Some(url) = url else { fail("usage: rewire remote set <git-url>") };
                let notes = {
                    let _g = writing(&brain);
                    sync::set_remote(&mut brain, &url)?
                };
                println!("{}", notes.join("\n"));
            } else {
                let url = sync::remote_url(&brain);
                println!(
                    "origin: {}; auto sync: {}; device: {}",
                    if url.is_empty() { "(none)".to_string() } else { ops::redact_url(&url) },
                    if brain.config.sync_auto { "on" } else { "off" },
                    brain.device()
                );
            }
        }
        Cmd::Sync { only } => {
            let brain = open_brain(bp);
            let res = (|| -> Result<()> {
                let _g = brain.lock(Duration::from_secs(120))?;
                if only.as_deref() != Some("push") {
                    for n in sync::pull(&brain, sync::GIT_TIMEOUT)? {
                        println!("{n}");
                    }
                }
                if only.as_deref() != Some("pull") {
                    for n in sync::push(&brain, sync::GIT_TIMEOUT)? {
                        println!("{n}");
                    }
                }
                Ok(())
            })();
            if let Err(e) = res {
                fail(format!("sync failed: {e}"));
            }
        }
        Cmd::Mcp => {
            let brain = Brain::open(bp.as_deref());
            let url = sync::env_remote();
            if !brain.exists() && !url.is_empty() {
                // a cloud session starts the MCP server and the session-start hook together: either may clone first.
                // Only the hook turns an empty repository into a brain (two would push two); here, wait for it.
                if let Err(e) = Brain::fetch(&url, &brain.root, false) {
                    let until = std::time::Instant::now() + std::time::Duration::from_secs(45);
                    while !brain.exists() && std::time::Instant::now() < until {
                        std::thread::sleep(std::time::Duration::from_millis(500));
                    }
                    if !brain.exists() {
                        fail(format!("could not fetch the brain from REWIRE_REMOTE: {e}"));
                    }
                }
            }
            crate::mcp::serve(&open_brain(bp))?
        }
        Cmd::Hook { event, harness } => {
            let brain = Brain::open(bp.as_deref());
            let mut raw = String::new();
            let payload: Value = if std::io::stdin().is_terminal() {
                json!({})
            } else {
                let _ = std::io::stdin().read_to_string(&mut raw);
                serde_json::from_str(if raw.trim().is_empty() { "{}" } else { &raw }).unwrap_or_else(|_| json!({}))
            };
            // a memory hook must never break the host session
            match integrations::hook(brain, &event, &payload, &harness) {
                Ok(out) if !out.is_empty() => println!("{out}"),
                Ok(_) => {}
                Err(e) => eprintln!("rewire hook error: {e}"),
            }
        }
        Cmd::Install { harness, write, skills_dir, remote, review } => {
            if harness == "cloud" {
                // which repository the whole brain syncs with: the user's call, as with `remote set`
                if let Some(marker) = agent_shell() {
                    fail(format!("install cloud is refused inside an agent session ({marker}): run it in the environment's setup script"));
                }
                let Some(url) = remote.filter(|u| !u.trim().is_empty()) else { fail("usage: rewire install cloud --remote <git-url>") };
                println!("{}", integrations::install_cloud(url.trim(), !review, &Brain::open(bp.as_deref()).root)?);
            } else if harness == "claude-code" {
                println!("{}", integrations::install_claude_code(write, None, Some(&Brain::open(bp.as_deref()).root))?);
            } else {
                println!("{}", integrations::install_codex(skills_dir.as_deref())?);
            }
        }
    }
    Ok(())
}

/// Python list repr of strings: ['secret', 'injection'].
fn python_list(v: &Value) -> String {
    let items: Vec<String> = v.as_array().map(|a| a.iter().map(|x| format!("'{}'", py_str(x))).collect()).unwrap_or_default();
    format!("[{}]", items.join(", "))
}

pub fn default_brain_path() -> PathBuf {
    default_path()
}
