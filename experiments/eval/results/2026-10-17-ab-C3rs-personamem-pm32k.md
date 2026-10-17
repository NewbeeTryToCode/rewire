# personamem · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | 10 | 30.0% | - | - | - | - | 11,413 / 12,661 | 15,335 / 544 | 102 / 168 |
| provide_preference_aligned_recommendations | 10 | 60.0% | - | - | - | - | 11,803 / 12,331 | 16,116 / 579 | 104 / 141 |
| recall_user_shared_facts | 10 | 50.0% | - | - | - | - | 11,801 / 16,851 | 15,469 / 432 | 100 / 110 |
| recalling_facts_mentioned_by_the_user | 10 | 50.0% | - | - | - | - | 13,985 / 23,819 | 18,518 / 523 | 103 / 114 |
| recalling_the_reasons_behind_previous_updates | 10 | 90.0% | - | - | - | - | 12,278 / 18,506 | 16,601 / 492 | 112 / 128 |
| suggest_new_ideas | 10 | 30.0% | - | - | - | - | 11,311 / 12,483 | 15,452 / 568 | 103 / 127 |
| track_full_preference_evolution | 10 | 80.0% | - | - | - | - | 13,089 / 22,348 | 17,740 / 750 | 115 / 123 |
| overall | 70 | 55.7% | - | - | - | - | 12,240 / 20,819 | 16,462 / 555 | 104 / 128 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | - | - | - | 3.3× | 168 | 50 |
| provide_preference_aligned_recommendations | - | - | - | 3.0× | 141 | 66 |
| recall_user_shared_facts | - | - | - | 3.1× | 110 | 55 |
| recalling_facts_mentioned_by_the_user | - | - | - | 2.6× | 114 | 56 |
| recalling_the_reasons_behind_previous_updates | - | - | - | 2.8× | 128 | 53 |
| suggest_new_ideas | - | - | - | 3.2× | 127 | 53 |
| track_full_preference_evolution | - | - | - | 2.6× | 123 | 67 |
| overall | - | - | - | 2.9× | 168 | 57 |

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|

Retrieval tokens per question: context 12,240 (p95 20,819); extra-query LLM calls 1,741 prompt + 100 completion on average over all questions, 1,688 + 349 on the 4 questions given extra queries.

Per question, answer + judge, including cached replies: 16,462 prompt tokens, 555 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
