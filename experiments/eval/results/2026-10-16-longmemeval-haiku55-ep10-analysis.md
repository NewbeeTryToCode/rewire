# LongMemEval_S, all 500 questions: Haiku 5.5 answers, Haiku 4.5 judges, per-question diagnosis

Run of 2026-10-16. Files in this folder:

| File | Contents |
|---|---|
| `2026-10-16-longmemeval-haiku55-ep10.md` | runner report (every metric of `docs/evaluation.md` §3 that applies to LongMemEval) |
| `2026-10-16-longmemeval-retrieval.md` | the same retrieval with no LLM (0 tokens) |
| `2026-10-16-longmemeval-haiku55-ep10-diagnosis.md` | **every question**: question, gold, answer, rule diagnosis, evidence ranks, why it is right or wrong, root cause, key insight |
| `2026-10-16-longmemeval-haiku55-ep10-diagnosis.csv` | the same, one row per question (for filtering) |

## Setup

| | |
|---|---|
| Data | `longmemeval_s_cleaned.json`, 500 questions (30 `_abs`), 133 multi-session / 133 temporal / 78 knowledge-update / 70 single-session-user / 56 single-session-assistant / 30 single-session-preference. Taken from Hugging Face through `datasets-server` (mirror `lorenoamori/longmemeval`, same size ±2.5 KB as `xiaowu0162/longmemeval-cleaned`): the HF file CDN (`us.aws.cdn.hf.co`, `*.xethub.hf.co`) is blocked by the session proxy |
| Memory | `--mode episodes` (transcripts only, no LLM extraction), keyword recall (no semantic index), k=30, **10 excerpts** |
| Answerer | `claude-haiku-5-5` via `claude -p` (`claude-cli/claude-haiku-5-5`), official LongMemEval answer template |
| Judge | `claude-haiku-4-5` via `claude -p`, official per-type LongMemEval judge prompts |
| Analyst (diagnosis only, not scored) | `claude-sonnet-5-5` via `claude -p` (`experiments/eval/diagnose_lme.py`) |
| Recall clock | pinned to the end of each brain's ingest (new in this run for LongMemEval, rule 5 of §6) |

Not comparable with public numbers: the official judge is GPT-4o (§6, rule 1).

## Metrics (all of `docs/evaluation.md` §3 that apply)

| category | n | J | evidence turn recall | all evidence | evidence session found | MRR | context precision | ctx tokens mean / p95 | compression | answer words |
|---|---|---|---|---|---|---|---|---|---|---|
| single-session-user | 70 | 94.3% | 100.0% | 100.0% | 100.0% | 0.943 | 18.5% | 7,539 / 9,999 | 16.2× | 127 |
| knowledge-update | 78 | 93.6% | 97.2% | 95.8% | 98.6% | 0.951 | 32.2% | 8,851 / 10,986 | 13.8× | 160 |
| single-session-assistant | 56 | 85.7% | 98.2% | 100.0% | 100.0% | 0.982 | 13.6% | 8,555 / 10,254 | 14.3× | 150 |
| temporal-reasoning | 133 | 78.9% | 84.2% | 75.6% | 96.1% | 0.798 | 25.6% | 9,379 / 11,219 | 13.0× | 161 |
| multi-session | 133 | 65.4% | 79.6% | 78.5% | 99.2% | 0.844 | 32.1% | 8,955 / 11,203 | 13.7× | 172 |
| single-session-preference | 30 | 43.3% | 75.0% | 76.7% | 76.7% | 0.517 | 11.8% | 9,520 / 11,218 | 12.9× | 409 |
| **overall** | **500** | **78.4%** | 86.2% | 85.7% | 97.0% | 0.857 | 25.0% | 8,842 / 11,052 | 13.8× | 173 |

