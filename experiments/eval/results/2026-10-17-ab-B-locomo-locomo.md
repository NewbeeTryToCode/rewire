# locomo · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| multi-hop | 10 | 80.0% | 0.232 | 0.148 | 81.2% | 60.0% | 3,397 / 4,480 | 8,644 / 807 | 28 / 47 |
| open-domain | 10 | 80.0% | 0.146 | 0.092 | 67.6% | 50.0% | 3,361 / 4,262 | 8,546 / 815 | 18 / 32 |
| single-hop | 10 | 100.0% | 0.487 | 0.377 | 100.0% | 100.0% | 3,282 / 5,020 | 8,443 / 372 | 19 / 30 |
| temporal | 10 | 90.0% | 0.191 | 0.102 | 86.7% | 80.0% | 3,619 / 7,204 | 8,994 / 645 | 20 / 56 |
| overall | 40 | 87.5% | 0.264 | 0.180 | 79.3% | 72.5% | 3,415 / 5,294 | 8,657 / 660 | 20 / 47 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| multi-hop | 0.664 | 28.5% | - | 5.3× | 47 | 26 |
| open-domain | 0.645 | 18.0% | - | 5.7× | 32 | 43 |
| single-hop | 0.933 | 14.0% | - | 5.4× | 30 | 14 |
| temporal | 0.800 | 12.9% | - | 4.9× | 56 | 28 |
| overall | 0.761 | 18.4% | - | 5.3× | 56 | 28 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| multi-hop | 100.0% | 50.0% | 60.0% |
| open-domain | 80.0% | 80.0% | 50.0% |
| single-hop | 100.0% | - | 100.0% |
| temporal | 100.0% | 50.0% | 80.0% |
| overall | 96.6% | 63.6% | 72.5% |

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
| multi-hop | 80.0% | 80.0% | 5 | 100.0% | 100.0% |
| open-domain | 80.0% | 80.0% | 0 | - | - |
| single-hop | 100.0% | 100.0% | 2 | 100.0% | 100.0% |
| temporal | 90.0% | 90.0% | 0 | - | - |
| overall | 87.5% | 87.5% | 7 | 100.0% | 100.0% |

The list judge (same model as the judge) splits the gold list and counts the items the answer names, paraphrases included.

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| expand | 40 | 62,861 | 2,792 | 0 | 1,572 / 70 | 0.0000 | 4725 / 5704 |
| answer | 40 | 346,270 | 26,392 | 15,136 | 8,657 / 660 | 0.0000 | 7793 / 10714 |
| judge | 40 | 72,306 | 28,767 | 25,765 | 1,808 / 719 | 0.0000 | 8625 / 26514 |
| list_judge | 7 | 8,489 | 2,190 | 2,045 | 1,213 / 313 | 0.0000 | 6065 / 8674 |

Per question, answer + judge, including cached replies: 10,464 prompt tokens, 1,379 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
