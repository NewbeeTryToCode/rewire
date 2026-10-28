# rewire's own measured latency, token usage and ingest cost

Scope: measurements in the local repo (`/home/user/rewire`, read-only) and result files on this machine (`/tmp/...`), as of 2026-10-20. Nothing was re-run; aggregates marked "(computed)" were calculated with a read-only script from the per-question `results-*.json` rows (`retrieval_ms`, `context_tokens`, `raw_tokens`, `answer_call.prompt_tokens`, `expand_call`) using linear-interpolated percentiles, so they can differ by a few ms from the runner's own nearest-rank percentiles (`experiments/eval/run.py` `pct()`, line 72).

Definitions (from code):
- `context_tokens` / `raw_tokens` are **estimates = characters / 4** (`est_tokens`, `run.py` l.68); `raw_tokens` is "the whole history, as full-context would send it" (`run.py` l.314), so "compression vs full history" = Σraw / Σcontext (`run.py` l.816). — [run.py](/home/user/rewire/experiments/eval/run.py)
- Python `retrieval_ms` times only the recall call (`memories_for(...)` / `ops.recall(...)`), with extra queries (`also`) already computed, and after `index.refresh()` — i.e. **pure retrieval, no LLM time, index already built** (`run.py` l.336–343, 600–608). — [run.py](/home/user/rewire/experiments/eval/run.py)
- "answer tokens in" in reports is the actual API/`claude -p` `usage.prompt_tokens` of the answer call; `answer_prompt_tokens_est` is chars/4 of the prompt text. Actual exceeds the estimate by ~3.2k (LoCoMo) to ~6.7k (LongMemEval) tokens.

---

## Q1. What the reports say: latency, context tokens, answer tokens, per benchmark and configuration

### Takeaway
Pure retrieval is fast: Python p50 ≈ 6–28 ms (LoCoMo), 13–88 ms (LongMemEval_S, default W), 95–153 ms (BEAM 100K), 10–42 ms (PersonaMem 32k); Rust CLI after subtracting a fixed 96 ms process+WordLlama load: p50 7–165 ms. Memory context per question is ≈2.8k tokens (LoCoMo episodes), ≈3.9–4.1k (LoCoMo full mode), ≈15k (LongMemEval_S), ≈18k (BEAM 100K), ≈11.8k (PersonaMem 32k); answer prompts are ≈7.7k / 9.3k / 21.9k / 26.8k / 15.9k actual tokens respectively.

### Cited Findings

**Master table — default-ish configs (episodes mode, k=30, 10 excerpts unless noted; Haiku 5.5 answerer)**

