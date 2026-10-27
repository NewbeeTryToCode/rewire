# beam · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| abstention | 2 | 100.0% | - | - | - | - | 11,318 / 11,812 | 17,530 / 94 | 488 / 488 |
| contradiction_resolution | 2 | 31.2% | - | - | 83.3% | 50.0% | 12,376 / 13,187 | 18,222 / 64 | 550 / 550 |
| event_ordering | 2 | 45.0% | - | - | 46.2% | 0.0% | 11,562 / 12,131 | 17,017 / 116 | 287 / 287 |
| information_extraction | 2 | 25.0% | - | - | 0.0% | 0.0% | 11,605 / 11,763 | 16,824 / 128 | 213 / 213 |
| instruction_following | 2 | 50.0% | - | - | 50.0% | 50.0% | 13,294 / 14,973 | 19,356 / 126 | 448 / 448 |
| knowledge_update | 2 | 0.0% | - | - | 100.0% | 100.0% | 23,372 / 35,630 | 33,570 / 23 | 629 / 629 |
| multi_session_reasoning | 2 | 50.0% | - | - | 100.0% | 100.0% | 37,213 / 45,114 | 58,465 / 948 | 384 / 384 |
| preference_following | 2 | 37.5% | - | - | 50.0% | 50.0% | 11,840 / 12,078 | 18,586 / 292 | 117 / 117 |
| summarization | 2 | 45.0% | - | - | 20.0% | 0.0% | 12,124 / 13,149 | 19,332 / 1,702 | 458 / 458 |
| temporal_reasoning | 2 | 25.0% | - | - | 100.0% | 100.0% | 33,892 / 34,813 | 47,250 / 90 | 700 / 700 |
| overall | 20 | 40.9% | - | - | 60.9% | 52.9% | 17,860 / 45,114 | 26,615 / 358 | 359 / 700 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| abstention | - | - | - | 10.6× | 488 | 46 |
| contradiction_resolution | 0.583 | 22.9% | - | 10.9× | 550 | 34 |
| event_ordering | 0.667 | 18.3% | - | 12.4× | 287 | 42 |
| information_extraction | 0.000 | 0.0% | - | 12.8× | 213 | 69 |
| instruction_following | 0.155 | 11.2% | - | 14.2× | 448 | 18 |
| knowledge_update | 1.000 | 16.1% | - | 7.1× | 629 | 12 |
| multi_session_reasoning | 1.000 | 12.5% | - | 3.2× | 384 | 14 |
| preference_following | 0.250 | 6.2% | - | 11.3× | 117 | 33 |
| summarization | 0.500 | 11.1% | - | 15.5× | 458 | 266 |
| temporal_reasoning | 0.600 | 10.0% | - | 5.0× | 700 | 11 |
| overall | 0.530 | 12.1% | - | 8.5× | 700 | 55 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| abstention | - | - | - |
| contradiction_resolution | 37.5% | 25.0% | 50.0% |
| event_ordering | - | 45.0% | 0.0% |
| information_extraction | - | 25.0% | 0.0% |
| instruction_following | 100.0% | 0.0% | 50.0% |
| knowledge_update | 0.0% | - | 100.0% |
| multi_session_reasoning | 50.0% | - | 100.0% |
| preference_following | 75.0% | 0.0% | 50.0% |
| summarization | - | 50.0% | 0.0% |
| temporal_reasoning | 25.0% | - | 100.0% |
| overall | 40.3% | 26.9% | 52.9% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

Abstention accuracy: 100.0%

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 20 | 532,302 | 7,167 | 4,541 | 26,615 / 358 | 0.0000 | 6172 / 12867 |
| judge | 47 | 101,750 | 55,871 | 47,515 | 2,165 / 1,189 | 0.0000 | 12233 / 28181 |

Retrieval tokens per question: context 17,860 (p95 45,114); extra-query LLM calls 474 prompt + 18 completion on average over all questions, 1,580 + 62 on the 6 questions given extra queries.

Per question, answer + judge, including cached replies: 32,203 prompt tokens, 3,376 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
