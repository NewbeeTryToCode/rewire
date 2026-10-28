# locomo · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| multi-hop | 10 | 80.0% | 0.177 | 0.116 | 65.6% | 40.0% | 1,984 / 3,104 | 6,549 / 685 | 6 / 11 |
| open-domain | 10 | 80.0% | 0.095 | 0.065 | 52.9% | 50.0% | 1,749 / 2,300 | 6,208 / 888 | 6 / 12 |
| single-hop | 10 | 100.0% | 0.330 | 0.245 | 100.0% | 100.0% | 1,986 / 2,483 | 6,562 / 478 | 8 / 9 |
| temporal | 10 | 90.0% | 0.254 | 0.148 | 86.7% | 80.0% | 1,930 / 2,439 | 6,517 / 606 | 6 / 23 |
| overall | 40 | 87.5% | 0.214 | 0.144 | 68.5% | 67.5% | 1,912 / 2,483 | 6,459 / 664 | 6 / 13 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| multi-hop | 0.733 | 28.9% | - | 9.1× | 11 | 30 |
| open-domain | 0.800 | 29.3% | - | 11.0× | 12 | 48 |
| single-hop | 0.917 | 14.1% | - | 9.0× | 9 | 21 |
| temporal | 0.883 | 17.1% | - | 9.2× | 23 | 21 |
| overall | 0.833 | 22.4% | - | 9.5× | 23 | 30 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| multi-hop | 100.0% | 66.7% | 40.0% |
| open-domain | 80.0% | 80.0% | 50.0% |
| single-hop | 100.0% | - | 100.0% |
| temporal | 100.0% | 50.0% | 80.0% |
| overall | 96.3% | 69.2% | 67.5% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

| category | all evidence (excerpts) | all evidence (excerpts or a fact from the turn) | list answers n | list J | list items found |
|---|---|---|---|---|---|
| multi-hop | 40.0% | 40.0% | 5 | 80.0% | 55.0% |
| open-domain | 50.0% | 50.0% | - | - | - |
| single-hop | 100.0% | 100.0% | 2 | 100.0% | 75.0% |
| temporal | 80.0% | 80.0% | - | - | - |
| overall | 67.5% | 67.5% | 7 | 85.7% | 60.7% |

List J follows the official judge (one right item is enough); list items found is the share of gold items present in the answer, without an LLM.

| category | J | strict J (lists must be complete) | list answers n | list items named (LLM) | list answers complete |
|---|---|---|---|---|---|
| multi-hop | 80.0% | 80.0% | 5 | 80.0% | 80.0% |
| open-domain | 80.0% | 80.0% | 0 | - | - |
| single-hop | 100.0% | 100.0% | 2 | 100.0% | 100.0% |
| temporal | 90.0% | 90.0% | 0 | - | - |
| overall | 87.5% | 87.5% | 7 | 85.7% | 85.7% |

The list judge (same model as the judge) splits the gold list and counts the items the answer names, paraphrases included.

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 40 | 258,361 | 26,572 | 14,017 | 6,459 / 664 | 0.0000 | 7931 / 10469 |
| judge | 40 | 72,430 | 39,978 | 36,922 | 1,811 / 999 | 0.0000 | 8643 / 53045 |
| list_judge | 7 | 8,485 | 2,539 | 2,399 | 1,212 / 363 | 0.0000 | 6748 / 13318 |

Per question, answer + judge, including cached replies: 8,270 prompt tokens, 1,664 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
