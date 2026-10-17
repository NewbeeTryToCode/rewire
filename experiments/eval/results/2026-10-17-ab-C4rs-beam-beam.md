# beam · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| abstention | 2 | 100.0% | - | - | - | - | 15,934 / 20,855 | 23,690 / 78 | 267 / 267 |
| contradiction_resolution | 2 | 18.8% | - | - | 100.0% | 100.0% | 15,594 / 17,872 | 22,472 / 56 | 243 / 243 |
| event_ordering | 2 | 37.5% | - | - | 46.2% | 0.0% | 15,344 / 15,717 | 22,782 / 80 | 323 / 323 |
| information_extraction | 2 | 25.0% | - | - | 0.0% | 0.0% | 13,840 / 15,675 | 19,978 / 448 | 235 / 235 |
| instruction_following | 2 | 50.0% | - | - | 25.0% | 50.0% | 13,341 / 15,832 | 19,556 / 58 | 223 / 223 |
| knowledge_update | 2 | 50.0% | - | - | 100.0% | 100.0% | 21,726 / 33,143 | 31,270 / 32 | 365 / 365 |
| multi_session_reasoning | 2 | 0.0% | - | - | 100.0% | 100.0% | 42,218 / 51,904 | 66,003 / 802 | 331 / 331 |
| preference_following | 2 | 50.0% | - | - | 50.0% | 50.0% | 11,316 / 11,474 | 18,014 / 358 | 144 / 144 |
| summarization | 2 | 65.0% | - | - | 60.0% | 0.0% | 20,638 / 21,342 | 30,977 / 1,150 | 327 / 327 |
| temporal_reasoning | 2 | 25.0% | - | - | 100.0% | 100.0% | 33,593 / 34,634 | 47,114 / 150 | 645 / 645 |
| overall | 20 | 42.1% | - | - | 65.2% | 58.8% | 20,355 / 51,904 | 30,186 / 321 | 243 / 645 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| abstention | - | - | - | 7.6× | 267 | 43 |
| contradiction_resolution | 1.000 | 31.7% | - | 8.6× | 243 | 28 |
| event_ordering | 0.250 | 20.0% | - | 9.4× | 323 | 38 |
| information_extraction | 0.000 | 0.0% | - | 10.7× | 235 | 116 |
| instruction_following | 0.125 | 7.1% | - | 14.2× | 223 | 27 |
| knowledge_update | 1.000 | 21.7% | - | 7.6× | 365 | 18 |
| multi_session_reasoning | 1.000 | 12.5% | - | 2.8× | 331 | 28 |
| preference_following | 0.250 | 7.1% | - | 11.8× | 144 | 53 |
| summarization | 0.100 | 10.0% | - | 9.1× | 327 | 242 |
| temporal_reasoning | 0.750 | 10.0% | - | 5.0× | 645 | 26 |
| overall | 0.521 | 13.5% | - | 7.4× | 645 | 62 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| abstention | - | - | - |
| contradiction_resolution | 18.8% | - | 100.0% |
| event_ordering | - | 37.5% | 0.0% |
| information_extraction | - | 25.0% | 0.0% |
| instruction_following | 100.0% | 0.0% | 50.0% |
| knowledge_update | 50.0% | - | 100.0% |
| multi_session_reasoning | 0.0% | - | 100.0% |
| preference_following | 100.0% | 0.0% | 50.0% |
| summarization | - | 100.0% | 0.0% |
| temporal_reasoning | 25.0% | - | 100.0% |
| overall | 38.8% | 32.1% | 58.8% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

Abstention accuracy: 100.0%

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|

Per question, answer + judge, including cached replies: 35,762 prompt tokens, 3,062 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
