# A/B: embedding models and cross-encoder reranking (C2 as the baseline)

Run of 2026-10-17, Python engine, extra queries as in C2 (`--expand-when gather`, expansions from the C2 cache).
Same questions as the earlier A/B runs: LoCoMo 40 (10 per category), LoCoMo multi-hop 40 more (seed 1),
LongMemEval_S 60, BEAM 100K 20, PersonaMem 32k 70. Answerer `claude-haiku-5-5`, judge `claude-haiku-4-5`.

| arm | what changes (all opt-in, env vars; the default is unchanged) |
|---|---|
| **W** (baseline) | WordLlama 256-d static embeddings (current default) |
| **P** | Model2Vec `minishlab/potion-base-8M` static embeddings (`REWIRE_EMBED=potion`) |
| **G** | `BAAI/bge-small-en-v1.5` through fastembed ONNX, query instruction for queries (`REWIRE_EMBED=bge-small`); LoCoMo and LongMemEval only — embedding BEAM and PersonaMem histories takes hours on this 4-vCPU container (≈8 long excerpts/s) |
| **R** | cross-encoder `Xenova/ms-marco-MiniLM-L-6-v2` reorders the top 30 fused excerpts (`REWIRE_RERANK`) — retrieval only |
| **R2** | as R, but the reranker reads the first 600 chars of each of the top 20, and its order is fused with recall's by rank (`REWIRE_RERANK_POOL=20`, `_CHARS=600`, `_BLEND=1`) |
| **PR, PR2** | potion with R / R2 — retrieval only |

Also in this change (no effect on results): vector search selects the top n with `argpartition` instead of sorting
every vector (same order, 1M vectors 31 → 6 ms).

## Retrieval (no LLM)

| benchmark | arm | all evidence | evidence turns | MRR | context tokens | recall p50 / p95 |
|---|---|---|---|---|---|---|
| LoCoMo | W | 72.5% | 70.7% | 0.825 | 2,769 | 21 / 61 ms |
| LoCoMo | P | 77.5% | 73.9% | 0.794 | 2,787 | 22 / 67 ms |
| LoCoMo | R | 77.5% | 78.3% | 0.827 | 2,824 | 6917 / 34131 ms |
| LoCoMo | PR | 77.5% | 76.1% | 0.828 | 2,842 | 7192 / 33998 ms |
| LoCoMo | R2 | 75.0% | 71.7% | 0.860 | 2,750 | 1015 / 4862 ms |
| LoCoMo | PR2 | 77.5% | 79.3% | 0.838 | 2,807 | 920 / 4265 ms |
| LoCoMo | G | 80.0% | 75.0% | 0.815 | 2,702 | 141 / 630 ms |
| LoCoMo multi-hop (extra) | W | 32.5% | 59.4% | 0.615 | 2,917 | 10 / 46 ms |
| LoCoMo multi-hop (extra) | P | 35.0% | 59.4% | 0.633 | 2,914 | 8 / 39 ms |
| LoCoMo multi-hop (extra) | R | 42.5% | 65.6% | 0.695 | 2,974 | 6649 / 34406 ms |
| LoCoMo multi-hop (extra) | PR | 45.0% | 66.4% | 0.695 | 2,974 | 6660 / 39042 ms |
| LoCoMo multi-hop (extra) | R2 | 45.0% | 64.8% | 0.704 | 2,919 | 505 / 4011 ms |
| LoCoMo multi-hop (extra) | PR2 | 45.0% | 66.4% | 0.690 | 2,898 | 484 / 3807 ms |
| LoCoMo multi-hop (extra) | G | 45.0% | 66.4% | 0.730 | 2,964 | 86 / 300 ms |
| LongMemEval_S | W | 98.3% | 96.9% | 0.893 | 15,060 | 88 / 259 ms |
| LongMemEval_S | P | 100.0% | 96.9% | 0.853 | 14,814 | 101 / 347 ms |
| LongMemEval_S | R | 96.6% | 91.8% | 0.801 | 13,907 | 30189 / 45556 ms |
| LongMemEval_S | PR | 96.6% | 90.8% | 0.797 | 13,953 | 31305 / 44686 ms |
| LongMemEval_S | R2 | 98.3% | 96.9% | 0.878 | 14,319 | 2170 / 4766 ms |
| LongMemEval_S | PR2 | 98.3% | 95.9% | 0.833 | 14,253 | 1965 / 5012 ms |
| LongMemEval_S | G | 100.0% | 99.0% | 0.841 | 14,684 | 346 / 1165 ms |
| BEAM 100K | W | 58.8% | 63.0% | 0.479 | 17,945 | 107 / 612 ms |
| BEAM 100K | P | 52.9% | 60.9% | 0.530 | 17,860 | 106 / 900 ms |
| BEAM 100K | R | 41.2% | 56.5% | 0.451 | 17,565 | 9332 / 21990 ms |
| BEAM 100K | PR | 41.2% | 56.5% | 0.440 | 17,234 | 13177 / 44182 ms |
| BEAM 100K | R2 | 64.7% | 60.9% | 0.523 | 17,882 | 1093 / 4874 ms |
| BEAM 100K | PR2 | 52.9% | 58.7% | 0.522 | 17,715 | 1007 / 5571 ms |

Latency here was measured with two arms running at once (except G); R's 7–30 s p50 is mostly 512-token pairs on
long excerpts — single-threaded, one reranked recall took ≈2.1 s.

