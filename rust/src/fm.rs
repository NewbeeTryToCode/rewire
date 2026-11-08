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
    // git on Windows checks text files out with CRLF (core.autocrlf=true): a note that only differs in its line
    // endings must not become invisible to recall, so read it as if it were written here.
    let converted;
    let text = if text.starts_with("---\r\n") {
        converted = text.replace("\r\n", "\n");
        converted.as_str()
    } else {
        text
    };
    let Some(rest) = text.strip_prefix("---\n") else {
        return Ok((Meta::new(), text.to_string()));
    };
    let Some(end) = rest.find("\n---") else {
        return Ok((Meta::new(), text.to_string()));
    };
    let block = &rest[..end];
    let body = rest[end + 4..].trim_start_matches('\n').to_string();
    let mut meta = Meta::new();
    // Obsidian writes a list property as a block under its key ("keywords:" then "  - deploy"): those items belong to
    // the key above them. Anything else without a ":" is not frontmatter this parser knows: say so, never drop it.
    let mut list_key: Option<String> = None;
    for line in block.lines() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        let item = (line.starts_with(' ') || line.starts_with('\t')).then(|| line.trim_start().strip_prefix("- ")).flatten();
        if let Some(item) = item {
            let Some(key) = list_key.clone() else {
                return Err(format!("invalid frontmatter line: {line:?}"));
            };
            let value = scalar(item.trim());
            let entry = meta.entry(key).or_insert(Value::Null);
            match entry {
                Value::Array(items) => items.push(value),
                Value::Null => *entry = Value::Array(vec![value]),
                _ => return Err(format!("invalid frontmatter line: {line:?} (a list item under a single value)")),
            }
            continue;
        }
        let Some((key, raw)) = line.split_once(':') else {
            return Err(format!("invalid frontmatter line: {line:?}"));
        };
        let key = key.trim().to_string();
        meta.insert(key.clone(), scalar(raw.trim()));
        list_key = Some(key);
    }
    Ok((meta, body))
}

/// A frontmatter value: JSON where it parses (what `dumps` writes), plain text otherwise (what a person writes).
fn scalar(raw: &str) -> Value {
    if raw.is_empty() {
        Value::Null
    } else {
        serde_json::from_str(raw).unwrap_or_else(|_| Value::String(raw.to_string()))
    }
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
