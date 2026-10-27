# locomo · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| multi-hop | 40 | 95.0% | 0.159 | 0.114 | 59.4% | 32.5% | 2,917 / 7,674 | 7,917 / 780 | 9 / 48 |
| overall | 40 | 95.0% | 0.159 | 0.114 | 59.4% | 32.5% | 2,917 / 7,674 | 7,917 / 780 | 9 / 48 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| multi-hop | 0.615 | 20.3% | - | 6.3× | 50 | 43 |
| overall | 0.615 | 20.3% | - | 6.3× | 50 | 43 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| multi-hop | 100.0% | 92.6% | 32.5% |
| overall | 100.0% | 92.6% | 32.5% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

| category | all evidence (excerpts) | all evidence (excerpts or a fact from the turn) | list answers n | list J | list items found |
|---|---|---|---|---|---|
| multi-hop | 32.5% | 32.5% | 14 | 92.9% | 44.0% |
| overall | 32.5% | 32.5% | 14 | 92.9% | 44.0% |

List J follows the official judge (one right item is enough); list items found is the share of gold items present in the answer, without an LLM.

| category | J | strict J (lists must be complete) | list answers n | list items named (LLM) | list answers complete |
|---|---|---|---|---|---|
| multi-hop | 95.0% | 72.5% | 14 | 52.4% | 28.6% |
| overall | 95.0% | 72.5% | 14 | 52.4% | 28.6% |

The list judge (same model as the judge) splits the gold list and counts the items the answer names, paraphrases included.

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 40 | 316,665 | 31,212 | 17,776 | 7,917 / 780 | 0.0000 | 8939 / 11340 |
| judge | 40 | 73,143 | 24,999 | 21,817 | 1,829 / 625 | 0.0000 | 9182 / 13220 |
| list_judge | 14 | 17,310 | 5,864 | 5,568 | 1,236 / 419 | 0.0000 | 6908 / 15392 |

Retrieval tokens per question: context 2,917 (p95 7,674); extra-query LLM calls 275 prompt + 13 completion on average over all questions, 1,569 + 74 on the 7 questions given extra queries.

Per question, answer + judge, including cached replies: 9,745 prompt tokens, 1,405 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
