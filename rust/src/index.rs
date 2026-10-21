//! Derived SQLite index: rebuildable at any time from nodes/, events/ and sessions/.
//!
//! Kept fresh incrementally: only node files whose mtime/size changed are re-parsed, only bytes
//! appended to event logs are read, and activation is recomputed for touched nodes (all nodes once it
//! is older than ACT_MAX_AGE). With the optional WordLlama model installed, nodes and excerpts also get
//! static vectors, fused with the keyword ranking (see semantic.rs).

use crate::activation::{self, FLOOR, MIN_AGE_H};
use crate::brain::Brain;
use crate::util::{now_ts, parse_iso, relpath, round_to, short_of, slugify, take};
use crate::{nodefmt, semantic, text, Error, Meta, Result};
use indexmap::{IndexMap, IndexSet};
use regex::Regex;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, Row};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::UNIX_EPOCH;

pub const SCHEMA: &str = "9";
const HOP_DECAY: f64 = 0.5;
const ACT_MAX_AGE: f64 = 300.0;
/// Directory mtimes do not change on an in-place edit (an editor saving a note): the full stat scan runs at
/// least this often, so a hand edit shows up in recall within seconds.
const SCAN_MAX_AGE: f64 = 2.0;
/// Rows embedded per refresh: the first build after `rewire semantic install` is spread over several refreshes
/// instead of stalling one hook past its time limit (rows without vectors are found by keywords meanwhile).
pub const EMBED_BATCH: usize = 256;
const EP_WINDOW: usize = 6;
const EP_STEP: usize = 4;
pub const RRF_K: f64 = 60.0;

fn rel_weight(rel: &str) -> f64 {
    match rel {
        "part_of" | "depends_on" => 0.8,
        "related_to" => 0.6,
        "about" => 0.5,
        "contradicts" => 0.4,
        "supersedes" => 0.2,
        _ => 0.3,
    }
}

const DDL: &str = "
CREATE TABLE meta(k TEXT PRIMARY KEY, v TEXT);
CREATE TABLE nodes(id TEXT PRIMARY KEY, type TEXT, title TEXT, body TEXT, scope TEXT,
                   importance INT, active INT, recorded_at TEXT, path TEXT, meta TEXT, short TEXT);
CREATE INDEX nodes_short ON nodes(short);
CREATE TABLE edges(src TEXT, dst TEXT, rel TEXT);
CREATE INDEX edges_src ON edges(src);
CREATE INDEX edges_dst ON edges(dst);
CREATE TABLE act(id TEXT PRIMARY KEY, b REAL);
CREATE TABLE files(path TEXT PRIMARY KEY, mtime_ns INT, size INT, id TEXT);
CREATE TABLE evfiles(path TEXT PRIMARY KEY, offset INT, tail TEXT);
CREATE TABLE uses(node TEXT, key TEXT, t TEXT, w REAL, PRIMARY KEY(node, key));
CREATE VIRTUAL TABLE fts USING fts5(id UNINDEXED, title, body, keywords,
                                  tokenize='unicode61 remove_diacritics 2');
CREATE TABLE epfiles(path TEXT PRIMARY KEY, mtime_ns INT, size INT);
CREATE TABLE episodes(path TEXT, date TEXT, text TEXT, first INT, last INT, scope TEXT);
CREATE INDEX episodes_path ON episodes(path);
CREATE VIRTUAL TABLE fts_ep USING fts5(text, tokenize='unicode61 remove_diacritics 2');
CREATE TABLE vec(id TEXT PRIMARY KEY, v BLOB);
CREATE TABLE epvec(rid INTEGER PRIMARY KEY, v BLOB);
";

/// The bundled static embeddings are much weaker than BM25 + keywords on their own (R@5 57% vs 90%),
/// so they only add a small bonus; tuned on the earlier Python engine (experiments/retrieval), not re-measured in Rust.
pub const SEM_WEIGHT: f64 = 0.15;
pub const SEM_TOP: usize = 10;

fn sem_flag() -> &'static str {
    if semantic::available() {
        "1"
    } else {
        "0"
    }
}

/// (ids, scopes, vectors) of every node with a vector.
type NodeVectors = (Vec<String>, Vec<String>, Vec<Vec<f32>>);

/// (rowids, session scopes, vectors) of every episode with a vector.
type EpisodeVectors = (Vec<i64>, Vec<String>, Vec<Vec<f32>>);

/// Indices of `sims` from the most to the least similar.
fn by_similarity(sims: &[f32]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..sims.len()).collect();
    order.sort_by(|a, b| sims[*b].partial_cmp(&sims[*a]).unwrap_or(std::cmp::Ordering::Equal));
    order
}

/// One row of the `nodes` table plus its activation.
#[derive(Debug, Clone)]
pub struct NodeRow {
    pub id: String,
    pub type_: String,
    pub title: String,
    pub body: String,
    pub scope: String,
    pub importance: i64,
    pub active: bool,
    pub recorded_at: String,
    pub path: String,
    pub meta: Meta,
    pub b: f64,
}

impl NodeRow {
    fn from_row(r: &Row) -> rusqlite::Result<NodeRow> {
        let meta_text: String = r.get("meta")?;
        Ok(NodeRow {
            id: r.get("id")?,
            type_: r.get::<_, Option<String>>("type")?.unwrap_or_default(),
            title: r.get::<_, Option<String>>("title")?.unwrap_or_default(),
            body: r.get::<_, Option<String>>("body")?.unwrap_or_default(),
            scope: r.get::<_, Option<String>>("scope")?.unwrap_or_default(),
            importance: r.get::<_, Option<i64>>("importance")?.unwrap_or(3),
            active: r.get::<_, i64>("active")? != 0,
            recorded_at: r.get::<_, Option<String>>("recorded_at")?.unwrap_or_default(),
            path: r.get::<_, Option<String>>("path")?.unwrap_or_default(),
            meta: serde_json::from_str(&meta_text).unwrap_or_default(),
            b: r.get::<_, Option<f64>>("b")?.unwrap_or(FLOOR),
        })
    }

    pub fn valid_from(&self) -> String {
        self.meta.get("valid_from").and_then(Value::as_str).unwrap_or("").chars().take(10).collect()
    }
}

pub const SUMMARY_CHARS: usize = 600;

/// Scope of the session one consolidation run belongs to ("global" when it had no project). A part
/// without the header (a `---` rule inside a summary splits a run) keeps the scope of the part before it.
pub fn session_scope(part: &str, previous: &str) -> String {
    static R: OnceLock<Regex> = OnceLock::new();
    let re = R.get_or_init(|| Regex::new(r"(?m)^- project: (.+?)[ \t]*$").unwrap());
    match re.captures(part) {
        None => previous.to_string(),
        Some(c) if &c[1] != "-" => format!("project:{}", &c[1]),
        Some(_) => "global".into(),
    }
}

/// (turn, question) lines of one consolidation run's "## Questions" section.
pub fn session_questions(part: &str) -> Vec<(i64, String)> {
    static S: OnceLock<Regex> = OnceLock::new();
    static Q: OnceLock<Regex> = OnceLock::new();
    let section = S.get_or_init(|| Regex::new(r"(?s)\n## Questions\n(.*?)(?:\n## |\z)").unwrap());
    let line = Q.get_or_init(|| Regex::new(r"(?m)^- \[(\d+)\] (.+)$").unwrap());
    let Some(m) = section.captures(part) else { return Vec::new() };
    line.captures_iter(&m[1]).filter_map(|c| Some((c[1].parse().ok()?, c[2].trim().to_string()))).collect()
}

/// "Summary: …" row text of one consolidation run in a session file ("" if none).
pub fn session_summary(part: &str) -> String {
    static R: OnceLock<Regex> = OnceLock::new();
    let re = R.get_or_init(|| Regex::new(r"(?s)\n## Summary\n(.*?)(?:\n## |\z)").unwrap());
    let body = re.captures(part).map(|c| c[1].split_whitespace().collect::<Vec<_>>().join(" ")).unwrap_or_default();
    if body.is_empty() || body == "-" {
        return String::new();
    }
    format!("Summary: {}", body.chars().take(SUMMARY_CHARS).collect::<String>())
}

/// A recall hit: a memory node or (type "episode") a transcript excerpt.
#[derive(Debug, Clone)]
pub struct Hit {
    pub id: String,
    pub type_: String,
    pub title: String,
    pub scope: String,
    pub snippet: String,
    pub score: f64,
    pub b: f64,
    pub date: String,
    pub status: String,
    pub via: Option<String>,
    /// episodes only
    pub session: String,
    pub text: String,
    /// the top excerpt only: its session's goal and outcome
    pub summary: Option<String>,
}

