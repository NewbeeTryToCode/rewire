# personamem · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| acknowledge_latest_user_preferences | 20 | 65.0% | - | - | - | - | 11,144 / 12,951 | 15,056 / 433 | 84 / 211 |
| generalizing_to_new_scenarios | 20 | 30.0% | - | - | - | - | 11,684 / 13,398 | 15,847 / 503 | 65 / 304 |
| provide_preference_aligned_recommendations | 20 | 60.0% | - | - | - | - | 11,800 / 12,758 | 16,233 / 539 | 53 / 220 |
| recall_user_shared_facts | 20 | 50.0% | - | - | - | - | 11,540 / 13,211 | 15,551 / 524 | 74 / 263 |
| revisit_reasons_behind_preference_updates | 20 | 75.0% | - | - | - | - | 12,959 / 22,501 | 17,538 / 518 | 91 / 826 |
| suggest_new_ideas | 20 | 20.0% | - | - | - | - | 11,571 / 12,948 | 15,840 / 596 | 53 / 221 |
| track_full_preference_updates | 20 | 60.0% | - | - | - | - | 11,526 / 21,116 | 15,991 / 689 | 93 / 855 |
| overall | 140 | 51.4% | - | - | - | - | 11,746 / 13,398 | 16,008 / 543 | 77 / 220 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| acknowledge_latest_user_preferences | - | - | - | 104.3× | 211 | 50 |
| generalizing_to_new_scenarios | - | - | - | 100.5× | 304 | 53 |
| provide_preference_aligned_recommendations | - | - | - | 100.0× | 220 | 67 |
| recall_user_shared_facts | - | - | - | 101.2× | 263 | 55 |
| revisit_reasons_behind_preference_updates | - | - | - | 88.6× | 826 | 64 |
| suggest_new_ideas | - | - | - | 101.2× | 221 | 63 |
| track_full_preference_updates | - | - | - | 101.4× | 855 | 60 |
| overall | - | - | - | 99.4× | 826 | 59 |

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 1 | 15,514 | 361 | 228 | 15,514 / 361 | 0.0000 | 6294 / 6294 |

Per question, answer + judge, including cached replies: 16,008 prompt tokens, 543 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
