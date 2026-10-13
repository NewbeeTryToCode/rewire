//! Semantic search (own process: the model is loaded once per process). Uses the model files of a
//! local `wordllama` Python package; skipped when there is none.

use rewire::brain::Brain;
use rewire::index::{Index, SearchOpts};
use rewire::{ops, semantic};
use serde_json::json;
use std::path::PathBuf;

fn wordllama_dir() -> Option<PathBuf> {
    let venv = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.venv/lib");
    let from_venv = std::fs::read_dir(&venv).ok()?.flatten().map(|e| e.path().join("site-packages/wordllama")).find(|p| p.is_dir());
    from_venv.or_else(|| {
        let out = std::process::Command::new("python3")
            .args(["-c", "import os, wordllama; print(os.path.dirname(wordllama.__file__))"])
            .output()
            .ok()?;
        let d = String::from_utf8_lossy(&out.stdout).trim().to_string();
        (!d.is_empty()).then(|| PathBuf::from(d))
    })
}

#[test]
fn semantic_vectors_find_what_keywords_miss() {
    let Some(src) = wordllama_dir() else {
        eprintln!("skipped: no wordllama package");
        return;
    };
    let tmp = tempfile::tempdir().unwrap();
    std::env::set_var("REWIRE_MODEL_DIR", tmp.path().join("model"));
    semantic::install(Some(src.to_str().unwrap())).unwrap();
    assert!(semantic::available());
    let v = semantic::embed(&["hello world".to_string()]).unwrap().remove(0);
    assert_eq!(v.len(), semantic::DIM);
    assert!((v.iter().map(|x| x * x).sum::<f32>() - 1.0).abs() < 1e-4);

    let brain = Brain::init(Some(&tmp.path().join("brain")), Some("t")).unwrap();
    for text in ["The user drinks black coffee every morning", "Deploys of atlas run on Fly.io"] {
        let change = json!({"action": "NEW", "type": "fact", "text": text, "confidence": "explicit", "provenance": "user"});
        ops::apply_change(&brain, change.as_object().unwrap(), None).unwrap();
    }
    let mut index = Index::new(&brain);
    index.refresh().unwrap();
    // no keyword in common with either note: only the vectors can rank them
    let hits = index.search("espresso beverage", &SearchOpts { scopes: Some(vec!["global".into()]), ..Default::default() }).unwrap();
    assert_eq!(hits.first().map(|h| h.title.as_str()), Some("The user drinks black coffee every morning"));
}
