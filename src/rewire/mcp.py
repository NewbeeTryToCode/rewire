"""Minimal MCP server over stdio (newline-delimited JSON-RPC 2.0).

Three tools, deliberately small (docs/design/mvp-spec.md §5.1):
recall, get, remember.
"""

import json
import os
import sys

from . import __version__, ops
from .brain import Brain
from .index import Index
from .snapshot import FENCE_CLOSE, FENCE_OPEN, escape
from .util import ulid

PROTOCOL = "2025-06-18"

# Recall is keyword search over memories written in the user's language(s); the
# calling agent is an LLM and widens the query far better than a stemmer can.
QUERY_GUIDE = (
    "Write the query as keywords, not a sentence: the key terms plus their synonyms, and the same "
    "terms in both English and Indonesian (memories may be written in either language)."
)

REFERS_TO_GUIDE = (
    "When the question is about a time ('last week', 'in March', 'kemarin'), resolve it to ISO dates "
    "{\"from\": \"YYYY-MM-DD\", \"to\": \"YYYY-MM-DD\"}; when a word is ambiguous, cover both readings. "
    "Notes and excerpts from that period rank higher (nothing is filtered out)."
)

TOOLS = [
    {
        "name": "recall",
        "description": (
            "Search the user's long-term memory (preferences, decisions, facts, project context) "
            "before re-deriving something you may already know. Returns short summaries with ids; "
            "call `get` for full text."
        ),
        "inputSchema": {
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "What you want to remember. " + QUERY_GUIDE},
                "k": {"type": "integer", "minimum": 1, "maximum": 50, "default": 8},
                "episodes": {"type": "integer", "minimum": 0, "maximum": 30, "default": 3,
                             "description": "Also return this many raw excerpts from past session transcripts. "
                             "Use ~20 when the answer spans sessions: counting ('how many times'), totals, "
                             "timelines and 'how long between' questions (measured: experiments/longmemeval)."},
                "scope": {"type": "string", "description": "'global', 'project:<slug>' or 'all'. "
                          "Default: global + the project of the current directory."},
                "refers_to": {"type": "object", "description": REFERS_TO_GUIDE,
                              "properties": {"from": {"type": "string"}, "to": {"type": "string"}},
                              "additionalProperties": False},
            },
            "required": ["query"],
            "additionalProperties": False,
        },
    },
    {
        "name": "get",
        "description": "Read memory notes in full by id (from `recall` or the session snapshot).",
        "inputSchema": {
            "type": "object",
            "properties": {"ids": {"type": "array", "items": {"type": "string"}, "maxItems": 10}},
            "required": ["ids"],
            "additionalProperties": False,
        },
    },
    {
        "name": "remember",
        "description": (
            "Propose a new memory: a decision, preference, rule or fact worth keeping across sessions. "
            "One self-contained claim per call. It is queued for review, not applied immediately."
        ),
        "inputSchema": {
            "type": "object",
            "properties": {
                "text": {"type": "string", "description": "One self-contained claim (max 2 sentences)"},
                "type": {"type": "string", "enum": ["constraint", "preference", "fact", "decision",
                                                    "person", "concept"]},
                "scope": {"type": "string", "description": "'global' or 'project:<slug>'"},
            },
            "required": ["text"],
            "additionalProperties": False,
        },
    },
]


class Server:
    def __init__(self, brain: Brain, cwd: str | None = None):
        self.brain = brain
        self.index = Index(brain)
        self.cwd = cwd or os.getcwd()
        self.session = f"mcp-{ulid()}"

    def call(self, name: str, args: dict):
        if name == "recall":
            self.index.refresh()
            return ops.recall(self.brain, self.index, args["query"], k=int(args.get("k", 8)),
                              cwd=self.cwd, scope=args.get("scope"), episodes=max(0, min(int(args.get("episodes", 3)), 30)),
                              refers_to=args.get("refers_to"))
        if name == "get":
            return ops.get(self.brain, list(args["ids"]), session=self.session)
        if name == "remember":
            inbox_id = ops.remember(self.brain, args["text"], type_=args.get("type", "fact"),
                                    scope=args.get("scope", "global"), provenance="assistant",
                                    source={"harness": "mcp", "session": self.session})
            return {"queued": inbox_id, "note": "queued for the user's review"}
        raise ops.OpError(f"unknown tool {name}")

    def handle(self, msg: dict) -> dict | None:
        method, mid = msg.get("method"), msg.get("id")
        if mid is None:
            return None  # notification
        try:
            if method == "initialize":
                version = (msg.get("params") or {}).get("protocolVersion") or PROTOCOL
                result = {"protocolVersion": version, "capabilities": {"tools": {}},
                          "serverInfo": {"name": "rewire", "version": __version__}}
            elif method == "ping":
                result = {}
            elif method == "tools/list":
                result = {"tools": TOOLS}
            elif method == "tools/call":
                params = msg.get("params") or {}
                try:
                    data = self.call(params.get("name"), params.get("arguments") or {})
                    text = json.dumps(data, ensure_ascii=False)
                    if params.get("name") in ("recall", "get"):  # memory as data, dropped again from transcripts
                        text = f"{FENCE_OPEN}\n{escape(text)}\n{FENCE_CLOSE}"
                    result = {"content": [{"type": "text", "text": text}]}
                except (ops.OpError, KeyError, ValueError) as exc:
                    result = {"content": [{"type": "text", "text": f"error: {exc}"}], "isError": True}
            else:
                return {"jsonrpc": "2.0", "id": mid, "error": {"code": -32601, "message": f"unknown method {method}"}}
            return {"jsonrpc": "2.0", "id": mid, "result": result}
        except Exception as exc:  # never crash the host harness
            return {"jsonrpc": "2.0", "id": mid, "error": {"code": -32603, "message": str(exc)}}


def serve(brain: Brain, stdin=None, stdout=None) -> None:
    stdin, stdout = stdin or sys.stdin, stdout or sys.stdout
    server = Server(brain)
    for line in stdin:
        line = line.strip()
        if not line:
            continue
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            reply = {"jsonrpc": "2.0", "id": None, "error": {"code": -32700, "message": "parse error"}}
        else:
            reply = server.handle(msg)
        if reply is not None:
            stdout.write(json.dumps(reply, ensure_ascii=False) + "\n")
            stdout.flush()
