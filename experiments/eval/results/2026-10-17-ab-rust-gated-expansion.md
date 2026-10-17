# A/B on the Rust engine with C2 as the baseline: model-gated extra queries (C3) and rule-or-model (C4)

Run of 2026-10-17. Recall runs in the Rust binary (`--engine rust`, `rust/target/release/rewire recall --json`,
semantic on with the WordLlama model from `rewire semantic install`); Python only writes the session files.
Same 190 questions as the earlier A/B runs (LoCoMo 40, LongMemEval_S 60, BEAM 100K 20, PersonaMem 32k 70; seed 0)
plus 40 more LoCoMo multi-hop questions (seed 1), because multi-hop is where the C2 rule rarely fires.
Answerer `claude-haiku-5-5`, judge `claude-haiku-4-5`, extra queries by `claude-haiku-5-5`.

| arm | extra queries |
|---|---|
| **C2** (baseline) | only for questions a rule marks as counting / total / ordering / time (`--expand-when gather`) |
| **C3** | the expansion model decides, and may write none (`--expand-when model`) |
| **C4** | the model decides; when it writes none and the rule fires, the rule's queries are used (`--expand-when union`) |

**Rust = Python.** Rust C2 gives the same verdict as Python C2 on all 190 questions (72.2% both; 189/190 identical
context sizes), so the Rust port reproduces the measured results end to end.

**Latency** is the wall time of one `rewire recall` process minus a fixed 96 ms (process start + WordLlama load,
measured on an empty brain; 6 ms without semantic). A long-running MCP server loads the model once, so the
numbers below are closer to what it sees.

## Summary

| benchmark | n | J C2 / C3 / C4 | all evidence C2 / C3 / C4 | evidence turns C2 / C3 / C4 | recall p50 C2 / C3 / C4 |
|---|---|---|---|---|---|
| locomo | 40 | 90% / 90% / 90% | 72% / 78% / 78% | 71% / 73% / 73% | 14 / 16 / 14 ms |
| longmemeval | 60 | 88% / 90% / 90% | 98% / 97% / 97% | 97% / 97% / 97% | 156 / 165 / 157 ms |
| beam | 20 | 51% / 37% / 42% | 59% / 53% / 59% | 63% / 63% / 65% | 73 / 142 / 147 ms |
| personamem | 70 | 54% / 56% / 56% | - / - / - | - / - / - | 10 / 20 / 14 ms |
| locomo — extra multi-hop sample (seed 1) | 40 | 92% / 95% / 95% | 32% / 50% / 50% | 59% / 71% / 71% | 13 / 22 / 7 ms |
| **all 190** (without the extra sample) | 190 | 72.2% / 71.8% / 72.3% | | | |


How often extra queries are written (questions with non-empty `also`):

| benchmark | n | C2 (rule) | C3 (model) | both |
|---|---|---|---|---|
| LoCoMo | 40 | 13 | 22 | 9 |
| LoCoMo multi-hop (extra) | 40 | 7 | 30 | 7 |
| LongMemEval_S | 60 | 30 | 26 | 23 |
| BEAM 100K | 20 | 6 | 12 | 5 |
| PersonaMem 32k | 70 | 0 | 4 | 0 |

## Reading

- **Overall J is a tie**: 72.2% (C2), 71.8% (C3), 72.3% (C4) on the 190 questions; on all 230 with the extra
  multi-hop sample, C2 75.7% and C4 76.3%. No arm is clearly better at this sample size (±6 points).
- **The model finds the list questions the rule misses**: on LoCoMo multi-hop it writes extra queries for 30/40
  questions (the rule: 7), which lifts *all evidence* from 32% to 50% and evidence turns from 59% to 71%, and J
  92% → 95% (2 better, 1 worse). This is the retrieval gain the earlier LoCoMo A/B found for extra queries.
