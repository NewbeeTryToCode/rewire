//! Fixtures and helpers shared by every module of the suite.
//!
//! Anything here is `pub` for the sibling test modules; it is not part of the crate under test.

use rewire::brain::Brain;
use rewire::consolidate::Consolidator;
use rewire::index::{Hit, Index, SearchOpts};
use rewire::llm::Provider;
use rewire::{ops, sync, Meta, Result};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

pub static SERIAL: Mutex<()> = Mutex::new(());

pub fn fix(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(name)
}

pub fn m(v: Value) -> Meta {
    v.as_object().unwrap().clone()
}

pub struct Case {
    _tmp: tempfile::TempDir,
    pub dir: PathBuf,
    pub brain: Brain,
    pub index: Index,
}

impl Case {
    pub fn new() -> Case {
        std::env::set_var("REWIRE_SEMANTIC", "0"); // keyword ranking only: tests must not depend on a local model
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_path_buf();
        let brain = Brain::init(Some(&dir.join("brain")), Some("test")).unwrap();
        let index = Index::new(&brain);
        Case { _tmp: tmp, dir, brain, index }
    }

    pub fn add(&mut self, type_: &str, text: &str, extra: Value) -> rewire::brain::Node {
        let mut change = m(json!({"action": "NEW", "type": type_, "text": text, "scope": "global", "importance": 3,
                                   "confidence": "explicit", "provenance": "user"}));
        for (k, v) in extra.as_object().unwrap() {
            change.insert(k.clone(), v.clone());
        }
        let node = ops::apply_change(&self.brain, &change, None).unwrap();
        self.index.refresh().unwrap();
        node
    }

    pub fn search(&mut self, q: &str, scopes: &[&str]) -> Vec<Hit> {
        let opts = SearchOpts { scopes: Some(scopes.iter().map(|s| s.to_string()).collect()), ..Default::default() };
        self.index.search(q, &opts).unwrap()
    }

    pub fn reopen(&self) -> Brain {
        Brain::open(Some(&self.brain.root))
    }
}

pub fn set_spread(c: &mut Case, mode: &str) {
    let toml = c.brain.path(&["brain.toml"]);
    let text = std::fs::read_to_string(&toml).unwrap();
    std::fs::write(
        &toml,
        regex::Regex::new(r#"(?m)^spread = "\w+""#).unwrap().replace(&text, format!("spread = \"{mode}\"").as_str()).as_ref(),
    )
    .unwrap();
    c.brain = c.reopen();
    c.index = Index::new(&c.brain);
    assert_eq!(c.brain.config.spread, mode);
}

pub fn titles(hits: &[Hit]) -> Vec<String> {
    hits.iter().map(|h| h.title.clone()).collect()
}

pub fn md_count(dir: &Path) -> usize {
    std::fs::read_dir(dir).unwrap().flatten().filter(|e| e.file_name().to_string_lossy().ends_with(".md")).count()
}

pub fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap()
}

pub static PROMPTS: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub struct Fake {
    responses: RefCell<VecDeque<Meta>>,
}

impl Fake {
    pub fn new(r: Vec<Value>) -> Box<Fake> {
        Box::new(Fake { responses: RefCell::new(r.into_iter().map(m).collect()) })
    }
}

impl Provider for Fake {
    fn complete_json(&self, _system: &str, prompt: &str, _schema: &Value) -> Result<Meta> {
        PROMPTS.lock().unwrap_or_else(|e| e.into_inner()).push(prompt.to_string());
        Ok(self.responses.borrow_mut().pop_front().expect("unexpected LLM call"))
    }
}

pub fn extraction(claims: Vec<Value>, used: Vec<String>) -> Value {
    json!({"summary": "s", "claims": claims, "used_memory_ids": used})
}

pub fn claim(text: &str, turn: i64, type_: &str) -> Value {
    json!({"text": text, "title": text, "type": type_, "scope": "global", "importance": 3, "confidence": "explicit",
           "evidence_turn": turn})
}

pub fn run_fixture(brain: &Brain, provider: Box<Fake>) -> Meta {
    let mut con = Consolidator::new(brain, Some(provider), Box::new(|_| {})).unwrap();
    con.run_file(&fix("claude_code.jsonl"), None).unwrap()
}

pub fn list_titles(v: &Value) -> Vec<String> {
    v.as_array().unwrap().iter().map(|x| x["title"].as_str().unwrap().to_string()).collect()
}

pub fn as_user(cmd: &mut Command) -> &mut Command {
    for k in ["CLAUDECODE", "AI_AGENT", "CODEX_SANDBOX", "CODEX_SANDBOX_NETWORK_DISABLED", "REWIRE_INTERNAL"] {
        cmd.env_remove(k);
    }
    cmd.env("REWIRE_NOT_AN_AGENT", "1") // `cargo test` itself may run under an agent
}

