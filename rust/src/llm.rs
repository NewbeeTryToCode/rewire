//! LLM providers for consolidation (the only place rewire spends tokens). Each returns a JSON object.
//!
//! - command: any CLI that reads a prompt on stdin and prints text (e.g. `claude -p`). Runs with
//!   REWIRE_INTERNAL=1 inside the brain's .state/ dir so hooks skip it and its own transcript is
//!   never consolidated (no recursion).
//! - anthropic: the Anthropic Messages API with structured outputs (ANTHROPIC_API_KEY).

use crate::config::Config;
use crate::transcripts::INTERNAL_MARKER;
use crate::{Error, Meta, Result};
use serde_json::{json, Value};

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

pub trait Provider {
    fn complete_json(&self, system: &str, prompt: &str, schema: &Value) -> Result<Meta>;
}

fn llm<T>(msg: impl Into<String>) -> Result<T> {
    Err(Error::Llm(msg.into()))
}

pub fn extract_json(text: &str) -> Result<Meta> {
    let mut text = text.trim().to_string();
    if text.starts_with("```") {
        text = text.trim_matches('`').to_string();
        if let Some(i) = text.find('\n') {
            text = text[i + 1..].to_string();
        }
    }
    let (Some(start), Some(end)) = (text.find('{'), text.rfind('}')) else {
        return llm(format!("no JSON object in model output: {:?}", text.chars().take(200).collect::<String>()));
    };
    if end <= start {
        return llm(format!("no JSON object in model output: {:?}", text.chars().take(200).collect::<String>()));
    }
    match serde_json::from_str::<Meta>(&text[start..=end]) {
        Ok(m) => Ok(m),
        Err(e) => {
            // an object followed by more text (or a second object): keep the first complete one
            let mut stream = serde_json::Deserializer::from_str(&text[start..]).into_iter::<Meta>();
            match stream.next() {
                Some(Ok(m)) => Ok(m),
                _ => llm(format!("invalid JSON from model: {e}")),
            }
        }
    }
}

/// LLM work done by consolidation, for cost reports (`rewire import --llm`).
#[derive(Debug, Default)]
pub struct Usage {
    pub calls: std::cell::Cell<u64>,
    pub failed: std::cell::Cell<u64>,
    pub prompt_chars: std::cell::Cell<u64>,
    pub completion_chars: std::cell::Cell<u64>,
    pub seconds: std::cell::Cell<f64>,
}

/// Wraps a provider and adds each call to `usage`.
pub struct Counting<'a> {
    pub inner: Box<dyn Provider + 'a>,
    pub usage: &'a Usage,
}

impl Provider for Counting<'_> {
    fn complete_json(&self, system: &str, prompt: &str, schema: &Value) -> Result<Meta> {
        let start = Instant::now();
        let out = self.inner.complete_json(system, prompt, schema);
        let u = self.usage;
        u.calls.set(u.calls.get() + 1);
        u.seconds.set(u.seconds.get() + start.elapsed().as_secs_f64());
        u.prompt_chars.set(u.prompt_chars.get() + (system.chars().count() + prompt.chars().count()) as u64);
        match &out {
            Ok(m) => u
                .completion_chars
                .set(u.completion_chars.get() + crate::pyjson::dumps_utf8(&Value::Object(m.clone())).chars().count() as u64),
            Err(_) => u.failed.set(u.failed.get() + 1),
        }
        out
    }
}

/// For work that must not call a model (writing transcripts only).
pub struct NoProvider;

impl Provider for NoProvider {
    fn complete_json(&self, _: &str, _: &str, _: &Value) -> Result<Meta> {
        llm("no LLM provider in this command")
    }
}

/// The provider that could not be set up (no API key, an unknown provider, an unsafe brain): every call fails with
/// the reason, so transcripts are still kept and the failure is recorded and reported.
pub struct Unavailable(pub String);

impl Provider for Unavailable {
    fn complete_json(&self, _: &str, _: &str, _: &Value) -> Result<Meta> {
        llm(self.0.clone())
    }
}

pub struct CommandProvider {
    pub argv: Vec<String>,
    pub workdir: PathBuf,
    pub timeout: Duration,
}

impl CommandProvider {
    pub fn new(command: &str, workdir: &Path) -> CommandProvider {
        CommandProvider {
            argv: shlex::split(command).unwrap_or_else(|| command.split_whitespace().map(String::from).collect()),
            workdir: workdir.to_path_buf(),
            timeout: Duration::from_secs(900),
        }
    }
}

