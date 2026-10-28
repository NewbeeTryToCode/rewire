# locomo · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| multi-hop | 10 | 90.0% | 0.186 | 0.112 | 68.8% | 60.0% | 2,284 / 3,124 | 6,999 / 711 | 9 / 21 |
| open-domain | 10 | 70.0% | 0.140 | 0.089 | 52.9% | 50.0% | 2,395 / 2,956 | 7,151 / 783 | 9 / 16 |
| single-hop | 10 | 100.0% | 0.364 | 0.260 | 100.0% | 100.0% | 2,316 / 2,806 | 7,045 / 459 | 8 / 16 |
| temporal | 10 | 90.0% | 0.287 | 0.198 | 86.7% | 80.0% | 2,624 / 5,383 | 7,527 / 635 | 8 / 25 |
| overall | 40 | 87.5% | 0.244 | 0.165 | 69.6% | 72.5% | 2,405 / 3,350 | 7,180 / 647 | 8 / 21 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| multi-hop | 0.733 | 32.2% | - | 7.9× | 21 | 33 |
| open-domain | 0.800 | 16.2% | - | 8.0× | 16 | 43 |
| single-hop | 0.933 | 13.6% | - | 7.7× | 16 | 22 |
| temporal | 0.817 | 13.9% | - | 6.7× | 25 | 21 |
| overall | 0.821 | 19.0% | - | 7.6× | 25 | 30 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| multi-hop | 100.0% | 75.0% | 60.0% |
| open-domain | 80.0% | 60.0% | 50.0% |
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
| multi-hop | 90.0% | 80.0% | 5 | 90.0% | 80.0% |
| open-domain | 70.0% | 70.0% | 0 | - | - |
| single-hop | 100.0% | 100.0% | 2 | 100.0% | 100.0% |
| temporal | 90.0% | 90.0% | 0 | - | - |
| overall | 87.5% | 85.0% | 7 | 92.9% | 85.7% |

The list judge (same model as the judge) splits the gold list and counts the items the answer names, paraphrases included.

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 40 | 287,218 | 25,882 | 14,482 | 7,180 / 647 | 0.0000 | 7534 / 9911 |
| judge | 40 | 72,470 | 28,799 | 25,835 | 1,812 / 720 | 0.0000 | 8631 / 24192 |
| list_judge | 7 | 8,536 | 2,242 | 2,102 | 1,219 / 320 | 0.0000 | 6079 / 7675 |

Per question, answer + judge, including cached replies: 8,992 prompt tokens, 1,367 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
