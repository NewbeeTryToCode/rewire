<!-- seed 0, REWIRE_SEMANTIC=0, recall clock frozen; Haiku brains from the full run -->
# locomo · mode full · k=30 · episodes=20
answer model `claude-cli/haiku` · judge model `claude-cli/haiku`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| multi-hop | 282 | 90.8% | 0.175 | 0.115 | 69.6% | 48.6% | 5,133 / 6,139 | 11,277 / 853 | 9 / 14 |
| overall | 282 | 90.8% | 0.175 | 0.115 | 69.6% | 48.6% | 5,133 / 6,139 | 11,277 / 853 | 9 / 14 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| multi-hop | 0.599 | 13.9% | - | 3.6× | 73 | 50 |
| overall | 0.599 | 13.9% | - | 3.6× | 73 | 50 |

| category | J | disputed gold excluded | J without them |
|---|---|---|---|
| multi-hop | 90.8% | 3 | 91.8% |
| overall | 90.8% | 3 | 91.8% |

Disputed gold: experiments/eval/locomo_errata.json (speaker swapped in the question, gold contradicts the conversation, answer only in a photo). J is the official, comparable number.

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| multi-hop | 94.9% | 86.9% | 48.6% |
| overall | 94.9% | 86.9% | 48.6% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

| category | all evidence (excerpts) | all evidence (excerpts or a fact from the turn) | list answers n | list J | list items found |
|---|---|---|---|---|---|
| multi-hop | 48.6% | 56.4% | 124 | 96.8% | 67.0% |
| overall | 48.6% | 56.4% | 124 | 96.8% | 67.0% |

List J follows the official judge (one right item is enough); list items found is the share of gold items present in the answer, without an LLM.

| category | J | strict J (lists must be complete) | list answers n | list items named (LLM) | list answers complete |
|---|---|---|---|---|---|
| multi-hop | 90.8% | 77.3% | 124 | 82.3% | 66.1% |
| overall | 90.8% | 77.3% | 124 | 82.3% | 66.1% |

The list judge (same model as the judge) splits the gold list and counts the items the answer names, paraphrases included.

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 95 | 1,039,116 | 73,905 | 40,621 | 10,938 / 778 | 0.0000 | 7761 / 10039 |
| judge | 91 | 245,528 | 8,936 | 1,307 | 2,698 / 98 | 0.0000 | 4702 / 6251 |
| list_judge | 80 | 142,258 | 8,198 | 6,998 | 1,778 / 102 | 0.0000 | 4790 / 6224 |

Per question, answer + judge, including cached replies: 13,989 prompt tokens, 960 completion tokens, $0.00000.

NOTE: answers are judged by the same model (`claude-cli/haiku`); a model tends to accept its own answers, so J may read higher than with an independent judge.

## Ingest (mode full)

272 sessions consolidated (0 failed attempts retried, 1 replies that were not JSON). Facts per session: 15.8 (4120 applied, 191 queued then approved, 0 rejected). Hypothetical questions: 3345 (0.57 per turn). Seconds per session p50: 42.

Active nodes by type: fact 2401, preference 553, decision 449, person 35, concept 8

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
