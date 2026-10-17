# beam · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| abstention | 2 | 50.0% | - | - | - | - | 10,666 / 11,019 | 16,548 / 84 | 166 / 166 |
| contradiction_resolution | 2 | 0.0% | - | - | 100.0% | 100.0% | 12,243 / 13,323 | 18,160 / 234 | 124 / 124 |
| event_ordering | 2 | 51.2% | - | - | 30.8% | 0.0% | 11,726 / 12,178 | 17,334 / 154 | 344 / 344 |
| information_extraction | 2 | 20.8% | - | - | 0.0% | 0.0% | 11,552 / 12,011 | 16,852 / 465 | 91 / 91 |
| instruction_following | 2 | 50.0% | - | - | 50.0% | 50.0% | 13,311 / 15,766 | 19,508 / 118 | 263 / 263 |
| knowledge_update | 2 | 50.0% | - | - | 100.0% | 100.0% | 21,732 / 33,149 | 31,280 / 112 | 451 / 451 |
| multi_session_reasoning | 2 | 50.0% | - | - | 100.0% | 100.0% | 40,410 / 48,553 | 63,482 / 917 | 415 / 415 |
| preference_following | 2 | 50.0% | - | - | 50.0% | 50.0% | 11,322 / 11,480 | 18,021 / 425 | 155 / 155 |
| summarization | 2 | 65.0% | - | - | 60.0% | 0.0% | 11,550 / 12,081 | 18,294 / 1,226 | 102 / 102 |
| temporal_reasoning | 2 | 50.0% | - | - | 100.0% | 100.0% | 34,997 / 35,703 | 48,783 / 220 | 661 / 661 |
| overall | 20 | 43.7% | - | - | 63.0% | 58.8% | 17,951 / 48,553 | 26,826 / 396 | 155 / 661 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| abstention | - | - | - | 11.3× | 166 | 15 |
| contradiction_resolution | 1.000 | 29.2% | - | 11.0× | 124 | 12 |
| event_ordering | 0.583 | 18.1% | - | 12.3× | 344 | 62 |
| information_extraction | 0.000 | 0.0% | - | 12.8× | 91 | 117 |
| instruction_following | 0.208 | 12.1% | - | 14.2× | 263 | 26 |
| knowledge_update | 1.000 | 21.7% | - | 7.6× | 451 | 56 |
| multi_session_reasoning | 0.600 | 12.5% | - | 2.9× | 415 | 34 |
| preference_following | 0.250 | 7.1% | - | 11.8× | 155 | 37 |
| summarization | 0.200 | 14.3% | - | 16.3× | 102 | 210 |
| temporal_reasoning | 0.333 | 10.0% | - | 4.8× | 661 | 29 |
| overall | 0.479 | 13.9% | - | 8.4× | 661 | 60 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| abstention | - | - | - |
| contradiction_resolution | 0.0% | - | 100.0% |
| event_ordering | - | 51.2% | 0.0% |
| information_extraction | - | 20.8% | 0.0% |
| instruction_following | 100.0% | 0.0% | 50.0% |
| knowledge_update | 50.0% | - | 100.0% |
| multi_session_reasoning | 50.0% | - | 100.0% |
| preference_following | 100.0% | 0.0% | 50.0% |
| summarization | - | 80.0% | 0.0% |
| temporal_reasoning | 50.0% | - | 100.0% |
| overall | 50.0% | 32.0% | 58.8% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

Abstention accuracy: 50.0%

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 20 | 536,519 | 7,914 | 5,019 | 26,826 / 396 | 0.0000 | 7132 / 13142 |
| judge | 50 | 108,106 | 59,277 | 50,437 | 2,162 / 1,186 | 0.0000 | 11178 / 31195 |

Per question, answer + judge, including cached replies: 32,431 prompt tokens, 3,444 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
