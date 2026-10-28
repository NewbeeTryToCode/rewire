# locomo · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| multi-hop | 10 | 100.0% | 0.156 | 0.096 | 78.1% | 70.0% | 2,542 / 5,452 | 7,391 / 734 | 61 / 267 |
| open-domain | 10 | 70.0% | 0.116 | 0.081 | 55.9% | 60.0% | 2,555 / 4,358 | 7,374 / 838 | 104 / 136 |
| single-hop | 10 | 100.0% | 0.415 | 0.289 | 100.0% | 100.0% | 2,446 / 3,377 | 7,225 / 500 | 25 / 134 |
| temporal | 10 | 90.0% | 0.297 | 0.204 | 86.7% | 80.0% | 3,604 / 7,531 | 8,970 / 594 | 22 / 52 |
| overall | 40 | 90.0% | 0.246 | 0.167 | 73.9% | 77.5% | 2,787 / 5,452 | 7,740 / 666 | 49 / 222 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| multi-hop | 0.798 | 34.0% | - | 7.1× | 267 | 26 |
| open-domain | 0.800 | 16.6% | - | 7.5× | 136 | 41 |
| single-hop | 0.920 | 13.1% | - | 7.3× | 134 | 16 |
| temporal | 0.658 | 13.2% | - | 4.9× | 52 | 22 |
| overall | 0.794 | 19.2% | - | 6.5× | 267 | 26 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| multi-hop | 100.0% | 100.0% | 70.0% |
| open-domain | 66.7% | 75.0% | 60.0% |
| single-hop | 100.0% | - | 100.0% |
| temporal | 100.0% | 50.0% | 80.0% |
| overall | 93.5% | 77.8% | 77.5% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

| category | all evidence (excerpts) | all evidence (excerpts or a fact from the turn) | list answers n | list J | list items found |
|---|---|---|---|---|---|
| multi-hop | 70.0% | 70.0% | 5 | 100.0% | 65.0% |
| open-domain | 60.0% | 60.0% | - | - | - |
| single-hop | 100.0% | 100.0% | 2 | 100.0% | 100.0% |
| temporal | 80.0% | 80.0% | - | - | - |
| overall | 77.5% | 77.5% | 7 | 100.0% | 75.0% |

List J follows the official judge (one right item is enough); list items found is the share of gold items present in the answer, without an LLM.

| category | J | strict J (lists must be complete) | list answers n | list items named (LLM) | list answers complete |
|---|---|---|---|---|---|
| multi-hop | 100.0% | 90.0% | 5 | 90.0% | 80.0% |
| open-domain | 70.0% | 70.0% | 0 | - | - |
| single-hop | 100.0% | 100.0% | 2 | 100.0% | 100.0% |
| temporal | 90.0% | 90.0% | 0 | - | - |
| overall | 90.0% | 87.5% | 7 | 92.9% | 85.7% |

The list judge (same model as the judge) splits the gold list and counts the items the answer names, paraphrases included.

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 17 | 139,795 | 9,632 | 5,807 | 8,223 / 567 | 0.0000 | 7400 / 9847 |
| judge | 18 | 32,365 | 17,154 | 15,721 | 1,798 / 953 | 0.0000 | 9542 / 57194 |
| list_judge | 2 | 2,409 | 586 | 550 | 1,204 / 293 | 0.0000 | 6824 / 6824 |

Retrieval tokens per question: context 2,787 (p95 5,452); extra-query LLM calls 510 prompt + 23 completion on average over all questions, 1,571 + 71 on the 13 questions given extra queries.

Per question, answer + judge, including cached replies: 9,545 prompt tokens, 1,461 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
