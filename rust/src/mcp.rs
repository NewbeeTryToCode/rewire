//! Minimal MCP server over stdio (newline-delimited JSON-RPC 2.0) with three tools:
//! recall, get, remember (L0 ← L1/L2 on demand).

use crate::brain::Brain;
use crate::index::Index;
use crate::util::ulid;
use crate::{ops, pyjson, Error, Result, VERSION};
use serde_json::{json, Value};
use std::io::{BufRead, Write};

pub const PROTOCOL: &str = "2025-06-18";

/// Recall is keyword search over memories written in the user's language(s); the calling agent is an
/// LLM and widens the query far better than a stemmer can.
pub const QUERY_GUIDE: &str = "Write the query as keywords, not a sentence: the key terms plus their synonyms, in the language the user writes in and in English (memories are written in the user's language, keywords in both).";

pub const REFERS_TO_GUIDE: &str = "When the question is about a time ('last week', 'in March', 'yesterday'), resolve it to ISO dates {\"from\": \"YYYY-MM-DD\", \"to\": \"YYYY-MM-DD\"}; when a word is ambiguous, cover both readings. Notes and excerpts from that period rank higher (nothing is filtered out).";

pub const ALSO_GUIDE: &str = "Up to 4 extra keyword queries (3-8 words each), searched too and merged by rank. Add them when the answer is spread over several memories or hinges on time: counts, totals, lists, orderings, 'when' / 'how long ago' questions. Write what those messages would say: specific instances of a category ('kitchen appliance' -> 'smoker grill air fryer') and one query per counted item ('how many doctors' -> 'dermatologist appointment', 'ENT specialist visit'). Leave them out for preference, advice or recommendation questions: broad extra queries pull in the user's older preferences. A time mentioned in passing ('on my trip last weekend, I ...') is not a time question.";

pub fn tools() -> Value {
    json!([
        {
            "name": "recall",
            "description": "Search the user's long-term memory (preferences, decisions, facts, project context) before re-deriving something you may already know. Returns short summaries with ids; call `get` for full text.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": format!("What you want to remember. {QUERY_GUIDE}")},
                    "k": {"type": "integer", "minimum": 1, "maximum": 50, "default": 8},
                    "episodes": {"type": "integer", "minimum": 0, "maximum": 30, "default": 3,
                                 "description": "Also return this many raw excerpts from past session transcripts. Use ~20 when the answer spans sessions: counting ('how many times'), totals, timelines and 'how long between' questions (measured: experiments/longmemeval)."},
                    "scope": {"type": "string", "description": "'global', 'project:<slug>' or 'all'. Default: global + the project of the current directory."},
                    "refers_to": {"type": "object", "description": REFERS_TO_GUIDE,
                                  "properties": {"from": {"type": "string"}, "to": {"type": "string"}},
                                  "additionalProperties": false},
                    "also": {"type": "array", "items": {"type": "string"}, "maxItems": 4, "description": ALSO_GUIDE},
                },
                "required": ["query"],
                "additionalProperties": false,
            },
        },
        {
            "name": "get",
            "description": "Read memory notes in full by id (from `recall` or the session snapshot), or a skill as `skill:<name>`.",
            "inputSchema": {
                "type": "object",
                "properties": {"ids": {"type": "array", "items": {"type": "string"}, "maxItems": 10}},
                "required": ["ids"],
                "additionalProperties": false,
            },
        },
        {
            "name": "remember",
            "description": "Propose a new memory: a decision, preference, rule or fact worth keeping across sessions. One self-contained claim per call. It is queued for review, not applied immediately.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "text": {"type": "string", "description": "One self-contained claim (max 2 sentences)"},
                    "type": {"type": "string", "enum": ["constraint", "preference", "fact", "decision", "person", "concept"]},
                    "scope": {"type": "string", "description": "'global' (true in every project) or 'project:<slug>'. Default: the project of the current directory, else global."},
                },
                "required": ["text"],
                "additionalProperties": false,
            },
        },
    ])
}

pub struct Server<'a> {
    brain: &'a Brain,
    index: Index,
    cwd: String,
    session: String,
}

fn int_arg(args: &Value, key: &str, default: i64) -> Result<i64> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(default),
        Some(v) => v
            .as_i64()
            .or_else(|| v.as_f64().map(|f| f as i64))
            .or_else(|| v.as_str().and_then(|s| s.trim().parse().ok()))
            .ok_or_else(|| Error::Op(format!("invalid literal for int(): {v}"))),
    }
}

