"""Latency benchmark for the hot paths.

Builds synthetic brains (N nodes, ~10 use events per node) and times:
  hook session-start (end-to-end CLI, what a harness waits for), prime,
  recall (CLI and warm MCP server), get, transcript re-scan, index rebuild.

Run: python experiments/perf/bench.py [sizes...]   (default: 500 2000 10000)
"""

import json
import os
import random
import shutil
import statistics
import subprocess
import sys
import tempfile
import time

ROOT = os.path.join(os.path.dirname(__file__), "..", "..")
sys.path.insert(0, os.path.join(ROOT, "src"))

from rewire import transcripts as tx  # noqa: E402
from rewire.brain import Brain  # noqa: E402
from rewire.index import Index  # noqa: E402
from rewire.mcp import Server  # noqa: E402

WORDS = ("memory graph sqlite python deploy review rust api cache test docker auth billing search "
         "index event session agent harness portable brain sync budget token project design").split()
TYPES = ["fact"] * 5 + ["decision"] * 2 + ["preference", "concept", "constraint"]


def build(n: int, root: str) -> Brain:
    rng = random.Random(n)
    brain = Brain.init(root, device="bench")
    events = []
    for i in range(n):
        t = TYPES[i % len(TYPES)]
        words = " ".join(rng.choice(WORDS) for _ in range(12))
        scope = "global" if i % 3 else f"project:p{i % 20}"
        meta = brain.new_meta(t, f"{t} {i} {words[:60]}", scope=scope, importance=rng.randint(1, 5),
                              confidence="explicit", provenance="user")
        brain.write_node(meta, f"{words}. Detail number {i}.")
        for _ in range(10):
            day = rng.randint(1, 28)
            events.append({"ev": "use", "node": meta["id"], "w": 1.0, "session": f"s{rng.randint(0, 5000)}",
                           "t": f"2026-09-{day:02d}T10:00:00Z"})
    brain.append_events(events)
    brain.commit("bench data")
    return brain


def timed(fn, repeat=5):
    samples = []
    for _ in range(repeat):
        t0 = time.perf_counter()
        fn()
        samples.append((time.perf_counter() - t0) * 1000)
    return statistics.median(samples)


def cli(brain, *args, stdin=None):
    env = {**os.environ, "REWIRE_BRAIN": brain.root, "PYTHONPATH": os.path.join(ROOT, "src"),
           "REWIRE_INTERNAL": "0", "CLAUDE_CONFIG_DIR": brain.path(".state", "none"),
           "CODEX_HOME": brain.path(".state", "none")}
    r = subprocess.run([sys.executable, "-m", "rewire", *args], input=stdin, env=env,
                       capture_output=True, text=True)
    assert r.returncode == 0, r.stderr


def make_transcript(path: str, turns: int) -> None:
    with open(path, "w") as fh:
        for i in range(turns):
            role = "user" if i % 2 == 0 else "assistant"
            content = "please keep going " * 20 if role == "user" else [{"type": "text", "text": "ok " * 200}]
            fh.write(json.dumps({"type": role, "sessionId": "s", "cwd": "/w",
                                 "message": {"role": role, "content": content}}) + "\n")


def main(sizes):
    rows = []
    for n in sizes:
        tmp = tempfile.mkdtemp()
        try:
            t0 = time.perf_counter()
            brain = build(n, os.path.join(tmp, "brain"))
            build_s = time.perf_counter() - t0
            index = Index(brain)
            ids = [node.id for node in list(brain.iter_nodes())[:3]]
            server = Server(brain)
            server.call("recall", {"query": "warmup"})
            hook_in = json.dumps({"cwd": tmp})
            transcript = os.path.join(tmp, "t.jsonl")
            make_transcript(transcript, 4000)
            r = {
                "nodes": n,
                "rebuild": timed(index.rebuild, 3),
                "hook session-start (CLI)": timed(lambda: cli(brain, "hook", "session-start", stdin=hook_in), 3),
                "prime (CLI)": timed(lambda: cli(brain, "prime", "--project", "p1"), 3),
                "recall (CLI)": timed(lambda: cli(brain, "recall", "sqlite", "cache"), 3),
                "recall (MCP, warm)": timed(lambda: server.call("recall", {"query": "sqlite cache"})),
                "get by short id (MCP)": timed(lambda: server.call("get", {"ids": [i[-8:].lower() for i in ids]})),
                "transcript re-scan, no new lines (4k turns)": timed(
                    lambda: tx.read(transcript, start_line=4000), 3),
            }
            rows.append(r)
            print(f"built {n} nodes in {build_s:.1f}s", file=sys.stderr)
        finally:
            shutil.rmtree(tmp)
    keys = [k for k in rows[0] if k != "nodes"]
    print("| operation (median ms) | " + " | ".join(f"{r['nodes']} nodes" for r in rows) + " |")
    print("|---|" + "---|" * len(rows))
    for k in keys:
        print(f"| {k} | " + " | ".join(f"{r[k]:.0f}" for r in rows) + " |")


if __name__ == "__main__":
    main([int(a) for a in sys.argv[1:]] or [500, 2000, 10000])
