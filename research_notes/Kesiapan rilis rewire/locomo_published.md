# Published LoCoMo results for agent-memory systems, and the disputes over comparing them

Research date: 2026-10-20. Method note: arxiv.org, dev.to, getzep.com, letta.com and memmachine.ai could not be fetched from this sandbox (DNS/proxy blocks). github.com pages could be fetched. Numbers attributed to arXiv papers or vendor blogs are taken from search-result extracts of those pages. Those extracts are marked "(search extract)" and should be spot-checked before a number is quoted publicly. Anything fetched directly from GitHub is marked "(fetched)".

Default protocol unless a row says otherwise: LoCoMo categories 1–4 only (1,540 questions: single-hop n=841, multi-hop n=282, temporal n=321, open-domain n=96). Category 5 (adversarial, 446 questions) is excluded. The metric is binary LLM-judge accuracy "J" in percent.

---

## Q1. Which LoCoMo J scores have been published for each system, and with which answer and judge models?

### Takeaway
There are three distinct groups of numbers, and they should never be put on one axis:
- **(a) The 2025 "Mem0-paper era" numbers** (gpt-4o-mini answerer and judge, Mem0's lenient judge prompt). Here full-context scores ~73, Mem0 ~67–68, Zep 66 to 75 (disputed), and Memobase ~76.
- **(b) The 2025–26 vendor self-reports at 85–94.7.** Examples: MemOS, EverMemOS, MemMachine, Backboard, Hindsight with Gemini-3, Mem0's new algorithm at 91.6/92.5, and Zep at 94.7 with GPT-5.4. These use stronger answer models and the vendors' own judges and prompts.
- **(c) Third-party reproductions in one harness** (OmniMemEval snapshot of Oct 2026: gpt-4.1-mini answerer, gpt-4o-mini judge). Here the spread is wider: MemOS 88.8, Cognee 83.5, Hindsight 82.0, Mem0 77.7, Letta 77.1, Supermemory 73.5, Zep 63.8, Backboard 22.4.

### Cited Findings

**Master table.** VSR = vendor self-reported. 3P = third party, either a competitor or an independent reproducer.

| System / config | Overall J | Single-hop | Multi-hop | Open-dom | Temporal | Answer model | Judge | Who ran it | Date | Source |
|---|---|---|---|---|---|---|---|---|---|---|
| Full-context (whole conversation in prompt) | 72.90 | – | – | – | – | gpt-4o-mini | gpt-4o-mini, Mem0 prompt | Mem0 (competitor baseline) | Apr 2025 | [Mem0 paper](https://arxiv.org/abs/2504.19413) (search extract) |
| Mem0 (base) | 66.88 | 67.13 | 51.15 | 72.93 | 55.51 | gpt-4o-mini | gpt-4o-mini | VSR | Apr 2025 | [Memobase README, copying Mem0 paper](https://github.com/memodb-io/memobase/blob/main/docs/experiments/locomo-benchmark/README.md) (fetched) |
| Mem0-graph (Mem0g) | 68.44 | 65.71 | 47.19 | 75.71 | 58.13 | gpt-4o-mini | gpt-4o-mini | VSR | Apr 2025 | same (fetched) |
| Zep (as configured by Mem0) | 65.99 ±0.16 | 61.70 | 41.35 | 76.60 | 49.31 | gpt-4o-mini | gpt-4o-mini | 3P (competitor) | Apr 2025 | same (fetched) |
| LangMem | 58.10 ±0.21 | 62.23 | 47.92 | 71.12 | 23.43 | gpt-4o-mini | gpt-4o-mini | 3P (Mem0) | Apr 2025 | same (fetched) |
| OpenAI ChatGPT memory | 52.90 | 63.79 | 42.92 | 62.29 | 21.71 | gpt-4o-mini | gpt-4o-mini | 3P (Mem0) | Apr 2025 | same (fetched) |
| A-Mem | 48.38 ±0.15 | – | – | – | – | gpt-4o-mini | gpt-4o-mini | 3P (Mem0) | Apr 2025 | [Mem0 paper](https://arxiv.org/pdf/2504.19413) (search extract) |
| Zep (Zep's original claim) | ~84 | – | – | – | – | gpt-4o-mini | Mem0 judge | VSR | early May 2025 | [zep-papers issue #5](https://github.com/getzep/zep-papers/issues/5) (fetched) |
| Zep (Mem0's re-score of Zep's own run) | 58.44 ±0.20 | – | – | – | – | gpt-4o-mini | gpt-4o-mini | 3P (competitor) | 8 May 2025 | [issue #5](https://github.com/getzep/zep-papers/issues/5) (fetched) |
| Zep (Zep's corrected figure, 10 runs) | 75.14 ±0.17 | 74.11 | 66.04 | 67.71 | 79.79 | gpt-4o-mini | gpt-4o-mini | VSR | 12 May 2025 | [issue #5](https://github.com/getzep/zep-papers/issues/5); per-category figures from the [Memobase README](https://github.com/memodb-io/memobase/blob/main/docs/experiments/locomo-benchmark/README.md) (fetched) |
| Memobase v0.0.32 | 70.91 | 63.83 | 52.08 | 71.82 | 80.37 | not stated (assumed gpt-4o-mini, as in the Mem0 harness) | README says LLM judge "e.g. gpt-4o" | VSR | ~May 2025 | [Memobase README](https://github.com/memodb-io/memobase/blob/main/docs/experiments/locomo-benchmark/README.md) (fetched) |
| Memobase v0.0.37 | 75.78 | 70.92 | 46.88 | 77.17 | 85.05 | same | same | VSR | ~Jul 2025 | same (fetched) |
| Letta filesystem agent (conversation stored as files; agent greps and searches them) | 74.0 | – | – | – | – | gpt-4o-mini | gpt-4o-mini (cat 5 dropped) | VSR | Aug 2025 | [Letta blog](https://www.letta.com/blog/benchmarking-ai-agent-memory/) (search extract) |
| MIRIX | ~84–87.3 over 3 runs (85.38 often cited) | – | – | – | – | gpt-4.1-mini | gpt-4.1, per the Mnemon paper's table | VSR | Jul 2025 | [MIRIX paper](https://arxiv.org/pdf/2507.07957); [Mnemon table](https://arxiv.org/html/2609.36059v1) (search extract) |
| MemOS (own paper) | ranked 1st in all categories (Table 3 values not retrieved) | | | | | gpt-4o-mini backbone for all methods | GPT-4o-mini plus two auxiliary judges, averaged | VSR | Jul 2025 onward | [MemOS paper](https://arxiv.org/pdf/2507.03724) (search extract) |
| MemMachine | 84.87 (Sep 2025 blog); 91.69 best later figure | – | – | – | – | not retrieved | LLM judge | VSR | Sep 2025 / 2026 | [MemMachine blog](https://memmachine.ai/blog/2025/09/memmachine-reaches-new-heights-on-locomo/) (search extract) |
| Hindsight (OSS-20B answerer) | 83.18 | 74.11 | 64.58 | 90.96 | 76.32 | gpt-oss-20b | GPT-OSS-120B (per paper) | VSR | Dec 2025 | [hindsight-benchmarks](https://github.com/vectorize-io/hindsight-benchmarks) (fetched); [paper](https://arxiv.org/pdf/2512.12818) |
| Hindsight (OSS-120B) | 85.67 | 76.79 | 62.50 | 93.68 | 79.44 | gpt-oss-120b | GPT-OSS-120B | VSR | Dec 2025 | same |
| Hindsight (Gemini-3) | 89.61 | 86.17 | 70.83 | 95.12 | 83.80 | Gemini-3 | GPT-OSS-120B | VSR | Dec 2025 | same |
| Backboard | 90.0 / 90.1 | 89.36 | 75.00 | 91.20 | 91.90 | not retrieved | GPT-4.1 | VSR | 2025 | [Backboard blog](https://backboard.io/blog/best-ai-memory-score-in-the-world) (search extract); [hindsight-benchmarks table](https://github.com/vectorize-io/hindsight-benchmarks) (fetched) |
| EverMemOS / EverOS | 92.32 (README); 93.05 elsewhere | 95.96 | 91.37 | – | – | gpt-4.1-mini | GPT-4o-mini plus 2 auxiliary judges | VSR | Jan 2026 | [EverMemOS paper](https://arxiv.org/pdf/2601.02163); [locomo-audit](https://github.com/dial481/locomo-audit) (fetched) |
| EverMemOS, third-party reproduction | 38.38 (other attempts reached 38–52) | | | | | | | 3P | 2026 | [locomo-audit](https://github.com/dial481/locomo-audit) (fetched); [Benchmark Theatre essay](https://essays.bloo-mind.ai/posts/2026-05-20-mem-eval/) (search extract) |
| Full-context with gpt-4.1-mini and a CoT answer prompt | 92.62 (the EverMemOS paper's full-context baseline: 91.21) | | | | | gpt-4.1-mini | EverMemOS judge | 3P (Penfield audit) | ~Feb–Apr 2026 | [locomo-audit](https://github.com/dial481/locomo-audit) (fetched) |
| Mem0 "token-efficient" algorithm v3 (managed platform) | 91.6, then 92.5 | – | – | – | – | not retrieved | not retrieved | VSR | Apr–May 2026 | [Mem0 blog](https://mem0.ai/blog/mem0-the-token-efficient-memory-algorithm); [migration docs](https://docs.mem0.ai/migration/platform-v2-to-v3) (search extract) |
| Mem0 previous algorithm (per Mem0 docs) | 71.4 | | | | | | | VSR | 2026 | [Mem0 docs](https://docs.mem0.ai/migration/platform-v2-to-v3) (search extract) |
| Zep / Graphiti, 2026 | 94.7 (1,459 of 1,540) | – | – | – | – | GPT-5.4 | GPT-5.4 | VSR | 2026 | [Graphiti page](https://www.getzep.com/platform/graphiti/); [atlan](https://atlan.com/know/zep-vs-mem0/) (search extract) |
| ByteRover 2.0 | 92.2 | | | | | Gemini 3 Pro ("justifier") | Gemini 3 Flash; Hindsight's prompt | VSR | 2026 | [ByteRover blog](https://www.byterover.dev/blog/benchmark-ai-agent-memory) (search extract) |
| T-Mem | 80.26 (+3.25 over HyperMem) | | | | | gpt-4o-mini | gpt-4o-mini | VSR (academic) | Jun 2026 | [T-Mem](https://arxiv.org/pdf/2606.15405) (search extract) |
| T-Mem's same-backbone reruns: MIRIX / Memobase / MemOS | 64.33 / 72.01 / 75.80 | | | | | gpt-4o-mini | gpt-4o-mini | 3P (academic) | Jun 2026 | [T-Mem](https://sherlockwz.github.io/T-Mem/) (search extract) |
| User-as-Code, vs full-context upper bound | 78.8 vs 79.8 (reimplemented SOTA systems: 55.5–72.7) | | | | | same backbone for all | Gemini 3 Flash | 3P (academic) | Jun 2026 | [arXiv 2606.16707](https://arxiv.org/pdf/2606.16707) (search extract) |
| MindMemOS paper's runs: MemOS / Zep / EverOS / Mem0 | 80.76 / 85.22 / 93.05 / 64.20 | | | | | own setup | own setup | 3P (academic, competitor) | 2026 | [MindMemOS](https://arxiv.org/pdf/2608.12428) (search extract) |

**OmniMemEval reproduced snapshot.** Dated 2026-10-08, republished in the Tenconic LEADERBOARD. Answerer gpt-4.1-mini-2025-04-14, judge gpt-4o-mini-2024-07-18, categories 1–4, n=1,540, binary judge. Tenconic says this is "not a synchronized controlled rerun". Source: [Tenconic LEADERBOARD.md](https://github.com/tenconic-ai/tenconic-memory-evaluation/blob/main/LEADERBOARD.md) (fetched).

| Rank | System | Single-hop | Multi-hop | Temporal | Open-dom | Overall |
|---|---|---|---|---|---|---|
| 1 | MemOS | 92.51 | 88.65 | 85.05 | 69.79 | 88.83 |
| 2 | Tenconic (self-run) | 94.65 | 77.66 | 84.42 | 69.79 | 87.86 |
| 3 | Cognee | 87.99 | 78.84 | 81.83 | 63.19 | 83.48 |
| 4 | EverOS | 86.80 | 77.78 | 84.11 | 57.29 | 82.75 |
| 5 | Hindsight | 88.98 | 78.84 | 73.52 | 58.33 | 81.99 |
| 6 | Mem0 | 81.09 | 76.12 | 77.15 | 54.17 | 77.68 |
| 7 | Letta | 87.99 | 76.24 | 53.48 | 63.54 | 77.12 |
| 8 | MemMachine | 83.47 | 53.19 | 71.96 | 57.29 | 73.90 |
| 9 | mem9 | 79.27 | 62.88 | 73.62 | 55.90 | 73.64 |
| 10 | Supermemory | 75.39 | 77.07 | 67.60 | 66.67 | 73.53 |
| 11 | MemoryLake | 70.87 | 75.30 | 79.75 | 54.17 | 72.49 |
| 12 | Viking | 78.04 | 73.29 | 48.81 | 50.00 | 69.33 |
| 13 | Zep | 65.36 | 68.79 | 55.56 | 63.54 | 63.83 |
| 14 | Memori | 47.32 | 44.09 | 22.53 | 43.75 | 41.34 |
| 15 | Backboard.io | 25.09 | 22.34 | 13.40 | 29.17 | 22.40 |

**Original LoCoMo paper.** Maharana et al., ACL 2024. It used token-level F1, not J. The conversations average about 300 turns and 9K tokens over up to 35 sessions. The often-cited human F1 is 87.9 and the best model score is GPT-4 at 32.1 F1, but both figures come from an aggregator summary, not the paper's table. The paper says long-context and RAG methods lag humans "by 56%" — [Maharana et al.](https://arxiv.org/pdf/2402.17753); [Emergent Mind summary](https://www.emergentmind.com/topics/locomo-1b7af604-e910-443f-a64e-87a74b0b1021) (search extract).

**Latency and token budgets:**
- From the Mem0 paper: Mem0 p95 total latency is 1.44 s against 17.12 s for full-context. Full-context uses about 26K tokens per query. Mem0 keeps about 7K memory tokens per conversation, and Mem0g about 14K — [Mem0 paper](https://arxiv.org/pdf/2504.19413) (search extract).
- Mem0 v3 claims under 7K tokens per retrieval call (6.8–7.0K at top-200) versus 25K+ for full-context — [Mem0 docs](https://docs.mem0.ai/migration/platform-v2-to-v3) (search extract).
- Zep 2026 claims 104 ms p50 / 162 ms p95 retrieval latency, with 155 ms p95 on the Graphiti page — [atlan](https://atlan.com/know/zep-vs-mem0/) (search extract).
- EverMemOS's README claims 2,298 average tokens per question. Its paper's Table 8 shows 6,669 (gpt-4.1-mini) and 6,045 (gpt-4o-mini). That makes the real reduction against full-context 67%, not the 89% claimed — [locomo-audit](https://github.com/dial481/locomo-audit) (fetched).

### Inferences
- Within one harness, full-context is close to the ceiling. Examples: 72.9 against Mem0's 66.9–68.4 with gpt-4o-mini, 79.8 in User-as-Code, and 92.6 with gpt-4.1-mini CoT. Most of the jump from ~67 (2025) to ~90 (2026) appears to come from moving the answer model up (gpt-4o-mini → gpt-4.1-mini, Gemini-3, GPT-5.4) rather than from memory architecture.
- The big rank inversions between self-reports and the OmniMemEval reruns are the strongest evidence that vendor numbers aren't comparable. Examples: Zep 94.7 self-reported vs 63.8 reproduced; Backboard 90.1 vs 22.4; EverMemOS 92–93 vs 38–83.
- Some open-domain columns are suspicious. Mem0-paper-era open-domain figures are 72–77, while the OmniMemEval reruns show 50–70. This may reflect the known confusion over which raw category id is "open-domain" versus "single-hop" in different harnesses (see Gaps).

### Gaps
- I did not retrieve the exact per-category values in MemOS Table 3, any LoCoMo J for Cognee from Cognee itself (one source says Cognee publishes none), any Supermemory self-reported LoCoMo J (one 2026 roundup "estimated ~70%"), or the RAG baseline rows of the Mem0 paper.
- The Letta 74.0 and the Mem0-paper MemGPT/Letta numbers were not fetched first-hand.
- I could not confirm the "Zep 80.32 at 189 ms" figure found in one search snippet.
- The answer and judge models behind the Backboard and Mem0 v3 claims were not retrieved.

---

## Q2. What happened in the Zep vs Mem0 dispute, and in the other disputes?

### Takeaway
The dispute started when Mem0's April 2025 paper scored Zep at 65.99. Zep answered with a blog post claiming ~84% and a 24% lead. Mem0's CTO showed that Zep's arithmetic counted category-5 correct answers in the numerator but not in the denominator, and re-scored Zep at 58.44. Zep admitted "an error in our calculation", re-reported 75.14 ±0.17 (a 10% lead), and kept its critique of how Mem0 configured Zep. The issue was closed without agreement.

### Cited Findings
- Deshraj, Mem0's co-founder and CTO, opened getzep/zep-papers issue #5 on **8 May 2025**:
  - He argued that Zep's 84% included category-5 correct answers in the numerator while the denominator excluded category 5. That inflated the score by about 25.56 points.
  - He re-scored Zep at **58.44% ±0.20** (10 runs).
  - He also criticized Zep's modified system prompt, its changed retrieval template, its role assignment, its timestamp handling, and its single-run reporting.
  - Source: [issue #5](https://github.com/getzep/zep-papers/issues/5) (fetched).
- Zep's replies on the same issue:
  - **12 May 2025:** Zep thanked Mem0 for "pointing out the error in our calculation" and re-reported **75.14% ±0.17** over 10 runs. It said it "stand[s] by our critique".
  - Zep argued that its timestamps reflect when events occurred, that the `created_at` field had existed since May 2024, that Mem0 ran two Zep searches serially (penalizing latency), and that Mem0 should share its data and evaluate on LongMemEval instead.
  - **19 May 2025:** Daniel Chalef closed the issue for inactivity, with no agreement reached.
  - Source: [issue #5](https://github.com/getzep/zep-papers/issues/5) (fetched).
- Zep's blog post "Lies, Damn Lies, & Statistics: Is Mem0 Really SOTA in Agent Memory?" made these claims:
  - Mem0 misconfigured Zep: both speakers were assigned the user role, timestamps were put in the message text instead of `created_at`, and searches ran sequentially.
  - Category 5 "unusable due to missing ground truth".
  - LoCoMo conversations fit in a single context window.
  - Sources: [Zep blog](https://www.getzep.com/blog/lies-damn-lies-statistics-is-mem0-really-sota-in-agent-memory/); [past.dev summary](https://www.past.dev/blog/mem0-vs-zep) (search extract).
  - The headline was quietly changed from a 24% lead to a 10% lead — [Benchmark Theatre essay, May 2026](https://essays.bloo-mind.ai/posts/2026-05-20-mem-eval/) (search extract).
- Memobase's README lists "Zep* (updated)" at 75.14 next to Mem0's own numbers. It cites the rerun to the Zep team via memobase issue #101 — [Memobase README](https://github.com/memodb-io/memobase/blob/main/docs/experiments/locomo-benchmark/README.md) (fetched).
- **Letta (Aug 2025).** In "Is a Filesystem All You Need?", a gpt-4o-mini agent that only greps and searches files holding the raw conversation scored 74.0. That beats Mem0g's 68.5 and the full-context 72.9. Tool rules constrained the agent because gpt-4o-mini is weak — [Letta blog](https://www.letta.com/blog/benchmarking-ai-agent-memory/) (search extract). The underlying point: LoCoMo rewards having verbatim access to the conversation, not a "memory layer".
- **EverMemOS (2026).** Its claim of 92.32/93.05 was challenged in three ways:
  - Its single-hop (95.96) and multi-hop (91.37) scores exceed the ceilings set by the wrong answer keys (95.72 and 90.07).
  - A third party reproduced it at 38.38.
  - Its token-saving claim is overstated (89% claimed vs 67% actual).
  - Source: [locomo-audit](https://github.com/dial481/locomo-audit) (fetched).
- **Hindsight.** Its own benchmark repo says the authors "do not consider this benchmark to be a reliable indicator of memory system quality". They cite flawed ground truth, ambiguous questions, conversations that fit in context, and no knowledge-update testing, and they recommend LongMemEval — [hindsight-benchmarks](https://github.com/vectorize-io/hindsight-benchmarks) (fetched).
- **Mem0 v3 (91.6–92.5) and Zep 2026 (94.7).** Both are self-reported with different models and setups, and not directly comparable — [Memobase-search roundup / ByteRover](https://www.byterover.dev/blog/benchmark-ai-agent-memory); [atlan](https://atlan.com/know/zep-vs-mem0/) (search extract).
- **The 2026 "Benchmark Theatre" essay** argues that several reported scores exceed the ~93.6% error-adjusted ceiling — [bloo-mind](https://essays.bloo-mind.ai/posts/2026-05-20-mem-eval/) (search extract).

### Inferences
- The specific Zep-vs-Mem0 numbers are unresolvable: 58.44, 65.99 and 75.14 are all "Zep on LoCoMo" under different ingestion configurations and arithmetic. The independent OmniMemEval rerun (63.83) falls between them.
- Every vendor in this dispute has acknowledged LoCoMo's weakness while still publishing on it: Zep, Hindsight, Letta, and Mem0 (which moved its headline to LongMemEval and BEAM).

### Gaps
- Mem0 never responded publicly on issue #5 to the 75.14 figure. I could not read the Zep blog text directly. I found no MemOS-vs-Memobase head-to-head dispute document.

---

## Q3. What is known to be wrong with the LoCoMo dataset itself?

### Takeaway
LoCoMo has about 6.4% wrong gold answers, which caps the attainable score near 93.6%. Category 5 can't really be scored. Conversations are short enough (~9K tokens on average in the paper; ~16–26K tokens of prompt in the Mem0 full-context setup) that full-context is roughly the upper bound. Category sizes are small, so many per-category differences are statistically meaningless. No official corrected release exists.

### Cited Findings
- **Penfield Labs audit** (repo dial481/locomo-audit, Feb–Apr 2026):
  - 156 issues found: 99 score-corrupting, 57 citation-only.
  - **99 of 1,540 questions (6.4%) have wrong gold answers.** Error types: invented facts, wrong date reasoning, misattributed speakers.
  - Theoretical ceiling: 93.57% overall; 95.72% single-hop; 90.07% multi-hop.
  - The audit builds on 29 errors first reported in snap-research/locomo#27.
  - Sources: [locomo-audit](https://github.com/dial481/locomo-audit) (fetched); [Penfield Substack](https://penfieldlabs.substack.com/p/we-audited-locomo-64-of-the-answer) (search extract).
- **Statistical power:** Category sizes range from 96 to 841. With Wilson 95% intervals, 56% of adjacent per-category comparisons are statistically indistinguishable — [locomo-audit](https://github.com/dial481/locomo-audit) (fetched).
- **Category 5:** 446 questions (22.5%). The original multiple-choice formatter references a field that is missing on 444 of the 446, and its keyword matching accepts only two phrases. That is why everyone drops the category — [locomo-audit](https://github.com/dial481/locomo-audit) (fetched).
- **Length:** The paper reports an average of 300 turns and 9K tokens per conversation, over up to 35 sessions — [Maharana et al.](https://arxiv.org/pdf/2402.17753) (search extract).
  - The MIRIX authors: "as the average length in this dataset is only 9k, Full-Context is almost the upper-bound" — [MIRIX](https://arxiv.org/pdf/2507.07957) (search extract).
  - Full-context in the Mem0 paper used about 26K tokens per query — [Mem0 paper](https://arxiv.org/pdf/2504.19413) (search extract).
- **Stricter judge:** A "LoCoMo-Refined" judge checks exact entities, dates, temporal state, constraints and unsupported additions. Its authors warn that scores from it should not be mixed with scores from the original judge — [MemChain arXiv 2607.24097](https://arxiv.org/pdf/2607.24097) (search extract).
- **Licensing:** LoCoMo's noncommercial license keeps some evaluators (Tenconic) from redistributing answers. That limits auditing — [Tenconic](https://github.com/tenconic-ai/tenconic-memory-evaluation) (fetched).

### Inferences
- With ~6.4% wrong keys and a lenient judge, scores above ~93% are suspect (they imply the judge is accepting wrong answers or the answer key is being matched by other means). Differences of 1–3 points overall, or 5+ points in the 96-question open-domain category, are within noise.

### Gaps
- I found no official errata release from snap-research. The audit's adjusted scores for five systems (in its `results-audit` folder) were not visible on the page.
- I could not confirm the exact "category id ↔ name" mislabel history across harnesses from a primary source.

---

## Q4. How lenient is the common judge, and do scores move when the judge model changes?

### Takeaway
Most numbers use the Mem0 judge prompt, adapted from MemGPT/Packer et al. It tells gpt-4o-mini to be generous: lenient on length, mark the answer correct if it "touches on the same topic", and accept equivalent date formats. Penfield measured this kind of judge accepting 62.8% of deliberately vague but on-topic wrong answers. It accepted 10.6% of specific-but-wrong answers.

### Cited Findings
- The Mem0 judge prompt adapts the prompt from Packer et al. (MemGPT). It instructs leniency on answer length and treats an answer as correct if it covers the same topic as the gold answer. It accepts different date formats or relative time references when they point to the same date — [Mem0 paper](https://arxiv.org/pdf/2504.19413) (search extract).
- Other projects copy this prompt nearly verbatim to make their numbers comparable to Mem0's: D-Mem, GASim, Cognis, and the FlowCraft Go package. Synthius-Mem notes that "Mem0 instructs its judge to be generous" — [D-Mem](https://arxiv.org/pdf/2603.18631); [FlowCraft](https://pkg.go.dev/github.com/GizClaw/flowcraft/eval); [Synthius-Mem](https://arxiv.org/pdf/2604.11563) (search extract).
- **Penfield leniency test.** It used the EverMemOS pipeline's judge prompts and the same judge as the original evaluation (gpt-4o-mini, per DEV summary). "Specific-but-wrong" answers were accepted **10.61%** of the time. "Vague-but-topical" answers were accepted **62.81%** of the time (6x leniency) — [locomo-audit](https://github.com/dial481/locomo-audit) (fetched).
  - Contradiction: one commentary cites a separate LoCoMo judge probe that found 0% false positives. Judge behaviour appears to depend on configuration — [Towards AI commentary](https://pub.towardsai.net/what-a-6-4-percent-error-rate-in-a-benchmarks-answer-key-taught-me-about-trusting-agent-memory-58415a8a8842?gi=fb200d7cbd62) (search extract).
- **Judge setups vary widely:**
  - MemOS and EverMemOS average GPT-4o-mini with two auxiliary judges — [EverMemOS](https://arxiv.org/pdf/2601.02163).
  - Hindsight uses GPT-OSS-120B — [Hindsight](https://arxiv.org/pdf/2512.12818).
  - Backboard uses GPT-4.1.
  - ByteRover uses Gemini 3 Flash.
  - Zep 2026 uses GPT-5.4 as both reader and judge.
  - The hindsight-benchmarks repo recommends an OpenAI judge key, which conflicts with the paper's GPT-OSS-120B judge (search extract).
- **Judge-swap study.** The "Cooperative Memory Paging" paper re-scored 100 probes with 4 judges (GPT-4o-mini, DeepSeek-v3.2, Claude Haiku 4.5, GLM-5). Method rankings held, but absolute scores were not reported in the snippet — [arXiv 2604.12376](https://arxiv.org/pdf/2604.12376) (search extract).
- In a non-LoCoMo setting, GPT-4o-mini as a judge shows verbosity bias and accepts vague or irrelevant explanations — [MSQA arXiv 2505.23982](https://arxiv.org/pdf/2505.23982) (search extract).

### Inferences
- Because the judge rewards on-topic vagueness, systems that return verbose or hedged answers can gain several points without better recall. Answer-length rules (e.g., the "5–6 word" answer constraint in the Mem0 harness — unverified, see Gaps) partly limit this.

### Gaps
- I could not fetch the Mem0 `evaluation/prompts.py` text verbatim (the URL returned 404, and the path may have moved).
- I found no published study that reports how much an identical prediction set's LoCoMo J changes when only the judge is swapped (e.g., gpt-4o-mini vs gpt-4.1 vs a stricter refined judge).

---

## Q5. What do best-in-class LoCoMo J scores look like in 2025–2026?

### Takeaway
Self-reported top scores are now 89–94.7, all with frontier answer models. That range presses against or exceeds the ~93.6% answer-key ceiling. Under a common gpt-4.1-mini answerer and gpt-4o-mini judge, the best reproduced figure is MemOS at 88.8, and most products land at 74–84. Plain full-context with gpt-4.1-mini CoT reaches ~92.6. With a gpt-4o-mini backbone, academic same-backbone comparisons put the best systems near 78–80 against a full-context ~80.

### Cited Findings
- Self-reported highs:
  - Zep 94.7 (GPT-5.4) — [atlan](https://atlan.com/know/zep-vs-mem0/)
  - EverMemOS 93.05 / 92.32 — [locomo-audit](https://github.com/dial481/locomo-audit)
  - Mem0 v3 92.5 — [Mem0 blog](https://mem0.ai/blog/mem0-the-token-efficient-memory-algorithm)
  - ByteRover 92.2 — [ByteRover](https://www.byterover.dev/blog/benchmark-ai-agent-memory)
  - MemMachine 91.69 (search extract)
  - Backboard 90.1 — [Backboard](https://backboard.io/blog/best-ai-memory-score-in-the-world)
  - Hindsight-Gemini-3 89.61 — [hindsight-benchmarks](https://github.com/vectorize-io/hindsight-benchmarks)
- Reproduced (OmniMemEval, Oct 2026): MemOS 88.83, Cognee 83.48, EverOS 82.75, Hindsight 81.99, Mem0 77.68, Letta 77.12 — [Tenconic LEADERBOARD](https://github.com/tenconic-ai/tenconic-memory-evaluation/blob/main/LEADERBOARD.md) (fetched).
- Full-context with gpt-4.1-mini CoT: 92.62 — [locomo-audit](https://github.com/dial481/locomo-audit).
- Academic same-backbone comparisons: User-as-Code 78.8 vs full-context 79.8 — [arXiv 2606.16707](https://arxiv.org/pdf/2606.16707). T-Mem 80.26 with gpt-4o-mini — [T-Mem](https://arxiv.org/pdf/2606.15405) (search extract).

### Inferences
- A credible new claim in late 2026 should report:
  1. categories 1–4, n=1,540;
  2. the answer and judge models and the judge prompt;
  3. a full-context baseline with the same answer model;
  4. multiple runs with error bars;
  5. ideally, scores against the Penfield-corrected key or a stricter refined judge.

  Without these, a number above ~90 is not informative.

### Gaps
- No neutral, official, continuously maintained LoCoMo leaderboard exists. OmniMemEval is the closest, and its methodology page was not retrieved directly; I only saw it through the Tenconic re-publication.
