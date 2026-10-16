<!-- BEAM 100K chats 1,7,13,19 (80 questions), --mode full: every session consolidated by claude-cli/haiku (general profile, inbox accepted), k=30, --episodes 40 size-capped excerpts, keyword recall, answer claude-cli/haiku, judge claude-cli/sonnet. Baseline on the same questions (mode episodes): 36.5%. Paired: 26 better, 17 worse. -->
# beam · mode full · k=30 · episodes=40
answer model `claude-cli/haiku` · judge model `claude-cli/sonnet`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| abstention | 8 | 62.5% | - | - | - | - | 12,419 / 17,902 | 18,805 / 298 | 24 / 93 |
| contradiction_resolution | 8 | 32.8% | - | - | 100.0% | 100.0% | 12,996 / 19,533 | 19,638 / 66 | 27 / 77 |
| event_ordering | 8 | 27.8% | - | - | 40.9% | 12.5% | 14,143 / 17,774 | 21,129 / 390 | 22 / 43 |
| information_extraction | 8 | 50.0% | - | - | 81.8% | 87.5% | 12,372 / 16,558 | 18,840 / 48 | 24 / 36 |
| instruction_following | 8 | 34.4% | - | - | 55.6% | 50.0% | 12,761 / 17,457 | 19,241 / 374 | 15 / 33 |
| knowledge_update | 8 | 62.5% | - | - | 93.8% | 87.5% | 12,752 / 17,931 | 19,357 / 137 | 16 / 23 |
| multi_session_reasoning | 8 | 35.6% | - | - | 63.3% | 37.5% | 13,375 / 17,652 | 20,162 / 286 | 22 / 32 |
| preference_following | 8 | 68.8% | - | - | 66.7% | 62.5% | 12,284 / 17,974 | 18,570 / 447 | 17 / 26 |
| summarization | 8 | 18.0% | - | - | 37.3% | 0.0% | 14,184 / 19,052 | 21,206 / 705 | 23 / 27 |
| temporal_reasoning | 8 | 37.5% | - | - | 100.0% | 100.0% | 12,114 / 14,390 | 18,625 / 120 | 18 / 32 |
| overall | 80 | 43.0% | - | - | 60.9% | 59.7% | 12,940 / 17,974 | 19,557 / 287 | 20 / 43 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| abstention | - | - | - | 11.9× | 93 | 44 |
| contradiction_resolution | 0.812 | 8.0% | - | 11.4× | 77 | 32 |
| event_ordering | 0.376 | 6.8% | - | 10.4× | 43 | 51 |
| information_extraction | 0.367 | 2.9% | - | 11.9× | 36 | 18 |
| instruction_following | 0.200 | 1.8% | - | 11.6× | 33 | 56 |
| knowledge_update | 0.844 | 7.4% | - | 11.6× | 23 | 12 |
| multi_session_reasoning | 0.687 | 7.8% | - | 11.0× | 32 | 39 |
| preference_following | 0.199 | 2.4% | - | 12.0× | 26 | 56 |
| summarization | 0.258 | 6.6% | - | 10.4× | 27 | 160 |
| temporal_reasoning | 0.650 | 6.2% | - | 12.2× | 32 | 33 |
| overall | 0.488 | 5.5% | - | 11.4× | 93 | 50 |

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| abstention | - | - | - |
| contradiction_resolution | 32.8% | - | 100.0% |
| event_ordering | 0.0% | 31.8% | 12.5% |
| information_extraction | 57.1% | 0.0% | 87.5% |
| instruction_following | 37.5% | 31.2% | 50.0% |
| knowledge_update | 57.1% | 100.0% | 87.5% |
| multi_session_reasoning | 66.7% | 17.0% | 37.5% |
| preference_following | 60.0% | 83.3% | 62.5% |
| summarization | - | 18.0% | 0.0% |
| temporal_reasoning | 37.5% | - | 100.0% |
| overall | 46.8% | 32.0% | 59.7% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

Abstention accuracy: 62.5%

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| ingest | 37 | 666,297 | 341,174 | 183,640 | 18,008 / 9,221 | 0.0000 | 49391 / 82698 |
| answer | 80 | 1,564,585 | 22,968 | 13,284 | 19,557 / 287 | 0.0000 | 7354 / 12311 |
| judge | 213 | 621,432 | 24,940 | 0 | 2,918 / 117 | 0.0000 | 6375 / 8327 |

Per question, answer + judge, including cached replies: 27,325 prompt tokens, 599 completion tokens, $0.00000.

## Ingest (mode full)

18 sessions consolidated (0 failed attempts retried, 0 replies that were not JSON). Facts per session: 36.2 (599 applied, 52 queued then approved, 0 rejected). Hypothetical questions: 415 (0.38 per turn). Seconds per session p50: 71.

Active nodes by type: fact 310, decision 179, preference 43, person 20, concept 1

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
