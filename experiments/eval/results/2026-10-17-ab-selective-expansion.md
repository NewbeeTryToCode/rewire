# A/B with A as the baseline: selective extra queries (C, C2) and oldest-first excerpts (D)

Run of 2026-10-17, on the same 190 questions as `2026-10-17-ab-four-benchmarks.md` (LoCoMo 40, LongMemEval_S 60,
BEAM 100K 20, PersonaMem 32k 70; seed 0), answerer `claude-haiku-5-5`, judge `claude-haiku-4-5`.

| arm | what changes against A | why (insight from the previous A/B) |
|---|---|---|
| **A** (baseline) | `main`: whole turns + condensing, session opener, 2× excerpts for aggregation questions, semantic 0.25 | — |
| **C** | extra queries (Haiku 5.5) only for counting / total / ordering / time questions (`--expand-when gather`) | extra queries helped counting and temporal questions, hurt preference questions |
| **D** | C + BEAM and PersonaMem excerpts shown oldest first (`--excerpt-order chrono`) | PersonaMem's one undated session may hide which preference is newer |
| **C2** | C, but only for text in question form | C's three PersonaMem losses were all *statements* with narrative time words |

Questions that an arm does not expand get exactly A's context, so their answers and verdicts come from A's cache:
differences between arms come only from the changed questions (no re-answering noise).

## Overall

| benchmark | n | A | C | D | C2 |
|---|---|---|---|---|---|
| LoCoMo | 40 | 88% | 90% | 90% | 90% |
| LongMemEval_S | 60 | 90% | 88% | 88% | 88% |
| BEAM 100K (rubric score) | 20 | 39% | 51% | 44% | 51% |
| PersonaMem 32k | 70 | 54% | 50% | 51% | 54% |
| **all 190** | 190 | 70.9% | 70.6% | 70.4% | **72.2%** |
| questions better / worse than A | | | 4 / 4 | 12 / 13 | **4 / 1** |

D's LoCoMo and LongMemEval rows equal C's (both already show excerpts in time order).

## What changed per question

| benchmark | question | category | C2 vs A |
|---|---|---|---|
| LoCoMo | How many hikes has Joanna been on? | open-domain | wrong → right |
| BEAM | How many words does my final essay draft contain? | knowledge update | 0 → 1 |
| BEAM | How many scenes had I filmed in total by July 5 and how many were left …? | multi-session reasoning | 0 → 1 |
| BEAM | How many days after I started limiting work emails after 7 PM did I begin …? | temporal reasoning | 0 → 0.5 |
| LongMemEval | How many graduation ceremonies have I attended in the past three months? | multi-session | right → wrong |
| PersonaMem | three statements ("During my trip last weekend, I …", "I also opted out of cooking classes that I once enjoyed …", "Just recently, I found myself …") | facts / preference evolution | wrong in C, back to A's right answers in C2 |

## Reading

- **Selective extra queries (C2) are the best arm**: +1.3 points over A on 190 questions, 4 better and 1 worse, and
  every gain is a "how many" question, the case the diagnosis predicted. Expansion fires on 13/40 LoCoMo, 30/60
  LongMemEval, 6/20 BEAM and 0/70 PersonaMem questions in C2, so the extra cost (one Haiku call, recall with five queries,
  average context +15–30% on LoCoMo, LongMemEval and BEAM) is paid only where it helps.
- **Expanding everything (B) is still worse than A** (67.3% vs 70.9%, previous run): the gate is what makes extra
  queries pay off.
- **Oldest-first excerpts (D) do not help**: 12 better and 13 worse against A, PersonaMem 54% → 51%. The staleness
  hypothesis is not supported at this sample size; keep best-match-first.
- The sample is small (±6.5 points on 190 questions); +1.3 points is a direction, confirmed by the one-sided flips
  (4 vs 1), not a measured effect size. A confirmation run on full LongMemEval with C2 would cost ~250 expanded
  questions (answers for the other 250 come from the A cache).

## Product consequence

The MCP `also` guide (Python and Rust) asks agents for extra queries on counting, total, list, ordering and time
questions, not on preference or advice questions, and (from C2) not when a time is only mentioned in passing.

## Per-benchmark tables (A / C / C2)

### locomo

| category | n | J A | J C | J C2 | all evidence A / C / C2 | context tokens A / C / C2 |
|---|---|---|---|---|---|---|
| multi-hop | 10 | 90% | 90% | 90% | 60% / 60% / 60% | 2.3k / 2.4k / 2.4k |
| open-domain | 10 | 70% | 80% | 80% | 50% / 50% / 50% | 2.4k / 2.5k / 2.5k |
| single-hop | 10 | 100% | 100% | 100% | 100% / 100% / 100% | 2.3k / 2.5k / 2.5k |
| temporal | 10 | 90% | 90% | 90% | 80% / 80% / 80% | 2.6k / 3.6k / 3.6k |
| **overall** | 40 | 88% | 90% | 90% | 72% / 72% / 72% | 2.4k / 2.8k / 2.8k |