- **BEAM goes the other way** (51% → 37% C3, 42% C4; 20 questions, rubric scores): the model declined
  "How many words does my final essay draft contain?" (right in C2 thanks to the rule's queries; C4 restores it),
  and wrote different queries for the "how many scenes …" and "how many days after …" questions, which then came
  out worse. With two questions per category, this is the noisiest benchmark here.
- **PersonaMem** stays safe under the model gate: it expands 4 of 70 (statements and advice requests are skipped)
  and gains one question.
- **Latency** (Rust, after the CLI overhead): p50 7–165 ms in every arm; C3/C4 add 0–75 ms p50 where they expand
  more (BEAM 73 → 142–147 ms).

## Recommendation

- Keep **C2 as the eval baseline** (a deterministic rule, same J, cheapest).
- For agents using rewire through MCP, the model-gated behaviour (C3/C4) is the realistic one: the agent decides
  whether to fill `also`. The `also` guide already names lists and categories next to counts and time, which is what
  lifted LoCoMo multi-hop evidence here. C4 (model or rule) is the safest choice when a harness can do both.
- What would move J now is not more recall: LongMemEval has 97–98% all evidence in every arm with J 88–90%; the
  remaining misses are answering (counting slips, self-contradicting answers) and BEAM's retrieval-heavy categories
  (information extraction 0% evidence in every arm).

## Per-benchmark tables

### locomo

| category | n | J C2 / C3 / C4 | all evidence C2 / C3 / C4 | evidence turns C2 / C3 / C4 | recall p50 C2 / C3 / C4 | recall p95 C2 / C3 / C4 |
|---|---|---|---|---|---|---|
| multi-hop | 10 | 90% / 100% / 100% | 60% / 70% / 70% | 72% / 75% / 75% | 15 / 24 / 24 ms | 48 / 59 / 38 ms |
| open-domain | 10 | 80% / 70% / 70% | 50% / 60% / 60% | 53% / 56% / 56% | 14 / 16 / 16 ms | 28 / 65 / 57 ms |
| single-hop | 10 | 100% / 100% / 100% | 100% / 100% / 100% | 100% / 100% / 100% | 11 / 16 / 14 ms | 21 / 35 / 49 ms |
| temporal | 10 | 90% / 90% / 90% | 80% / 80% / 80% | 87% / 87% / 87% | 36 / 23 / 11 ms | 59 / 47 / 50 ms |
| **overall** | 40 | 90% / 90% / 90% | 72% / 78% / 78% | 71% / 73% / 73% | 14 / 16 / 14 ms | 57 / 59 / 50 ms |
C3 vs C2 per question: 1 better, 1 worse
C4 vs C2 per question: 1 better, 1 worse

### longmemeval

| category | n | J C2 / C3 / C4 | all evidence C2 / C3 / C4 | evidence turns C2 / C3 / C4 | recall p50 C2 / C3 / C4 | recall p95 C2 / C3 / C4 |
|---|---|---|---|---|---|---|
| knowledge-update | 10 | 100% / 100% / 100% | 100% / 100% / 100% | 100% / 100% / 100% | 190 / 167 / 180 ms | 349 / 300 / 294 ms |
| multi-session | 10 | 70% / 80% / 80% | 100% / 100% / 100% | 100% / 100% / 100% | 236 / 215 / 211 ms | 283 / 306 / 237 ms |
| single-session-assistant | 10 | 100% / 100% / 100% | 100% / 100% / 100% | 100% / 100% / 100% | 175 / 163 / 162 ms | 441 / 231 / 222 ms |
| single-session-preference | 10 | 60% / 60% / 60% | 90% / 90% / 90% | 80% / 80% / 80% | 133 / 136 / 120 ms | 211 / 328 / 141 ms |
| single-session-user | 10 | 100% / 100% / 100% | 100% / 100% / 100% | 100% / 100% / 100% | 114 / 154 / 118 ms | 163 / 190 / 190 ms |
| temporal-reasoning | 10 | 100% / 100% / 100% | 100% / 89% / 89% | 100% / 100% / 100% | 207 / 227 / 232 ms | 299 / 285 / 301 ms |
| **overall** | 60 | 88% / 90% / 90% | 98% / 97% / 97% | 97% / 97% / 97% | 156 / 165 / 157 ms | 299 / 300 / 267 ms |
C3 vs C2 per question: 1 better, 0 worse
C4 vs C2 per question: 1 better, 0 worse

### beam

| category | n | J C2 / C3 / C4 | all evidence C2 / C3 / C4 | evidence turns C2 / C3 / C4 | recall p50 C2 / C3 / C4 | recall p95 C2 / C3 / C4 |
|---|---|---|---|---|---|---|
| abstention | 2 | 100% / 100% / 100% | - / - / - | - / - / - | 73 / 148 / 171 ms | 73 / 148 / 171 ms |
| contradiction_resolution | 2 | 25% / 19% / 19% | 100% / 100% / 100% | 100% / 100% / 100% | 53 / 101 / 147 ms | 53 / 101 / 147 ms |
| event_ordering | 2 | 66% / 38% / 38% | 0% / 0% / 0% | 31% / 46% / 46% | 86 / 354 / 227 ms | 86 / 354 / 227 ms |
| information_extraction | 2 | 25% / 25% / 25% | 0% / 0% / 0% | 0% / 0% / 0% | 78 / 176 / 139 ms | 78 / 176 / 139 ms |
| instruction_following | 2 | 50% / 50% / 50% | 50% / 50% / 50% | 50% / 25% / 25% | 175 / 129 / 127 ms | 175 / 129 / 127 ms |
| knowledge_update | 2 | 50% / 0% / 50% | 100% / 50% / 100% | 100% / 75% / 100% | 260 / 81 / 269 ms | 260 / 81 / 269 ms |
| multi_session_reasoning | 2 | 50% / 0% / 0% | 100% / 100% / 100% | 100% / 100% / 100% | 306 / 240 / 235 ms | 306 / 240 / 235 ms |
| preference_following | 2 | 50% / 50% / 50% | 50% / 50% / 50% | 50% / 50% / 50% | 49 / 50 / 48 ms | 49 / 50 / 48 ms |
| summarization | 2 | 45% / 65% / 65% | 0% / 0% / 0% | 60% / 60% / 60% | 62 / 240 / 231 ms | 62 / 240 / 231 ms |
| temporal_reasoning | 2 | 50% / 25% / 25% | 100% / 100% / 100% | 100% / 100% / 100% | 404 / 410 / 549 ms | 404 / 410 / 549 ms |
| **overall** | 20 | 51% / 37% / 42% | 59% / 53% / 59% | 63% / 63% / 65% | 73 / 142 / 147 ms | 404 / 410 / 549 ms |
C3 vs C2 per question: 3 better, 5 worse
C4 vs C2 per question: 3 better, 4 worse

### personamem

| category | n | J C2 / C3 / C4 | all evidence C2 / C3 / C4 | evidence turns C2 / C3 / C4 | recall p50 C2 / C3 / C4 | recall p95 C2 / C3 / C4 |
|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | 10 | 30% / 30% / 30% | - / - / - | - / - / - | 5 / 27 / 17 ms | 15 / 166 / 30 ms |
| provide_preference_aligned_recommendations | 10 | 60% / 60% / 60% | - / - / - | - / - / - | 15 / 46 / 10 ms | 27 / 70 / 30 ms |
| recall_user_shared_facts | 10 | 50% / 50% / 50% | - / - / - | - / - / - | 13 / 24 / 8 ms | 32 / 43 / 26 ms |
| recalling_facts_mentioned_by_the_user | 10 | 40% / 50% / 50% | - / - / - | - / - / - | 11 / 15 / 15 ms | 35 / 46 / 51 ms |
| recalling_the_reasons_behind_previous_updates | 10 | 90% / 90% / 90% | - / - / - | - / - / - | 12 / 28 / 15 ms | 40 / 71 / 32 ms |
| suggest_new_ideas | 10 | 30% / 30% / 30% | - / - / - | - / - / - | 5 / 27 / 19 ms | 50 / 97 / 58 ms |
| track_full_preference_evolution | 10 | 80% / 80% / 80% | - / - / - | - / - / - | 14 / 19 / 21 ms | 44 / 46 / 38 ms |
| **overall** | 70 | 54% / 56% / 56% | - / - / - | - / - / - | 10 / 20 / 14 ms | 35 / 70 / 38 ms |
C3 vs C2 per question: 1 better, 0 worse
C4 vs C2 per question: 1 better, 0 worse

### locomo — extra multi-hop sample (seed 1)

| category | n | J C2 / C3 / C4 | all evidence C2 / C3 / C4 | evidence turns C2 / C3 / C4 | recall p50 C2 / C3 / C4 | recall p95 C2 / C3 / C4 |
|---|---|---|---|---|---|---|
| multi-hop | 40 | 92% / 95% / 95% | 32% / 50% / 50% | 59% / 71% / 71% | 13 / 22 / 7 ms | 52 / 61 / 41 ms |
| **overall** | 40 | 92% / 95% / 95% | 32% / 50% / 50% | 59% / 71% / 71% | 13 / 22 / 7 ms | 52 / 61 / 41 ms |
C3 vs C2 per question: 2 better, 1 worse
C4 vs C2 per question: 2 better, 1 worse
