# LoCoMo: rewire vs Mem0, Zep and others

> **Historical.** This experiment ran on rewire's earlier Python implementation, which has been removed; its script is in git history (before the commit that removed `src/`). Current benchmarks run through [`experiments/eval`](../eval) on the Rust binary.

Official prompts: [`../eval/locomo_prompts.py`](../eval/locomo_prompts.py) (copied verbatim from [mem0ai/memory-benchmarks](https://github.com/mem0ai/memory-benchmarks)) · Per-question results: [`results.json`](results.json)

## Setup

- **Data:** [LoCoMo](https://github.com/snap-research/locomo) (ACL 2024). Iteration was done on 3 conversations (`conv-26`, `conv-30`, `conv-49`, 389 questions); the final configuration was run on **all 10 conversations: 272 sessions, 1,540 questions** in categories 1–4. Category 5 (adversarial) is excluded, the same as in the Mem0 harness.
- **Ingest:** each session is consolidated into a rewire brain in order, exactly like a harness transcript (`claude -p`). The inbox is approved in full (the benchmark has no human reviewer).
- **Answers:** `recall` retrieves the top-K memories → an LLM answers with **Mem0's official answer prompt** (memories sorted chronologically with dates). Difference: 8 questions per call (to save cost), whereas Mem0 uses 1 per call.
- **Scoring:** **Mem0's official judge** (CORRECT/WRONG) = the J score, plus token F1 as a strict comparison.

## Full results: 10 conversations, 1,540 questions

Final configuration (`general` profile, event dates, 10 episodes, top-50, ~4.7k tokens of context/question):

| Configuration | single-hop (841) | multi-hop (282) | temporal (321) | open-domain (96) | **Overall J** | F1 |
|---|---|---|---|---|---|---|
| rewire, hybrid (BM25 + WordLlama semantics) | 95.8% | 95.7% | 94.1% | 75.0% | **94.2% ± 1.2** | 0.454 |
| rewire, keywords only (`REWIRE_SEMANTIC=0`) | 96.9% | 96.5% | 94.1% | 76.0% | **94.9% ± 1.1** | 0.451 |
| Mem0 platform (published, same judge) | 91.2 | 91.3 | 92.0 | 72.7 | **92.5** | – |

Per conversation (hybrid): 88.8% (conv-43) to 97.4% (conv-26, conv-41).

**Semantic ablation:** semantics **does not help** on LoCoMo. Of the 1,540 questions, 17 were correct only with hybrid and 29 were correct only with keywords (McNemar p ≈ 0.10, not significant). The retrieved context is 87% identical. The reason: raw episodes + bilingual keywords from consolidation already cover most paraphrase cases, and static embeddings (WordLlama) are weaker than BM25 for names and specific details. On the synthetic eval ([experiments/retrieval](../retrieval/README.md)) semantics helps paraphrased queries, so it remains **optional** (`pip install 'rewire[semantic]'`), not required.

**Open-domain (75%) is the largest remaining weakness — and mostly not a memory problem.** Of the 24 wrong answers, the majority are speculative questions whose gold answer requires world knowledge or a specific guess ("which outdoor brand might endorse John?" → *Under Armour*; "which national park?" → *Voyageurs*; "what card game?" → *Exploding Kittens*). The memory provides the right clues; the answer is determined by the answerer's reasoning. Mem0 is also lowest in this category (72.7%).

## Iteration 7: from failure analysis to retrieval improvements

LoCoMo marks the evidence turns (`evidence`) for every question. [`evidence.py`](evidence.py) measures whether those turns make it into the context retrieved by rewire: **no LLM, ~20 seconds** for 1,540 questions. This is now the main iteration loop; the J score varies by ±3 points between runs (46 questions flip between correct/wrong with 87% identical context), which is too noisy for small changes.

**Diagnosis of 90 wrong answers (hybrid run):** 57% already had *all* the evidence in context (the error is in the answerer), 21% had some of it (almost all multi-hop), and in 22% the evidence was not retrieved.

| Change | All evidence in context (hybrid / keywords) | Full LoCoMo J | Decision |
|---|---|---|---|
| Before | 75.5% / 78.0% | 94.2% / 94.9% | – |
| **Facts pull in the transcript chunks they came from** (claim source → session + turn) | **79.5% / 79.2%** (temporal 81 → 90%) | **94.0%** (23 fixed, 25 broken vs hybrid) | Kept: better retrieval, J unchanged |
| Counting questions ("how many") get 2× chunks | multi-hop +0.8 points | 28/35 vs 29–31/35 | **Discarded**: the same event across several sessions gets counted twice |
| At most N chunks per session (diversity) | dropped (single-hop 94 → 84% at N=1) | – | **Discarded** |
| Timeline rules in extraction (start dates for durations) | requires re-ingest | not yet measured | Kept (applies to new consolidations) |

Bug found: claim sources stored session ids truncated to 8 characters (`conv-26-`), so the fact → transcript link was broken. Full ids are now stored; the eval brain was repaired from git history (`link_sources`).

**Conclusion:** on LoCoMo, retrieval is no longer the bottleneck. More complete evidence does not raise J because the remaining errors are in the answerer (picking the wrong person/event, miscounting) and in speculative questions. LoCoMo is saturated at ~94% for this setup; the next improvements need a benchmark that tests other things (LongMemEval: *knowledge update*, *abstention*).

## Iterations on the 3-conversation subset

| Configuration | single-hop | multi-hop | temporal | open-domain | **Overall J** | F1 | context/question |
|---|---|---|---|---|---|---|---|
| rewire v0.4 as-is (`coding` profile) | 73.8% | 90.0% | 38.5% | 92.3% | **69.7% ± 4.6** | 0.303 | ~1.8k tokens |
| + `general` profile + event dates | 88.2% | 95.0% | 93.8% | 92.3% | **91.3% ± 2.8** | 0.392 | ~1.9k tokens |
| + episodes (raw transcript chunks) | 95.2% | 100% | 96.9% | 96.2% | **96.7% ± 1.8** | 0.456 | ~4.4k tokens |
| + semantics, time window, history intent (v4) | – | – | – | – | **95.4%** | 0.443 | ~4.7k tokens |

v3 → v4: 0 fixed, 5 broken, context 83–93% identical: the difference is answer/judge variation, not a retrieval regression. ± = 95% confidence interval (binomial). n: single 187, multi 80, temporal 96, open 26.

### Numbers published by others (not rerun here)

| System | LoCoMo J | Source & notes |
|---|---|---|
| **Mem0 platform** (2026) | 92.5 (single 91.2 · multi 91.3 · temporal 92.0 · open 72.7) | [Mem0 documentation](https://github.com/mem0ai/mem0/blob/main/docs/core-concepts/memory-evaluation.mdx) — the **same** judge & prompt as this run, top-200 memories, ~7k tokens, answer model not stated, paid platform |
| Full-context (the whole conversation in the prompt) | 72.9 | Mem0 2025 paper ([arXiv 2504.19413](https://arxiv.org/abs/2504.19413)), an older, stricter judge |
| Mem0g / Mem0 (2025) | 68.4 / 66.9 | Mem0 2025 paper, older judge |
| Zep | ~66 according to the Mem0 paper; **75.1** according to Zep's own correction; latest claim 94.7 | Methodology dispute between the two vendors |
| LangMem / OpenAI Memory | ~58 / ~53 | Mem0 2025 paper |

**A fair comparison is only possible with the first row** (identical judge and answer prompt). The 2025 numbers use a much stricter judge and cannot be compared directly.

## So, are they better?

In this setup, **no** — across all 1,540 questions rewire scores 94.2–94.9% vs 92.5% for the Mem0 platform, with a smaller context (~4.7k vs ~7k tokens). (The 3-conversation subset briefly showed 96.7%; the full number is lower, as expected from the ±1.8 interval.) But **do not read this as "rewire beats Mem0"**:

1. **The gap is small**: +1.7 to +2.4 points, slightly above our ±1.1–1.2 interval, and with no interval on Mem0's side.
2. **Different answer model & judge**: we use Claude for consolidation, answering, **and** judging. Mem0's answer model is not published. A judge from the same family as the answerer may be more generous.
3. **The judge is very lenient** (one correct item from a list = correct, dates ±14 days, durations ±50%); our spot-check found "About seven years" judged correct for "Since 2016". The strict F1 (0.456) shows that answers are still often not exact.
4. **LoCoMo is small**: an entire conversation fits in the context window. Zep and Mem0 themselves state that this benchmark is easy and can be "bought" with a large context; BEAM (1M–10M tokens) is more relevant for production scale.
5. We have not yet run Mem0/Zep in the same harness.

What is valid: the **relative improvement** within rewire (69.7 → 96.7), measured with an identical setup.

## Why the rewire baseline lagged — and the insights adopted

| Cause (from analyzing wrong answers) | How competitors handle it | What was implemented in rewire |
|---|---|---|
| **No event dates.** Memories were recorded with the consolidation date; "kemarin" ("yesterday") was not converted. Temporal 38.5% | Mem0: a separate *temporal reasoning* pass that records when events happened. Zep: facts with validity windows (bi-temporal) | Claims get `valid_from` = the session date; the extractor must convert relative dates to absolute ones; answers are given memories in order with dates. **Temporal 38.5 → 93.8%** |
| **Extraction too sparse.** The coding profile deliberately discards "chatter" (3–5 claims/session) | Mem0: extract every fact, *ADD-only* (never overwrites) | A `general` profile (9–15 claims/session, people's names always mentioned); the `coding` profile remains the default for coding agents |
| **Fine details lost in summarization** (exact words, photo contents, poster text) | Zep: raw **episodes** are stored as the source of every fact; the full-context baseline (72.9) beat Mem0 (66.9) in the Mem0 paper | Session transcripts are stored (secrets masked), chunked every 6 turns, indexed, and returned by `recall` after the facts. **+23 correct answers, −2** |
| Word-only search | Hybrid: semantic + BM25 + entity/graph + reranker | Already present: BM25 + EN/ID stemming + bilingual keywords + spreading activation. **Not yet**: embeddings & reranker (see below) |

### Insights not yet implemented

- ~~**Embedding / semantic search**~~ → **implemented** (offline WordLlama, additive fusion). Does not help on LoCoMo (see the ablation), helps paraphrases on the synthetic eval; remains optional.
- ~~**ADD-only vs supersede**~~ → **implemented**: history questions ("dulu", "used to", "first") return `superseded` facts with a label.
- ~~**Time window**~~ → **implemented**: "in July 2023" gives a bonus to memories from that month.
- **Reranker** (Zep: cross-encoder / graph distance). Not yet; with static embeddings it does not help, and a reranker likely needs a heavier model.
- **Counting and time differences** (the remaining multi-hop/temporal failures) need reasoning on the answerer's side, not memory.

## Cost

Subset: ingest ~63 sessions × (extraction + judge) ≈ 125 `claude -p` calls (~10–25 seconds/session, running in the background after a session ends in real usage). Answers ~49 calls, judge ~20 calls per configuration.

Full run: 272 sessions ≈ 550 ingest calls, then ~195 answer calls + ~80 judge calls per configuration. The run was interrupted several times by usage limits; the harness resumed from the cache (`locomo_done.json`, `answers-*.json`, `judged-*.json`). The full run also found one real bug: `time_window` crashed on year ranges such as "2022-23" (fixed, with a test).
