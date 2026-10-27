# longmemeval · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| knowledge-update | 10 | 100.0% | - | - | 100.0% | 100.0% | 11,221 / 20,589 | 16,886 / 593 | 56 / 124 |
| multi-session | 10 | 80.0% | - | - | 100.0% | 88.9% | 15,934 / 19,670 | 22,771 / 822 | 65 / 106 |
| single-session-assistant | 10 | 100.0% | - | - | 100.0% | 100.0% | 10,091 / 16,929 | 15,049 / 726 | 60 / 83 |
| single-session-preference | 10 | 60.0% | - | - | 80.0% | 90.0% | 9,664 / 11,373 | 14,877 / 1,776 | 40 / 52 |
| single-session-user | 10 | 100.0% | - | - | 100.0% | 100.0% | 8,134 / 9,617 | 12,385 / 448 | 34 / 40 |
| temporal-reasoning | 10 | 100.0% | - | - | 94.4% | 77.8% | 14,624 / 18,543 | 21,332 / 814 | 62 / 152 |
| overall | 60 | 90.0% | - | - | 95.9% | 93.1% | 11,611 / 19,423 | 17,217 / 863 | 51 / 124 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| knowledge-update | 1.000 | 31.8% | 100.0% | 11.0× | 124 | 154 |
| multi-session | 1.000 | 25.7% | 100.0% | 7.8× | 106 | 165 |
| single-session-assistant | 1.000 | 14.0% | 100.0% | 12.1× | 83 | 169 |
| single-session-preference | 0.725 | 13.9% | 90.0% | 12.7× | 52 | 406 |
| single-session-user | 0.950 | 14.6% | 100.0% | 15.0× | 40 | 110 |
| temporal-reasoning | 0.889 | 19.9% | 100.0% | 8.4× | 152 | 164 |
| overall | 0.927 | 19.9% | 98.3% | 10.6× | 152 | 195 |

| category | evidence turn coverage (share of its sentences in the context) | evidence turns complete (≥90%) |
|---|---|---|
| knowledge-update | 100.0% | 100.0% |
| multi-session | 95.8% | 95.8% |
| single-session-assistant | 99.2% | 100.0% |
| single-session-preference | 80.0% | 80.0% |
| single-session-user | 96.2% | 90.9% |
| temporal-reasoning | 94.4% | 94.4% |
| overall | 94.4% | 93.9% |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| knowledge-update | 100.0% | - | 100.0% |
| multi-session | 87.5% | 0.0% | 88.9% |
| single-session-assistant | 100.0% | - | 100.0% |
| single-session-preference | 66.7% | 0.0% | 90.0% |
| single-session-user | 100.0% | - | 100.0% |
| temporal-reasoning | 100.0% | 100.0% | 77.8% |
| overall | 92.6% | 50.0% | 93.1% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

Abstention accuracy: 100.0%

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 60 | 1,033,009 | 51,780 | 26,402 | 17,217 / 863 | 0.0000 | 8174 / 15857 |
| judge | 60 | 90,003 | 30,572 | 29,242 | 1,500 / 510 | 0.0000 | 6627 / 12710 |

Per question, answer + judge, including cached replies: 18,717 prompt tokens, 1,373 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
