# BEAM 100K: summary of the first end-to-end runs (2026-10-16)

Setup: BEAM commit `b2da22e`, 100K size (20 chats, 400 questions, 10 categories), `--mode episodes` (raw transcript
excerpts, no LLM extraction), k=30, 10 excerpts ≈ 11.9k context tokens per question (12.9× smaller than the chat),
keyword recall. Official BEAM RAG answer prompt and per-rubric-item judge (`beam_prompts.py`). Answerer
`claude-cli/haiku`; judges `claude-cli/haiku` (all 400) and `claude-cli/sonnet` (100, 10 per category, same answers).

Files: [full run](2026-10-16-beam-100k-haiku.md) · [per-question diagnosis, Haiku judge](2026-10-16-beam-100k-haiku-diagnosis.md)
([csv](2026-10-16-beam-100k-haiku-diagnosis.csv)) · [Sonnet re-judge](2026-10-16-beam-100k-s10-sonnet-judge.md) ·
[per-question diagnosis, Sonnet judge](2026-10-16-beam-100k-s10-sonnet-judge-diagnosis.md) · retrieval only:
[100K](2026-10-15-beam-100k-retrieval.md), [500K + 1M](2026-10-15-beam-500k-1m-retrieval.md).

## The judge moves the score by 17 points

| category | n | Haiku judge | Sonnet judge | all evidence in context |
|---|---|---|---|---|
| abstention | 10 | 80.0% | 90.0% | – |
| contradiction_resolution | 10 | 2.5% | 18.8% | 70% |
| event_ordering | 10 | 5.0% | 33.8% | 0% |
| information_extraction | 10 | 14.2% | 40.4% | 70% |
| instruction_following | 10 | 30.0% | 40.0% | 30% |
| knowledge_update | 10 | 5.0% | 25.0% | 90% |
| multi_session_reasoning | 10 | 15.0% | 25.0% | 30% |
| preference_following | 10 | 22.5% | 41.0% | 30% |
| summarization | 10 | 21.7% | 33.6% | 0% |
| temporal_reasoning | 10 | 10.0% | 30.0% | 90% |
| **overall** | 100 | **20.6%** | **37.7%** | 46.6% |

All 400 with the Haiku judge: **20.7%**. The Haiku judge is not usable for BEAM:
- BEAM's judge prompt does not include the question, and its answer prompt asks for the bare answer. Haiku scored
  38 correct bare answers ("21 days", "Yes.") as 0 because "it is unclear what it answers".
- 52 of 1,027 Haiku judge replies give `"score": 0.0` while their own reason says the required element is present
  ("…should score as complete"). The score is written before the reasoning.

With Sonnet, no bare answers are marked down (`terse_judged` = 0). **Use a stronger judge than Haiku for BEAM** (the
BEAM authors use GPT-4-class judges); Haiku-judged BEAM numbers should not be reported.

## Where the points are lost (Sonnet judge, 100 questions)

| cause | questions | meaning |
|---|---|---|
| ok (score 1) | 21 | |
| retrieval_partial | 27 | some evidence turns in context, others not |
| retrieval_miss | 17 | no evidence turn in context |
| answer_or_judge | 19 | all evidence in context, answer incomplete or wrong |
| answer_missed | 13 | all evidence in context, but the answer said "I can't determine…" |
| other | 3 | hallucinated abstention, no evidence ids |

Retrieval accounts for 44 questions, the answer step for 32.

## Key insights for rewire

1. **Standing instructions and preferences must not depend on recall** (instruction_following: 22/40 had *no*
   evidence retrieved; preference_following: 16/40). An instruction stated once ("always show dates as MM/DD/YYYY")
   shares no words with later questions. rewire already has the right mechanism, `constraint`/`preference` nodes that
   `prime` injects in every session and that never decay, but `--mode episodes` bypasses it. Next: a BEAM
   `--mode full` (or extraction of instructions/preferences only) on a few chats to measure the gain.
