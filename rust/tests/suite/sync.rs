//! Talking to a git remote: clone, push, pull, conflicts, commits and remote classification.

use crate::support::*;
use rewire::brain::Brain;
use rewire::index::Index;
use rewire::{integrations, ops, sync};
use serde_json::json;
use std::path::Path;
use std::process::Command;

#[test]
fn sync_clone_events_and_conflicts() {
    let t = two();
    assert_eq!(t.b.iter_nodes().iter().map(|n| n.title().to_string()).collect::<Vec<_>>(), ["Shared fact from the laptop"]);
    assert!(t.b.config.sync_auto);
    assert_eq!((t.a.device(), t.b.device()), ("laptop".to_string(), "codespace".to_string()));

    // events from both devices merge
    let node = t.a.iter_nodes().remove(0);
    ops::get(&t.a, &[node.id().to_string()], "sa", "mcp").unwrap();
    ops::get(&t.b, &[node.id().to_string()], "sb", "mcp").unwrap();
    add_commit(&t.b, "preference", "Codespace preference");
    sync::sync(&t.b).unwrap();
    sync::sync(&t.a).unwrap();
    sync::sync(&t.b).unwrap();
    for brain in [&t.a, &t.b] {
        let mut sessions: Vec<String> = brain.iter_events().iter().map(|e| e["session"].as_str().unwrap().to_string()).collect();
        sessions.sort();
        assert_eq!(sessions, ["sa", "sb"]);
        assert_eq!(brain.iter_nodes().len(), 2);
    }
    assert!(t.a.path(&["events", "laptop"]).is_dir() && t.a.path(&["events", "codespace"]).is_dir());

    // node conflict: newest wins, loser goes to the inbox
    let na = t.a.get_node(node.id()).unwrap();
    let nb = t.b.get_node(node.id()).unwrap();
    let mut ma = na.meta.clone();
    ma.insert("title".into(), json!("Laptop edit"));
    ma.insert("recorded_at".into(), json!("2030-01-01T00:00:00Z"));
    t.a.write_node(&ma, "laptop body", Some(&na.path)).unwrap();
    t.a.commit("edit a");
    let mut mb = nb.meta.clone();
    mb.insert("title".into(), json!("Codespace edit"));
    mb.insert("recorded_at".into(), json!("2030-01-02T00:00:00Z"));
    t.b.write_node(&mb, "codespace body", Some(&nb.path)).unwrap();
    t.b.commit("edit b");
    sync::sync(&t.a).unwrap();
    let notes = sync::sync(&t.b).unwrap();
    assert!(notes.iter().any(|n| n.contains("newest")), "{notes:?}");
    assert_eq!(t.b.get_node(node.id()).unwrap().title(), "Codespace edit");
    let inbox: Vec<(String, String, String)> = t
        .b
        .iter_inbox()
        .iter()
        .map(|i| {
            (i.meta["action"].as_str().unwrap().into(), i.meta["target"].as_str().unwrap().into(), i.meta["title"].as_str().unwrap().into())
        })
        .collect();
    assert_eq!(inbox, vec![("UPDATE".to_string(), node.id().to_string(), "Laptop edit".to_string())]);
    assert_eq!(rewire::brain::out_str(&t.b.git(&["diff", "--name-only", "--diff-filter=U"])), "");
    sync::sync(&t.a).unwrap();
    assert_eq!(t.a.get_node(node.id()).unwrap().title(), "Codespace edit");
}

