# LoCoMo: trimming transcript excerpts and speeding up the hot path

Date 2026-10-14 · LoCoMo 10 conversations, 1,540 questions in categories 1–4 · mode `episodes`, `--retrieval-only`
(no LLM), keywords only (`REWIRE_SEMANTIC=0`), k=30. Source of the ideas: a performance review of
[styler-ai/ProjectAtlas](https://github.com/styler-ai/ProjectAtlas) (MIT) and
[Bpolat0/atlasmemory](https://github.com/Bpolat0/atlasmemory) (GPL-3.0; only ideas were taken, no code).

```bash
python experiments/eval/run.py --bench locomo --data locomo10.json --out OUT --sample 0 --retrieval-only \
  --episodes 10 --label trim-e10
```

## 1. Transcript excerpts keep only the relevant turns

The idea comes from atlasmemory (per-file snippets are ranked by query words and the rest is dropped). In rewire: the top two
excerpts stay whole; the following excerpts keep only the turns that share a content word with the
question (ignoring the `Name:` speaker prefix), plus one turn on each side. Excerpts with no matching word
(those that arrive via vectors, a time window, or a fact source) stay whole.

| configuration | evidence turns | all evidence | MRR | context tokens (mean / p95) | retrieval p50 / p95 ms |
|---|---|---|---|---|---|
| before, 8 excerpts | 70.6% | 75.2% | 0.745 | 1,816 / 2,295 | 3 / 8 |
| **before, 10 excerpts (baseline)** | 73.1% | 77.3% | 0.750 | 2,238 / 2,822 | 4 / 9 |
| before, 12 excerpts | 75.5% | 79.1% | 0.754 | 2,655 / 3,301 | 6 / 12 |
| before, 15 excerpts | 77.4% | 80.1% | 0.755 | 3,274 / 4,052 | 7 / 16 |
| **trim, 10 excerpts** | 72.5% | 77.0% | 0.749 | **1,993** / 2,652 | 6 / 14 |
| **trim, 12 excerpts** | **74.7%** | **78.7%** | 0.753 | 2,332 / 3,084 | 8 / 16 |

- With the same number of excerpts (10): tokens −11%, evidence −0.6 points, all evidence −0.3 points.
- With the **same token budget** (≈2,330 tokens; the "before" curve interpolated = 73.6% / 77.7%): trim + 12
  excerpts gives **+1.1 points evidence turns, +1.0 points all evidence**. Multi-hop all evidence 32.3% → 34.0%.
- Cost: Python retrieval p50 rises by ~2 ms (word check per line).

**Tried and not used**: a coordination bonus (how many question words appear in the excerpt; the idea comes from
atlasmemory's phrase + OR query): weights 0.1 / 0.3 / 0.6 gave +0.1–0.4 points evidence and +0.01 MRR, which vanished when
combined with trim, and doubled latency. Trim with 2 context turns: the same as 1 turn, with more
tokens.

## 2. Hot-path latency

`python experiments/perf/bench.py 2000 10000` (Python, median ms, cloud container, 3 repetitions; CLI numbers
include Python startup):

| operation | 2,000 nodes before → after | 10,000 nodes before → after |
|---|---|---|
| session-start hook (CLI) | 200 → **155** | 491 → **331** |
| prime (CLI) | 101 → **90** | 271 → **235** |
| recall (CLI) | 108 → **60** | 68 → 67 |
| recall (MCP, warm server) | 6 → **3** | 14 → **10** |

Changes:
- **The snapshot does not format lines that certainly will not fit** (ProjectAtlas idea: every read has a limit). Profiling the
  hook at 2,000 nodes: 60% of the time was spent formatting and scanning ±1,600 lines that were never shown. Now lines
  whose title (a lower bound on line length) already exceeds the remaining budget are counted without being formatted. Output is identical
  byte for byte (checked with `prime` before/after).
- **Git cache per cwd for 60 seconds**: the two `git` processes (toplevel + remote) per recall took ~3 ms, most
  of the time of a warm MCP recall.

## 3. Parity

`experiments/conformance/recall_parity.py` on conv-26 and conv-30 (233 questions, k=50, 10 excerpts): Python and Rust
results are identical 233/233 (same items and order).

## Other candidates from the review (not yet done)

- Spreading activation only through neighbors relevant to the query, with a cap on the graph bonus (ProjectAtlas
  `projectatlas-service/src/lib.rs` ~2451). Needs `full` mode (facts) to measure; `episodes` mode has no
  graph.
- Tabular MCP recall output (TOON style) without `score`/`b`: an estimated −8–15% tokens.
- File content hash alongside mtime, so that `git checkout`/sync does not trigger re-parsing and re-embedding.
- Classification of each missed question: not a candidate, cut off at k, or lost during merge/trim.
