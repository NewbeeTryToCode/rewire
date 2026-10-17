# longmemeval · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| knowledge-update | 10 | 100.0% | - | - | 100.0% | 100.0% | 17,308 / 27,280 | 25,236 / 625 | 739 / 975 |
| multi-session | 10 | 70.0% | - | - | 100.0% | 100.0% | 22,628 / 27,801 | 31,840 / 789 | 412 / 751 |
| single-session-assistant | 10 | 100.0% | - | - | 100.0% | 100.0% | 14,315 / 26,873 | 20,919 / 748 | 146 / 328 |
| single-session-preference | 10 | 30.0% | - | - | 86.7% | 90.0% | 14,431 / 18,000 | 21,578 / 1,900 | 162 / 207 |
| single-session-user | 10 | 90.0% | - | - | 100.0% | 100.0% | 12,528 / 15,005 | 18,331 / 448 | 99 / 203 |
| temporal-reasoning | 10 | 100.0% | - | - | 100.0% | 100.0% | 23,297 / 32,944 | 32,982 / 940 | 223 / 316 |
| overall | 60 | 81.7% | - | - | 98.0% | 98.3% | 17,418 / 29,025 | 25,148 / 908 | 203 / 822 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| knowledge-update | 1.000 | 28.0% | 100.0% | 7.1× | 975 | 163 |
| multi-session | 0.944 | 22.2% | 100.0% | 5.5× | 751 | 174 |
| single-session-assistant | 0.870 | 14.0% | 100.0% | 8.5× | 328 | 168 |
| single-session-preference | 0.783 | 16.0% | 90.0% | 8.5× | 207 | 448 |
| single-session-user | 0.767 | 15.0% | 100.0% | 9.8× | 203 | 125 |
| temporal-reasoning | 0.796 | 21.1% | 100.0% | 5.3× | 316 | 154 |
| overall | 0.860 | 19.3% | 98.3% | 7.0× | 975 | 205 |

| category | evidence turn coverage (share of its sentences in the context) | evidence turns complete (≥90%) |
|---|---|---|
| knowledge-update | 100.0% | 100.0% |
| multi-session | 100.0% | 100.0% |
| single-session-assistant | 99.2% | 100.0% |
| single-session-preference | 86.7% | 86.7% |
| single-session-user | 96.2% | 90.9% |
| temporal-reasoning | 100.0% | 100.0% |
| overall | 97.4% | 96.9% |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| knowledge-update | 100.0% | - | 100.0% |
| multi-session | 66.7% | - | 100.0% |
| single-session-assistant | 100.0% | - | 100.0% |
| single-session-preference | 33.3% | 0.0% | 90.0% |
| single-session-user | 90.0% | - | 100.0% |
| temporal-reasoning | 100.0% | - | 100.0% |
| overall | 82.5% | 0.0% | 98.3% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

Abstention accuracy: 100.0%

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| expand | 60 | 94,781 | 4,378 | 242 | 1,580 / 73 | 0.0000 | 4902 / 6312 |
| answer | 60 | 1,508,862 | 54,495 | 27,914 | 25,148 / 908 | 0.0000 | 8589 / 16400 |
| judge | 60 | 90,810 | 35,753 | 33,702 | 1,514 / 596 | 0.0000 | 6596 / 25960 |

Per question, answer + judge, including cached replies: 26,661 prompt tokens, 1,504 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
