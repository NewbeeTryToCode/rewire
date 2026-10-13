"""Harness transcript parsers -> one shared turn format, plus discovery.

The only harness-specific code in rewire. A turn is:
  {"n": <line number>, "role": "user"|"assistant"|"tool", "text": str, "ts": str}
"""

import glob
import json
import os
import re

INTERNAL_MARKER = "[rewire-internal]"
TOOL_CHARS = 300
_TAGS = re.compile(r"<(system-reminder|command-[a-z-]+|local-command-[a-z-]+)>.*?</\1>", re.S)
_INJECTED = re.compile(r"<rewire-memory>.*?(?:</rewire-memory>|\Z)", re.S)


def strip_injected(text: str) -> str:
    """Drop memory rewire itself injected (snapshot, recall results), wherever it shows up in a
    transcript: it must never be re-extracted as something the user said."""
    return _INJECTED.sub("", text)


def _clean(text: str) -> str:
    return _TAGS.sub("", strip_injected(text)).strip()


def _blocks_text(content, kinds=("text",)) -> str:
    if isinstance(content, str):
        return content
    parts = []
    for block in content or []:
        if isinstance(block, dict) and block.get("type") in kinds:
            parts.append(block.get("text") or "")
    return "\n".join(parts)


def _tool_text(content) -> str:
    out = []
    for block in content or []:
        if isinstance(block, dict) and block.get("type") == "tool_result":
            inner = block.get("content")
            out.append(inner if isinstance(inner, str) else _blocks_text(inner))
    text = strip_injected("\n".join(out)).strip()
    return text[:TOOL_CHARS] + (" …" if len(text) > TOOL_CHARS else "")


def parse_claude_code(lines) -> tuple[dict, list[dict]]:
    meta, turns = {}, []
    for n, line in lines:
        try:
            d = json.loads(line)
        except json.JSONDecodeError:
            continue
        if d.get("sessionId") and "session" not in meta:
            meta["session"] = d["sessionId"]
        if d.get("cwd") and "cwd" not in meta:
            meta["cwd"] = d["cwd"]
        if d.get("isSidechain") or d.get("isMeta"):
            continue
        msg, kind = d.get("message") or {}, d.get("type")
        content = msg.get("content")
        if kind == "user":
            if isinstance(content, list) and any(isinstance(b, dict) and b.get("type") == "tool_result" for b in content):
                text, role = _tool_text(content), "tool"
            else:
                text, role = _clean(_blocks_text(content)), "user"
        elif kind == "assistant":
            text, role = strip_injected(_blocks_text(content)).strip(), "assistant"
        else:
            continue
        if text:
            turns.append({"n": n, "role": role, "text": text, "ts": d.get("timestamp", "")})
    return meta, turns


def parse_codex(lines) -> tuple[dict, list[dict]]:
    """Codex rollout JSONL: {"timestamp", "type", "payload"} per line.

    Format details come from third-party docs; the parser is lenient and
    only reads message-shaped response items (verify against real rollouts).
    """
    meta, turns = {}, []
    for n, line in lines:
        try:
            d = json.loads(line)
        except json.JSONDecodeError:
            continue
        kind, p = d.get("type"), d.get("payload") or {}
        if kind == "session_meta":
            meta["session"] = p.get("id") or meta.get("session")
            meta["cwd"] = p.get("cwd") or meta.get("cwd")
            continue
        if kind != "response_item":
            continue
        if p.get("type") == "message" and p.get("role") in ("user", "assistant"):
            kinds = ("input_text", "output_text", "text")
            text = _clean(_blocks_text(p.get("content"), kinds))
            if p["role"] == "user" and (text.startswith("<environment_context>") or text.startswith("<user_instructions>")):
                continue
            role = p["role"]
        elif p.get("type") in ("function_call_output", "custom_tool_call_output"):
            out = p.get("output")
            text = out if isinstance(out, str) else json.dumps(out)
            text, role = strip_injected(text).strip()[:TOOL_CHARS], "tool"
        else:
            continue
        if text:
            turns.append({"n": n, "role": role, "text": text, "ts": d.get("timestamp", "")})
    return meta, turns


PARSERS = {"claude-code": parse_claude_code, "codex": parse_codex}


def guess_harness(path: str) -> str:
    return "codex" if os.path.basename(path).startswith("rollout-") else "claude-code"


def read(path: str, start_line: int = 0, harness: str | None = None):
    """Parse lines after start_line. Returns (harness, meta, turns, last_line)."""
    harness = harness or guess_harness(path)
    with open(path, encoding="utf-8", errors="replace") as fh:
        lines = [(n, line) for n, line in enumerate(fh, 1)]
    head_meta, _ = PARSERS[harness](lines[:50])  # session/cwd live near the top
    meta, turns = PARSERS[harness]([(n, l) for n, l in lines if n > start_line])
    meta = {**head_meta, **{k: v for k, v in meta.items() if v}}
    return harness, meta, turns, (lines[-1][0] if lines else start_line)


def is_internal(meta: dict, turns: list[dict], brain_root: str) -> bool:
    cwd = os.path.abspath(meta.get("cwd") or "/")
    if cwd == brain_root or cwd.startswith(brain_root + os.sep):
        return True
    first_user = next((t["text"] for t in turns if t["role"] == "user"), "")
    return first_user.startswith(INTERNAL_MARKER)


def discover(since_ts: float) -> list[str]:
    claude_dir = os.path.expanduser(os.environ.get("CLAUDE_CONFIG_DIR", "~/.claude"))
    codex_dir = os.path.expanduser(os.environ.get("CODEX_HOME", "~/.codex"))
    paths = glob.glob(os.path.join(claude_dir, "projects", "*", "*.jsonl"))
    paths += glob.glob(os.path.join(codex_dir, "sessions", "**", "rollout-*.jsonl"), recursive=True)
    return sorted(p for p in paths if os.path.getmtime(p) >= since_ts)
