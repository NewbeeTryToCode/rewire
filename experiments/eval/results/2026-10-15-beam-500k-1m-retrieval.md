<!-- BEAM 500K + 1M, all 1,400 questions, --retrieval-only (no LLM), mode episodes, k=30, episodes=10, keyword recall, BEAM commit b2da22e -->
# beam · mode episodes · k=30 · episodes=10
answer model `openai/gpt-4o-mini` · judge model `openai/gpt-4o-mini`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| abstention | 140 | - | - | - | - | - | 11,189 / 14,183 | - / - | 37 / 204 |
| contradiction_resolution | 140 | - | - | - | 68.6% | 50.7% | 11,043 / 14,716 | - / - | 31 / 157 |
| event_ordering | 140 | - | - | - | 7.4% | 0.0% | 12,042 / 15,076 | - / - | 40 / 154 |
| information_extraction | 140 | - | - | - | 48.8% | 50.0% | 11,164 / 14,495 | - / - | 32 / 138 |
| instruction_following | 140 | - | - | - | 32.6% | 28.6% | 11,161 / 14,141 | - / - | 51 / 119 |
| knowledge_update | 140 | - | - | - | 66.1% | 46.8% | 11,261 / 14,632 | - / - | 50 / 139 |
| multi_session_reasoning | 140 | - | - | - | 39.9% | 24.3% | 11,987 / 15,208 | - / - | 58 / 166 |
| preference_following | 140 | - | - | - | 33.0% | 25.2% | 12,229 / 15,550 | - / - | 55 / 157 |
| summarization | 140 | - | - | - | 9.5% | 1.5% | 12,173 / 15,149 | - / - | 59 / 143 |
| temporal_reasoning | 140 | - | - | - | 66.7% | 46.4% | 11,660 / 14,730 | - / - | 69 / 251 |
| overall | 1400 | - | - | - | 22.5% | 30.5% | 11,591 / 14,674 | - / - | 50 / 152 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| abstention | - | - | - | 74.6× | 222 | - |
| contradiction_resolution | 0.786 | 25.8% | - | 75.6× | 205 | - |
| event_ordering | 0.272 | 13.1% | - | 69.3× | 226 | - |
| information_extraction | 0.347 | 9.1% | - | 74.8× | 284 | - |
| instruction_following | 0.156 | 4.3% | - | 74.8× | 161 | - |
| knowledge_update | 0.711 | 23.0% | - | 74.2× | 192 | - |
| multi_session_reasoning | 0.558 | 21.4% | - | 69.7× | 279 | - |
| preference_following | 0.198 | 6.9% | - | 68.3× | 324 | - |
| summarization | 0.290 | 13.0% | - | 68.6× | 206 | - |
| temporal_reasoning | 0.657 | 20.9% | - | 71.6× | 619 | - |
| overall | 0.442 | 15.3% | - | 72.1× | 279 | - |

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|

NOTE: answers are judged by the same model (`openai/gpt-4o-mini`); a model tends to accept its own answers, so J may read higher than with an independent judge.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
