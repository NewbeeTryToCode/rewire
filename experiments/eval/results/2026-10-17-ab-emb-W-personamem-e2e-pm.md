# personamem · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | 10 | 30.0% | - | - | - | - | 11,413 / 12,661 | 15,335 / 544 | 22 / 34 |
| provide_preference_aligned_recommendations | 10 | 60.0% | - | - | - | - | 11,803 / 12,331 | 16,116 / 579 | 23 / 65 |
| recall_user_shared_facts | 10 | 50.0% | - | - | - | - | 11,209 / 12,045 | 14,772 / 426 | 25 / 94 |
| recalling_facts_mentioned_by_the_user | 10 | 40.0% | - | - | - | - | 11,233 / 12,473 | 15,184 / 536 | 32 / 79 |
| recalling_the_reasons_behind_previous_updates | 10 | 90.0% | - | - | - | - | 12,278 / 18,506 | 16,601 / 492 | 31 / 79 |
| suggest_new_ideas | 10 | 30.0% | - | - | - | - | 11,311 / 12,483 | 15,452 / 568 | 42 / 69 |
| track_full_preference_evolution | 10 | 80.0% | - | - | - | - | 13,089 / 22,348 | 17,740 / 750 | 68 / 115 |
| overall | 70 | 54.3% | - | - | - | - | 11,762 / 18,506 | 15,886 / 556 | 27 / 94 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | - | - | - | 3.3× | 34 | 50 |
| provide_preference_aligned_recommendations | - | - | - | 3.0× | 65 | 66 |
| recall_user_shared_facts | - | - | - | 3.3× | 94 | 55 |
| recalling_facts_mentioned_by_the_user | - | - | - | 3.2× | 79 | 56 |
| recalling_the_reasons_behind_previous_updates | - | - | - | 2.8× | 79 | 53 |
| suggest_new_ideas | - | - | - | 3.2× | 69 | 53 |
| track_full_preference_evolution | - | - | - | 2.6× | 115 | 67 |
| overall | - | - | - | 3.1× | 115 | 57 |

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|

Retrieval tokens per question: context 11,762 (p95 18,506); extra-query LLM calls 0 prompt + 0 completion on average over all questions, 0 + 0 on the 0 questions given extra queries.

Per question, answer + judge, including cached replies: 15,886 prompt tokens, 556 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
