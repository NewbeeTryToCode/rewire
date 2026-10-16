<!-- BEAM 100K, all 400 questions, --retrieval-only (no LLM), mode episodes, k=30, episodes=10, keyword recall (no semantic model), BEAM commit b2da22e -->
# beam · mode episodes · k=30 · episodes=10
answer model `openai/gpt-4o-mini` · judge model `openai/gpt-4o-mini`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| abstention | 40 | - | - | - | - | - | 11,735 / 14,723 | - / - | 13 / 59 |
| contradiction_resolution | 40 | - | - | - | 82.4% | 65.0% | 11,503 / 18,413 | - / - | 19 / 88 |
| event_ordering | 40 | - | - | - | 34.6% | 2.5% | 12,685 / 19,055 | - / - | 29 / 103 |
| information_extraction | 40 | - | - | - | 66.2% | 62.5% | 11,486 / 17,015 | - / - | 24 / 60 |
| instruction_following | 40 | - | - | - | 34.6% | 27.5% | 11,107 / 14,997 | - / - | 33 / 83 |
| knowledge_update | 40 | - | - | - | 84.3% | 75.0% | 11,671 / 17,196 | - / - | 32 / 96 |
| multi_session_reasoning | 40 | - | - | - | 62.4% | 32.5% | 12,273 / 18,193 | - / - | 41 / 105 |
| preference_following | 40 | - | - | - | 53.2% | 51.3% | 12,095 / 18,322 | - / - | 46 / 99 |
| summarization | 40 | - | - | - | 29.7% | 0.0% | 12,358 / 17,842 | - / - | 51 / 92 |
| temporal_reasoning | 40 | - | - | - | 91.2% | 82.5% | 11,931 / 16,399 | - / - | 49 / 128 |
| overall | 400 | - | - | - | 52.2% | 44.8% | 11,884 / 17,135 | - / - | 33 / 92 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| abstention | - | - | - | 13.0× | 106 | - |
| contradiction_resolution | 0.849 | 26.0% | - | 13.3× | 136 | - |
| event_ordering | 0.416 | 22.2% | - | 12.1× | 129 | - |
| information_extraction | 0.424 | 10.3% | - | 13.3× | 63 | - |
| instruction_following | 0.113 | 6.5% | - | 13.8× | 102 | - |
| knowledge_update | 0.758 | 26.7% | - | 13.1× | 115 | - |
| multi_session_reasoning | 0.675 | 26.4% | - | 12.5× | 107 | - |
| preference_following | 0.270 | 9.8% | - | 12.6× | 125 | - |
| summarization | 0.428 | 21.5% | - | 12.4× | 101 | - |
| temporal_reasoning | 0.786 | 26.2% | - | 12.8× | 145 | - |
| overall | 0.526 | 19.5% | - | 12.9× | 128 | - |

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|

NOTE: answers are judged by the same model (`openai/gpt-4o-mini`); a model tends to accept its own answers, so J may read higher than with an independent judge.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
