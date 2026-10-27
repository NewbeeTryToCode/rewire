# personamem · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | 10 | 30.0% | - | - | - | - | 17,658 / 20,198 | 22,753 / 503 | 268 / 360 |
| provide_preference_aligned_recommendations | 10 | 70.0% | - | - | - | - | 19,962 / 26,242 | 25,814 / 527 | 49 / 67 |
| recall_user_shared_facts | 10 | 50.0% | - | - | - | - | 18,702 / 21,680 | 23,633 / 471 | 47 / 61 |
| recalling_facts_mentioned_by_the_user | 10 | 50.0% | - | - | - | - | 20,661 / 24,178 | 26,321 / 518 | 48 / 56 |
| recalling_the_reasons_behind_previous_updates | 10 | 70.0% | - | - | - | - | 20,192 / 34,301 | 25,992 / 528 | 73 / 179 |
| suggest_new_ideas | 10 | 0.0% | - | - | - | - | 17,764 / 21,279 | 23,070 / 608 | 44 / 71 |
| track_full_preference_evolution | 10 | 60.0% | - | - | - | - | 21,640 / 39,324 | 27,855 / 850 | 81 / 220 |
| overall | 70 | 47.1% | - | - | - | - | 19,511 / 34,301 | 25,063 / 572 | 52 / 310 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | - | - | - | 2.2× | 360 | 52 |
| provide_preference_aligned_recommendations | - | - | - | 1.8× | 67 | 53 |
| recall_user_shared_facts | - | - | - | 2.0× | 61 | 59 |
| recalling_facts_mentioned_by_the_user | - | - | - | 1.8× | 56 | 54 |
| recalling_the_reasons_behind_previous_updates | - | - | - | 1.7× | 179 | 55 |
| suggest_new_ideas | - | - | - | 2.1× | 71 | 60 |
| track_full_preference_evolution | - | - | - | 1.6× | 220 | 61 |
| overall | - | - | - | 1.8× | 360 | 56 |

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| expand | 68 | 110,805 | 4,873 | 129 | 1,629 / 72 | 0.0000 | 4930 / 5740 |
| answer | 70 | 1,754,377 | 40,051 | 31,313 | 25,063 / 572 | 0.0000 | 7436 / 10105 |

Per question, answer + judge, including cached replies: 25,063 prompt tokens, 572 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
