# beam · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| abstention | 2 | 100.0% | - | - | - | - | 10,660 / 11,013 | 16,542 / 78 | 224 / 224 |
| contradiction_resolution | 2 | 25.0% | - | - | 100.0% | 100.0% | 12,237 / 13,317 | 18,148 / 50 | 182 / 182 |
| event_ordering | 2 | 66.2% | - | - | 30.8% | 0.0% | 11,720 / 12,172 | 17,329 / 122 | 347 / 347 |
| information_extraction | 2 | 25.0% | - | - | 0.0% | 0.0% | 11,546 / 12,005 | 16,840 / 572 | 305 / 305 |
| instruction_following | 2 | 50.0% | - | - | 50.0% | 50.0% | 13,305 / 15,760 | 19,500 / 59 | 1279 / 1279 |
| knowledge_update | 2 | 50.0% | - | - | 100.0% | 100.0% | 21,726 / 33,143 | 31,270 / 32 | 1519 / 1519 |
| multi_session_reasoning | 2 | 50.0% | - | - | 100.0% | 100.0% | 40,404 / 48,547 | 63,473 / 766 | 1588 / 1588 |
| preference_following | 2 | 50.0% | - | - | 50.0% | 50.0% | 11,316 / 11,474 | 18,014 / 358 | 764 / 764 |
| summarization | 2 | 45.0% | - | - | 60.0% | 0.0% | 11,544 / 12,075 | 18,290 / 354 | 397 / 397 |
| temporal_reasoning | 2 | 50.0% | - | - | 100.0% | 100.0% | 34,991 / 35,697 | 48,771 / 77 | 1155 / 1155 |
| overall | 20 | 51.1% | - | - | 63.0% | 58.8% | 17,945 / 48,547 | 26,818 / 247 | 347 / 1588 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| abstention | - | - | - | 11.3× | 224 | 44 |
| contradiction_resolution | 1.000 | 29.2% | - | 11.0× | 182 | 26 |
| event_ordering | 0.583 | 18.1% | - | 12.3× | 347 | 44 |
| information_extraction | 0.000 | 0.0% | - | 12.8× | 305 | 114 |
| instruction_following | 0.208 | 12.1% | - | 14.2× | 1279 | 28 |
| knowledge_update | 1.000 | 21.7% | - | 7.6× | 1519 | 18 |
| multi_session_reasoning | 0.600 | 12.5% | - | 2.9× | 1588 | 36 |
| preference_following | 0.250 | 7.1% | - | 11.8× | 764 | 53 |
| summarization | 0.200 | 14.3% | - | 16.3× | 397 | 148 |
| temporal_reasoning | 0.333 | 10.0% | - | 4.8× | 1155 | 38 |
| overall | 0.479 | 13.9% | - | 8.4× | 1588 | 55 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| abstention | - | - | - |
| contradiction_resolution | 25.0% | - | 100.0% |
| event_ordering | - | 66.2% | 0.0% |
| information_extraction | - | 25.0% | 0.0% |
| instruction_following | 100.0% | 0.0% | 50.0% |
| knowledge_update | 50.0% | - | 100.0% |
| multi_session_reasoning | 50.0% | - | 100.0% |
| preference_following | 100.0% | 0.0% | 50.0% |
| summarization | - | 70.0% | 0.0% |
| temporal_reasoning | 50.0% | - | 100.0% |
| overall | 55.0% | 36.1% | 58.8% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

Abstention accuracy: 100.0%

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|

Retrieval tokens per question: context 17,945 (p95 48,547); extra-query LLM calls 474 prompt + 18 completion on average over all questions, 1,580 + 62 on the 6 questions given extra queries.

Per question, answer + judge, including cached replies: 32,330 prompt tokens, 2,908 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
