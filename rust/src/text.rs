//! Query-side text handling for recall: stopwords and light stemming (EN + ID), time windows,
//! history intent. Index text is stored as written; the query is widened instead: every content word
//! is searched as itself OR as a prefix of its stem(s), so "deployed" finds "deploy" and
//! "menyimpan" finds "simpan". Deliberately crude; measured in experiments/retrieval.

use chrono::{Datelike, Duration, NaiveDate};
use regex::Regex;
use std::collections::BTreeSet;
use std::sync::OnceLock;

const STOPWORDS_TEXT: &str = "
a an and are as at be been but by can could do does did for from has have how i if in into is it its
me my no not of on or our should so that the their them then there these they this to us was we were
what when where which who why will with would you your still any some about must need use used using
apa ada adalah agar akan aku anda apakah atau bagaimana bahwa banyak belum bila boleh bisa dalam dan
dari dengan di dia harus hanya ini itu jika juga kalau kami kamu kan ke kenapa ketika kita lagi lain
mana masih mau mereka nya oleh pada para perlu saat saja sama sampai saya sebelum sedang sesudah setelah
siapa sudah supaya tapi tetapi tidak untuk yang
";

fn stopwords() -> &'static std::collections::HashSet<&'static str> {
    static S: OnceLock<std::collections::HashSet<&'static str>> = OnceLock::new();
    S.get_or_init(|| STOPWORDS_TEXT.split_whitespace().collect())
}

pub fn is_stopword(w: &str) -> bool {
    stopwords().contains(w)
}

const VOWELS: &str = "aeiou";

fn clen(s: &str) -> usize {
    s.chars().count()
}

fn en(term: &str) -> Vec<String> {
    for (suffix, repl) in [("ies", "y"), ("ing", ""), ("ed", ""), ("es", ""), ("s", ""), ("ly", "")] {
        if term.ends_with(suffix) && clen(term) - clen(suffix) >= 3 {
            return vec![format!("{}{}", &term[..term.len() - suffix.len()], repl)];
        }
    }
    Vec::new()
}

fn id(term: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut base = term.to_string();
    for suffix in ["nya", "lah", "kah", "kan", "an", "i"] {
        if base.ends_with(suffix) && clen(&base) - clen(suffix) >= 4 {
            base.truncate(base.len() - suffix.len());
            out.push(base.clone());
            break;
        }
    }
    let words: BTreeSet<String> = [term.to_string(), base].into_iter().collect();
    for word in words {
        for (prefix, vowel_repl) in [
            ("meny", "s"), ("peny", "s"), ("meng", "k"), ("peng", "k"), ("mem", "p"), ("pem", "p"), ("men", "t"),
            ("pen", "t"), ("ber", ""), ("ter", ""), ("me", ""), ("pe", ""), ("di", ""), ("ke", ""), ("se", ""),
        ] {
            if word.starts_with(prefix) && clen(&word) - clen(prefix) >= 3 {
                let rest = &word[prefix.len()..];
                out.push(rest.to_string());
                if !vowel_repl.is_empty() && rest.chars().next().is_some_and(|c| VOWELS.contains(c)) {
                    out.push(format!("{vowel_repl}{rest}"));
                }
                break;
            }
        }
    }
    out
}

pub fn stems(term: &str) -> BTreeSet<String> {
    en(term).into_iter().chain(id(term)).filter(|s| s != term && clen(s) >= 3).collect()
}

fn words_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\w+").unwrap())
}

/// FTS5 MATCH expression: (term OR stem*) for each content word, OR-ed.
pub fn match_expression(query: &str) -> Option<String> {
    let lower = query.to_lowercase();
    let words: Vec<&str> = words_re().find_iter(&lower).map(|m| m.as_str()).filter(|w| clen(w) > 1).collect();
    let content: Vec<&str> = words.iter().copied().filter(|w| !is_stopword(w)).collect();
    let content = if content.is_empty() { words } else { content };
    let mut seen = std::collections::HashSet::new();
    let mut parts = Vec::new();
    for w in content {
        if !seen.insert(w) {
            continue;
        }
        let mut alts = vec![format!("\"{w}\"")];
        for s in stems(w) {
            alts.push(if clen(&s) >= 4 { format!("\"{s}\"*") } else { format!("\"{s}\"") });
        }
        parts.push(if alts.len() > 1 { format!("({})", alts.join(" OR ")) } else { alts.remove(0) });
    }
    if parts.is_empty() { None } else { Some(parts.join(" OR ")) }
}

pub fn content_words(query: &str) -> Vec<String> {
    let lower = query.to_lowercase();
    let words: Vec<&str> = words_re().find_iter(&lower).map(|m| m.as_str()).filter(|w| clen(w) > 1).collect();
    let content: Vec<&str> = words.iter().copied().filter(|w| !is_stopword(w)).collect();
    let content = if content.is_empty() { words } else { content };
    let mut seen = std::collections::HashSet::new();
    content.into_iter().filter(|w| seen.insert(*w)).map(String::from).collect()
}

/// How many distinct content words of the query a document contains (as itself or a stem; words and
/// stems of 4+ letters match as prefixes). An absolute relevance signal: BM25 scores are only
/// comparable within one query.
pub fn overlap(query: &str, doc: &str) -> usize {
    let lower = doc.to_lowercase();
    let tokens: std::collections::HashSet<&str> = words_re().find_iter(&lower).map(|m| m.as_str()).collect();
    content_words(query)
        .iter()
        .filter(|w| {
            tokens.contains(w.as_str())
                || stems(w).iter().chain(std::iter::once(*w)).any(|s| tokens.iter().any(|t| *t == s.as_str() || (clen(s) >= 4 && t.starts_with(s.as_str()))))
        })
        .count()
}

