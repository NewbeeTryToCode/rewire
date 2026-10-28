# A/B: an episode-level entity graph (graph adoption, steps 2–3 of `reports/Graf memori untuk rewire.md`)

Run of 2026-10-19. rewire's existing graph (fact nodes, typed links, 2-hop spreading) exists only in `--mode full`;
in `--mode episodes`, which every sampled A/B uses, it is a no-op. This run tests a graph that works on transcripts:
excerpts are joined by the rare names they share, behind `REWIRE_EP_GRAPH` (off by default, Python only).

- **Entities**: capitalised words found in at most 2% of a brain's excerpts (so speaker names drop out), taken from
  the 3 best excerpts (seeds), at most 6, the rarest first, minus the query's own words.
- **rescore**: an excerpt already matched by the query gets a bonus of at most 0.25 x the best score, scaled by
  tanh(0.5 x shared names).
- **expand (0.25 / 0.5)**: excerpts naming those entities join the rank fusion as a third list at that weight, so a
  second hop without the query's words can enter.

Same samples as the earlier A/Bs: LoCoMo 40, LoCoMo multi-hop 40 (seed 1), LongMemEval_S 60, BEAM 100K 20,
PersonaMem 32k 70. Python engine, WordLlama semantic, rule-gated extra queries (C2). Answerer `claude-haiku-5-5`,
judge `claude-haiku-4-5`. Baseline is arm W of `2026-10-17-ab-embedding-rerank.md`, whose retrieval is identical to
the `off` arm here; unchanged contexts reuse its cached answers.

## Retrieval screen (no LLM; PersonaMem has no evidence labels)

| set | metric | off | rescore | expand 0.25 | expand 0.5 |
|---|---|---|---|---|---|
| LoCoMo 40 | all evidence / evidence turns | 72.5% / 70.7% | 77.5% / 72.8% | 77.5% / 75.0% | 75.0% / 73.9% |
| LoCoMo multi-hop 40 | all evidence / evidence turns | 32.5% / 59.4% | 30.0% / 59.4% | 30.0% / 59.4% | 30.0% / 58.6% |
| LongMemEval_S 60 | all evidence / evidence turns | 98.3% / 96.9% | 98.3% / 96.9% | 98.3% / 96.9% | 98.3% / 96.9% |
| BEAM 20 | all evidence / evidence turns | 58.8% / 63.0% | 58.8% / 63.0% | 58.8% / 63.0% | 58.8% / 63.0% |
| BEAM 20 | recall p50 / p95 | 153 / 1,001 ms | 645 / 3,183 ms | 378 / 1,968 ms | 190 / 1,014 ms |

LoCoMo gains two questions ("How many times has Jolene been to France?", "Which pet did Jolene adopt first"); every
arm loses one multi-hop question ("What is Caroline's identity?"). Context size moves by about 1%.

## End to end, expand 0.25 vs off (W)

| set | n | J off / expand | questions gained / lost | contexts changed | all evidence off / expand | recall p50 off / expand | retrieval tokens off / expand |
|---|---|---|---|---|---|---|---|
| LoCoMo | 40 | 90.0% / 92.5% | +1 / −0 | 24 | 72.5% / 77.5% | 102 / 11 ms | 2,769 / 2,819 |
| LoCoMo multi-hop | 40 | 95.0% / 95.0% | +0 / −0 | 21 | 32.5% / 30.0% | 9 / 9 ms | 2,917 / 2,932 |
| LongMemEval_S | 60 | 88.3% / 90.0% | +2 / −1 | 27 | 98.3% / 98.3% | 321 / 139 ms | 15,060 / 14,904 |
| BEAM 100K | 20 | 51.1% / 40.5% | +0 / −3 | 9 | 58.8% / 58.8% | 347 / 148 ms | 17,945 / 18,187 |
| PersonaMem 32k | 70 | 54.3% / 41.4% | +1 / −10 | 49 | - | 27 / 15 ms | 11,762 / 11,624 |
| **190** (without the extra multi-hop sample) | 190 | **72.2% / 67.4%** | +4 / −14 | | | | |

Retrieval tokens are the memory context per question; the extra-query LLM calls are the same in both arms (the graph
adds no LLM call). Recall latency in the end-to-end runs is wall time under 4 parallel answer calls and is noisier
than the screen above, which ran recall alone.

Every flip came from a changed context (no question flipped with an identical context), so the losses are the
graph's, not judge noise.

## Reading

- **Rejected.** The kill criterion set before the run (any multi-hop loss, or less than +2 points) is met, and the
  end-to-end result is worse: −4.8 points on the 190 questions, driven by PersonaMem (−10 questions) and BEAM (−3).
- **Why it hurts PersonaMem and BEAM**: their histories are long, single-user conversations in which capitalised
  words are mostly product, place and topic names spread across unrelated sessions. A name shared with the best
  excerpt is weak evidence of relevance there, and the third fused list displaces query-matched excerpts. LoCoMo,
  whose rare names are people and pets tied to specific events, is the only set that gains, and only by one question.
- **Same lesson as the earlier excerpt-level entity hop** (see `reports/Graf memori untuk rewire.md`): name overlap does not bridge the dominant
  failure (category vs instance vocabulary), and neighbours admitted for sharing a name crowd out relevant ones.
- The flag stays in the code, off by default, for reproduction; nothing is ported to Rust.
- **Next graph step**: observations (step 4: per-person/concept summaries that carry category words, with source
  links). It needs `--mode full` and new LLM calls at ingest, so it is a cost decision before it is a run.
