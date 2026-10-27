# A/B on all four benchmarks (sampled): baseline vs A vs B

Run of 2026-10-17. Same questions in every arm (stratified sample, seed 0), `--mode episodes`, k=30,
10 excerpts, answerer `claude-haiku-5-5`, judge `claude-haiku-4-5` (both through `claude -p`), official prompts.

| arm | recall code | semantic | extra queries |
|---|---|---|---|
| **base** | before PR #3 (commit d977c67) | off (keyword only, as the 78.4% LongMemEval run) | none |
| **A** | `main` after PR #3 (whole turns + condensing, session opener, 2× excerpts for aggregation questions, semantic at 0.25) | on | none |
| **B** | as A | on | 4 queries written by Haiku 5.5 per question (`also`) |

Samples: LoCoMo 10 per category (40), LongMemEval_S 10 per type (60), BEAM 100K 2 per category (20),
PersonaMem 32k 10 per type (70). Per-arm runner reports: `2026-10-17-ab-{base,A,B}-*.md`.

**Sample size warning.** A category holds 2–10 questions: one question moves it 10–50 points. Per-category numbers
show direction only; the 95% interval on a 60–70 question benchmark is about ±10 points, on 190 questions ±6.5.

## Overall

| benchmark | n | base | A | B |
|---|---|---|---|---|
| LoCoMo | 40 | 88% | 88% | 88% |
| LongMemEval_S | 60 | 82% | **90%** | 82% |
| BEAM 100K (rubric score) | 20 | 47% | 39% | **54%** |
| PersonaMem 32k (multiple choice) | 70 | **57%** | 54% | 47% |
| **all 190** | 190 | 70.2% | **70.9%** | 67.3% |

For reference, the full LongMemEval_S runs (500 questions, same models): base 78.4%, A 83.6%, B 85.6%.

## By broad category

| broad category | where measured | A vs base | B vs base |
|---|---|---|---|
| Fact recall (needle in a haystack) | LoCoMo single-hop, LME single-session-user/-assistant, PersonaMem recall facts | flat (already 90–100%) | flat, one LME miss |
| Cross-session synthesis / counting | LoCoMo multi-hop, LME multi-session, BEAM multi-session | **up** (LoCoMo 80→90%, LME 70→80%; full LME 65→75%) | evidence up (LME 78→100%), J flat in the sample; full LME 65→80% |
| Temporal reasoning | LoCoMo temporal, LME temporal-reasoning, BEAM temporal / event ordering | **up** (LME 70→100%; full LME 79→87%) | **up** (LME 70→100%; full 79→88%) |
| Knowledge update | LME knowledge-update, BEAM knowledge update | flat (LME 100%); BEAM 2 questions noisy | flat |
| Preferences / personalisation | LME single-session-preference, PersonaMem, BEAM preference following | mixed (LME 50→60%, PersonaMem 57→54%) | **down** in all three samples (LME 50→30%, PersonaMem 57→47%, BEAM 50→38%) |
| Generative / transfer (new ideas, generalising, summaries) | PersonaMem suggest/generalize, BEAM summarization | flat to down (PersonaMem generalize 50→30%) | down (PersonaMem suggest 30→0%) |
| Abstention | LME `_abs`, BEAM abstention | flat (100% in the samples) | flat |

## Reading

- **A is a safe default**: it helps where the diagnosis said it would (cross-session counting and temporal questions,
  through more evidence in context) and is flat elsewhere; over all 190 questions it is level with base (70.9% vs 70.2%).
  Cost: context +8–33% (aggregation questions get twice the excerpts), retrieval p50 6–20 ms → 8–106 ms.
- **B is not a safe default.** Extra queries bring the evidence for counting and temporal questions (LME all evidence
  90% → 98%), but they hurt preference and generative questions in every sample (PersonaMem 57% → 47%, 12 questions
  worse vs 5 better). A likely cause (inferred, not tested): PersonaMem keeps the whole history in one undated
  session, and the extra, broadly-worded queries ("e-books versus print books reading") add excerpts with the user's
  *older* preferences (excerpts per question 6.0 → 10.6) that the answerer cannot order in time. Context also grows
  1.8–2× and retrieval latency 3–29×.
- **Next step suggested by this run:** use extra queries selectively — for counting, total, ordering and temporal
  questions (where `text.aggregate` or a time phrase fires) and not for preference or advice questions — and say so
  in the MCP `also` guide. Then re-measure B on LongMemEval and PersonaMem.

Cost (API-price equivalent, subscription billed $0): base $1.50, A $1.48, B $1.88 for the 190 questions.

## Per-benchmark tables

### locomo

| category | n | J base | J A | J B | all evidence base / A / B | context tokens base / A / B |
|---|---|---|---|---|---|---|
| multi-hop | 10 | 80% | 90% | 80% | 40% / 60% / 60% | 2.0k / 2.3k / 3.4k |
| open-domain | 10 | 80% | 70% | 80% | 50% / 50% / 50% | 1.7k / 2.4k / 3.4k |
| single-hop | 10 | 100% | 100% | 100% | 100% / 100% / 100% | 2.0k / 2.3k / 3.3k |
| temporal | 10 | 90% | 90% | 90% | 80% / 80% / 80% | 1.9k / 2.6k / 3.6k |
| **overall** | 40 | 88% | 88% | 88% | 68% / 72% / 72% | 1.9k / 2.4k / 3.4k |