impl Hit {
    pub fn to_json(&self) -> Value {
        if self.type_ == "episode" {
            let mut v = json!({"id": self.id, "type": "episode", "date": self.date, "session": self.session,
                               "text": self.text});
            if let Some(s) = &self.summary {
                v["summary"] = json!(s);
            }
            return v;
        }
        let mut m = Meta::new();
        m.insert("id".into(), json!(self.id));
        m.insert("type".into(), json!(self.type_));
        m.insert("title".into(), json!(self.title));
        m.insert("scope".into(), json!(self.scope));
        m.insert("snippet".into(), json!(self.snippet));
        m.insert("score".into(), json!(self.score));
        m.insert("b".into(), json!(self.b));
        m.insert("date".into(), json!(self.date));
        m.insert("status".into(), json!(self.status));
        if let Some(v) = &self.via {
            m.insert("via".into(), json!(v));
        }
        Value::Object(m)
    }
}

pub struct SearchOpts {
    pub scopes: Option<Vec<String>>,
    pub k: usize,
    pub beta: Option<f64>,
    pub include_inactive: bool,
    pub spread: bool,
    pub history: Option<bool>,
    /// explicit time window from the calling model (`refers_to`); otherwise read from the query
    pub window: Option<(String, String)>,
}

impl Default for SearchOpts {
    fn default() -> Self {
        SearchOpts { scopes: None, k: 8, beta: None, include_inactive: false, spread: true, history: None, window: None }
    }
}

struct Stale;

pub struct Index {
    root: PathBuf,
    path: PathBuf,
    d: f64,
    recall_beta: f64,
    spread: String,
    db: Option<Connection>,
    pub seed_weight: f64,
    /// cached (ids, scopes, vectors) of nodes and (rowids, vectors) of episodes for semantic search
    mat: Option<NodeVectors>,
    epmat: Option<EpisodeVectors>,
    /// SQLite's data_version when the vectors were cached: another process (a hook, a sync) that changed the index
    /// changes it, so a long-running MCP server does not keep searching an old set of vectors
    data_version: Option<i64>,
    /// rows embedded per refresh (0 = all); hooks set EMBED_BATCH to stay within their time limit
    pub embed_limit: usize,
}

fn mtime_ns(md: &fs::Metadata) -> i64 {
    md.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_nanos() as i64).unwrap_or(0)
}

/// Excerpt rowid with its first and last turn.
type TurnSpan = (i64, Option<i64>, Option<i64>);

/// Episode row: path, date, text, first turn, last turn.
type EpRow = (String, String, String, i64, i64);

/// Excerpts of one session whose turn ranges overlap or touch become one excerpt (no line twice), at the
/// rank of the best of them. Windows overlap by two turns, so without this the same lines are sent twice:
/// 11-12% of the excerpt tokens on LoCoMo, with the same evidence (experiments/eval/results).
fn merge_excerpts(best: &[i64], rows: &HashMap<i64, EpRow>) -> Vec<(i64, String)> {
    struct Group<'a> {
        rid: i64,
        path: &'a str,
        lo: i64,
        hi: i64,
        lines: BTreeMap<i64, &'a str>,
    }
    let mut groups: Vec<Group> = Vec::new();
    for rid in best {
        let Some((path, _, text, lo, hi)) = rows.get(rid) else { continue };
        let mut lines: BTreeMap<i64, &str> = BTreeMap::new();
        for line in text.lines() {
            let n = turn_re().captures(line).and_then(|c| c[1].parse().ok()).unwrap_or(-(lines.len() as i64) - 1);
            lines.insert(n, line);
        }
        // a session summary (turn 0) is never merged; neither are runs that reuse turn numbers
        let target = groups.iter_mut().find(|g| {
            *lo > 0
                && g.lo > 0
                && g.path == path
                && *lo <= g.hi + 1
                && *hi >= g.lo - 1
                && lines.iter().all(|(n, l)| g.lines.get(n).is_none_or(|x| x == l))
        });
        match target {
            Some(g) => {
                g.lo = g.lo.min(*lo);
                g.hi = g.hi.max(*hi);
                g.lines.extend(lines);
            }
            None => groups.push(Group { rid: *rid, path, lo: *lo, hi: *hi, lines }),
        }
    }
    groups.into_iter().map(|g| (g.rid, g.lines.into_values().collect::<Vec<_>>().join("\n"))).collect()
}

/// Excerpts below the top EP_WHOLE keep only the turns that share a content word with the query, plus
/// EP_CONTEXT turns either side: 11% fewer excerpt tokens on LoCoMo, and at the same token budget (more
/// excerpts) +1 point of evidence recall (experiments/eval/results/2026-10-14-locomo-trim.md).
const EP_WHOLE: usize = 2;
const EP_CONTEXT: usize = 1;

fn trim_excerpt(query: &str, excerpt: &str) -> String {
    static SPEAKER: OnceLock<Regex> = OnceLock::new();
    let speaker = SPEAKER.get_or_init(|| Regex::new(r"^\[[^\]]*\] (?:\w+: )?").unwrap());
    let lines: Vec<&str> = excerpt.lines().collect();
    let hits: Vec<usize> = (0..lines.len()).filter(|&i| text::overlap(query, &speaker.replace(lines[i], "")) > 0).collect();
    if hits.is_empty() {
        return excerpt.to_string(); // matched through vectors or a time window: no word to anchor on, keep it whole
    }
    let keep: std::collections::BTreeSet<usize> =
        hits.iter().flat_map(|&i| i.saturating_sub(EP_CONTEXT)..=(i + EP_CONTEXT).min(lines.len() - 1)).collect();
    keep.into_iter().map(|j| lines[j]).collect::<Vec<_>>().join("\n")
}

/// Static word vectors (WordLlama) are a weak ranker: at equal weight in rank fusion, excerpts that sit mid-list
/// in both rankings pushed out the best keyword match (LongMemEval single-session-user all evidence 100% -> 95%).
/// At 0.25 they still find paraphrases (preference questions 77% -> 87%) without that (2026-10-16 results).
const EP_SEM_WEIGHT: f64 = 0.25;

/// Transcripts keep long turns whole (a detail late in a long assistant reply is still findable); an excerpt
/// line longer than LINE_CHARS is shown condensed to the sentences that best match the query, plus its opening.
/// Keywords only: embedding every sentence at recall time cost ~4x the latency for the same evidence.
const LINE_CHARS: usize = 1500;
/// The user's opening message states their situation, and facts are often said there in passing; an excerpt
/// from later in the session would lose it. On LongMemEval_S, 34 of the 45 evidence turns missing from
/// recalled sessions were turn 1. Shown condensed with every excerpt after it.
const OPENER_CHARS: usize = 700;

