# locomo · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| multi-hop | 40 | 90.0% | 0.171 | 0.117 | 64.8% | 45.0% | 2,919 / 7,518 | 7,926 / 772 | 843 / 4105 |
| overall | 40 | 90.0% | 0.171 | 0.117 | 64.8% | 45.0% | 2,919 / 7,518 | 7,926 / 772 | 843 / 4105 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| multi-hop | 0.704 | 20.9% | - | 6.3× | 4217 | 46 |
| overall | 0.704 | 20.9% | - | 6.3× | 4217 | 46 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| multi-hop | 88.9% | 90.9% | 45.0% |
| overall | 88.9% | 90.9% | 45.0% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

| category | all evidence (excerpts) | all evidence (excerpts or a fact from the turn) | list answers n | list J | list items found |
|---|---|---|---|---|---|
| multi-hop | 45.0% | 45.0% | 14 | 92.9% | 53.6% |
| overall | 45.0% | 45.0% | 14 | 92.9% | 53.6% |

List J follows the official judge (one right item is enough); list items found is the share of gold items present in the answer, without an LLM.

| category | J | strict J (lists must be complete) | list answers n | list items named (LLM) | list answers complete |
|---|---|---|---|---|---|
| multi-hop | 90.0% | 70.0% | 14 | 60.1% | 35.7% |
| overall | 90.0% | 70.0% | 14 | 60.1% | 35.7% |

The list judge (same model as the judge) splits the gold list and counts the items the answer names, paraphrases included.

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 40 | 317,045 | 30,899 | 18,411 | 7,926 / 772 | 0.0000 | 8325 / 12202 |
| judge | 40 | 73,340 | 25,635 | 22,527 | 1,834 / 641 | 0.0000 | 8931 / 15107 |
| list_judge | 14 | 17,538 | 4,346 | 4,057 | 1,253 / 310 | 0.0000 | 7090 / 8778 |

Retrieval tokens per question: context 2,919 (p95 7,518); extra-query LLM calls 275 prompt + 13 completion on average over all questions, 1,569 + 72 on the 7 questions given extra queries.

Per question, answer + judge, including cached replies: 9,760 prompt tokens, 1,413 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