#[test]
fn session_start_pulls_and_clones_missing_brain() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    integrations::SPAWN_BACKGROUND.store(false, std::sync::atomic::Ordering::Relaxed);
    let t = two();
    add_commit(&t.b, "fact", "Remember from the codespace");
    add_commit(&t.b, "preference", "Codespace preference waits for a review here");
    sync::sync(&t.b).unwrap();
    let out = integrations::hook(Brain::open(Some(&t.a.root)), "session-start", &json!({"cwd": t.tmp.path()}), "claude-code").unwrap();
    assert!(out.contains("Remember from the codespace"));
    assert!(!out.contains("Codespace preference") && out.contains("from another device are not shown"), "{out}");
    let target = t.tmp.path().join("fresh");
    std::env::set_var("REWIRE_REMOTE", &t.remote);
    let out = integrations::hook(Brain::open(Some(&target)), "session-start", &json!({"cwd": t.tmp.path()}), "claude-code").unwrap();
    std::env::remove_var("REWIRE_REMOTE");
    assert!(Brain::open(Some(&target)).exists());
    assert!(out.contains("Shared fact from the laptop"));
}

#[test]
fn cloud_session_fetches_brain_trusted_and_syncs() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    integrations::SPAWN_BACKGROUND.store(false, std::sync::atomic::Ordering::Relaxed);
    let tmp = tempfile::tempdir().unwrap();
    let remote = tmp.path().join("remote.git").to_string_lossy().into_owned();
    assert!(Command::new("git").args(["init", "-q", "--bare", &remote]).status().unwrap().success());
    // a brain pushed by hand: brain.toml leaves auto sync off
    let laptop = Brain::init(Some(&tmp.path().join("laptop")), Some("laptop")).unwrap();
    add_commit(&laptop, "preference", "Prefers tabs over spaces");
    laptop.git(&["remote", "add", "origin", &remote]);
    let branch = sync::branch(&laptop);
    assert!(laptop.git(&["push", "-q", "-u", "origin", &branch]).status.success());
    assert!(!laptop.config.sync_auto);

    let vars = [
        ("CLAUDE_CODE_REMOTE", "true"),
        ("CLAUDE_CODE_REMOTE_SESSION_ID", "session_01AbCdEfGh12345678"),
        ("REWIRE_REMOTE", remote.as_str()),
    ];
    for (k, v) in vars {
        std::env::set_var(k, v);
    }
    // not trusted: the preference waits for a review that a fresh VM never gets
    let untrusted = tmp.path().join("untrusted");
    let out = integrations::hook(Brain::open(Some(&untrusted)), "session-start", &json!({"cwd": tmp.path()}), "claude-code").unwrap();
    assert!(!out.contains("Prefers tabs"), "{out}");

    std::env::set_var("REWIRE_REMOTE_TRUSTED", "1");
    let vm = tmp.path().join("vm").join("brain");
    let out = integrations::hook(Brain::open(Some(&vm)), "session-start", &json!({"cwd": tmp.path()}), "claude-code").unwrap();
    let cloud = Brain::open(Some(&vm));
    let result = || {
        assert!(out.contains("Prefers tabs"), "{out}");
        assert!(cloud.untrusted_ids().is_empty());
        assert_eq!(cloud.device(), "cloud-12345678");
        assert!(cloud.config.sync_auto, "REWIRE_REMOTE turns auto sync on for the brain it names");
        // a second fetch (the MCP server starting with the hook) uses the brain already there
        assert!(Brain::fetch(&remote, &vm).unwrap().exists());
        let leftovers: Vec<_> = std::fs::read_dir(vm.parent().unwrap())
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.contains(".fetch-"))
            .collect();
        assert!(leftovers.is_empty(), "no fetch folder left behind: {leftovers:?}");

        // what the laptop pushes during the session is trusted on the VM too, and the VM's own notes reach the laptop
        add_commit(&laptop, "preference", "Prefers dark mode");
        assert!(laptop.git(&["push", "-q", "origin", &branch]).status.success());
        let learned = add_commit(&cloud, "fact", "Learned in a cloud session");
        ops::get(&cloud, &[learned.id().to_string()], "cloud-session", "mcp").unwrap(); // writes this VM's event log
        sync::sync(&cloud).unwrap();
        assert!(cloud.untrusted_ids().is_empty());
        assert!(cloud.iter_nodes().iter().any(|n| n.title() == "Prefers dark mode"));
        assert!(laptop.git(&["pull", "-q", "origin", &branch]).status.success());
        assert!(laptop.iter_nodes().iter().any(|n| n.title() == "Learned in a cloud session"));
        assert!(laptop.path(&["events", "cloud-12345678"]).is_dir(), "the VM syncs as its own device");

        // another origin than REWIRE_REMOTE is neither trusted nor synced automatically
        let other = Brain::init(Some(&tmp.path().join("other")), Some("other")).unwrap();
        other.git(&["remote", "add", "origin", &tmp.path().join("elsewhere.git").to_string_lossy()]);
        let other = Brain::open(Some(&other.root));
        assert!(!other.config.sync_auto && !sync::remote_trusted(&other));
    };
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(result));
    for k in ["CLAUDE_CODE_REMOTE", "CLAUDE_CODE_REMOTE_SESSION_ID", "REWIRE_REMOTE", "REWIRE_REMOTE_TRUSTED"] {
        std::env::remove_var(k);
    }
    if let Err(e) = outcome {
        std::panic::resume_unwind(e);
    }
}