fn condense(query: &str, excerpt: &str) -> String {
    excerpt
        .lines()
        .map(|l| if l.chars().count() > LINE_CHARS { condense_line(query, l, LINE_CHARS) } else { l.to_string() })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Sentences and list items of a turn: split after . ! ? : and before "12. " / "3) " / "- " / "* " / "• ".
fn segments(body: &str) -> Vec<&str> {
    static WS: OnceLock<Regex> = OnceLock::new();
    static ITEM: OnceLock<Regex> = OnceLock::new();
    let ws = WS.get_or_init(|| Regex::new(r"\s+").unwrap());
    let item = ITEM.get_or_init(|| Regex::new(r"^(\d{1,2}[.)]\s|[-*•]\s)").unwrap());
    let (mut out, mut start) = (Vec::new(), 0);
    for m in ws.find_iter(body) {
        let after_punct = body[..m.start()].chars().last().is_some_and(|c| ".!?:".contains(c));
        if after_punct || item.is_match(&body[m.end()..]) {
            out.push(&body[start..m.start()]);
            start = m.end();
        }
    }
    out.push(&body[start..]);
    out.into_iter().filter(|s| !s.trim().is_empty()).collect()
}

fn condense_line(query: &str, line: &str, limit: usize) -> String {
    static SPEAKER: OnceLock<Regex> = OnceLock::new();
    static ORDINAL: OnceLock<Regex> = OnceLock::new();
    static QUANTITY: OnceLock<Regex> = OnceLock::new();
    let speaker = SPEAKER.get_or_init(|| Regex::new(r"^\[[^\]]*\] (?:\w+: )?").unwrap());
    let (head, body) = match speaker.find(line) {
        Some(m) => (&line[..m.end()], &line[m.end()..]),
        None => ("", line),
    };
    if body.chars().count() <= limit {
        return line.to_string();
    }
    let segs: Vec<String> = segments(body).into_iter().map(|s| take(s, limit)).collect();
    if segs.len() < 2 {
        return format!("{head}{} …[condensed]", take(body, limit));
    }
    // query words weighted by how rare they are among the segments: a word in every sentence of the reply
    // ("marketing" in a marketing plan) says nothing about which sentence answers; "27th" also finds "27."
    let ordinal = ORDINAL.get_or_init(|| Regex::new(r"^(\d+)(?:st|nd|rd|th)$").unwrap());
    let mut words = text::content_words(query);
    let numbers: Vec<String> = words.iter().filter_map(|w| ordinal.captures(w).map(|c| c[1].to_string())).collect();
    for n in numbers {
        if !words.contains(&n) {
            words.push(n);
        }
    }
    let n = segs.len();
    let mut scores = vec![0.0f64; n];
    for w in &words {
        let hits: Vec<bool> = segs.iter().map(|s| text::overlap(w, s) > 0).collect();
        let df = hits.iter().filter(|&&h| h).count() as f64;
        let idf = ((n as f64 + 1.0) / (df + 0.5)).ln();
        for (i, h) in hits.iter().enumerate() {
            if *h {
                scores[i] += idf;
            }
        }
    }
    let quantity = QUANTITY
        .get_or_init(|| {
            Regex::new(r"(?i)\bhow (much|many|long|old|far|often)\b|\bwhat (time|date|year|day|price|percentage)\b|\bwhen\b").unwrap()
        })
        .is_match(query);
    for (i, s) in segs.iter().enumerate() {
        // a numbered item asked for by its position
        let numbered = words.iter().filter(|w| w.chars().all(|c| c.is_ascii_digit())).any(|w| {
            s.strip_prefix(w.as_str())
                .is_some_and(|r| (r.starts_with('.') || r.starts_with(')')) && r[1..].starts_with(char::is_whitespace))
        });
        if numbered {
            scores[i] += 5.0;
        }
        if quantity && s.chars().any(|c| c.is_ascii_digit()) {
            scores[i] += 1.0; // "how much / when ...": the sentences holding numbers
        }
    }
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| scores[b].partial_cmp(&scores[a]).unwrap_or(std::cmp::Ordering::Equal).then(a.cmp(&b)));
    let len = |s: &String| s.chars().count();
    let (mut keep, mut size) = (std::collections::BTreeSet::from([0usize]), len(&segs[0])); // the opening says what it is about
    for i in order {
        for j in [Some(i), i.checked_add(1), i.checked_sub(1)].into_iter().flatten() {
            if j < n && !keep.contains(&j) && size + len(&segs[j]) <= limit {
                keep.insert(j);
                size += len(&segs[j]) + 1;
            }
        }
        if size as f64 >= limit as f64 * 0.9 {
            break;
        }
    }
    let mut out: Vec<String> = Vec::new();
    let mut prev: Option<usize> = None;
    for &j in &keep {
        let gap = prev.map_or(j != 0, |p| j != p + 1);
        out.push(if gap { format!("… {}", segs[j]) } else { segs[j].clone() });
        prev = Some(j);
    }
    let cut = prev != Some(n - 1) || keep.len() < n;
    format!("{head}{}{}", out.join(" "), if cut { " …[condensed]" } else { "" })
}

fn turn_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^\[(\d+) ").unwrap())
}

fn by_energy(a: &(String, f64), b: &(String, f64)) -> std::cmp::Ordering {
    b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal).then_with(|| a.0.cmp(&b.0))
}

fn marks(n: usize) -> String {
    vec!["?"; n].join(",")
}

impl Index {
    pub fn new(brain: &Brain) -> Index {
        Index {
            root: brain.root.clone(),
            path: brain.path(&[".index", "brain.db"]),
            d: brain.config.d,
            recall_beta: brain.config.recall_beta,
            spread: brain.config.spread.clone(),
            db: None,
            // source-excerpt bonus for the top memory hit, as a share of the best excerpt score
            // (tuned: experiments/locomo/evidence.py)
            seed_weight: 0.35,
            mat: None,
            epmat: None,
            data_version: None,
            embed_limit: 0,
        }
    }

    fn connect(&self) -> Result<Connection> {
        fs::create_dir_all(self.path.parent().unwrap())?;
        let open = |p: &Path| -> Result<Connection> {
            let db = Connection::open(p)?;
            db.busy_timeout(std::time::Duration::from_secs(30))?;
            db.query_row("PRAGMA journal_mode=WAL", [], |_| Ok(()))?;
            Ok(db)
        };
        let db = match open(&self.path) {
            Ok(db) => db,
            Err(_) => {
                // unreadable (not a database): nobody can be using it, start over
                self.remove_files();
                open(&self.path)?
            }
        };
        let ok: Option<String> = db.query_row("SELECT v FROM meta WHERE k='schema'", [], |r| r.get(0)).optional().unwrap_or(None);
        if ok.as_deref() != Some(SCHEMA) {
            // missing or older schema: rebuilt from the files inside the same database file (see reset)
            Self::reset(&db)?;
        }
        Ok(db)
    }

    /// Empty the index in place, in one write transaction. Other processes (an MCP server, a hook) keep valid
    /// handles and see either the old index or the new one; deleting the files under them broke both.
    /// Whether semantic search is on is not part of the identity: vectors missing for some rows are filled in
    /// by `embed_pending`, and a process without the model simply ignores them.
    fn reset(db: &Connection) -> Result<()> {
        static TABLE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        let names: Vec<String> = TABLE
            .get_or_init(|| regex::Regex::new(r"CREATE (?:VIRTUAL )?TABLE (\w+)").unwrap())
            .captures_iter(DDL)
            .map(|c| c[1].to_string())
            .collect();
        db.execute_batch("BEGIN IMMEDIATE")?;
        let done = (|| -> Result<()> {
            for name in &names {
                db.execute_batch(&format!("DROP TABLE IF EXISTS {name}"))?;
            }
            db.execute_batch(DDL)?;
            db.execute("INSERT INTO meta VALUES ('schema', ?)", [SCHEMA])?;
            db.execute("INSERT INTO meta VALUES ('semantic', ?)", [sem_flag()])?;
            Ok(())
        })();
        match done {
            Ok(()) => Ok(db.execute_batch("COMMIT")?),
            Err(e) => {
                let _ = db.execute_batch("ROLLBACK");
                Err(e)
            }
        }
    }

    fn remove_files(&self) {
        for suffix in ["", "-wal", "-shm"] {
            let p = PathBuf::from(format!("{}{}", self.path.display(), suffix));
            let _ = fs::remove_file(p);
        }
    }

    pub fn close(&mut self) {
        self.db = None;
    }

    pub fn db(&mut self) -> Result<&Connection> {
        if self.db.is_none() {
            self.refresh()?;
        }
        Ok(self.db.as_ref().unwrap())
    }

    pub fn rebuild(&mut self) -> Result<i64> {
        if self.db.is_none() {
            self.db = Some(self.connect()?);
        }
        Self::reset(self.db.as_ref().unwrap())?;
        self.mat = None;
        self.epmat = None;
        self.refresh()?;
        Ok(self.db()?.query_row("SELECT count(*) FROM nodes", [], |r| r.get(0))?)
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_max_age(ACT_MAX_AGE)
    }

    pub fn refresh_max_age(&mut self, max_age: f64) -> Result<()> {
        if self.db.is_none() {
            self.db = Some(self.connect()?);
        }
        let outcome = {
            let db = self.db.as_ref().unwrap();
            db.execute_batch("BEGIN IMMEDIATE")?;
            match self.sync_all(db, max_age) {
                Ok(Ok(changed)) => {
                    db.execute_batch("COMMIT")?;
                    Ok(changed)
                }
                Ok(Err(Stale)) => {
                    db.execute_batch("ROLLBACK")?;
                    Err(Stale)
                }
                Err(e) => {
                    let _ = db.execute_batch("ROLLBACK");
                    return Err(e);
                }
            }
        };
        match outcome {
            Err(Stale) => {
                self.rebuild()?;
            }
            Ok(true) => {
                self.mat = None;
                self.epmat = None;
            }
            Ok(false) => {}
        }
        let version: i64 = self.db.as_ref().map_or(Ok(0), |db| db.query_row("PRAGMA data_version", [], |r| r.get(0)))?;
        if self.data_version != Some(version) {
            self.mat = None;
            self.epmat = None;
            self.data_version = Some(version);
        }
        Ok(())
    }

    fn sync_all(&self, db: &Connection, max_age: f64) -> Result<std::result::Result<bool, Stale>> {
        let changed = self.sync_nodes(db)?;
        let touched = match self.sync_events(db)? {
            Ok(t) => t,
            Err(Stale) => return Ok(Err(Stale)),
        };
        let ep_changed = self.sync_episodes(db)?;
        let any = !changed.is_empty() || ep_changed;
        if any || Self::vectors_missing(db)? {
            Self::embed_pending(db, self.embed_limit)?;
        }
        let row: Option<String> = db.query_row("SELECT v FROM meta WHERE k='act_at'", [], |r| r.get(0)).optional()?;
        let stale = match row.and_then(|v| v.parse::<f64>().ok()) {
            Some(at) => now_ts() - at > max_age,
            None => true,
        };
        if stale {
            self.compute_act(db, None)?;
            db.execute("INSERT OR REPLACE INTO meta VALUES ('act_at', ?)", [now_ts().to_string()])?;
        } else if !changed.is_empty() || !touched.is_empty() {
            let ids: Vec<String> = changed.union(&touched).cloned().collect();
            self.compute_act(db, Some(ids))?;
        }
        Ok(Ok(any))
    }

