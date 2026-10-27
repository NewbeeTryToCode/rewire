# locomo · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| multi-hop | 40 | 92.5% | 0.178 | 0.124 | 59.4% | 32.5% | 2,952 / 7,960 | 7,968 / 763 | 105 / 132 |
| overall | 40 | 92.5% | 0.178 | 0.124 | 59.4% | 32.5% | 2,952 / 7,960 | 7,968 / 763 | 105 / 132 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| multi-hop | 0.620 | 20.3% | - | 6.2× | 153 | 38 |
| overall | 0.620 | 20.3% | - | 6.2× | 153 | 38 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| multi-hop | 92.3% | 92.6% | 32.5% |
| overall | 92.3% | 92.6% | 32.5% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

| category | all evidence (excerpts) | all evidence (excerpts or a fact from the turn) | list answers n | list J | list items found |
|---|---|---|---|---|---|
| multi-hop | 32.5% | 32.5% | 14 | 92.9% | 44.0% |
| overall | 32.5% | 32.5% | 14 | 92.9% | 44.0% |

List J follows the official judge (one right item is enough); list items found is the share of gold items present in the answer, without an LLM.

| category | J | strict J (lists must be complete) | list answers n | list items named (LLM) | list answers complete |
|---|---|---|---|---|---|
| multi-hop | 92.5% | 70.0% | 14 | 56.0% | 28.6% |
| overall | 92.5% | 70.0% | 14 | 56.0% | 28.6% |

The list judge (same model as the judge) splits the gold list and counts the items the answer names, paraphrases included.

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|

Retrieval tokens per question: context 2,952 (p95 7,960); extra-query LLM calls 274 prompt + 13 completion on average over all questions, 1,568 + 76 on the 7 questions given extra queries.

Per question, answer + judge, including cached replies: 9,790 prompt tokens, 1,485 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