| Benchmark / config (date) | n | Retrieval p50 / p95 / p99 ms | Memory context tokens mean / p95 | Full history tokens/question (raw mean) | Compression | Answer prompt tokens (actual) | Source |
|---|---|---|---|---|---|---|---|
| LoCoMo, keyword-only baseline, 1,540 Q (2026-10-14) | 1540 | 4 / – / 16 | ≈2.5k (≈2.2k after merging) | – | 7.4× | – (LLM-free) | [docs/evaluation.md §8](/home/user/rewire/docs/evaluation.md) |
| LoCoMo **full mode**, k=30, 20 excerpts, 1,540 Q (2026-10-15) | 1540 | 9 / 20 / 29 | 5,013 / 6,067 | ≈18.5k (5,013×3.7) | 3.7× | 11,093 in / 645 out (per call 10,812 / 611) | [2026-10-15-locomo-haiku-full.md, main tables + LLM usage](/home/user/rewire/experiments/eval/results/2026-10-15-locomo-haiku-full.md) |
| LoCoMo base (pre-PR#3, keyword only) | 40 | 6 / 12 / 19 (computed) | 1,912 / 2,441 | 18,234 | 9.5× | 6,459 | [2026-10-17-ab-four-benchmarks.md §locomo](/home/user/rewire/experiments/eval/results/2026-10-17-ab-four-benchmarks.md); /tmp/ab/base/results-locomo-locomo.json |
| LoCoMo A (semantic 0.25, whole turns, 2× aggregate) | 40 | 8 / 17 / 23 (computed) | 2,405 / 3,135 | 18,234 | 7.6× | 7,180 | same; /tmp/ab/A/ |
| LoCoMo B (expand every Q, 4 queries) | 40 | 20 / 38 / 52 (computed) | 3,415 / 5,034 | 18,234 | 5.3× | 8,657 | same; /tmp/ab/B/ |
| LoCoMo C2 / **W default** (rule-gated expansion) — retrieval-only, Python | 40 | 20 / 61 / 68 | 2,769 / 5,294 (report p95) | 18,234 | 6.6× | 7,716 | [2026-10-17-ab-embedding-rerank.md §Retrieval](/home/user/rewire/experiments/eval/results/2026-10-17-ab-embedding-rerank.md); /tmp/emb/W/results-locomo-locomo.json |
| LoCoMo W, measured alone later | 40 | 28 / 89 | 2,769 | – | – | – | [2026-10-19-ab-additive-graph.md l.34](/home/user/rewire/experiments/eval/results/2026-10-19-ab-additive-graph.md); [2026-10-20-locomo-full-graph-ablation.md §Retrieval](/home/user/rewire/experiments/eval/results/2026-10-20-locomo-full-graph-ablation.md) |
| LoCoMo W, end-to-end under 4 parallel answer calls | 40 | 102 / 292 / 341 | 2,769 / 5,294 | – | 6.6× | 7,716 | [2026-10-17-ab-emb-W-locomo-e2e-locomo.md](/home/user/rewire/experiments/eval/results/2026-10-17-ab-emb-W-locomo-e2e-locomo.md) |
| LoCoMo W replicates r1/r2 (e2e, cached context) | 40 | 12/34/47 and 9/43/45 (computed) | 2,769 | 18,234 | 6.6× | 7,715 | /tmp/gr/E-W/results-locomo-e2e-locomo-r{1,2}.json |
| LoCoMo multi-hop extra (seed 1), W | 40 | 10 / 38 / 49 (retrieval-only) | 2,917 (p95 6,855 computed) | 18,416 | 6.3× | 7,917 | embedding-rerank report; /tmp/emb/W/ |
| LoCoMo k12 (12 excerpts) | 40 | 11–15 / 46–252 (computed, e2e) | 3,283 / 5,698 | 18,234 | 5.6× | 8,462 | /tmp/gr/E-k12/; [2026-10-20-replicated-ab.md](/home/user/rewire/experiments/eval/results/2026-10-20-replicated-ab.md) |
| LoCoMo k12, retrieval alone | 40 | 49 / 180 | – | – | – | – | [2026-10-19-ab-additive-graph.md l.34–35](/home/user/rewire/experiments/eval/results/2026-10-19-ab-additive-graph.md) |
| LoCoMo **F0** (full mode, spreading off), e2e | 40 | 18–20 / 45–47 / 77–86 (computed) | 3,863 / 5,893 | 18,234 | 4.7× | 9,326 | [2026-10-20-locomo-full-graph-ablation.md](/home/user/rewire/experiments/eval/results/2026-10-20-locomo-full-graph-ablation.md); /tmp/fg/F0/ |
| LoCoMo **F2** (full mode, 2-hop spreading = default), e2e | 40 | 18–21 / 46–52 / 57–90 (computed) | 3,883 / 5,909 | 18,234 | 4.7× | 9,357 | same; /tmp/fg/F2/ |
| LoCoMo F0 / F2, retrieval-only run (includes first-use index build) | 40 | 288 / 726 and 288 / 767 | 3,863 / 3,883 | – | – | – | full-graph-ablation §Retrieval |
| LoCoMo multi-hop F0 / F2 (e2e) | 40 | 15–18 / 48–245 (computed) | 4,129 / 4,103 (p95 ≈7.1–7.3k) | 18,416 | 4.5× | 9,650–9,691 | /tmp/fg/F{0,2}/ |
| LongMemEval_S baseline (keyword, 500 Q, 2026-10-16) | 500 | 21 / 56 / 195 | 8,842 / 11,052 | ≈122.4k (computed) | 13.8× | 13,613 in / 760 out | [2026-10-16-longmemeval-haiku55-ep10.md](/home/user/rewire/experiments/eval/results/2026-10-16-longmemeval-haiku55-ep10.md); /tmp/eval-out/ |
| LongMemEval_S A (2026-10-16, sentence-vector condensing at recall time) | 500 | 235 / 651 / 978 | 12,490 / 19,310 | 122.4k | 9.8× | 18,414 / 774 | [2026-10-16-longmemeval-improvements.md §Results](/home/user/rewire/experiments/eval/results/2026-10-16-longmemeval-improvements.md); [-A-improved.md](/home/user/rewire/experiments/eval/results/2026-10-16-longmemeval-haiku55-A-improved.md) |
| LongMemEval_S B (expansion on every Q) | 500 | 1,119 / 3,404 / 4,387 | 19,117 / 30,067 | 122.4k | 6.4× | 27,376 / 816 | improvements.md; [-B-expand.md](/home/user/rewire/experiments/eval/results/2026-10-16-longmemeval-haiku55-B-expand.md) |
| LongMemEval_S base / A / B (60-Q sample, 2026-10-17) | 60 | p50 13 / 51 / 203 | 8.7k / 11.6k / 17.4k | 122.7k | 14.2× / 10.6× / 7.0× (computed) | 13,332 / 17,217 / 25,148 (computed) | [ab-four-benchmarks.md §longmemeval](/home/user/rewire/experiments/eval/results/2026-10-17-ab-four-benchmarks.md) |
| LongMemEval_S **W / C2** retrieval-only (Python) | 60 | 88 / 259 / 322 | 15,060 / 27,862 | 122,693 | 8.1× | 21,911 | embedding-rerank §Retrieval; /tmp/emb/W/ |
| LongMemEval_S W e2e (parallel) | 60 | 321 / 1,034 | 15,060 | – | 8.1× | 21,911 | embedding-rerank §End to end |
| LongMemEval_S k12 | 60 | 70–194 / 249–493 (computed) | 17,528 / 32,955 | 122,693 | 7.0× | 25,252 | /tmp/gr/E-k12/ |
| BEAM 100K base / A / B | 20 | p50 17 / 106 / 493 | 11.6k / 13.8k / 22.5k | 150,951 | 13.1× / 11.0× / 6.7× (computed) | 17,993 / 20,862 / 33,226 | ab-four-benchmarks §beam |
| BEAM 100K **W / C2** retrieval-only | 20 | 107 / 612 (report); 95 / 513 / 592 (computed) | 17,945 / 48,547 (report p95) | 150,951 | 8.4× | 26,818 | embedding-rerank §Retrieval; [ab-C2-beam-beam.md](/home/user/rewire/experiments/eval/results/2026-10-17-ab-C2-beam-beam.md) |
| BEAM 100K W measured alone (2026-10-19) | 20 | 153 / 1,001 | – | – | – | – | [2026-10-19-ab-episode-entity-graph.md l.27](/home/user/rewire/experiments/eval/results/2026-10-19-ab-episode-entity-graph.md) |
| BEAM 100K k12 | 20 | 244 / 1,215 (alone) | 21,646 / 46,341 (computed) | 150,951 | 7.0× | 32,002 | additive-graph l.34–35; /tmp/gr/E-k12/ |
| BEAM 100K dry run (early) | 20 | – | ≈13.4k | – | ≈11× | – | evaluation.md §8 |
| PersonaMem 32k all (2026-10-17) | 589 | 42 / 181 / 277 | 11,263 / 13,522 | 34,951 (computed) | 3.1× | 15,285 / 540 | [2026-10-17-personamem-haiku55.md](/home/user/rewire/experiments/eval/results/2026-10-17-personamem-haiku55.md); [-32k-haiku55.md](/home/user/rewire/experiments/eval/results/2026-10-17-personamem-32k-haiku55.md) |
| PersonaMem 128k (20/type) | 140 | 57 / 214 / 274 | 11,017 / 12,676 | 159,538 (computed) | 14.5× | 15,037 / 520 | personamem-haiku55.md; [-128k-s20](/home/user/rewire/experiments/eval/results/2026-10-17-personamem-128k-s20-haiku55.md) |
| PersonaMem 1M (20/type) | 140 | 77 / 220 / 826 | 11,746 / 13,398 | 1,167,656 (computed) | 99.4× | 16,008 / 543 | personamem-haiku55.md; [-1M-s20](/home/user/rewire/experiments/eval/results/2026-10-17-personamem-1M-s20-haiku55.md) |
| PersonaMem 32k **W / C2** (70-Q sample, e2e) | 70 | 27 / 94 (report); r1/r2 15/34 and 15/31 (computed) | 11,762 / 18,182 | 36,006 | 3.1× | 15,886 | embedding-rerank §End to end; /tmp/gr/E-W/ |
| PersonaMem 32k base / A / B | 70 | p50 20 / 20 / 52 | 10.9k / 11.8k / 19.5k | 36,006 | 3.3× / 3.1× / 1.8× (computed) | 14,915 / 15,886 / 25,063 | ab-four-benchmarks §personamem |
| PersonaMem 32k k12 | 70 | 18–24 / 33–258 (computed) | 13,912 / 20,863 | 36,006 | 2.6× | 18,429 | /tmp/gr/E-k12/ |

**Rust engine via CLI (C2rs/C3rs/C4rs, 2026-10-17)** — latency = wall time of one `rewire recall` process **minus a fixed 96 ms** (process start + WordLlama load, measured on an empty brain; 6 ms without semantic). — [2026-10-17-ab-rust-gated-expansion.md, intro + Summary](/home/user/rewire/experiments/eval/results/2026-10-17-ab-rust-gated-expansion.md)

| Benchmark | n | Rust recall p50 C2 / C3 / C4 (ms, net) | Rust recall p95 C2 / C3 / C4 (net) | Raw wall p50 / p95 / p99 C2 (incl. 96 ms, computed) | Memory context C2 mean / p95 | Answer prompt C2 |
|---|---|---|---|---|---|---|
| LoCoMo | 40 | 14 / 16 / 14 | 57 / 59 / 50 | 110 / 238 / 243 | 2,769 / 5,294 | 7,716 |
| LongMemEval_S | 60 | 156 / 165 / 157 | 299 / 300 / 267 | 304 / 574 / 645 | 15,062 / 29,025 | 21,915 |
| BEAM 100K | 20 | 73 / 142 / 147 | 404 / 410 / 549 | 226 / 415 / 482 | 17,945 / 48,547 | 26,818 |
| PersonaMem 32k | 70 | 10 / 20 / 14 | 35 / 70 / 38 | 108 / 141 / 165 | 11,762 / 18,506 | 15,886 |
| LoCoMo multi-hop extra | 40 | 13 / 22 / 7 | 52 / 61 / 41 | 105 / 131 / 145 | 2,952 / 7,960 | 7,968 |
| all 190 | 190 | – | – | – | 11,562 mean (C2), 11,890 (C3), 12,083 (C4) | 17,220 / 17,650 / 17,937 |

- "Rust = Python": Rust C2 gives the same verdict as Python C2 on all 190 questions (72.2% both; 189/190 identical context sizes). — rust-gated-expansion.md, intro
- "A long-running MCP server loads the model once, so the numbers below are closer to what it sees." — rust-gated-expansion.md, intro
- README claim: Rust binary "~9 MB, no runtime; `prime` ~20 ms, recall ~20 ms". — [README.md l.20](/home/user/rewire/README.md)
- Earlier port comparison (keyword-only): Rust `prime` 11 ms vs Python 97 ms, `recall` 10 ms vs 84 ms (CLI). — [docs/iterations.md l.75](/home/user/rewire/docs/iterations.md)
- Warm MCP server, synthetic brain (Python): recall 6 ms @500 nodes, 8 ms @2,000, 34 ms @10,000 nodes; recall via CLI 87 / 81 / 95 ms (includes ~70 ms Python startup); session-start hook 77 / 83 / 168 ms; `get` by short ID via MCP 3 / 5 / 25 ms. — [experiments/perf/README.md table](/home/user/rewire/experiments/perf/README.md); also [docs/iterations.md l.36, l.49](/home/user/rewire/docs/iterations.md) ("MCP recall 908 → 34 ms at 10,000 memories"; "16 ms at 10,000 memories with 20,000 relations")

**Embedding / rerank variants (Python, retrieval-only, 2026-10-17)** — recall p50 / p95: LoCoMo W 21/61, P (potion) 22/67, G (bge-small ONNX) 141/630, R (cross-encoder) 6,917/34,131, R2 1,015/4,862 ms; LongMemEval W 88/259, P 101/347, G 346/1,165, R 30,189/45,556, R2 2,170/4,766 ms; BEAM W 107/612, P 106/900, R2 1,093/4,874 ms. "Latency here was measured with two arms running at once (except G)"; "single-threaded, one reranked recall took ≈2.1 s". — [2026-10-17-ab-embedding-rerank.md §Retrieval](/home/user/rewire/experiments/eval/results/2026-10-17-ab-embedding-rerank.md)
- Vector top-n via `argpartition`: 1M vectors 31 → 6 ms. — same report, intro

### Inferences
- The defensible "product" retrieval latency is: Python in-process p50 ≈10–30 ms (LoCoMo/PersonaMem), ≈90 ms (LongMemEval_S), ≈100–150 ms (BEAM 100K); p95 ≈60–90 / 260 / 600–1,000 ms. Rust net-of-startup is similar to or slower than Python on long-history benchmarks (LME 156 ms vs Python 88 ms p50), even though it is faster on small brains; per-call CLI adds ~96 ms with WordLlama.
- LongMemEval_S A/B numbers from 2026-10-16 (235 ms / 1.1 s p50) are **obsolete**: they used sentence-vector condensing at recall time that was later dropped "for latency with the same evidence" (personamem-haiku55.md l.33–34); the later W (same recall logic minus sentence vectors, plus rule-gated expansion) is 88 ms p50 retrieval-only.
- The `answer_prompt_tokens_est` (chars/4) underestimates actual billed prompt tokens by ~40%; for comparisons with other systems use the actual "answer tokens in" values.

### Gaps
- No p99 reported for the Rust arms in the repo; the computed raw-wall p99 above includes the 96 ms overhead and n=20–70.
- No direct measurement of the Rust engine as a warm MCP server on benchmark brains; only the subtraction estimate and the synthetic Python perf bench.
- No retrieval latency measured for LongMemEval/BEAM/PersonaMem in full mode (full mode only run on LoCoMo).

---

## Q2. Full-context comparison: tokens per question if the full history were sent

### Takeaway
By rewire's own chars/4 estimate, full history per question is ≈18.2–18.5k tokens (LoCoMo), ≈122.4–122.7k (LongMemEval_S), ≈151k (BEAM 100K), ≈35–36k (PersonaMem 32k), ≈160k (128k) and ≈1.17M (1M); rewire's context is 3.7–6.6× smaller on LoCoMo, ≈8–14× on LongMemEval_S, ≈8.4× on BEAM 100K, ≈3.1× / 14.5× / 99.4× on PersonaMem 32k / 128k / 1M.

### Cited Findings
- LoCoMo raw history per question (chars/4): 18,234 (40-Q sample), 18,416 (multi-hop seed 1) (computed from `raw_tokens` in /tmp/emb/W/results-locomo-*.json); full-mode 1,540 Q run: context 5,013 × 3.7× ≈ 18.5k. — [2026-10-15-locomo-haiku-full.md](/home/user/rewire/experiments/eval/results/2026-10-15-locomo-haiku-full.md)
- Compression vs full history, LoCoMo: 7.4× keyword baseline (evaluation.md §8), 6.6× W/C2, 5.6× k12, 4.7× F0/F2 full mode (k=30, 10 excerpts), 3.7× full mode with 20 excerpts. — reports above
- Mem0 paper full-context reference: "≈26k tokens per conversation, p95 ≈17 s", J ≈72.9%; Mem0 ≈7k tokens, p95 ≈1.44 s (as quoted in rewire's docs). — [docs/evaluation.md §4 table](/home/user/rewire/docs/evaluation.md)
- LongMemEval_S: "500 questions, each with a history of ~50 sessions (~115k tokens)"; "context ≈11.2k tokens out of ≈115k (≈10× smaller)" (iteration 8). — [docs/evaluation.md §2, §8](/home/user/rewire/docs/evaluation.md). rewire's own chars/4 raw estimate is 122,448 (500 Q) / 122,693 (60 Q) (computed, /tmp/eval-out, /tmp/emb/W).
- LongMemEval_S compression: 13.8× (keyword baseline, 8.8k ctx), 9.8× (A), 8.1× (W/C2), 7.0× (k12), 6.4× (B). — reports above
- BEAM 100K raw ≈150,951 tokens/question (computed); compression 8.4× (W/C2), 13.1× (base), 6.7× (B), 7.0× (k12); dry run ≈11×. — /tmp/ab/*, emb/W; evaluation.md §8
- PersonaMem: "compression vs the history up to the question 3.1× / 14.5× / 99.4×" for 32k / 128k / 1M at ≈11–11.7k context. — [2026-10-17-personamem-haiku55.md](/home/user/rewire/experiments/eval/results/2026-10-17-personamem-haiku55.md). Dataset sizes: "≈27k tokens per context" (32k), "≈925k tokens per context (median)" (1M). — evaluation.md §2; dry run 1M: "context ≈12.7k tokens out of ≈925k (compression ≈92×)". — evaluation.md §8

### Inferences
- rewire's LoCoMo raw estimate (≈18.3k, chars/4) is lower than Mem0's ≈26k "per conversation" — different tokenizer, and rewire counts only turn text up to the question; rewire's LoCoMo context (≈2.8k, W) is ≈1/9 of the Mem0 26k figure and ≈1/6.6 of its own raw estimate. When comparing, state which denominator is used.
- rewire's actual answer prompt (7.7k LoCoMo; 21.9k LME_S) is still far below full-context (26k / 115k+), but the LoCoMo answer prompt is larger than Mem0's ≈7k-token retrieval alone only by the instructions + `claude -p` overhead.

### Gaps
- No tokenizer-accurate (non chars/4) full-history counts in the repo.
- No full-context baseline run by rewire itself (no measured J or latency for full-context in the same harness).

---

## Q3. LLM tokens spent on retrieval (query expansion), and which latencies include LLM calls

### Takeaway
Rule-gated expansion (C2/W) costs ≈252–262 prompt + 62–76 completion tokens per *expanded* question via the API (≈1,570–1,580 with the 1,319-token `claude -p` overhead), fires on 0–50% of questions, and adds one LLM call of p50 ≈4.9 s / p95 ≈6.0–6.3 s. Model-gated (C3/C4) pays a call on every question (~390 + 150 tokens avg, call p50 ≈5.2–5.4 s). All retrieval_ms numbers exclude the LLM expansion call.

### Cited Findings
- "Expansion tokens exclude the 1,319 prompt tokens that `claude -p` adds to every call (measured with a one-word prompt); through the API the expansion prompt itself is about 170 (rule) or 280 (model-gated) tokens. Model-gated arms (C3, C4) make one call for every question … completion tokens include the model's thinking." — [rust-gated-expansion.md §Tokens spent on retrieval](/home/user/rewire/experiments/eval/results/2026-10-17-ab-rust-gated-expansion.md)
- Per expanded question (in + out, excl. overhead), C2: LoCoMo 252 + 71, LME 262 + 69, BEAM 260 + 62, LoCoMo multi-hop 249 + 76; C3: 361–381 + 206–349; C4: 369–433 + 207–246. Averaged over all 190 questions: C2 67 + 18, C3 390 + 151, C4 407 + 156. — same section
- Questions expanded (C2 rule / C3 model): LoCoMo 13 / 22 of 40; multi-hop 7 / 30 of 40; LME 30 / 26 of 60; BEAM 6 / 12 of 20; PersonaMem 0 / 4 of 70. — same report §Summary
- Including overhead (raw `expand_call` rows, computed): C2 per expanded question 1,568–1,581 prompt + 62–76 completion; C3 1,680–1,741 + 100–212; C4 1,680–1,879 + 100–219. — /tmp/ab/C2rs, C3rs, C4rs results JSON
- W LoCoMo report: "extra-query LLM calls 510 prompt + 23 completion on average over all questions, 1,571 + 71 on the 13 questions given extra queries". — [2026-10-17-ab-emb-W-locomo-e2e-locomo.md](/home/user/rewire/experiments/eval/results/2026-10-17-ab-emb-W-locomo-e2e-locomo.md)
- Additive-graph report: extra-query calls "about 270–790 prompt tokens per question on LoCoMo/LongMemEval, none on PersonaMem" (averaged, incl. overhead). — [2026-10-19-ab-additive-graph.md l.31–32](/home/user/rewire/experiments/eval/results/2026-10-19-ab-additive-graph.md)
- B (expand every question) expand-stage LLM usage, per call in/out and latency p50/p95: LoCoMo 1,572/70, 4,725/5,704 ms; LME 1,580/73, 4,902/6,312 ms; BEAM 1,583/76, 5,007/6,296 ms; PersonaMem 1,629/72, 4,930/5,740 ms; LoCoMo ab80-B 1,737/72, 4,231/6,113 ms. — `| expand |` rows in [2026-10-17-ab-B-*.md](/home/user/rewire/experiments/eval/results/2026-10-17-ab-B-longmemeval-lme.md), [2026-10-15-locomo-ab80-B.md](/home/user/rewire/experiments/eval/results/2026-10-15-locomo-ab80-B.md)
- Expand-call latency from caches (computed): C2rs/W p50 4,906–4,909 ms, p95 6,004–6,172 ms (n=195); C3rs p50 5,408 / p95 6,874 (n=416); C4rs p50 5,221 / p95 6,649 (n=611). — /tmp/ab/*/cache/llm.jsonl, /tmp/emb/W/cache/llm.jsonl
- Retrieval timing excludes LLM: the runner computes `also` beforehand and times only `memories_for`/`ops.recall`. — [run.py l.336–343, 600–608](/home/user/rewire/experiments/eval/run.py)
- Extra queries via MCP need no rewire LLM call: "The calling model (an agent using rewire over MCP) is already an LLM, so it writes the extra queries … rewire itself needs no extra LLM call." — [2026-10-15-locomo-ab-extra-queries.md §56](/home/user/rewire/experiments/eval/results/2026-10-15-locomo-ab-extra-queries.md)
- Expansion also grows recall time because recall runs 5 queries: B LME p50 1.1 s (with old condensing); A→B on 60-Q sample 51 → 203 ms; "retrieval latency 3–29×" for B. — improvements.md; ab-four-benchmarks.md §Reading
- Rust: "C3/C4 add 0–75 ms p50 where they expand more (BEAM 73 → 142–147 ms)". — rust-gated-expansion.md §Reading

**Which latencies are noisy (parallel load)**
- "Recall latency in the end-to-end runs is under load (4 questions in parallel, answering running alongside)." — embedding-rerank.md, end of §End to end. E.g., LoCoMo W e2e 102/292 ms vs 20/61 ms retrieval-only; LME W 321/1,034 vs 88/259; BEAM 347/1,588 vs 107/612.
- "Recall latency in the end-to-end runs is wall time under 4 parallel answer calls and is noisier" — [2026-10-19-ab-episode-entity-graph.md l.44](/home/user/rewire/experiments/eval/results/2026-10-19-ab-episode-entity-graph.md)
- Embedding retrieval-only latencies "measured with two arms running at once (except G)". — embedding-rerank.md §Retrieval
- Excerpt-budget report: e2e p50 "within the run-to-run spread: 9–41 ms LoCoMo, 98–321 ms LongMemEval, 94–347 ms BEAM, 12–27 ms PersonaMem". — [2026-10-19-ab-excerpt-budget-by-type.md l.38–39](/home/user/rewire/experiments/eval/results/2026-10-19-ab-excerpt-budget-by-type.md)
- Clean "measured alone" numbers: LoCoMo 28/89 ms (W), 60/171 (add2), 49/180 (k12); BEAM 153/1,001 (W). — additive-graph.md l.34–35
- The 2026-10-17 selective-expansion A/B (C, C2, D on Python) reports retrieval p50 182–448 ms (LoCoMo C2 182 ms, LME 368, BEAM 448, PM 96) — these used the older recall with sentence-vector condensing and/or parallel load and are not representative. — [2026-10-17-ab-selective-expansion.md](/home/user/rewire/experiments/eval/results/2026-10-17-ab-selective-expansion.md)

### Inferences
- For a "per-query latency incl. LLM" figure with expansion via `claude -p`, add ≈4.9 s p50 (≈6 s p95) on expanded questions; through the direct API it would be smaller but this is not measured. With an MCP agent writing `also`, the expansion cost is part of the agent's own turn.
- Effective rule-gated expansion cost per question averaged over a benchmark is small vs. answer prompts (67 + 18 API tokens avg, or ≈200–800 with `claude -p` overhead) compared with 8k–30k answer prompt tokens.

### Gaps
- Expansion latency through the direct Anthropic API (no `claude -p`) is not measured.
- Answer-call `claude -p` overhead is not separately measured (only the 1,319-token figure for a one-word prompt).

---

## Q4. Ingest cost (episodes vs full), index build time, disk footprint

### Takeaway
Episodes mode makes zero LLM calls at ingest; full mode consolidates each session with ~2 LLM calls, measured p50 42 s/session (2026-10-15) and 37 s/session (2026-10-20 rebuild; p95 55.5 s, ≈2.9 h serial for 272 LoCoMo sessions). Ingest token totals were not recorded. A LoCoMo conversation brain is ≈1.1 MB on disk in episodes mode (.index 612 KB) and ≈6.6 MB in full mode (.index 1.9 MB, .git 3.3 MB).

### Cited Findings
- "`episodes`: transcripts are stored and indexed exactly as consolidation would produce them, **without** LLM extraction. Free." "`full`: every session is consolidated by an LLM (`--ingest-model`) … Most realistic, most expensive." — [docs/evaluation.md §2 Ingest modes](/home/user/rewire/docs/evaluation.md)
- Ingest stage: "~2 calls per transcript session, not per question. LoCoMo: ~272 sessions; LongMemEval: ~50 sessions × 500 questions …, so `full` for LongMemEval is only feasible for small samples". — evaluation.md §5 table. Free-model plan: "±550 [requests] for ingest (extraction + judge per session, 272 sessions)". — evaluation.md §9
- LoCoMo full (Haiku 5.5, 2026-10-15): "272 sessions consolidated (0 failed …, 1 replies that were not JSON). Facts per session: 15.8 (4120 applied, 191 queued then approved). Hypothetical questions: 3345 (0.57 per turn). Seconds per session p50: 42." Active nodes: fact 2401, preference 553, decision 449, person 35, concept 8. — [2026-10-15-locomo-haiku-full.md §Ingest](/home/user/rewire/experiments/eval/results/2026-10-15-locomo-haiku-full.md)
- LoCoMo full re-ingest for graph ablation (2026-10-20): 272 sessions all ok, 218–458 nodes per conversation (~3,600 total), facts/session 16.0, "Seconds per session p50: 37". — [2026-10-20-locomo-full-graph-ablation.md](/home/user/rewire/experiments/eval/results/2026-10-20-locomo-full-graph-ablation.md); /tmp/fg/F0/report-locomo-e2e-locomo.md §Ingest
- Computed from `/tmp/graph/locomo/brains-full/*/.state/eval_ingest.jsonl`: 272 sessions, 5,882 turns, seconds/session p50 37.3, p95 55.5, mean 38.6, total 2.91 h of session wall time.
- Full mode "costs an LLM ingest (about 2 calls per session, 272 sessions here), about 40% more context per question, and a slower first index build." — full-graph-ablation.md §Reading
- Production consolidation: "The dominant cost of consolidation is still the LLM call (~10–17 seconds per session, running in the background after a session ends)"; consolidation excluding LLM time 385 ms (10k nodes), 140 ms (500 nodes). — [experiments/perf/README.md](/home/user/rewire/experiments/perf/README.md)
- Index build: full rebuild 76 ms (500 nodes), 281 ms (2,000), 1,300–1,600 ms (10,000 nodes; "happens once per machine"). — perf/README.md table. LoCoMo full mode: retrieval-only p50 288 ms "include building the full-mode index on first use; in the end-to-end runs, with the index built, recall p50 / p95 was 15–21 / 54–72 ms". — full-graph-ablation.md §Retrieval
- bge-small (G) note: "embedding BEAM and PersonaMem histories takes hours on this 4-vCPU container (≈8 long excerpts/s)"; "slow indexing on CPU". — embedding-rerank.md
- Disk footprint (measured with `du`, 2026-10-20):
  - Full-mode LoCoMo brain `/tmp/graph/locomo/brains-full/conv-26`: 6.6 MB total = `.index` 1.9 MB (`brain.db` 1,949,696 B) + `.git` 3.3 MB + `nodes/` 1.2 MB (270 node files) + `sessions/` 244 KB + `events/` 24 KB. All 10 conversations: 87 MB total, `.index` 25.2 MB total.
  - Episodes-mode LoCoMo brain `/tmp/gr/E-W/locomo/brains-episodes/conv-26`: 1.1 MB total = `.index` 612 KB (`brain.db` 622,592 B) + `.git` 200 KB + `sessions/` 188 KB; 0 node files. All 10 conversations: 13 MB, `.index` 7.3 MB total.
  - Episodes-mode other benchmarks: LongMemEval_S 60 brains 172 MB (≈2.8–3.1 MB each; `.index` 122 MB total ≈2 MB each); BEAM 100K 14 brains 46 MB (3.0–4.6 MB each; `.index` 33.9 MB total); PersonaMem 32k 37 MB.
- Rust binary size "~9 MB, no runtime". — README.md l.20

### Inferences
- Full-mode LoCoMo ingest ≈ 272 × 2 ≈ 544 LLM calls for 10 conversations (≈54 per conversation); serial time ≈2.9 h, i.e. ≈17 min per LoCoMo conversation, or ≈37–42 s per session — vs. ~10–17 s per session reported for real-use background consolidation (shorter sessions).
- Episodes-mode `.index` is ≈3× the raw transcript text size (612 KB index vs 188 KB sessions for conv-26), driven by FTS + WordLlama vectors; full mode adds node files and a 3× larger index.

### Gaps
- **Ingest tokens are not recorded anywhere found**: report "LLM usage" tables list only answer/judge(/expand) stages; no `ingest` stage entries exist in any `/tmp/*/cache/llm.jsonl` checked (fg, ab, emb, gr, eval-out, eval-v5/v6, eval-pm). Total ingest tokens for full mode must be estimated, not cited.
- No measured index build time for benchmark brains (only the synthetic perf bench and the "first use" note).
- Disk footprint of the Rust index (if different from Python's `brain.db`) not checked.
