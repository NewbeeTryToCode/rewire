# beam · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| abstention | 2 | 100.0% | - | - | - | - | 11,748 / 12,269 | 18,008 / 72 | 3921 / 3921 |
| contradiction_resolution | 2 | 12.5% | - | - | 100.0% | 100.0% | 11,732 / 12,765 | 17,394 / 62 | 3677 / 3677 |
| event_ordering | 2 | 61.3% | - | - | 23.1% | 0.0% | 12,056 / 12,276 | 17,763 / 110 | 1138 / 1138 |
| information_extraction | 2 | 0.0% | - | - | 0.0% | 0.0% | 11,502 / 11,557 | 16,846 / 109 | 956 / 956 |
| instruction_following | 2 | 50.0% | - | - | 50.0% | 50.0% | 12,404 / 13,957 | 18,184 / 400 | 3001 / 3001 |
| knowledge_update | 2 | 75.0% | - | - | 100.0% | 100.0% | 21,387 / 31,965 | 30,988 / 72 | 3187 / 3187 |
| multi_session_reasoning | 2 | 50.0% | - | - | 100.0% | 100.0% | 36,720 / 42,411 | 57,911 / 842 | 5442 / 5442 |
| preference_following | 2 | 50.0% | - | - | 100.0% | 100.0% | 12,664 / 13,381 | 19,944 / 154 | 801 / 801 |
| summarization | 2 | 30.0% | - | - | 40.0% | 0.0% | 11,306 / 12,298 | 18,274 / 600 | 1134 / 1134 |
| temporal_reasoning | 2 | 25.0% | - | - | 100.0% | 100.0% | 37,306 / 38,126 | 51,942 / 249 | 3404 / 3404 |
| overall | 20 | 45.4% | - | - | 60.9% | 64.7% | 17,882 / 42,411 | 26,725 / 267 | 3001 / 5442 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| abstention | - | - | - | 10.3× | 3921 | 38 |
| contradiction_resolution | 0.333 | 31.0% | - | 11.5× | 3677 | 32 |
| event_ordering | 0.556 | 11.8% | - | 11.9× | 1138 | 42 |
| information_extraction | 0.000 | 0.0% | - | 12.9× | 956 | 56 |
| instruction_following | 0.750 | 12.1% | - | 15.2× | 3001 | 11 |
| knowledge_update | 1.000 | 19.3% | - | 7.7× | 3187 | 32 |
| multi_session_reasoning | 1.000 | 12.5% | - | 3.2× | 5442 | 38 |
| preference_following | 0.181 | 11.1% | - | 10.5× | 801 | 24 |
| summarization | 0.500 | 12.5% | - | 16.6× | 1134 | 150 |
| temporal_reasoning | 0.375 | 10.0% | - | 4.5× | 3404 | 2.000 |
| overall | 0.523 | 13.4% | - | 8.4× | 5442 | 43 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| abstention | - | - | - |
| contradiction_resolution | 12.5% | - | 100.0% |
| event_ordering | - | 61.3% | 0.0% |
| information_extraction | - | 0.0% | 0.0% |
| instruction_following | 100.0% | 0.0% | 50.0% |
| knowledge_update | 75.0% | - | 100.0% |
| multi_session_reasoning | 50.0% | - | 100.0% |
| preference_following | 50.0% | - | 100.0% |
| summarization | - | 30.0% | 0.0% |
| temporal_reasoning | 25.0% | - | 100.0% |
| overall | 47.7% | 25.4% | 64.7% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

Abstention accuracy: 100.0%

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 20 | 534,505 | 5,340 | 3,454 | 26,725 / 267 | 0.0000 | 6465 / 14959 |
| judge | 50 | 104,782 | 49,013 | 40,438 | 2,096 / 980 | 0.0000 | 12164 / 23849 |

Retrieval tokens per question: context 17,882 (p95 42,411); extra-query LLM calls 474 prompt + 18 completion on average over all questions, 1,580 + 62 on the 6 questions given extra queries.

Per question, answer + judge, including cached replies: 32,164 prompt tokens, 2,796 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
