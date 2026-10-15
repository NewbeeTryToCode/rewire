# LoCoMo, Nemotron 3 Ultra (free), 80 questions, 2026-10-14

Run from the runbook `docs/evaluation.md` §9, step 4.

- Command: `run.py --bench locomo --sample 20 --answer-model M --judge-model M --reasoning-effort low --answer-max-tokens 4000 --parallel 2 --label nemotron-s20`,
  M = `nvidia/nemotron-3-ultra-550b-a55b:free`. Default runner seed; 20 questions per category (stratified).
- Mode `episodes`, keyword only (no semantic), k=30, 10 excerpts: the same as the retrieval baseline in §8.
- `--answer-max-tokens 4000` (not the default 1500): in the smoke test, 2 of 8 answers were truncated because
  reasoning used up the limit, even with `--reasoning-effort low`. In this run, 1 of 80 was still truncated.
- Cost $0; ±5.1k prompt + 1.1k completion tokens per question; answer p50 ≈27 s, p95 ≈99 s. Total duration ≈25 minutes.

## Reading notes

- **J of 88.8% is higher than "all evidence present" (67.5%).** Of the 80 questions, 19 were judged correct even though
  not all evidence was retrieved (multi-hop 12, open-domain 6). Some are genuinely correct from partial evidence or
  general knowledge (open-domain), and some are due to a lenient judge: e.g. "Where has Evan been on roadtrips with
  his family?" (gold: Rockies, Jasper) was answered "Banff" and judged correct. The answerer and the judge are the same model
  (§7): this number is for internal regression tracking, not a public claim.
- 95% confidence interval for n=80, p≈0.89: ±6.9 points; per category (n=20) ±13–18 points.
- The second model (`thinkingmachines/inkling:free`) could not be used: OpenRouter returned 403, because that model is only
  open to registered "agentic harnesses", not to direct API calls.

## Report runner

answer model `nvidia/nemotron-3-ultra-550b-a55b:free` · judge model `nvidia/nemotron-3-ultra-550b-a55b:free`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| multi-hop | 20 | 95.0% | 0.208 | 0.135 | 65.6% | 35.0% | 2,640 / 3,355 | 4,304 / 1,006 | 2 / 9 |
| open-domain | 20 | 80.0% | 0.204 | 0.152 | 47.4% | 50.0% | 2,514 / 3,048 | 4,188 / 1,170 | 2 / 6 |
| single-hop | 20 | 95.0% | 0.301 | 0.214 | 95.5% | 95.0% | 2,626 / 3,134 | 4,295 / 689 | 2 / 3 |
| temporal | 20 | 85.0% | 0.216 | 0.151 | 90.9% | 90.0% | 2,443 / 2,916 | 4,139 / 1,094 | 2 / 2 |
| overall | 80 | 88.8% | 0.232 | 0.163 | 66.7% | 67.5% | 2,556 / 3,120 | 4,232 / 990 | 2 / 4 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| multi-hop | 0.796 | 25.5% | - | 7.4× | 9 | 32 |
| open-domain | 0.560 | 14.0% | - | 7.3× | 6 | 66 |
| single-hop | 0.771 | 12.5% | - | 6.8× | 3 | 29 |
| temporal | 0.717 | 10.5% | - | 7.3× | 2 | 150 |
| overall | 0.711 | 15.6% | - | 7.2× | 9 | 69 |

WARNING: 0 empty and 1 truncated answers (max_tokens reached): raise --answer-max-tokens or pass --reasoning-effort low.

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| judge | 79 | 66,941 | 9,399 | 6,995 | 847 / 119 | 0.0000 | 4677 / 17514 |
| answer | 78 | 330,386 | 77,037 | 59,148 | 4,236 / 988 | 0.0000 | 26848 / 98796 |

Per question, answer + judge, including cached replies: 5,078 prompt tokens, 1,109 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
