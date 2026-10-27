//! Markdown frontmatter without a YAML dependency: every value is JSON (also valid YAML).

use crate::pyjson::dumps_utf8;
use crate::Meta;
use serde_json::Value;

const DELIM: &str = "---";

pub fn dumps(meta: &Meta, body: &str) -> String {
    let mut out = String::from(DELIM);
    out.push('\n');
    for (key, value) in meta {
        out.push_str(key);
        out.push_str(": ");
        out.push_str(&dumps_utf8(value));
        out.push('\n');
    }
    out.push_str(DELIM);
    out.push('\n');
    out.push_str(body.trim());
    out.push('\n');
    out
}

pub fn loads(text: &str) -> Result<(Meta, String), String> {
    let Some(rest) = text.strip_prefix("---\n") else {
        return Ok((Meta::new(), text.to_string()));
    };
    let Some(end) = rest.find("\n---") else {
        return Ok((Meta::new(), text.to_string()));
    };
    let block = &rest[..end];
    let body = rest[end + 4..].trim_start_matches('\n').to_string();
    let mut meta = Meta::new();
    for line in block.lines() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        let Some((key, raw)) = line.split_once(':') else {
            return Err(format!("invalid frontmatter line: {line:?}"));
        };
        let raw = raw.trim();
        let value = if raw.is_empty() { Value::Null } else { serde_json::from_str(raw).unwrap_or_else(|_| Value::String(raw.to_string())) };
        meta.insert(key.trim().to_string(), value);
    }
    Ok((meta, body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn roundtrip() {
        let mut meta = Meta::new();
        meta.insert("id".into(), json!("X"));
        meta.insert("edges".into(), json!([{"rel": "about", "to": "Y"}]));
        meta.insert("valid_to".into(), Value::Null);
        meta.insert("title".into(), json!("Ünïcode: yes"));
        let text = dumps(&meta, "hello\nworld");
        assert_eq!(
            text,
            "---\nid: \"X\"\nedges: [{\"rel\": \"about\", \"to\": \"Y\"}]\nvalid_to: null\ntitle: \"Ünïcode: yes\"\n---\nhello\nworld\n"
        );
        let (m2, body) = loads(&text).unwrap();
        assert_eq!(m2, meta);
        assert_eq!(body, "hello\nworld\n");
    }

    #[test]
    fn plain_yaml_scalars() {
        let (m, _) = loads("---\ntitle: hand written\nn: 3\n---\nx").unwrap();
        assert_eq!(m["title"], json!("hand written"));
        assert_eq!(m["n"], json!(3));
    }
}
