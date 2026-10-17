# personamem · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | 10 | 30.0% | - | - | - | - | 11,413 / 12,661 | 15,335 / 544 | 45 / 71 |
| provide_preference_aligned_recommendations | 10 | 60.0% | - | - | - | - | 11,803 / 12,331 | 16,116 / 579 | 85 / 116 |
| recall_user_shared_facts | 10 | 40.0% | - | - | - | - | 12,511 / 19,022 | 16,338 / 464 | 37 / 214 |
| recalling_facts_mentioned_by_the_user | 10 | 40.0% | - | - | - | - | 11,233 / 12,473 | 15,184 / 536 | 57 / 165 |
| recalling_the_reasons_behind_previous_updates | 10 | 90.0% | - | - | - | - | 15,953 / 34,301 | 20,960 / 482 | 162 / 584 |
| suggest_new_ideas | 10 | 30.0% | - | - | - | - | 11,311 / 12,483 | 15,452 / 568 | 90 / 198 |
| track_full_preference_evolution | 10 | 60.0% | - | - | - | - | 19,806 / 39,324 | 25,693 / 803 | 500 / 684 |
| overall | 70 | 50.0% | - | - | - | - | 13,433 / 34,301 | 17,868 / 568 | 81 / 584 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | - | - | - | 3.3× | 71 | 50 |
| provide_preference_aligned_recommendations | - | - | - | 3.0× | 116 | 66 |
| recall_user_shared_facts | - | - | - | 2.9× | 214 | 55 |
| recalling_facts_mentioned_by_the_user | - | - | - | 3.2× | 165 | 56 |
| recalling_the_reasons_behind_previous_updates | - | - | - | 2.2× | 584 | 55 |
| suggest_new_ideas | - | - | - | 3.2× | 198 | 53 |
| track_full_preference_evolution | - | - | - | 1.7× | 684 | 67 |
| overall | - | - | - | 2.7× | 684 | 57 |

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|

Per question, answer + judge, including cached replies: 17,868 prompt tokens, 568 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
