# locomo · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| multi-hop | 10 | 90.0% | 0.186 | 0.112 | 71.9% | 60.0% | 2,422 / 4,480 | 7,212 / 712 | 192 / 245 |
| open-domain | 10 | 80.0% | 0.140 | 0.089 | 52.9% | 50.0% | 2,526 / 4,262 | 7,328 / 800 | 113 / 181 |
| single-hop | 10 | 100.0% | 0.357 | 0.255 | 100.0% | 100.0% | 2,507 / 4,532 | 7,329 / 476 | 106 / 118 |
| temporal | 10 | 90.0% | 0.191 | 0.102 | 86.7% | 80.0% | 3,619 / 7,204 | 8,994 / 645 | 111 / 147 |
| overall | 40 | 90.0% | 0.219 | 0.140 | 70.7% | 72.5% | 2,769 / 5,294 | 7,716 / 658 | 111 / 239 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| multi-hop | 0.767 | 32.7% | - | 7.5× | 245 | 33 |
| open-domain | 0.800 | 16.5% | - | 7.6× | 181 | 43 |
| single-hop | 0.933 | 13.9% | - | 7.1× | 118 | 22 |
| temporal | 0.800 | 12.9% | - | 4.9× | 147 | 28 |
| overall | 0.825 | 19.0% | - | 6.6× | 245 | 31 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| multi-hop | 100.0% | 75.0% | 60.0% |
| open-domain | 80.0% | 80.0% | 50.0% |
| single-hop | 100.0% | - | 100.0% |
| temporal | 100.0% | 50.0% | 80.0% |
| overall | 96.6% | 72.7% | 72.5% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

| category | all evidence (excerpts) | all evidence (excerpts or a fact from the turn) | list answers n | list J | list items found |
|---|---|---|---|---|---|
| multi-hop | 60.0% | 60.0% | 5 | 100.0% | 70.0% |
| open-domain | 50.0% | 50.0% | - | - | - |
| single-hop | 100.0% | 100.0% | 2 | 100.0% | 100.0% |
| temporal | 80.0% | 80.0% | - | - | - |
| overall | 72.5% | 72.5% | 7 | 100.0% | 78.6% |

List J follows the official judge (one right item is enough); list items found is the share of gold items present in the answer, without an LLM.

| category | J | strict J (lists must be complete) | list answers n | list items named (LLM) | list answers complete |
|---|---|---|---|---|---|
| multi-hop | 90.0% | 80.0% | 5 | 90.0% | 80.0% |
| open-domain | 80.0% | 80.0% | 0 | - | - |
| single-hop | 100.0% | 100.0% | 2 | 100.0% | 100.0% |
| temporal | 90.0% | 90.0% | 0 | - | - |
| overall | 90.0% | 87.5% | 7 | 92.9% | 85.7% |

The list judge (same model as the judge) splits the gold list and counts the items the answer names, paraphrases included.

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|

Retrieval tokens per question: context 2,769 (p95 5,294); extra-query LLM calls 510 prompt + 23 completion on average over all questions, 1,571 + 71 on the 13 questions given extra queries.

Per question, answer + judge, including cached replies: 9,530 prompt tokens, 1,354 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
