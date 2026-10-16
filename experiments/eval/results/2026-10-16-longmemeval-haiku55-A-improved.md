# longmemeval · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| knowledge-update | 78 | 92.3% | - | - | 97.2% | 95.8% | 12,536 / 19,355 | 18,575 / 683 | 239 / 651 |
| multi-session | 133 | 75.2% | - | - | 93.8% | 87.6% | 14,133 / 19,583 | 20,635 / 845 | 252 / 682 |
| single-session-assistant | 56 | 91.1% | - | - | 94.6% | 100.0% | 9,245 / 17,611 | 13,910 / 605 | 244 / 504 |
| single-session-preference | 30 | 50.0% | - | - | 84.1% | 86.7% | 9,554 / 11,502 | 14,583 / 1,692 | 189 / 386 |
| single-session-user | 70 | 92.9% | - | - | 100.0% | 100.0% | 8,736 / 14,830 | 13,266 / 451 | 137 / 431 |
| temporal-reasoning | 133 | 86.5% | - | - | 93.7% | 84.3% | 14,826 / 19,802 | 21,568 / 791 | 294 / 762 |
| overall | 500 | 83.6% | - | - | 94.4% | 91.1% | 12,490 / 19,310 | 18,414 / 774 | 235 / 651 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| knowledge-update | 0.965 | 24.6% | 100.0% | 9.8× | 1070 | 164 |
| multi-session | 0.897 | 25.0% | 100.0% | 8.7× | 752 | 171 |
| single-session-assistant | 0.935 | 13.0% | 100.0% | 13.3× | 752 | 147 |
| single-session-preference | 0.708 | 12.9% | 86.7% | 12.9× | 392 | 414 |
| single-session-user | 0.923 | 14.8% | 100.0% | 14.0× | 516 | 130 |
| temporal-reasoning | 0.856 | 18.1% | 97.6% | 8.3× | 1161 | 156 |
| overall | 0.892 | 19.5% | 98.5% | 9.8× | 978 | 172 |

| category | evidence turn coverage (share of its sentences in the context) | evidence turns complete (≥90%) |
|---|---|---|
| knowledge-update | 97.2% | 97.2% |
| multi-session | 93.0% | 92.9% |
| single-session-assistant | 89.6% | 75.0% |
| single-session-preference | 84.1% | 84.1% |
| single-session-user | 98.8% | 97.0% |
| temporal-reasoning | 93.8% | 93.7% |
| overall | 93.7% | 92.6% |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| knowledge-update | 97.1% | 33.3% | 95.8% |
| multi-session | 84.9% | 13.3% | 87.6% |
| single-session-assistant | 91.1% | - | 100.0% |
| single-session-preference | 57.7% | 0.0% | 86.7% |
| single-session-user | 92.2% | - | 100.0% |
| temporal-reasoning | 93.5% | 45.0% | 84.3% |
| overall | 89.3% | 28.6% | 91.1% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

Abstention accuracy: 80.0%

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 500 | 9,206,957 | 387,053 | 201,926 | 18,414 / 774 | 0.0000 | 8799 / 13493 |
| judge | 500 | 733,499 | 225,787 | 211,739 | 1,467 / 452 | 0.0000 | 7307 / 13594 |

Per question, answer + judge, including cached replies: 19,881 prompt tokens, 1,226 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
