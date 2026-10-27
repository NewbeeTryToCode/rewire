# longmemeval · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| knowledge-update | 10 | 100.0% | - | - | 100.0% | 100.0% | 14,712 / 25,112 | 21,616 / 633 | 307 / 396 |
| multi-session | 10 | 80.0% | - | - | 100.0% | 100.0% | 23,049 / 29,698 | 32,569 / 710 | 420 / 738 |
| single-session-assistant | 10 | 100.0% | - | - | 100.0% | 100.0% | 12,687 / 24,536 | 19,040 / 696 | 275 / 435 |
| single-session-preference | 10 | 60.0% | - | - | 80.0% | 90.0% | 9,664 / 11,373 | 14,877 / 1,776 | 273 / 318 |
| single-session-user | 10 | 100.0% | - | - | 100.0% | 100.0% | 8,483 / 11,676 | 12,852 / 432 | 227 / 337 |
| temporal-reasoning | 10 | 100.0% | - | - | 100.0% | 88.9% | 22,374 / 30,568 | 31,717 / 940 | 315 / 399 |
| overall | 60 | 90.0% | - | - | 96.9% | 96.6% | 15,162 / 28,302 | 22,112 / 864 | 300 / 462 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| knowledge-update | 1.000 | 31.3% | 100.0% | 8.4× | 396 | 164 |
| multi-session | 0.861 | 21.7% | 100.0% | 5.4× | 738 | 159 |
| single-session-assistant | 0.920 | 15.2% | 100.0% | 9.6× | 435 | 159 |
| single-session-preference | 0.725 | 13.9% | 90.0% | 12.7× | 318 | 406 |
| single-session-user | 0.950 | 15.2% | 100.0% | 14.4× | 337 | 109 |
| temporal-reasoning | 0.763 | 22.2% | 100.0% | 5.5× | 399 | 163 |
| overall | 0.872 | 19.8% | 98.3% | 8.1× | 738 | 193 |

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

Retrieval tokens per question: context 15,162 (p95 28,302); extra-query LLM calls 1,879 prompt + 165 completion on average over all questions, 2,032 + 207 on the 33 questions given extra queries.

Per question, answer + judge, including cached replies: 23,608 prompt tokens, 1,376 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
