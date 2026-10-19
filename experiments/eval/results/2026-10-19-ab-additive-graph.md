# A/B: additive graph neighbours, with a token-matched control

Run of 2026-10-19, following `2026-10-19-ab-episode-entity-graph.md`, whose entity graph lost 4.8 points (190
questions) because neighbours fused into the ranking displaced query-matched excerpts (PersonaMem −10, BEAM −3).
Same samples, models, engine and baseline (W, 10 excerpts) as that run.

## Insights tested

1. **Add, do not displace.** Graph neighbours come after the normal top excerpts; the query's own ranking is untouched.
2. **One shared name is weak evidence.** A neighbour must share two rare names with the seed excerpts, or one name and
   a query word (`Index.entity_neighbours`).
3. **Control for context size.** An arm with two more ordinary excerpts separates "graph" from "more context".

| arm | what |
|---|---|
| off (W) | 10 excerpts |
| add2 | 10 excerpts + at most 2 graph neighbours (`REWIRE_EP_GRAPH=add`, `REWIRE_EP_GRAPH_ADD=2`) |
| k12 | 12 excerpts, no graph (`--episodes 12`) |

## Results

| set | n | J off / add2 / k12 | gained / lost vs off: add2, k12 | all evidence off / add2 / k12 | retrieval tokens off / add2 / k12 |
|---|---|---|---|---|---|
| LoCoMo | 40 | 90.0% / 90.0% / 92.5% | +0 −0, +1 −0 | 72.5% / 72.5% / 77.5% | 2,769 / 2,924 / 3,283 |
| LoCoMo multi-hop (seed 1) | 40 | 95.0% / 92.5% / 92.5% | +0 −1, +0 −1 | 32.5% / 32.5% / 35.0% | 2,917 / 2,979 / 3,410 |
| LongMemEval_S | 60 | 88.3% / 88.3% / 93.3% | +0 −0, +4 −1 | 98.3% / 98.3% / 100% | 15,060 / 15,121 / 17,528 |
| BEAM 100K | 20 | 51.1% / 52.4% / 48.7% | +1 −0, +5 −5 | 58.8% / 58.8% / 64.7% | 17,945 / 18,082 / 21,646 |
| PersonaMem 32k | 70 | 54.3% / 54.3% / 45.7% | +1 −1, +2 −8 | - | 11,762 / 12,022 / 13,912 |
| **190** (without the extra multi-hop sample) | 190 | **72.2% / 72.4% / 70.9%** | | | |

Retrieval tokens are the memory context per question; extra-query LLM calls are identical in all arms (rule-gated,
about 270–790 prompt tokens per question on LoCoMo/LongMemEval, none on PersonaMem).

Recall latency, measured alone (retrieval-only runs, p50 / p95): LoCoMo 28 / 89 ms off, 60 / 171 ms add2,
49 / 180 ms k12; BEAM 153 / 1,001 ms off, 337 / 1,342 ms add2, 244 / 1,215 ms k12. The add2 cost is the
document-frequency lookups for candidate names.

## Reading

- **Insights 1 and 2 hold**: added after the matches and gated on two shared names, the graph no longer hurts
  (72.4% vs 72.2%; PersonaMem and BEAM back to baseline). It also does not help: it fires on a minority of questions
  (2–15 contexts changed per set), finds no new evidence session, and loses one multi-hop question. **Not adopted**;
  the flag stays off.
- **More context is not uniformly good** (insight 3): k12 raises evidence everywhere and J on LongMemEval (+5.0) and
  LoCoMo (+2.5), but costs PersonaMem 8.6 points and nets zero on BEAM. PersonaMem's multiple-choice preference
  questions get worse as more near-miss history is shown, the same pattern as abstention on LongMemEval (B arm,
  2026-10-16). The excerpt budget should depend on the question type rather than the graph.
- **Next**: a per-question-type excerpt budget (more for factual lookups and counting, fewer for preference and
  recommendation questions), tested on the same five samples. For the graph, the remaining step is observations
  (`--mode full`, new LLM calls at ingest).