    // ---- incremental sync ------------------------------------------------

    /// Vectors for nodes/episodes that have none yet (derived data, like everything here).
    /// The model is installed but some rows have no vector yet (it was installed after they were indexed).
    fn vectors_missing(db: &Connection) -> Result<bool> {
        if !semantic::available() {
            return Ok(false);
        }
        let missing: Option<i64> = db
            .query_row(
                "SELECT 1 WHERE EXISTS (SELECT 1 FROM nodes WHERE id NOT IN (SELECT id FROM vec)) \
                 OR EXISTS (SELECT 1 FROM episodes WHERE rowid NOT IN (SELECT rid FROM epvec))",
                [],
                |r| r.get(0),
            )
            .optional()?;
        Ok(missing.is_some())
    }

    fn embed_pending(db: &Connection, limit: usize) -> Result<()> {
        let limit = if limit == 0 { -1 } else { limit as i64 };
        if !semantic::available() {
            return Ok(());
        }
        let rows: Vec<(String, String, String, String)> = {
            let mut st =
                db.prepare(&format!("SELECT id, title, body, meta FROM nodes WHERE id NOT IN (SELECT id FROM vec) LIMIT {limit}"))?;
            let it = st.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?;
            it.collect::<rusqlite::Result<Vec<_>>>()?
        };
        let texts: Vec<String> = rows
            .iter()
            .map(|(_, title, body, meta)| {
                let kw = serde_json::from_str::<Value>(meta)
                    .ok()
                    .and_then(|m| m.get("keywords").and_then(Value::as_array).cloned())
                    .map(|a| a.iter().map(crate::util::value_text).collect::<Vec<_>>().join(" "))
                    .unwrap_or_default();
                format!("{title}. {body} {kw}")
            })
            .collect();
        if let Some(vecs) = semantic::embed(&texts) {
            for ((id, ..), v) in rows.iter().zip(vecs) {
                db.execute("INSERT OR REPLACE INTO vec VALUES (?,?)", params![id, semantic::to_blob(&v)])?;
            }
        }
        let eps: Vec<(i64, String)> = {
            let mut st =
                db.prepare(&format!("SELECT rowid, text FROM episodes WHERE rowid NOT IN (SELECT rid FROM epvec) LIMIT {limit}"))?;
            let it = st.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
            it.collect::<rusqlite::Result<Vec<_>>>()?
        };
        let texts: Vec<String> = eps.iter().map(|(_, t)| t.clone()).collect();
        if let Some(vecs) = semantic::embed(&texts) {
            for ((rid, _), v) in eps.iter().zip(vecs) {
                db.execute("INSERT OR REPLACE INTO epvec VALUES (?,?)", params![rid, semantic::to_blob(&v)])?;
            }
        }
        Ok(())
    }

