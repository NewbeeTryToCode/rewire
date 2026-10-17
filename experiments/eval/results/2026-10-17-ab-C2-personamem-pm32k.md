# personamem · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | 10 | 30.0% | - | - | - | - | 11,413 / 12,661 | 15,335 / 544 | 109 / 148 |
| provide_preference_aligned_recommendations | 10 | 60.0% | - | - | - | - | 11,803 / 12,331 | 16,116 / 579 | 112 / 181 |
| recall_user_shared_facts | 10 | 50.0% | - | - | - | - | 11,209 / 12,045 | 14,772 / 426 | 28 / 167 |
| recalling_facts_mentioned_by_the_user | 10 | 40.0% | - | - | - | - | 11,233 / 12,473 | 15,184 / 536 | 39 / 78 |
| recalling_the_reasons_behind_previous_updates | 10 | 90.0% | - | - | - | - | 12,278 / 18,506 | 16,601 / 492 | 146 / 271 |
| suggest_new_ideas | 10 | 30.0% | - | - | - | - | 11,311 / 12,483 | 15,452 / 568 | 96 / 210 |
| track_full_preference_evolution | 10 | 80.0% | - | - | - | - | 13,089 / 22,348 | 17,740 / 750 | 242 / 422 |
| overall | 70 | 54.3% | - | - | - | - | 11,762 / 18,506 | 15,886 / 556 | 96 / 271 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | - | - | - | 3.3× | 148 | 50 |
| provide_preference_aligned_recommendations | - | - | - | 3.0× | 181 | 66 |
| recall_user_shared_facts | - | - | - | 3.3× | 167 | 55 |
| recalling_facts_mentioned_by_the_user | - | - | - | 3.2× | 78 | 56 |
| recalling_the_reasons_behind_previous_updates | - | - | - | 2.8× | 271 | 53 |
| suggest_new_ideas | - | - | - | 3.2× | 210 | 53 |
| track_full_preference_evolution | - | - | - | 2.6× | 422 | 67 |
| overall | - | - | - | 3.1× | 422 | 57 |

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|

Per question, answer + judge, including cached replies: 15,886 prompt tokens, 556 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
