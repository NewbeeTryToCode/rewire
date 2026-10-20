//! JSON text exactly as Python's `json.dumps` writes it (separators `", "` / `": "`).
//!
//! The on-disk format (docs/design/format.md) was first written by a Python implementation (since removed), so
//! brains made by it read and diff unchanged: frontmatter values, event lines and inbox items all go through this.

use serde_json::Value;
use std::fmt::Write;

#[derive(Clone, Copy, Default)]
pub struct Opts {
    pub ensure_ascii: bool,
    pub sort_keys: bool,
}

pub fn dumps(value: &Value, opts: Opts) -> String {
    let mut out = String::new();
    write_value(&mut out, value, opts);
    out
}

/// `json.dumps(value, ensure_ascii=False)`.
pub fn dumps_utf8(value: &Value) -> String {
    dumps(value, Opts { ensure_ascii: false, sort_keys: false })
}

fn write_value(out: &mut String, value: &Value, opts: Opts) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => out.push_str(&number(n)),
        Value::String(s) => write_str(out, s, opts.ensure_ascii),
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                write_value(out, item, opts);
            }
            out.push(']');
        }
        Value::Object(map) => {
            out.push('{');
            let mut keys: Vec<&String> = map.keys().collect();
            if opts.sort_keys {
                keys.sort();
            }
            for (i, key) in keys.into_iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                write_str(out, key, opts.ensure_ascii);
                out.push_str(": ");
                write_value(out, &map[key], opts);
            }
            out.push('}');
        }
    }
}

/// Python float repr: always has a '.' or an exponent ("1.0", "0.5", "1e-05").
fn number(n: &serde_json::Number) -> String {
    if let Some(f) = n.as_f64().filter(|_| n.is_f64()) {
        if f.is_finite() && f == f.trunc() && f.abs() < 1e16 {
            return format!("{f:.1}");
        }
        let s = format!("{f:?}");
        // Rust prints 1e-5 where Python prints 1e-05
        if let Some(pos) = s.find('e') {
            let (mant, exp) = s.split_at(pos);
            let exp = &exp[1..];
            let (sign, digits) = if let Some(d) = exp.strip_prefix('-') { ("-", d) } else { ("+", exp) };
            return format!("{mant}e{sign}{digits:0>2}");
        }
        return s;
    }
    n.to_string()
}

fn write_str(out: &mut String, s: &str, ensure_ascii: bool) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c if ensure_ascii && (c as u32) > 0x7e => {
                let mut buf = [0u16; 2];
                for unit in c.encode_utf16(&mut buf) {
                    let _ = write!(out, "\\u{:04x}", unit);
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// `json.dumps(data, indent=1, sort_keys=True)` (device-local state files).
pub fn dumps_indent1_sorted(value: &Value) -> String {
    let mut out = String::new();
    write_indented(&mut out, value, 0, Opts { ensure_ascii: true, sort_keys: true });
    out
}

/// `json.dumps(data, ensure_ascii=False, indent=1)` (CLI --json output).
pub fn dumps_indent1(value: &Value) -> String {
    let mut out = String::new();
    write_indented(&mut out, value, 0, Opts { ensure_ascii: false, sort_keys: false });
    out
}

fn write_indented(out: &mut String, value: &Value, depth: usize, opts: Opts) {
    match value {
        Value::Array(items) if !items.is_empty() => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                out.push_str(if i > 0 { ",\n" } else { "\n" });
                out.push_str(&" ".repeat(depth + 1));
                write_indented(out, item, depth + 1, opts);
            }
            out.push('\n');
            out.push_str(&" ".repeat(depth));
            out.push(']');
        }
        Value::Object(map) if !map.is_empty() => {
            out.push('{');
            let mut keys: Vec<&String> = map.keys().collect();
            if opts.sort_keys {
                keys.sort();
            }
            for (i, key) in keys.into_iter().enumerate() {
                out.push_str(if i > 0 { ",\n" } else { "\n" });
                out.push_str(&" ".repeat(depth + 1));
                write_str(out, key, opts.ensure_ascii);
                out.push_str(": ");
                write_indented(out, &map[key], depth + 1, opts);
            }
            out.push('\n');
            out.push_str(&" ".repeat(depth));
            out.push('}');
        }
        other => write_value(out, other, opts),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn matches_python() {
        assert_eq!(dumps_utf8(&json!(["a", "b"])), r#"["a", "b"]"#);
        assert_eq!(dumps_utf8(&json!({"k": 1, "w": 1.0})), r#"{"k": 1, "w": 1.0}"#);
        assert_eq!(dumps_utf8(&json!("Ünï · x")), "\"Ünï · x\"");
        assert_eq!(dumps(&json!("\u{e9}"), Opts { ensure_ascii: true, sort_keys: false }), "\"\\u00e9\"");
        assert_eq!(dumps(&json!("\u{1F600}"), Opts { ensure_ascii: true, sort_keys: false }), "\"\\ud83d\\ude00\"");
        assert_eq!(dumps_utf8(&json!(0.5)), "0.5");
        assert_eq!(dumps_indent1_sorted(&json!({"b": 1, "a": {"x": [1]}})), "{\n \"a\": {\n  \"x\": [\n   1\n  ]\n },\n \"b\": 1\n}");
    }
}
