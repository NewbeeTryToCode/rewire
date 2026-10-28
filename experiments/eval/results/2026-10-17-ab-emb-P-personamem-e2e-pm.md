# personamem · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | 10 | 50.0% | - | - | - | - | 11,331 / 12,727 | 15,242 / 524 | 14 / 23 |
| provide_preference_aligned_recommendations | 10 | 60.0% | - | - | - | - | 11,699 / 13,179 | 15,953 / 531 | 14 / 27 |
| recall_user_shared_facts | 10 | 50.0% | - | - | - | - | 11,064 / 12,283 | 14,602 / 439 | 14 / 18 |
| recalling_facts_mentioned_by_the_user | 10 | 40.0% | - | - | - | - | 11,175 / 12,473 | 15,096 / 547 | 14 / 22 |
| recalling_the_reasons_behind_previous_updates | 10 | 90.0% | - | - | - | - | 12,199 / 18,199 | 16,504 / 566 | 23 / 42 |
| suggest_new_ideas | 10 | 0.0% | - | - | - | - | 11,409 / 12,626 | 15,558 / 637 | 16 / 30 |
| track_full_preference_evolution | 10 | 80.0% | - | - | - | - | 13,043 / 22,210 | 17,694 / 830 | 30 / 64 |
| overall | 70 | 52.9% | - | - | - | - | 11,703 / 18,199 | 15,807 / 582 | 16 / 42 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | - | - | - | 3.4× | 23 | 52 |
| provide_preference_aligned_recommendations | - | - | - | 3.1× | 27 | 61 |
| recall_user_shared_facts | - | - | - | 3.3× | 18 | 50 |
| recalling_facts_mentioned_by_the_user | - | - | - | 3.3× | 22 | 61 |
| recalling_the_reasons_behind_previous_updates | - | - | - | 2.8× | 42 | 70 |
| suggest_new_ideas | - | - | - | 3.2× | 30 | 57 |
| track_full_preference_evolution | - | - | - | 2.6× | 64 | 70 |
| overall | - | - | - | 3.1× | 64 | 60 |

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 68 | 1,079,430 | 39,513 | 30,667 | 15,874 / 581 | 0.0000 | 7482 / 9953 |

Retrieval tokens per question: context 11,703 (p95 18,199); extra-query LLM calls 0 prompt + 0 completion on average over all questions, 0 + 0 on the 0 questions given extra queries.

Per question, answer + judge, including cached replies: 15,807 prompt tokens, 582 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
