# Retrieval experiments on BEAM 100K (2026-10-16, no LLM)

`--retrieval-only`, all 400 BEAM 100K questions and all 1,540 LoCoMo questions (regression check), k=30.
"All evidence" = share of questions with every source turn in the recalled context. Variants override
`Index.EP_WINDOW`/`EP_STEP` (turns per excerpt) or replace fixed-turn chunks with size-capped chunks
(at most N characters, at most 6 turns, one turn overlap); the `--episodes` count is raised to hold the
context budget near the baseline's ≈12k tokens.

| variant | semantic | BEAM all evidence | BEAM ctx tokens | LoCoMo all evidence |
|---|---|---|---|---|
| baseline: 6-turn excerpts, 10 excerpts | no | 44.8% | 11.9k | – |
| baseline | WordLlama | 47.0% | 12.2k | 74.3% |
| 4-turn excerpts × 14 | no | 47.9% | 11.7k | 74.3% |
| 2-turn excerpts × 25 | no | 51.5% | 12.0k | – |
| 2-turn excerpts × 25 | WordLlama | **54.6%** | 12.2k | **69.6%** (−4.7) |
| size-capped 1,200 chars × 40 | WordLlama | **54.1%** | 11.6k | **74.4%** (±0) |
| size-capped 1,200 chars × 20 | WordLlama | 48.5% | **5.8k** | 74.4% (10 excerpts) |
| 6-turn excerpts × 20 (double budget) | no | 58.6% | 22.4k | – |

Per category, size-capped 1,200 × 40 vs baseline (semantic): contradiction 60 → 80%, knowledge update 75 → 85%,
multi-session 37.5 → 52.5%, temporal 80 → 95%, preference 59 → 56%, instruction following 37.5 → 35%,
summarization 0 → 5.6%, event ordering 2.5 → 5%.

Findings:
- **Excerpt size, not count, is the lever.** BEAM turns are long (assistant replies with code), so a 6-turn
  excerpt spends most of its tokens on text unrelated to the question. Smaller excerpts at the same budget
  raise all-evidence by 7–8 points.
- **Fixed small turn windows break LoCoMo** (short chat turns lose their neighbours: −4.7 points); capping the
  excerpt by size keeps LoCoMo unchanged while giving BEAM the gain. Size-capped chunks are the candidate default.
- Instruction following, preference following, summarization and event ordering barely move with any
  chunking: they are not ranking problems (see the summary: standing instructions belong in `prime`,
  summaries/ordering need per-session summaries).
- Semantic recall (WordLlama) adds +2.2 points on BEAM at the baseline (it did not help on LoCoMo).

Not yet measured: the answer score with size-capped excerpts (one LLM run on the 100-question sample), and
500K/1M sizes.
