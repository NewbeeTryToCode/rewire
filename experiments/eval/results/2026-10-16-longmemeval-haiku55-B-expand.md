# longmemeval · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| knowledge-update | 78 | 89.7% | - | - | 98.6% | 98.6% | 19,433 / 28,861 | 27,902 / 675 | 1027 / 2120 |
| multi-session | 133 | 79.7% | - | - | 98.5% | 98.3% | 21,863 / 31,035 | 31,035 / 924 | 1167 / 4000 |
| single-session-assistant | 56 | 91.1% | - | - | 94.6% | 100.0% | 12,812 / 24,215 | 18,857 / 605 | 682 / 1159 |
| single-session-preference | 30 | 56.7% | - | - | 93.2% | 96.7% | 14,001 / 16,942 | 20,940 / 1,842 | 689 / 887 |
| single-session-user | 70 | 95.7% | - | - | 100.0% | 100.0% | 14,313 / 23,906 | 20,842 / 440 | 2108 / 3536 |
| temporal-reasoning | 133 | 88.0% | - | - | 98.4% | 96.9% | 22,525 / 31,261 | 31,887 / 846 | 1243 / 2376 |
| overall | 500 | 85.6% | - | - | 98.1% | 98.3% | 19,117 / 30,067 | 27,376 / 816 | 1119 / 3404 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| knowledge-update | 0.938 | 22.7% | 100.0% | 6.3× | 2546 | 163 |
| multi-session | 0.909 | 23.3% | 100.0% | 5.6× | 4415 | 178 |
| single-session-assistant | 0.878 | 11.2% | 100.0% | 9.6× | 1203 | 150 |
| single-session-preference | 0.773 | 14.3% | 96.7% | 8.8× | 1008 | 432 |
| single-session-user | 0.816 | 13.5% | 100.0% | 8.5× | 4580 | 128 |
| temporal-reasoning | 0.780 | 17.3% | 98.4% | 5.4× | 3225 | 164 |
| overall | 0.853 | 18.2% | 99.4% | 6.4× | 4387 | 177 |

| category | evidence turn coverage (share of its sentences in the context) | evidence turns complete (≥90%) |
|---|---|---|
| knowledge-update | 98.6% | 98.6% |
| multi-session | 98.3% | 98.1% |
| single-session-assistant | 89.8% | 76.8% |
| single-session-preference | 93.2% | 93.2% |
| single-session-user | 99.0% | 97.0% |
| temporal-reasoning | 98.4% | 98.4% |
| overall | 97.6% | 96.6% |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| knowledge-update | 94.4% | 0.0% | 98.6% |
| multi-session | 82.4% | 50.0% | 98.3% |
| single-session-assistant | 91.1% | - | 100.0% |
| single-session-preference | 58.6% | 0.0% | 96.7% |
| single-session-user | 95.3% | - | 100.0% |
| temporal-reasoning | 90.2% | 0.0% | 96.9% |
| overall | 87.7% | 12.5% | 98.3% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

Abstention accuracy: 73.3%

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| judge | 403 | 601,760 | 189,282 | 177,210 | 1,493 / 470 | 0.0000 | 7277 / 14642 |
| answer | 392 | 10,856,749 | 332,263 | 177,162 | 27,696 / 848 | 0.0000 | 9060 / 15432 |

Per question, answer + judge, including cached replies: 28,851 prompt tokens, 1,262 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