#[test]
fn new_repository_becomes_a_brain_on_first_clone() {
    let tmp = tempfile::tempdir().unwrap();
    let git = |dir: &Path, args: &[&str]| assert!(Command::new("git").current_dir(dir).args(args).status().unwrap().success());
    let seeded = |name: &str, files: &[&str]| {
        let remote = tmp.path().join(format!("{name}.git"));
        git(tmp.path(), &["init", "-q", "--bare", &remote.to_string_lossy()]);
        if !files.is_empty() {
            let work = tmp.path().join(format!("{name}-work"));
            git(tmp.path(), &["clone", "-q", &remote.to_string_lossy(), &work.to_string_lossy()]);
            for f in files {
                std::fs::create_dir_all(work.join(f).parent().unwrap()).unwrap();
                std::fs::write(work.join(f), "x\n").unwrap();
            }
            git(&work, &["add", "-A"]);
            git(&work, &["-c", "user.name=t", "-c", "user.email=t@t", "commit", "-q", "-m", "Initial commit"]);
            git(&work, &["push", "-q", "origin", "HEAD"]);
        }
        remote.to_string_lossy().into_owned()
    };

    let github = seeded("github", &["README.md", "LICENSE", ".gitignore"]);
    let brain = Brain::clone_from(&github, Some(&tmp.path().join("a")), Some("laptop")).unwrap();
    assert!(brain.exists() && brain.config.sync_auto);
    assert!(brain.path(&["LICENSE"]).exists());
    // the repository shows the brain's layout from the first push (git keeps no empty folder)
    let pushed = rewire::brain::out_str(&brain.git(&["ls-tree", "-r", "--name-only", "origin/HEAD"]));
    let pushed: Vec<&str> = pushed.lines().collect();
    for f in [
        "brain.toml",
        "README.md",
        ".gitignore",
        ".gitattributes",
        "LICENSE",
        "nodes/.gitkeep",
        "events/.gitkeep",
        "sessions/.gitkeep",
        "skills/.gitkeep",
        "inbox/.gitkeep",
    ] {
        assert!(pushed.contains(&f), "{f} not in {pushed:?}");
    }
    assert!(!pushed.iter().any(|f| f.starts_with(".state") || f.starts_with(".index") || *f == "local.toml"), "{pushed:?}");
    let other = Brain::clone_from(&github, Some(&tmp.path().join("b")), Some("desk")).unwrap();
    assert!(other.exists() && other.path(&["brain.toml"]).exists(), "the brain was pushed");

    let empty = seeded("empty", &[]);
    let brain = Brain::fetch(&empty, &tmp.path().join("vm").join("brain")).unwrap();
    assert!(brain.exists());
    assert!(Brain::clone_from(&empty, Some(&tmp.path().join("c")), None).unwrap().exists(), "the brain was pushed");

    let code = seeded("code", &["README.md", "src/main.rs"]);
    let err = Brain::clone_from(&code, Some(&tmp.path().join("d")), None).err().unwrap().to_string();
    assert!(err.contains("not a rewire brain"), "{err}");
    assert!(!tmp.path().join("d").exists());
}

