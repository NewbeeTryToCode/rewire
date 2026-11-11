//! Reading, writing and importing session transcripts.

use crate::support::*;
use rewire::index::Index;
use rewire::{ops, transcripts as tx};
use serde_json::{json, Value};
use std::path::Path;
use std::process::Command;

#[test]
fn injected_memory_never_reaches_consolidation() {
    let snap = "<rewire-memory>\n# rewire memory\n- 2023-01-01 · User prefers tabs [abcdefgh]\n</rewire-memory>";
    let lines: Vec<(usize, String)> = [
        json!({"type": "user", "sessionId": "s", "message": {"content": format!("{snap}\nplease fix the build")}}),
        json!({"type": "assistant", "message": {"content": [{"type": "text", "text": format!("Noted. {snap}")}]}}),
        json!({"type": "user", "message": {"content": [{"type": "tool_result", "content": "<rewire-memory>\n[{\"id\": \"x\""}]}}),
    ]
    .iter()
    .enumerate()
    .map(|(i, v)| (i + 1, v.to_string()))
    .collect();
    let (_, turns) = tx::parse_claude_code(&lines);
    assert_eq!(turns.iter().map(|t| t.text.as_str()).collect::<Vec<_>>(), ["please fix the build", "Noted."]);
}

#[test]
fn transcripts_claude_code_codex_and_internal() {
    let (harness, meta, turns, last) = tx::read(&fix("claude_code.jsonl"), 0, None).unwrap();
    assert_eq!(harness, "claude-code");
    assert_eq!(meta.session.as_deref(), Some("sess-1"));
    assert_eq!(turns.iter().map(|t| t.role.as_str()).collect::<Vec<_>>(), ["user", "assistant", "tool", "user"]);
    assert!(!turns[0].text.contains("system-reminder"));
    let (_, _, later, _) = tx::read(&fix("claude_code.jsonl"), last, None).unwrap();
    assert!(later.is_empty());
    let (harness, meta, turns, _) = tx::read(&fix("rollout-2026-10-01T10-00-00-abc.jsonl"), 0, None).unwrap();
    assert_eq!(harness, "codex");
    assert_eq!(meta.cwd.as_deref(), Some("/work/app"));
    assert_eq!(turns.iter().map(|t| t.role.as_str()).collect::<Vec<_>>(), ["user", "assistant", "tool"]);
    let internal = tx::SessionMeta { cwd: Some("/b/.state/llm".into()), ..Default::default() };
    assert!(tx::is_internal(&internal, &[], Path::new("/b")));
    let marked = tx::Turn { n: 1, role: "user".into(), text: format!("{} x", tx::INTERNAL_MARKER), ts: String::new() };
    assert!(tx::is_internal(&Default::default(), &[marked], Path::new("/b")));
    let work = tx::SessionMeta { cwd: Some("/work".into()), ..Default::default() };
    let hi = tx::Turn { n: 1, role: "user".into(), text: "hi".into(), ts: String::new() };
    assert!(!tx::is_internal(&work, &[hi], Path::new("/b")));
}

#[test]
fn published_artifacts_and_pull_requests_are_outputs() {
    let (_, meta, _, _) = tx::read(&fix("claude_code_outputs.jsonl"), 0, None).unwrap();
    let got: Vec<(usize, &str, &str, &str)> =
        meta.outputs.iter().map(|o| (o.n, o.kind.as_str(), o.title.as_str(), o.url.as_str())).collect();
    // a read is no output, a republish of the same page is not a second one, a refused publish is none
    assert_eq!(
        got,
        [
            (3, "artifact", "Roadmap AI Engineering", "https://claude.ai/artifact/DYry1ig7z7QJPjcHdW4BC6"),
            (7, "pull request", "Add [learning] notes", "https://github.com/me/notes/pull/7"),
            (7, "pull request", "Fix typo", "https://github.com/me/notes/pull/8"),
        ]
    );
}

#[test]
fn transcript_kept_redacted_searchable_and_profile_filtered() {
    let mut c = Case::new();
    c.brain.config.transcripts = "all".into();
    let r = run_fixture(&c.brain, Fake::new(vec![extraction(vec![], vec![])]));
    assert_eq!(r["status"], "consolidated");
    let day = c.brain.path(&["sessions", "2026-10-01"]); // dated by the session, not by consolidation
    let file = std::fs::read_dir(&day).unwrap().next().unwrap().unwrap().path();
    let content = read(&file);
    assert!(content.contains("## Transcript") && content.contains("never deploy on Fridays"));
    let mut index = Index::new(&c.brain);
    let hits = index.search_episodes("deploy fridays", 5, &[], None, None).unwrap();
    assert!(hits[0].text.contains("Fridays"));
    // coding profile (default): only user turns are kept
    let c = Case::new();
    run_fixture(&c.brain, Fake::new(vec![extraction(vec![], vec![])]));
    let day = c.brain.path(&["sessions", "2026-10-01"]);
    let content = read(&std::fs::read_dir(&day).unwrap().next().unwrap().unwrap().path());
    assert!(content.contains("concise"));
    assert!(!content.contains("I think you like YAML"));
    assert!(!content.contains("evil.example"));
}

