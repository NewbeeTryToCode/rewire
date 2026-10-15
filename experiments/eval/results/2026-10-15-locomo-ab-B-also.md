<!-- seed 0, REWIRE_SEMANTIC=0, jam recall dikunci; brain Haiku dari run penuh -->
# locomo · mode full · k=30 · episodes=20
answer model `claude-cli/haiku` · judge model `claude-cli/haiku` · extra queries by `claude-cli/haiku`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| multi-hop | 282 | 92.9% | 0.177 | 0.118 | 82.8% | 66.7% | 7,044 / 8,864 | 14,056 / 911 | 29 / 365 |
| overall | 282 | 92.9% | 0.177 | 0.118 | 82.8% | 66.7% | 7,044 / 8,864 | 14,056 / 911 | 29 / 365 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| multi-hop | 0.592 | 13.6% | - | 2.7× | 531 | 54 |
| overall | 0.592 | 13.6% | - | 2.7× | 531 | 54 |

| category | J | disputed gold excluded | J without them |
|---|---|---|---|
| multi-hop | 92.9% | 3 | 93.2% |
| overall | 92.9% | 3 | 93.2% |

Disputed gold: experiments/eval/locomo_errata.json (speaker swapped in the question, gold contradicts the conversation, answer only in a photo). J is the official, comparable number.

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| multi-hop | 94.7% | 89.4% | 66.7% |
| overall | 94.7% | 89.4% | 66.7% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

| category | all evidence (excerpts) | all evidence (excerpts or a fact from the turn) | list answers n | list J | list items found |
|---|---|---|---|---|---|
| multi-hop | 66.7% | 71.3% | 124 | 98.4% | 73.2% |
| overall | 66.7% | 71.3% | 124 | 98.4% | 73.2% |

List J follows the official judge (one right item is enough); list items found is the share of gold items present in the answer, without an LLM.

| category | J | strict J (lists must be complete) | list answers n | list items named (LLM) | list answers complete |
|---|---|---|---|---|---|
| multi-hop | 92.9% | 81.6% | 124 | 87.2% | 72.6% |
| overall | 92.9% | 81.6% | 124 | 87.2% | 72.6% |

The list judge (same model as the judge) splits the gold list and counts the items the answer names, paraphrases included.

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 37 | 514,670 | 31,638 | 18,387 | 13,910 / 855 | 0.0000 | 7932 / 10424 |
| judge | 34 | 92,072 | 3,858 | 1,031 | 2,708 / 113 | 0.0000 | 4596 / 5658 |
| list_judge | 14 | 24,996 | 1,443 | 1,233 | 1,785 / 103 | 0.0000 | 5403 / 6990 |

Per question, answer + judge, including cached replies: 16,775 prompt tokens, 1,023 completion tokens, $0.00000.

NOTE: answers are judged by the same model (`claude-cli/haiku`); a model tends to accept its own answers, so J may read higher than with an independent judge.

## Ingest (mode full)

272 sessions consolidated (0 failed attempts retried, 1 replies that were not JSON). Facts per session: 15.8 (4120 applied, 191 queued then approved, 0 rejected). Hypothetical questions: 3345 (0.57 per turn). Seconds per session p50: 42.

Active nodes by type: fact 2401, preference 553, decision 449, person 35, concept 8

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