retrieval p50 A / C / C2: 8 ms / 207 ms / 182 ms
C vs A, per question: 1 better, 0 worse
C2 vs A, per question: 1 better, 0 worse

### longmemeval

| category | n | J A | J C | J C2 | all evidence A / C / C2 | context tokens A / C / C2 |
|---|---|---|---|---|---|---|
| knowledge-update | 10 | 100% | 100% | 100% | 100% / 100% / 100% | 11.2k / 14.8k / 14.8k |
| multi-session | 10 | 80% | 70% | 70% | 89% / 100% / 100% | 15.9k / 21.8k / 21.8k |
| single-session-assistant | 10 | 100% | 100% | 100% | 100% / 100% / 100% | 10.1k / 12.9k / 12.9k |
| single-session-preference | 10 | 60% | 60% | 60% | 90% / 90% / 90% | 9.7k / 9.7k / 9.7k |
| single-session-user | 10 | 100% | 100% | 100% | 100% / 100% / 100% | 8.1k / 8.5k / 8.5k |
| temporal-reasoning | 10 | 100% | 100% | 100% | 78% / 100% / 100% | 14.6k / 22.7k / 22.7k |
| **overall** | 60 | 90% | 88% | 88% | 93% / 98% / 98% | 11.6k / 15.1k / 15.1k |

retrieval p50 A / C / C2: 51 ms / 296 ms / 368 ms · abstention: 100% / 100% / 100%
C vs A, per question: 0 better, 1 worse
C2 vs A, per question: 0 better, 1 worse

### beam

| category | n | J A | J C | J C2 | all evidence A / C / C2 | context tokens A / C / C2 |
|---|---|---|---|---|---|---|
| abstention | 2 | 100% | 100% | 100% | - / - / - | 10.7k / 10.7k / 10.7k |
| contradiction_resolution | 2 | 25% | 25% | 25% | 100% / 100% / 100% | 12.2k / 12.2k / 12.2k |
| event_ordering | 2 | 66% | 66% | 66% | 0% / 0% / 0% | 11.7k / 11.7k / 11.7k |
| information_extraction | 2 | 25% | 25% | 25% | 0% / 0% / 0% | 11.5k / 11.5k / 11.5k |
| instruction_following | 2 | 50% | 50% | 50% | 50% / 50% / 50% | 11.2k / 13.3k / 13.3k |
| knowledge_update | 2 | 0% | 50% | 50% | 50% / 100% / 100% | 15.6k / 21.7k / 21.7k |
| multi_session_reasoning | 2 | 0% | 50% | 50% | 100% / 100% / 100% | 19.9k / 40.4k / 40.4k |
| preference_following | 2 | 50% | 50% | 50% | 50% / 50% / 50% | 11.3k / 11.3k / 11.3k |
| summarization | 2 | 45% | 45% | 45% | 0% / 0% / 0% | 11.5k / 11.5k / 11.5k |
| temporal_reasoning | 2 | 25% | 50% | 50% | 100% / 100% / 100% | 22.0k / 35.0k / 35.0k |
| **overall** | 20 | 39% | 51% | 51% | 53% / 59% / 59% | 13.8k / 17.9k / 17.9k |

retrieval p50 A / C / C2: 106 ms / 318 ms / 448 ms · abstention: 100% / 100% / 100%
C vs A, per question: 3 better, 0 worse
C2 vs A, per question: 3 better, 0 worse

### personamem

| category | n | J A | J C | J C2 | all evidence A / C / C2 | context tokens A / C / C2 |
|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | 10 | 30% | 30% | 30% | - / - / - | 11.4k / 11.4k / 11.4k |
| provide_preference_aligned_recommendations | 10 | 60% | 60% | 60% | - / - / - | 11.8k / 11.8k / 11.8k |
| recall_user_shared_facts | 10 | 50% | 40% | 50% | - / - / - | 11.2k / 12.5k / 11.2k |
| recalling_facts_mentioned_by_the_user | 10 | 40% | 40% | 40% | - / - / - | 11.2k / 11.2k / 11.2k |
| recalling_the_reasons_behind_previous_updates | 10 | 90% | 90% | 90% | - / - / - | 12.3k / 16.0k / 12.3k |
| suggest_new_ideas | 10 | 30% | 30% | 30% | - / - / - | 11.3k / 11.3k / 11.3k |
| track_full_preference_evolution | 10 | 80% | 60% | 80% | - / - / - | 13.1k / 19.8k / 13.1k |
| **overall** | 70 | 54% | 50% | 54% | - / - / - | 11.8k / 13.4k / 11.8k |

retrieval p50 A / C / C2: 20 ms / 81 ms / 96 ms
C vs A, per question: 0 better, 3 worse
C2 vs A, per question: 0 better, 0 worse
