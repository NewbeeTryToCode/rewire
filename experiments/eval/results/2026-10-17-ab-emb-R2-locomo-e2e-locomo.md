# locomo · mode episodes · k=30 · episodes=10
answer model `claude-cli/claude-haiku-5-5` · judge model `claude-cli/claude-haiku-4-5` · extra queries by `claude-cli/claude-haiku-5-5`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| multi-hop | 10 | 90.0% | 0.221 | 0.151 | 71.9% | 60.0% | 2,521 / 4,671 | 7,358 / 646 | 964 / 13577 |
| open-domain | 10 | 70.0% | 0.108 | 0.078 | 55.9% | 60.0% | 2,412 / 4,204 | 7,192 / 890 | 639 / 4051 |
| single-hop | 10 | 100.0% | 0.374 | 0.285 | 100.0% | 100.0% | 2,541 / 4,254 | 7,370 / 470 | 868 / 3976 |
| temporal | 10 | 90.0% | 0.306 | 0.222 | 86.7% | 80.0% | 3,526 / 6,874 | 8,874 / 606 | 4201 / 4851 |
| overall | 40 | 87.5% | 0.252 | 0.184 | 71.7% | 75.0% | 2,750 / 5,224 | 7,699 / 653 | 908 / 9813 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| multi-hop | 0.792 | 29.8% | - | 7.2× | 13577 | 28 |
| open-domain | 0.750 | 16.7% | - | 8.0× | 4051 | 46 |
| single-hop | 0.950 | 14.0% | - | 7.0× | 3976 | 20 |
| temporal | 0.950 | 11.5% | - | 5.0× | 4851 | 21 |
| overall | 0.860 | 18.0% | - | 6.6× | 13577 | 29 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| multi-hop | 100.0% | 75.0% | 60.0% |
| open-domain | 66.7% | 75.0% | 60.0% |
| single-hop | 100.0% | - | 100.0% |
| temporal | 100.0% | 50.0% | 80.0% |
| overall | 93.3% | 70.0% | 75.0% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

| category | all evidence (excerpts) | all evidence (excerpts or a fact from the turn) | list answers n | list J | list items found |
|---|---|---|---|---|---|
| multi-hop | 60.0% | 60.0% | 5 | 100.0% | 70.0% |
| open-domain | 60.0% | 60.0% | - | - | - |
| single-hop | 100.0% | 100.0% | 2 | 100.0% | 100.0% |
| temporal | 80.0% | 80.0% | - | - | - |
| overall | 75.0% | 75.0% | 7 | 100.0% | 78.6% |

List J follows the official judge (one right item is enough); list items found is the share of gold items present in the answer, without an LLM.

| category | J | strict J (lists must be complete) | list answers n | list items named (LLM) | list answers complete |
|---|---|---|---|---|---|
| multi-hop | 90.0% | 90.0% | 5 | 100.0% | 100.0% |
| open-domain | 70.0% | 70.0% | 0 | - | - |
| single-hop | 100.0% | 100.0% | 2 | 100.0% | 100.0% |
| temporal | 90.0% | 90.0% | 0 | - | - |
| overall | 87.5% | 87.5% | 7 | 100.0% | 100.0% |

The list judge (same model as the judge) splits the gold list and counts the items the answer names, paraphrases included.

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 40 | 307,942 | 26,120 | 14,674 | 7,699 / 653 | 0.0000 | 8067 / 11153 |
| judge | 38 | 68,834 | 31,785 | 28,822 | 1,811 / 836 | 0.0000 | 9297 / 33954 |
| list_judge | 6 | 7,305 | 5,092 | 4,972 | 1,218 / 849 | 0.0000 | 7051 / 31494 |

Retrieval tokens per question: context 2,750 (p95 5,224); extra-query LLM calls 510 prompt + 23 completion on average over all questions, 1,571 + 71 on the 13 questions given extra queries.

Per question, answer + judge, including cached replies: 9,508 prompt tokens, 1,464 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
