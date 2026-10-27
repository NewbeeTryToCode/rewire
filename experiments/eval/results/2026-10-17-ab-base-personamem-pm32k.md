# personamem · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | 10 | 50.0% | - | - | - | - | 11,158 / 12,490 | 15,042 / 517 | 40 / 99 |
| provide_preference_aligned_recommendations | 10 | 70.0% | - | - | - | - | 11,463 / 12,348 | 15,705 / 545 | 20 / 45 |
| recall_user_shared_facts | 10 | 50.0% | - | - | - | - | 11,110 / 12,045 | 14,664 / 411 | 13 / 16 |
| recalling_facts_mentioned_by_the_user | 10 | 30.0% | - | - | - | - | 11,302 / 12,724 | 15,280 / 562 | 17 / 22 |
| recalling_the_reasons_behind_previous_updates | 10 | 90.0% | - | - | - | - | 10,367 / 11,548 | 14,348 / 503 | 45 / 95 |
| suggest_new_ideas | 10 | 30.0% | - | - | - | - | 11,326 / 12,421 | 15,453 / 603 | 18 / 26 |
| track_full_preference_evolution | 10 | 80.0% | - | - | - | - | 9,846 / 11,778 | 13,915 / 763 | 56 / 95 |
| overall | 70 | 57.1% | - | - | - | - | 10,939 / 12,362 | 14,915 / 558 | 20 / 89 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | - | - | - | 3.4× | 99 | 51 |
| provide_preference_aligned_recommendations | - | - | - | 3.1× | 45 | 64 |
| recall_user_shared_facts | - | - | - | 3.3× | 16 | 47 |
| recalling_facts_mentioned_by_the_user | - | - | - | 3.2× | 22 | 70 |
| recalling_the_reasons_behind_previous_updates | - | - | - | 3.4× | 95 | 64 |
| suggest_new_ideas | - | - | - | 3.2× | 26 | 64 |
| track_full_preference_evolution | - | - | - | 3.4× | 95 | 66 |
| overall | - | - | - | 3.3× | 99 | 61 |

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 70 | 1,044,071 | 39,047 | 29,774 | 14,915 / 558 | 0.0000 | 7449 / 8838 |

Per question, answer + judge, including cached replies: 14,915 prompt tokens, 558 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
