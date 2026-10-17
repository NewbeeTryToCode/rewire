# PersonaMem (v1): 32k all, 128k and 1M sampled, Haiku 5.5

Run of 2026-10-17. Per-size runner reports: `2026-10-17-personamem-{32k,128k-s20,1M-s20}-haiku55.md`.

| | |
|---|---|
| Data | Hugging Face `bowen-upenn/PersonaMem-v1` (the COLM 2025 release; `bowen-upenn/PersonaMem` now redirects there), downloaded directly (the HF file CDN was opened for this session). Validated: 32k 589 questions / 37 contexts, 128k 2,727 / 110, 1M 2,674 / 33; every context id exists; answer keys spread over (a)–(d) |
| Memory | `--mode episodes`, k=30, 10 excerpts, recall with the 2026-10-16 changes (whole turns + condensing, session opener, semantic at 0.25, 2× excerpts for counting questions); excerpts after the question's position are dropped |
| Answerer | `claude-haiku-5-5` via `claude -p`, official PersonaMem instruction, options shown; **no judge** (the option letter is matched) |
| Sample | 32k: all 589. 128k and 1M: 20 per question type (seed 0), 140 each |

## Results

| question type (32k name / 1M name) | 32k | 128k | 1M |
|---|---|---|---|
| recall_user_shared_facts | 70.5% (129) | 70.0% | 50.0% |
| recalling_facts_mentioned_by_the_user (32k only) | 41.2% (17) | | |
| recalling_the_reasons_behind_previous_updates / revisit_reasons_behind_preference_updates | 80.8% (99) | 75.0% | 75.0% |
| track_full_preference_evolution / _updates | 64.0% (139) | 50.0% | 60.0% |
| acknowledge_latest_user_preferences (128k/1M only) | | 65.0% | 65.0% |
| provide_preference_aligned_recommendations | 58.2% (55) | 40.0% | 60.0% |
| generalizing_to_new_scenarios | 45.6% (57) | 35.0% | 30.0% |
| suggest_new_ideas | 25.8% (93) | 55.0% | 20.0% |
| **overall** | **59.3%** (589) | **55.7%** (140) | **51.4%** (140) |

| | 32k | 128k | 1M |
|---|---|---|---|
| context tokens per question (mean) | 11.3k | 11.0k | 11.7k |
| compression vs the history up to the question | 3.1× | 14.5× | 99.4× |
| answer prompt tokens | 15.3k | 15.0k | 16.0k |
| retrieval p50 / p95 | 42 / 181 ms | 57 / 214 ms | 77 / 220 ms |

Cost: $0 billed (subscription); API-price equivalent $2.89 for the 869 answers. The runs used the recall
of 2026-10-16 (sentence vectors in condensing, since dropped for latency with the same evidence on LongMemEval).

Questions per type are 17–139 at 32k and 20 at 128k/1M, so per-type numbers carry ±10–20 points of sampling
error (95% CI ≈ ±1.96·√(p(1−p)/n)); the overall 32k number ±4 points, 128k/1M ±8 points.

## Reading

- **For scale:** the PersonaMem paper reports frontier models at roughly 50% overall when given the whole
  history in context. These numbers are not comparable (different answerer, rewire's retrieved context instead
  of the full history, sampled 128k/1M), but they place rewire + Haiku 5.5 in the same range while sending ~11k
  tokens instead of up to ~1M.
- **Accuracy falls slowly with history size** (59% → 56% → 51%) while the context stays at ~11k tokens: retrieval
  keeps up with a 99× compression at 1M.
- **Recall-type questions are strong** (shared facts, reasons behind updates: 70–81%). **Generative questions are
  weak**: `suggest_new_ideas` (20–26%, the 128k 55% is a 20-question sample) asks for something the user has *not*
  done yet, and `generalizing_to_new_scenarios` (30–46%) asks to transfer a preference to a new domain. Both need
  the user's whole preference profile rather than the excerpts that match the question's words — the case for
  `--mode full` (extracted preference facts) or a profile summary, not for more excerpts.
- No per-question diagnosis was run for PersonaMem (multiple choice, no evidence annotation to locate misses).