2. **Summarization and event ordering need the whole thread**, not 10 excerpts (0–2.5% of questions have all evidence).
   Add a recall intent for "summarize / in what order / timeline" that returns per-session summaries
   (`sessions/*.md`) ordered by date, one per session, instead of raw excerpts.
3. **The answer over-abstains**: 71 of 360 non-abstention answers say "I can't determine…", 13 of the 100 with all
   evidence present. BEAM's RAG prompt ("ONLY the context") makes Haiku cautious when the value appears in a long
   excerpt among 10. Higher rank for the evidence excerpt (MRR is 0.11–0.43 in the weak categories) and shorter
   excerpts (long assistant turns split) would help; so would a stronger answerer.
4. **Contradiction resolution asks the memory to flag conflicts**: the gold answer is "you said X, but also said
   never X; which is right?". Both statements are retrieved in 70% of cases, but nothing marks them as conflicting.
   Consolidation could record a `conflicts_with` link when a new claim contradicts an active one, and recall would
   show both with a label.
5. **Knowledge update and temporal reasoning are retrieval-solved (90% all evidence) but answer-limited** (25–30%):
   old and new values arrive together without a clear "newest" marker, and day counts are miscomputed. Dates on every
   excerpt and the supersede mechanism (old value `valid_to`) target exactly this.
6. **Scale**: at 500K/1M, all-evidence drops from 44.8% to 30.5% (retrieval-only), with compression ≈72×. The
   insights above matter more at larger sizes.

## Follow-up: what was implemented and measured (2026-10-16)

All runs: answer `claude-cli/haiku`, judge `claude-cli/sonnet`, keyword recall, ≈12k context tokens per question.

| step | change | questions | before | after | paired better / worse |
|---|---|---|---|---|---|
| 1–2 | excerpts capped at 1,200 chars as well as 6 turns (index schema 10), 40 excerpts | 100 (10 per category) | 37.7% (all evidence 46.6%) | **40.4%** (53.4%) | 24 / 30 |
| 3 | `--mode full` (Haiku consolidates every session, inbox accepted) vs `episodes` | 80 (chats 1, 7, 13, 19) | 36.5% (all evidence 52.8%) | **43.0%** (59.7%) | 26 / 17 |

Files: [chunks run](2026-10-16-beam-100k-s10-chunks.md) ([diagnosis](2026-10-16-beam-100k-s10-chunks-diagnosis.md)) ·
[episodes, 4 chats](2026-10-16-beam-100k-ep-4chats.md) ([diagnosis](2026-10-16-beam-100k-ep-4chats-diagnosis.md)) ·
[full, 4 chats](2026-10-16-beam-100k-full-4chats.md) ([diagnosis](2026-10-16-beam-100k-full-4chats-diagnosis.md)).

Per category, `full` vs `episodes` (n = 8 each, so only large moves mean anything): preference following
43.8 → 68.8%, temporal 18.8 → 37.5%, instruction following 21.9 → 34.4%, contradiction 20.3 → 32.8%,
knowledge update 50.0 → 62.5%; abstention 75.0 → 62.5% (extracted facts tempt the answer to fill gaps, e.g.
"It was a great success, with 12 attendees" for a detail the chat never gives); summarization and event ordering
unchanged (18–30%). Ingest: 18 sessions, 36 facts per session, ≈71 s per session.

What this says:
- **Small excerpts trade retrieval for answer confidence.** With 40 short excerpts, Haiku more often refuses
  ("the excerpts are fragmentary"): retrieval losses 44 → 37 questions, answer losses 32 → 40. Next: present
  excerpts of the same session merged and in turn order, so the answer reads a conversation, not fragments.
- **Consolidation is the larger lever** (+6.5 points): facts and preferences extracted per session reach the
  context where raw excerpts do not. This is rewire's real mode; `episodes` understates it.
- **Instruction following is still low with facts** (34%): in the benchmark the instruction only reaches the
  answer through recall. In real use `prime` injects constraints and preferences at session start; the eval does
  not yet include the `prime` snapshot in the answer context, which is the next thing to measure.
- Summarization and event ordering need the per-session summary intent; no change here touched them.