impl Provider for CommandProvider {
    fn complete_json(&self, system: &str, prompt: &str, schema: &Value) -> Result<Meta> {
        std::fs::create_dir_all(&self.workdir)?;
        let full = format!(
            "{INTERNAL_MARKER}\n{system}\n\nRespond with ONLY a JSON object matching this JSON schema, no prose, no code fences:\n{}\n\n{prompt}",
            crate::pyjson::dumps(schema, crate::pyjson::Opts { ensure_ascii: true, sort_keys: false })
        );
        let Some((prog, args)) = self.argv.split_first() else { return llm("empty command") };
        let mut cmd = Command::new(prog);
        cmd.args(args).current_dir(&self.workdir).env("REWIRE_INTERNAL", "1");
        let out = match crate::util::run_with_timeout(cmd, Some(full), self.timeout) {
            Ok(out) => out,
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                return llm(format!("command provider failed: timed out after {}s", self.timeout.as_secs()))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return llm(format!(
                    "command provider failed: `{prog}` not found; install it or set the command (or provider = \"anthropic\") in local.toml"
                ))
            }
            Err(e) => return llm(format!("command provider failed: `{prog}`: {e}")),
        };
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr).trim().chars().take(300).collect::<String>();
            let said = if err.trim().is_empty() { "no output on stderr".to_string() } else { err.trim().to_string() };
            return llm(format!(
                "`{prog}` exited with status {} ({said}); check the [consolidation] command in local.toml",
                out.status.code().unwrap_or(-1)
            ));
        }
        extract_json(&String::from_utf8_lossy(&out.stdout))
    }
}

pub struct AnthropicProvider {
    pub model: String,
    pub effort: String,
    key: String,
}

impl AnthropicProvider {
    pub fn new(model: &str, effort: &str) -> Result<AnthropicProvider> {
        let key =
            std::env::var("ANTHROPIC_API_KEY").map_err(|_| Error::Llm("set ANTHROPIC_API_KEY to use the anthropic provider".into()))?;
        Ok(AnthropicProvider { model: model.into(), effort: effort.into(), key })
    }
}

impl Provider for AnthropicProvider {
    fn complete_json(&self, system: &str, prompt: &str, schema: &Value) -> Result<Meta> {
        let base = std::env::var("ANTHROPIC_BASE_URL").unwrap_or_else(|_| "https://api.anthropic.com".into());
        let body = json!({
            "model": self.model,
            "max_tokens": 16000,
            "system": system,
            "messages": [{"role": "user", "content": prompt}],
            "output_config": {"effort": self.effort, "format": {"type": "json_schema", "schema": schema}},
            "fallbacks": "default",
        });
        let resp = ureq::post(&format!("{}/v1/messages", base.trim_end_matches('/')))
            .set("x-api-key", &self.key)
            .set("anthropic-version", "2023-06-01")
            .set("anthropic-beta", "server-side-fallback-2026-07-01")
            .set("content-type", "application/json")
            .timeout(Duration::from_secs(900))
            .send_json(body);
        let value: Value = match resp {
            Ok(r) => r.into_json().map_err(|e| Error::Llm(format!("connection error: {e}")))?,
            Err(ureq::Error::Status(code, r)) => {
                let msg = r.into_string().unwrap_or_default();
                return llm(format!("API error {code}: {}", msg.chars().take(300).collect::<String>()));
            }
            Err(e) => return llm(format!("connection error: {e}")),
        };
        match value.get("stop_reason").and_then(Value::as_str) {
            Some("refusal") => return llm("model declined the request"),
            Some("max_tokens") => return llm("output truncated (max_tokens)"),
            _ => {}
        }
        let text = value
            .get("content")
            .and_then(Value::as_array)
            .and_then(|blocks| blocks.iter().find(|b| b.get("type").and_then(Value::as_str) == Some("text")))
            .and_then(|b| b.get("text").and_then(Value::as_str))
            .unwrap_or("");
        extract_json(text)
    }
}

pub fn make(config: &Config, brain_root: &Path) -> Result<Box<dyn Provider>> {
    match config.provider.as_str() {
        "command" => {
            // local.toml is never synced; one that git tracks may have come from another device or the remote
            let brain = crate::brain::Brain::open(Some(brain_root));
            let found = crate::brain::unsafe_paths(&brain, None);
            if !found.is_empty() {
                return llm(format!(
                    "git tracks {} in the brain, so what runs may come from another device: not running the LLM command \
                     (see `rewire doctor`)",
                    found.join(", ")
                ));
            }
            Ok(Box::new(CommandProvider::new(&config.command, &brain_root.join(".state").join("llm"))))
        }
        "anthropic" => Ok(Box::new(AnthropicProvider::new(&config.model, &config.effort)?)),
        other => llm(format!("unknown provider {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts() {
        assert_eq!(extract_json("```json\n{\"a\": 1}\n```").unwrap()["a"], 1);
        assert_eq!(extract_json("{\"a\": 1}\n\n{\"b\": 2}").unwrap()["a"], 1);
        assert_eq!(extract_json("sure: {\"a\": {\"x\": 1}} done").unwrap()["a"]["x"], 1);
        assert!(extract_json("nothing").is_err());
    }
}
