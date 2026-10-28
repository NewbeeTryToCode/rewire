# Published latency, token usage and ingest cost of agent memory systems and full-context baselines (as of 20 Oct 2026)

**Method note.** Direct page fetches were blocked in this session: arxiv.org, mem0.ai and getzep.com returned a proxy 403 / DNS failure, and the GitHub benchmark repos were not attached. So almost every figure below comes from web-search extracts of the primary page. Each one is marked "(search extract)", with the primary URL given. Figures that two or more independent extracts agreed on are marked "(consistent across extracts)". Every number is self-reported (SR) unless marked 3P (third party) or IND (independent harness). None of it was re-verified against the PDF, so check exact numbers against the paper before citing them in public.

**Legend.** SR = the vendor or author measured its own system. 3P = a third party measured it, often a competitor, so not neutral. IND = a common harness not run by any listed vendor. "Search" = retrieval/search call only. "Total" = end-to-end answer latency (retrieval plus answer LLM). "Ctx tok/query" = memory or context tokens sent to the answer model per question. "Footprint" = total tokens stored per conversation.

## Master table

| System | Benchmark | Search p50 / p95 | Total (E2E) p50 / p95 | Ctx tok/query | Footprint (tok per conv) | Ingest cost | Deployment / how measured | Status, date, source |
|---|---|---|---|---|---|---|---|---|
| Full-context (gpt-4o-mini) | LoCoMo | – | 9.870 s / 17.117 s | ~26,031 | n/a (raw ~26K) | none | OpenAI API | SR by Mem0 authors, Apr 2025 [Mem0 paper](https://arxiv.org/html/2504.19413v1) (search extract, consistent) |
| Mem0 (base) | LoCoMo | 0.148 s / 0.200 s | 0.708 s / 1.440 s | 1,764 | ~7K | 2 LLM steps per message pair (extract, then update via tool call: ADD/UPDATE/DELETE/NOOP) | Mem0 platform/OSS + gpt-4o-mini | SR, Apr 2025 [Mem0 paper](https://arxiv.org/pdf/2504.19413) (search extract, consistent) |
| Mem0g (graph) | LoCoMo | 0.476 s / 0.657 s | p50 not confirmed / 2.590 s | 3,616 | ~14K | extraction + graph update; graph built "in under a minute" worst case | same | SR, Apr 2025, same (search extract) |
| Zep (as run by Mem0) | LoCoMo | 0.513 s / 0.778 s | 1.292 s / 2.926 s | 3,911 | >600K (summary per node plus facts on edges, ~20x raw) | async graph building; searches "hours later" scored better | Zep Cloud | 3P (competitor), Apr 2025 [Mem0 paper](https://arxiv.org/pdf/2504.19413) (search extract) |
| Zep (Zep's rebuttal, concurrent search) | LoCoMo | p95 0.632 s | – | – | – | – | Zep Cloud | SR, May 2025 [Zep blog](https://blog.getzep.com/lies-damn-lies-statistics-is-mem0-really-sota-in-agent-memory/) (search extract) |
| LangMem | LoCoMo | 17.99 s / 59.82 s | 18.53 s / 60.40 s | 127 | – | – | local LangGraph store | 3P by Mem0, Apr 2025 [Mem0 paper](https://arxiv.org/html/2504.19413v1) (search extract) |
| A-Mem | LoCoMo | 0.668 s / 1.485 s | 1.410 s / 4.374 s | 2,520 | – | – | local | 3P by Mem0, same (search extract) |
| OpenAI memory (ChatGPT) | LoCoMo | n/a (memories extracted manually from playground) | 0.466 s / 0.889 s | 4,437 | – | – | ChatGPT | 3P by Mem0, same (search extract) |
| Mem0 (as run by MemOS) | LoCoMo | 1,297 ms / 1,416 ms | 4,906 ms / 5,962 ms | 1,171 (top-k 20) | – | – | Mem0 | 3P (competitor), Jul 2025 [MemOS paper](https://statics.memtensor.com.cn/files/MemOS_0707.pdf) (search extract) |
| MemOS-0630 | LoCoMo | 1,758 ms / 1,969 ms | 4,942 ms / 7,937 ms | 1,593 (top-k 20) | – | – | MemOS | SR, Jul 2025 [arXiv 2507.03724](https://arxiv.org/html/2507.03724v1) (search extract; table text garbled) |
| RAG rows in MemOS table (chunks 128–512, k 1–2) | LoCoMo | ~480–575 ms p50 | ~1.3–1.5 s p50; ~2.0–4.3 s p95 | 128–1,024 | – | embedding only | – | SR, same (search extract) |
| Mem0 v3 "token-efficient" algorithm | LoCoMo / LongMemEval / BEAM-1M / BEAM-10M | – | p50 0.88 s / 1.09 s / 1.00 s / 1.05 s | ~6,700–6,956 (under 7K at top-200) vs 25K+ full context | – | single-pass, ADD-only extraction (one LLM call, was two); extraction latency "roughly in half" | managed Mem0 Platform | SR, Apr–May 2026 [Mem0 blog](https://mem0.ai/blog/mem0-the-token-efficient-memory-algorithm), [docs](https://docs.mem0.ai/migration/platform-v2-to-v3) (search extract) |
| Zep (2026, gpt-5.4 reader) | LongMemEval_S | 104 ms / 162 ms | – | median 4,408 | – | – | Zep Cloud | SR, ~May 2026 [getzep.com/research](https://www.getzep.com/research/) (search extract, consistent) |
| Zep (2026) | LoCoMo (1,540 Q) | 87 ms / 155 ms | – | median 5,760 | – | – | Zep Cloud | SR, same (search extract) |
| Zep (paper, gpt-4o) | LongMemEval_S | – | median 2.58 s (IQR 0.68) vs full-context 28.9 s | ~1.6K vs 115K | – | – | hosted Zep, called from a Boston laptop (network included) | SR, Jan 2025 [arXiv 2501.13956](https://arxiv.org/html/2501.13956v1) (from earlier notes, fetched then) |
| Zep (paper, gpt-4o-mini) | LongMemEval_S | – | median 3.20 s vs 31.3 s | ~1.6K | – | – | same | SR, same |
| Zep (scale claim) | production | p95 ~161 ms across 10M graphs | – | – | – | – | Zep Cloud | SR, Jun 2026 [Zep](https://getzep.com/platform/context-graph-engine/) (search extract) |
| EmergenceMem Internal | LongMemEval_S | – | median 5.65 s | – | – | – | – | SR, Jun 2025 [Emergence blog](https://www.emergence.ai/blog/sota-on-longmemeval-with-rag) (earlier notes) |
| EmergenceMem Simple Fast | LongMemEval_S | 3.6 s retrieval | – | – | – | – | – | SR, same (earlier notes) |
| Hindsight (recall) | product | 50–100 ms without rerank; 200–500 ms with rerank; reflect 1–10 s | – | 8,192-token budget for the 91.4% LME run | – | retain 500–2,000 ms per batch, bottlenecked by LLM fact extraction | self-host or Vectorize cloud | SR [FAQ](https://hindsight.vectorize.io/faq), [performance docs](https://hindsight.vectorize.io/developer/performance) (search extract) |
| Hindsight Memlake | 3M-memory bank | 71 ms p50 / 90 ms p99, flat from 1 to 200 recalls/s; "<100 ms p99 at 100M memories" | – | – | – | – | object storage | SR, 13 Oct 2026 [blog](https://hindsight.vectorize.io/blog/2026/10/13/memlake) (search extract) |
| Hindsight retain (community run) | LoCoMo | – | – | – | – | full LoCoMo retain 772.7 s with 581 model calls (Qwen 27B local vLLM) | local vLLM | 3P user issue [hindsight-benchmarks #20](https://github.com/vectorize-io/hindsight-benchmarks/issues/20) (search extract) |
| Hindsight (Tenure test) | small own eval | 589.86 ms p50 (cross-encoder rerank) | – | – | – | – | Docker, CPU | 3P (competitor) [tenureai.dev](https://tenureai.dev/compare/hindsight/) (search extract) |
| Supermemory vs Mem0 (Lobu) | LongMemEval oracle-50 | 702 ms vs Mem0 753 ms (retrieval only) | – | – | – | – | hosted APIs | 3P [lobu.ai](https://lobu.ai/guides/memory-benchmarks/) (search extract) |
| Supermemory vs Mem0 (Lobu) | LoCoMo-50 | 532 ms vs 606 ms | – | – | – | – | hosted | 3P, same (accuracy 23.2% vs 41.5%) |
| Mastra Observational Memory | LongMemEval_S | no retrieval step | – | ~30K average context (observation log) | – | observer LLM call at 30K raw tokens; reflector at 40K observation tokens; append-only prefix is cacheable | gemini-2.5-flash observer/reflector | SR, early 2026 [Mastra blog](https://mastra.ai/blog/observational-memory); 30K average via [VentureBeat](https://venturebeat.com/ai/observational-memory-cuts-ai-agent-costs-10x-and-outscores-rag-on-long) (search extract) |
| EverMemOS | LoCoMo | – | median 2.46 s (lite) vs full context 1.78 s, Mem0 2.19 s | ~2.8K (main table); README 2,298; App. Table 8 6,669 (4.1-mini) / 6,045 (4o-mini) | – | "high-cost group" for indexing tokens (H-Mem) | – | SR tokens [ACL 2026](https://aclanthology.org/2026.acl-long.2125.pdf); latency 3P [User as Code arXiv 2606.16707](https://arxiv.org/html/2606.16707v1); audit [locomo-audit](https://github.com/dial481/locomo-audit) (search extract) |
| MemMachine | LoCoMo (gpt-4.1-mini) | – | – | total 4.20M input + 43K output tokens (memory mode); 8.57M (agent mode); Mem0 19.21M in same setup, so ~78% fewer | – | – | – | SR, Apr 2026 [arXiv 2604.04853](https://arxiv.org/pdf/2604.04853) (search extract) |
| MIRIX | LoCoMo | – | – | answers from retrieved memories only (conversations ~26K) | ScreenshotVQA: 15.89 MB vs 15.07 GB for SigLIP@50 RAG (-99.9%) | multi-agent (6 memory agents) | – | SR, Jul 2025 [arXiv 2507.07957](https://arxiv.org/html/2507.07957v1) (search extract) |
| LazyMem 4B / 32B | LongMemEval | – | mean E2E lower than prior query-time method | 213 (4B, 0.85 acc); 1,041 (32B, 0.93); Mem0 1,731 | – | lazy construction | – | SR, Jul 2026 [arXiv 2607.22690](https://arxiv.org/html/2607.22690v2) (search extract) |

### Independent or common-harness efficiency tables

| Harness | System | Add / ingest | Retrieve | Tokens | Source |
|---|---|---|---|---|---|
| RealMem (Table 5) | Graph Mem | 26.898 s per add | 0.188 s | 1,224 | IND-ish (academic), Jan 2026 [arXiv 2601.06966](https://arxiv.org/pdf/2601.06966) (search extract) |
| RealMem | Mem0 | 11.751 s | 1.261 s | 4,178 | same |
| RealMem | A-Mem | 8.663 s | 0.013 s | 500 | same |
| RealMem | MemoryOS | 14.755 s | 0.151 s | 9,201 | same |
| HaluMem-Medium (minutes: add / retrieve / total) | Mem0 | 2,768.14 | 41.66 | 2,809.8 | MemTensor (MemOS authors, so not neutral), Nov 2025 [arXiv 2511.03506](https://arxiv.org/pdf/2511.03506) (search extract) |
| HaluMem-Medium | Mem0-Graph | 2,840.07 | 54.65 | 2,894.72 | same |
| HaluMem-Medium | Memobase | 293.30 | 139.95 | 433.25 | same |
| HaluMem-Medium | MemOS | 1,028.84 | 20.52 | 1,049.37 | same |
| HaluMem-Medium | Supermemory | 273.21 | 95.53 | 368.74 | same |
| HaluMem-Medium | Zep | not reported | 53.34 | – | same |
| HaluMem-Long | Mem0 / Mem0-Graph / Memobase / MemOS / Supermemory / Zep | 691.62 / 870.32 / 239.29 / 1,524.39 / 1,672.53 / – | 39.15 / 62.42 / 136.19 / 20.96 / 137.02 / 50.22 | – | same |
| User as Code (LoCoMo, answer-time only; excludes memory build) | full context 1.78 s, Mem0 2.19 s, EverMemOS-lite 2.46 s (medians) | – | – | – | 3P, Jun 2026 [arXiv 2606.16707](https://arxiv.org/html/2606.16707v1) (search extract) |
| "Beyond the Context Window" (Mem0-based fact memory vs GPT-5-mini long context) | memory beats long context on cost after ~10 turns at 100K context; break-even falls from 13 turns (30K) to 9 turns (500K); assumes 90% prompt-cache discount from turn 2 | – | – | – | 3P, Mar 2026 [arXiv 2603.04814](https://arxiv.org/abs/2603.04814) (search extract) |

## What did the Mem0 paper (arXiv 2504.19413) report in its latency and token table?

### Takeaway
On LoCoMo with gpt-4o-mini, Mem0 reported search at 0.148 / 0.200 s (p50/p95), total at 0.708 / 1.440 s and 1,764 memory tokens per query. Full context was 9.870 / 17.117 s at about 26K tokens. Zep, LangMem, A-Mem and OpenAI were all run by Mem0, a competitor, and Zep disputes its own row.

### Cited Findings
- Table rows (p50/p95 in seconds; then memory/context tokens):
  - Mem0: search 0.148/0.200, total 0.708/1.440, 1,764 tokens.
  - Mem0g: search 0.476/0.657, total p95 2.590, 3,616 tokens.
  - Zep: search 0.513/0.778, total 1.292/2.926, 3,911 tokens.
  - LangMem: search 17.99/59.82, total 18.53/60.40, 127 tokens.
  - A-Mem: search 0.668/1.485, total 1.410/4.374, 2,520 tokens.
  - OpenAI: total 0.466/0.889, 4,437 tokens. There was no search step because memories were extracted manually from the playground.
  - Full context: total 9.870/17.117, ~26,031 tokens.

  J scores: full context 72.90, Mem0g 68.44, A-Mem 48.38. — [Mem0 paper HTML](https://arxiv.org/html/2504.19413v1), [alphaxiv](https://www.alphaxiv.org/abs/2504.19413), [digest](https://yiwei-chen.github.io/paper-digest/summaries/en/mem0.html) (search extracts, mutually consistent)
- The table column is labelled "chunk size / memory tokens", so for memory systems it is the tokens of memory sent per query, not the stored size — [Mem0 paper](https://arxiv.org/html/2504.19413v1) (search extract)
- Storage footprint, in the paper's text: Mem0 about 7K tokens per conversation, Mem0g about 14K, and Zep "in excess of 600k tokens". Mem0 attributes Zep's size to caching an abstractive summary at every node plus facts on edges, roughly 20x the ~26K raw conversation — [Mem0 paper PDF](https://arxiv.org/pdf/2504.19413) (search extract)
- Ingest:
  - Mem0 runs an extraction LLM call per new message pair. Its context is a conversation summary, the last m=10 messages and the new exchange.
  - An update phase follows. It retrieves the s most similar memories and makes an LLM tool call choosing ADD/UPDATE/DELETE/NOOP. Sources disagree on whether that call is per fact or per batch.
  - Mem0g graph construction completes "in under a minute" even in the worst case. Zep searches run immediately after ingest missed answers but did much better "hours later", which the authors read as async background LLM processing.
  — [Mem0 paper](https://arxiv.org/pdf/2504.19413) (search extract); [secondary summaries](https://www.emergentmind.com/topics/mem0) (search extract)
- Zep's rebuttal: run with concurrent searches, Zep's search p95 is 0.632 s, not 0.778 s, which makes it slightly faster than Mem0g's 0.657 s. Zep also says Mem0 misconfigured it: one user for both speakers, and timestamps put in message text instead of `created_at`. Zep's corrected LoCoMo score is 75.14 ± 0.17 — [Zep blog](https://blog.getzep.com/lies-damn-lies-statistics-is-mem0-really-sota-in-agent-memory/) (search extract)
- In the MemOS paper's own LoCoMo run, Mem0 was much slower than in Mem0's paper: search 1,297/1,416 ms and total 4,906/5,962 ms at top-k 20 with 1,171 tokens — [MemOS PDF](https://statics.memtensor.com.cn/files/MemOS_0707.pdf) (search extract)
- Mem0 v3 (managed platform, April 2026):
  - p50 end-to-end latency: 0.88 s (LoCoMo), 1.09 s (LongMemEval), 1.00 s (BEAM-1M), 1.05 s (BEAM-10M).
  - About 6,700–6,956 tokens per query, under 7K at top-200, against 25K+ for full context.
  - Extraction is now single-pass and ADD-only (one LLM call instead of two), which Mem0 says roughly halves extraction latency.
  - Open-source users should expect "directionally similar" but not identical numbers.
  — [Mem0 blog](https://mem0.ai/blog/mem0-the-token-efficient-memory-algorithm), [migration docs](https://docs.mem0.ai/migration/platform-v2-to-v3), [research page](https://mem0.ai/research-5) (search extract)

### Inferences
- Mem0's own 2025 latency for Mem0 (p50 total 0.708 s) is about 7x faster than MemOS's measurement of Mem0 (4.9 s). Retrieval and answer latency for the same system varies by about an order of magnitude depending on who runs it, top-k, region and answer model. Cross-paper latency comparisons are only indicative.
- The v3 algorithm traded about 4x more context tokens per query (1.8K to about 7K) for accuracy (LoCoMo 66.9 to 92.5). Its "token efficiency" is relative to full context, not to the 2025 Mem0.
- LangMem's 60 s p95 search reflects running LangMem's local store with LLM-mediated search in Mem0's harness. It is not representative of a tuned deployment, and no LangMem-authored latency was found.

### Gaps
- Mem0g's total p50 and the RAG best-configuration row could not be verified from an extract.
- Rows for LoCoMo-baseline, ReadAgent, MemoryBank and MemGPT in the latency table: I could not confirm whether the paper reports latency for them.
- I found no published Mem0 v3 search-only p95 or end-to-end p95.

## What latency and context tokens did Zep, Hindsight, Supermemory, Emergence, Mastra, MemOS and EverMemOS report?

### Takeaway
Retrieval-only latencies in 2026 cluster around 70–200 ms for the managed or optimized systems: Zep 87–104 ms p50, Hindsight Memlake 71 ms p50. End-to-end answer latencies are about 1–5 s: Mem0 v3 about 1 s, Zep 2025 2.58 s, EverMemOS 2.46 s, MemOS 4.9 s, Emergence 5.65 s. Context sent per query ranges from about 200 tokens (LazyMem) through 1.6–7K (most retrieval systems) to about 30K (Mastra OM). That compares with ~26K (LoCoMo) and ~115K (LongMemEval_S) for full context.

### Cited Findings
- **Zep, January 2025 paper, LongMemEval_S:**
  - Median latency 2.58 s (IQR 0.68) with gpt-4o, against 28.9 s for full context. With gpt-4o-mini it was 3.20 s against 31.3 s.
  - Context about 1.6K tokens, against 115K for full context.
  - Latency includes the network round trip from a Boston laptop to hosted Zep, which the full-context baseline did not incur.
  — [arXiv 2501.13956](https://arxiv.org/html/2501.13956v1) (fetched in earlier notes)
- **Zep, 2026 research page:**
  - LongMemEval: 90.2%, retrieval 104/162 ms p50/p95, median 4,408 tokens per question.
  - LoCoMo (1,540 questions): 94.7%, 87/155 ms, median 5,760 tokens.
  - Reader and judge were both gpt-5.4.
  — [getzep.com/research](https://www.getzep.com/research/) (search extract). Zep also claims 161 ms p95 across 10M graphs — [Zep](https://getzep.com/platform/context-graph-engine/) (search extract)
- A Mem0-authored comparison lists Zep at "80.32% @ 189 ms" on LoCoMo. That conflicts with Zep's own figures — [atlan / Mem0 pages via search](https://atlan.com/know/best-ai-agent-memory-frameworks-2026/) (search extract)
- **Hindsight:**
  - Recall takes 50–100 ms without rerank and 200–500 ms with rerank. Reflect takes 1–10 s — [FAQ](https://hindsight.vectorize.io/faq) (search extract)
  - The 91.4% LongMemEval result used an 8,192-token retrieval budget, about twice Zep's 4,408 (Zep's framing) — [Zep comparison page](https://www.getzep.com/vectorize-hindsight-alternative/) (search extract)
  - Memlake (13 Oct 2026): 71 ms p50 / 90 ms p99 on a 3M-memory bank, flat from 1 to 200 recalls/s — [blog](https://hindsight.vectorize.io/blog/2026/10/13/memlake) (search extract)
  - A third-party test measured 589.86 ms p50 with a CPU cross-encoder — [Tenure](https://tenureai.dev/compare/hindsight/) (search extract)
- **Supermemory:**
  - No self-published p95 was found. Its MemoryBench tool reports latency percentiles and "tokens sent to answering model" per provider — [docs](https://supermemory.ai/docs/memorybench/quickstart) (search extract)
  - Third-party retrieval-only latency: 702 ms against Mem0's 753 ms on LongMemEval oracle-50, and 532 ms against 606 ms on LoCoMo-50 — [lobu.ai](https://lobu.ai/guides/memory-benchmarks/) (search extract)
- **Emergence:** median 5.65 s for EmergenceMem Internal, and 3.6 s retrieval for Simple Fast — [Emergence blog](https://www.emergence.ai/blog/sota-on-longmemeval-with-rag) (earlier notes)
- **Mastra OM:** there is no retrieval call. The observer compresses at 30K raw tokens and the reflector at 40K observation tokens. The average context in the LongMemEval run was about 30K tokens. The append-only prefix makes it cacheable, and the "10x cost" headline rests on generic 4–10x prompt-cache discounts, not a measured bill — [Mastra blog](https://mastra.ai/blog/observational-memory), [VentureBeat](https://venturebeat.com/ai/observational-memory-cuts-ai-agent-costs-10x-and-outscores-rag-on-long) (search extract)
- **MemOS-0630, LoCoMo:** search 1,758/1,969 ms, total 4,942/7,937 ms, 1,593 tokens at top-k 20. The paper calls this "competitive latency", but its total p95 is the highest of the visible rows — [arXiv 2507.03724](https://arxiv.org/html/2507.03724v1) (search extract)
- **EverMemOS token figures:**
  - About 2.8K tokens per question in the main ACL table — [ACL 2026](https://aclanthology.org/2026.acl-long.2125.pdf) (search extract)
  - The README says 2,298, while Appendix Table 8 says 6,669 / 6,045. The audit concludes the real reduction against full context is 67%, not the 89% claimed — [locomo-audit](https://github.com/dial481/locomo-audit) (earlier notes, fetched)
- **EverMemOS latency (3P):** median 2.46 s against full context 1.78 s, measured with answer calls only — [User as Code](https://arxiv.org/html/2606.16707v1) (search extract)

### Inferences
- In User as Code, full context was faster end to end (1.78 s) than either memory system on LoCoMo. With modern fast long-context models and ~26K-token prompts, the "91% lower latency than full context" claim from the 2025 gpt-4o-mini era may no longer hold. LoCoMo is too short to show a latency advantage.
- Zep's numbers moved from 2.58 s end to end to 104 ms retrieval only. These are different metrics, not a 25x speedup.

### Gaps
- I found no published latency or token figures for Letta/MemGPT, Cognee, Honcho, Memobase (outside HaluMem), MIRIX (latency) or the OpenAI memory product (outside Mem0's run).
- Supermemory's self-reported p50/p95: none found.

## Ingest-side cost: LLM calls and time per message or session

### Takeaway
Ingest dominates cost and time for every LLM-extraction system. Every independent table finds that write latency far exceeds read latency: seconds per add and hours per dataset, against tens to hundreds of ms per read. Few vendors publish LLM calls per message. Mem0 is the exception: two calls in 2025, one in v3.

### Cited Findings
- **Mem0 paper:** an extraction LLM call and an update LLM tool call per message pair, as described above — [Mem0 paper](https://arxiv.org/pdf/2504.19413) (search extract). In v3 this became a single-pass ADD-only call, with "roughly half" the extraction latency — [Mem0 blog](https://mem0.ai/blog/mem0-the-token-efficient-memory-algorithm) (search extract)
- **Zep:** graph construction is asynchronous and uses multiple LLM calls (Mem0's observation that results improved hours later) — [Mem0 paper](https://arxiv.org/pdf/2504.19413) (search extract). HaluMem did not report Zep's add time — [HaluMem](https://arxiv.org/pdf/2511.03506) (search extract)
- **Hindsight:**
  - Retain takes 500–2,000 ms per batch, bottlenecked by LLM fact extraction — [performance docs](https://hindsight.vectorize.io/developer/performance) (search extract)
  - A community run took 772.7 s and 581 model calls for the full LoCoMo retain (10 conversations), about 58 calls per conversation — [issue #20](https://github.com/vectorize-io/hindsight-benchmarks/issues/20) (search extract)
- **Cognee cognify:** the docs describe "many short, schema-constrained calls per document" and recommend a light model. The default rate limiter is 60 requests per minute. No per-document count or timing is published — [Cognee docs](https://docs.cognee.ai/guides/setup-configuration/llm-providers) (search extract)
- **RealMem, average per add:** Mem0 11.751 s, A-Mem 8.663 s, MemoryOS 14.755 s, Graph Mem 26.898 s. "Memory incorporation latency consistently exceeds retrieval" — [arXiv 2601.06966](https://arxiv.org/pdf/2601.06966) (search extract)
- **HaluMem-Medium, total add minutes:** Mem0 2,768, Mem0-Graph 2,840, MemOS 1,029, Memobase 293, Supermemory 273. On HaluMem-Long the ordering changes (Supermemory 1,673, MemOS 1,524), which the authors attribute to how many memory points each extracts — [arXiv 2511.03506](https://arxiv.org/pdf/2511.03506) (search extract). The v1 and v2 texts disagree on which system has the best retrieval.
- A third-party report says storing the first LoCoMo conversation in Mem0 took more than 6 minutes — via [User as Code search extract](https://arxiv.org/html/2606.16707v1) (attribution uncertain)
- MemMachine reports LoCoMo input-token totals of 4.20M (memory mode) against 19.21M for Mem0 under gpt-4.1-mini. It is unclear whether this includes ingest — [arXiv 2604.04853](https://arxiv.org/pdf/2604.04853) (search extract)
- H-Mem puts EverMemOS, MemoryOS and MemTree in a "high indexing-token-cost" group — [arXiv 2605.15701](https://arxiv.org/pdf/2605.15701) (search extract)
- LeanMem claims up to 17.24x fewer construction tokens and 8.41x fewer inference tokens than A-Mem — [arXiv 2608.03463](https://arxiv.org/pdf/2608.03463) (search extract)

### Inferences
- The ingest cost per message for extraction-based systems is about 1–2 LLM calls (Mem0) up to dozens per session (Hindsight, roughly 58 calls per LoCoMo conversation). Ingest wall time is seconds per message, so a full LoCoMo or LongMemEval ingest takes minutes to hours. A non-LLM ingest (embedding only) avoids this whole cost line.

### Gaps
- I found no published LLM-call counts per message for Zep/Graphiti, MemOS, EverMemOS, Letta, Supermemory or Honcho.
- The EverMemOS Appendix Table 8 lifecycle token breakdown was not extracted.

## Independent measurements under a common harness

### Takeaway
There are few truly neutral harnesses:
- RealMem (academic) covers Mem0, A-Mem, MemoryOS and Graph Mem.
- HaluMem is by MemTensor, the MemOS authors.
- User as Code and Lobu are third parties with their own products.
- Supermemory's MemoryBench and Mem0's memory-benchmarks are vendor-run open harnesses.
- "Beyond the Context Window" gives a cost model rather than latency.
- I found no OmniMemEval or MemoryStackBench latency or token tables.

### Cited Findings
- See the independent table above. Sources: [RealMem](https://arxiv.org/pdf/2601.06966), [HaluMem](https://arxiv.org/pdf/2511.03506), [User as Code](https://arxiv.org/html/2606.16707v1), [lobu.ai](https://lobu.ai/guides/memory-benchmarks/), [Beyond the Context Window](https://arxiv.org/abs/2603.04814), [MemoryBench](https://supermemory.ai/docs/memorybench/quickstart) (all search extracts)

### Gaps
- I found no common-harness p95 latency table covering Zep, Hindsight, Supermemory, Letta, Cognee and Mem0 together.

## How latency was measured (qualifiers)

### Cited Findings
- **Mem0 paper:** Zep was called serially, by Zep's account, and the OpenAI row had no search step. Latencies include API calls to gpt-4o-mini and the managed services — [Zep rebuttal](https://blog.getzep.com/lies-damn-lies-statistics-is-mem0-really-sota-in-agent-memory/) (search extract)
- **Zep paper:** includes network from Boston to Zep Cloud. Zep 2026: retrieval-only figures — [arXiv 2501.13956](https://arxiv.org/html/2501.13956v1); [getzep research](https://www.getzep.com/research/)
- **Mem0 v3:** end-to-end p50 on the managed platform — [Mem0 blog](https://mem0.ai/blog/mem0-the-token-efficient-memory-algorithm)
- **Lobu:** retrieval only, mixing local and hosted APIs, "not apples-to-apples" — [lobu.ai](https://lobu.ai/guides/memory-benchmarks/)
- **User as Code:** wall clock from question to answer, excluding the memory build — [arXiv 2606.16707](https://arxiv.org/html/2606.16707v1)
- **Hindsight embeddings leaderboard:** cross-encoder on CPU in Docker, so faster on a GPU — [leaderboard](https://benchmarks.hindsight.vectorize.io/leaderboard/embeddings) (search extract)

### Inferences
- When comparing a local system against these figures, report retrieval-only p50/p95 and end-to-end p50/p95 separately. State the network (none vs managed cloud), the answer model and top-k or token budget. Report ingest calls and time per message as a separate line.

### Gaps
- Most vendor pages do not state hardware, region or concurrency.
