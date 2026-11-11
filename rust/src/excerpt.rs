//! How a recalled transcript excerpt is shaped: merged with its neighbours, trimmed around the query and
//! condensed to its most relevant sentences. Pure string work: no database, no index.

use crate::text;
use crate::util::take;
use regex::Regex;
use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;

/// One episode row: path, date, text, first turn, last turn.
pub struct Chunk {
    pub path: String,
    pub date: String,
    pub text: String,
    pub lo: i64,
    pub hi: i64,
}

/// Excerpts of one session whose turn ranges overlap or touch become one excerpt (no line twice), at the
/// rank of the best of them. Windows overlap by two turns, so without this the same lines are sent twice:
/// 11-12% of the excerpt tokens on LoCoMo, with the same evidence (experiments/eval/results).
pub fn merge(best: &[i64], rows: &HashMap<i64, Chunk>) -> Vec<(i64, String)> {
    struct Group<'a> {
        rid: i64,
        path: &'a str,
        lo: i64,
        hi: i64,
        lines: BTreeMap<i64, &'a str>,
    }
    let mut groups: Vec<Group> = Vec::new();
    for rid in best {
        let Some(Chunk { path, text, lo, hi, .. }) = rows.get(rid) else { continue };
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
pub const EP_WHOLE: usize = 2;
const EP_CONTEXT: usize = 1;

pub fn trim(query: &str, excerpt: &str) -> String {
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

/// Transcripts keep long turns whole (a detail late in a long assistant reply is still findable); an excerpt
/// line longer than LINE_CHARS is shown condensed to the sentences that best match the query, plus its opening.
/// Keywords only: embedding every sentence at recall time cost ~4x the latency for the same evidence.
const LINE_CHARS: usize = 1500;
/// The user's opening message states their situation, and facts are often said there in passing; an excerpt
/// from later in the session would lose it. On LongMemEval_S, 34 of the 45 evidence turns missing from
/// recalled sessions were turn 1. Shown condensed with every excerpt after it.
pub const OPENER_CHARS: usize = 700;

pub fn condense(query: &str, excerpt: &str) -> String {
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

pub fn condense_line(query: &str, line: &str, limit: usize) -> String {
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

/// The `[<turn> ` prefix of a transcript line, so shaping and the sync agree on turn numbers.
pub fn turn_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"^\[(\d+) ").unwrap())
}

#[cfg(test)]
mod trim_tests {
    use super::trim;

    #[test]
    fn trim_keeps_matching_turns_and_their_neighbours() {
        let ex = "[1 user] Ann: hello there\n[2 user] Bob: the weather is nice\n[3 user] Ann: I adopted a dog\n\
                  [4 user] Bob: what is its name\n[5 user] Ann: we went hiking\n[6 user] Bob: sounds fun";
        assert_eq!(trim("Ann dog", ex), "[2 user] Bob: the weather is nice\n[3 user] Ann: I adopted a dog\n[4 user] Bob: what is its name");
        assert_eq!(trim("kubernetes", ex), ex); // nothing to anchor on: kept whole
    }
}

#[cfg(test)]
mod merge_tests {
    use super::{merge, Chunk};
    use std::collections::HashMap;

    fn chunk(path: &str, turns: std::ops::RangeInclusive<i64>) -> Chunk {
        let (lo, hi) = (*turns.start(), *turns.end());
        let text = turns.map(|n| format!("[{n} user] turn {n}")).collect::<Vec<_>>().join("\n");
        Chunk { path: path.into(), date: "2026-01-01".into(), text, lo, hi }
    }

    fn turns(text: &str) -> Vec<i64> {
        text.lines().map(|l| super::turn_re().captures(l).unwrap()[1].parse().unwrap()).collect()
    }

    #[test]
    fn adjacent_windows_of_one_session_merge_without_repeating_a_turn() {
        let rows: HashMap<i64, Chunk> =
            [(1, chunk("sessions/a.md", 1..=6)), (2, chunk("sessions/a.md", 5..=10))].into_iter().collect();
        let merged = merge(&[1, 2], &rows);
        assert_eq!(merged.len(), 1, "two overlapping windows are one excerpt");
        assert_eq!(merged[0].0, 1, "the excerpt keeps the rank of the better window");
        assert_eq!(turns(&merged[0].1), (1..=10).collect::<Vec<_>>(), "each turn once, in order");
    }

    #[test]
    fn windows_of_different_sessions_stay_apart() {
        let rows: HashMap<i64, Chunk> =
            [(1, chunk("sessions/a.md", 1..=6)), (2, chunk("sessions/b.md", 5..=10))].into_iter().collect();
        assert_eq!(merge(&[1, 2], &rows).len(), 2, "sessions are never joined");
    }

    #[test]
    fn windows_with_a_gap_stay_apart() {
        let rows: HashMap<i64, Chunk> =
            [(1, chunk("sessions/a.md", 1..=3)), (2, chunk("sessions/a.md", 9..=11))].into_iter().collect();
        assert_eq!(merge(&[1, 2], &rows).len(), 2, "a gap means the missing turns would look present");
    }

    #[test]
    fn a_session_summary_is_never_folded_into_an_excerpt() {
        let summary = Chunk { path: "sessions/a.md".into(), date: "2026-01-01".into(), text: "done: shipped".into(), lo: 0, hi: 0 };
        let rows: HashMap<i64, Chunk> = [(1, summary), (2, chunk("sessions/a.md", 1..=6))].into_iter().collect();
        assert_eq!(merge(&[1, 2], &rows).len(), 2, "turn 0 means no transcript, so no line order to merge by");
    }

    #[test]
    fn runs_that_reuse_turn_numbers_are_not_merged() {
        let mut other = chunk("sessions/a.md", 3..=5);
        other.text = other.text.replace("turn", "second pass");
        let rows: HashMap<i64, Chunk> = [(1, chunk("sessions/a.md", 1..=6)), (2, other)].into_iter().collect();
        assert_eq!(merge(&[1, 2], &rows).len(), 2, "the same turn number with different text is a different run");
    }
}