impl<'a> Server<'a> {
    pub fn new(brain: &'a Brain) -> Server<'a> {
        Server {
            brain,
            index: Index::new(brain),
            cwd: std::env::current_dir().map(|p| p.to_string_lossy().into_owned()).unwrap_or_default(),
            session: format!("mcp-{}", ulid()),
        }
    }

    fn call(&mut self, name: &str, args: &Value) -> Result<Value> {
        match name {
            "recall" => {
                let query = args.get("query").and_then(Value::as_str).ok_or_else(|| Error::Op("missing argument: query".into()))?;
                self.index.refresh()?;
                let k = int_arg(args, "k", 8)?.max(1) as usize;
                let episodes = int_arg(args, "episodes", 3)?.clamp(0, 30) as usize;
                let scope = args.get("scope").and_then(Value::as_str);
                let also: Vec<String> = match args.get("also") {
                    None | Some(Value::Null) => Vec::new(),
                    Some(Value::Array(a)) => a.iter().filter_map(|q| q.as_str().map(String::from)).collect(),
                    Some(_) => return Err(Error::Op("also must be a list of queries".into())),
                };
                let hits = ops::recall(
                    self.brain,
                    &mut self.index,
                    query,
                    k,
                    Some(&self.cwd),
                    scope,
                    None,
                    episodes,
                    args.get("refers_to"),
                    &also,
                )?;
                Ok(Value::Array(hits.iter().map(|h| h.to_json()).collect()))
            }
            "get" => {
                let ids: Vec<String> = args
                    .get("ids")
                    .and_then(Value::as_array)
                    .ok_or_else(|| Error::Op("missing argument: ids".into()))?
                    .iter()
                    .map(|v| v.as_str().map(String::from).unwrap_or_else(|| v.to_string()))
                    .collect();
                Ok(Value::Array(ops::get(self.brain, &ids, &self.session, "mcp")?))
            }
            "remember" => {
                let text = args.get("text").and_then(Value::as_str).ok_or_else(|| Error::Op("missing argument: text".into()))?;
                let inbox_id = ops::remember(
                    self.brain,
                    text,
                    args.get("type").and_then(Value::as_str).unwrap_or("fact"),
                    &args
                        .get("scope")
                        .and_then(Value::as_str)
                        .map(String::from)
                        .unwrap_or_else(|| ops::default_scope(self.brain, Some(&self.cwd))),
                    "assistant",
                    Some(json!({"harness": "mcp", "session": self.session})),
                )?;
                Ok(json!({"queued": inbox_id, "note": "queued for the user's review"}))
            }
            other => Err(Error::Op(format!("unknown tool {other}"))),
        }
    }

    pub fn handle(&mut self, msg: &Value) -> Option<Value> {
        let mid = msg.get("id").filter(|v| !v.is_null())?.clone(); // notifications get no reply
        let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
        let params = msg.get("params").cloned().unwrap_or(Value::Null);
        let result = match method {
            "initialize" => {
                let version = params.get("protocolVersion").and_then(Value::as_str).filter(|v| !v.is_empty()).unwrap_or(PROTOCOL);
                json!({"protocolVersion": version, "capabilities": {"tools": {}},
                       "serverInfo": {"name": "rewire", "version": VERSION}})
            }
            "ping" => json!({}),
            "tools/list" => json!({"tools": tools()}),
            "tools/call" => {
                let name = params.get("name").and_then(Value::as_str).unwrap_or("");
                let args = params.get("arguments").cloned().filter(|a| a.is_object()).unwrap_or_else(|| json!({}));
                match self.call(name, &args) {
                    Ok(data) => {
                        let mut text = pyjson::dumps_utf8(&data);
                        if name == "recall" || name == "get" {
                            // memory as data, dropped again from transcripts
                            text = format!(
                                "{}\n{}\n{}",
                                crate::snapshot::FENCE_OPEN,
                                crate::snapshot::printable(&crate::snapshot::escape(&text)),
                                crate::snapshot::FENCE_CLOSE
                            );
                        }
                        if name == "recall" {
                            let query = args.get("query").and_then(Value::as_str).unwrap_or("");
                            if ops::weak_match(query, &data) {
                                text.push_str(
                                    "\nweak match: no result contains the question's key words. The memory may not hold \
                                               the answer: say so, or ask the user, rather than guessing from these results.",
                                );
                            }
                        }
                        json!({"content": [{"type": "text", "text": text}]})
                    }
                    Err(Error::Op(e)) | Err(Error::Schema(e)) => {
                        json!({"content": [{"type": "text", "text": format!("error: {e}")}], "isError": true})
                    }
                    Err(e) => return Some(json!({"jsonrpc": "2.0", "id": mid, "error": {"code": -32603, "message": e.to_string()}})),
                }
            }
            other => {
                return Some(json!({"jsonrpc": "2.0", "id": mid, "error": {"code": -32601, "message": format!("unknown method {other}")}}))
            }
        };
        Some(json!({"jsonrpc": "2.0", "id": mid, "result": result}))
    }
}

pub fn serve(brain: &Brain) -> Result<()> {
    let mut server = Server::new(brain);
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(line) {
            Ok(msg) => server.handle(&msg),
            Err(_) => Some(json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": "parse error"}})),
        };
        if let Some(r) = reply {
            writeln!(stdout, "{}", pyjson::dumps_utf8(&r))?;
            stdout.flush()?;
        }
    }
    Ok(())
}