retrieval p50 base / A / B: 6 ms / 8 ms / 20 ms
A vs base, per question: 1 better, 1 worse
B vs base, per question: 1 better, 1 worse

### longmemeval

| category | n | J base | J A | J B | all evidence base / A / B | context tokens base / A / B |
|---|---|---|---|---|---|---|
| knowledge-update | 10 | 100% | 100% | 100% | 100% / 100% / 100% | 8.3k / 11.2k / 17.3k |
| multi-session | 10 | 70% | 80% | 70% | 78% / 89% / 100% | 9.4k / 15.9k / 22.6k |
| single-session-assistant | 10 | 100% | 100% | 100% | 100% / 100% / 100% | 9.0k / 10.1k / 14.3k |
| single-session-preference | 10 | 50% | 60% | 30% | 80% / 90% / 90% | 9.5k / 9.7k / 14.4k |
| single-session-user | 10 | 100% | 100% | 90% | 100% / 100% / 100% | 6.8k / 8.1k / 12.5k |
| temporal-reasoning | 10 | 70% | 100% | 100% | 78% / 78% / 100% | 8.9k / 14.6k / 23.3k |
| **overall** | 60 | 82% | 90% | 82% | 90% / 93% / 98% | 8.7k / 11.6k / 17.4k |

retrieval p50 base / A / B: 13 ms / 51 ms / 203 ms · abstention: 100% / 100% / 100%
A vs base, per question: 6 better, 1 worse
B vs base, per question: 4 better, 4 worse

### beam

| category | n | J base | J A | J B | all evidence base / A / B | context tokens base / A / B |
|---|---|---|---|---|---|---|
| abstention | 2 | 100% | 100% | 100% | - / - / - | 12.0k / 10.7k / 18.0k |
| contradiction_resolution | 2 | 25% | 25% | 12% | 100% / 100% / 50% | 10.8k / 12.2k / 22.7k |
| event_ordering | 2 | 44% | 66% | 66% | 0% / 0% / 0% | 11.6k / 11.7k / 14.8k |
| information_extraction | 2 | 21% | 25% | 21% | 0% / 0% / 50% | 12.2k / 11.5k / 17.1k |
| instruction_following | 2 | 50% | 50% | 50% | 50% / 50% / 50% | 11.5k / 11.2k / 15.6k |
| knowledge_update | 2 | 50% | 0% | 100% | 100% / 50% / 100% | 10.7k / 15.6k / 24.1k |
| multi_session_reasoning | 2 | 50% | 0% | 50% | 100% / 100% / 100% | 11.6k / 19.9k / 40.4k |
| preference_following | 2 | 50% | 50% | 38% | 0% / 50% / 100% | 11.6k / 11.3k / 16.4k |
| summarization | 2 | 45% | 45% | 55% | 0% / 0% / 0% | 11.9k / 11.5k / 21.4k |
| temporal_reasoning | 2 | 38% | 25% | 50% | 100% / 100% / 100% | 11.7k / 22.0k / 35.0k |
| **overall** | 20 | 47% | 39% | 54% | 53% / 53% / 65% | 11.6k / 13.8k / 22.5k |

retrieval p50 base / A / B: 17 ms / 106 ms / 493 ms · abstention: 100% / 100% / 100%
A vs base, per question: 5 better, 5 worse
B vs base, per question: 5 better, 2 worse

### personamem

| category | n | J base | J A | J B | all evidence base / A / B | context tokens base / A / B |
|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | 10 | 50% | 30% | 30% | - / - / - | 11.2k / 11.4k / 17.7k |
| provide_preference_aligned_recommendations | 10 | 70% | 60% | 70% | - / - / - | 11.5k / 11.8k / 20.0k |
| recall_user_shared_facts | 10 | 50% | 50% | 50% | - / - / - | 11.1k / 11.2k / 18.7k |
| recalling_facts_mentioned_by_the_user | 10 | 30% | 40% | 50% | - / - / - | 11.3k / 11.2k / 20.7k |
| recalling_the_reasons_behind_previous_updates | 10 | 90% | 90% | 70% | - / - / - | 10.4k / 12.3k / 20.2k |
| suggest_new_ideas | 10 | 30% | 30% | 0% | - / - / - | 11.3k / 11.3k / 17.8k |
| track_full_preference_evolution | 10 | 80% | 80% | 60% | - / - / - | 9.8k / 13.1k / 21.6k |
| **overall** | 70 | 57% | 54% | 47% | - / - / - | 10.9k / 11.8k / 19.5k |

retrieval p50 base / A / B: 20 ms / 20 ms / 52 ms
A vs base, per question: 6 better, 8 worse
B vs base, per question: 5 better, 12 worse
