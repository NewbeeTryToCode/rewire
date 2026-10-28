# Critical path performance

> **Historical.** This experiment ran on rewire's earlier Python implementation, which has been removed; its script is in git history (before the commit that removed `src/`). Current benchmarks run through [`experiments/eval`](../eval) on the Rust binary.

Recall latency of the Rust binary over MCP on benchmark brains: [`../eval/latency.py`](../eval/latency.py).

A synthetic brain with N nodes and ~10 usage events per node (10,000 nodes = 100,000 events). Medians in milliseconds, measured in a cloud container (Python 3.13). CLI numbers include ~70 ms of Python startup.

| Operation | 500 nodes (before → after) | 2,000 nodes | 10,000 nodes |
|---|---|---|---|
| **session-start hook (CLI)** — what the harness waits for | 197 → **77** | 359 → **83** | 1,240 → **168** |
| prime (CLI) | 209 → **72** | 266 → **84** | 1,099 → **140** |
| recall (CLI) | 190 → **87** | 268 → **81** | 999 → **95** |
| recall (MCP, warm server) | 94 → **6** | 188 → **8** | 908 → **34** |
| get with a short ID (MCP) | 61 → **3** | 176 → **5** | 910 → **25** |
| full rebuild (once, at init/clone) | 48 → 76 | 177 → 281 | 907 → 1,300–1,600 |
| consolidation excluding LLM time | — | — | 385 (500 nodes: 140) |

## What was changed

1. **Incremental index** (the main cause). Previously every `prime`/`recall`/hook rebuilt the index from scratch. Now only node files whose mtime/size changed are reread, only new bytes in the event log are processed (offset + verification of the last 64 bytes; a rewritten file, e.g. by `git revert`, triggers a full rebuild), and activation is recomputed only for touched nodes (in full every 5 minutes, because activation changes on the scale of hours).
2. **Skip the directory scan** when the folder mtime has not changed and the last full scan was < 60 seconds ago (rewire always writes with an atomic rename, which changes the folder mtime).
3. **Short ID → file name glob** (`*-<8 characters>.md`), instead of parsing all nodes.
4. **Frontmatter is parsed once per file** (not one `json.loads` per line); events are parsed from strings, not bytes, and inserted with `executemany`.
5. **Consolidation without a full scan**: project hubs are looked up in `nodes/project/` only; used memory IDs are resolved one by one.
6. `importlib.resources` is imported only on `install` (−16 ms startup).

## Trade-offs

- A full rebuild is ~1.6× slower than before (WAL + upsert per event). It happens once per machine; this was chosen because all daily operations became 3–30× faster.
- Manual *in-place* edits to a node file (without a rename) may take up to 60 seconds to become visible. Ordinary editors and rewire itself write with a rename, so those are visible immediately.
- The dominant cost of consolidation is still the LLM call (~10–17 seconds per session, running in the background after a session ends).