## End to end (J)

| benchmark | arm | n | J | vs W better / worse | all evidence | evidence turns | context tokens | recall p50 / p95 |
|---|---|---|---|---|---|---|---|---|
| LoCoMo | W | 40 | 90.0% | - | 72.5% | 70.7% | 2,769 | 102 / 292 ms |
| LoCoMo | P | 40 | 90.0% | 1 / 1 | 77.5% | 73.9% | 2,787 | 49 / 222 ms |
| LoCoMo | G | 40 | 92.5% | 1 / 0 | 80.0% | 75.0% | 2,702 | 148 / 615 ms |
| LoCoMo | R2 | 40 | 87.5% | 1 / 2 | 75.0% | 71.7% | 2,750 | 908 / 9813 ms |
| LoCoMo multi-hop (extra) | W | 40 | 95.0% | - | 32.5% | 59.4% | 2,917 | 9 / 48 ms |
| LoCoMo multi-hop (extra) | P | 40 | 87.5% | 0 / 3 | 35.0% | 59.4% | 2,914 | 10 / 34 ms |
| LoCoMo multi-hop (extra) | G | 40 | 87.5% | 0 / 3 | 45.0% | 66.4% | 2,964 | 114 / 440 ms |
| LoCoMo multi-hop (extra) | R2 | 40 | 90.0% | 0 / 2 | 45.0% | 64.8% | 2,919 | 843 / 4105 ms |
| LongMemEval_S | W | 60 | 88.3% | - | 98.3% | 96.9% | 15,060 | 321 / 1034 ms |
| LongMemEval_S | P | 60 | 91.7% | 2 / 0 | 100.0% | 96.9% | 14,814 | 151 / 565 ms |
| LongMemEval_S | G | 60 | 91.7% | 2 / 0 | 100.0% | 99.0% | 14,684 | 444 / 1058 ms |
| LongMemEval_S | R2 | 60 | 88.3% | 2 / 2 | 98.3% | 96.9% | 14,319 | 2167 / 8030 ms |
| BEAM 100K | W | 20 | 51.1% | - | 58.8% | 63.0% | 17,945 | 347 / 1588 ms |
| BEAM 100K | P | 20 | 40.9% | 2 / 7 | 52.9% | 60.9% | 17,860 | 359 / 700 ms |
| BEAM 100K | R2 | 20 | 45.4% | 2 / 6 | 64.7% | 60.9% | 17,882 | 3001 / 5442 ms |
| PersonaMem 32k | W | 70 | 54.3% | - | - | - | 11,762 | 27 / 94 ms |
| PersonaMem 32k | P | 70 | 52.9% | 2 / 3 | - | - | 11,703 | 16 / 42 ms |
| PersonaMem 32k | R2 | 70 | 54.3% | 6 / 6 | - | - | 11,771 | 802 / 3093 ms |

LoCoMo + LoCoMo multi-hop + LongMemEval (140, all four arms): W 90.7%, P 90.0%, G 90.7%, R2 88.6%
The 190 of the earlier A/B (LoCoMo, LongMemEval, BEAM, PersonaMem): W 72.2%, P 71.7%, R2 71.1%

Recall latency in the end-to-end runs is under load (4 questions in parallel, answering running alongside).

## Reading

- **Better retrieval did not turn into more right answers.** bge-small has the best retrieval (LoCoMo all evidence
  72.5% → 80.0%, multi-hop 32.5% → 45.0%, LongMemEval evidence turns 96.9% → 99.0%), but J over the 140
  LoCoMo + LongMemEval questions is 90.7% for both W and G. On the 190 earlier questions: W 72.2%, P 71.7%, R2 71.1%.
- **Why:** where J moved, the evidence often had not. Of G's three multi-hop losses, two had the same complete
  evidence as W (2/2, 3/3) and the answerer miscounted ("3" for "two") or answered in another form; the third lost one
  of six evidence turns. A different context composition shifts the answerer by a question or two per benchmark,
  which on 20–60 questions is 2–10 points of noise. LoCoMo and LongMemEval J are at 88–95%, where the remaining
  misses are answering, not finding (as the LongMemEval diagnosis already showed).
- **Reranking costs too much for what it gives**: R2 lifts multi-hop and BEAM evidence, but J does not rise and
  recall takes 0.5–3 s p50 per query (five queries with extra queries). Plain R (full text, no fusion) hurts
  LongMemEval and BEAM evidence: the MS MARCO model reads only the first 512 tokens of long chat excerpts.
- **potion** is as fast as WordLlama and slightly better on LoCoMo/LongMemEval evidence, but lower on BEAM
  (evidence and J, 2 better / 7 worse on 20 questions) and PersonaMem (2 / 3).

## Decision

- Keep WordLlama as the default; the options stay available for experiments (`REWIRE_EMBED`, `REWIRE_RERANK*`).
- bge-small is the one to revisit if retrieval becomes the bottleneck again (e.g. BEAM's information-extraction
  questions or `full` mode at scale): best evidence, latency in the hundreds of ms, but an ONNX dependency, slow
  indexing on CPU and a Rust port still to do.
- No reranker by default: no J gain, seconds of latency.
- The next lever is the answerer side (counting slips, answer form) or measuring the graph (see
  `reports/Graf memori untuk rewire.md`), not the embedding.
