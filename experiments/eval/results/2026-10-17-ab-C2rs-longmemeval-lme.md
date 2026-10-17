# longmemeval · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| knowledge-update | 10 | 100.0% | - | - | 100.0% | 100.0% | 14,779 / 27,280 | 21,638 / 619 | 286 / 445 |
| multi-session | 10 | 70.0% | - | - | 100.0% | 100.0% | 21,815 / 27,801 | 30,752 / 794 | 332 / 379 |
| single-session-assistant | 10 | 100.0% | - | - | 100.0% | 100.0% | 12,921 / 26,873 | 19,182 / 742 | 271 / 537 |
| single-session-preference | 10 | 60.0% | - | - | 80.0% | 90.0% | 9,664 / 11,373 | 14,877 / 1,776 | 229 / 307 |
| single-session-user | 10 | 100.0% | - | - | 100.0% | 100.0% | 8,483 / 11,676 | 12,852 / 432 | 210 / 259 |
| temporal-reasoning | 10 | 100.0% | - | - | 100.0% | 100.0% | 22,713 / 32,944 | 32,188 / 958 | 303 / 395 |
| overall | 60 | 88.3% | - | - | 96.9% | 98.3% | 15,062 / 29,025 | 21,915 / 887 | 252 / 395 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| knowledge-update | 1.000 | 31.3% | 100.0% | 8.3× | 445 | 158 |
| multi-session | 1.000 | 23.9% | 100.0% | 5.7× | 379 | 178 |
| single-session-assistant | 0.870 | 15.2% | 100.0% | 9.4× | 537 | 166 |
| single-session-preference | 0.725 | 13.9% | 90.0% | 12.7× | 307 | 406 |
| single-session-user | 0.950 | 15.2% | 100.0% | 14.4× | 259 | 109 |
| temporal-reasoning | 0.815 | 20.6% | 100.0% | 5.4× | 395 | 159 |
| overall | 0.893 | 19.9% | 98.3% | 8.1× | 537 | 196 |

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
| multi-session | 66.7% | - | 100.0% |
| single-session-assistant | 100.0% | - | 100.0% |
| single-session-preference | 66.7% | 0.0% | 90.0% |
| single-session-user | 100.0% | - | 100.0% |
| temporal-reasoning | 100.0% | - | 100.0% |
| overall | 89.5% | 0.0% | 98.3% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

Abstention accuracy: 100.0%

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 1 | 18,585 | 486 | 140 | 18,585 / 486 | 0.0000 | 4847 / 4847 |
| judge | 1 | 1,469 | 533 | 526 | 1,469 / 533 | 0.0000 | 7479 / 7479 |

Per question, answer + judge, including cached replies: 23,416 prompt tokens, 1,396 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
