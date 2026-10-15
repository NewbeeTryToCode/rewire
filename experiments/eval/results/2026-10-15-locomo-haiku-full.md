<!-- LoCoMo penuh (1.540 soal, kategori 1-4), mode full, k=30, episodes=20, ingest/jawab/juri claude-cli/haiku (Haiku 5.5), seed 0, sebelum jam recall dikunci -->
# locomo · mode full · k=30 · episodes=20
answer model `claude-cli/haiku` · judge model `claude-cli/haiku`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| multi-hop | 282 | 92.2% | 0.164 | 0.106 | 69.4% | 47.9% | 5,127 / 6,139 | 11,267 / 855 | 9 / 18 |
| open-domain | 96 | 66.7% | 0.124 | 0.093 | 49.5% | 46.7% | 4,947 / 6,234 | 10,989 / 860 | 9 / 21 |
| single-hop | 841 | 95.8% | 0.263 | 0.181 | 98.2% | 98.2% | 5,006 / 6,043 | 11,068 / 578 | 9 / 19 |
| temporal | 321 | 94.1% | 0.293 | 0.202 | 91.4% | 91.6% | 4,952 / 5,856 | 11,037 / 569 | 9 / 25 |
| overall | 1540 | 93.0% | 0.243 | 0.166 | 82.1% | 84.5% | 5,013 / 6,067 | 11,093 / 645 | 9 / 20 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| multi-hop | 0.603 | 13.9% | - | 3.6× | 27 | 51 |
| open-domain | 0.440 | 7.8% | - | 3.8× | 45 | 52 |
| single-hop | 0.790 | 7.3% | - | 3.8× | 28 | 32 |
| temporal | 0.746 | 7.1% | - | 3.7× | 38 | 20 |
| overall | 0.725 | 8.5% | - | 3.7× | 29 | 34 |

| category | J | disputed gold excluded | J without them |
|---|---|---|---|
| multi-hop | 92.2% | 3 | 93.2% |
| open-domain | 66.7% | 2 | 68.1% |
| single-hop | 95.8% | 19 | 98.1% |
| temporal | 94.1% | 5 | 95.6% |
| overall | 93.0% | 29 | 94.8% |

Disputed gold: experiments/eval/locomo_errata.json (speaker swapped in the question, gold contradicts the conversation, answer only in a photo). J is the official, comparable number.

| category | J when all evidence retrieved | J when evidence missing | share with all evidence |
|---|---|---|---|
| multi-hop | 96.3% | 88.4% | 47.9% |
| open-domain | 86.0% | 46.9% | 46.7% |
| single-hop | 96.4% | 66.7% | 98.2% |
| temporal | 96.2% | 70.4% | 91.6% |
| overall | 96.0% | 76.5% | 84.5% |

Low J with all evidence = the answering model; low share with all evidence = retrieval.

| category | all evidence (excerpts) | all evidence (excerpts or a fact from the turn) | list answers n | list J | list items found |
|---|---|---|---|---|---|
| multi-hop | 47.9% | 56.0% | 124 | 96.8% | 67.2% |
| open-domain | 46.7% | 53.3% | 9 | 66.7% | 12.0% |
| single-hop | 98.2% | 99.2% | 159 | 95.6% | 73.6% |
| temporal | 91.6% | 93.1% | 3 | 100.0% | 100.0% |
| overall | 84.5% | 87.2% | 295 | 95.3% | 69.3% |

List J follows the official judge (one right item is enough); list items found is the share of gold items present in the answer, without an LLM.

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| answer | 405 | 4,378,759 | 247,548 | 131,627 | 10,812 / 611 | 0.0000 | 6347 / 9492 |
| judge | 357 | 953,229 | 32,965 | 4,230 | 2,670 / 92 | 0.0000 | 4427 / 5427 |

Per question, answer + judge, including cached replies: 13,766 prompt tokens, 739 completion tokens, $0.00000.

NOTE: answers are judged by the same model (`claude-cli/haiku`); a model tends to accept its own answers, so J may read higher than with an independent judge.

## Ingest (mode full)

272 sessions consolidated (0 failed attempts retried, 1 replies that were not JSON). Facts per session: 15.8 (4120 applied, 191 queued then approved, 0 rejected). Hypothetical questions: 3345 (0.57 per turn). Seconds per session p50: 42.

Active nodes by type: fact 2401, preference 553, decision 449, person 35, concept 8

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.

## Juri kedua: Haiku 4.5 (jawaban yang sama)

| kategori | n | J (juri Haiku 5.5) | J (juri Haiku 4.5) | kedua juri sepakat | J ketat (daftar harus lengkap) | J tanpa 29 gold bermasalah (juri 4.5) |
|---|---|---|---|---|---|---|
| multi-hop | 282 | 92,2% | 92,2% | 99,3% | 79,4% | 93,2% |
| open-domain | 96 | 66,7% | 71,9% | 94,8% | 61,5% | 73,4% |
| single-hop | 841 | 95,8% | 96,0% | 99,2% | 94,2% | 98,1% |
| temporal | 321 | 94,1% | 95,0% | 97,2% | 94,1% | 96,5% |
| overall | 1.540 | 93,0% | 93,6% | 98,5% | 89,4% | 95,3% |