    /// Node ids most similar to the query (in scope), best first.
    fn semantic_ranks(&mut self, query: &str, scopes: Option<&Vec<String>>, limit: usize) -> Result<Vec<String>> {
        if !semantic::available() {
            return Ok(Vec::new());
        }
        if self.mat.is_none() {
            let loaded = {
                let db = self.db()?;
                let mut st = db.prepare("SELECT nodes.id, nodes.scope, vec.v FROM vec JOIN nodes USING(id)")?;
                let rows = st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, Vec<u8>>(2)?)))?;
                let (mut ids, mut sc, mut vs) = (Vec::new(), Vec::new(), Vec::new());
                for r in rows {
                    let (i, s, v) = r?;
                    ids.push(i);
                    sc.push(s);
                    vs.push(semantic::from_blob(&v));
                }
                (ids, sc, vs)
            };
            self.mat = Some(loaded);
        }
        let Some(qv) = semantic::embed(&[query.to_string()]).and_then(|mut v| v.pop()) else { return Ok(Vec::new()) };
        let (ids, node_scopes, vs) = self.mat.as_ref().unwrap();
        let sims: Vec<f32> = vs.iter().map(|v| semantic::dot(v, &qv)).collect();
        Ok(by_similarity(&sims)
            .into_iter()
            .filter(|&i| scopes.is_none_or(|s| s.contains(&node_scopes[i])))
            .take(limit)
            .map(|i| ids[i].clone())
            .collect())
    }

    /// Episode rowids most similar to the query, best first.
    fn semantic_episode_ranks(&mut self, query: &str, scopes: Option<&[String]>, limit: usize) -> Result<Vec<i64>> {
        if !semantic::available() {
            return Ok(Vec::new());
        }
        if self.epmat.is_none() {
            let loaded = {
                let db = self.db()?;
                let mut st = db.prepare("SELECT rid, v, e.scope FROM epvec JOIN episodes e ON e.rowid = epvec.rid ORDER BY rid")?;
                let rows = st.query_map([], |r| {
                    Ok((r.get::<_, i64>(0)?, r.get::<_, Vec<u8>>(1)?, r.get::<_, Option<String>>(2)?.unwrap_or_default()))
                })?;
                let (mut ids, mut scs, mut vs) = (Vec::new(), Vec::new(), Vec::new());
                for r in rows {
                    let (i, v, sc) = r?;
                    ids.push(i);
                    scs.push(sc);
                    vs.push(semantic::from_blob(&v));
                }
                (ids, scs, vs)
            };
            self.epmat = Some(loaded);
        }
        let Some(qv) = semantic::embed(&[query.to_string()]).and_then(|mut v| v.pop()) else { return Ok(Vec::new()) };
        let (ids, scs, vs) = self.epmat.as_ref().unwrap();
        let sims: Vec<f32> = vs.iter().map(|v| semantic::dot(v, &qv)).collect();
        Ok(by_similarity(&sims).into_iter().filter(|&i| scopes.is_none_or(|s| s.contains(&scs[i]))).take(limit).map(|i| ids[i]).collect())
    }

    fn delete_node(db: &Connection, node_id: &str) -> Result<()> {
        let rowid: Option<i64> = db.query_row("SELECT rowid FROM nodes WHERE id=?", [node_id], |r| r.get(0)).optional()?;
        if let Some(rid) = rowid {
            db.execute("DELETE FROM fts WHERE rowid=?", [rid])?;
        }
        db.execute("DELETE FROM nodes WHERE id=?", [node_id])?;
        db.execute("DELETE FROM edges WHERE src=?", [node_id])?;
        db.execute("DELETE FROM act WHERE id=?", [node_id])?;
        db.execute("DELETE FROM vec WHERE id=?", [node_id])?;
        Ok(())
    }

    fn sync_nodes(&self, db: &Connection) -> Result<HashSet<String>> {
        let root = self.root.join("nodes");
        let mut changed = HashSet::new();
        if !root.is_dir() {
            return Ok(changed);
        }
        // rewire writes atomically (rename), which bumps the directory mtime; so if no directory
        // changed and a full scan ran recently, nothing needs re-reading.
        let mut subdirs: Vec<(String, PathBuf, i64)> = fs::read_dir(&root)?
            .flatten()
            .filter(|e| e.path().is_dir())
            .map(|e| {
                let md = e.metadata().ok();
                (e.file_name().to_string_lossy().into_owned(), e.path(), md.map(|m| mtime_ns(&m)).unwrap_or(0))
            })
            .collect();
        subdirs.sort_by(|a, b| a.0.cmp(&b.0));
        let stamp = subdirs.iter().map(|(n, _, m)| format!("{n}:{m}")).collect::<Vec<_>>().join(",");
        let row: Option<String> = db.query_row("SELECT v FROM meta WHERE k='scan'", [], |r| r.get(0)).optional()?;
        if let Some(v) = row {
            if let Some((last_stamp, last_at)) = v.rsplit_once('@') {
                if last_stamp == stamp && now_ts() - last_at.parse::<f64>().unwrap_or(0.0) < SCAN_MAX_AGE {
                    return Ok(changed);
                }
            }
        }
        let mut seen: IndexMap<String, (i64, i64)> = IndexMap::new();
        for (_, dir, _) in &subdirs {
            for e in fs::read_dir(dir)?.flatten() {
                let name = e.file_name().to_string_lossy().into_owned();
                if name.ends_with(".md") {
                    if let Ok(md) = e.metadata() {
                        seen.insert(e.path().to_string_lossy().into_owned(), (mtime_ns(&md), md.len() as i64));
                    }
                }
            }
        }
        db.execute("INSERT OR REPLACE INTO meta VALUES ('scan', ?)", [format!("{stamp}@{}", now_ts())])?;
        let mut known: HashMap<String, (i64, i64, Option<String>)> = HashMap::new();
        {
            let mut st = db.prepare("SELECT path, mtime_ns, size, id FROM files")?;
            let rows = st.query_map([], |r| Ok((r.get::<_, String>(0)?, (r.get(1)?, r.get(2)?, r.get(3)?))))?;
            for r in rows {
                let (p, v) = r?;
                known.insert(p, v);
            }
        }
        let fresh = known.is_empty();
        for (path, (_, _, id)) in &known {
            if !seen.contains_key(path) {
                if let Some(id) = id {
                    Self::delete_node(db, id)?;
                    changed.insert(id.clone());
                }
                db.execute("DELETE FROM files WHERE path=?", [path])?;
            }
        }
        for (path, (mtime, size)) in &seen {
            let old = known.get(path);
            if let Some((m, s, _)) = old {
                if m == mtime && s == size {
                    continue;
                }
            }
            let Ok(text) = fs::read_to_string(path) else { continue };
            let Ok((meta, body)) = nodefmt::parse(&text) else { continue };
            if let Some((_, _, Some(old_id))) = old {
                Self::delete_node(db, old_id)?;
            }
            let node_id = meta.get("id").and_then(Value::as_str).filter(|s| !s.is_empty()).map(String::from);
            if let Some(node_id) = &node_id {
                if !fresh {
                    Self::delete_node(db, node_id)?;
                }
                let active = !crate::brain::is_set(meta.get("retracted_at")) && !crate::brain::is_set(meta.get("valid_to"));
                let s = |k: &str| meta.get(k).and_then(Value::as_str).unwrap_or("").to_string();
                let scope = meta.get("scope").and_then(Value::as_str).unwrap_or("global").to_string();
                db.execute(
                    "INSERT INTO nodes VALUES (?,?,?,?,?,?,?,?,?,?,?)",
                    params![
                        node_id,
                        s("type"),
                        s("title"),
                        body,
                        scope,
                        meta.get("importance").and_then(Value::as_i64).unwrap_or(3),
                        active as i64,
                        s("recorded_at"),
                        path,
                        crate::pyjson::dumps_utf8(&Value::Object(meta.clone())),
                        short_of(node_id),
                    ],
                )?;
                let rowid = db.last_insert_rowid();
                let keywords = match meta.get("keywords") {
                    Some(Value::Array(a)) => a.iter().map(|k| k.as_str().unwrap_or("").to_string()).collect::<Vec<_>>().join(" "),
                    Some(Value::String(s)) => s.clone(),
                    _ => String::new(),
                };
                db.execute(
                    "INSERT INTO fts(rowid, id, title, body, keywords) VALUES (?,?,?,?,?)",
                    params![rowid, node_id, s("title"), body, keywords],
                )?;
                for (rel, to) in nodefmt::edges_of(&meta) {
                    db.execute("INSERT INTO edges VALUES (?,?,?)", params![node_id, to, rel])?;
                }
                changed.insert(node_id.clone());
            }
            db.execute("INSERT OR REPLACE INTO files VALUES (?,?,?,?)", params![path, mtime, size, node_id])?;
        }
        Ok(changed)
    }

    fn session_files(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        if let Ok(days) = fs::read_dir(self.root.join("sessions")) {
            for d in days.flatten() {
                if let Ok(files) = fs::read_dir(d.path()) {
                    for f in files.flatten() {
                        if f.file_name().to_string_lossy().ends_with(".md") {
                            out.push(f.path());
                        }
                    }
                }
            }
        }
        out.sort();
        out
    }

    fn sync_episodes(&self, db: &Connection) -> Result<bool> {
        let mut seen: IndexMap<String, (i64, i64)> = IndexMap::new();
        for p in self.session_files() {
            if let Ok(md) = fs::metadata(&p) {
                seen.insert(p.to_string_lossy().into_owned(), (mtime_ns(&md), md.len() as i64));
            }
        }
        let mut known: HashMap<String, (i64, i64)> = HashMap::new();
        {
            let mut st = db.prepare("SELECT path, mtime_ns, size FROM epfiles")?;
            for r in st.query_map([], |r| Ok((r.get::<_, String>(0)?, (r.get(1)?, r.get(2)?))))? {
                let (p, v) = r?;
                known.insert(p, v);
            }
        }
        let mut changed = false;
        for path in known.keys() {
            if !seen.contains_key(path) {
                Self::delete_episodes(db, path)?;
                changed = true;
            }
        }
        static DATE: OnceLock<Regex> = OnceLock::new();
        let date_re = DATE.get_or_init(|| Regex::new(r"(?m)^- date: (\S+)").unwrap());
        for (path, (mtime, size)) in &seen {
            if known.get(path) == Some(&(*mtime, *size)) {
                continue;
            }
            Self::delete_episodes(db, path)?;
            changed = true;
            let content = fs::read_to_string(path).unwrap_or_default();
            let folder = Path::new(path).parent().and_then(|p| p.file_name()).map(|s| s.to_string_lossy().into_owned());
            let mut scope = String::from("global");
            for part in content.split("\n---\n") {
                let date = date_re.captures(part).map(|c| c[1].to_string()).unwrap_or_else(|| folder.clone().unwrap_or_default());
                scope = session_scope(part, &scope);
                let summary = session_summary(part);
                if !summary.is_empty() {
                    // the session's goal and outcome, searchable like an excerpt (first = last = 0)
                    db.execute("INSERT INTO episodes VALUES (?,?,?,?,?,?)", params![path, date, summary, 0, 0, scope])?;
                    let rid = db.last_insert_rowid();
                    db.execute("INSERT INTO fts_ep(rowid, text) VALUES (?,?)", params![rid, summary])?;
                }
                let Some((_, transcript)) = part.split_once("\n## Transcript\n") else { continue };
                let lines: Vec<&str> = transcript.lines().filter(|l| l.starts_with('[')).collect();
                let questions = session_questions(part);
                let n = lines.len();
                let mut start = 0;
                while start < n.saturating_sub(2).max(1) {
                    let part_lines = &lines[start.min(n)..(start + EP_WINDOW).min(n)];
                    let chunk = part_lines.join("\n");
                    if !chunk.is_empty() {
                        let turns: Vec<i64> =
                            part_lines.iter().filter_map(|l| turn_re().captures(l).and_then(|c| c[1].parse().ok())).collect();
                        let (first, last) = (turns.iter().min().copied().unwrap_or(0), turns.iter().max().copied().unwrap_or(0));
                        db.execute("INSERT INTO episodes VALUES (?,?,?,?,?,?)", params![path, date, chunk, first, last, scope])?;
                        let rid = db.last_insert_rowid();
                        // the chunk's hypothetical questions make it findable by them; the excerpt is unchanged
                        let indexed = std::iter::once(chunk.as_str())
                            .chain(questions.iter().filter(|(n, _)| (first..=last).contains(n)).map(|(_, q)| q.as_str()))
                            .collect::<Vec<_>>()
                            .join("\n");
                        db.execute("INSERT INTO fts_ep(rowid, text) VALUES (?,?)", params![rid, indexed])?;
                    }
                    start += EP_STEP;
                }
            }
            db.execute("INSERT OR REPLACE INTO epfiles VALUES (?,?,?)", params![path, mtime, size])?;
        }
        Ok(changed)
    }

    fn delete_episodes(db: &Connection, path: &str) -> Result<()> {
        db.execute("DELETE FROM fts_ep WHERE rowid IN (SELECT rowid FROM episodes WHERE path=?)", [path])?;
        db.execute("DELETE FROM epvec WHERE rid IN (SELECT rowid FROM episodes WHERE path=?)", [path])?;
        db.execute("DELETE FROM episodes WHERE path=?", [path])?;
        db.execute("DELETE FROM epfiles WHERE path=?", [path])?;
        Ok(())
    }

    fn event_files(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        if let Ok(devs) = fs::read_dir(self.root.join("events")) {
            for d in devs.flatten() {
                if let Ok(files) = fs::read_dir(d.path()) {
                    for f in files.flatten() {
                        if f.file_name().to_string_lossy().ends_with(".jsonl") {
                            out.push(f.path());
                        }
                    }
                }
            }
        }
        out.sort();
        out
    }

    fn sync_events(&self, db: &Connection) -> Result<std::result::Result<HashSet<String>, Stale>> {
        let mut touched = HashSet::new();
        let mut known: HashMap<String, (i64, String)> = HashMap::new();
        {
            let mut st = db.prepare("SELECT path, offset, tail FROM evfiles")?;
            for r in st.query_map([], |r| Ok((r.get::<_, String>(0)?, (r.get(1)?, r.get::<_, Option<String>>(2)?.unwrap_or_default()))))? {
                let (p, v) = r?;
                known.insert(p, v);
            }
        }
        for path in self.event_files() {
            let key = path.to_string_lossy().into_owned();
            let size = fs::metadata(&path).map(|m| m.len() as i64).unwrap_or(0);
            let row = known.get(&key);
            let offset = row.map(|r| r.0).unwrap_or(0);
            let mut fh = fs::File::open(&path)?;
            if let Some((_, tail)) = row {
                if size < offset {
                    return Ok(Err(Stale));
                }
                // tail is stored as latin-1 text of the last bytes before the offset
                let tail_bytes: Vec<u8> = tail.chars().map(|c| c as u32 as u8).collect();
                if !tail_bytes.is_empty() {
                    fh.seek(SeekFrom::Start((offset - tail_bytes.len() as i64).max(0) as u64))?;
                    let mut buf = vec![0u8; tail_bytes.len()];
                    if fh.read_exact(&mut buf).is_err() || buf != tail_bytes {
                        return Ok(Err(Stale));
                    }
                }
                if size == offset {
                    continue;
                }
            }
            fh.seek(SeekFrom::Start(offset as u64))?;
            let mut chunk = Vec::new();
            fh.read_to_end(&mut chunk)?;
            let Some(last_nl) = chunk.iter().rposition(|&b| b == b'\n') else { continue };
            let end = last_nl + 1; // only complete lines; a torn last line is read next time
            for line in String::from_utf8_lossy(&chunk[..end]).lines() {
                let Ok(ev) = serde_json::from_str::<Value>(line) else { continue };
                let evt = ev.get("ev").and_then(Value::as_str).unwrap_or("");
                let node = ev.get("node").and_then(Value::as_str).unwrap_or("");
                let t = ev.get("t").and_then(Value::as_str).unwrap_or("");
                if !activation::USE_EVENTS.contains(&evt) || node.is_empty() || t.is_empty() {
                    continue;
                }
                let session = ev.get("session").and_then(Value::as_str).filter(|s| !s.is_empty()).unwrap_or(t);
                let w = ev.get("w").and_then(Value::as_f64).unwrap_or(1.0);
                db.execute(
                    "INSERT INTO uses VALUES (?,?,?,?) ON CONFLICT(node, key) DO UPDATE \
                     SET t=excluded.t, w=excluded.w WHERE excluded.w > uses.w",
                    params![node, session, t, w],
                )?;
                touched.insert(node.to_string());
            }
            let new_offset = offset + end as i64;
            let tail: String = chunk[end.saturating_sub(64)..end].iter().map(|&b| b as char).collect();
            db.execute("INSERT OR REPLACE INTO evfiles VALUES (?,?,?)", params![key, new_offset, tail])?;
        }
        Ok(Ok(touched))
    }

    fn compute_act(&self, db: &Connection, ids: Option<Vec<String>>) -> Result<()> {
        let now = now_ts();
        let mut nodes: Vec<(String, String)> = Vec::new();
        let mut uses: Vec<(String, String, f64)> = Vec::new();
        match ids {
            None => {
                db.execute("DELETE FROM act", [])?;
                let mut st = db.prepare("SELECT id, recorded_at FROM nodes")?;
                for r in st.query_map([], |r| Ok((r.get(0)?, r.get::<_, Option<String>>(1)?.unwrap_or_default())))? {
                    nodes.push(r?);
                }
                let mut st = db.prepare("SELECT node, t, w FROM uses")?;
                for r in st.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))? {
                    uses.push(r?);
                }
            }
            Some(ids) => {
                for part in ids.chunks(500) {
                    let q = format!("SELECT id, recorded_at FROM nodes WHERE id IN ({})", marks(part.len()));
                    let mut st = db.prepare(&q)?;
                    for r in
                        st.query_map(params_from_iter(part), |r| Ok((r.get(0)?, r.get::<_, Option<String>>(1)?.unwrap_or_default())))?
                    {
                        nodes.push(r?);
                    }
                    let q = format!("SELECT node, t, w FROM uses WHERE node IN ({})", marks(part.len()));
                    let mut st = db.prepare(&q)?;
                    for r in st.query_map(params_from_iter(part), |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))? {
                        uses.push(r?);
                    }
                }
            }
        }
        let d = self.d;
        let mut total: IndexMap<String, f64> = IndexMap::new();
        let mut add = |id: &str, when: &str, w: f64| {
            if let Some(t) = parse_iso(when) {
                let age_h = ((now - t.timestamp_micros() as f64 / 1e6) / 3600.0).max(MIN_AGE_H);
                *total.entry(id.to_string()).or_insert(0.0) += w * age_h.powf(-d);
            }
        };
        let present: HashSet<&str> = nodes.iter().map(|(i, _)| i.as_str()).collect();
        for (id, rec) in &nodes {
            if !rec.is_empty() {
                add(id, rec, 1.0);
            }
        }
        for (node, t, w) in &uses {
            if present.contains(node.as_str()) {
                add(node, t, *w);
            }
        }
        for (id, v) in total {
            db.execute("INSERT OR REPLACE INTO act VALUES (?,?)", params![id, if v > 0.0 { v.ln() } else { FLOOR }])?;
        }
        Ok(())
    }

    // ---- queries ---------------------------------------------------------

    pub fn rows(&mut self, where_: &str, params: &[&dyn rusqlite::ToSql]) -> Result<Vec<NodeRow>> {
        let db = self.db()?;
        let sql = format!("SELECT nodes.*, COALESCE(act.b, -20) AS b FROM nodes LEFT JOIN act USING(id) WHERE {where_}");
        let mut st = db.prepare(&sql)?;
        let rows = st.query_map(params, NodeRow::from_row)?.collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    /// (neighbor full id, relation) in both directions.
    /// Linked notes: (id, relation, incoming), incoming when the other note's link points at this one (a note
    /// that supersedes this one links to it).
    pub fn neighbors(&mut self, node_id: &str) -> Result<Vec<(String, String, bool)>> {
        let short = short_of(node_id);
        let db = self.db()?;
        let mut st = db.prepare(
            "SELECT n.id, e.rel, 0 FROM edges e JOIN nodes n ON n.short = e.dst WHERE e.src = ? \
             UNION ALL SELECT e.src, e.rel, 1 FROM edges e WHERE e.dst = ?",
        )?;
        let out = st
            .query_map(params![node_id, short], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)? == 1)))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(out.into_iter().filter(|(id, _, _)| id != node_id).collect())
    }

    fn fetch(&mut self, info: &mut HashMap<String, NodeRow>, ids: &[String], scopes: Option<&[String]>) -> Result<()> {
        for part in ids.chunks(500) {
            let cond = scopes.map(|s| format!(" AND nodes.scope IN ({})", marks(s.len()))).unwrap_or_default();
            let sql = format!(
                "SELECT nodes.*, COALESCE(act.b,-20) AS b FROM nodes LEFT JOIN act USING(id) WHERE id IN ({}){cond}",
                marks(part.len())
            );
            let mut values: Vec<String> = part.to_vec();
            if let Some(s) = scopes {
                values.extend(s.iter().cloned());
            }
            let db = self.db()?;
            let mut st = db.prepare(&sql)?;
            for r in st.query_map(params_from_iter(values.iter()), NodeRow::from_row)? {
                let r = r?;
                info.insert(r.id.clone(), r);
            }
        }
        Ok(())
    }

    /// Keyword recall (BM25 over title/body/keywords + query stemming), time-window intent, then links: by default
    /// only from outdated memories to their replacements (`spread = "full"` spreads over the whole graph).
    ///
    /// Superseded memories hand their activation to their replacement; they are returned themselves
    /// (marked "superseded") only for history questions ("used to", "first", "dulu").
    pub fn search(&mut self, query: &str, opts: &SearchOpts) -> Result<Vec<Hit>> {
        let limit = (opts.k * 4).max(20);
        let scopes = opts.scopes.clone();
        let mut fts_ids: Vec<String> = Vec::new();
        let mut info: HashMap<String, NodeRow> = HashMap::new();
        let mut ranks: HashMap<String, f64> = HashMap::new();
        if let Some(m) = text::match_expression(query) {
            let mut sql = String::from(
                "SELECT nodes.*, COALESCE(act.b,-20) AS b, bm25(fts, 3.0, 1.0, 2.0) AS rank FROM fts \
                 JOIN nodes ON nodes.rowid = fts.rowid LEFT JOIN act ON act.id = nodes.id WHERE fts MATCH ?",
            );
            let mut values = vec![m];
            if let Some(s) = &scopes {
                sql.push_str(&format!(" AND nodes.scope IN ({})", marks(s.len())));
                values.extend(s.iter().cloned());
            }
            sql.push_str(" ORDER BY bm25(fts, 3.0, 1.0, 2.0), nodes.id LIMIT ?");
            values.push(limit.to_string());
            let db = self.db()?;
            let mut st = db.prepare(&sql)?;
            let rows = st.query_map(params_from_iter(values.iter()), |r| Ok((NodeRow::from_row(r)?, r.get::<_, f64>("rank")?)))?;
            for r in rows {
                let (row, rank) = r?;
                fts_ids.push(row.id.clone());
                ranks.insert(row.id.clone(), rank);
                info.insert(row.id.clone(), row);
            }
        }
        let sem_ids = if SEM_WEIGHT > 0.0 { self.semantic_ranks(query, scopes.as_ref(), SEM_TOP)? } else { Vec::new() };
        let mut fused: IndexSet<String> = fts_ids.iter().chain(sem_ids.iter()).cloned().collect();
        let window = opts.window.clone().or_else(|| text::time_window(query)); // an explicit window comes from the calling model
        if let Some((start, end)) = &window {
            // dated memories inside the asked window join the candidates
            let rows = self.rows("json_extract(meta, '$.valid_from') BETWEEN ? AND ? LIMIT 200", &[start as &dyn rusqlite::ToSql, end])?;
            for r in rows {
                if scopes.as_ref().is_none_or(|s| s.contains(&r.scope)) {
                    fused.insert(r.id.clone());
                    info.entry(r.id.clone()).or_insert(r);
                }
            }
        }
        if fused.is_empty() {
            return Ok(Vec::new());
        }
        let missing: Vec<String> = fused.iter().filter(|i| !info.contains_key(*i)).cloned().collect();
        self.fetch(&mut info, &missing, None)?;
        // BM25 keeps its magnitudes (they carry real signal).
        let (best, worst) = if ranks.is_empty() {
            (0.0, 0.0)
        } else {
            (ranks.values().cloned().fold(f64::INFINITY, f64::min), ranks.values().cloned().fold(f64::NEG_INFINITY, f64::max))
        };
        let span = if worst - best != 0.0 { worst - best } else { 1.0 };
        let mut rel: IndexMap<String, f64> =
            fused.iter().map(|i| (i.clone(), ranks.get(i).map_or(0.0, |r| 1.0 - (r - best) / span * 0.9))).collect();
        // the semantic ranker adds a bonus that decays with its rank (plain RRF flattened BM25 and hurt MRR)
        for (pos, i) in sem_ids.iter().enumerate() {
            if let Some(v) = rel.get_mut(i) {
                *v += SEM_WEIGHT * (1.0 - pos as f64 / sem_ids.len().max(1) as f64);
            }
        }
        let top = rel.values().cloned().fold(f64::NEG_INFINITY, f64::max);
        let top = if top != 0.0 { top } else { 1.0 };
        for v in rel.values_mut() {
            *v = (*v / top).max(0.1);
        }
        if let Some((start, end)) = &window {
            for (i, v) in rel.iter_mut() {
                let vf = info.get(i).map(|r| r.valid_from()).unwrap_or_default();
                if start.as_str() <= vf.as_str() && vf.as_str() <= end.as_str() {
                    *v += 0.5;
                }
            }
            let top = rel.values().cloned().fold(f64::NEG_INFINITY, f64::max);
            for v in rel.values_mut() {
                *v /= top;
            }
        }
        let history = opts.history.unwrap_or_else(|| text::history_intent(query));
        let beta = opts.beta.unwrap_or(self.recall_beta);
        let mut energy: IndexMap<String, f64> = rel.clone();
        let mut via: HashMap<String, String> = HashMap::new();
        // "supersedes" (default) only forwards an outdated memory to its replacement; "full" spreads over every link
        let full = self.spread == "full";
        if opts.spread && self.spread != "off" {
            let mut frontier: Vec<(String, f64)> = rel.iter().map(|(k, v)| (k.clone(), *v)).collect();
            frontier.sort_by(by_energy);
            frontier.truncate(10);
            for hop in [1, 2] {
                let mut nxt: IndexMap<String, f64> = IndexMap::new();
                for (src, amount) in &frontier {
                    let links = self.neighbors(src)?;
                    if links.is_empty() {
                        continue;
                    }
                    let fan = 1.0 + (1.0 + links.len() as f64).ln();
                    let src_inactive = info.get(src).is_some_and(|r| !r.active);
                    for (dst, relation, incoming) in links {
                        // an outdated note leads to the note that replaced it (the one whose link points at it),
                        // never back to the note it replaced itself
                        let forward = src_inactive && incoming && relation == "supersedes";
                        if !full && !forward {
                            continue;
                        }
                        let gain = if forward {
                            amount * 0.9 // an outdated memory points straight at its replacement
                        } else {
                            amount * rel_weight(&relation) * HOP_DECAY.powi(hop) / fan
                        };
                        if gain < 0.02 {
                            continue;
                        }
                        *nxt.entry(dst.clone()).or_insert(0.0) += gain;
                        if !rel.contains_key(&dst) && !via.contains_key(&dst) {
                            let label = if forward {
                                format!("replaces outdated: {}", info[src].title)
                            } else {
                                let other = info.get(src).map(|r| r.title.clone()).unwrap_or_else(|| short_of(src));
                                format!("{relation} ↔ {other}")
                            };
                            via.insert(dst.clone(), label);
                        }
                    }
                }
                for (dst, gain) in &nxt {
                    *energy.entry(dst.clone()).or_insert(0.0) += gain;
                }
                frontier = nxt.into_iter().collect();
                frontier.sort_by(by_energy);
                frontier.truncate(10);
            }
            let missing: Vec<String> = energy.keys().filter(|i| !info.contains_key(*i)).cloned().collect();
            self.fetch(&mut info, &missing, scopes.as_deref())?;
        }
        let mut out = Vec::new();
        for (node_id, e) in &energy {
            let Some(r) = info.get(node_id) else { continue }; // out of scope
            if r.type_ == "project" && !rel.contains_key(node_id) {
                continue; // hubs reached only through links
            }
            let status = if r.active { "active" } else { "superseded" };
            if status != "active" && !(opts.include_inactive || history) {
                continue;
            }
            let score = e * (1.0 + beta * activation::squash(r.b)) * if status == "active" { 1.0 } else { 0.6 };
            out.push(Hit {
                id: node_id.clone(),
                type_: r.type_.clone(),
                title: r.title.clone(),
                scope: r.scope.clone(),
                snippet: r.body.chars().take(160).collect(),
                score: round_to(score, 4),
                b: round_to(r.b, 3),
                date: r.valid_from(),
                status: status.into(),
                via: via.get(node_id).cloned(),
                session: String::new(),
                text: String::new(),
                summary: None,
            });
        }
        // ties by id: deterministic across implementations
        out.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal).then_with(|| a.id.cmp(&b.id)));
        out.truncate(opts.k);
        Ok(out)
    }

    /// Raw transcript excerpts (L1 episodic memory). `seeds` are memory hits for the same query: the
    /// excerpt each one was extracted from (its source session and turn) is boosted, so a well-matched
    /// fact brings its context.
    /// `scopes` limits excerpts to sessions of those scopes, as for memories (None: every session).
    pub fn search_episodes(
        &mut self,
        query: &str,
        k: usize,
        seeds: &[Hit],
        window: Option<&(String, String)>,
        scopes: Option<&[String]>,
    ) -> Result<Vec<Hit>> {
        let scopes = scopes.filter(|s| !s.is_empty());
        let cond = scopes.map(|s| format!(" AND e.scope IN ({})", marks(s.len()))).unwrap_or_default();
        let scope_params: Vec<String> = scopes.map(|s| s.to_vec()).unwrap_or_default();
        let limit = (k * 4).max(20);
        let mut fused: IndexMap<i64, f64> = IndexMap::new();
        if let Some(m) = text::match_expression(query) {
            let db = self.db()?;
            let mut st = db.prepare(&format!(
                "SELECT fts_ep.rowid FROM fts_ep JOIN episodes e ON e.rowid = fts_ep.rowid WHERE fts_ep MATCH ?{cond} \
                 ORDER BY bm25(fts_ep), e.path, e.first LIMIT ?"
            ))?;
            let mut p: Vec<rusqlite::types::Value> = vec![m.into()];
            p.extend(scope_params.iter().cloned().map(Into::into));
            p.push((limit as i64).into());
            let ids = st.query_map(params_from_iter(p), |r| r.get::<_, i64>(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
            for (rank, rid) in ids.into_iter().enumerate() {
                *fused.entry(rid).or_insert(0.0) += 1.0 / (RRF_K + rank as f64 + 1.0);
            }
        }
        for (rank, rid) in self.semantic_episode_ranks(query, scopes, limit)?.into_iter().enumerate() {
            *fused.entry(rid).or_insert(0.0) += EP_SEM_WEIGHT / (RRF_K + rank as f64 + 1.0);
        }
        let scale = fused.values().cloned().fold(f64::NEG_INFINITY, f64::max);
        let scale = if scale.is_finite() { scale } else { 1.0 / (RRF_K + 1.0) }; // bonus relative to the best match
        let seeded = self.seed_episodes(seeds)?;
        for (rid, bonus) in &seeded {
            *fused.entry(*rid).or_insert(0.0) += bonus * scale;
        }
        let window = window.cloned().or_else(|| text::time_window(query));
        let mut rows: HashMap<i64, EpRow> = HashMap::new();
        if !fused.is_empty() {
            let ids: Vec<i64> = fused.keys().cloned().collect();
            let db = self.db()?;
            for part in ids.chunks(500) {
                let sql =
                    format!("SELECT rowid, path, date, text, first, last FROM episodes e WHERE rowid IN ({}){cond}", marks(part.len()));
                let mut st = db.prepare(&sql)?;
                let mut p: Vec<rusqlite::types::Value> = part.iter().map(|&i| i.into()).collect();
                p.extend(scope_params.iter().cloned().map(Into::into));
                for r in st.query_map(params_from_iter(p), |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        (
                            r.get::<_, String>(1)?,
                            r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                            r.get::<_, String>(3)?,
                            r.get::<_, Option<i64>>(4)?.unwrap_or(0),
                            r.get::<_, Option<i64>>(5)?.unwrap_or(0),
                        ),
                    ))
                })? {
                    let (rid, v) = r?;
                    rows.insert(rid, v);
                }
            }
        }
        fused.retain(|rid, _| rows.contains_key(rid)); // seeds may point out of scope
        if let Some((start, end)) = &window {
            for (rid, (_, date, _, _, _)) in &rows {
                let d: String = date.chars().take(10).collect();
                if start.as_str() <= d.as_str() && d.as_str() <= end.as_str() {
                    *fused.get_mut(rid).unwrap() += 1.0 / RRF_K;
                }
            }
        }
        let mut order: Vec<(i64, f64)> = fused.into_iter().collect();
        let tie = |rid: &i64| rows.get(rid).map(|r| (r.0.clone(), r.3)).unwrap_or_default();
        order.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal).then_with(|| tie(&a.0).cmp(&tie(&b.0))));
        order.truncate(k);
        let root = self.root.clone();
        // the top excerpt carries its session's goal and outcome (bookends), unless it is the summary itself
        let mut bookend = None;
        if let Some((path, _, _, first, _)) = order.first().and_then(|(rid, _)| rows.get(rid)) {
            if *first != 0 {
                let db = self.db()?;
                let mut st = db.prepare("SELECT text FROM episodes WHERE path = ? AND first = 0 AND last = 0 ORDER BY rowid")?;
                let texts = st.query_map([path], |r| r.get::<_, String>(0))?.collect::<rusqlite::Result<Vec<_>>>()?;
                if !texts.is_empty() {
                    bookend = Some(texts.join(" / ").chars().take(300).collect::<String>());
                }
            }
        }
        let mut merged = merge_excerpts(&order.iter().map(|(rid, _)| *rid).collect::<Vec<_>>(), &rows);
        let mut opened: HashSet<String> = HashSet::new();
        for (pos, (rid, text)) in merged.iter_mut().enumerate() {
            let first = rows.get(rid).map(|r| r.3).unwrap_or(0);
            if pos >= EP_WHOLE && first != 0 && !seeded.contains_key(rid) {
                *text = trim_excerpt(query, text);
            }
            *text = condense(query, text);
            let path = rows.get(rid).map(|r| r.0.clone()).unwrap_or_default();
            if first > 1 && !opened.contains(&path) {
                // the session's opening user message, once per session
                if let Some(opener) = self.opener(&path)? {
                    *text = format!("{}\n{}", condense_line(query, &opener, OPENER_CHARS), text);
                }
            }
            opened.insert(path);
        }
        let mut out: Vec<Hit> = merged
            .into_iter()
            .filter_map(|(rid, text)| {
                let (path, date, _, _, _) = rows.get(&rid)?;
                Some(Hit {
                    id: format!("ep:{rid}"),
                    type_: "episode".into(),
                    title: String::new(),
                    scope: String::new(),
                    snippet: String::new(),
                    score: 0.0,
                    b: 0.0,
                    date: date.clone(),
                    status: String::new(),
                    via: None,
                    session: relpath(Path::new(path), &root),
                    text,
                    summary: None,
                })
            })
            .collect();
        if let Some(first) = out.first_mut() {
            first.summary = bookend;
        }
        Ok(out)
    }

    /// The first line of a session's first excerpt when it is the user's opening message.
    fn opener(&mut self, path: &str) -> Result<Option<String>> {
        let db = self.db()?;
        let text: Option<String> = db
            .query_row("SELECT text FROM episodes WHERE path = ? AND first = 1 ORDER BY rowid LIMIT 1", [path], |r| r.get(0))
            .optional()?;
        Ok(text.map(|t| t.split('\n').next().unwrap_or("").to_string()).filter(|l| l.starts_with("[1 user]")))
    }

    /// Episode rowid -> bonus, for the excerpts the seed memories were extracted from.
    fn seed_episodes(&mut self, seeds: &[Hit]) -> Result<IndexMap<i64, f64>> {
        let mut out = IndexMap::new();
        let seeds: Vec<&Hit> = seeds.iter().filter(|h| h.type_ != "episode").collect();
        if seeds.is_empty() || self.seed_weight <= 0.0 {
            return Ok(out);
        }
        let top = seeds.iter().map(|h| h.score).fold(f64::NEG_INFINITY, f64::max);
        let top = if top != 0.0 { top } else { 1.0 };
        let ids: Vec<String> = seeds.iter().map(|h| h.id.clone()).collect();
        let mut metas: HashMap<String, Meta> = HashMap::new();
        {
            let db = self.db()?;
            let sql = format!("SELECT id, meta FROM nodes WHERE id IN ({})", marks(ids.len()));
            let mut st = db.prepare(&sql)?;
            for r in st.query_map(params_from_iter(ids.iter()), |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))? {
                let (id, m) = r?;
                metas.insert(id, serde_json::from_str(&m).unwrap_or_default());
            }
        }
        let sep = std::path::MAIN_SEPARATOR;
        let seed_weight = self.seed_weight;
        let db = self.db()?;
        let mut st = db.prepare("SELECT rowid, first, last FROM episodes WHERE path LIKE ?")?;
        // excerpts per session file, read once: seeds often share a session, and LIKE scans the table
        let mut by_name: HashMap<String, Vec<TurnSpan>> = HashMap::new();
        for h in seeds {
            let Some(meta) = metas.get(&h.id) else { continue };
            let Some((harness, session, turn)) = nodefmt::parse_source(meta.get("source")) else { continue };
            let name = format!("{harness}-{}.md", slugify(&session, 40));
            let bonus = seed_weight * h.score / top;
            if !by_name.contains_key(&name) {
                let rows = st
                    .query_map(params![format!("%{sep}{name}")], |r| {
                        Ok((r.get::<_, i64>(0)?, r.get::<_, Option<i64>>(1)?, r.get::<_, Option<i64>>(2)?))
                    })?
                    .collect::<rusqlite::Result<Vec<_>>>()?;
                by_name.insert(name.clone(), rows);
            }
            for &(rid, first, last) in &by_name[&name] {
                if first.is_some_and(|f| f <= turn) && last.is_some_and(|l| l >= turn) {
                    // NULL never matches, as in SQL
                    *out.entry(rid).or_insert(0.0) += bonus;
                }
            }
        }
        Ok(out)
    }

    pub fn count_nodes(&mut self) -> Result<i64> {
        Ok(self.db()?.query_row("SELECT count(*) FROM nodes", [], |r| r.get(0))?)
    }
}

impl From<Stale> for Error {
    fn from(_: Stale) -> Self {
        Error::Other("stale index".into())
    }
}

#[cfg(test)]
mod trim_tests {
    use super::trim_excerpt;

    #[test]
    fn trim_keeps_matching_turns_and_their_neighbours() {
        let ex = "[1 user] Ann: hello there\n[2 user] Bob: the weather is nice\n[3 user] Ann: I adopted a dog\n\
                  [4 user] Bob: what is its name\n[5 user] Ann: we went hiking\n[6 user] Bob: sounds fun";
        assert_eq!(
            trim_excerpt("Ann dog", ex),
            "[2 user] Bob: the weather is nice\n[3 user] Ann: I adopted a dog\n[4 user] Bob: what is its name"
        );
        assert_eq!(trim_excerpt("kubernetes", ex), ex); // nothing to anchor on: kept whole
    }
}
