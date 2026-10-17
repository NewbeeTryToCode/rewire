# Graph-, temporal- and belief-based agent memory: does the storage model drive benchmark scores? (as of Oct 2026)

Method note: arxiv.org, aclanthology.org and blog.getzep.com were unreachable from this session (proxy/DNS refusal), so arXiv-only figures are taken from search snippets, vendor pages or the prior notes in `docs/evaluation.md` §4 (which were checked against vendor eval code on 2026-10-14). Items marked **[unverified-here]** are from search summaries or memory of the paper and were not re-read in full during this session.

## 1. Systems: data model, engine, write cost, read latency, published numbers, reproductions

### Takeaway
Every system with a headline score of 90%+ (Mem0 2026, Mastra OM, Hindsight, Agent Zero Memory) reports it with a strong answer model (GPT-5, GPT-5-mini, Gemini-3 Pro) and its own harness. Where third parties have reproduced results, the numbers drop by 10–60 points. The best-documented case: Zep Cloud reproduces at ≈74.6% on LoCoMo, while its open-source engine Graphiti scores ≈53% in the same harness, so the published scores belong to the full product pipeline, not to the graph store.

### Cited Findings
**Zep / Graphiti (temporal knowledge graph; Neo4j, FalkorDB or Kuzu; bi-temporal edges with valid/invalid time)**
- Ingest pipeline per episode: create the episode, retrieve recent episodes for context, extract entities with an LLM, resolve entities and deduplicate, extract facts (edges), resolve edges, invalidate contradicted edges (e.g. an old employer edge is invalidated), extract attributes, persist — [Graphiti summary](https://codex.danielvaughan.com/2026/03/30/graphiti-agent-memory-store/); [Graphiti episodes docs mirror](https://mintlify.com/getzep/graphiti/concepts/episodes)
- Third-party estimate of LLM calls per episode: ≈2–3 for entity extraction, 1–2 for deduplication, 2–3 for fact extraction, 1–2 for summarization (≈6–10 in total; concurrent requests = SEMAPHORE_LIMIT × 6–10, default limit 20). Not confirmed against Graphiti source code — [Graphiti performance tuning (mintlify mirror)](https://mintlify.com/getzep/graphiti/advanced/performance-tuning)
- Retrieval is hybrid (semantic, full-text and BFS graph traversal) with no LLM in the reranking step. The sub-200 ms figures describe Zep's managed platform — [Graphiti summary](https://codex.danielvaughan.com/2026/03/30/graphiti-agent-memory-store/)
- Zep paper (2025): LongMemEval_S 71.2% with gpt-4o versus a 60.2% full-context baseline, about 90% lower response latency, and a reported regression on single-session-assistant questions **[unverified-here]** — [Zep paper arXiv 2501.13956](https://arxiv.org/abs/2501.13956). The prior notes found that Zep's LongMemEval code uses its own grader prompt, not the official LongMemEval one — `docs/evaluation.md` §4
- LoCoMo dispute: Zep first reported 84%. Mem0's CTO argued the adversarial category had been counted in the numerator but left out of the denominator, giving 58.44%. Zep re-ran and reported 75.14% ± 0.17 (gpt-4o-mini as answerer and judge). Zep says Mem0's run of Zep had three errors: both speakers were given the user role, timestamps were appended to the message text, and searches ran sequentially — [Zep blog "Is Mem0 really SOTA"](https://blog.getzep.com/lies-damn-lies-statistics-is-mem0-really-sota-in-agent-memory/) (via search summary); [maximem comparison](https://www.maximem.ai/compare/mem0-vs-zep)
- Mem0 paper's run of Zep: ≈66.0% J on LoCoMo (gpt-4o-mini answerer and judge) — `docs/evaluation.md` §4; [Mem0 paper](https://arxiv.org/abs/2504.19413)
- Independent rerun (a40-labs): Zep Cloud **0.7461** on LoCoMo (contexts re-scored with gpt-4o-mini as reader and judge, within one point of Zep's 75.14). **Graphiti OSS: 0.5338** with an OpenAI-class extractor and 0.5286 with a bge-m3 embedder; LongMemEval_S 0.35. Their verdict: "the 21-point gap [Cloud vs OSS] is real and reproducible; its cause is not observable from outside", and the extractor model class does not explain it. Ingesting LongMemEval-M with Graphiti cost about 600 single-stream GPU-days, or ≈$7,000 hosted — [a40-labs memory repo](https://github.com/agent-memory-atlas-archive/a40-labs--memory)
- Memori Labs' third-party LoCoMo run: Zep 79.1%, LangMem 78.1%, Mem0 62.5%. Zep sent ≈1,602 context tokens per question — `docs/evaluation.md` §4; [Memori results](https://memorilabs.ai/docs/memori-cloud/benchmark/results/)

**Mem0 (vector store of extracted facts) and Mem0ᵍ (adds a graph store, Neo4j)**
- 2025 paper, LoCoMo J (gpt-4o-mini answerer and judge): Mem0 ≈66.9%, Mem0ᵍ ≈68.4%, full context ≈72.9% (≈26k tokens, p95 ≈17 s). Mem0's p95 total latency was ≈1.44 s — `docs/evaluation.md` §4; [Mem0 paper](https://arxiv.org/pdf/2504.19413)
- The graph variant has higher p95 latency: search 0.66 s and total 2.59 s for Mem0ᵍ, versus ≈1.40 s total for base Mem0 — [Mem0 blog](https://mem0.ai/blog/ai-agent-memory-benchmark) (via search summary)
- The 2025 write path was two LLM passes per message pair: extract candidate facts, then reconcile each against similar existing memories with ADD/UPDATE/DELETE/NOOP — [Mem0 migration guide](https://docs.mem0.ai/migration/platform-v2-to-v3)
- **The 2026 algorithm drops the reconciliation step.** It makes one LLM call that only adds memories, so "memory counts will grow over time… by design, since retrieval handles relevance ranking". The guide says this roughly halves extraction latency. It lists LoCoMo 91.6 (+20.2 over the previous 71.4) and LongMemEval 93.4 (+25.6 over 67.8) at 6.8–7.0k tokens per query (top-200). Mem0's launch post gives 92.5 / 94.4 and BEAM 64.1 (1M) / 48.6 (10M) — [Mem0 migration guide](https://docs.mem0.ai/migration/platform-v2-to-v3); [Mem0 token-efficient algorithm blog](https://mem0.ai/blog/mem0-the-token-efficient-memory-algorithm); [Mem0 research](https://mem0.ai/research)
- Those 2026 numbers use GPT-5 as both answerer and judge, come from the managed platform, and use a 200-memory retrieval budget. Mem0 says open-source SDK users should expect "directionally similar gains but not identical numbers" — `docs/evaluation.md` §4 (memory-benchmarks repo); [mem0 GitHub README](https://github.com/mem0ai/mem0)
- Third-party results for Mem0: 62.5% on LoCoMo (Memori); a reimplementation scoring 29.3% (User as Code paper); one Spanish-language review citing 49.0% on LongMemEval — `docs/evaluation.md` §4; [User as Code](https://arxiv.org/pdf/2606.16707); [vantaige review](https://vantaige.io/es/ai-tool/letta)

**Hindsight (Vectorize; PostgreSQL + pgvector; four networks: world facts, experiences, opinions/beliefs with confidence scores, entity summaries; retain / recall / reflect)**
- LongMemEval 91.4% with a Gemini-3 Pro backbone. With an open-source 20B model it scores 83.6%, versus 39% for full context on the same backbone. The paper is vendor-authored — [Hindsight arXiv 2512.12818](https://arxiv.org/pdf/2512.12818); [ACL 2026 demo](https://aclanthology.org/2026.acl-demo.27/); [rywalker overview](https://rywalker.com/research/hindsight)
- An independent reimplementation reported **73.0%** on all 500 LongMemEval questions with Gemini-3 Flash and minimal infrastructure — cited via search summary; the source appears to be [User as Code](https://arxiv.org/pdf/2606.16707) **[unverified-here]**
- On BEAM, Hindsight claims 64.1% at 10M tokens, against LIGHT 26.6% and RAG 24.9% (vendor blog) — [Hindsight BEAM post](https://hindsight.vectorize.io/blog/2026/04/02/beam-sota)

**Mastra observational memory (no database-side structure: an Observer and a Reflector LLM rewrite history into a dated observation log that stays in the prompt)**
- LongMemEval 94.87% with gpt-5-mini and 84.23% with gpt-4o. The same system shows a ≈10.6-point swing from the answer model alone — [Mastra research](https://mastra.ai/research/observational-memory)
- The Observer fires at ≈30k accumulated message tokens and the Reflector at ≈40k observation tokens. ≈6× text compression on LongMemEval. Cost depends on prompt caching of a stable prefix — [Mastra research](https://mastra.ai/research/observational-memory); [Mastra docs](https://mastra.ai/docs/memory/observational-memory)
- Critique: reflection is lossy, and detail is permanently dropped. The source is a competitor — [OMEGA vs Mastra](https://omegamax.co/blog/omega-vs-mastra-observational-memory)

**Agent Zero Memory (2026; episodic timeline + entity-event knowledge graph + "Hierarchical Documentary Memory" semantic store, provenance-aware)**
- Self-reported LongMemEval 95.60% (versus Mastra 94.87%) and LoCoMo 93.60%. Accuracy stays within ≈3.4 points across 8 backbone LLMs; the authors read this as evidence that the memory design drives quality — [Agent Zero Memory arXiv 2608.29606](https://arxiv.org/abs/2608.29606) (via search summary)

**Letta / MemGPT, A-Mem, Cognee, Supermemory**
- A-Mem ≈48.4% J and LangMem ≈58.1% (latency ≈60 s) on LoCoMo, both as measured by the Mem0 paper — `docs/evaluation.md` §4
- Supermemory 85.2% on LongMemEval_S, as summarized in the Agent Zero Memory paper — `docs/evaluation.md` §4
- Letta: the only number found is 74% on LoCoMo, on a competitor's page (MemoryLake). No primary Letta source was found — [MemoryLake vs Letta](https://www.memorylake.ai/products/compare/vs-letta)
- Cognee: no published LoCoMo or LongMemEval numbers were found in this session.

### Inferences
- The large drop from Zep Cloud (≈75%) to Graphiti OSS (≈53%) on the same graph data model shows that the scores depend on undisclosed pipeline choices (prompts, reranking, context assembly), not on Neo4j or FalkorDB as such.
- Mem0's biggest gain (+20 to +26 points) came from **removing** consolidation (ADD-only), retrieving far more (top-200, ≈7k tokens) and moving to GPT-5. That points to retrieval budget and answer model, not a richer store.

### Gaps
- Cognee and Supermemory data models and primary benchmark numbers were not verified. Letta has no primary LoCoMo or LongMemEval figure.
- No published PersonaMem numbers were found for any of these systems.
- Graphiti's exact LLM calls per episode were not verified against its source code.

## 2. Ablations: does a graph or belief layer add accuracy over flat facts plus good retrieval?

### Takeaway
Controlled studies mostly say no, or only on narrow question types. Graphs help a little on temporal and updated-fact questions. They can hurt multi-hop questions and overall F1. Most differences trace back to "foundational system settings" (granularity, retrieval, answer model, judge) rather than to the graph.

### Cited Findings
- Hu et al., "Does Memory Need Graphs?" (ACL 2026): a unified framework with stage-wise ablations on LongMemEval and HaluMem finds "many performance differences are driven by foundational system settings rather than specific architectural innovations". Graphs can help, but "inappropriate graph construction or retrieval strategies can actually hurt performance" — [arXiv 2601.01280](https://arxiv.org/abs/2601.01280); [ACL Anthology](https://aclanthology.org/2026.acl-long.1232/). A graph-DB vendor quotes 89% (graph) versus 76% (flat vector) on LongMemEval from this paper; not verified against the tables — [FalkorDB blog](https://www.falkordb.com/blog/agent-memory-for-long-running-agents/)
- A controlled KG-versus-flat study (HF daily papers listing) turned each turn into typed nodes and edges and answered from a 2-hop subgraph. At a matched budget of five retrieval roots, token F1 was 0.417 versus 0.468 for a flat vector baseline (Δ −0.050, 95% CI [−0.085, −0.016], 500 questions) — [HF papers search "LongMemEval"](https://huggingface.co/papers?q=LongMemEval) **[unverified-here: paper title not captured]**
- Mem0 versus Mem0ᵍ on LoCoMo: graph temporal 58.13 versus 55.51 (≈+2.6), but multi-hop 47.19 versus 51.15 (−4.0). Overall gain ≈+1.5 J at about 1.8× p95 latency — [Mem0 paper](https://arxiv.org/pdf/2504.19413) (per-category values via search summary)
- a40-labs: swapping the reader and judge on identical retrieval moved scores by 6.9 points, more than the 0.3–3.6-point spread between three working stores on LoCoMo. A hybrid structured store and MemPalace tie on LoCoMo (p = 0.06) but separate by 15 points on LongMemEval-M (p = 0.008) — a bundled comparison, so no single mechanism is isolated — [a40-labs](https://github.com/agent-memory-atlas-archive/a40-labs--memory)
- Agent-native memory study: temporal or graph-organized memory helps mostly on revised personal facts and dated events. Keeping raw user content beat compressed versions on LongMemEval (substring EM 26.0 versus 10.7) — [Are We Ready For An Agent-Native Memory System?](https://arxiv.org/html/2606.24775v1)
- Flat typed store: Memanto self-reports 89.8% on LongMemEval with a single retrieval query, claiming to beat hybrid graph+vector systems — [Memanto arXiv 2604.22085](https://arxiv.org/pdf/2604.22085)
- Answer-model sensitivity: Mastra OM 84.2 → 94.9 on LongMemEval from gpt-4o → gpt-5-mini; Hindsight 83.6 (20B) → 91.4 (Gemini-3 Pro) — [Mastra](https://mastra.ai/research/observational-memory); [Hindsight](https://arxiv.org/pdf/2512.12818)
- Counterclaim: Agent Zero Memory reports only a ≈3.4-point spread across 8 backbones (self-reported) — [arXiv 2608.29606](https://arxiv.org/abs/2608.29606)

### Inferences
- The components that seem to matter: (1) answer model; (2) retrieval budget and recall (top-k, context size); (3) keeping raw evidence or provenance next to extracted facts; (4) timestamps and temporal validity for knowledge-update and temporal questions; (5) entity resolution, mainly for multi-session aggregation. The physical database (Neo4j, pgvector, SQLite) is not what any study isolates as a driver, and none shows it to be one.
- Bi-temporal edge invalidation is a pipeline feature. It can be built as columns (`valid_from`, `invalid_at`) in a relational table.

### Gaps
- No study found swaps only the storage engine (e.g. Graphiti on Neo4j versus Kuzu versus a relational equivalent) with the pipeline held fixed.
- The exact "Does Memory Need Graphs?" tables could not be read (ACL and arXiv blocked here).

## 3. Write-time cost: LLM calls for extraction and consolidation, and latency of background "belief" building

### Takeaway
Structured memories pay heavily at write time: Graphiti makes about 6–10 LLM calls per episode, Mem0's 2025 version 2 per message, and Hindsight has a large fixed ingest cost. A cost study finds these may never break even against full history within a normal conversation length.

### Cited Findings
- Graphiti: ≈6–10 LLM calls per episode (third-party estimate). Ingesting LongMemEval-M took ≈600 GPU-days, or ≈$7,000 hosted — [Graphiti tuning](https://mintlify.com/getzep/graphiti/advanced/performance-tuning); [a40-labs](https://github.com/agent-memory-atlas-archive/a40-labs--memory)
- Mem0: the 2025 version made 2 LLM passes per message pair (extract + update). The 2026 version makes 1 ADD-only pass, which roughly halves extraction latency — [Mem0 migration](https://docs.mem0.ai/migration/platform-v2-to-v3)
- Mastra OM: LLM calls are batched; the Observer fires at ≈30k message tokens and the Reflector at ≈40k observation tokens — [Mastra](https://mastra.ai/research/observational-memory)
- "Total Recall at What Cost?" (Aug 2026; 665 LoCoMo questions, two backbones, up to 400 turns): memory-system cost misses a regression fit to the baselines by 18–69%. Break-even against full history on gpt-oss-20b comes at turn 0 for Mastra OM, turn 82 for Mem0 and turn 356 for Hindsight. "Most memory systems have not yet paid back their ingest cost." Accuracy spans 21–54%, and no system wins on both cost and accuracy. A reviewer notes that Hindsight's ingest configuration was not controlled — [arXiv 2608.11879](https://arxiv.org/abs/2608.11879v1); [Pith review](https://pith.science/paper/2608.11879)
- Read-side context: Zep ≈1.6k tokens per question, Mem0 2025 ≈1.8k, Mem0 2026 ≈7k (200 facts), full context ≈26k (LoCoMo) — `docs/evaluation.md` §4

### Inferences
- For a local, single-user tool, an LLM-free ingest (raw episodes plus indexes) wins on cost. Extraction or belief layers only pay off if they are reused across many queries or very long horizons.

### Gaps
- No published per-session wall-clock latency was found for Hindsight's "reflect" step or Graphiti's background processing.

## 4. Failure modes reported

### Takeaway
The write path is the main source of errors. Extraction misses or omits facts far more often than it hallucinates them. Update and conflict handling is weak (Mem0 omits ≈74% of updates). Lossy compression destroys raw detail.

### Cited Findings
- HaluMem-Medium: Mem0 recalls ≈42.9% of reference memory points at extraction (Mem0ᵍ ≈43.3%). On updates, Mem0 is 25.5% correct, 0.45% hallucinated and 74.0% omitted; Mem0ᵍ is 24.5% correct and 75.2% omitted; Zep is 47.3% correct, 0.42% hallucinated and 52.3% omitted (dropping to 37.4% correct on HaluMem-Long). QA: Zep 55.5% versus Mem0 53.0%. Omissions trace back to upstream extraction — [HaluMem GitHub](https://github.com/MemTensor/HaluMem); [HaluMem arXiv 2511.03506](https://arxiv.org/pdf/2511.03506)
- The graph variant does not fix update handling: Mem0ᵍ ≈ Mem0 on HaluMem update scores — [HaluMem](https://github.com/MemTensor/HaluMem)
- Lost raw detail: compression drops LongMemEval substring EM from 26.0 to 10.7 — [Agent-native memory study](https://arxiv.org/html/2606.24775v1). Mastra reflection is lossy (competitor's critique) — [OMEGA](https://omegamax.co/blog/omega-vs-mastra-observational-memory)
- Staleness and growth: Mem0 2026 abandons update/delete reconciliation, so the memory count grows and stale facts are left for retrieval ranking to handle — [Mem0 migration](https://docs.mem0.ai/migration/platform-v2-to-v3)
- Configuration sensitivity: Mem0's misconfigured run of Zep (speaker roles, timestamps in text) is blamed for a ≈9-point gap — [Zep blog](https://blog.getzep.com/lies-damn-lies-statistics-is-mem0-really-sota-in-agent-memory/)
- Benchmark scope: including or excluding LoCoMo's adversarial category swings Zep from 84 to 58.44 — [a40-labs](https://github.com/agent-memory-atlas-archive/a40-labs--memory)

### Inferences
- Storing the raw transcript alongside any derived facts or beliefs, with provenance links (as Agent Zero Memory and Eywa emphasize), protects against both extraction omission and stale beliefs.

### Gaps
- No quantified stale-belief rates were found for Hindsight's opinion network or for Graphiti edge invalidation.
