# beam · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| abstention | 2 | 100.0% | - | - | - | - | 10,660 / 11,013 | 16,542 / 78 | 137 / 137 |
| contradiction_resolution | 2 | 25.0% | - | - | 100.0% | 100.0% | 12,237 / 13,317 | 18,148 / 50 | 259 / 259 |
| event_ordering | 2 | 66.2% | - | - | 30.8% | 0.0% | 11,720 / 12,172 | 17,329 / 122 | 261 / 261 |
| information_extraction | 2 | 25.0% | - | - | 0.0% | 0.0% | 11,546 / 12,005 | 16,840 / 572 | 106 / 106 |
| instruction_following | 2 | 50.0% | - | - | 25.0% | 50.0% | 11,250 / 11,650 | 16,692 / 141 | 97 / 97 |
| knowledge_update | 2 | 0.0% | - | - | 75.0% | 50.0% | 15,618 / 20,927 | 22,825 / 19 | 97 / 97 |
| multi_session_reasoning | 2 | 0.0% | - | - | 100.0% | 100.0% | 19,922 / 21,680 | 32,472 / 672 | 120 / 120 |
| preference_following | 2 | 50.0% | - | - | 50.0% | 50.0% | 11,316 / 11,474 | 18,014 / 358 | 209 / 209 |
| summarization | 2 | 45.0% | - | - | 60.0% | 0.0% | 11,544 / 12,075 | 18,290 / 354 | 102 / 102 |
| temporal_reasoning | 2 | 25.0% | - | - | 100.0% | 100.0% | 21,958 / 22,589 | 31,473 / 324 | 223 / 223 |
| overall | 20 | 38.6% | - | - | 58.7% | 52.9% | 13,777 / 22,589 | 20,862 / 269 | 106 / 261 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| abstention | - | - | - | 11.3× | 137 | 44 |
| contradiction_resolution | 1.000 | 29.2% | - | 11.0× | 259 | 26 |
| event_ordering | 0.583 | 18.1% | - | 12.3× | 261 | 44 |
| information_extraction | 0.000 | 0.0% | - | 12.8× | 106 | 114 |
| instruction_following | 0.125 | 7.1% | - | 16.8× | 97 | 23 |
| knowledge_update | 0.750 | 21.2% | - | 10.6× | 97 | 11 |
| multi_session_reasoning | 1.000 | 20.2% | - | 5.9× | 120 | 12 |
| preference_following | 0.250 | 7.1% | - | 11.8× | 209 | 53 |
| summarization | 0.200 | 14.3% | - | 16.3× | 102 | 148 |
| temporal_reasoning | 0.333 | 11.5% | - | 7.7× | 223 | 5.000 |
| overall | 0.487 | 14.3% | - | 11.0× | 261 | 48 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| abstention | - | - | - |
| contradiction_resolution | 25.0% | - | 100.0% |
| event_ordering | - | 66.2% | 0.0% |
| information_extraction | - | 25.0% | 0.0% |
| instruction_following | 100.0% | 0.0% | 50.0% |
| knowledge_update | 0.0% | 0.0% | 50.0% |
| multi_session_reasoning | 0.0% | - | 100.0% |
| preference_following | 100.0% | 0.0% | 50.0% |
| summarization | - | 70.0% | 0.0% |
| temporal_reasoning | 25.0% | - | 100.0% |
| overall | 33.3% | 31.6% | 52.9% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

Abstention accuracy: 100.0%

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 20 | 417,248 | 5,381 | 3,223 | 20,862 / 269 | 0.0000 | 6171 / 11358 |
| judge | 52 | 109,881 | 53,314 | 44,457 | 2,113 / 1,025 | 0.0000 | 11488 / 20950 |

Per question, answer + judge, including cached replies: 26,356 prompt tokens, 2,935 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
