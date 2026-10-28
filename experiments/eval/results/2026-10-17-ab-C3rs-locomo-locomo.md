# locomo · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| multi-hop | 10 | 100.0% | 0.254 | 0.173 | 75.0% | 70.0% | 3,366 / 5,031 | 8,557 / 715 | 111 / 165 |
| open-domain | 10 | 70.0% | 0.138 | 0.088 | 55.9% | 60.0% | 2,692 / 3,909 | 7,593 / 847 | 106 / 114 |
| single-hop | 10 | 100.0% | 0.552 | 0.436 | 100.0% | 100.0% | 2,426 / 3,076 | 7,201 / 418 | 105 / 110 |
| temporal | 10 | 90.0% | 0.247 | 0.137 | 86.7% | 80.0% | 3,417 / 7,021 | 8,669 / 602 | 109 / 127 |
| overall | 40 | 90.0% | 0.297 | 0.209 | 72.8% | 77.5% | 2,975 / 5,031 | 8,005 / 645 | 106 / 133 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| multi-hop | 0.820 | 26.8% | - | 5.4× | 165 | 28 |
| open-domain | 0.750 | 16.0% | - | 7.1× | 114 | 42 |
| single-hop | 0.883 | 14.1% | - | 7.4× | 110 | 16 |
| temporal | 0.900 | 13.2% | - | 5.2× | 127 | 18 |
| overall | 0.838 | 17.5% | - | 6.1× | 165 | 26 |

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

Retrieval tokens per question: context 2,975 (p95 5,031); extra-query LLM calls 1,685 prompt + 212 completion on average over all questions, 1,684 + 258 on the 22 questions given extra queries.

Per question, answer + judge, including cached replies: 9,811 prompt tokens, 1,333 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