#[test]
fn concurrent_first_fetches_create_one_brain() {
    let tmp = tempfile::tempdir().unwrap();
    let remote = tmp.path().join("new.git").to_string_lossy().into_owned();
    assert!(Command::new("git").args(["init", "-q", "--bare", &remote]).status().unwrap().success());
    // pushes held for a moment, so both machines' first pushes are in flight together and one is rejected
    let hook = Path::new(&remote).join("hooks").join("pre-receive");
    std::fs::write(&hook, "#!/bin/sh\ncat >/dev/null\nsleep 1\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let roots = [tmp.path().join("vm1").join("brain"), tmp.path().join("vm1").join("brain"), tmp.path().join("vm2").join("brain")];
    let handles: Vec<_> = roots
        .iter()
        .cloned()
        .map(|root| {
            let remote = remote.clone();
            std::thread::spawn(move || Brain::fetch(&remote, &root).map(|b| b.root))
        })
        .collect();
    for h in handles {
        assert!(h.join().unwrap().is_ok());
    }
    let out = |dir: &Path, args: &[&str]| {
        String::from_utf8_lossy(&Command::new("git").current_dir(dir).args(args).output().unwrap().stdout).trim().to_string()
    };
    let remote_head = out(Path::new(&remote), &["rev-parse", "HEAD"]);
    assert_eq!(out(Path::new(&remote), &["rev-list", "--max-parents=0", "HEAD"]).lines().count(), 1, "one brain, one history");
    for vm in ["vm1", "vm2"] {
        let root = tmp.path().join(vm).join("brain");
        assert!(Brain::open(Some(&root)).exists());
        assert_eq!(out(&root, &["rev-parse", "HEAD"]), remote_head, "{vm} is on the pushed brain");
    }
    assert_eq!(std::fs::read_dir(tmp.path().join("vm1")).unwrap().filter(|e| e.as_ref().unwrap().file_name() == "brain").count(), 1);
}

#[test]
fn brain_commits_use_the_configured_author() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    std::env::set_var("REWIRE_GIT_AUTHOR", "Ana <1+ana@users.noreply.github.com>");
    let brain = Brain::init(Some(&tmp.path().join("b")), Some("vm"));
    let author = brain.as_ref().map(|b| {
        add_commit(b, "fact", "Written in a cloud session");
        rewire::brain::out_str(&b.git(&["log", "--format=%an <%ae>|%cn <%ce>"]))
    });
    std::env::remove_var("REWIRE_GIT_AUTHOR");
    for line in author.unwrap().lines() {
        assert_eq!(line, "Ana <1+ana@users.noreply.github.com>|Ana <1+ana@users.noreply.github.com>");
    }
    assert_eq!(rewire::brain::parse_identity("Ana <a@b.c>"), Some(("Ana".into(), "a@b.c".into())));
    assert_eq!(rewire::brain::parse_identity("<a@b.c>"), None);
    assert_eq!(rewire::brain::parse_identity("Ana a@b.c"), None);
}

#[test]
fn unreachable_brain_says_how_to_attach_it() {
    let msg = sync::fetch_failure(
        "https://user:tok@github.com/me/brain",
        "git clone failed: fatal: could not read Username for 'https://github.com': terminal prompts disabled",
    );
    assert!(msg.contains("environment's repositories") && msg.contains("https://***@github.com/me/brain"), "{msg}");
    assert!(!msg.contains("tok@"));
    let other = sync::fetch_failure("https://example.com/b", "git clone failed: fatal: unable to access: Could not resolve host");
    assert!(other.starts_with("could not fetch https://example.com/b: git clone failed"), "{other}");

    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    std::env::set_var("REWIRE_REMOTE", tmp.path().join("missing.git"));
    let out = integrations::hook(Brain::open(Some(&tmp.path().join("b"))), "session-start", &json!({}), "claude-code").unwrap();
    std::env::remove_var("REWIRE_REMOTE");
    assert!(out.contains("Memory unavailable this session: could not fetch"), "{out}");
}

