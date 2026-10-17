# longmemeval · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| knowledge-update | 78 | 93.6% | - | - | 97.2% | 95.8% | 8,851 / 10,986 | 13,720 / 628 | 19 / 55 |
| multi-session | 133 | 65.4% | - | - | 79.6% | 78.5% | 8,955 / 11,203 | 13,749 / 807 | 19 / 40 |
| single-session-assistant | 56 | 85.7% | - | - | 98.2% | 100.0% | 8,555 / 10,254 | 13,144 / 623 | 29 / 70 |
| single-session-preference | 30 | 43.3% | - | - | 75.0% | 76.7% | 9,520 / 11,218 | 14,884 / 1,713 | 22 / 34 |
| single-session-user | 70 | 94.3% | - | - | 100.0% | 100.0% | 7,539 / 9,999 | 11,721 / 436 | 19 / 214 |
| temporal-reasoning | 133 | 78.9% | - | - | 84.2% | 75.6% | 9,379 / 11,219 | 14,321 / 801 | 23 / 53 |
| overall | 500 | 78.4% | - | - | 86.2% | 85.7% | 8,842 / 11,052 | 13,613 / 760 | 21 / 56 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| knowledge-update | 0.951 | 32.2% | 98.6% | 13.8× | 88 | 160 |
| multi-session | 0.844 | 32.1% | 99.2% | 13.7× | 47 | 172 |
| single-session-assistant | 0.982 | 13.6% | 100.0% | 14.3× | 99 | 150 |
| single-session-preference | 0.517 | 11.8% | 76.7% | 12.9× | 39 | 409 |
| single-session-user | 0.943 | 18.5% | 100.0% | 16.2× | 237 | 127 |
| temporal-reasoning | 0.798 | 25.6% | 96.1% | 13.0× | 89 | 161 |
| overall | 0.857 | 25.0% | 97.0% | 13.8× | 195 | 173 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| knowledge-update | 98.6% | 0.0% | 95.8% |
| multi-session | 83.2% | 7.7% | 78.5% |
| single-session-assistant | 85.7% | - | 100.0% |
| single-session-preference | 56.5% | 0.0% | 76.7% |
| single-session-user | 93.8% | - | 100.0% |
| temporal-reasoning | 90.6% | 38.7% | 75.6% |
| overall | 88.1% | 20.9% | 85.7% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

Abstention accuracy: 76.7%

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 494 | 6,724,041 | 373,394 | 191,381 | 13,611 / 756 | 0.0000 | 8297 / 13531 |
| judge | 494 | 724,048 | 215,665 | 201,613 | 1,466 / 437 | 0.0000 | 7373 / 13698 |

Per question, answer + judge, including cached replies: 15,080 prompt tokens, 1,196 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
