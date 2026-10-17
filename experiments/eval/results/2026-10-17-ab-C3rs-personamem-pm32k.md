# personamem · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | 10 | 30.0% | - | - | - | - | 11,413 / 12,661 | 15,335 / 544 | 123 / 262 |
| provide_preference_aligned_recommendations | 10 | 60.0% | - | - | - | - | 11,803 / 12,331 | 16,116 / 579 | 142 / 166 |
| recall_user_shared_facts | 10 | 50.0% | - | - | - | - | 11,801 / 16,851 | 15,469 / 432 | 120 / 139 |
| recalling_facts_mentioned_by_the_user | 10 | 50.0% | - | - | - | - | 13,985 / 23,819 | 18,518 / 523 | 111 / 142 |
| recalling_the_reasons_behind_previous_updates | 10 | 90.0% | - | - | - | - | 12,278 / 18,506 | 16,601 / 492 | 124 / 167 |
| suggest_new_ideas | 10 | 30.0% | - | - | - | - | 11,311 / 12,483 | 15,452 / 568 | 123 / 193 |
| track_full_preference_evolution | 10 | 80.0% | - | - | - | - | 13,089 / 22,348 | 17,740 / 750 | 115 / 142 |
| overall | 70 | 55.7% | - | - | - | - | 12,240 / 20,819 | 16,462 / 555 | 116 / 166 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | - | - | - | 3.3× | 262 | 50 |
| provide_preference_aligned_recommendations | - | - | - | 3.0× | 166 | 66 |
| recall_user_shared_facts | - | - | - | 3.1× | 139 | 55 |
| recalling_facts_mentioned_by_the_user | - | - | - | 2.6× | 142 | 56 |
| recalling_the_reasons_behind_previous_updates | - | - | - | 2.8× | 167 | 53 |
| suggest_new_ideas | - | - | - | 3.2× | 193 | 53 |
| track_full_preference_evolution | - | - | - | 2.6× | 142 | 67 |
| overall | - | - | - | 2.9× | 262 | 57 |

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| expand | 68 | 118,461 | 6,831 | 6,027 | 1,742 / 100 | 0.0000 | 5826 / 7023 |
| answer | 4 | 102,518 | 1,856 | 1,368 | 25,630 / 464 | 0.0000 | 6855 / 7773 |

Per question, answer + judge, including cached replies: 16,462 prompt tokens, 555 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