- **Abstention** (30 `_abs`): 76.7% (23/30); single-session-user 6/6, temporal 6/6, knowledge-update 5/6, multi-session 6/12. J on the 470 answerable questions: 78.5%.
- **Retrieval vs answering:** J 88.1% when every evidence session is in the context (n=403 of 470), 20.9% when one is missing.
- **F1 / BLEU-1** (LoCoMo metrics, computed here for completeness): 0.053 / 0.029. Meaningless for LongMemEval: the official template asks for step-by-step answers (173 words on average), so token overlap with a short gold is always low.
- **Strict J / J without disputed gold:** LoCoMo-only metrics. The closest equivalent here: the analyst disputes the judge on 8 wrong verdicts (all 8 say "wrong" for a correct answer) and the gold on 2 questions; correcting them would give ≈80.4% (inferred, one analyst model).
- **Tokens per response:** answer 13,611 prompt / 756 completion (≈250 of them thinking); judge 1,466 / 437 (≈410 thinking). Per question, answer + judge: 15,080 prompt + 1,196 completion tokens.
- **Cost:** $0 billed (subscription). API-price equivalent reported by the CLI: answer $1.55, judge $1.82, i.e. **$3.37 for 500 questions** ($0.0067 / question); the analyst cost ≈$13 equivalent. The Haiku 4.5 judge costs more than the answerer because it thinks ~400 tokens per verdict.
- **Latency:** retrieval p50 21 ms / p95 56 ms / p99 195 ms (index built before timing; with 8 parallel workers). LLM: answer p50 8.3 s / p95 13.5 s, judge p50 7.4 s / p95 13.7 s (CLI process start included).
- **Empty / truncated answers:** 0 / 0.

Compared with iteration 8 (85.4% at 10 excerpts, Claude via `claude -p`, model not recorded): retrieval is the same (evidence session found 97.0% in both), so the 7-point gap comes from the answerer/judge pair, not from memory.

## Per-question diagnosis: summary

Rule diagnosis (no LLM):

| category | ok_full | ok_partial | ok_abstain | wrong_answer | wrong_retrieval | wrong_abstain |
|---|---|---|---|---|---|---|
| overall (500) | 355 (71%) | 14 (3%) | 23 (5%) | 48 (10%) | 53 (11%) | 7 (1%) |

Root cause of the 108 wrong answers (analyst, one cause each):

| owner | n | root causes |
|---|---|---|
| **retrieval** | **68 (63%)** | not recalled at all 37 · ranked just below the budget 20 · session recalled but the answer turn trimmed/condensed away 10 · (1 other) |
| answerer | 30 (28%) | over-abstaining with evidence present 9 · wrong detail 8 · generic preference advice 8 · date arithmetic 2 · counting 2 · invented answer to an `_abs` question 1 |
| judge | 8 (7%) | Haiku 4.5 calls grounded personalisation "fabrication" (preference), rejects a right "0 / not recorded" on `_abs`, or penalises hedging |
| dataset | 2 | ambiguous interval / gold date differs from session dates |

Where the missing evidence sits (deep recall, 60 excerpts, same brain and clock): 63 evidence sessions outside the context; **32 ranked 11–20**, 23 ranked 21–60, 8 not in the top 60. Of the 101 wrong answerable questions, 19 have every missing session ranked ≤ 20 and 11 more ≤ 30. Note: in 59 wrong questions no evidence session is ranked out — the session is in, but either the answering failed or the excerpt dropped the turn that holds the answer (the excerpt-level failure is under-counted by the session-level "all evidence" metric).

## Key insights (from the 500 per-question insights, ordered by expected gain)

