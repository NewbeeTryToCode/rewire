# personamem · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| acknowledge_latest_user_preferences | 20 | 65.0% | - | - | - | - | 10,963 / 12,500 | 14,542 / 447 | 110 / 254 |
| generalize_to_new_scenarios | 20 | 35.0% | - | - | - | - | 11,569 / 13,070 | 15,618 / 517 | 48 / 125 |
| provide_preference_aligned_recommendations | 20 | 40.0% | - | - | - | - | 11,451 / 13,011 | 15,764 / 549 | 56 / 352 |
| recall_user_shared_facts | 20 | 70.0% | - | - | - | - | 10,991 / 12,503 | 14,848 / 506 | 26 / 104 |
| revisit_reasons_behind_preference_updates | 20 | 75.0% | - | - | - | - | 9,603 / 12,318 | 13,446 / 465 | 94 / 274 |
| suggest_new_ideas | 20 | 55.0% | - | - | - | - | 11,440 / 13,048 | 15,603 / 569 | 34 / 246 |
| track_full_preference_evolution | 20 | 50.0% | - | - | - | - | 11,106 / 21,249 | 15,438 / 590 | 101 / 214 |
| overall | 140 | 55.7% | - | - | - | - | 11,017 / 12,676 | 15,037 / 520 | 57 / 214 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| acknowledge_latest_user_preferences | - | - | - | 14.6× | 254 | 47 |
| generalize_to_new_scenarios | - | - | - | 14.0× | 125 | 66 |
| provide_preference_aligned_recommendations | - | - | - | 14.1× | 352 | 59 |
| recall_user_shared_facts | - | - | - | 14.7× | 104 | 72 |
| revisit_reasons_behind_preference_updates | - | - | - | 16.5× | 274 | 57 |
| suggest_new_ideas | - | - | - | 13.9× | 246 | 57 |
| track_full_preference_evolution | - | - | - | 13.9× | 214 | 62 |
| overall | - | - | - | 14.5× | 274 | 60 |

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 140 | 2,105,164 | 72,867 | 54,597 | 15,037 / 520 | 0.0000 | 7950 / 10294 |

Per question, answer + judge, including cached replies: 15,037 prompt tokens, 520 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
