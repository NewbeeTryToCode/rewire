# beam · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| abstention | 2 | 100.0% | - | - | - | - | 15,934 / 20,855 | 23,690 / 78 | 244 / 244 |
| contradiction_resolution | 2 | 18.8% | - | - | 100.0% | 100.0% | 15,594 / 17,872 | 22,472 / 56 | 197 / 197 |
| event_ordering | 2 | 37.5% | - | - | 46.2% | 0.0% | 15,344 / 15,717 | 22,782 / 80 | 450 / 450 |
| information_extraction | 2 | 25.0% | - | - | 0.0% | 0.0% | 13,840 / 15,675 | 19,978 / 448 | 272 / 272 |
| instruction_following | 2 | 50.0% | - | - | 25.0% | 50.0% | 13,341 / 15,832 | 19,556 / 58 | 225 / 225 |
| knowledge_update | 2 | 0.0% | - | - | 75.0% | 50.0% | 15,618 / 20,927 | 22,825 / 19 | 177 / 177 |
| multi_session_reasoning | 2 | 0.0% | - | - | 100.0% | 100.0% | 42,218 / 51,904 | 66,003 / 802 | 336 / 336 |
| preference_following | 2 | 50.0% | - | - | 50.0% | 50.0% | 11,316 / 11,474 | 18,014 / 358 | 146 / 146 |
| summarization | 2 | 65.0% | - | - | 60.0% | 0.0% | 20,638 / 21,342 | 30,977 / 1,150 | 336 / 336 |
| temporal_reasoning | 2 | 25.0% | - | - | 100.0% | 100.0% | 33,593 / 34,634 | 47,114 / 150 | 506 / 506 |
| overall | 20 | 37.1% | - | - | 63.0% | 52.9% | 19,744 / 51,904 | 29,341 / 320 | 238 / 506 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| abstention | - | - | - | 7.6× | 244 | 43 |
| contradiction_resolution | 1.000 | 31.7% | - | 8.6× | 197 | 28 |
| event_ordering | 0.250 | 20.0% | - | 9.4× | 450 | 38 |
| information_extraction | 0.000 | 0.0% | - | 10.7× | 272 | 116 |
| instruction_following | 0.125 | 7.1% | - | 14.2× | 225 | 27 |
| knowledge_update | 0.750 | 21.2% | - | 10.6× | 177 | 11 |
| multi_session_reasoning | 1.000 | 12.5% | - | 2.8× | 336 | 28 |
| preference_following | 0.250 | 7.1% | - | 11.8× | 146 | 53 |
| summarization | 0.100 | 10.0% | - | 9.1× | 336 | 242 |
| temporal_reasoning | 0.750 | 10.0% | - | 5.0× | 506 | 26 |
| overall | 0.491 | 13.5% | - | 7.6× | 506 | 61 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| abstention | - | - | - |
| contradiction_resolution | 18.8% | - | 100.0% |
| event_ordering | - | 37.5% | 0.0% |
| information_extraction | - | 25.0% | 0.0% |
| instruction_following | 100.0% | 0.0% | 50.0% |
| knowledge_update | 0.0% | 0.0% | 50.0% |
| multi_session_reasoning | 0.0% | - | 100.0% |
| preference_following | 100.0% | 0.0% | 50.0% |
| summarization | - | 100.0% | 0.0% |
| temporal_reasoning | 25.0% | - | 100.0% |
| overall | 31.9% | 28.1% | 52.9% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

Abstention accuracy: 100.0%

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| expand | 20 | 33,930 | 3,847 | 3,019 | 1,696 / 192 | 0.0000 | 5962 / 7200 |
| answer | 12 | 432,935 | 5,052 | 3,237 | 36,078 / 421 | 0.0000 | 6696 / 13095 |
| judge | 33 | 71,593 | 35,118 | 29,245 | 2,169 / 1,064 | 0.0000 | 12471 / 24929 |

Per question, answer + judge, including cached replies: 34,916 prompt tokens, 3,046 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
