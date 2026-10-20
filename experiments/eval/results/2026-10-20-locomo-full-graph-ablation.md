# LoCoMo `--mode full`: what the fact graph adds (spreading ablation, replicated)

Run of 2026-10-20. Step 1 of `reports/Graf memori untuk rewire.md`. rewire's fact graph (LLM-extracted nodes,
typed links, 2-hop spreading in `Index.search`) only exists in `--mode full`, so it had never been measured on the
sampled sets, which all run in `--mode episodes`.

- **Ingest**: all 10 LoCoMo conversations (272 sessions, all ok), consolidated by `claude-haiku-5-5`, giving
  218–458 nodes per conversation (about 3,600 in all). The sampled questions touch every conversation, so all of
  them had to be ingested.
- **Samples**: LoCoMo 40 (10 per category, seed 0) and LoCoMo multi-hop 40 (seed 1). Answerer `claude-haiku-5-5`,
  judge `claude-haiku-4-5`, rule-gated extra queries. Every arm was answered and judged **3 times**
  (`--replicate`) and compared per question, paired, with a 95% bootstrap interval (`compare.py`).

| arm | mode | spreading |
|---|---|---|
| W | episodes (baseline, from `2026-10-20-replicated-ab.md`) | – |
| F0 | full | off (`REWIRE_SPREAD_HOPS=0`) |
| F2 | full | 2 hops (rewire's default) |
| Fs | full | only along `supersedes` (`REWIRE_SPREAD_MODE=supersedes`) |

## Retrieval (no LLM)

| set | arm | evidence turns | all evidence (excerpts or a fact from the turn) | context tokens | recall p50 / p95 |
|---|---|---|---|---|---|
| LoCoMo 40 | W | 70.7% | 72.5% | 2,769 | 28 / 89 ms |
| | F0 | 65.2% | 70.0% | 3,863 | 288 / 726 ms |
| | F2 | 68.5% | 72.5% | 3,883 | 288 / 767 ms |
| | Fs | 65.2% | 70.0% | 3,863 | 258 / 745 ms |
| LoCoMo multi-hop 40 | W | 59.4% | 32.5% | 2,917 | 25 / 102 ms |
| | F0 | 64.8% | 42.5% | 4,129 | 269 / 687 ms |
| | F2 | 65.6% | 40.0% | 4,103 | 295 / 837 ms |
| | Fs | 64.8% | 42.5% | 4,133 | 271 / 780 ms |

Fs gives the same contexts as F0 on these questions (supersedes links are rare in LoCoMo), so it was not run end to
end. The retrieval-only latencies include building the full-mode index on first use; in the end-to-end runs, with the
index built, recall p50 / p95 was 15–21 / 54–72 ms for F0 and F2, against 9–102 ms p50 for W. Retrieval tokens are
the memory context per question (facts + excerpts); the extra-query calls are the same in every arm.

## End to end (mean of 3 replicates)

| set | n | W | F0 | F2 | F2 − F0 [95% CI] | F0 − W [95% CI] | F2 − W [95% CI] |
|---|---|---|---|---|---|---|---|
| LoCoMo | 40 | 86.7% | 91.7% | 89.2% | −2.5 [−9.2, +4.2] | +5.0 [−5.0, +15.0] | +2.5 [−4.2, +10.0] |
| LoCoMo multi-hop | 40 | 92.5% | 90.8% | 93.3% | +2.5 [−1.7, +8.3] | −1.7 [−8.3, +3.3] | +0.8 [−4.2, +5.8] |
| **both** | 80 | **89.6%** | **91.2%** | **91.2%** | **0.0 [−4.2, +4.6]** | +1.7 [−4.2, +7.5] | +1.7 [−2.5, +6.2] |

By category (sum of J over the 10 questions, LoCoMo 40): multi-hop W 9.0 / F0 9.0 / F2 10.0; open-domain
7.3 / 8.0 / 6.7; single-hop 10 / 10 / 10; temporal 8.3 / 9.7 / 9.0.

F2 and F0 recall different contexts on 38 of 40 LoCoMo and 32 of 40 multi-hop questions. On those, spreading gains
1.3 questions on multi-hop and loses 1.0 on LoCoMo, mostly open-domain.

## Reading

- **Spreading activation adds no J**: F2 = F0 = 91.2% over 80 questions, interval [−4.2, +4.6]. It changes almost
  every context and helps multi-hop slightly while hurting open-domain slightly. That is the same pattern as the
  earlier excerpt-level entity hop, now measured on rewire's own graph.
- **Full mode itself is not proven better than episodes on these samples**: +1.7 points with an interval from −4.2
  to +7.5. It costs an LLM ingest (about 2 calls per session, 272 sessions here), about 40% more context per question,
  and a slower first index build.
- **What the facts do bring** is retrieval on multi-hop: all evidence counting facts rises from 32.5% (episodes) to
  40–42.5%. At 80 questions that does not reach J, because multi-hop J is already 91–93% with excerpts alone.
- **Conclusion for graph adoption**: with spreading measured at zero on the sampled LoCoMo sets, step 1's kill criterion
  ("if off ≥ on, freeze graph work and keep spreading only for supersedes forwarding") applies. Spreading stays as it
  is in full mode (it does no harm); no further graph retrieval work is planned. Observations (step 4) remain the only
  graph idea aimed at the vocabulary gap, and they would need new LLM calls on top of full mode.