pub struct Script {
    replies: RefCell<VecDeque<Value>>,
    pub prompts: RefCell<Vec<String>>,
}

impl Script {
    pub fn new(replies: Vec<Value>) -> Script {
        Script { replies: RefCell::new(replies.into()), prompts: RefCell::new(Vec::new()) }
    }
}

impl Provider for &Script {
    fn complete_json(&self, _system: &str, prompt: &str, _schema: &Value) -> Result<Meta> {
        self.prompts.borrow_mut().push(prompt.to_string());
        let empty = json!({"summary": "", "claims": [], "used_memory_ids": [], "outdated": []});
        Ok(m(self.replies.borrow_mut().pop_front().unwrap_or(empty)))
    }
}

pub fn write_session(c: &Case, name: &str, project: &str, lines: &[&str]) {
    let path = c.brain.path(&["sessions", "2023-05-08", &format!("claude-code-{name}.md")]);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        &path,
        format!("# s\n\n- date: 2023-05-08\n- project: {project}\n\n## Applied\n- -\n\n## Transcript\n\n{}\n", lines.join("\n")),
    )
    .unwrap();
}

pub struct Two {
    pub tmp: tempfile::TempDir,
    pub remote: String,
    pub a: Brain,
    pub b: Brain,
}

pub fn add_commit(brain: &Brain, type_: &str, text: &str) -> rewire::brain::Node {
    let change = m(json!({"action": "NEW", "type": type_, "text": text, "confidence": "explicit", "provenance": "user"}));
    let n = ops::apply_change(brain, &change, None).unwrap();
    brain.commit("add");
    n
}

pub fn two() -> Two {
    let tmp = tempfile::tempdir().unwrap();
    let remote = tmp.path().join("remote.git").to_string_lossy().into_owned();
    assert!(Command::new("git").args(["init", "-q", "--bare", &remote]).status().unwrap().success());
    let mut a = Brain::init(Some(&tmp.path().join("a")), Some("laptop")).unwrap();
    sync::set_remote(&mut a, &remote).unwrap();
    add_commit(&a, "fact", "Shared fact from the laptop");
    sync::sync(&a).unwrap();
    let b = Brain::clone_from(&remote, Some(&tmp.path().join("b")), Some("codespace")).unwrap();
    Two { tmp, remote, a, b }
}

pub fn episode_texts(c: &mut Case, query: &str, episodes: usize) -> Vec<String> {
    ops::recall(&c.brain, &mut c.index, query, 8, None, Some("all"), None, episodes, None, &[])
        .unwrap()
        .into_iter()
        .filter(|h| h.type_ == "episode")
        .map(|h| h.text)
        .collect()
}

pub fn push_from(t: &Two, files: &[(&str, &str)], symlinks: &[(&str, &str)]) {
    let blob = |content: &str| {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), content).unwrap();
        let out = t.a.git(&["hash-object", "-w", "--no-filters", &tmp.path().to_string_lossy()]);
        assert!(out.status.success());
        rewire::brain::out_str(&out)
    };
    let entries =
        files.iter().map(|(p, text)| ("100644", *p, blob(text))).chain(symlinks.iter().map(|(p, target)| ("120000", *p, blob(target))));
    for (mode, path, sha) in entries.collect::<Vec<_>>() {
        assert!(t.a.git(&["update-index", "--add", "--cacheinfo", &format!("{mode},{sha},{path}")]).status.success(), "{path}");
    }
    assert!(t.a.git(&["commit", "-q", "-m", "from the other device"]).status.success());
    assert!(t.a.git(&["push", "-q", "origin", "HEAD"]).status.success());
}

pub fn disguised_rule(t: &Two, file: &str, frontmatter_type: &str, text: &str) {
    let base = add_commit(&t.a, "fact", "placeholder for a disguised rule");
    let raw = std::fs::read_to_string(&base.path).unwrap();
    let raw = raw.replace("placeholder for a disguised rule", text);
    let raw = regex::Regex::new(r#"(?m)^type: .*$"#).unwrap().replace(&raw, frontmatter_type).into_owned();
    // the file is named after its id, as rewire names files
    let suffix = file.trim_end_matches(".md").rsplit_once('-').unwrap().1.to_uppercase();
    let raw = raw.replace(base.id(), &format!("01M5ZZZZZZZZZZZZZZ{suffix}"));
    std::fs::remove_file(&base.path).unwrap();
    push_from(t, &[(file, &raw)], &[]);
}

pub fn primed(brain: &Brain) -> String {
    let b = Brain::open(Some(&brain.root));
    rewire::snapshot::build(&b, &mut Index::new(&b), None, None).unwrap().text
}
