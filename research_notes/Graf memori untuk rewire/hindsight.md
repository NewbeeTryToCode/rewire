# Hindsight (vectorize-io/hindsight) as of Oct 2026: memory model, graph, retrieval, reflection, evidence, and what rewire could adopt

Scope note: the code facts below come from the local clone `/home/user/vectorize-io/hindsight` at commit `5331da2` (2026-10-16). Links point to `https://github.com/vectorize-io/hindsight/blob/5331da2/<path>`, which is the same path in the clone. All Python paths are under `hindsight-api-slim/hindsight_api/` (shortened below as `HA/`).

Web access to arxiv.org, ar5iv and papers.cool was blocked by the egress proxy, so I could not read the paper itself (arXiv 2512.12818). Paper numbers come from search-result summaries and from the earlier notes in `research_notes/CortexDB versus SQLite untuk rewire/graph_memory.md`, and they are marked as such.

**Important correction to the earlier notes and to the paper's description.** The paper and those notes describe four networks: world, experience, opinion (with confidence) and observation/entity summaries. The current code no longer has opinions or confidence scores.
- A 2026-04-02 migration deleted all `opinion` rows and dropped `confidence_score`.
- Since May 2026, entity edges are no longer stored as link rows.
- Graph retrieval is now a bounded one-hop "link expansion", not multi-hop spreading activation. Spreading activation survives only inside the temporal arm.

## 1. Data model: memory networks, entities, links, storage, and how the graph is built

### Takeaway
Everything lives in PostgreSQL with pgvector (or an embedded Postgres via `pg0`; Oracle is also supported).

- **Memories.** One `memory_units` table holds `fact_type ∈ {world, experience, observation}`. Each row has an embedding, a time range (event/occurred/mentioned timestamps), tags, and for observations a `proof_count`, `source_memory_ids` and a history.
- **Entities** are separate rows, joined to memories through `unit_entities`, with an `entity_cooccurrences` table.
- **Edges** live in `memory_links` (types: temporal, semantic, causal). Entity edges are derived at query time instead of stored.
- **Building the graph** takes one LLM extraction call per ≈3,000-character chunk. That call returns facts, entities, dates and `caused_by` relations. Everything else is non-LLM: entity resolution, temporal links, and semantic kNN links.

### Cited Findings
**Schema**
- The ORM schema has these tables:
  - `documents`
  - `memory_units`: `text`, pgvector `embedding`, `event_date`, `occurred_start/end`, `mentioned_at`, `fact_type`, metadata
  - `entities`: `canonical_name`, `first_seen`, `last_seen`, `mention_count`
  - `unit_entities`
  - `entity_cooccurrences`: an ordered pair with `cooccurrence_count` and `last_cooccurred`
  - `memory_links`: `from_unit_id`, `to_unit_id`, `link_type`, `entity_id`, `weight ∈ [0,1]`
  - `banks`: `disposition` JSONB and `background`

  Two CHECK constraints are worth noting: `fact_type IN ('world','experience','observation')` and `link_type IN ('temporal','semantic','entity','causes','caused_by','enables','prevents')`. — [HA/models.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/models.py) lines 98–338

