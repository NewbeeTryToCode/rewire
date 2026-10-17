# locomo · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| multi-hop | 10 | 100.0% | 0.254 | 0.173 | 75.0% | 70.0% | 3,366 / 5,031 | 8,557 / 715 | 111 / 130 |
| open-domain | 10 | 70.0% | 0.138 | 0.088 | 55.9% | 60.0% | 2,692 / 3,909 | 7,593 / 847 | 107 / 159 |
| single-hop | 10 | 100.0% | 0.544 | 0.431 | 100.0% | 100.0% | 2,617 / 4,532 | 7,485 / 434 | 112 / 122 |
| temporal | 10 | 90.0% | 0.236 | 0.132 | 86.7% | 80.0% | 3,550 / 7,021 | 8,887 / 624 | 112 / 123 |
| overall | 40 | 90.0% | 0.293 | 0.206 | 72.8% | 77.5% | 3,056 / 5,031 | 8,130 / 655 | 110 / 130 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| multi-hop | 0.820 | 26.8% | - | 5.4× | 130 | 28 |
| open-domain | 0.750 | 16.0% | - | 7.1× | 159 | 42 |
| single-hop | 0.883 | 14.4% | - | 6.8× | 122 | 16 |
| temporal | 0.750 | 14.3% | - | 5.0× | 123 | 22 |
| overall | 0.801 | 17.9% | - | 6.0× | 159 | 27 |

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
| multi-hop | 100.0% | 100.0% | 5 | 100.0% | 100.0% |
| open-domain | 70.0% | 70.0% | 0 | - | - |
| single-hop | 100.0% | 100.0% | 2 | 100.0% | 100.0% |
| temporal | 90.0% | 90.0% | 0 | - | - |
| overall | 90.0% | 90.0% | 7 | 100.0% | 100.0% |

The list judge (same model as the judge) splits the gold list and counts the items the answer names, paraphrases included.

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|

Retrieval tokens per question: context 3,056 (p95 5,031); extra-query LLM calls 1,842 prompt + 219 completion on average over all questions, 1,925 + 246 on the 26 questions given extra queries.

Per question, answer + judge, including cached replies: 9,938 prompt tokens, 1,362 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