#[test]
fn import_writes_transcripts_without_an_llm() {
    let tmp = tempfile::tempdir().unwrap();
    let brain = tmp.path().join("b");
    let rewire = |args: &[&str], input: &str| {
        use std::io::Write;
        let mut child = Command::new(env!("CARGO_BIN_EXE_rewire"))
            .arg("--brain")
            .arg(&brain)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
        child.wait_with_output().unwrap()
    };
    assert!(rewire(&["init"], "").status.success());
    let toml = brain.join("brain.toml");
    std::fs::write(&toml, read(&toml).replace("profile = \"coding\"", "profile = \"general\"")).unwrap();
    let sessions = [
        json!({"harness": "chat", "session": "s1", "date": "2024-03-01",
               "turns": [{"n": 1, "role": "user", "text": "I adopted a corgi named Biscuit"},
                         {"n": 2, "role": "assistant", "text": "Congratulations on Biscuit!"}]}),
        json!({"harness": "chat", "session": "s2", "date": "2024-04-02",
               "turns": [{"n": 1, "role": "user", "text": "We are moving to Lisbon in June"}]}),
    ];
    let input = sessions.iter().map(|s| s.to_string()).collect::<Vec<_>>().join("\n");
    let out = rewire(&["import", "-"], &input);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let records: Vec<Value> = String::from_utf8_lossy(&out.stdout).lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    assert_eq!(records.len(), 2);
    assert!(records.iter().all(|r| r["ok"] == true && r["llm_calls"] == 0));
    let session = read(&brain.join("sessions").join("2024-03-01").join("chat-s1.md"));
    assert!(session.contains("[1 user] I adopted a corgi named Biscuit") && session.contains("[2 assistant]"));
    let hits: Value =
        serde_json::from_str(&String::from_utf8_lossy(&rewire(&["recall", "--json", "--episodes", "2", "corgi"], "").stdout)).unwrap();
    assert_eq!(hits[0]["type"], "episode");
    assert!(hits[0]["text"].as_str().unwrap().contains("Biscuit"));
}

#[test]
fn import_checks_every_line_before_writing() {
    let tmp = tempfile::tempdir().unwrap();
    let brain = tmp.path().join("b");
    let rewire =
        |args: &[&str]| as_user(Command::new(env!("CARGO_BIN_EXE_rewire")).arg("--brain").arg(&brain).args(args)).output().unwrap();
    assert!(rewire(&["init"]).status.success());
    let file = tmp.path().join("s.jsonl");
    std::fs::write(&file, "{\"session\": \"good\", \"turns\": [{\"role\": \"user\", \"text\": \"hi\"}]}\ngarbage\n").unwrap();
    let out = rewire(&["import", file.to_str().unwrap()]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("line 2"), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(std::fs::read_dir(brain.join("sessions")).unwrap().all(|e| e.unwrap().file_name() == ".gitkeep"));
}

#[test]
fn forget_sessions_and_undo_without_a_trace_in_recall() {
    let mut c = Case::new();
    let node = c.add("fact", "The staging database lives in eu-west", json!({}));
    assert!(ops::forget(&c.brain, &rewire::util::short_of(node.id()), "moved").unwrap().starts_with("forgot"));
    assert!(c.search("staging database", &["global"]).is_empty());

    let tmp = tempfile::tempdir().unwrap();
    let brain = tmp.path().join("b");
    let rewire = |args: &[&str], input: &str| {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_rewire"));
        as_user(cmd.arg("--brain").arg(&brain).args(args)).stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped());
        let mut child = cmd.spawn().unwrap();
        std::io::Write::write_all(&mut child.stdin.take().unwrap(), input.as_bytes()).unwrap();
        child.wait_with_output().unwrap()
    };
    assert!(rewire(&["init"], "").status.success());
    let line = "{\"harness\": \"claude-code\", \"session\": \"sess-abc\", \"date\": \"2026-10-01\", \"turns\": [{\"role\": \"user\", \"text\": \"the launch codename is bluefalcon\"}]}\n";
    assert!(rewire(&["import", "-"], line).status.success());
    assert!(String::from_utf8_lossy(&rewire(&["sessions"], "").stdout).contains("sess-abc"));
    assert!(String::from_utf8_lossy(&rewire(&["recall", "bluefalcon"], "").stdout).contains("bluefalcon"));
    assert!(rewire(&["undo", "sess-abc"], "").status.success());
    assert!(!String::from_utf8_lossy(&rewire(&["recall", "bluefalcon"], "").stdout).contains("bluefalcon"));
    assert!(String::from_utf8_lossy(&rewire(&["sessions"], "").stdout).contains("(undone)"));
}

#[test]
fn import_never_writes_outside_the_brain_and_reports_every_bad_record() {
    let tmp = tempfile::tempdir().unwrap();
    let brain = tmp.path().join("b");
    let rewire =
        |args: &[&str]| as_user(Command::new(env!("CARGO_BIN_EXE_rewire")).arg("--brain").arg(&brain).args(args)).output().unwrap();
    assert!(rewire(&["init"]).status.success());
    let file = tmp.path().join("s.jsonl");
    std::fs::write(
        &file,
        [
            r#"{"session": "esc", "harness": "h", "date": "../../outside", "turns": []}"#,
            r#"{"session": "esc2", "harness": "../../../outside2", "turns": []}"#,
            r#"{"session": "c1", "turns": "nope"}"#,
            r#"{"turns": []}"#,
        ]
        .join("\n"),
    )
    .unwrap();
    let out = rewire(&["import", file.to_str().unwrap()]);
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success());
    for n in 1..=4 {
        assert!(err.contains(&format!("line {n}:")), "{err}");
    }
    assert!(!tmp.path().join("outside").exists() && !tmp.path().join("outside2-esc2.md").exists());
    // a good record imported twice is written once
    std::fs::write(&file, r#"{"session": "s1", "date": "2026-10-01T10:00:00Z", "turns": [{"role": "user", "text": "hi"}]}"#).unwrap();
    assert!(rewire(&["import", file.to_str().unwrap()]).status.success());
    assert!(String::from_utf8_lossy(&rewire(&["import", file.to_str().unwrap()]).stdout).contains("already imported"));
    assert!(brain.join("sessions/2026-10-01/import-s1.md").is_file());
}
