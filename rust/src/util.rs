use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};
use rand::RngCore;
use std::fs;
use std::io::Write;
use std::path::Path;

const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// 26-char ULID: 48-bit millisecond timestamp + 80 random bits.
pub fn ulid() -> String {
    let ms = Utc::now().timestamp_millis() as u128;
    let mut rnd = [0u8; 10];
    rand::thread_rng().fill_bytes(&mut rnd);
    let mut value: u128 = ms << 80;
    for (i, b) in rnd.iter().enumerate() {
        value |= (*b as u128) << (8 * (9 - i));
    }
    let mut out = [0u8; 26];
    for slot in out.iter_mut().rev() {
        *slot = CROCKFORD[(value & 31) as usize];
        value >>= 5;
    }
    String::from_utf8(out.to_vec()).unwrap()
}

pub fn now_iso() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

/// The clock recall ranks by (activation decays with age). REWIRE_NOW (RFC 3339) pins it, so a benchmark
/// recalls every question as of the moment its brain finished ingesting.
pub fn now_ts() -> f64 {
    static PINNED: std::sync::OnceLock<Option<f64>> = std::sync::OnceLock::new();
    let pinned = PINNED.get_or_init(|| {
        std::env::var("REWIRE_NOW").ok().and_then(|v| parse_iso(v.trim())).map(|t| t.timestamp_micros() as f64 / 1e6)
    });
    pinned.unwrap_or_else(|| Utc::now().timestamp_micros() as f64 / 1e6)
}

/// Parse "2026-10-11T11:05:54Z" (or any RFC 3339 / bare date) to UTC.
pub fn parse_iso(value: &str) -> Option<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f") {
        return Some(dt.and_utc());
    }
    NaiveDate::parse_from_str(value.get(..10)?, "%Y-%m-%d").ok().map(|d| d.and_hms_opt(0, 0, 0).unwrap().and_utc())
}

/// Python `re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-")[:max_len].rstrip("-") or "node"`.
pub fn slugify(text: &str, max_len: usize) -> String {
    let lower = text.to_lowercase();
    let mut slug = String::new();
    let mut dash = false;
    for c in lower.chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() {
            slug.push(c);
            dash = false;
        } else if !dash {
            slug.push('-');
            dash = true;
        }
    }
    let slug = slug.trim_matches('-');
    let cut: String = slug.chars().take(max_len).collect();
    let cut = cut.trim_end_matches('-');
    if cut.is_empty() { "node".to_string() } else { cut.to_string() }
}

pub fn atomic_write(path: &Path, content: &str) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_file_name(format!(
        "{}.tmp-{}",
        path.file_name().unwrap().to_string_lossy(),
        std::process::id()
    ));
    {
        let mut fh = fs::File::create(&tmp)?;
        fh.write_all(content.as_bytes())?;
        fh.sync_all()?; // the rename must never expose an empty or half-written file after a crash
    }
    fs::rename(tmp, path)
}

/// First `n` characters (Python `s[:n]` counts code points, not bytes).
pub fn take(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

pub fn char_len(s: &str) -> usize {
    s.chars().count()
}

/// Last 8 chars of an id, lower case.
pub fn short_of(id: &str) -> String {
    let chars: Vec<char> = id.chars().collect();
    let start = chars.len().saturating_sub(8);
    chars[start..].iter().collect::<String>().to_lowercase()
}

/// Python `os.path.relpath(target, start)` for absolute paths.
pub fn relpath(target: &Path, start: &Path) -> String {
    let t: Vec<_> = target.components().collect();
    let s: Vec<_> = start.components().collect();
    let common = t.iter().zip(s.iter()).take_while(|(a, b)| a == b).count();
    let mut parts: Vec<String> = Vec::new();
    for _ in common..s.len() {
        parts.push("..".into());
    }
    for c in &t[common..] {
        parts.push(c.as_os_str().to_string_lossy().into_owned());
    }
    if parts.is_empty() { ".".into() } else { parts.join("/") }
}

/// Python `round(x, 4)` (correctly rounded, half-even on the exact binary value).
pub fn round_to(x: f64, digits: usize) -> f64 {
    format!("{x:.digits$}").parse().unwrap_or(x)
}


/// Run a command with a deadline. stdout and stderr are drained on their own threads (a child blocked on a full
/// pipe would otherwise hang until the deadline), stdin is written on another, and a child past the deadline is
/// killed and reaped. Err(TimedOut) on the deadline.
pub fn run_with_timeout(mut cmd: std::process::Command, input: Option<String>, timeout: std::time::Duration) -> std::io::Result<std::process::Output> {
    use std::io::{Read, Write};
    use std::process::Stdio;
    cmd.stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() }).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn()?;
    let writer = child.stdin.take().map(|mut stdin| {
        let data = input.unwrap_or_default();
        std::thread::spawn(move || {
            let _ = stdin.write_all(data.as_bytes());
        })
    });
    let drain = |pipe: Option<Box<dyn Read + Send>>| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(mut p) = pipe {
                let _ = p.read_to_end(&mut buf);
            }
            buf
        })
    };
    let out = drain(child.stdout.take().map(|p| Box::new(p) as Box<dyn Read + Send>));
    let err = drain(child.stderr.take().map(|p| Box::new(p) as Box<dyn Read + Send>));
    let deadline = std::time::Instant::now() + timeout;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(std::io::Error::new(std::io::ErrorKind::TimedOut, "timed out"));
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    };
    if let Some(w) = writer {
        let _ = w.join();
    }
    Ok(std::process::Output { status, stdout: out.join().unwrap_or_default(), stderr: err.join().unwrap_or_default() })
}

/// A JSON value as plain text for memory content: a string as itself, null as "" (never "None"), anything else
/// as its JSON text.
pub fn value_text(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs() {
        assert_eq!(slugify("Melanie's family: mountain camping trip!", 48), "melanie-s-family-mountain-camping-trip");
        assert_eq!(slugify("conv-26-session_1", 40), "conv-26-session-1");
        assert_eq!(slugify("¿?", 48), "node");
    }

    #[test]
    fn ulids() {
        let a = ulid();
        assert_eq!(a.len(), 26);
        assert!(a.chars().all(|c| CROCKFORD.contains(&(c as u8))));
    }

    #[test]
    fn relpaths() {
        assert_eq!(relpath(Path::new("/b/nodes/project/x.md"), Path::new("/b/nodes/fact")), "../project/x.md");
        assert_eq!(relpath(Path::new("/b/nodes/fact/y.md"), Path::new("/b/nodes/fact")), "y.md");
    }
}
