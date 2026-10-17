# longmemeval · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| knowledge-update | 10 | 100.0% | - | - | 100.0% | 100.0% | 14,761 / 25,112 | 21,722 / 633 | 249 / 475 |
| multi-session | 10 | 80.0% | - | - | 100.0% | 100.0% | 23,049 / 29,698 | 32,569 / 710 | 384 / 768 |
| single-session-assistant | 10 | 100.0% | - | - | 100.0% | 100.0% | 10,868 / 24,536 | 16,151 / 657 | 266 / 362 |
| single-session-preference | 10 | 60.0% | - | - | 80.0% | 90.0% | 9,664 / 11,373 | 14,877 / 1,776 | 299 / 375 |
| single-session-user | 10 | 100.0% | - | - | 100.0% | 100.0% | 8,134 / 9,617 | 12,385 / 448 | 247 / 408 |
| temporal-reasoning | 10 | 100.0% | - | - | 100.0% | 88.9% | 22,374 / 30,568 | 31,717 / 940 | 372 / 646 |
| overall | 60 | 90.0% | - | - | 96.9% | 96.6% | 14,808 / 28,302 | 21,570 / 861 | 275 / 562 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| knowledge-update | 1.000 | 30.8% | 100.0% | 8.3× | 475 | 164 |
| multi-session | 0.861 | 21.7% | 100.0% | 5.4× | 768 | 159 |
| single-session-assistant | 1.000 | 14.2% | 100.0% | 11.2× | 362 | 159 |
| single-session-preference | 0.725 | 13.9% | 90.0% | 12.7× | 375 | 406 |
| single-session-user | 0.950 | 14.6% | 100.0% | 15.0× | 408 | 110 |
| temporal-reasoning | 0.763 | 22.2% | 100.0% | 5.5× | 646 | 163 |
| overall | 0.886 | 19.5% | 98.3% | 8.3× | 768 | 193 |

| category | evidence turn coverage (share of its sentences in the context) | evidence turns complete (≥90%) |
|---|---|---|
| knowledge-update | 100.0% | 100.0% |
| multi-session | 100.0% | 100.0% |
| single-session-assistant | 99.2% | 100.0% |
| single-session-preference | 80.0% | 80.0% |
| single-session-user | 96.2% | 90.9% |
| temporal-reasoning | 100.0% | 100.0% |
| overall | 96.4% | 95.9% |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| knowledge-update | 100.0% | - | 100.0% |
| multi-session | 77.8% | - | 100.0% |
| single-session-assistant | 100.0% | - | 100.0% |
| single-session-preference | 66.7% | 0.0% | 90.0% |
| single-session-user | 100.0% | - | 100.0% |
| temporal-reasoning | 100.0% | 100.0% | 88.9% |
| overall | 91.1% | 50.0% | 96.6% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

Abstention accuracy: 100.0%

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|

Retrieval tokens per question: context 14,808 (p95 28,302); extra-query LLM calls 1,693 prompt + 155 completion on average over all questions, 1,689 + 206 on the 26 questions given extra queries.

Per question, answer + judge, including cached replies: 23,067 prompt tokens, 1,364 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
