# longmemeval · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| knowledge-update | 10 | 100.0% | - | - | 100.0% | 100.0% | 8,315 / 10,865 | 12,942 / 577 | 59 / 96 |
| multi-session | 10 | 70.0% | - | - | 87.5% | 77.8% | 9,438 / 10,957 | 14,162 / 727 | 14 / 16 |
| single-session-assistant | 10 | 100.0% | - | - | 100.0% | 100.0% | 9,008 / 12,300 | 13,855 / 754 | 18 / 24 |
| single-session-preference | 10 | 50.0% | - | - | 73.3% | 80.0% | 9,477 / 11,218 | 14,748 / 1,792 | 13 / 20 |
| single-session-user | 10 | 100.0% | - | - | 100.0% | 100.0% | 6,784 / 10,064 | 10,642 / 442 | 10 / 12 |
| temporal-reasoning | 10 | 70.0% | - | - | 83.3% | 77.8% | 8,900 / 10,765 | 13,641 / 854 | 13 / 30 |
| overall | 60 | 81.7% | - | - | 89.8% | 89.7% | 8,654 / 10,957 | 13,332 / 858 | 13 / 89 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| knowledge-update | 1.000 | 35.7% | 100.0% | 14.8× | 96 | 153 |
| multi-session | 0.889 | 28.6% | 100.0% | 13.1× | 16 | 164 |
| single-session-assistant | 1.000 | 14.1% | 100.0% | 13.5× | 24 | 163 |
| single-session-preference | 0.500 | 12.9% | 80.0% | 12.9× | 20 | 422 |
| single-session-user | 1.000 | 22.0% | 100.0% | 18.0× | 12 | 120 |
| temporal-reasoning | 0.815 | 28.1% | 100.0% | 13.8× | 30 | 175 |
| overall | 0.868 | 23.4% | 96.6% | 14.2× | 96 | 200 |

| category | evidence turn coverage (share of its sentences in the context) | evidence turns complete (≥90%) |
|---|---|---|
| knowledge-update | 100.0% | 100.0% |
| multi-session | 83.3% | 83.3% |
| single-session-assistant | 99.2% | 100.0% |
| single-session-preference | 73.3% | 73.3% |
| single-session-user | 96.5% | 90.9% |
| temporal-reasoning | 83.3% | 83.3% |
| overall | 88.3% | 87.8% |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| knowledge-update | 100.0% | - | 100.0% |
| multi-session | 85.7% | 0.0% | 77.8% |
| single-session-assistant | 100.0% | - | 100.0% |
| single-session-preference | 62.5% | 0.0% | 80.0% |
| single-session-user | 100.0% | - | 100.0% |
| temporal-reasoning | 71.4% | 50.0% | 77.8% |
| overall | 88.5% | 16.7% | 89.7% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

Abstention accuracy: 100.0%

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 60 | 799,895 | 51,452 | 25,384 | 13,332 / 858 | 0.0000 | 7888 / 15236 |
| judge | 60 | 90,452 | 29,507 | 27,074 | 1,508 / 492 | 0.0000 | 6511 / 17996 |

Per question, answer + judge, including cached replies: 14,839 prompt tokens, 1,349 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
