# locomo · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| multi-hop | 10 | 100.0% | 0.194 | 0.123 | 78.1% | 70.0% | 2,484 / 4,370 | 7,278 / 738 | 238 / 703 |
| open-domain | 10 | 80.0% | 0.127 | 0.079 | 55.9% | 60.0% | 2,368 / 3,723 | 7,125 / 859 | 112 / 553 |
| single-hop | 10 | 100.0% | 0.383 | 0.268 | 100.0% | 100.0% | 2,462 / 4,018 | 7,252 / 519 | 119 / 573 |
| temporal | 10 | 90.0% | 0.287 | 0.207 | 93.3% | 90.0% | 3,494 / 7,750 | 8,826 / 548 | 330 / 615 |
| overall | 40 | 92.5% | 0.248 | 0.169 | 75.0% | 80.0% | 2,702 / 4,690 | 7,620 / 666 | 148 / 615 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| multi-hop | 0.853 | 35.9% | - | 7.3× | 703 | 29 |
| open-domain | 0.800 | 18.4% | - | 8.1× | 553 | 51 |
| single-hop | 0.883 | 14.6% | - | 7.3× | 573 | 18 |
| temporal | 0.725 | 12.4% | - | 5.1× | 615 | 23 |
| overall | 0.815 | 20.3% | - | 6.7× | 703 | 30 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| multi-hop | 100.0% | 100.0% | 70.0% |
| open-domain | 83.3% | 75.0% | 60.0% |
| single-hop | 100.0% | - | 100.0% |
| temporal | 100.0% | 0.0% | 90.0% |
| overall | 96.9% | 75.0% | 80.0% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

| category | all evidence (excerpts) | all evidence (excerpts or a fact from the turn) | list answers n | list J | list items found |
|---|---|---|---|---|---|
| multi-hop | 70.0% | 70.0% | 5 | 100.0% | 70.0% |
| open-domain | 60.0% | 60.0% | - | - | - |
| single-hop | 100.0% | 100.0% | 2 | 100.0% | 100.0% |
| temporal | 90.0% | 90.0% | - | - | - |
| overall | 80.0% | 80.0% | 7 | 100.0% | 78.6% |

List J follows the official judge (one right item is enough); list items found is the share of gold items present in the answer, without an LLM.

| category | J | strict J (lists must be complete) | list answers n | list items named (LLM) | list answers complete |
|---|---|---|---|---|---|
| multi-hop | 100.0% | 90.0% | 5 | 90.0% | 80.0% |
| open-domain | 80.0% | 80.0% | 0 | - | - |
| single-hop | 100.0% | 100.0% | 2 | 100.0% | 100.0% |
| temporal | 90.0% | 90.0% | 0 | - | - |
| overall | 92.5% | 90.0% | 7 | 92.9% | 85.7% |

The list judge (same model as the judge) splits the gold list and counts the items the answer names, paraphrases included.

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 37 | 284,021 | 24,169 | 12,956 | 7,676 / 653 | 0.0000 | 7809 / 11063 |
| judge | 34 | 61,649 | 28,344 | 25,701 | 1,813 / 834 | 0.0000 | 9598 / 23385 |
| list_judge | 5 | 6,051 | 1,841 | 1,732 | 1,210 / 368 | 0.0000 | 6583 / 9674 |

Retrieval tokens per question: context 2,702 (p95 4,690); extra-query LLM calls 510 prompt + 23 completion on average over all questions, 1,571 + 71 on the 13 questions given extra queries.

Per question, answer + judge, including cached replies: 9,430 prompt tokens, 1,451 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
