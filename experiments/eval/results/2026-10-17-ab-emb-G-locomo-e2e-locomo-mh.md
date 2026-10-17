# locomo · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| multi-hop | 40 | 87.5% | 0.191 | 0.131 | 66.4% | 45.0% | 2,964 / 7,287 | 7,975 / 761 | 114 / 440 |
| overall | 40 | 87.5% | 0.191 | 0.131 | 66.4% | 45.0% | 2,964 / 7,287 | 7,975 / 761 | 114 / 440 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| multi-hop | 0.730 | 23.1% | - | 6.2× | 478 | 42 |
| overall | 0.730 | 23.1% | - | 6.2× | 478 | 42 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| multi-hop | 88.9% | 86.4% | 45.0% |
| overall | 88.9% | 86.4% | 45.0% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

| category | all evidence (excerpts) | all evidence (excerpts or a fact from the turn) | list answers n | list J | list items found |
|---|---|---|---|---|---|
| multi-hop | 45.0% | 45.0% | 14 | 85.7% | 39.9% |
| overall | 45.0% | 45.0% | 14 | 85.7% | 39.9% |

List J follows the official judge (one right item is enough); list items found is the share of gold items present in the answer, without an LLM.

| category | J | strict J (lists must be complete) | list answers n | list items named (LLM) | list answers complete |
|---|---|---|---|---|---|
| multi-hop | 87.5% | 70.0% | 14 | 60.1% | 35.7% |
| overall | 87.5% | 70.0% | 14 | 60.1% | 35.7% |

The list judge (same model as the judge) splits the gold list and counts the items the answer names, paraphrases included.

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 40 | 318,994 | 30,438 | 17,565 | 7,975 / 761 | 0.0000 | 8355 / 11952 |
| judge | 40 | 73,120 | 25,168 | 22,080 | 1,828 / 629 | 0.0000 | 9171 / 14795 |
| list_judge | 14 | 17,226 | 4,383 | 4,097 | 1,230 / 313 | 0.0000 | 6722 / 9907 |

Retrieval tokens per question: context 2,964 (p95 7,287); extra-query LLM calls 274 prompt + 12 completion on average over all questions, 1,568 + 70 on the 7 questions given extra queries.

Per question, answer + judge, including cached replies: 9,803 prompt tokens, 1,390 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