**Opinions removed**
- The 2026-04-02 migration says: "Remove the deprecated 'opinion' fact type… delete any remaining opinion rows, and drop the confidence_score column (was only used for opinions, always NULL otherwise)." — [HA/alembic/versions/g2h3i4j5k6l7_remove_opinion_fact_type.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/alembic/versions/g2h3i4j5k6l7_remove_opinion_fact_type.py)
- The README now lists world facts, experiences, observations ("consolidated, evidence-backed beliefs formed from many memories") and mental models. It does not list opinions. — [README.md](https://github.com/vectorize-io/hindsight/blob/5331da2/README.md) "Memory Types"

**Entity edges no longer stored**
- The 2026-05-26 migration explains: "Entity edges are no longer stored in `memory_links`. The /graph endpoint derives them on demand from `unit_entities`, and recall already used the `unit_entities` self-join… on a 10k-unit benchmark bank, entity rows were 53% of all link rows (~190 MB after indexes) and recall never touched them." — [HA/alembic/versions/e9b2c7d1f3a4_drop_entity_memory_links.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/alembic/versions/e9b2c7d1f3a4_drop_entity_memory_links.py)

**Observation provenance**
- Observations are stored as `memory_units` rows with `fact_type='observation'`, carrying `proof_count`, a `source_memory_ids` array (GIN-indexed) and a `history` JSONB. History was later split into an `observation_history` table. — [HA/engine/consolidation/consolidator.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/consolidation/consolidator.py) docstring; [HA/alembic/versions/a7b8c9d0e1f2_split_history_into_own_tables.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/alembic/versions/a7b8c9d0e1f2_split_history_into_own_tables.py); [HA/alembic/versions/a2b3c4d5e6f8_add_gin_index_source_memory_ids.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/alembic/versions/a2b3c4d5e6f8_add_gin_index_source_memory_ids.py)

**LLM extraction schema**
- Each extracted fact has these fields: `what`, `when`, `where`, `who`, `why`, `fact_kind` (event/conversation), `occurred_start`, `occurred_end`, `fact_type ∈ {world, assistant}`, `entities: list[str]`, and `causal_relations` (`target_index`, `relation_type: Literal["caused_by"]`).
- Concise, verbose, no-causal and verbatim variants exist.
- Defaults: chunk size 3,000 characters; maximum completion tokens 64,000; `EXTRACT_CAUSAL_LINKS=True`; extraction mode "concise".
- Sources: [HA/engine/retain/fact_extraction.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/retain/fact_extraction.py) lines 162–315; [HA/config.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/config.py) lines 1749–1765

**Causal links**
- Retain writes only the canonical `caused_by` relation. The other causal types (`causes`, `enables`, `prevents`) are still recognised so that older or imported memories stay traversable. — [HA/engine/retain/link_creation.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/retain/link_creation.py)

**Temporal links**
- For each new unit, a bidirectional B-tree index scan on `(bank_id, fact_type, event_date)` fetches the nearest K neighbours in time.
- Weight is `max(0.3, 1 − Δhours/24)` with a 24-hour window, capped at `MAX_TEMPORAL_LINKS_PER_UNIT = 20`.
- The code comment reports this as "120x faster at 164k units (0.6ms vs 74ms)".
- Source: [HA/engine/memories/pg/links.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/memories/pg/links.py) lines 47, 457–580

**Semantic links**
- At insert time, an ANN probe links each new fact to up to `top_k=20` existing facts with cosine ≥ `semantic_link_min_similarity`, which defaults to 0.7.
- Near-duplicates are therefore kept and linked, not deleted.
- Sources: [HA/engine/retain/orchestrator.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/retain/orchestrator.py) ~line 2138; [HA/config.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/config.py) line 1380; [blog "Bring facts, not beliefs"](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-docs/blog/2026-09-23-bring-facts-not-beliefs.md)

**Entity resolution (no LLM)**
- Candidates come from pg_trgm trigram lookup and must pass a trigram gate (`merge_min_similarity` 0.3) plus a word-by-word compatibility check.
- The score is then:
  - name `SequenceMatcher` ratio × 0.5
  - plus co-occurring-entity overlap, weighted by selectivity, × 0.3
  - plus temporal proximity to `last_seen` (inside 7 days) × 0.2
- A candidate merges when the score is ≥ 0.6. Identical trigram sets merge at score 1.0.
- Within one batch, a separate dedup pass uses Jaccard ≥ 0.5 and is skipped above 250 new names.
- The module docstring mentions spaCy, but the current scorer is the heuristic above.
- Source: [HA/engine/memories/pg/entity_resolver.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/memories/pg/entity_resolver.py) lines 1–85, 1246–1345
- The vendor blog says entity resolution uses "a co-occurrence graph rather than embeddings". — [blog: knowledge graphs vs vector search](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-docs/blog/2026-08-24-knowledge-graphs-vs-vector-search.md)

**Embeddings and reranker defaults**
- Local `BAAI/bge-small-en-v1.5` at 384 dimensions.
- Local cross-encoder `cross-encoder/ms-marco-MiniLM-L-6-v2`, with 300 reranker candidates.
- Source: [HA/config.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/config.py) lines 1278–1378

### Inferences
- In practice, Hindsight's "graph" is a property graph over atomic facts, not a subject–predicate–object knowledge graph. Facts are nodes; edges are time-adjacency, embedding-kNN and LLM-extracted `caused_by`; entities act as hyperedges through a join table. There are no typed relations such as `works_at`. The vendor blog's "Alice → works_at → Google" example describes the concept, not the implementation.
- The project's own history argues against materialising entity edges. They took 53% of link storage and recall never read them. A join table (`unit_entities`) plus a query-time self-join was enough.

### Gaps
- I could not confirm whether the paper's v0.1.0 data model (opinion network with confidence) was what produced the paper's benchmark numbers; it very likely was. Reason: arxiv.org was unreachable.
- I did not measure the average number of facts, entities or links per message.

## 2. Retrieval (recall): channels, fusion, reranking, token budget, latency

### Takeaway
Recall runs four arms in parallel, separately for each fact type: semantic (pgvector), keyword (BM25 or Postgres `ts_rank_cd`), graph (one-hop link expansion from semantic seeds), and temporal. The temporal arm uses a rule-based date parser, coverage-spread entry points, and spreading over temporal and causal links with 0.7 decay per hop. Results are merged with RRF (k=60), reranked with a cross-encoder plus small multiplicative recency, temporal and proof-count boosts, then filled to a token budget (default `max_tokens` 4,096). No LLM is called during recall. The vendor states recall latency of 100–600 ms.

### Cited Findings
**Graph arm**
- `LinkExpansionRetriever` starts from up to `GRAPH_SEED_LIMIT=20` semantic seeds with similarity ≥ 0.3. It then runs one CTE that unions three expansions:
  - **entity:** `COUNT(DISTINCT entity_id)` shared with the seeds, through a `unit_entities` self-join, with a per-entity LATERAL cap of 200
  - **semantic:** stored kNN links in both directions; score = link weight
  - **causal:** score = link weight
- The candidate's score is `tanh(0.5·shared_entities) + semantic_weight + causal_weight`, a value in [0, 3]. The docstring says causal links get "+1.0 (boosted as highest-quality signal)", but the merge code adds only the raw weight. This mismatch is unresolved; I did not inspect the SQL in `ops`.
- On timeout, entity expansion is dropped.
- Sources: [HA/engine/memories/pg/link_expansion.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/memories/pg/link_expansion.py); [HA/engine/memories/base.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/memories/base.py) line 796; [HA/config.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/config.py) line 1378
- `GraphRetriever` is an abstract strategy interface, so graph algorithms are pluggable. — [HA/engine/search/graph_retrieval.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/search/graph_retrieval.py)

**Temporal arm**
- The query's time window is parsed without an LLM, using `dateparser` and a period extractor. — [HA/engine/query_analyzer.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/query_analyzer.py)
- From an ANN pool inside the window, it keeps 10 entry points per fact type spread over 8 time buckets.
- Each entry point's score is its temporal proximity to the window midpoint.
- Breadth-first spreading over `temporal`/`causes`/`caused_by`/`enables`/`prevents` links, with:
  - propagated score = `parent × weight × causal_boost × 0.7`
  - causal boosts: causes/caused_by 2.0, enables/prevents 1.5
  - 10 neighbours per source and at most 5 iterations
- Source: [HA/engine/memories/pg/recall.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/memories/pg/recall.py) lines 310–317, 374–640
- The vendor describes this in its blog as "time-aware spreading activation" (Collins & Loftus). — [blog 2026-03-12](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-docs/blog/2026-03-12-spreading-activation-memory-graphs.md)

**Fusion**
- RRF `Σ 1/(60+rank)` over the arms [semantic, bm25, graph, temporal], with an optional cap per source. — [HA/engine/search/fusion.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/search/fusion.py)
- `HINDSIGHT_API_RECALL_STRATEGY_BOOSTS` (for example `graph:high`) can favour one arm, both in rank space before the reranker cap and additively after reranking. — [HA/engine/search/recall_boost.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/search/recall_boost.py)

**Reranking**
- The cross-encoder score gets multiplicative boosts:
  - recency, alpha 0.2: linear over 365 days with a 0.1 floor, or exponential with a 90-day half-life
  - temporal proximity, alpha 0.2
  - proof_count, alpha 0.1 (at most ±5%)
- Source: [HA/engine/search/reranking.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/search/reranking.py) lines 1–80

**Keyword search**
- Five BM25 backends are available. The default `native` backend is "PostgreSQL `tsvector` + `ts_rank_cd` (TF-IDF, not true BM25)". — [hindsight-docs/docs/developer/retrieval.md](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-docs/docs/developer/retrieval.md)

**Budgets**
- The `budget` setting controls search depth: low = 100, mid = 300, high = 1,000 candidates per arm. An adaptive mode instead scales depth with `max_tokens`.
- The output is filled top-down until `max_tokens` (default 4,096) is used up.
- Source: [retrieval.md](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-docs/docs/developer/retrieval.md) lines 137, 354–385

**Latency (vendor-stated, not measured here)**
- Recall 100–600 ms; the main bottleneck is the cross-encoder on CPU.
- Reflect 800–3,000 ms.
- Retain 500–2,000 ms per batch; the main bottleneck is LLM extraction.
- Source: [hindsight-docs/docs/developer/performance.md](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-docs/docs/developer/performance.md) lines 28–32

### Inferences
- Without the paper's ablations, the graph arm's only demonstrable job is to add candidates that share entities with, or are kNN or causal neighbours of, the top semantic hits. It never sees a query entity directly: seeds are purely semantic.
- This makes it close to "pseudo-relevance feedback via entity overlap". That is cheap to reproduce in SQLite.

### Gaps
- I found no per-arm ablation (for example removing the graph arm) in the repo, docs or blogs. The paper may contain one, but I could not read it. Two searches for an ablation turned up nothing ([WebSearch summary of arXiv 2512.12818](https://arxiv.org/abs/2512.12818)).
- I found no independent latency measurements.

## 3. Reflection, observations (consolidation) and the "opinion" lineage

### Takeaway
"Opinions with confidence" have been replaced by **observations**.

**Observations** are built in the background after each retain. A "decision model" can first triage easy facts:
- a fact that restates an existing observation is attached as evidence;
- a novel fact becomes its own observation;
- chatter is dropped.

Only facts that change an existing observation go to an LLM rewrite, batched 8 facts per call. Observations carry `proof_count`, `source_memory_ids`, quoted evidence and a capped history. Near-duplicates (cosine ≥ 0.97) go through a merge-or-keep check.

**Reflect** is an on-demand agentic tool loop of up to 10 iterations. It searches in this order: mental models, then observations, then raw facts. A bank "disposition" (skepticism, literalism, empathy, each 1–5) shapes the answer, and citations are validated.

**Mental models / knowledge pages** are standing questions whose answers are regenerated in the background and can be projected to markdown files on disk.

### Cited Findings
- The consolidation engine "runs as a background job after retain… Creates new observations… Updates existing observations when new evidence supports/contradicts/refines them". — [HA/engine/consolidation/consolidator.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/consolidation/consolidator.py)
- Consolidation settings and their defaults:

  | Setting | Default |
  |---|---|
  | `CONSOLIDATION_LLM_BATCH_SIZE` | 8 facts per LLM call |
  | `CONSOLIDATION_BATCH_SIZE` | 50 |
  | `CONSOLIDATION_DEDUP_THRESHOLD` | 0.97 |
  | `OBSERVATION_HISTORY_MAX_ENTRIES` | 50 |
  | `CONSOLIDATION_USE_DECISION_MODEL` | True |
  | `DECISION_MODEL_PROVIDER` | "none", so triage is inactive unless a provider such as Jev, Cloudflare or OpenAI Decisions is configured |

  Source: [HA/config.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/config.py) lines 1538–1546, 1832–1874
- How triage works: "Most facts reaching a bank fall in one of three easy cases… restate… add something no observation holds yet… or are chatter… Only a fact that affects what an existing observation says… needs the LLM's rewrite." Each fact is shown 10 candidate observations, and a verdict stands only when the answers agree. — [HA/engine/consolidation/triage.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/consolidation/triage.py)
- How contradictions are handled: rather than overwriting an observation, the system rewrites it to capture the change (for example "previously a React enthusiast… now switched to Vue"). Raw facts are always preserved. When newer memories have not yet been consolidated, reflect treats the affected observations as stale and checks them against raw facts. — [hindsight-docs/docs/developer/observations.mdx](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-docs/docs/developer/observations.mdx)
- The reflect tools are `search_mental_models`, `read_mental_models`, `search_observations`, `recall`, `expand` and `done`. The agent must gather evidence before answering and runs up to 10 iterations. Disposition traits are rated 1–5. — [hindsight-docs/docs/developer/reflect.mdx](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-docs/docs/developer/reflect.mdx)
- Knowledge pages: "the CLI can mirror a bank's knowledge base onto disk… each page a real markdown file with YAML frontmatter, kept current by a background refresh loop… they aren't the storage, they're the rendering." — [hindsight-docs/docs/developer/knowledge-pages.mdx](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-docs/docs/developer/knowledge-pages.mdx)
- In the paper (v0.1.0, per search summary), CARA handles reflect "with configurable disposition parameters… and maintains an opinion network that changes over time". — [arXiv 2512.12818](https://arxiv.org/abs/2512.12818) (via search summary; not read directly)

### Inferences
- The evidence-counting approach is close to rewire's design (supersede rather than overwrite, bitemporal facts, provenance). Hindsight dropped its separate confidence number in favour of `proof_count` plus evidence links. That suggests rewire should derive confidence from provenance and event counts rather than store an LLM-assigned score.
- The triage idea — decide most facts cheaply and send only the "changes an existing belief" cases to an LLM — is the main cost-control lesson.

### Gaps
- I did not trace whether CARA's earlier numeric rules for updating opinion confidence still exist anywhere. They were deleted along with the opinion type.
- I found no published wall-clock time or token cost per consolidation.

## 4. Benchmarks: numbers, models, independence

### Takeaway
Every headline number is produced by the vendor, with strong answer models and the vendor's own harness (AMB):
- LongMemEval 91.4% (paper, v0.1.0, Gemini-3 Pro) and 94.6% (blog, v0.4.19)
- LoCoMo 89.61% (paper) and 92.0% (blog)

The vendor claims an independent reproduction (Virginia Tech Sanghani Center, The Washington Post), but I found no reproduction report. An independent reimplementation got 73.0%, and an independent cost study is unfavourable. I found **no ablation showing what the graph contributes**. The vendor itself credits its gains to observations, better extraction and a reworked retrieval pipeline.

### Cited Findings
- Paper results (vendor): with a 20B open-source backbone, LongMemEval rises from 39.0% (full context) to 83.6% and LoCoMo from 75.78% to 85.67%. With larger backbones: LongMemEval 91.4% and LoCoMo 89.61%. — [arXiv 2512.12818](https://arxiv.org/abs/2512.12818) (search summary); earlier notes cite the 91.4% as Gemini-3 Pro ([graph_memory.md](/home/user/rewire/research_notes/CortexDB%20versus%20SQLite%20untuk%20rewire/graph_memory.md))
- AMB re-run of v0.4.19 in single-query mode (vendor): LoCoMo 92.0%, LongMemEval 94.6%, LifeBench 71.5%, PersonaMem 86.6%. The vendor attributes the gains to "Observations", a "Better retain process" and a reworked "Retrieval algorithm", adding that "attribution across dozens of changes is never clean". The same post warns that judge and answer prompts "can swing accuracy scores by double digits", and that LoCoMo and LongMemEval now "mostly measure whether your LLM can read". — [blog 2026-03-23 AMB](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-docs/blog/2026-03-23-agent-memory-benchmark.mdx)
- BEAM (vendor): 64.1% at 10M tokens, against Honcho 40.6%, LIGHT 26.6% and RAG 24.9% (the last two are paper baselines on Llama-4-Maverick). At 100K / 500K / 1M tokens Hindsight scores 73.4% / 71.1% / 73.9%. — [blog 2026-04-02 BEAM](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-docs/blog/2026-04-02-beam-sota.md)
- The README claims the results were "independently reproduced by research collaborators at the Virginia Tech Sanghani Center… and The Washington Post". I found no standalone reproduction document. — [README.md](https://github.com/vectorize-io/hindsight/blob/5331da2/README.md)
- The in-repo LoCoMo and LongMemEval runner was removed; results now come from AMB and are published to a dashboard repo. — [scripts/benchmarks/publish-locomo-results.sh](https://github.com/vectorize-io/hindsight/blob/5331da2/scripts/benchmarks/publish-locomo-results.sh). The in-repo `hindsight-dev/benchmarks/` holds only consolidation, observation, performance and micro benchmarks, with no recall-arm ablation.
- Independent evidence, carried over from the earlier notes and not re-verified here (web blocked):
  - A reimplementation scored 73.0% on LongMemEval-500 with Gemini-3 Flash.
  - "Total Recall at What Cost?" finds Hindsight breaks even against full history only at turn 356 (gpt-oss-20b). A reviewer notes that Hindsight's ingest configuration was not controlled.
  - Source: [graph_memory.md](/home/user/rewire/research_notes/CortexDB%20versus%20SQLite%20untuk%20rewire/graph_memory.md); [arXiv 2608.11879](https://arxiv.org/abs/2608.11879v1)

### Inferences
- The graph's marginal value is unproven. If anything, Hindsight's own engineering history points the other way: entity edges removed, multi-hop spreading reduced to one hop, observations credited for the gains.
- For rewire, the higher-value ideas are evidence-backed consolidation and query-free temporal handling, not a richer graph.

### Gaps
- The paper's ablation tables and its judge model could not be read (arxiv blocked).
- The answer and judge models used for the v0.4.19 AMB numbers are not stated in the blog post.

## 5. Cost, infrastructure, licence

### Takeaway
**Writes.**
- Retain costs about one LLM call per ≈3,000-character chunk, with up to 64k output tokens allowed.
- Consolidation adds about one LLM call per 8 facts that change an existing observation; without a decision model, every new fact goes through the LLM batches.
- Optional near-duplicate merge checks add more.
- Embeddings and reranking run locally on CPU by default.

**Reads.** Recall uses no LLM. Reflect uses up to 10 tool-loop LLM turns.

**Infrastructure.** PostgreSQL with pgvector (embedded `pg0` in the Docker image) or Oracle; there is no SQLite backend. The licence is MIT.

### Cited Findings
- Retain configuration: chunk size 3,000 characters, maximum completion tokens 64,000, causal extraction on. Retain can also use LLM Batch APIs, but only when async. — [HA/config.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/config.py) lines 1749–1811
- The vendor states the design trade-off directly: "prioritize read performance over write performance… The LLM is the primary bottleneck for write latency." — [performance.md](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-docs/docs/developer/performance.md)
- Embedded Postgres runs via `pg0-embedded` ([HA/pg0.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/pg0.py)). Oracle is supported ([README.md](https://github.com/vectorize-io/hindsight/blob/5331da2/README.md)). A grep for "sqlite" in the API package matched only `db_url.py`.
- The licence is MIT, "Copyright (c) 2025 Vectorize AI, Inc." — [LICENSE](https://github.com/vectorize-io/hindsight/blob/5331da2/LICENSE)

### Gaps
- I found no measured tokens per message for retain plus consolidation.

## 6. Separable mechanisms rewire could implement on files + SQLite, without a server

### Takeaway
Most of Hindsight's graph and retrieval machinery is plain SQL plus heuristics and maps cleanly onto rewire's derived SQLite index. Only fact extraction and observation rewriting need an LLM, and both fit rewire's existing write and review paths. The ranked list below is my judgement; each item cites the Hindsight mechanism it is based on.

### Cited Findings (mechanism → source)
1. **Entity join table instead of entity edges**, with entity-overlap expansion from the top semantic and FTS hits scored `tanh(0.5·n_shared)`, and a per-entity fan-out cap. — [link_expansion.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/memories/pg/link_expansion.py); [e9b2c7d1f3a4 migration](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/alembic/versions/e9b2c7d1f3a4_drop_entity_memory_links.py)
2. **Precomputed semantic kNN edges** (cosine ≥ 0.7, top 20), recomputable when the index is rebuilt; and **temporal adjacency edges** (24-hour window, weight `max(0.3, 1−Δh/24)`, cap 20). — [links.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/memories/pg/links.py); [orchestrator.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/retain/orchestrator.py)
3. **Temporal arm without an LLM**: parse the date window with `dateparser`, pick entry points spread across 8 time buckets, spread over temporal and causal edges with 0.7 decay per hop. — [query_analyzer.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/query_analyzer.py); [recall.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/memories/pg/recall.py)
4. **RRF across arms with a cap per source**, then a small multiplicative boost for recency, temporal proximity and evidence count after reranking. — [fusion.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/search/fusion.py); [reranking.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/search/reranking.py)
5. **Heuristic entity resolution** (trigram gate + name ratio 0.5 + co-occurrence 0.3 + recency 0.2, threshold 0.6). — [entity_resolver.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/memories/pg/entity_resolver.py)
6. **Observations as nodes with provenance** (`source_memory_ids`, `proof_count`, history), refined rather than overwritten, with triage so that only belief-changing facts reach an LLM. — [consolidator.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/consolidation/consolidator.py); [triage.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/consolidation/triage.py)
7. **Mental models / knowledge pages as markdown renderings** regenerated from memory. — [knowledge-pages.mdx](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-docs/docs/developer/knowledge-pages.mdx)

### Inferences
- **(1)–(4) belong in rewire's derived index** and change neither the file format nor git.
  - `unit_entities` and `entities` become SQLite tables rebuilt from node frontmatter or extracted names.
  - kNN and temporal edges can be computed by numpy brute force over the existing 256-dimension WordLlama BLOBs.
  - Entity overlap is a single `GROUP BY` self-join.
  - Of these, (1) and (3) are the cheapest to A/B against rewire's LongMemEval/LoCoMo harness. The temporal arm in particular targets known weak categories (temporal reasoning) and needs no LLM.
- **(2) carries a precision risk.** Rewire's static WordLlama embeddings are weaker than bge-small, so the 0.7 threshold would need re-tuning. kNN edges may add little over the semantic arm itself.
- **Causal `caused_by` edges** need an LLM at write time. Rewire's typed graph links between nodes (already in `edges`) cover the same role and are reviewable in markdown.
- **(6) is the best conceptual fit** with rewire's P1/P4 principles ("store events, not weights"; files are the truth).
  - An observation can be a markdown node whose frontmatter lists supporting node or event IDs.
  - `proof_count` is derived from events, not stored.
  - Supersession preserves history in git.
  - The LLM rewrite runs at session end or in review, not as a server daemon.
- **(7) Hindsight is converging on rewire's model.** Its knowledge pages project to markdown on disk. Rewire already treats markdown as the source of truth.
- **Do not adopt:**
  - Postgres/pgvector or the server architecture.
  - Storing entity edges.
  - Multi-hop spreading for general queries. Hindsight itself narrowed it to one hop plus a temporal-only spread.
  - LLM-assigned confidence scores, which Hindsight removed.

### Gaps
- No evidence quantifies how much each arm (especially the graph) adds. Any adoption should be gated by rewire's own A/B on LongMemEval, LoCoMo and BEAM.
- The web sources (paper ablations, the AMB judge configuration, the reproduction reports) were unreachable from this session.
