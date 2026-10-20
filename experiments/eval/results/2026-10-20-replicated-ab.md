# Replicated A/B: does anything tested beat the baseline?

Run of 2026-10-19/20. The previous three A/Bs (`2026-10-19-ab-episode-entity-graph.md`, `-ab-additive-graph.md`,
`-ab-excerpt-budget-by-type.md`) each answered every question once. A noise check showed that one PersonaMem run moves
by ±4–6 points from answer sampling alone. Here every arm is answered and judged **three times** on the same recalled
contexts (`run.py --replicate 1/2`: answer and judge calls get their own cache entries, while the extra-query calls stay
shared, so the context is identical across replicates). A question's score is its mean J over the three; arms are
compared per question, paired, with a 95% bootstrap interval over questions (`experiments/eval/compare.py`).

Samples, models and engine as before: LoCoMo 40, LoCoMo multi-hop 40 (seed 1), LongMemEval_S 60, BEAM 100K 20,
PersonaMem 32k 70; Python engine, WordLlama, rule-gated extra queries; answerer `claude-haiku-5-5`, judge
`claude-haiku-4-5`.

| arm | what |
|---|---|
| **W** | baseline: 10 excerpts |
| k12 | 12 excerpts |
| add2 | 10 excerpts + at most 2 graph neighbours after them (`REWIRE_EP_GRAPH=add`) |
| ex25 | entity graph fused into the ranking (`REWIRE_EP_GRAPH=expand`) |
| t6 | 6 excerpts for advice/opinion requests and statements, 10 otherwise |
| t6f12 | 6 for advice, 12 otherwise |

## Results (mean of 3 replicates)

| set | n | W | k12 | add2 | ex25 | t6 | t6f12 |
|---|---|---|---|---|---|---|---|
| LoCoMo | 40 | 86.7% | 90.8% | 88.3% | 89.2% | 88.3% | 89.2% |
| LoCoMo multi-hop | 40 | 92.5% | 92.5% | 91.7% | 92.5% | 94.2% | 90.8% |
| LongMemEval_S | 60 | 87.2% | 91.7% | 91.1% | 88.3% | 87.2% | 88.9% |
| BEAM 100K | 20 | 45.8% | 43.1% | 46.7% | 39.2% | 48.9% | 47.4% |
| PersonaMem 32k | 70 | 51.4% | 48.1% | 49.0% | 45.7% | 43.8% | 47.1% |
| **190** (without the extra multi-hop sample) | 190 | **69.6%** | 70.3% | 70.4% | 67.6% | 67.4% | 69.2% |

Difference to W, paired per question, with 95% interval (* = excludes 0):

| set | k12 | add2 | ex25 | t6 | t6f12 |
|---|---|---|---|---|---|
| LoCoMo | +4.2 [−0.8, +10.8] | +1.7 [−1.7, +5.8] | +2.5 [−1.7, +8.3] | +1.7 [−1.7, +5.8] | +2.5 [−2.5, +9.2] |
| LoCoMo multi-hop | 0.0 [−2.5, +2.5] | −0.8 [−5.8, +4.2] | 0.0 [−2.5, +2.5] | +1.7 [−1.7, +5.8] | −1.7 [−4.2, 0.0] |
| LongMemEval_S | +4.4 [−1.1, +11.1] | +3.9 [+0.6, +7.8] * | +1.1 [−2.2, +5.0] | 0.0 [−4.4, +5.0] | +1.7 [−3.9, +7.8] |
| BEAM 100K | −2.8 [−12.4, +6.2] | +0.8 [−6.0, +8.1] | −6.7 [−15.3, +0.1] | +3.1 [−2.5, +9.1] | +1.6 [−9.9, +15.4] |
| PersonaMem 32k | −3.3 [−9.0, +2.4] | −2.4 [−6.7, +1.4] | −5.7 [−11.9, 0.0] * | −7.6 [−13.3, −2.4] * | −4.3 [−11.0, +2.4] |
| **190** | +0.8 [−2.5, +4.2] | +0.8 [−1.4, +3.0] | −1.9 [−4.7, +0.8] | −2.1 [−5.0, +0.6] | −0.4 [−3.8, +3.2] |

## Where the differences come from

A difference on a question whose recalled context is identical in both arms can only be answer/judge noise. Split
by whether the context changed (sum of per-question J differences, in questions):

| arm | set | contexts changed | difference on changed | difference on identical |
|---|---|---|---|---|
| add2 | LongMemEval_S | 2 of 60 | 0.00 | **+2.33** |
| add2 | PersonaMem | 12 of 70 | 0.00 | −1.67 |
| ex25 | PersonaMem | 49 of 70 | −3.67 | −0.33 |
| t6 | PersonaMem | 59 of 70 | −4.33 | −1.00 |
| k12 | LoCoMo / LongMemEval / PersonaMem | all | +1.67 / +2.67 / −2.33 | – |

- **add2's "significant" LongMemEval gain is a false positive**: all of it comes from the 58 questions whose context
  add2 did not change. With 30 comparisons at 95%, about 1.5 such false positives are expected; this one is caught by
  the split. Even averaged over three replicates, two arms with the same context differ by up to ~2 questions per set.
- **ex25 and t6 lose on PersonaMem through changed contexts** (−3.7 and −4.3 questions), so those losses are real.
  PersonaMem is hurt both by name-based neighbours displacing matches and by cutting its excerpts from 10 to 6.

## Conclusions

1. **The graph adds no J.** Fusing entity neighbours into the ranking (ex25) loses on PersonaMem and BEAM. Adding them
   after the matches (add2) changes few contexts, and on those it gains nothing (0.00 on changed contexts in every
   set but one, −0.33 there). This covers the episode-level graph only; rewire's fact graph (`--mode full`) and
   observations remain untested on these samples.
2. **No tested arm beats W.** Every overall interval includes 0. k12 is a trade-off, not an improvement: about +2
   questions each on LoCoMo and LongMemEval, −2 on PersonaMem, +0.8 points overall. Fewer excerpts for advice (t6)
   is worse. W stays the default. That makes it the best configuration among those tested, not a proven optimum.
3. **The baseline's true level is lower than the single cached run suggested**: 69.6% over three replicates against
   72.2% once (PersonaMem 51.4% against 54.3%).
4. **Method from now on**: replicate (`--replicate`), compare paired (`compare.py`), and check that a difference comes
   from questions whose context changed before believing it.
