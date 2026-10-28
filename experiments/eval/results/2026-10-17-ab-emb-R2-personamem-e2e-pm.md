# personamem · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | 10 | 40.0% | - | - | - | - | 11,313 / 12,364 | 15,213 / 446 | 1561 / 3885 |
| provide_preference_aligned_recommendations | 10 | 60.0% | - | - | - | - | 11,352 / 12,459 | 15,576 / 551 | 803 / 1416 |
| recall_user_shared_facts | 10 | 50.0% | - | - | - | - | 11,339 / 12,156 | 14,956 / 418 | 486 / 802 |
| recalling_facts_mentioned_by_the_user | 10 | 60.0% | - | - | - | - | 11,146 / 11,839 | 15,060 / 578 | 703 / 1203 |
| recalling_the_reasons_behind_previous_updates | 10 | 90.0% | - | - | - | - | 12,424 / 18,931 | 16,768 / 437 | 1009 / 1298 |
| suggest_new_ideas | 10 | 10.0% | - | - | - | - | 11,371 / 12,755 | 15,493 / 604 | 764 / 1142 |
| track_full_preference_evolution | 10 | 70.0% | - | - | - | - | 13,453 / 22,348 | 18,169 / 844 | 863 / 1688 |
| overall | 70 | 54.3% | - | - | - | - | 11,771 / 18,931 | 15,891 / 554 | 802 / 3093 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| generalizing_to_new_scenarios | - | - | - | 3.4× | 3885 | 49 |
| provide_preference_aligned_recommendations | - | - | - | 3.1× | 1416 | 59 |
| recall_user_shared_facts | - | - | - | 3.2× | 802 | 52 |
| recalling_facts_mentioned_by_the_user | - | - | - | 3.3× | 1203 | 55 |
| recalling_the_reasons_behind_previous_updates | - | - | - | 2.8× | 1298 | 57 |
| suggest_new_ideas | - | - | - | 3.2× | 1142 | 53 |
| track_full_preference_evolution | - | - | - | 2.5× | 1688 | 66 |
| overall | - | - | - | 3.1× | 3885 | 56 |

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 70 | 1,112,367 | 38,778 | 30,077 | 15,891 / 554 | 0.0000 | 7758 / 10636 |

Retrieval tokens per question: context 11,771 (p95 18,931); extra-query LLM calls 0 prompt + 0 completion on average over all questions, 0 + 0 on the 0 questions given extra queries.

Per question, answer + judge, including cached replies: 15,891 prompt tokens, 554 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