#[test]
fn legacy_device_in_toml_is_not_shared() {
    let t = two();
    let root = t.tmp.path().join("legacy");
    let legacy = Brain::init(Some(&root), None).unwrap();
    let toml = legacy.path(&["brain.toml"]);
    std::fs::write(&toml, read(&toml).replacen("schema = 1\n", "schema = 1\ndevice = \"old-box\"\n", 1)).unwrap();
    let _ = std::fs::remove_file(legacy.path(&[".state", "device.json"]));
    let mut legacy = Brain::open(Some(&root));
    let remote2 = t.tmp.path().join("remote2.git").to_string_lossy().into_owned();
    assert!(Command::new("git").args(["init", "-q", "--bare", &remote2]).status().unwrap().success());
    sync::set_remote(&mut legacy, &remote2).unwrap();
    sync::sync(&legacy).unwrap();
    assert_eq!(legacy.device(), "old-box");
    assert!(!read(&toml).contains("old-box"));
    let other = Brain::clone_from(&remote2, Some(&t.tmp.path().join("other")), None).unwrap();
    assert_ne!(other.device(), "old-box");
}

#[test]
fn transcripts_setting_can_be_pinned_per_device_and_changes_are_reported() {
    let t = two();
    let toml = t.a.path(&["brain.toml"]);
    std::fs::write(&toml, read(&toml).replace("transcripts = \"auto\"", "transcripts = \"all\"")).unwrap();
    t.a.commit("more transcripts");
    sync::push(&t.a, 30).unwrap();
    let notes = sync::pull(&t.b, 30).unwrap();
    assert!(notes.iter().any(|n| n.contains("transcripts: auto -> all")), "{notes:?}");
    std::fs::write(t.b.path(&["local.toml"]), "[consolidation]\ntranscripts = \"user\"\n").unwrap();
    let b = Brain::open(Some(&t.b.root));
    assert_eq!((b.config.transcripts.as_str(), b.config.synced_transcripts.as_str()), ("user", "all"));
}

#[test]
fn a_stale_index_lock_does_not_block_commits_forever() {
    let mut c = Case::new();
    let lock = c.brain.root.join(".git").join("index.lock");
    std::fs::write(&lock, "").unwrap();
    let old = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
    std::fs::File::options().write(true).open(&lock).unwrap().set_modified(old).unwrap();
    c.add("fact", "Committed despite a dead git's lock", json!({}));
    assert!(c.brain.commit("after a crash"));
    assert!(!lock.exists());
}

#[test]
fn the_generated_home_page_never_conflicts_on_sync() {
    let t = two();
    add_commit(&t.a, "fact", "Laptop fact");
    ops::write_home(&t.a, &mut Index::new(&t.a)).unwrap();
    t.a.commit("home a");
    add_commit(&t.b, "fact", "Codespace fact");
    ops::write_home(&t.b, &mut Index::new(&t.b)).unwrap();
    t.b.commit("home b");
    sync::sync(&t.a).unwrap();
    sync::sync(&t.b).unwrap();
    assert!(!t.b.root.join("README.md.from-other-device").exists());
    let home = read(&t.b.path(&["README.md"]));
    assert!(home.contains("**3 active memories**"), "{home}");
}

