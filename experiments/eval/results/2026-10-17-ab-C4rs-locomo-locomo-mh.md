# locomo · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| multi-hop | 40 | 95.0% | 0.192 | 0.127 | 71.1% | 50.0% | 3,548 / 8,024 | 8,817 / 793 | 110 / 131 |
| overall | 40 | 95.0% | 0.192 | 0.127 | 71.1% | 50.0% | 3,548 / 8,024 | 8,817 / 793 | 110 / 131 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| multi-hop | 0.613 | 21.5% | - | 5.2× | 136 | 44 |
| overall | 0.613 | 21.5% | - | 5.2× | 136 | 44 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| multi-hop | 95.0% | 95.0% | 50.0% |
| overall | 95.0% | 95.0% | 50.0% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

| category | all evidence (excerpts) | all evidence (excerpts or a fact from the turn) | list answers n | list J | list items found |
|---|---|---|---|---|---|
| multi-hop | 50.0% | 50.0% | 14 | 100.0% | 61.9% |
| overall | 50.0% | 50.0% | 14 | 100.0% | 61.9% |

List J follows the official judge (one right item is enough); list items found is the share of gold items present in the answer, without an LLM.

| category | J | strict J (lists must be complete) | list answers n | list items named (LLM) | list answers complete |
|---|---|---|---|---|---|
| multi-hop | 95.0% | 77.5% | 14 | 73.2% | 50.0% |
| overall | 95.0% | 77.5% | 14 | 73.2% | 50.0% |

The list judge (same model as the judge) splits the gold list and counts the items the answer names, paraphrases included.

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|

Retrieval tokens per question: context 3,548 (p95 8,024); extra-query LLM calls 1,680 prompt + 211 completion on average over all questions, 1,680 + 240 on the 30 questions given extra queries.

Per question, answer + judge, including cached replies: 10,648 prompt tokens, 1,450 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
