# longmemeval · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| knowledge-update | 10 | 90.0% | - | - | 95.0% | 100.0% | 14,185 / 25,712 | 20,949 / 604 | 4497 / 9086 |
| multi-session | 10 | 80.0% | - | - | 100.0% | 100.0% | 20,536 / 31,767 | 28,987 / 774 | 4920 / 5841 |
| single-session-assistant | 10 | 100.0% | - | - | 100.0% | 100.0% | 11,914 / 27,347 | 17,716 / 705 | 3589 / 8030 |
| single-session-preference | 10 | 70.0% | - | - | 86.7% | 90.0% | 9,647 / 10,888 | 14,821 / 1,776 | 722 / 1123 |
| single-session-user | 10 | 100.0% | - | - | 100.0% | 100.0% | 8,392 / 9,612 | 12,620 / 437 | 931 / 2364 |
| temporal-reasoning | 10 | 90.0% | - | - | 100.0% | 100.0% | 21,239 / 30,036 | 30,280 / 972 | 3065 / 4425 |
| overall | 60 | 88.3% | - | - | 96.9% | 98.3% | 14,319 / 28,669 | 20,896 / 878 | 2167 / 8030 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| knowledge-update | 0.950 | 24.6% | 100.0% | 8.7× | 9086 | 144 |
| multi-session | 0.944 | 20.8% | 100.0% | 6.0× | 5841 | 156 |
| single-session-assistant | 0.883 | 14.3% | 100.0% | 10.2× | 8030 | 156 |
| single-session-preference | 0.708 | 15.0% | 90.0% | 12.7× | 1123 | 444 |
| single-session-user | 0.900 | 13.5% | 100.0% | 14.6× | 2364 | 121 |
| temporal-reasoning | 0.889 | 18.6% | 100.0% | 5.8× | 4425 | 171 |
| overall | 0.878 | 17.7% | 98.3% | 8.6× | 9086 | 199 |

| category | evidence turn coverage (share of its sentences in the context) | evidence turns complete (≥90%) |
|---|---|---|
| knowledge-update | 95.0% | 95.0% |
| multi-session | 100.0% | 100.0% |
| single-session-assistant | 99.2% | 100.0% |
| single-session-preference | 86.7% | 86.7% |
| single-session-user | 96.2% | 90.9% |
| temporal-reasoning | 100.0% | 100.0% |
| overall | 96.4% | 95.9% |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| knowledge-update | 90.0% | - | 100.0% |
| multi-session | 77.8% | - | 100.0% |
| single-session-assistant | 100.0% | - | 100.0% |
| single-session-preference | 77.8% | 0.0% | 90.0% |
| single-session-user | 100.0% | - | 100.0% |
| temporal-reasoning | 100.0% | - | 100.0% |
| overall | 91.2% | 0.0% | 98.3% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

Abstention accuracy: 50.0%

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 60 | 1,253,733 | 52,679 | 26,719 | 20,896 / 878 | 0.0000 | 8598 / 15385 |
| judge | 60 | 90,316 | 30,282 | 28,978 | 1,505 / 505 | 0.0000 | 7212 / 17908 |

Retrieval tokens per question: context 14,319 (p95 28,669); extra-query LLM calls 791 prompt + 35 completion on average over all questions, 1,581 + 69 on the 30 questions given extra queries.

Per question, answer + judge, including cached replies: 22,401 prompt tokens, 1,383 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
