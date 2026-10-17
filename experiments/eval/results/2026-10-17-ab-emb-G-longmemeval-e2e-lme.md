# longmemeval · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| knowledge-update | 10 | 100.0% | - | - | 100.0% | 100.0% | 14,710 / 27,062 | 21,490 / 634 | 567 / 1071 |
| multi-session | 10 | 90.0% | - | - | 100.0% | 100.0% | 21,550 / 29,543 | 30,651 / 813 | 694 / 986 |
| single-session-assistant | 10 | 100.0% | - | - | 100.0% | 100.0% | 11,814 / 22,937 | 17,844 / 777 | 550 / 1028 |
| single-session-preference | 10 | 60.0% | - | - | 93.3% | 100.0% | 9,370 / 11,361 | 14,524 / 1,904 | 279 / 366 |
| single-session-user | 10 | 100.0% | - | - | 100.0% | 100.0% | 8,482 / 11,331 | 12,854 / 402 | 213 / 435 |
| temporal-reasoning | 10 | 100.0% | - | - | 100.0% | 100.0% | 22,178 / 29,903 | 31,588 / 864 | 691 / 1195 |
| overall | 60 | 91.7% | - | - | 99.0% | 100.0% | 14,684 / 28,861 | 21,492 / 899 | 444 / 1058 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| knowledge-update | 1.000 | 24.9% | 100.0% | 8.4× | 1071 | 169 |
| multi-session | 0.944 | 23.7% | 100.0% | 5.7× | 986 | 182 |
| single-session-assistant | 0.808 | 16.3% | 100.0% | 10.3× | 1028 | 158 |
| single-session-preference | 0.716 | 17.1% | 100.0% | 13.1× | 366 | 447 |
| single-session-user | 0.900 | 14.4% | 100.0% | 14.4× | 435 | 112 |
| temporal-reasoning | 0.670 | 23.7% | 100.0% | 5.5× | 1195 | 153 |
| overall | 0.841 | 19.9% | 100.0% | 8.4× | 1195 | 203 |

| category | evidence turn coverage (share of its sentences in the context) | evidence turns complete (≥90%) |
|---|---|---|
| knowledge-update | 100.0% | 100.0% |
| multi-session | 100.0% | 100.0% |
| single-session-assistant | 99.2% | 100.0% |
| single-session-preference | 93.3% | 93.3% |
| single-session-user | 96.2% | 90.9% |
| temporal-reasoning | 100.0% | 100.0% |
| overall | 98.5% | 98.0% |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| knowledge-update | 100.0% | - | 100.0% |
| multi-session | 88.9% | - | 100.0% |
| single-session-assistant | 100.0% | - | 100.0% |
| single-session-preference | 60.0% | - | 100.0% |
| single-session-user | 100.0% | - | 100.0% |
| temporal-reasoning | 100.0% | - | 100.0% |
| overall | 91.4% | - | 100.0% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

Abstention accuracy: 100.0%

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 60 | 1,289,502 | 53,958 | 27,519 | 21,492 / 899 | 0.0000 | 8626 / 15225 |
| judge | 60 | 90,712 | 26,446 | 24,692 | 1,512 / 441 | 0.0000 | 6925 / 17471 |

Retrieval tokens per question: context 14,684 (p95 28,861); extra-query LLM calls 791 prompt + 35 completion on average over all questions, 1,581 + 69 on the 30 questions given extra queries.

Per question, answer + judge, including cached replies: 23,004 prompt tokens, 1,340 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
