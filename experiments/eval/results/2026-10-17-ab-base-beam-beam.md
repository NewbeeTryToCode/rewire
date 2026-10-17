# beam · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| abstention | 2 | 100.0% | - | - | - | - | 12,045 / 12,170 | 18,802 / 55 | 112 / 112 |
| contradiction_resolution | 2 | 25.0% | - | - | 100.0% | 100.0% | 10,836 / 13,566 | 16,650 / 306 | 37 / 37 |
| event_ordering | 2 | 43.8% | - | - | 30.8% | 0.0% | 11,557 / 11,737 | 17,826 / 1,274 | 115 / 115 |
| information_extraction | 2 | 20.8% | - | - | 0.0% | 0.0% | 12,172 / 12,424 | 18,300 / 798 | 18 / 18 |
| instruction_following | 2 | 50.0% | - | - | 25.0% | 50.0% | 11,509 / 11,721 | 17,148 / 558 | 14 / 14 |
| knowledge_update | 2 | 50.0% | - | - | 100.0% | 100.0% | 10,713 / 10,962 | 16,276 / 110 | 13 / 13 |
| multi_session_reasoning | 2 | 50.0% | - | - | 100.0% | 100.0% | 11,620 / 13,410 | 18,802 / 951 | 17 / 17 |
| preference_following | 2 | 50.0% | - | - | 0.0% | 0.0% | 11,612 / 12,168 | 18,953 / 379 | 19 / 19 |
| summarization | 2 | 45.0% | - | - | 20.0% | 0.0% | 11,858 / 13,561 | 19,556 / 1,327 | 17 / 17 |
| temporal_reasoning | 2 | 37.5% | - | - | 100.0% | 100.0% | 11,742 / 12,084 | 17,622 / 142 | 16 / 16 |
| overall | 20 | 47.2% | - | - | 54.3% | 52.9% | 11,567 / 13,566 | 17,993 / 590 | 17 / 115 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| abstention | - | - | - | 10.0× | 112 | 32 |
| contradiction_resolution | 1.000 | 33.0% | - | 12.4× | 37 | 32 |
| event_ordering | 0.500 | 14.3% | - | 12.4× | 115 | 48 |
| information_extraction | 0.000 | 0.0% | - | 12.2× | 18 | 91 |
| instruction_following | 0.125 | 6.2% | - | 16.4× | 14 | 19 |
| knowledge_update | 0.500 | 33.3% | - | 15.4× | 13 | 22 |
| multi_session_reasoning | 1.000 | 22.6% | - | 10.1× | 17 | 14 |
| preference_following | 0.000 | 0.0% | - | 11.5× | 19 | 42 |
| summarization | 0.333 | 14.3% | - | 15.9× | 17 | 236 |
| temporal_reasoning | 0.375 | 22.6% | - | 14.4× | 16 | 13 |
| overall | 0.431 | 16.4% | - | 13.1× | 115 | 55 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| abstention | - | - | - |
| contradiction_resolution | 25.0% | - | 100.0% |
| event_ordering | - | 43.8% | 0.0% |
| information_extraction | - | 20.8% | 0.0% |
| instruction_following | 100.0% | 0.0% | 50.0% |
| knowledge_update | 50.0% | - | 100.0% |
| multi_session_reasoning | 50.0% | - | 100.0% |
| preference_following | - | 50.0% | 0.0% |
| summarization | - | 60.0% | 0.0% |
| temporal_reasoning | 37.5% | - | 100.0% |
| overall | 47.2% | 36.1% | 52.9% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

Abstention accuracy: 100.0%

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 20 | 359,867 | 11,803 | 9,262 | 17,993 / 590 | 0.0000 | 7063 / 16018 |
| judge | 52 | 111,426 | 59,311 | 50,338 | 2,143 / 1,141 | 0.0000 | 11801 / 23994 |

Per question, answer + judge, including cached replies: 23,565 prompt tokens, 3,556 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
