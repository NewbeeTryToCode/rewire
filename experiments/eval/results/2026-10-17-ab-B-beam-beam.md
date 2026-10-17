# beam · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| abstention | 2 | 100.0% | - | - | - | - | 18,010 / 21,129 | 26,768 / 70 | 1690 / 1690 |
| contradiction_resolution | 2 | 12.5% | - | - | 50.0% | 50.0% | 22,660 / 25,911 | 32,114 / 32 | 1826 / 1826 |
| event_ordering | 2 | 66.2% | - | - | 53.8% | 0.0% | 14,792 / 17,239 | 21,768 / 170 | 1750 / 1750 |
| information_extraction | 2 | 20.8% | - | - | 33.3% | 50.0% | 17,112 / 17,751 | 24,690 / 154 | 333 / 333 |
| instruction_following | 2 | 50.0% | - | - | 50.0% | 50.0% | 15,572 / 15,760 | 22,790 / 64 | 259 / 259 |
| knowledge_update | 2 | 100.0% | - | - | 100.0% | 100.0% | 24,083 / 33,143 | 34,300 / 43 | 497 / 497 |
| multi_session_reasoning | 2 | 50.0% | - | - | 100.0% | 100.0% | 40,404 / 48,547 | 63,473 / 766 | 493 / 493 |
| preference_following | 2 | 37.5% | - | - | 100.0% | 100.0% | 16,366 / 17,009 | 25,232 / 176 | 405 / 405 |
| summarization | 2 | 55.0% | - | - | 0.0% | 0.0% | 21,414 / 22,107 | 32,356 / 1,194 | 773 / 773 |
| temporal_reasoning | 2 | 50.0% | - | - | 100.0% | 100.0% | 34,991 / 35,697 | 48,771 / 77 | 1064 / 1064 |
| overall | 20 | 54.2% | - | - | 60.9% | 64.7% | 22,540 / 48,547 | 33,226 / 275 | 493 / 1826 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| abstention | - | - | - | 6.7× | 1690 | 39 |
| contradiction_resolution | 1.000 | 20.0% | - | 6.0× | 1826 | 14 |
| event_ordering | 0.250 | 25.0% | - | 9.7× | 1750 | 62 |
| information_extraction | 0.167 | 5.0% | - | 8.6× | 333 | 77 |
| instruction_following | 0.250 | 10.0% | - | 12.1× | 259 | 35 |
| knowledge_update | 0.750 | 15.0% | - | 6.9× | 497 | 22 |
| multi_session_reasoning | 0.600 | 12.5% | - | 2.9× | 493 | 36 |
| preference_following | 0.134 | 10.0% | - | 8.2× | 405 | 26 |
| summarization | 0.000 | 0.0% | - | 8.8× | 773 | 216 |
| temporal_reasoning | 0.333 | 10.0% | - | 4.8× | 1064 | 38 |
| overall | 0.410 | 12.6% | - | 6.7× | 1826 | 57 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| abstention | - | - | - |
| contradiction_resolution | 25.0% | 0.0% | 50.0% |
| event_ordering | - | 66.2% | 0.0% |
| information_extraction | 41.7% | 0.0% | 50.0% |
| instruction_following | 100.0% | 0.0% | 50.0% |
| knowledge_update | 100.0% | - | 100.0% |
| multi_session_reasoning | 50.0% | - | 100.0% |
| preference_following | 37.5% | - | 100.0% |
| summarization | - | 60.0% | 0.0% |
| temporal_reasoning | 50.0% | - | 100.0% |
| overall | 58.3% | 32.1% | 64.7% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

Abstention accuracy: 100.0%

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| expand | 20 | 31,668 | 1,525 | 150 | 1,583 / 76 | 0.0000 | 5007 / 6296 |
| answer | 20 | 664,527 | 5,494 | 2,940 | 33,226 / 275 | 0.0000 | 6046 / 12873 |
| judge | 52 | 110,941 | 56,314 | 47,063 | 2,133 / 1,083 | 0.0000 | 12042 / 22753 |

Per question, answer + judge, including cached replies: 38,773 prompt tokens, 3,090 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
