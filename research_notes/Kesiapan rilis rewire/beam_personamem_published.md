# Published results on BEAM and PersonaMem (plus other memory benchmarks), as of Oct 2026

Method note: in this session arxiv.org, mem0.ai, alphaxiv and most vendor sites could not be fetched directly (proxy/DNS denials). Findings come from web-search snippets and GitHub code-search fragments, often third-party repos that quote the papers. **Primary paper tables (full per-ability grids) could not be read directly.** Treat numbers attributed to "the paper via repo X" as secondary until checked against the PDF. Almost every system-level number is **self-reported by the vendor**.

## Q1. What did the BEAM paper report (long-context LLMs, RAG, LIGHT), and how is it scored?

### Takeaway
BEAM (Tavakoli et al., arXiv 2510.27246, 31 Oct 2025; ICLR 2026) scores answers with an LLM judge over rubric "nuggets" (0 / 0.5 / 1 partial credit, averaged per question), plus Kendall-tau for event ordering. The paper's absolute numbers are low: the best is LIGHT on Llama-4-Maverick at about 0.36 at 100K/500K, 0.336 at 1M and 0.266 at 10M. Contradiction resolution is near zero for every method.

### Cited Findings
**Dataset and design**
- Paper: "Beyond a Million Tokens: Benchmarking and Enhancing Long-Term Memory in LLMs" by Tavakoli, Salemi, Ye, Abdalla, Zamani and Mitchell. arXiv 2510.27246, submitted 31 Oct 2025, ICLR 2026 poster. — [arXiv](https://arxiv.org/abs/2510.27246); [ICLR](https://iclr.cc/virtual/2026/poster/10006595)
- Contents: 100 synthetic conversations, 2,000 human-validated questions, 10 abilities, sizes 100K/500K/1M/10M. Three abilities are new: contradiction resolution, event ordering and instruction following. — [arXiv](https://arxiv.org/abs/2510.27246); [paper notes (RhysEJF repo)](https://github.com/RhysEJF/cognitive-shift-resources/blob/main/research/memory/tavakoli-2026-beam-light.md)
- Per-ability table, quoted in a third-party repo: 200 questions per ability. — [somnigraph notes](https://github.com/AlexisOlson/somnigraph/blob/main/research/sources/beam.md)
- 10M tier: 10 conversations with about 200 questions. — [Zikkaron README](https://github.com/amanhij/Zikkaron); [Mem0 memory-benchmarks](https://github.com/mem0ai/memory-benchmarks)
- Mem0 runs 700 questions at 1M (491/700 pass). That implies 35 conversations × 20 questions at 1M, and loom's README lists 35 conversations at 500K and 1M (20 at 100K). — [Mem0 memory-benchmarks via search](https://github.com/mem0ai/memory-benchmarks); [use-fabrica/loom README](https://github.com/use-fabrica/loom/blob/main/benchmark/beam/README.md)
- Generation pipeline: an MBTI-grounded profile and timeline; user turns from LLaMA-3.3-70B; assistant turns from GPT-4.1. Human ratings: coherence 4.53, realism 4.57, complexity 4.64 (out of 5); κ = 0.70–0.78. — [RhysEJF notes](https://github.com/RhysEJF/cognitive-shift-resources/blob/main/research/memory/tavakoli-2026-beam-light.md)
- Dataset on Hugging Face: `Mohammadta/BEAM`, with splits 100K / 500K / 1M (10M is a separate release). — [loom README](https://github.com/use-fabrica/loom/blob/main/benchmark/beam/README.md); [official repo](https://github.com/mohammadtavakoli78/BEAM)

**Backbones and methods**
- Four 1M-context backbones: GPT-4.1-nano, Gemini-2.0-flash, Qwen2.5-32B-AWQ and Llama-4-Maverick-fp8. Each was run three ways: Vanilla (long context), RAG (top-k retrieval) and LIGHT. — [Paper-Notes-en](https://github.com/zhaoyang97/Paper-Notes-en/blob/main/docs/ICLR2026/llm_evaluation/beyond_a_million_tokens_benchmarking_and_enhancing_long-term_memory_in_llms.md); [RhysEJF notes](https://github.com/RhysEJF/cognitive-shift-resources/blob/main/research/memory/tavakoli-2026-beam-light.md)
- LIGHT has three parts:
  - Episodic memory: a FAISS index with bge-small-en-v1.5 embeddings.
  - A scratchpad of salient facts distilled by Qwen2.5-32B and compressed with GPT-4.1-nano.
  - Working memory of the last N turns.

  It also uses a yes/no noise filter. The best retrieval budget is K=15. — [RhysEJF notes](https://github.com/RhysEJF/cognitive-shift-resources/blob/main/research/memory/tavakoli-2026-beam-light.md)

**Scoring**
- LLM-as-judge with a rubric of atomic nuggets. Each nugget scores 0 / 0.5 / 1, and the question score is the nugget mean. The scale is 0–1. — [search summary of the paper/third-party write-up](https://mnemoverse.com/docs/technology/benchmarks/beam); [vectorize-io/agent-memory-benchmark models.py](https://github.com/vectorize-io/agent-memory-benchmark) ("continuous score 0-1 (used by BEAM paper scoring)")
- Event ordering: items are aligned to the reference, then P/R/F1 is computed, then Kendall τ-b. Final score = (τ+1)/2 × F1. — [Scriptorium protocol notes](https://github.com/Qi202/Scriptorium/blob/main/docs/experiments/protocols/unified_evaluation_protocol.md)
- Mem0's re-implementation differs from the official scoring:
  - It judges each nugget with gpt-5.
  - It clamps scores (≥0.75→1, ≥0.25→0.5, else 0).
  - It reports both `avg_score` and a pass rate (share of questions with score ≥0.5).

  The Scriptorium notes describe this as "systematically more lenient than official". — [Scriptorium notes](https://github.com/Qi202/Scriptorium/blob/main/docs/experiments/protocols/unified_evaluation_protocol.md); [mem0 memory-benchmarks run.py](https://github.com/mem0ai/memory-benchmarks)
- Vendors' "BEAM %" is usually the mean nugget score × 100, not an accuracy. Mem0 says its "64.1" is avg score, and its pass rate is 70.1%. — [Mem0 "How to read memory benchmarks"](https://mem0.ai/library/agent-memory/how-to-read-memory-benchmarks)

**Reported numbers (paper, mostly via secondary quotes)**

| Method (backbone) | 100K | 500K | 1M | 10M | Source |
|---|---|---|---|---|---|
| LIGHT (Llama-4-Maverick), best in paper | 0.358 | 0.359 | 0.336 | 0.266 | [Scriptorium, quoting paper Table 1](https://github.com/Qi202/Scriptorium/blob/main/docs/experiments/protocols/unified_evaluation_protocol.md); [bigfeels](https://github.com/promptclickrun/bigfeels/blob/main/benchmarks/results/2026-10-08-beam-answers.md); [AMB external_results.json](https://github.com/vectorize-io/agent-memory-benchmark/blob/main/external_results.json) |
| RAG (Llama-4-Maverick) | 0.323 | 0.330 | 0.307 | n/a found | [bigfeels](https://github.com/promptclickrun/bigfeels/blob/main/benchmarks/results/2026-10-08-beam-answers.md) |
| GPT-4.1-nano: vanilla → LIGHT at 10M | – | – | – | 0.109 → 0.226 (+107.3%) | [search summary of paper](https://mohammadtavakoli78.github.io/beam-light/) |
| Long-context baselines at 10M (range) | – | – | – | 0.133–0.199 | search summary (secondary) |
| LIGHT vs ReadAgent (App. Table 13) | ~0.31 vs 0.19–0.21 (avg over 100K/500K/1M) | | | | [arXiv PDF via search](https://arxiv.org/pdf/2510.27246) |

- LIGHT's average gain over the strongest baseline is +3.5% to +12.69%, depending on backbone. At 1M the gains are +75.9% (GPT-4.1-nano) and +60.1% (Qwen2.5-32B). At 10M they are +155.7% (Maverick) and +107.3% (GPT-4.1-nano). — [arXiv abstract](https://arxiv.org/abs/2510.27246); [zhaoweiguo notes](https://github.com/zhaoweiguo/knowledge.zhaoweiguo.com)
- Ability-level gains from LIGHT: summarization +160.6%, preference following +76.5%, information extraction +56.7%, temporal reasoning +56.3%, multi-hop +27.2%. Abstention is the easiest ability for all methods. Contradiction resolution is the hardest, "near floor" for everyone; LIGHT CR ≈ 0.03–0.05. — [RhysEJF notes](https://github.com/RhysEJF/cognitive-shift-resources/blob/main/research/memory/tavakoli-2026-beam-light.md); [inite-brain-service audit](https://github.com/inite-ai/inite-brain-service/blob/main/docs/roadmap/v10-audit-2026-08.md); [Scriptorium](https://github.com/Qi202/Scriptorium/blob/main/docs/experiments/protocols/unified_evaluation_protocol.md) ("all methods contradiction_resolution ≈ 0")
- Ablation: removing working memory is roughly neutral at ≤1M but costs −5.7% at 10M. — [RhysEJF notes](https://github.com/RhysEJF/cognitive-shift-resources/blob/main/research/memory/tavakoli-2026-beam-light.md)
- A partial extract of the 100K abstention row reads "0.300 0.650 0.475 0.200 0.800 0.600". Its column mapping is unclear. — [search extract of arXiv HTML](https://arxiv.org/html/2510.27246)
- The cdeust/Cortex repo compares against a "LIGHT" per-ability column at 100K: CR 0.050, EO 0.266, IE 0.375, TR 0.075, overall 0.329. Its overall does not match the 0.358 above, so the column's provenance (backbone, version) is unclear. — [cdeust/Cortex a3_beam_100k_post_refactor.md](https://github.com/cdeust/Cortex/blob/main/benchmarks/results/a3_beam_100k_post_refactor.md)

### Inferences
- The paper's numbers (~0.27–0.36) and vendor numbers (~0.4–0.85) are on the same nominal 0–1 nugget scale. However, vendors use much stronger readers and judges (GPT-5-class, Gemini 3) and sometimes lenient clamping. Much of the gap is backbone and judge, not memory architecture.
- Each size tier is small: 10M has about 200 questions, and Mem0 estimates about ±6.9 pts sampling noise there. Differences of a few points are not meaningful.

### Gaps
- The full per-ability × per-backbone × per-size grid from the BEAM paper (Table 1/2) could not be read, because arxiv was blocked. Only the Maverick overall row and scattered ability facts were obtained. The judge model used in the paper is also unconfirmed: one vendor page says the Hindsight result "used Llama-4-Maverick as the judge", which may mirror the paper.

## Q2. Have memory projects published BEAM numbers?

### Takeaway
Yes, since early 2026 BEAM has become the headline "can't-stuff-the-context" benchmark. Hindsight (64.1% at 10M), Honcho, Mem0, Cognee, Exabase M-1 (68.0% at 10M), past.dev (85.03% at 10M, self-reported), AutoMem and several small OSS projects all publish BEAM numbers. I found no BEAM numbers from Zep, Letta, Supermemory, MemOS or EverMemOS. All of these are self-reported, with differing readers and judges.

### Cited Findings
| System | 100K | 500K | 1M | 10M | Reader/judge, date, notes | Source |
|---|---|---|---|---|---|---|
| Hindsight (Vectorize) | 73.4 | 71.1 | 73.9 | 64.1 | Blog 2 Apr 2026. Self-reported; the vendor's AMB page says the judge was Llama-4-Maverick. Still cited as "#1 on BEAM" in Oct 2026. | [Hindsight blog](https://hindsight.vectorize.io/blog/2026/04/02/beam-sota); [AMB comparison](https://hindsight.vectorize.io/guides/2026/04/21/comparison-agent-memory-benchmark-hindsight-vs-alternatives); [memlake post 13 Oct 2026](https://github.com/vectorize-io/hindsight/blob/main/hindsight-docs/blog/2026-10-13-memlake.md) |
| Honcho (Plastic Labs) | 63.0 | 64.6 (blog: 64.9) | 61.8 (blog: 63.1) | 40.9 (40.6 cited by Hindsight) | Self-reported, small inconsistencies across its own pages. Baseline at 100K: Claude Haiku 4.5 full-context, 0.533. | [honcho.dev/evals](https://honcho.dev/evals/); [Benchmarking Honcho blog](https://honcho.dev/blog/blog/benchmarking-honcho); [Honcho on X](https://x.com/honchodotdev/status/2007110666348597652) |
| Mem0 (platform, "token-efficient algorithm") | – | – | 64.1 avg (70.1% pass, 491/700) | 48.6 avg (50.5% pass, 200 Q) | Apr 2026, gpt-5 judge, clamped partial credit. Per ability: PF 88.3 (1M) / 90.4 (10M); CR 35.7 / 32.5; TR 61.8 / 16.3. Average retrieval about 6.7–6.9K tokens. | [Mem0 blog](https://mem0.ai/blog/mem0-the-token-efficient-memory-algorithm); [mem0ai/memory-benchmarks](https://github.com/mem0ai/memory-benchmarks); [Mem0 docs](https://docs.mem0.ai/core-concepts/memory-evaluation) |
| Exabase M-1 | SOTA claimed | – | SOTA claimed | 68.0 | Gemini 3 Flash reader; competitors cited with Gemini 3 Pro. Self-reported. | [Exabase blog](https://exabase.io/blog/exabase-m1-achieves-state-of-the-art-on-beam-benchmark) |
| past.dev | – | – | – | 85.03 | Self-reported. Competitor figures are "the highest each vendor published". | [past.dev/benchmarks](https://past.dev/benchmarks) |
| AutoMem | ahead of Honcho at every scale | | | 57.4 | Self-reported, "submission under review", not a SOTA claim. | [automem.ai/benchmarks](https://automem.ai/benchmarks/) |
| Cognee | 0.79 | – | – | 0.67 ("exploratory") | Blog Jun 2026, claims +6.5% over prior SOTA at 100K. | [Cognee blog](https://www.cognee.ai/blog/deep-dives/benchmarking-cognee-on-beam); [opensourceaireview](https://www.opensourceaireview.com/blog/which-ai-memory-layer-has-the-best-published-benchmarks-in-2026) |
| Mnemosyne OS | 61.7 | – | – | 49.2 | Self-reported, published per question. Another run (mnemosyne-oss) used a Llama-3.3-70B reader with a DeepSeek V4 Flash judge and says it is "not directly comparable". | [mnemosyne-os blog](https://mnemosyne-os.io/blog/agent-memory-tools-compared); [mnemosyne-oss beam-benchmark.md](https://github.com/mnemosyne-oss/mnemosyne/blob/main/docs/beam-benchmark.md) |
| Zikkaron (OSS) | – | – | – | 0.404 | Self-reported README | [Zikkaron](https://github.com/amanhij/Zikkaron) |
| bigfeels (OSS) | 54.8 / 60.6 (agent search) | 51.8 / 55.8 | 51.2 / 55.5 | – | GPT-6.1/GPT-6 readers, 8 Oct 2026, self-reported | [bigfeels results](https://github.com/promptclickrun/bigfeels/blob/main/benchmarks/results/2026-10-08-beam-answers-agent.md) |

- No vendor-published BEAM numbers were found for Zep, Letta, Supermemory, MemOS or EverMemOS. One comparison says "We have not found Zep-published BEAM numbers" and lists Letta BEAM as "Not published". — [search summary: dev.to comparison](https://dev.to/varun_pratapbhardwaj_b13/5-ai-agent-memory-systems-compared-mem0-zep-letta-supermemory-superlocalmemory-2026-benchmark-59p3); [opensourceaireview](https://www.opensourceaireview.com/blog/which-ai-memory-layer-has-the-best-published-benchmarks-in-2026)
- Vectorize maintains an open "Agent Memory Benchmark" (AMB) harness with an `external_results.json` that lists BEAM entries, for example Honcho 0.63 at 100K and LIGHT (Maverick) 0.266 at 10M. Several OSS projects reuse it (redhat-ai-americas/memory-hub, yantrikdb). — [vectorize-io/agent-memory-benchmark](https://github.com/vectorize-io/agent-memory-benchmark); [benchmarks.hindsight.vectorize.io](https://benchmarks.hindsight.vectorize.io/)
- Hindsight argues that LoCoMo and LongMemEval now let "dump everything into context" baselines score competitively, and that BEAM 10M is the one where stuffing is impossible. — [Hindsight blog, 31 Jul 2026](https://github.com/vectorize-io/hindsight/blob/main/hindsight-docs/blog/2026-07-31-evaluate-agent-memory-system.md)

### Inferences
- The current "leaderboard" at 10M, all self-reported and on different stacks: past.dev 85.0 > Exabase 68.0 > Hindsight 64.1 > AutoMem 57.4 > Mnemosyne 49.2 ≈ Mem0 48.6 > Honcho 40.9 ≈ Zikkaron 40.4 >> LIGHT paper 26.6. At 100K: Cognee 79 > Hindsight 73.4 > Honcho 63.0 > Mnemosyne 61.7.
- A new project reporting BEAM should state: the tier, number of questions, reader model, judge model, whether partial credit is clamped, and avg-score vs pass-rate.

### Gaps
- Per-ability breakdowns from Hindsight, Honcho, Exabase and Cognee could not be fetched. The only per-ability vendor numbers found are Mem0's.
- Exabase's 100K/1M exact values were not retrieved.

## Q3. What did the PersonaMem paper report, which memory systems report PersonaMem, and is there a v2?

### Takeaway
PersonaMem v1 (Jiang et al., "Know Me, Respond to Me", arXiv 2504.14225, COLM 2025) is 4-option multiple choice over 32k/128k/1M histories with 7 question types. The best frontier models (GPT-4.5, GPT-4.1, Gemini-1.5-Flash) reach about 52% at 128k; o1 is 50% and Gemini-2.0-Flash 49%. Memory vendors report roughly 38–68% on PersonaMem, usually the 32k split. PersonaMem-v2 (arXiv 2512.06688, Dec 2025) exists: frontier models get 37–48%, and the authors' agentic memory gets about 55%.

### Cited Findings
**Paper**
- Paper: "Know Me, Respond to Me: Benchmarking LLMs for Dynamic User Profiling and Personalized Responses at Scale". COLM 2025, repo bowen-upenn/PersonaMem. The README says GPT-4.5, GPT-4.1 and Gemini-1.5 are best "but still hovered around 52% in a multiple-choice setting". The abstract phrasing is "~50% overall accuracy" for GPT-4.1, o4-mini, GPT-4.5, o1 and Gemini-2.0. — [GitHub](https://github.com/bowen-upenn/PersonaMem); [arXiv HTML](https://arxiv.org/html/2504.14225v2); [COLM PDF](https://cogcomp.seas.upenn.edu/papers/JHCLYCea25.pdf)
- Context tiers: the leaderboard figure has a top panel (up to 20 sessions / 128k) and a bottom panel (up to 60 sessions / 1M). A 32k split also exists in the data (`questions_32k`, `shared_contexts_32k`). — [GitHub](https://github.com/bowen-upenn/PersonaMem); [saxenauts/persona eval logs](https://github.com/saxenauts/persona/blob/main/docs/archive/eval/2026-02/FAILURE_ANALYSIS_20260209.md)
- Seven question types: recall_user_shared_facts, suggest_new_ideas, acknowledge_latest_user_preferences, track_full_preference_evolution, revisit_reasons_behind_preference_updates (also named recalling_the_reasons_behind_previous_updates), provide_preference_aligned_recommendations and generalizing_to_new_scenarios. — [MemCoE evaluator](https://github.com/Applied-Machine-Learning-Lab/ACL2026_MemCoE/blob/main/MGI_module/evaluators/mcq_evaluator.py); [saxenauts/persona](https://github.com/saxenauts/persona)
- 128k overall accuracy from the paper's Figure 4 (upper panel), as transcribed in a third-party repo:

  | Model | Accuracy |
  |---|---|
  | GPT-4.1 | 52% |
  | GPT-4.5 | 52% |
  | Gemini-1.5-Flash | 52% |
  | o1 | 50% |
  | Gemini-2.0-Flash | 49% |
  | o3-mini | 39% |
  | Llama-3.1-8B | 31% |
  | Claude-3.5-Haiku | 30% |

  — [ya-way/CSIEE personamem-v1-comparison](https://github.com/ya-way/CSIEE/blob/main/docs/personamem-v1-comparison.md)
- GPT-4o at 32k full context: 39.20% overall, as cited by PersonaVLM from the original paper. — [PersonaVLM arXiv 2604.13074](https://arxiv.org/pdf/2604.13074)
- Per-type pattern: models do relatively well on recalling facts and on reasons behind updates. They are weak on generalizing to new scenarios and weakest on suggest_new_ideas / novel suggestions. The paper's Figure 9 gives per-type results for GPT-4o and GPT-4o-mini at 128k, but the exact values were not retrieved. — [GitHub](https://github.com/bowen-upenn/PersonaMem); [dev.to summary](https://dev.to/aimodels-fyi/ai-fail-llms-forget-you-new-benchmark-exposes-personalization-gaps-4o4o)

**Memory systems on PersonaMem (4-option accuracy, usually 32k)**

| System | Accuracy | Context tokens | Source / note |
|---|---|---|---|
| MIRIX | 38.30 | – | MemOS eval table (MemTensor HF), self-run by MemOS |
| Mem0 | 43.12 | 140 | MemOS eval table |
| Zep | 57.83 | 1,657 | MemOS eval table |
| MemoBase | 58.89 | 2,092 | MemOS eval table |
| MemOS | 61.17 | 1,424 | MemOS eval table / MemOS paper ("61.2%"), self-reported |
| Memind | 67.91 | – | memind README, self-reported |

Sources: [MemTensor/MemOS_eval_result](https://huggingface.co/datasets/MemTensor/MemOS_eval_result); [MemOS arXiv 2507.03724](https://arxiv.org/pdf/2507.03724); [openmemind/memind](https://github.com/openmemind/memind)

- An independent re-run (BMAM paper, arXiv 2601.20465, Table 14, "precision") gives MIRIX 38.4, **Mem0 53.9**, Zep 57.8, Memobase 58.9 and **MemOS 34.0**. The MemOS figure was re-run with the official MemOS scripts. This contradicts MemOS's self-reported 61.17 and Mem0's 43.12. — [BMAM arXiv 2601.20465](https://arxiv.org/pdf/2601.20465)
- memind's per-type breakdown: recalling_the_reasons_behind_previous_updates 88.89% (88/99), suggest_new_ideas 38.71% (36/93), track_full_preference_evolution 60.43% (84/139). Type sizes visible: 99, 93, 139, and 57 for generalizing_to_new_scenarios. — [memind README](https://github.com/openmemind/memind); [bettermemory pm1-E2.json](https://github.com/0Mattias/bettermemory)
- O-Mem (arXiv 2511.13593) and MindMemOS (arXiv 2608.12428) also evaluate on PersonaMem with MC accuracy; their numbers were not retrieved. RGMem (2510.16392) also reports PersonaMem. — [O-Mem](https://arxiv.org/pdf/2511.13593); [MindMemOS](https://arxiv.org/pdf/2608.12428); [RGMem](https://arxiv.org/pdf/2510.16392)
- Small OSS projects report very high scores on subsets. memory-hub shows 94.9% on recalling reasons and 94.7% on generalizing; saxenauts/persona reports 63–67% on 100-question subsamples. These are self-reported and use small n. — [memory-hub RESULTS.md](https://github.com/redhat-ai-americas/memory-hub/blob/main/benchmarks/RESULTS.md); [saxenauts/persona](https://github.com/saxenauts/persona)

**PersonaMem-v2**
- arXiv 2512.06688, 7 Dec 2025: "PersonaMem-v2: Towards Personalized Intelligence via Learning Implicit User Personas and Agentic Memory". It has 1,000 simulated users and 300+ scenarios, with mostly implicit preferences.
  - Frontier LLMs: 37–48%.
  - RFT Qwen3-4B: about 53%, beating GPT-5.
  - Agentic memory with about 2k-token memory per user: about 55%, using 16× fewer input tokens.

  — [arXiv 2512.06688](https://arxiv.org/abs/2512.06688v1)
- EverMemOS (arXiv 2601.02163) runs a PersonaMem-v2 profile study alongside LoCoMo and LongMemEval. — [EverMemOS](https://arxiv.org/pdf/2601.02163); [context_poison registry](https://github.com/Nyamtulla/context_poison)

### Inferences
- PersonaMem is multiple choice, so random chance is 25%. A memory system at about 60% is not directly comparable to the paper's 128k/1M long-context numbers, because vendors mostly use the 32k split and add their own answer models.
- The suggest_new_ideas and generalizing types are the hard tail. Vendors with high overall scores still sit around 39% on suggest_new_ideas (memind).

### Gaps
- Exact per-type and per-size numbers (32k and 1M panels, Figure 9) from the PersonaMem paper could not be read.
- Whether MemOS's eval used 32k or 128k could not be confirmed. The "Memobase 73.3% / GPT-4o-mini 78.0% (Table 4, clean single-domain)" rows in the CSIEE README come from a different paper/table and were not identified.

## Q4. Other benchmarks memory projects report in 2025–2026, and comparability caveats

### Takeaway
Besides BEAM and PersonaMem, the standard set is LoCoMo and LongMemEval-S (both now near saturation at 90%+ in vendor claims). Newer additions are HaluMem (operation-level hallucination), MemoryAgentBench (ICLR 2026) and PersonaMem-v2. Cross-vendor comparisons are weak because readers, judges, subsets and scoring differ.

### Cited Findings
- Vendor headline numbers:
  - Hindsight: LongMemEval-S 94.6%, LoCoMo 92%, BEAM-10M 64.1% (Oct 2026).
  - Mem0: LoCoMo 92.5, LongMemEval 94.4 (Apr 2026).
  - Zep: LoCoMo 94.7 (claimed) and LongMemEval 90.2.
  - EverMemOS: LoCoMo 92.3, LongMemEval-S 82.

  All are self-reported. — [Hindsight memlake](https://github.com/vectorize-io/hindsight/blob/main/hindsight-docs/blog/2026-10-13-memlake.md); [Mem0 blog](https://mem0.ai/blog/mem0-the-token-efficient-memory-algorithm); [search summaries](https://www.opensourceaireview.com/blog/which-ai-memory-layer-has-the-best-published-benchmarks-in-2026)
- LoCoMo answer-key audit: a third-party audit reports 6.4% of the LoCoMo answer key is wrong, and that a gpt-4o-mini judge accepts up to 62.8% of deliberately wrong answers. — [Penfield Labs on dev.to via ItshMoh notes](https://dev.to/penfieldlabs/we-audited-locomo-64-of-the-answer-key-is-wrong-and-the-judge-accepts-up-to-63-of-intentionally-33lg)
- HaluMem (arXiv 2511.03506, Nov 2025):
  - Evaluates extraction, updating and QA separately for Mem0, Mem0-Graph, Memobase, MemOS, Supermemory and Zep.
  - Extraction recall stays below 60% and accuracy below 62%, and all systems are worse on the long variant.
  - MemOS has the best recall on the medium split (about 74%, weighted about 85%).
  - Zep's extraction metrics could not be computed because Zep has no dialogue-memory API.

  — [HaluMem arXiv](https://arxiv.org/pdf/2511.03506); [HF papers](https://huggingface.co/papers/2511.03506)
- MemoryAgentBench (arXiv 2507.05257, ICLR 2026) has four competencies: accurate retrieval, test-time learning, long-range understanding, and conflict resolution (v1), later renamed selective forgetting. No current method masters all four. Per-competency numbers were not retrieved. — [arXiv](https://arxiv.org/html/2507.05257v3); [ICLR](https://iclr.cc/virtual/2026/poster/10010781)
- MemOS paper: on LoCoMo with GPT-4o-mini, MemOS 75.8 vs Mem0 64.6, Zep 59.2, Memobase 72.0 (vendor-run tables). — [ItshMoh notes citing arXiv 2507.03724](https://github.com/ItshMoh/infinity/blob/main/research/02-academic-landscape.md)
- Comparability caveats, with sources:
  - BEAM: avg nugget score vs pass rate; clamped vs raw partial credit.
  - BEAM: the 10M tier has only about 200 questions (±6.9 pts by Mem0's estimate).
  - BEAM: reader and judge differ by vendor (Maverick, gpt-5, DeepSeek, Gemini 3).
  - PersonaMem: 4-option MCQ (25% chance floor) vs generative scoring.
  - PersonaMem: "precision" vs "accuracy" naming differs.
  - PersonaMem: the MemOS self-run (61.2) vs the BMAM re-run (34.0).

  — [Mem0 how-to-read](https://mem0.ai/library/agent-memory/how-to-read-memory-benchmarks); [Scriptorium](https://github.com/Qi202/Scriptorium/blob/main/docs/experiments/protocols/unified_evaluation_protocol.md); [BMAM](https://arxiv.org/pdf/2601.20465); [mnemosyne-oss](https://github.com/mnemosyne-oss/mnemosyne/blob/main/docs/beam-benchmark.md)

### Inferences
- For a release claim, the most credible positioning is per-tier BEAM numbers stated with the reader and judge, reported against both the paper's LIGHT row (0.358/0.359/0.336/0.266) and vendor rows. Pair it with PersonaMem 32k per-type numbers against the MemOS table, while noting the BMAM re-run discrepancy.

### Gaps
- MemBench results and MemoryAgentBench per-system numbers were not retrieved.
- No independent, single-harness BEAM leaderboard covering all vendors was found. Vectorize's AMB is the closest, but it is vendor-run.