#[test]
fn pinning_keeps_local_toml_valid_with_a_commented_header() {
    let t = two();
    std::fs::write(t.b.path(&["local.toml"]), "[consolidation]   # this device\ncommand = \"my-llm\"\n").unwrap();
    let toml = t.a.path(&["brain.toml"]);
    std::fs::write(&toml, read(&toml).replace("transcripts = \"auto\"", "transcripts = \"all\"")).unwrap();
    t.a.commit("all");
    sync::push(&t.a, 30).unwrap();
    sync::pull(&t.b, 30).unwrap();
    let b = Brain::open(Some(&t.b.root));
    assert!(b.config.error.is_none(), "{:?}", b.config.error);
    assert_eq!((b.config.command.as_str(), b.config.transcripts.as_str()), ("my-llm", "auto"));
}

#[test]
fn conflict_losers_are_labelled_and_the_winning_own_edit_is_not_foreign() {
    let t = two();
    let rule = add_commit(&t.a, "constraint", "Deploy only on weekdays");
    sync::push(&t.a, 30).unwrap();
    sync::pull(&t.b, 30).unwrap();
    let key = ops::unreviewed_key(&Brain::open(Some(&t.b.root)), &rewire::util::short_of(rule.id())).unwrap();
    ops::trust(&t.b, &key).unwrap();
    for (brain, title, at) in
        [(&t.a, "Deploy only on Mondays", "2030-01-01T00:00:00Z"), (&t.b, "Deploy only on Tuesdays", "2030-01-02T00:00:00Z")]
    {
        let n = brain.get_node(rule.id()).unwrap();
        let mut meta = n.meta.clone();
        meta.insert("title".into(), json!(title));
        meta.insert("recorded_at".into(), json!(at));
        brain.write_node(&meta, title, Some(&n.path)).unwrap();
        brain.commit("edit");
    }
    sync::push(&t.a, 30).unwrap();
    sync::pull(&t.b, 30).unwrap();
    let b = Brain::open(Some(&t.b.root));
    assert!(primed(&b).contains("Tuesdays"), "own winning edit stays trusted");
    let loser = b.iter_inbox().into_iter().find(|i| i.body.contains("Mondays")).unwrap();
    assert!(b.is_untrusted(&loser.path));
}

#[test]
fn local_and_unknown_remotes_are_not_called_public() {
    assert_eq!(ops::remote_is_public("/tmp/remote.git"), None);
    assert_eq!(ops::remote_is_public("file:///tmp/remote.git"), None);
}

#[test]
fn a_note_renamed_on_another_device_syncs() {
    let t = two();
    let n = add_commit(&t.a, "fact", "The cache lives in redis");
    sync::push(&t.a, 30).unwrap();
    sync::pull(&t.b, 30).unwrap();
    let rel = n.path.strip_prefix(&t.a.root).unwrap().to_string_lossy().into_owned();
    let renamed = rel.replace("the-cache", "renamed-the-cache");
    assert!(t.a.git(&["mv", &rel, &renamed]).status.success());
    t.a.commit("rename");
    sync::push(&t.a, 30).unwrap();
    sync::pull(&t.b, 30).unwrap();
    assert!(t.b.root.join(&renamed).is_file());
}

#[test]
fn a_hand_edit_made_later_wins_a_conflict() {
    let t = two();
    let rule = add_commit(&t.a, "constraint", "RULE base");
    sync::push(&t.a, 30).unwrap();
    sync::pull(&t.b, 30).unwrap();
    let edit = |brain: &Brain, text: &str| {
        let path = brain.get_node(rule.id()).unwrap().path;
        std::fs::write(&path, read(&path).replace("RULE base", text)).unwrap();
        brain.commit("hand edit");
    };
    edit(&t.a, "RULE older");
    std::thread::sleep(std::time::Duration::from_millis(1100));
    edit(&t.b, "RULE newer");
    sync::sync(&t.b).unwrap();
    sync::sync(&t.a).unwrap();
    assert!(read(&t.a.get_node(rule.id()).unwrap().path).contains("RULE newer"));
}

#[test]
fn remote_urls_are_shown_without_credentials() {
    assert_eq!(ops::redact_url("https://alice:ghp_SECRET@github.com/a/b.git"), "https://***@github.com/a/b.git");
}
