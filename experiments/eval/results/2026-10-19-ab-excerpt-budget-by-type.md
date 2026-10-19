# A/B: excerpt budget by question type, and how noisy one run is

Run of 2026-10-19, following `2026-10-19-ab-additive-graph.md`, where 12 excerpts instead of 10 (k12) raised
LongMemEval J by 5.0 points but cost PersonaMem 8.6. Hypothesis: preference and advice requests are hurt by more
near-miss history, factual questions helped by it, so the budget should depend on the question type.

Same samples, models, engine and baseline (W) as the two graph runs.

## What changed (experimental, off by default)

- `text.advice(query)`: the message asks for a suggestion or an opinion ("suggest", "tips", "any ideas", "what do you
  think", "should I"), or is a statement rather than a question, and does not ask to recall an earlier suggestion
  ("remind me what you recommended"). It marks 59/70 PersonaMem, 10/60 LongMemEval (exactly its preference
  questions), 4/20 BEAM, and 1/40 in each LoCoMo sample.
- `ops.recall`: `REWIRE_ADVICE_EPISODES` excerpts for those, `REWIRE_FACT_EPISODES` for the rest; the 2x budget for
  counting questions still applies.

| arm | advice | other |
|---|---|---|
| W | 10 | 10 |
| k12 | 12 | 12 |
| t6 | 6 | 10 |
| t6f12 | 6 | 12 |

## Results

| set | n | J W / k12 / t6 / t6f12 | gained / lost vs W: k12, t6, t6f12 | retrieval tokens W / k12 / t6 / t6f12 |
|---|---|---|---|---|
| LoCoMo | 40 | 90.0 / 92.5 / 90.0 / 87.5 | +1 −0, +0 −0, +1 −2 | 2,769 / 3,283 / 2,744 / 3,250 |
| LoCoMo multi-hop | 40 | 95.0 / 92.5 / 95.0 / 87.5 | +0 −1, +0 −0, +0 −3 | 2,917 / 3,410 / 2,894 / 3,375 |
| LongMemEval_S | 60 | 88.3 / 93.3 / 86.7 / 90.0 | +4 −1, +1 −2, +3 −2 | 15,060 / 17,528 / 14,470 / 16,666 |
| BEAM 100K | 20 | 51.1 / 48.7 / 51.2 / 49.0 | +5 −5, +2 −2, +4 −6 | 17,945 / 21,646 / 17,109 / 20,464 |
| PersonaMem 32k | 70 | 54.3 / 45.7 / 47.1 / 47.1 | +2 −8, +4 −9, +3 −8 | 11,762 / 13,912 / 8,014 / 8,359 |
| **190** (without the extra multi-hop sample) | 190 | **72.2 / 70.9 / 69.1 / 69.4** | | |

Advice questions only (J summed): PersonaMem 59 questions 36.0 (W) / 31.0 / 31.0 / 32.0; LongMemEval 10 questions
6 / 7 / 5 / 6. All evidence is unchanged except LoCoMo (72.5% W, 77.5% with 12 excerpts) and BEAM (58.8% W, 64.7% k12,
52.9% t6). Recall latency (end-to-end runs, p50) is within the run-to-run spread: 9–41 ms LoCoMo, 98–321 ms
LongMemEval, 94–347 ms BEAM, 12–27 ms PersonaMem.

## How noisy is one run?

PersonaMem fell with both fewer (6) and more (12) excerpts, which contradicts the hypothesis. So the W baseline was
answered twice more on the identical contexts, without the reply cache:

| | J |
|---|---|
| W (cached answers used by every A/B) | 54.3% |
| W, fresh answers, run 1 | 50.0% |
| W, fresh answers, run 2 | 48.6% |
| **mean of the three** | **51.0%** |

With the same context, 3–9 of 70 answers differ between runs (only 60 of 70 are stable across all three). So a single
PersonaMem run moves by ±4–6 points from answer sampling alone, and the cached W happens to sit at the top of that
range. Against the 51.0% mean, the budget arms (45.7–47.1%) are 4–5 points lower, about 3 questions: within one run's
noise. The entity-graph arm of the first run (41.4%) is 10 points lower and remains a real loss.

## Reading

- **Rejected; the flags stay off.** No arm beats W on the 190 questions, and the advice budget does not help the
  questions it targets: PersonaMem advice questions score the same or lower with 6 excerpts as with 12.
- **The hypothesis was wrong**: PersonaMem is not hurt by "more context", it is moved by "any context change" plus
  answer sampling. The k12 PersonaMem loss in the previous report reads as noise more than as a context-size effect.
- **k12's LongMemEval gain (+5.0, +4 −1)** is the one consistent signal across these runs (all evidence 98.3% →
  100%), but at 60 questions it is the same size as the noise measured here.
- **Methodological insight, the most useful result of this run**: on these samples a single end-to-end run cannot
  separate changes smaller than about 5 points per set. Before any further A/B, the runner should answer each question
  several times (or take a majority vote) and report the mean with a paired per-question interval, which is step 6 of
  `reports/Graf memori untuk rewire.md`.
