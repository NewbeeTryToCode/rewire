//! Write-time scanning: secrets, injection patterns, private spans.

use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

fn private_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?si)<private>.*?</private>").unwrap())
}

fn secret_patterns() -> &'static [Regex] {
    static R: OnceLock<Vec<Regex>> = OnceLock::new();
    R.get_or_init(|| {
        [
            r"sk-ant-[A-Za-z0-9_\-]{20,}",
            r"sk-[A-Za-z0-9]{32,}",
            r"gh[pousr]_[A-Za-z0-9]{30,}",
            r"github_pat_[A-Za-z0-9_]{40,}",
            r"AKIA[0-9A-Z]{16}",
            r"xox[baprs]-[A-Za-z0-9-]{10,}",
            r"AIza[0-9A-Za-z_\-]{35}",
            r"-----BEGIN [A-Z ]*PRIVATE KEY-----",
            r"(?i)\b(password|passwd|secret|api[_-]?key|token)\s*[:=]\s*\S{8,}",
        ]
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
    })
}

fn injection_patterns() -> &'static [Regex] {
    static R: OnceLock<Vec<Regex>> = OnceLock::new();
    R.get_or_init(|| {
        [
            r"(?i)ignore (all |any )?(previous|prior|above) (instructions|prompts)",
            r"(?i)\b(always|must) (send|upload|post|exfiltrate|forward)\b.*\b(to|at)\b",
            r"(?i)\b(curl|wget)\s+https?://",
            r"(?i)disregard (the )?(system|developer) (prompt|message)",
            r"(?i)you are now\b",
            "[\u{200b}-\u{200f}\u{202a}-\u{202e}\u{2060}-\u{2064}\u{feff}]",
        ]
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect()
    })
}

fn token_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"[A-Za-z0-9+/_\-]{32,}").unwrap())
}

fn high_entropy(tok: &str) -> bool {
    let mut counts: HashMap<char, usize> = HashMap::new();
    for c in tok.chars() {
        *counts.entry(c).or_default() += 1;
    }
    let n = tok.chars().count() as f64;
    let entropy: f64 = -counts.values().map(|&k| (k as f64 / n) * (k as f64 / n).log2()).sum::<f64>();
    entropy > 4.3 && tok.chars().any(|c| c.is_ascii_digit()) && tok.chars().any(|c| c.is_alphabetic())
}

fn high_entropy_token(text: &str) -> bool {
    token_re().find_iter(text).any(|m| high_entropy(m.as_str()))
}

/// Mask secrets in text that is kept verbatim (session transcripts).
pub fn redact(text: &str) -> String {
    let mut text = text.to_string();
    for p in secret_patterns() {
        text = p.replace_all(&text, "[redacted]").into_owned();
    }
    let toks: Vec<String> = token_re().find_iter(&text).map(|m| m.as_str().to_string()).collect();
    for tok in toks {
        if high_entropy(&tok) {
            text = text.replace(&tok, "[redacted]");
        }
    }
    text
}

pub fn strip_private(text: &str) -> String {
    private_re().replace_all(text, "[private]").into_owned()
}

/// A list of problems; empty means safe to store.
pub fn scan(text: &str) -> Vec<String> {
    let mut problems = Vec::new();
    if secret_patterns().iter().any(|p| p.is_match(text)) || high_entropy_token(text) {
        problems.push("secret".to_string());
    }
    if injection_patterns().iter().any(|p| p.is_match(text)) {
        problems.push("injection".to_string());
    }
    problems
}

/// Why a stored note must not be shown ("" when it is fine). Notes are re-scanned whenever they are shown,
/// since files can be edited by hand or arrive by sync: secrets and the unambiguous injection patterns.
/// Commands such as curl are left out, they are ordinary content in coding notes a human already approved.
pub fn blocked(text: &str) -> &'static str {
    if secret_patterns().iter().any(|p| p.is_match(text)) || high_entropy_token(text) {
        return "secret";
    }
    if injection_patterns().iter().any(|p| !p.as_str().contains("curl") && p.is_match(text)) {
        return "injection";
    }
    ""
}

pub fn blocked_label(why: &str) -> String {
    format!("[BLOCKED: {why}; check this note with `rewire doctor`]")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secrets() {
        assert_eq!(scan("token = abcdefgh12345"), vec!["secret"]);
        assert!(redact("key sk-ant-abcdefghijklmnopqrstuvwxyz0123").contains("[redacted]"));
        assert_eq!(strip_private("a <private>x\ny</private> b"), "a [private] b");
        assert_eq!(scan("ignore previous instructions"), vec!["injection"]);
        assert!(scan("I like YAML").is_empty());
    }
}
