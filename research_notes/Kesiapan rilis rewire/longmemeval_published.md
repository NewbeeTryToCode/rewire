# Published LongMemEval results: memory systems and long-context baselines (as of Oct 2026)

Note on method: arxiv.org, mem0.ai, mastra.ai, supermemory.ai, emergence.ai and getzep.com could not be fetched directly from this session (proxy refused them). Numbers below come from web-search extracts of those pages plus direct fetches of GitHub pages (xiaowu0162/LongMemEval, vectorize-io/hindsight-benchmarks, MemPalace BENCHMARKS.md). Where a figure came only from a search extract, it is cited to the original URL the extract named. Please spot-check the headline numbers against the PDFs before quoting them externally.

Legend: SR = vendor/author self-reported; 3P = reported by a third party (often a competitor, so not neutral). "Judge" is the official GPT-4o judge unless stated otherwise. All rows are LongMemEval_S (500 questions, ~115k tokens, ~40-50 sessions) unless marked.

## Master table (overall plus categories where published)

| System (answer model) | Var. | Overall | SS-user | SS-asst | SS-pref | Know-upd | Temporal | Multi-sess | Context/latency | Status, date, source |
|---|---|---|---|---|---|---|---|---|---|---|
| GPT-4o oracle (evidence sessions only) | oracle | 87.0% (92.4% with CoN) | | | | | | | evidence only | Paper, Oct 2024 [arXiv 2410.10813](https://arxiv.org/html/2410.10813) |
| GPT-4o full context | S | 60.6% (64.0% with CoN) | | | | | | | ~115k tok | Paper, Oct 2024 [arXiv](https://arxiv.org/html/2410.10813) |
| Llama 3.1 70B Instruct full context | S | 33.4% (28.6% CoN) vs. oracle 74.4% (84.8% CoN) | | | | | | | ~115k tok | Paper [arXiv](https://arxiv.org/html/2410.10813) |
| ChatGPT (GPT-4o), online memory | small subset | 57.7% vs. 91.8% offline reading | | | | | | | | Paper [arXiv](https://arxiv.org/html/2410.10813) via [search extract](https://www.emergentmind.com/topics/longmemeval-benchmark) |
| Coze (GPT-4o) | small subset | 33.0% | | | | | | | | Paper, same |
| ChatGPT (GPT-4o-mini) | small subset | 71.1% | | | | | | | | Paper, same |
| GPT-4o full context (Zep's run) | S | 60.2% | 81.4 | 94.6 | 20.0 | 78.2 | 45.1 | 44.3 | median 28.9 s | Zep paper, Jan 2025 [arXiv 2501.13956](https://arxiv.org/html/2501.13956v1); categories via [hindsight-benchmarks](https://github.com/vectorize-io/hindsight-benchmarks) |
| GPT-4o-mini full context (Zep's run) | S | 55.4% | | | | | | | median 31.3 s | Zep paper, Jan 2025 [arXiv](https://arxiv.org/html/2501.13956v1) |
| Zep/Graphiti (GPT-4o) | S | 71.2% | 92.9 | 80.4 | 56.7 | 83.3 | 62.4 | 57.9 | median 2.58 s (IQR 0.68), ~1.6k ctx tok | SR, Jan 2025 [arXiv](https://arxiv.org/html/2501.13956v1), [blog](https://blog.getzep.com/state-of-the-art-agent-memory/) |
| Zep/Graphiti (GPT-4o-mini) | S | 63.8% | | | | | | | median 3.20 s | SR, Jan 2025, same |
| Zep (newer, model not confirmed) | S | 90.2% | | | | | | | ~4,408 retrieved tok | SR, 2026, [getzep.com Hindsight comparison](https://www.getzep.com/vectorize-hindsight-alternative/) (search extract only) |
| EmergenceMem Internal (GPT-4o) | S | 86.0% | | | | | | | median 5.65 s | SR, Jun-Jul 2025 [blog](https://www.emergence.ai/blog/sota-on-longmemeval-with-rag), [X](https://x.com/emergence_ai/status/1938243480754786763) |
| EmergenceMem Simple / Simple Fast | S | 82.4% / 79.0% | | | | | | | 3.6 s retrieval (Fast) | SR, same |
| Supermemory (GPT-4o) | S | 81.6% | 97.1 | 96.4 | 70.0 | 88.5 | 76.7 | 71.4 | | SR (Supermemory research page) and repeated by Vectorize, 2025 [hindsight-benchmarks](https://github.com/vectorize-io/hindsight-benchmarks), [supermemory](https://supermemory.ai/research/longmembench/) |
| Supermemory (GPT-5) | S | 84.6% | 97.1 | 100 | 76.7 | 87.2 | 81.2 | 75.2 | | same |
| Supermemory (Gemini-3) | S | 85.2% | 98.6 | 98.2 | 70.0 | 89.7 | 82.0 | 76.7 | | same |
| Supermemory "ASMR" (experimental ensemble, Gemini 2.0 Flash / GPT-4o-mini) | S | ~99% | | | | | | | | SR, labelled experimental, not production; via [MemPalace BENCHMARKS.md](https://github.com/MemPalace/mempalace/blob/develop/benchmarks/BENCHMARKS.md) |
| Full context (GPT-OSS-20B) | S | 39.0% | 38.6 | 80.4 | 20.0 | 60.3 | 31.6 | 21.1 | | Hindsight paper, Dec 2025 [hindsight-benchmarks](https://github.com/vectorize-io/hindsight-benchmarks) |
| Hindsight (OSS-20B) | S | 83.6% | 95.7 | 94.6 | 66.7 | 84.6 | 79.7 | 79.7 | memory+judge run on OSS models, local MacBook + Postgres | SR, Dec 2025 [arXiv 2512.12818](https://arxiv.org/pdf/2512.12818), [repo](https://github.com/vectorize-io/hindsight-benchmarks); ACL 2026 demo [paper](https://aclanthology.org/2026.acl-demo.27.pdf) |
| Hindsight (OSS-120B) | S | 89.0% | 100 | 98.2 | 86.7 | 92.3 | 85.7 | 81.2 | | SR, same |
| Hindsight (Gemini-3 Pro answerer; memory + judge on GPT-OSS-120B) | S | 91.4% | 97.1 | 96.4 | 80.0 | 94.9 | 91.0 | 87.2 | 8,192-token retrieval budget per Zep | SR, Dec 2025; press Mar 2026 [PR Newswire](https://www.prnewswire.com/news-releases/vectorize-breaks-90-on-longmemeval-with-open-source-ai-agent-memory-system-302643146.html). Note non-GPT-4o judge |
| Mastra Observational Memory (actor gpt-5-mini; observer/reflector gemini-2.5-flash) | S | 94.87% | 95.7 | 94.6 | 100 | 96.2 | 95.5 | 87.2 | no retrieval; compressed observation log in context | SR, early 2026 [mastra.ai](https://mastra.ai/research/observational-memory) |
| Mastra OM (gpt-4o) | S | 84.23% | | | | | | | | SR, same |
| Mastra OM (gemini-3-pro-preview) | S | 93.27% | | | | | | | | SR, same (leaderboard on that page) |
| Mem0 new "token-efficient" algorithm (managed platform) | S (assumed) | 94.4% (blog) / 93.4% (docs) | | | | | | | <7,000 tok per retrieval | SR, Apr 2026 [blog](https://mem0.ai/blog/mem0-the-token-efficient-memory-algorithm), [docs](https://docs.mem0.ai/migration/platform-v2-to-v3); internal conflict 93.4 vs 94.4 |
| Mem0 (third-party runs) | S | 49.0% / 32.4% (OSS) | | | | | | | | 3P, 2026 [levh issue #340](https://github.com/ali-ulu/levh/issues/340), [dev.to](https://dev.to/everest_an/-i-benchmarked-ai-agent-memory-in-2026-and-the-numbers-tell-a-different-story-than-the-marketing-2ae4) |
| MemOS | S | 77.8% | | | | | | | | as cited in EverMemOS paper, Jan 2026 [ACL 2026](https://aclanthology.org/2026.acl-long.2125.pdf) |
| EverMemOS | S | 83.0% | | | | | | | | SR, Jan 2026 [alphaxiv 2601.02163](https://www.alphaxiv.org/abs/2601.02163), ACL 2026 |
| Memobase | S | 72.4% | | | | | | | | 3P table only; no official figure found ([search extract](https://www.emergentmind.com/topics/longmemeval-benchmark)) |
| TiMem (gpt-4o-mini) | S | 76.88% ± 0.30 | | | | | | | | SR, Jan 2026, repeated runs [arXiv 2601.02845](https://arxiv.org/pdf/2601.02845) |
| TiMem (gpt-4o) | S | 78.96% ± 0.26 | | | | | | | | SR, same |
| MemForest | S | 79.8% pass@1 | | | | | | | | SR, 2026 [arXiv 2605.23986](https://arxiv.org/pdf/2605.23986) |
| OMEGA | S | 95.4% | | | | | | | | SR [omegamax.co](https://omegamax.co/benchmarks) |
| Agent Zero Memory | S | 95.60% | | | | | | | | SR, Aug 2026 [arXiv 2608.29606](https://arxiv.org/pdf/2608.29606) |
| Scroll (Qwen backbone) | S | 94.8% | | | | | | | | SR, via [search extract](https://huggingface.co/papers?q=LongMemEval) |
| ByteRover | S | 92.8% | | | | | | | | SR, same |
| Oracle Agent Memory | S | 93.8% | | | | | | | | SR, same |
| FluctlightDB | S | 97.4% end-to-end QA (own reader/judge); 97.6% retrieval harness | | | | | | | | SR, different protocol, same |
| MemPalace hybrid + LLM rerank | S | 100% R@5 (tuned on test failures); 98.4% R@5 held-out 450; 96.6% R@5 raw | | | | | | | retrieval recall, not QA | SR [BENCHMARKS.md](https://github.com/MemPalace/mempalace/blob/develop/benchmarks/BENCHMARKS.md) |
| Letta, Backboard | — | none found | | | | | | | | — |

## What did the LongMemEval paper (Wu et al., ICLR 2025) report?

### Takeaway
The paper reports that long-context LLMs lose 30-60% accuracy reading the full ~115k-token LongMemEval_S history compared with the oracle evidence sessions. GPT-4o drops from 87.0% to 60.6%. Commercial assistants such as ChatGPT and Coze lose 37-64%. Its RAG recipe (round-level values, fact-augmented keys, time-aware queries, Chain-of-Note reading) is presented as a design guide, not a single leaderboard score.

### Cited Findings
- 500 questions in 6 types (single-session-user, single-session-assistant, single-session-preference, temporal-reasoning, knowledge-update, multi-session) plus abstention (`_abs`, 30 questions). S has ~115k tokens (~40 sessions, Llama-3 tokenizer). M has ~500 sessions per history. Oracle has only the evidence sessions. Scoring uses a GPT-4o QA script — [GitHub README](https://github.com/xiaowu0162/LongMemEval)
- Timeline: released Oct 2024, ICLR 2025 (Feb 2025). History sessions were cleaned in Sep 2025 to prevent interference with correctness (`longmemeval-cleaned` on HF). LongMemEval-V2 (agentic memory) followed in May 2026 — [GitHub README](https://github.com/xiaowu0162/LongMemEval)
- GPT-4o: oracle 0.870 vs. S 0.606 (-30.3%). With Chain-of-Note: 0.924 vs. 0.640. Llama 3.1 70B: 0.744 vs. 0.334 (-55.1%). With CoN: 0.848 vs. 0.286. Phi-3(-Medium) was also tested — [arXiv 2410.10813](https://arxiv.org/html/2410.10813)
- Commercial systems on a smaller, manually checked subset: offline GPT-4o reading 91.8%, ChatGPT (GPT-4o) 57.7%, Coze (GPT-4o) 33.0%, ChatGPT (GPT-4o-mini) 71.1%. The paper calls these 37% and 64% drops — [arXiv](https://arxiv.org/pdf/2410.10813), [emergentmind summary](https://www.emergentmind.com/topics/longmemeval-benchmark)
- RAG findings: decomposing sessions into rounds helps the GPT-4o reader. Fact-augmented key expansion gives +9.4% recall@k and +5.4% QA accuracy. Compressing values into facts alone loses information, though it helps multi-session — [arXiv](https://arxiv.org/html/2410.10813v2)
- The authors report that their GPT-4o judge prompt agrees with human judgement at 90% or more in all settings, checked on a sample — [arXiv](https://arxiv.org/pdf/2410.10813)

### Inferences
- The "oracle GPT-4o" reference point varies by source: 87.0% (plain), 92.4% (CoN), and 82.4% (Emergence's figure). Any "beats oracle" claim needs the exact baseline named.

### Gaps
- I could not retrieve the paper's exact LongMemEval_M numbers (full-context does not fit; RAG accuracy on M) or the per-category breakdown, because arxiv.org was unreachable. Pull them from Table 2/3 and the appendix of the PDF.

## What have vendors and academics published, and which are the top claims?

### Takeaway
Through 2025 the "state of the art" claim moved from Zep 71.2% (Jan 2025) to Emergence 86% (mid 2025), then Hindsight 91.4% (Dec 2025). In 2026 it reached about 94-96%: Mastra 94.87%, Mem0 94.4%, OMEGA 95.4% and Agent Zero 95.6%, plus retrieval-only claims of 97-100%. Nearly all are self-reported on LongMemEval_S with differing answer models and judges. Peer-reviewed academic systems (TiMem, MemOS, EverMemOS, MemForest) sit at about 77-83%. Almost nobody reports LongMemEval_M.

### Cited Findings
- See the master table above. Key sources: Zep [arXiv 2501.13956](https://arxiv.org/html/2501.13956v1); Emergence [blog](https://www.emergence.ai/blog/sota-on-longmemeval-with-rag); Supermemory and Hindsight [hindsight-benchmarks](https://github.com/vectorize-io/hindsight-benchmarks); Mastra [research](https://mastra.ai/research/observational-memory); Mem0 [blog](https://mem0.ai/blog/mem0-the-token-efficient-memory-algorithm); EverMemOS [ACL 2026](https://aclanthology.org/2026.acl-long.2125.pdf); TiMem [arXiv](https://arxiv.org/pdf/2601.02845); Agent Zero [arXiv](https://arxiv.org/pdf/2608.29606)
- Emergence's 86% came from an internal, unreleased configuration. Its public configs score 82.4% and 79%. Its blog compares against an "oracle GPT-4o 82.4%" — [Emergence blog](https://www.emergence.ai/blog/sota-on-longmemeval-with-rag)
- Mem0's managed-platform claim (93.4/94.4%) depends on proprietary optimisations that are absent from the OSS SDK — [preuve.ai](https://preuve.ai/blog/ai-memory-systems-statistics-2026), [Mem0 docs](https://docs.mem0.ai/migration/platform-v2-to-v3)
- No LongMemEval figure was found from Letta or Backboard. MemPalace's table lists Letta and Zep as "not published", which is wrong for Zep — [BENCHMARKS.md](https://github.com/MemPalace/mempalace/blob/develop/benchmarks/BENCHMARKS.md)

### Inferences
- The most credible numbers, ranked by transparency, are the ones with code, per-category tables and open models. That means Hindsight (OSS-20B 83.6%, OSS-120B 89.0%) and Zep 71.2% (paper with latency), followed by TiMem with mean ± std over repeats. Headline numbers above 93% are all single vendor runs without independent reproduction.
- On ~115k-token S histories, today's frontier long-context models probably score far above 2024's 60.6% for full-context GPT-4o. Few vendors re-run full-context baselines with modern models, so "+X over full context" claims against GPT-4o may be inflated.

### Gaps
- I found no published LongMemEval_M results from commercial vendors. Per-category numbers for Emergence, Mem0, EverMemOS and Agent Zero were not retrievable. Zep's newer 90.2% (model and judge) could not be verified at the source.

## How do results depend on answer model and judge?

### Takeaway
The answer model alone moves scores by 3-20 points for the same memory system. Mastra's results span 84.2% to 94.9% across answer models, Zep's 63.8% to 71.2%, Supermemory's 81.6% to 85.2%, and Hindsight's 83.6% to 91.4%. Judge and prompt choices can matter even more: Mem0 shows 94.4% vendor-run vs. 49.0% third-party.

### Cited Findings
- Zep: GPT-4o-mini 63.8% vs. GPT-4o 71.2%. Full context: GPT-4o-mini 55.4% vs. GPT-4o 60.2% — [Zep paper](https://arxiv.org/html/2501.13956v1)
- Supermemory: GPT-4o 81.6, GPT-5 84.6, Gemini-3 85.2. Hindsight: OSS-20B 83.6, OSS-120B 89.0, Gemini-3 91.4 — [hindsight-benchmarks](https://github.com/vectorize-io/hindsight-benchmarks)
- Mastra OM: gpt-4o 84.23%, gemini-3-pro 93.27%, gpt-5-mini 94.87% — [Mastra](https://mastra.ai/research/observational-memory)
- TiMem: gpt-4o-mini 76.88 ± 0.30 vs. gpt-4o 78.96 ± 0.26 — [TiMem](https://arxiv.org/pdf/2601.02845)
- Hindsight's Gemini-3 rows use Gemini-3 Pro only to generate answers. Its memory system and LLM judge run on GPT-OSS-120B, not the official GPT-4o judge — [search extract of Hindsight paper](https://arxiv.org/html/2512.12818v1)
- Mem0's LongMemEval spread is 45.4 points between vendor-run (94.4) and third-party (49.0) results. The issue thread attributes the gap to the answer and judge prompts, not the memory layer — [levh issue #340](https://github.com/ali-ulu/levh/issues/340)
- Category pattern: full-context GPT-4o is weakest on preference (20%), multi-session (44%) and temporal (45%). Memory systems gain most there. Single-session-assistant often slightly degrades with memory layers (Zep 94.6 → 80.4) — [hindsight-benchmarks](https://github.com/vectorize-io/hindsight-benchmarks), [Zep paper](https://arxiv.org/html/2501.13956v1)

### Inferences
- A fair comparison table should fix the answer model to GPT-4o (the most common shared reference: Zep 71.2, Supermemory 81.6, Mastra 84.2, Emergence 86) and use the official GPT-4o judge script. Mixing gpt-5-mini or Gemini-3 rows with GPT-4o rows is not apples to apples.

### Gaps
- No systematic study was found that swaps the judge (GPT-4o vs. others) on a fixed system output for LongMemEval.

## Criticisms and comparability issues

### Takeaway
There is no common leaderboard or standard protocol. Vendors differ in answer model, judge and judge prompt, top-k or token budget, dataset version (original vs. Sep-2025 cleaned) and metric (QA accuracy vs. retrieval recall). Most report single runs. Documented cases of test-set tuning (MemPalace) and prompt-level advantages (Mem0 dispute) mean that self-reported claims of 90% or more should be treated as upper bounds.

### Cited Findings
- No standard protocol exists for LoCoMo, LongMemEval or BEAM. Each paper picks its own judge model and prompt, reader, top-k and subset, so cross-vendor comparisons are not comparable unless those match — [mnemoverse field guide](https://mnemoverse.com/docs/research/evaluation/ai-memory-benchmarks-field-guide), [AgentOS audit](https://agentos.sh/blog/memory-benchmark-transparency-audit/)
- The ZenBrain paper notes that these are vendor self-reports, not entries on a common leaderboard, and are "not mutually commensurable" — [arXiv 2604.23878](https://arxiv.org/pdf/2604.23878)
- MemPalace's 100% R@5 came from fixes targeting 3 known failing questions ("teaching to the test"). Its clean held-out figure is 98.4% R@5. Critics called the 100% claim overfitting, and the raw 96.6% is essentially ChromaDB's default embeddings on verbatim chunks — [BENCHMARKS.md](https://github.com/MemPalace/mempalace/blob/develop/benchmarks/BENCHMARKS.md), [Vectorize critique](https://vectorize.io/articles/mempalace-benchmarks), [arXiv 2604.21284](https://arxiv.org/pdf/2604.21284)
- Critics of Mem0's answer layer found benchmark-specific equivalence rules, hidden reasoning instructions and judge-side bias checks — [levh issue #340](https://github.com/ali-ulu/levh/issues/340), [arXiv 2607.21503](https://arxiv.org/pdf/2607.21503)
- Retrieval recall (R@5 or R@20, e.g. Supermemory 97% R@20, MemPalace) is not QA accuracy — [MemPalace BENCHMARKS.md](https://github.com/MemPalace/mempalace/blob/develop/benchmarks/BENCHMARKS.md), [Mem0 benchmark overview](https://mem0.ai/blog/ai-memory-benchmarks-in-2026)
- Token budgets differ: Hindsight 91.4% used an 8,192-token retrieval budget, about 2× Zep's ~4,408 for 90.2% (a competitor's claim) — [getzep.com](https://www.getzep.com/vectorize-hindsight-alternative/)
- Zep's latency included network round trips from a Boston laptop to its hosted service, which the baselines did not incur — [Zep paper](https://arxiv.org/html/2501.13956v1)
- On S, the 115k-token history fits in modern context windows, so the "lift" largely measures compression rather than long-term retrieval. LongMemEval can also be gamed by verbatim retrieval. M (~1.5M tokens) is still far below production scale — [AgentOS audit](https://agentos.sh/blog/memory-benchmark-transparency-audit/), [mnemoverse](https://mnemoverse.com/docs/research/evaluation/ai-memory-benchmarks-field-guide)
- Repeats are rare: TiMem reports mean ± std, while most vendor posts give a single number — [TiMem](https://arxiv.org/pdf/2601.02845)
- The dataset was cleaned in Sep 2025, so pre- and post-cleaning runs may differ — [GitHub README](https://github.com/xiaowu0162/LongMemEval)

### Inferences
- When citing numbers, record: variant (S or M), dataset version (original or cleaned), answer model, judge model and prompt, retrieval budget, number of runs, and whether the result was vendor-run. Without those fields, scores above about 85% cannot be compared.
- Much of the criticism comes from competing vendors (Vectorize on MemPalace, Zep on Hindsight, a levh issue on Mem0), so it is not neutral either.

### Gaps
- No neutral, independently maintained LongMemEval leaderboard was found as of Oct 2026. omegamax.co and Mastra's own table are run by vendors.
- No published analysis of oracle-session leakage was found, i.e. whether systems use `answer_session_ids` or haystack metadata during ingestion.
