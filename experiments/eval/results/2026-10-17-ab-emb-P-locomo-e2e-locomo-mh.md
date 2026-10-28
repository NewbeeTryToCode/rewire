# locomo · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| multi-hop | 40 | 87.5% | 0.191 | 0.134 | 59.4% | 35.0% | 2,914 / 6,880 | 7,905 / 806 | 10 / 34 |
| overall | 40 | 87.5% | 0.191 | 0.134 | 59.4% | 35.0% | 2,914 / 6,880 | 7,905 / 806 | 10 / 34 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| multi-hop | 0.633 | 20.4% | - | 6.3× | 46 | 40 |
| overall | 0.633 | 20.4% | - | 6.3× | 46 | 40 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| multi-hop | 85.7% | 88.5% | 35.0% |
| overall | 85.7% | 88.5% | 35.0% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

| category | all evidence (excerpts) | all evidence (excerpts or a fact from the turn) | list answers n | list J | list items found |
|---|---|---|---|---|---|
| multi-hop | 35.0% | 35.0% | 14 | 92.9% | 38.1% |
| overall | 35.0% | 35.0% | 14 | 92.9% | 38.1% |

List J follows the official judge (one right item is enough); list items found is the share of gold items present in the answer, without an LLM.

| category | J | strict J (lists must be complete) | list answers n | list items named (LLM) | list answers complete |
|---|---|---|---|---|---|
| multi-hop | 87.5% | 65.0% | 14 | 56.0% | 28.6% |
| overall | 87.5% | 65.0% | 14 | 56.0% | 28.6% |

The list judge (same model as the judge) splits the gold list and counts the items the answer names, paraphrases included.

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 40 | 316,201 | 32,233 | 18,724 | 7,905 / 806 | 0.0000 | 8369 / 11040 |
| judge | 40 | 72,965 | 26,313 | 23,155 | 1,824 / 658 | 0.0000 | 8865 / 15551 |
| list_judge | 14 | 17,332 | 4,789 | 4,484 | 1,238 / 342 | 0.0000 | 6655 / 10069 |

Retrieval tokens per question: context 2,914 (p95 6,880); extra-query LLM calls 274 prompt + 12 completion on average over all questions, 1,568 + 71 on the 7 questions given extra queries.

Per question, answer + judge, including cached replies: 9,729 prompt tokens, 1,464 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
