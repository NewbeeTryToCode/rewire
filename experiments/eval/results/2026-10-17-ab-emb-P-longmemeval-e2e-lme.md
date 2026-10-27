# longmemeval · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| knowledge-update | 10 | 100.0% | - | - | 100.0% | 100.0% | 14,668 / 28,013 | 21,416 / 638 | 293 / 1994 |
| multi-session | 10 | 80.0% | - | - | 100.0% | 100.0% | 21,769 / 28,653 | 30,782 / 810 | 240 / 386 |
| single-session-assistant | 10 | 100.0% | - | - | 100.0% | 100.0% | 12,346 / 22,958 | 18,582 / 727 | 151 / 185 |
| single-session-preference | 10 | 70.0% | - | - | 80.0% | 100.0% | 9,375 / 11,539 | 14,321 / 1,703 | 95 / 326 |
| single-session-user | 10 | 100.0% | - | - | 100.0% | 100.0% | 8,487 / 11,542 | 12,947 / 436 | 84 / 186 |
| temporal-reasoning | 10 | 100.0% | - | - | 100.0% | 100.0% | 22,238 / 29,278 | 31,643 / 871 | 279 / 418 |
| overall | 60 | 91.7% | - | - | 96.9% | 100.0% | 14,814 / 28,653 | 21,615 / 864 | 151 / 565 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| knowledge-update | 1.000 | 27.7% | 100.0% | 8.4× | 1994 | 161 |
| multi-session | 1.000 | 23.3% | 100.0% | 5.7× | 386 | 167 |
| single-session-assistant | 0.853 | 16.2% | 100.0% | 9.9× | 185 | 147 |
| single-session-preference | 0.594 | 15.2% | 100.0% | 13.1× | 326 | 400 |
| single-session-user | 0.950 | 13.2% | 100.0% | 14.4× | 186 | 116 |
| temporal-reasoning | 0.720 | 21.5% | 100.0% | 5.5× | 418 | 168 |
| overall | 0.853 | 19.4% | 100.0% | 8.3× | 1994 | 193 |

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
| single-session-preference | 70.0% | - | 100.0% |
| single-session-user | 100.0% | - | 100.0% |
| temporal-reasoning | 100.0% | - | 100.0% |
| overall | 91.4% | - | 100.0% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

Abstention accuracy: 100.0%

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 59 | 1,284,623 | 51,279 | 26,445 | 21,773 / 869 | 0.0000 | 8277 / 14746 |
| judge | 59 | 88,266 | 29,358 | 27,689 | 1,496 / 498 | 0.0000 | 6957 / 16420 |

Retrieval tokens per question: context 14,814 (p95 28,653); extra-query LLM calls 791 prompt + 35 completion on average over all questions, 1,581 + 69 on the 30 questions given extra queries.

Per question, answer + judge, including cached replies: 23,110 prompt tokens, 1,358 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
