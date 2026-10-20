"""The rewire binary as the eval harness drives it: init, import sessions, approve, recall.

rewire is implemented in Rust only; the harness never imports it. Build it first:
    cargo build --release --manifest-path rust/Cargo.toml
or point REWIRE_BIN at another build.
"""

import glob
import json
import os
import re
import statistics
import subprocess
import tempfile
import time

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.normpath(os.path.join(HERE, "..", ".."))
BIN = os.environ.get("REWIRE_BIN") or os.path.join(ROOT, "rust", "target", "release", "rewire")


def check() -> None:
    if not os.access(BIN, os.X_OK):
        raise SystemExit(f"rewire binary not found at {BIN}: cargo build --release --manifest-path rust/Cargo.toml "
                         "(or set REWIRE_BIN)")


def run(root: str, *args: str, stdin: str | None = None, now: str | None = None) -> str:
    env = dict(os.environ)
    if now:  # recall ranks by activation, which decays with age: pin the clock (see frozen_now in run.py)
        env["REWIRE_NOW"] = now
    out = subprocess.run([BIN, "--brain", root, *args], input=stdin, capture_output=True, text=True, env=env)
    if out.returncode:
        raise RuntimeError(f"rewire {args[0]} failed ({out.returncode}): {(out.stderr or out.stdout).strip()[-500:]}")
    return out.stdout


def version() -> str:
    return subprocess.run([BIN, "--version"], capture_output=True, text=True).stdout.strip()


def init(root: str, profile: str = "general", command: str | None = None) -> None:
    """A new brain for one benchmark history. Profile "general" keeps every role in the transcript.
    `command` is the consolidation command for --mode full (reads the prompt on stdin, prints the reply)."""
    if os.path.exists(os.path.join(root, "brain.toml")):
        return
    os.makedirs(os.path.dirname(root) or ".", exist_ok=True)
    run(root, "init", "--device", "eval")
    path = os.path.join(root, "brain.toml")
    text = open(path, encoding="utf-8").read().replace('profile = "coding"', f'profile = "{profile}"')
    if command:
        text = re.sub(r'(?m)^provider = .*$', 'provider = "command"', text)
        text = re.sub(r'(?m)^command = .*$', lambda _: "command = " + json.dumps(command), text)
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(text)


def import_sessions(root: str, sessions: list[dict], llm: bool = False) -> list[dict]:
    """Sessions {harness, session, date, turns: [{n, role, text, ts}]} -> one record per session (ok, seconds,
    llm_calls, prompt_chars, completion_chars, and with llm: applied / queued / rejected). Stops at a failure."""
    if not sessions:
        return []
    lines = "\n".join(json.dumps(s, ensure_ascii=False) for s in sessions) + "\n"
    env = dict(os.environ)
    out = subprocess.run([BIN, "--brain", root, "import", "-", *(["--llm"] if llm else [])], input=lines,
                         capture_output=True, text=True, env=env)
    records = [json.loads(line) for line in out.stdout.splitlines() if line.strip().startswith("{")]
    if out.returncode and not records:
        raise RuntimeError(f"rewire import failed: {out.stderr.strip()[-500:]}")
    return records


def approve_all(root: str) -> None:
    """No human reviewer in a benchmark: inbox proposals are accepted in bulk, as a user would."""
    inbox = glob.glob(os.path.join(root, "inbox", "*.md"))
    if inbox:
        run(root, "review", "approve", "all")


def recall(root: str, query: str, k: int, episodes: int, also: list[str] | None = None,
           now: str | None = None) -> tuple[list[dict], float]:
    """(hits, wall ms). Memory hits carry their whole body, valid_from and source (`--full`)."""
    args = ["recall", "--json", "--full", "-k", str(k), "--episodes", str(episodes), "--scope", "all"]
    for q in (also or [])[:4]:
        args += ["--also", q]
    start = time.perf_counter()
    out = run(root, *args, "--", query, now=now)
    return json.loads(out), (time.perf_counter() - start) * 1000


def warm_up(root: str, now: str | None = None) -> None:
    """Build or refresh the index outside the timed recall."""
    run(root, "recall", "--json", "-k", "1", "--episodes", "1", "--scope", "all", "--", "index warm up", now=now)


def process_overhead(runs: int = 7) -> float:
    """Median wall ms of `rewire recall` on an empty brain: process start plus model load, the part of a CLI
    recall that a long-running MCP server pays once. Subtracted to report net recall latency."""
    with tempfile.TemporaryDirectory() as tmp:
        root = os.path.join(tmp, "b")
        init(root)
        warm_up(root)
        times = [recall(root, "overhead probe", 8, 10)[1] for _ in range(runs)]
    return statistics.median(times)


def slugify(text: str, max_len: int = 48) -> str:
    """rewire's slug rule (session file names)."""
    slug = re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-")
    return slug[:max_len].rstrip("-") or "node"


def active_node_types(root: str) -> dict[str, int]:
    """Active memory nodes by type (nodes/<type>/*.md, not retracted, not superseded)."""
    counts: dict[str, int] = {}
    for path in glob.glob(os.path.join(root, "nodes", "*", "*.md")):
        head = open(path, encoding="utf-8").read(4000).split("\n---", 1)[0]
        if re.search(r"(?m)^(retracted_at|valid_to): \S", head) and not re.search(r"(?m)^(retracted_at|valid_to): (null|~)\s*$", head):
            continue
        t = os.path.basename(os.path.dirname(path))
        counts[t] = counts.get(t, 0) + 1
    return dict(sorted(counts.items(), key=lambda kv: -kv[1]))
