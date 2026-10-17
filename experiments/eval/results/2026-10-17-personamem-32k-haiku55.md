# personamem · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | 57 | 45.6% | - | - | - | - | 11,388 / 12,666 | 15,318 / 472 | 29 / 60 |
| provide_preference_aligned_recommendations | 55 | 58.2% | - | - | - | - | 11,728 / 12,681 | 16,043 / 514 | 32 / 113 |
| recall_user_shared_facts | 129 | 70.5% | - | - | - | - | 10,994 / 12,436 | 14,621 / 464 | 23 / 76 |
| recalling_facts_mentioned_by_the_user | 17 | 41.2% | - | - | - | - | 11,329 / 12,491 | 15,265 / 525 | 35 / 185 |
| recalling_the_reasons_behind_previous_updates | 99 | 80.8% | - | - | - | - | 11,326 / 18,506 | 15,411 / 503 | 66 / 219 |
| suggest_new_ideas | 93 | 25.8% | - | - | - | - | 11,420 / 12,625 | 15,505 / 569 | 29 / 84 |
| track_full_preference_evolution | 139 | 64.0% | - | - | - | - | 11,120 / 19,733 | 15,356 / 657 | 90 / 229 |
| overall | 589 | 59.3% | - | - | - | - | 11,263 / 13,522 | 15,285 / 540 | 42 / 181 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | - | - | - | 3.1× | 95 | 51 |
| provide_preference_aligned_recommendations | - | - | - | 3.1× | 174 | 60 |
| recall_user_shared_facts | - | - | - | 3.3× | 101 | 56 |
| recalling_facts_mentioned_by_the_user | - | - | - | 3.1× | 185 | 59 |
| recalling_the_reasons_behind_previous_updates | - | - | - | 3.0× | 438 | 63 |
| suggest_new_ideas | - | - | - | 3.2× | 256 | 62 |
| track_full_preference_evolution | - | - | - | 2.9× | 365 | 60 |
| overall | - | - | - | 3.1× | 277 | 59 |

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 586 | 8,959,309 | 316,165 | 240,594 | 15,289 / 540 | 0.0000 | 8173 / 10811 |

Per question, answer + judge, including cached replies: 15,285 prompt tokens, 540 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
