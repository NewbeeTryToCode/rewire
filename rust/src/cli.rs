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
        Value::Bool(b) => if *b { "True".into() } else { "False".into() },
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
        #[arg(long, default_value = "global")]
        scope: String,
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
    /// revert a session's consolidation
    Undo { session: String },
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
        #[arg(value_parser = ["claude-code", "codex"])]
        harness: String,
        /// write Claude Code settings (backs up first)
        #[arg(long)]
        write: bool,
        /// Agent Skills directory for codex (default ~/.agents/skills)
        #[arg(long)]
        skills_dir: Option<String>,
    },
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

fn print_items(items: &[Value]) {
    println!("{}", crate::snapshot::FENCE_OPEN);
    for item in items {
        let s = |k: &str| crate::snapshot::escape(&item.get(k).map(py_str).unwrap_or_default());
        if item.get("error").is_some() {
            println!("! {}: {}", s("ref"), s("error"));
        } else if s("type") == "episode" {
            let excerpt: String = s("text").lines().collect::<Vec<_>>().join(" / ").chars().take(240).collect();
            println!("  ~ {} {}: {}", s("date"), s("session"), excerpt);
        } else if s("type") == "skill" {
            println!("[{}] {}\n{}", s("id"), s("description"), s("body"));
        } else if item.get("body").is_some() {
            println!("[{}] {} · {} · importance {}", crate::util::short_of(&s("id")), s("type"), s("scope"), s("importance"));
            println!("  {}\n  {}\n", s("title"), s("body"));
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
            let hits = match ops::recall(&brain, &mut index, &query.join(" "), k, Some(&here), scope.as_deref(), None, episodes, refers_to.as_ref(), &also) {
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
            if as_json { print_json(&Value::Array(items)) } else { print_items(&items) }
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
            let usage = crate::llm::Usage::default();
            let provider: Box<dyn crate::llm::Provider> =
                if use_llm { crate::llm::make(&brain.config, &brain.root)? } else { Box::new(crate::llm::NoProvider) };
            let mut con = Consolidator::new(&brain, Some(Box::new(crate::llm::Counting { inner: provider, usage: &usage })), Box::new(|_| {}))?;
            let _g = writing(&brain);
            let mut written = 0;
            for line in text.lines().filter(|l| !l.trim().is_empty()) {
                let s: Value = serde_json::from_str(line).map_err(|e| Error::Op(format!("bad session line: {e}")))?;
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
                    con.consolidate(&harness, &meta, &turns).map(|r| {
                        for key in ["applied", "queued", "rejected"] {
                            rec[key] = json!(r.get(key).and_then(|v| v.as_array()).map_or(0, |a| a.len()));
                        }
                    })
                } else {
                    let date = meta.date.clone().unwrap_or_default();
                    con.write_session(&harness, &session, None, &[], &[], &[], &[], 0, &Default::default(), &date, &con.transcript(&turns), &[])
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
            let session = session.unwrap_or_else(|| format!("cli-{}", std::os::unix::process::parent_id()));
            let items = ops::get(&brain, &ids, &session, "cli")?;
            if as_json { print_json(&Value::Array(items)) } else { print_items(&items) }
        }
        Cmd::Remember { text, type_, scope, importance, apply } => {
            let brain = open_brain(bp);
            let text = text.join(" ");
            let provenance = if apply { "user" } else { "assistant" };
            let result = (|| -> Result<()> {
                if apply {
                    if !std::io::stdin().is_terminal() {
                        fail("--apply is only allowed from an interactive terminal (agents use the review queue)");
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
                    println!("queued for review: {id} (approve with `rewire review approve {id}`)");
                }
                Ok(())
            })();
            if let Err(Error::Op(e)) | Err(Error::Schema(e)) = &result {
                fail(e);
            }
            result?;
        }
        Cmd::Consolidate { files, pending, harness, quiet, json: as_json } => {
            let brain = open_brain(bp);
            let log: Box<dyn Fn(&str)> = if quiet { Box::new(|_| {}) } else { Box::new(|m: &str| eprintln!("{m}")) };
            let results = (|| -> Result<Vec<Meta>> {
                let mut con = Consolidator::new(&brain, None, log)?;
                if pending || files.is_empty() {
                    con.run_pending(&files)
                } else {
                    files
                        .iter()
                        .map(|f| con.run_file(&crate::brain::absolute(f), harness.as_deref()))
                        .collect()
                }
            })();
            let results = match results {
                Err(Error::Locked(e)) => {
                    eprintln!("skipped: {e}");
                    return Ok(());
                }
                other => other?,
            };
            if brain.config.sync_auto && pending {
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
                    let g = |k: &str| m.get(k).map(py_str).unwrap_or_default();
                    let target = m.get("target").filter(|t| crate::brain::is_set(Some(t))).map(|t| format!(" → {}", py_str(t))).unwrap_or_default();
                    let action = m.get("action").map(py_str).unwrap_or_else(|| "NEW".into());
                    let flags = m.get("flags").filter(|f| f.as_array().is_some_and(|a| !a.is_empty()));
                    println!(
                        "{}  {action}{target}  [{} · {} · {}/{}]\n    {}\n    reason: {}{}",
                        g("inbox_id"),
                        g("type"),
                        g("scope"),
                        g("confidence"),
                        g("provenance"),
                        item.body.trim(),
                        g("reason"),
                        flags.map(|f| format!("  flags: {}", python_list(f))).unwrap_or_default()
                    );
                }
                return Ok(());
            }
            if ids.is_empty() {
                fail("give inbox ids, or `all`");
            }
            let _g = writing(&brain);
            let ids: Vec<String> = if ids == ["all"] {
                brain.iter_inbox().iter().map(|i| i.meta.get("inbox_id").map(py_str).unwrap_or_default()).collect()
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
                    Err(Error::Op(e)) | Err(Error::Schema(e)) => eprintln!("! {e}"),
                    other => other?,
                }
            }
            ops::write_home(&brain, &mut Index::new(&brain))?;
            brain.commit(&format!("review {action} {} item(s)", ids.len()));
        }
        Cmd::Rebuild => {
            let brain = open_brain(bp);
            println!("indexed {} nodes", Index::new(&brain).rebuild()?);
        }
        Cmd::Doctor => println!("{}", ops::doctor(&open_brain(bp)).join("\n")),
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
        Cmd::Undo { session } => {
            let brain = open_brain(bp);
            let undone = {
                let _g = writing(&brain);
                ops::undo(&brain, &session)
            };
            match undone {
                Ok(sha) => println!("reverted {sha}"),
                Err(e) => fail(e),
            }
        }
        Cmd::Remote { action, url } => {
            let mut brain = open_brain(bp);
            if action == "set" {
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
                    if url.is_empty() { "(none)" } else { &url },
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
        Cmd::Mcp => crate::mcp::serve(&open_brain(bp))?,
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
        Cmd::Install { harness, write, skills_dir } => {
            if harness == "claude-code" {
                println!("{}", integrations::install_claude_code(write, None)?);
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
