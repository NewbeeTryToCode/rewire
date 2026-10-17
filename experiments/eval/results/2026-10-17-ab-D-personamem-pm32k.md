# personamem · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | 10 | 40.0% | - | - | - | - | 11,419 / 12,667 | 15,343 / 501 | 54 / 77 |
| provide_preference_aligned_recommendations | 10 | 50.0% | - | - | - | - | 11,809 / 12,337 | 16,125 / 587 | 22 / 85 |
| recall_user_shared_facts | 10 | 40.0% | - | - | - | - | 12,517 / 19,028 | 16,347 / 451 | 15 / 64 |
| recalling_facts_mentioned_by_the_user | 10 | 50.0% | - | - | - | - | 11,239 / 12,479 | 15,192 / 513 | 17 / 29 |
| recalling_the_reasons_behind_previous_updates | 10 | 80.0% | - | - | - | - | 15,959 / 34,307 | 20,969 / 549 | 43 / 144 |
| suggest_new_ideas | 10 | 10.0% | - | - | - | - | 11,317 / 12,489 | 15,459 / 609 | 18 / 61 |
| track_full_preference_evolution | 10 | 90.0% | - | - | - | - | 19,812 / 39,330 | 25,702 / 705 | 76 / 200 |
| overall | 70 | 51.4% | - | - | - | - | 13,439 / 34,307 | 17,877 / 559 | 22 / 144 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | - | - | - | 3.3× | 77 | 51 |
| provide_preference_aligned_recommendations | - | - | - | 3.0× | 85 | 63 |
| recall_user_shared_facts | - | - | - | 2.9× | 64 | 54 |
| recalling_facts_mentioned_by_the_user | - | - | - | 3.2× | 29 | 64 |
| recalling_the_reasons_behind_previous_updates | - | - | - | 2.2× | 144 | 56 |
| suggest_new_ideas | - | - | - | 3.2× | 61 | 52 |
| track_full_preference_evolution | - | - | - | 1.7× | 200 | 68 |
| overall | - | - | - | 2.7× | 200 | 58 |

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 70 | 1,251,361 | 39,143 | 30,141 | 17,877 / 559 | 0.0000 | 7472 / 9122 |

Per question, answer + judge, including cached replies: 17,877 prompt tokens, 559 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
