<!-- BEAM 100K, 100 questions (10 per category, seed 0), the SAME cached Haiku answers as 2026-10-16-beam-100k-haiku.md, re-judged by claude-cli/sonnet (Sonnet 5.5). Haiku judge on these 100: 20.6%, Sonnet judge: 37.7% -->
# beam · mode episodes · k=30 · episodes=10
answer model `claude-cli/haiku` · judge model `claude-cli/sonnet`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| abstention | 10 | 90.0% | - | - | - | - | 11,661 / 14,723 | 17,774 / 88 | 54 / 96 |
| contradiction_resolution | 10 | 18.8% | - | - | 79.2% | 70.0% | 10,870 / 12,603 | 17,119 / 47 | 12 / 32 |
| event_ordering | 10 | 33.8% | - | - | 35.4% | 0.0% | 12,345 / 18,053 | 18,962 / 556 | 17 / 34 |
| information_extraction | 10 | 40.4% | - | - | 76.0% | 70.0% | 12,687 / 17,015 | 19,118 / 89 | 15 / 30 |
| instruction_following | 10 | 40.0% | - | - | 50.0% | 30.0% | 11,313 / 14,997 | 17,661 / 267 | 11 / 18 |
| knowledge_update | 10 | 25.0% | - | - | 95.5% | 90.0% | 11,623 / 17,196 | 18,410 / 195 | 11 / 20 |
| multi_session_reasoning | 10 | 25.0% | - | - | 58.6% | 30.0% | 11,786 / 18,193 | 18,347 / 175 | 14 / 28 |
| preference_following | 10 | 41.0% | - | - | 36.4% | 30.0% | 11,649 / 16,467 | 17,379 / 251 | 12 / 14 |
| summarization | 10 | 33.6% | - | - | 30.1% | 0.0% | 12,938 / 17,842 | 19,835 / 1,092 | 13 / 23 |
| temporal_reasoning | 10 | 30.0% | - | - | 95.0% | 90.0% | 11,147 / 12,121 | 16,983 / 187 | 14 / 30 |
| overall | 100 | 37.7% | - | - | 53.4% | 46.6% | 11,802 / 17,137 | 18,159 / 295 | 13 / 54 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| abstention | - | - | - | 12.0× | 96 | 35 |
| contradiction_resolution | 0.833 | 27.6% | - | 13.7× | 32 | 22 |
| event_ordering | 0.302 | 20.3% | - | 14.6× | 34 | 63 |
| information_extraction | 0.549 | 13.6% | - | 11.7× | 30 | 45 |
| instruction_following | 0.078 | 5.4% | - | 13.8× | 18 | 59 |
| knowledge_update | 0.853 | 30.2% | - | 13.8× | 20 | 16 |
| multi_session_reasoning | 0.633 | 23.1% | - | 14.7× | 28 | 17 |
| preference_following | 0.225 | 5.0% | - | 13.8× | 14 | 56 |
| summarization | 0.402 | 17.3% | - | 10.7× | 23 | 198 |
| temporal_reasoning | 0.800 | 26.9% | - | 13.4× | 30 | 22 |
| overall | 0.522 | 18.9% | - | 13.2× | 96 | 53 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| abstention | - | - | - |
| contradiction_resolution | 25.0% | 4.2% | 70.0% |
| event_ordering | - | 33.8% | 0.0% |
| information_extraction | 39.9% | 41.7% | 70.0% |
| instruction_following | 83.3% | 21.4% | 30.0% |
| knowledge_update | 27.8% | 0.0% | 90.0% |
| multi_session_reasoning | 50.0% | 14.3% | 30.0% |
| preference_following | 50.0% | 37.1% | 30.0% |
| summarization | - | 36.5% | 0.0% |
| temporal_reasoning | 33.3% | 0.0% | 90.0% |
| overall | 37.9% | 27.2% | 46.6% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

Abstention accuracy: 90.0%

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| judge | 262 | 771,282 | 31,224 | 0 | 2,944 / 119 | 0.0000 | 5748 / 7038 |

Per question, answer + judge, including cached replies: 25,982 prompt tokens, 610 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