fn month_number(name: &str) -> Option<u32> {
    const FULL: [&str; 12] = [
        "january", "february", "march", "april", "may", "june", "july", "august", "september", "october", "november",
        "december",
    ];
    if let Some(i) = FULL.iter().position(|m| *m == name) {
        return Some(i as u32 + 1);
    }
    if let Some(i) = FULL.iter().position(|m| &m[..3] == name) {
        return Some(i as u32 + 1);
    }
    match name {
        "januari" => Some(1),
        "februari" => Some(2),
        "maret" => Some(3),
        "mei" => Some(5),
        "juni" => Some(6),
        "juli" => Some(7),
        "agustus" | "agu" => Some(8),
        "oktober" | "okt" => Some(10),
        "desember" | "des" => Some(12),
        _ => None,
    }
}

fn month_window(y: i32, m: u32) -> Option<(String, String)> {
    let first = NaiveDate::from_ymd_opt(y, m, 1)?;
    let next = if m == 12 { NaiveDate::from_ymd_opt(y + 1, 1, 1)? } else { NaiveDate::from_ymd_opt(y, m + 1, 1)? };
    let last = next - Duration::days(1);
    Some((first.format("%Y-%m-%d").to_string(), format!("{:04}-{:02}-{:02}", y, m, last.day())))
}

fn days_window(y: i32, m: u32, d: u32, slack: i64) -> Option<(String, String)> {
    match NaiveDate::from_ymd_opt(y, m, d) {
        Some(center) => Some((
            (center - Duration::days(slack)).format("%Y-%m-%d").to_string(),
            (center + Duration::days(slack)).format("%Y-%m-%d").to_string(),
        )),
        None => month_window(y, m),
    }
}

fn re(pattern: &'static str, cell: &'static OnceLock<Regex>) -> &'static Regex {
    cell.get_or_init(|| Regex::new(pattern).unwrap())
}

/// Explicit time in the query -> (start, end) ISO dates, e.g. "in July 2023" -> July 2023.
pub fn time_window(query: &str) -> Option<(String, String)> {
    static ISO: OnceLock<Regex> = OnceLock::new();
    static DMY: OnceLock<Regex> = OnceLock::new();
    static MDY: OnceLock<Regex> = OnceLock::new();
    static YEAR: OnceLock<Regex> = OnceLock::new();
    let q = query.to_lowercase();
    if q.chars().filter(|c| c.is_numeric()).count() < 4 {
        return None; // every pattern below needs a four-digit year: skip compiling them
    }
    if let Some(c) = re(r"\b(\d{4})-(\d{2})(?:-(\d{2}))?\b", &ISO).captures(&q) {
        let mo: u32 = c[2].parse().ok()?;
        if (1..=12).contains(&mo) {
            let y: i32 = c[1].parse().ok()?;
            return match c.get(3) {
                Some(d) => days_window(y, mo, d.as_str().parse().ok()?, 3),
                None => month_window(y, mo),
            };
        }
    }
    if let Some(c) = re(r"\b(?:(\d{1,2})\s+)?([a-z]{3,9})\.?,?\s+(\d{4})\b", &DMY).captures(&q) {
        if let Some(mo) = month_number(&c[2]) {
            let y: i32 = c[3].parse().ok()?;
            return match c.get(1) {
                Some(d) => days_window(y, mo, d.as_str().parse().ok()?, 3),
                None => month_window(y, mo),
            };
        }
    }
    if let Some(c) = re(r"\b([a-z]{3,9})\s+(\d{1,2}),?\s+(\d{4})\b", &MDY).captures(&q) {
        if let Some(mo) = month_number(&c[1]) {
            return days_window(c[3].parse().ok()?, mo, c[2].parse().ok()?, 3);
        }
    }
    if let Some(c) = re(r"\b(?:in|during|tahun|pada)\s+((?:19|20)\d{2})\b", &YEAR).captures(&q) {
        return Some((format!("{}-01-01", &c[1]), format!("{}-12-31", &c[1])));
    }
    None
}

/// The question is about the past ("used to", "first", "dulu"): superseded memories are relevant.
pub fn history_intent(query: &str) -> bool {
    const WORDS: [&str; 14] = ["used to", "previously", "originally", "formerly", "first", "earlier", "before", "old",
        "dulu", "sebelumnya", "awalnya", "pertama", "semula", "pernah"];
    // (?i) also folds U+212A KELVIN SIGN to k (to_lowercase does too) and U+017F LONG S to s
    let low = query.to_lowercase().replace('\u{17f}', "s");
    if !WORDS.iter().any(|w| low.contains(w)) {
        return false; // none of the words: skip compiling the pattern (most queries)
    }
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r"(?i)\b(used to|previously|originally|formerly|at first|first|earlier|before|old|dulu|sebelumnya|awalnya|pertama|semula|pernah)\b",
        )
        .unwrap()
    })
    .is_match(query)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stemming() {
        assert!(stems("deployed").contains("deploy"));
        assert!(stems("menyimpan").contains("simpan"));
        assert_eq!(match_expression("the deploys"), Some("(\"deploys\" OR \"deploy\"*)".into()));
    }

    #[test]
    fn windows() {
        assert_eq!(time_window("What did Melanie paint in July 2023?"), Some(("2023-07-01".into(), "2023-07-31".into())));
        assert_eq!(time_window("on 8 May, 2023"), Some(("2023-05-05".into(), "2023-05-11".into())));
        assert_eq!(time_window("the 2022-23 season"), None);
        assert_eq!(time_window("since 2023-07"), Some(("2023-07-01".into(), "2023-07-31".into())));
        assert!(history_intent("what did I use before"));
        assert!(!history_intent("what do I use"));
    }
}