1. **Query-aware excerpt selection (≈15–20 questions).** The single most repeated insight: the evidence session is ranked 1–8, but the excerpt shown is the session head or a window that misses the answer turn, or the long assistant turn is cut at 1,500 chars with `…[condensed]` (e.g. 89527b6b "blue scaly body", 7a8d0b71 budget section, 8752c811 Nth list item, c4ea545c "four times a week", gpt4_e414231f, 71017277, 6e984302). Fix: pick the window by the best-matching turns inside the session, and condense long assistant turns around the query match instead of keeping the head. This also explains why single-session-assistant (100% session recall) is only 85.7% J.
2. **Larger or adaptive excerpt budget for aggregation / multi-event questions (≈19–30 questions).** Multi-session counting ("how many…", "total…", "which … first") needs 3–6 sessions; 32 missing sessions sit at ranks 11–20, crowded out by off-topic sessions. Earlier measurement: 20 excerpts ≈ 89.8% J (iteration 8). Cheaper variant: per-session diversity (at most one or two excerpts per session) plus `episodes=20` only when the question is an aggregation/temporal-ordering one.
3. **Category → instance query expansion (≈12–15 questions).** "kitchen appliance" vs "smoker", "homegrown ingredients" vs "cherry tomatoes from my garden", "business milestone" vs "signed a contract", "art-related event" vs "History Museum tour", "doctors" vs "ENT/primary care". The LoCoMo A/B with extra queries from the calling model (`also`, commit 0e743a9) lifted multi-hop all-evidence 56% → 71%; `--expand-model` is not wired for LongMemEval yet and should be the next A/B.
4. **User self-attributes buried in off-topic sessions (≈6 questions).** Age, family size, years of experience ("I'm 32" inside a skincare chat): keyword recall ranks by topic, so these never surface (3c1045c8, ba358f49, 157a136e, c18a7dc8, 8e91e7d9, d01c6aa8). `--mode full` fact extraction (one fact per self-disclosure) is the structural fix; a cheap alternative is indexing first-person attribute sentences as separate units.
5. **Preference questions (13 of 17 wrong).** Retrieval: MRR 0.52, session found 76.7% — the question ("any tips for tonight?") shares no words with the stated preference; expansion toward the user's interests helps. Answering: 8 answers bury the one relevant preference under generic tips (409 words on average vs 160 elsewhere). An answer-prompt variant "anchor on the user's stated preferences/constraints; drop generic items" is worth an A/B, but the official template must stay for public numbers.
6. **Answerer: commit instead of abstaining (9) and count only stated facts (≈6).** Over-abstention with evidence present (51c32626, a11281a2, 37f165cf, a2f3aa27, 0e5e2d1a), and the reverse on `_abs` multi-session totals (answers "0" or a partial sum when a component was never stated: 88432d0a_abs, edced276_abs, gpt4_372c3eed_abs, 09ba9854_abs). Multi-session abstention is 6/12. These are model behaviour; check whether a stronger answerer (e.g. Sonnet) closes them before changing prompts.
7. **Judge: Haiku 4.5 is noticeably strict on preference and `_abs`.** 8 of the 108 wrong verdicts are judge errors (analyst), 4 of them preference/abstention. Before a public LongMemEval number, re-judge the cached answers with GPT-4o (`--judge-model openai/gpt-4o`, answers come from the cache) — needs an OpenRouter key with a spending limit above $0 (the current key is capped at $0.00).

## Reproduce

```bash
D=/tmp/eval-data OUT=/tmp/eval-out
python experiments/eval/run.py --bench longmemeval --data $D/longmemeval_s_cleaned.json --out $OUT --sample 0 \
  --answer-model claude-cli/claude-haiku-5-5 --judge-model claude-cli/claude-haiku-4-5 --parallel 8 --label haiku55-ep10
python experiments/eval/diagnose_lme.py $OUT/results-longmemeval-haiku55-ep10.json $D/longmemeval_s_cleaned.json \
  $OUT $OUT/diagnosis-haiku55-ep10 --analyst-model claude-cli/claude-sonnet-5-5
```

Data download without the HF CDN: `datasets-server.huggingface.co/rows?dataset=lorenoamori/longmemeval&config=default&split=train&offset=N&length=10` for N = 0..490, then rows sorted by `row_idx` (`truncated_cells` must be empty; checked).
