# LongMemEval_S: recall improvements from the per-question diagnosis (baseline 78.4% -> 83.6% / 85.6%)

All 500 questions, `--mode episodes`, k=30, 10 excerpts, answerer `claude-haiku-5-5`, judge `claude-haiku-4-5`
(both through `claude -p`), official prompts. Baseline and diagnosis: `2026-10-16-longmemeval-haiku55-ep10-analysis.md`.
Reports: `2026-10-16-longmemeval-haiku55-A-improved.md` (A) and `2026-10-16-longmemeval-haiku55-B-expand.md` (B).

## What changed (in Python and Rust)

| # | Change | Why (diagnosis) |
|---|---|---|
| 1 | Semantic search on (`pip install -e '.[semantic]'`, WordLlama ships in the wheel), episode rank fusion weight **0.25** | the run used BM25 only; at weight 1.0 the static vectors pushed the best keyword match out of the top 10 (single-session-user all evidence 100% -> 95%) |
| 2 | Transcripts keep long assistant turns **whole**; recall condenses a line over 1,500 chars to the sentences that best match the query (rare words weighted, "27th" finds item "27.", numbers favoured for "how much / when") | the 1,500-char cut at write time destroyed the answer for good ("blue scaly body", 27th list item) |
| 3 | Every excerpt from later in a session carries the session's **opening user message** (condensed to 700 chars) | 34 of the 45 evidence turns missing from *recalled* sessions were turn 1 |
| 4 | **Twice the excerpt budget** for counting / total / ordering questions (`text.aggregate`) | 32 missing evidence sessions ranked 11-20, mostly multi-session counting |
| 5 | (config B only) **query expansion**: Haiku 5.5 writes 4 keyword queries, fused by rank (`also`, already in rewire; now wired for LongMemEval) | category vs instance vocabulary ("kitchen appliance" vs "smoker") |
| - | Tried and dropped: semantic line choice in `_trim` (+0.1 point evidence for +200 tokens) | |

## Results

Retrieval without an LLM (all 500):

| step | all evidence | evidence turns | MRR | context tokens |
|---|---|---|---|---|
| baseline | 85.7% | 86.2% | 0.857 | 8.8k |
| + semantic at 0.25 | 86.6% | 87.1% | 0.860 | 8.7k |
| + whole turns, condensing, session opener | 86.6% | 90.6% | 0.891 | 8.8k |
| + aggregate budget = **A** | 91.1% | 94.4% | 0.892 | 12.5k |
| A at 15 excerpts (token-matched to B) | 94.7% | 96.4% | 0.895 | 17.7k |
| + query expansion = **B** | **98.3%** | **98.1%** | 0.853 | 19.1k |

End to end (J, official judge prompts, Haiku 4.5 judge):

| category | n | baseline | A | B |
|---|---|---|---|---|
| single-session-user | 70 | 94.3% | 92.9% | 95.7% |
| knowledge-update | 78 | 93.6% | 92.3% | 89.7% |
| single-session-assistant | 56 | 85.7% | 91.1% | 91.1% |
| temporal-reasoning | 133 | 78.9% | 86.5% | 88.0% |
| multi-session | 133 | 65.4% | 75.2% | 79.7% |
| single-session-preference | 30 | 43.3% | 50.0% | 56.7% |
| **overall** | **500** | **78.4%** | **83.6%** | **85.6%** |
| abstention (30) | | 76.7% | 80.0% | 73.3% |
| J when all evidence is in the context | | 88.1% | 89.3% | 87.7% |
| questions turned right / wrong vs baseline | | | +41 / −15 | +60 / −24 |
| answer prompt tokens per question | | 13.6k | 18.4k | 27.4k |
| retrieval p50 / p95 | | 21 / 56 ms | 235 / 651 ms | 1,119 / 3,404 ms |
| API-price equivalent, answer + judge (+ expand) | | $3.37 | $3.89 | $4.96 |

Judge variance is ±1–3 points (§6 of `docs/evaluation.md`); A (+5.2) and B (+7.2) are well above it; the A→B step
(+2.0) is at its edge, with 98.3% vs 91.1% all evidence behind it.

## Reading

- **Retrieval is no longer the bottleneck in B**: 98.3% of questions have every evidence session, yet J is 85.6%;
  J with all evidence (87.7%) is now the ceiling. The remaining errors are answering and judging (over-abstaining,
  generic preference answers, counting, Haiku 4.5's strictness on preference/abstention — see the diagnosis).
  A stronger answerer, or a judge with the official GPT-4o, is the next lever, not more recall.
- **More context costs abstention and knowledge-update**: B's 19k-token context lowers abstention 80.0% -> 73.3%
  (more near-miss material to invent from) and knowledge-update 92.3% -> 89.7% (more stale values next to the new one).
- **Latency regressed** and needs work before shipping: condensing embeds every sentence of every long line at
  recall time (A: p50 21 -> 235 ms), and B runs it for five queries (1.1 s). Fix: cache sentence vectors in the index
  (embed once at refresh, like excerpt vectors), or condense by keywords only and embed only when the scores tie.
- Product defaults: changes 1-4 are on by default (no LLM). Expansion stays the caller's choice (`also` in MCP
  recall): an agent calling rewire writes the extra queries itself.
