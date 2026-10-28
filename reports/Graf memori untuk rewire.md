# Measure rewire's graph before growing it

rewire can be combined with a graph, but it already is one. It has a typed property graph of 7 node types and 6 relations, stored as wikilinks in markdown frontmatter, with bitemporal fields. That graph is copied into a derived SQLite `edges` table and traversed at recall by 2-hop spreading activation, with identical Python and Rust implementations. So the real question is which ideas from the four projects should change that graph, not which project to adopt. The answer is **none of them as a component**. cortex-works/cortex-db contains no code. CortexaDB's graph is weaker than rewire's and has no keyword search. Cortex-AI's extraction scores node F1 0.25 and relation F1 0.0. Hindsight is a PostgreSQL server whose own history argues against heavier graphs. Several small mechanisms are still worth taking. The strongest fact in the evidence is a gap: **no run in the repository has ever measured what the existing graph contributes**. Most result files use `episodes` mode, which has no graph. The full-mode LoCoMo run (93.0% J) never switched spreading off. And the only entity experiments, a second hop over transcript excerpts, made multi-hop *worse* (32.3% → 25.5% all-evidence at k=10). The recommended order is therefore: measure first (a free spreading ablation on the brains already ingested), then make spreading query-relevant (CortexaDB's re-score-the-neighbours pattern), then test a derived entity-overlap arm (Hindsight's join table) with a hard kill criterion. Only after that should rewire spend LLM calls on evidence-backed observation nodes (Hindsight) and add review tooling (Cortex-AI's source retraction and rule-based contradiction flags). Every retrieval step can be A/B tested at roughly $0 LLM cost with `--retrieval-only` on the cached Haiku brains. The real cost of each step is engineering time, doubled by the Python/Rust parity contract.

## rewire already runs a typed, bitemporal graph whose value is unmeasured

rewire's L2 layer defines claim types `constraint, preference, fact, decision`, hub types `project, person, concept`, and relations `about, part_of, depends_on, related_to, supersedes, contradicts` ([schema.py](../src/rewire/schema.py)). Each node carries world time (`valid_from`/`valid_to`) and belief time (`recorded_at`/`retracted_at`), and `status` is derived from them ([format.md](../docs/design/format.md)). The index rebuilds an `edges(src, dst, rel)` table from frontmatter. `Index.search` seeds the **top 10 BM25 hits** (plus a 0.15 semantic bonus and a time-window bonus) and spreads for **2 hops** with these parameters:

- **Relation weights:** `part_of`/`depends_on` 0.8, `related_to` 0.6, `about` 0.5, `contradicts` 0.4, `supersedes` 0.2.
- **Hop decay** 0.5, **fan divisor** `1 + ln(1+degree)`, and a **0.02 cutoff**.
- **A top-10 frontier** per hop.
- **Supersedes forwarding:** a superseded node passes 0.9 of its energy to its replacement.

The final score multiplies energy by ACT-R activation, and every hit carries a `via` explanation ([index.py](../src/rewire/index.py)). The Rust twin has the same constants and the same loop ([index.rs](../rust/src/index.rs)), and recall parity was 233/233 identical on two LoCoMo conversations ([LoCoMo trim](../experiments/eval/results/2026-10-14-locomo-trim.md)). Links cost nothing extra to produce: the single extraction call per chunk already returns `links[{rel ∈ part_of|depends_on|related_to, to}]`, and timeline changes are linked with `related_to` ([consolidate.py](../src/rewire/consolidate.py)).

Three structural facts constrain any graph extension:

- **Excerpts are outside the graph.** Episodes (transcript excerpts) are reachable only through `_seed_episodes`, which boosts the excerpt containing a fact's source turn by 0.35. `edges.dst` is an 8-character node suffix, so an edge to an excerpt cannot even be written.
- **The judge ignores the graph.** Consolidation dedup calls search with `spread=False`.
- **The graph has almost no entities.** The full LoCoMo ingest produced 2,401 facts, 553 preferences and 449 decisions, but only **35 person hubs and 8 concept hubs** across 10 conversations ([LoCoMo Haiku full](../experiments/eval/results/2026-10-15-locomo-haiku-full.md)). It is mostly claim-to-claim links plus `about` edges to project hubs.

The evidence about this graph is thinner than the architecture suggests. Of about 60 result files with a mode header, 48 are `--mode episodes`, which stores only transcripts. The repository says so outright: "`episodes` mode has no graph", and query-relevant spreading "needs `full` mode (facts) to measure" ([LoCoMo trim](../experiments/eval/results/2026-10-14-locomo-trim.md)). The four-benchmark A/B of 2026-10-17 (base 70.2%, A 70.9%, B 67.3% over 190 questions) ran in episodes mode too ([four-benchmark A/B](../experiments/eval/results/2026-10-17-ab-four-benchmarks.md)). The graph was exercised only in the full-mode LoCoMo runs. The best is 1,540 questions at **J 93.0%**, with multi-hop 92.2%, open-domain 66.7% and temporal 94.1%, and Haiku 5.5 as ingest model, answerer and judge ([LoCoMo Haiku full](../experiments/eval/results/2026-10-15-locomo-haiku-full.md)). None of those runs switched spreading off, so the graph's contribution is **unknown, not zero**. BEAM and PersonaMem refuse full mode by design, because of cost and because facts could leak turns that come after the question ([run.py](../experiments/eval/run.py)). For now, LoCoMo full mode is the only testbed where a graph change can show up at all.

The one graph-like experiment that was measured failed. On LoCoMo excerpts at k=10/20, the baseline scored 77.5/82.9 overall and **32.3/44.3 multi-hop**:

| Variant | Overall (k=10/20) | Multi-hop (k=10/20) | Verdict |
|---|---|---|---|
| Baseline | 77.5 / 82.9 | 32.3 / 44.3 | — |
| Entity route (capitalized question words) | 77.5 / 82.8 | 32.3 / 44.3 | neutral |
| Second hop through entities, weight 0.5 | 71.3 / 80.4 | 25.5 / 40.8 | worse |
| Second hop through entities, weight 1.0 | 70.2 / 76.5 | 23.4 / 35.8 | worse |

The diagnosis explains why: question entities are usually just the speaker's name, which appears everywhere, and new entities from top hits "bring in other topics, not the second piece of evidence". What did help was full mode itself: 30 facts plus 20 excerpts raised multi-hop all-evidence to **51.4%**, against 44.3% for excerpts alone ([LoCoMo analysis §8](../experiments/eval/results/2026-10-14-locomo-nemotron-s20-analysis.md)). In the full run, 12 of 26 retrieval errors came from **category words in the question not matching specific names in memory** (France ↔ "a year ago in Paris", ranked 104th). Half of open-domain evidence turns share no content word with their question ([recommendations](../experiments/eval/results/2026-10-15-locomo-haiku-full-recommendations.md)). This is a hypernym gap, and an entity graph built from names does not close it. Any graph proposal has to be judged against that failure, not against graph-memory marketing.

## Four "cortex" and hindsight projects offer mechanisms, not components

**cortex-works/cortex-db has nothing to combine with.** The repository holds a README, an MIT licence and a release script. Its README describes a planned SQLite-plus-LanceDB engine for Scrum tickets and agent memory, plus a roadmap item to migrate "from JSONL to Local-First DBs", which is the opposite of rewire's direction ([cortex-db](https://github.com/cortex-works/cortex-db)). It is also not the commercial CortexDB that the earlier storage report covered. That product is a hosted LLM pipeline (events → facts → beliefs). In its integrator's open evaluation it flagged **0% of planted conflicts**, belief building added **+0 pp**, and fact extraction was **67% of model cost** ([TinyMemory cortex-flags](https://github.com/tinyhumansai/tinymemory/blob/main/docs/evals/cortex-flags.md)).

**anaslimem/CortexaDB is real but has the weaker graph.** It is a ~14k-line embedded Rust library with PyO3 bindings, MIT/Apache-2.0, and its 229 Rust tests pass ([CortexaDB](https://github.com/anaslimem/CortexaDB)). Its limits:

- **Edges** are `{to, relation: String}` with no weight, time, confidence or provenance. They are created only by a manual `connect()` call, and traversal ignores relation labels ([graph.rs](https://github.com/anaslimem/CortexaDB/blob/main/crates/cortexadb-core/src/index/graph.rs)).
- **No keyword search.** BM25 and FTS are absent everywhere.
- **Fragile query machinery.** The "intent anchors" that choose hop depth are cached by vector dimension, and through the facade they are filled with the first query vector ever seen ([executor.rs](https://github.com/anaslimem/CortexaDB/blob/main/crates/cortexadb-core/src/query/executor.rs)).
- **Unsound ANN fallback.** Without HNSW, the "ANN" path scores only the first 8 vector dimensions ([vector.rs](https://github.com/anaslimem/CortexaDB/blob/main/crates/cortexadb-core/src/index/vector.rs)).

One retrieval pattern is worth taking. CortexaDB widens the candidate pool by BFS and then **re-scores every widened candidate by its own similarity**, so a neighbour can only win by being relevant itself ([hybrid.rs](https://github.com/anaslimem/CortexaDB/blob/main/crates/cortexadb-core/src/query/hybrid.rs)). Two smaller ideas are also useful: a per-hit score breakdown with a stage trace (`VectorScored`, `GraphExpanded`, `Ranked`), and tolerant truncation of a corrupt log tail ([wal.rs](https://github.com/anaslimem/CortexaDB/blob/main/crates/cortexadb-core/src/storage/wal.rs)).

**Junebugg1214/Cortex-AI is a versioned user-profile graph with weak extraction.** It is 73.5k lines of Python with about 1.5k tests. It stores a whole JSON graph per version in SQLite. Nodes are untyped entities tagged with ~19 profile categories, and node IDs are `sha256(label)` ([graph.py](https://github.com/Junebugg1214/Cortex-AI/blob/ef05a57/cortex/graph/graph.py)). Its committed extraction baseline scores **node F1 0.25, relation F1 0.0 and contradiction recall 0/1** ([baseline.json](https://github.com/Junebugg1214/Cortex-AI/blob/ef05a57/tests/extraction/corpus/baseline.json)). In a smoke test, it stored "we no longer use MySQL" as MySQL expertise. Its dedup merges by transitive lexical union-find, which the README admits can chain-merge. Context assembly is query-independent: a profile slice capped at 1,500 characters ([hooks.py](https://github.com/Junebugg1214/Cortex-AI/blob/ef05a57/cortex/hooks.py)). Its good ideas are about governance, not retrieval:

- `retract_source`, which strips one source's provenance everywhere and prunes orphans ([graph.py](https://github.com/Junebugg1214/Cortex-AI/blob/ef05a57/cortex/graph/graph.py)).
- Five rule-based contradiction detectors (negation vs positive, overlapping validity windows, and others) that output `needs_review` rather than rewriting anything ([contradictions.py](https://github.com/Junebugg1214/Cortex-AI/blob/ef05a57/cortex/graph/contradictions.py)).
- An extraction-eval discipline: failure taxonomy, replay cache, a regression gate at 0.01, and a prompt A/B that declares a winner only when the F1 delta clears a Wilson interval ([EXTRACTION_EVAL.md](https://github.com/Junebugg1214/Cortex-AI/blob/ef05a57/docs/EXTRACTION_EVAL.md)).

**vectorize-io/hindsight is the most instructive, mostly through what it removed.** The current code has moved away from the paper's design:

- **Opinions are gone.** A 2026-04-02 migration deleted the paper's opinion network and its `confidence_score` ([migration](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/alembic/versions/g2h3i4j5k6l7_remove_opinion_fact_type.py)).
- **Entity edges are no longer stored.** A 2026-05 migration explains that on a 10k-unit bank, entity edges were **53% of all link rows (~190 MB) and "recall never touched them"**. Entities now live in a `unit_entities` join table ([migration](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/alembic/versions/e9b2c7d1f3a4_drop_entity_memory_links.py)).
- **The graph arm is one hop.** It seeds from up to 20 semantic hits and scores candidates `tanh(0.5·shared_entities) + semantic_weight + causal_weight`, with a per-entity cap of 200 ([link_expansion.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/memories/pg/link_expansion.py)).
- **Multi-hop spreading survives only in the temporal arm**, at 0.7 decay per hop over temporal and causal links ([recall.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/memories/pg/recall.py)).
- **Observations replace opinions.** They are consolidated belief nodes with `proof_count`, `source_memory_ids` and a history. They are refined rather than overwritten, and a triage step sends only facts that *change* an existing observation to an LLM, 8 facts per call ([consolidator.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/consolidation/consolidator.py); [triage.py](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-api-slim/hindsight_api/engine/consolidation/triage.py)).

The headline scores are vendor-run: LoCoMo 92.0% and LongMemEval 94.6% on v0.4.19. The vendor credits the gains to "Observations", "better retain" and reworked retrieval, not to the graph. It also warns that judge and answer prompts "can swing accuracy scores by double digits" ([AMB blog](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-docs/blog/2026-03-23-agent-memory-benchmark.mdx)). No graph-arm ablation was found. Its knowledge pages now project to markdown files on disk, "the rendering", not the storage ([knowledge-pages](https://github.com/vectorize-io/hindsight/blob/5331da2/hindsight-docs/docs/developer/knowledge-pages.mdx)). That is a convergence on rewire's model from the server side.

The wider literature points the same way. "Does Memory Need Graphs?" (ACL 2026) attributes differences to "foundational system settings rather than specific architectural innovations" and warns that poor graph construction hurts ([arXiv 2601.01280](https://arxiv.org/abs/2601.01280)). Mem0's graph variant gained ≈2.6 points on temporal questions and **lost 4.0 on multi-hop** at about 1.8× p95 latency ([Mem0 paper](https://arxiv.org/pdf/2504.19413)).

## Six steps, ordered by evidence per unit of cost

The ordering rule is simple. Steps that are free to measure and could reveal that the current graph is useless or harmful come first. Steps that touch the file format or spend LLM calls come last and are gated on earlier results. Every retrieval step lives in the derived index, behind a flag, so it can be rolled back by deleting `.index/`.

| # | Insight (source) | Change in rewire | Primary A/B metric | Kill criterion | Engineering cost (estimate) | Run cost |
|---|---|---|---|---|---|---|
| 0 | Score breakdown + stage trace (CortexaDB) | Per-hit `bm25`, `sem`, `spread`, `act`, `time` components and candidate counts in recall debug output | Diagnostic only | — | ~1 day Py + Rust | $0 |
| 1 | Measure the graph (rewire's own gap) | `spread` on/off arm in the eval runner; sweep hops 0/1/2 | All-evidence "excerpts or a fact from the turn", MRR, multi-hop + open-domain J | If off ≥ on, freeze graph work and keep spreading only for supersedes forwarding | ~0.5 day | Retrieval-only $0; J on 378 Q ≈ 5M prompt tokens |
| 2 | Re-score widened neighbours by own relevance (CortexaDB) | Neighbours enter the candidate pool but are scored by their own BM25/semantic match, with a capped graph bonus; supersedes forwarding kept | Same as 1, plus context tokens | No gain over the better of arms on/off in 1 | 2–3 days Py + Rust + parity | Same as 1 |
| 3 | Entity join table + overlap expansion (Hindsight) | Derived `entities`/`node_entities` tables from `person`/`concept` hubs, `keywords` and names; one-hop `tanh(0.5·n_shared)` from fact seeds, per-entity cap, high-df entities (speaker names) dropped | Multi-hop all-evidence; open-domain | Any multi-hop drop (repeats §8), or < +2 pp | 3–5 days Py + Rust | Same as 1 |
| 4 | Observations with provenance (Hindsight) | Hub-body summaries per person/concept carrying category words, `source` node IDs, `proof_count` derived from events; triage so only belief-changing facts reach the LLM; gated to inbox | Open-domain J, multi-hop J; PersonaMem profile categories once a safe arm exists | < +3 pp open-domain, or review-inbox volume the user would not tolerate | 1–2 weeks | New LLM calls per touched entity; re-ingest 272 sessions |
| 5 | Retraction + contradiction lint (Cortex-AI) | `rewire retract <session>` that reverts by source and re-derives; SQL lint for overlapping validity windows and active-vs-superseded conflicts, emitted as inbox items | Correctness tests, not J | — | 2–4 days | $0 |
| 6 | Significance-gated A/B (Cortex-AI) | Paired per-question comparison with a Wilson/McNemar threshold in run.py's report | — | — | ~1 day | $0 |

### Steps 0–2: instrument, ablate, then make spreading query-relevant

Step 1 should come first because it is almost free. The full-mode LoCoMo brains from the Haiku run are reused by later A/Bs ("Haiku brains from the full run", recall clock frozen) ([A/B base](../experiments/eval/results/2026-10-15-locomo-ab-A-base.md)). The runner also has a `--retrieval-only` mode that measures evidence recall and context size with no LLM at all ([run.py](../experiments/eval/run.py)). `Index.search` already takes `spread=False`, so the arm needs only a runner flag that passes it through, plus a hop-count parameter. The decisive metric is the one the full run already reports: **all evidence (excerpts or a fact from the turn)**. It stands at 87.2% overall and 56.0% multi-hop, so a graph effect shows up there before it reaches J.

If retrieval moves, rerun answer+judge on multi-hop and open-domain only (282 + 96 questions). At the measured ~13.8k prompt tokens per question for answer plus judge, that is about 5M tokens per arm. The LLM reply cache keyed by (model, prompt) makes unchanged contexts free on rerun ([run.py](../experiments/eval/run.py)). Run both engines (`--engine rust`) and `recall_parity.py`, because any change to tie-breaking breaks the parity rewire guarantees ([README](../README.md)). Report Haiku 4.5 as a second judge, as the full run already does, since a same-model judge may read high ([LoCoMo Haiku full](../experiments/eval/results/2026-10-15-locomo-haiku-full.md)).

Step 2 targets the candidate rewire's notes already name, "spreading activation only through neighbors relevant to the query, with a cap on the graph bonus" ([LoCoMo trim](../experiments/eval/results/2026-10-14-locomo-trim.md)). Today a neighbour inherits energy `amount × REL_WEIGHT × 0.5^hop / fan` whether or not it matches the query. That is exactly the mechanism by which the entity second hop "brought in other topics". CortexaDB's pattern keeps the graph as a *candidate generator* and lets relevance decide the rank. Rewire's version would be:

- neighbours join the pool;
- each neighbour is scored by its own FTS5 and semantic match against the query (and the `also` queries);
- spread energy is added only as a bonus capped at, say, the 0.15 semantic weight;
- supersedes forwarding stays untouched, because it encodes belief revision, not association.

The arms are current spreading, no spreading, and re-scored widening. Python and Rust both need the change; the rest of the cost is a parity run.

### Step 3: a derived entity layer, tested against its own failure history

Hindsight's lesson is "join table, not edges". That fits rewire's contract, because derived tables need no format change, no inbox and no new relation. The tables are `entities(id, name, df)` and `node_entities(node, entity)`, rebuilt from hub nodes, `keywords` and names in claim text. Expansion is a single `GROUP BY` self-join from the fact seeds, scored `tanh(0.5·n_shared)`. The design has to answer the §8 failure explicitly:

- **Drop or down-weight entities with high document frequency.** Speaker names are in nearly every fact.
- **Seed from facts, not excerpts.**
- **Re-score as in step 2.**

The prior is low. The same idea at excerpt level lost up to 6.8 points multi-hop, and the dominant failure is category ↔ instance, which name overlap cannot bridge. Hindsight's own expansion has no published ablation either. Set the kill criterion before running: any multi-hop regression ends the step.

Rewire's hub filter also needs a change, because today it hides only *project* hubs reached through links ([index.py](../src/rewire/index.py)). Entity hubs would otherwise show up as results.

### Step 4: observations are where a graph might actually close the measured gap

The failure that dominates the full run is vocabulary: "countries" versus "Rio de Janeiro", "France" versus "Paris". Hindsight's observation nodes are consolidated statements about an entity, written by an LLM. They naturally carry category words ("has travelled to France, Brazil…") and link back to their sources. In rewire they belong in `person`/`concept` hub bodies, which have no 600-character limit, and they can list the supporting node IDs. `proof_count` is derived from events rather than stored, and an LLM confidence score is never stored, since Hindsight deleted exactly that field. This is also the file-derived profile the earlier storage research recommended for PersonaMem's weak profile questions, whose `suggest_new_ideas` scores are 20–26% ([PersonaMem report](../experiments/eval/results/2026-10-17-personamem-haiku55.md)).

It is the costliest step, for four reasons:

- **New LLM call type.** It adds a third call type beyond rewire's current ~2 per session ([architecture.md](../docs/design/architecture.md)).
- **Inbox load.** Under today's gate, every `inferred`/`assistant` write goes to the inbox ([consolidate.py](../src/rewire/consolidate.py)). That means either a new gate class for derived summaries or a flood of review items.
- **Re-ingest.** Measuring it requires re-running the 272-session ingest, at p50 42 s per session ([LoCoMo Haiku full](../experiments/eval/results/2026-10-15-locomo-haiku-full.md)).
- **Benchmark limits.** BEAM and PersonaMem currently refuse full mode.

Hindsight's triage, which sends only facts that change an existing observation to an LLM, is the cost control. The "sleep" phase already designed in rewire, with re-summarised hubs and its report under `sessions/dreams/`, is the natural slot ([synaptic-memory.md](../docs/design/synaptic-memory.md)).

### Steps 5–6: governance and statistics, cheap and independent

Cortex-AI's retraction maps onto rewire more simply than in the original. Because the index is derived, "forget this session" means reverting the source's commit and its nodes, then re-deriving. Lint should flag rather than resolve, in line with the design rule that destructive inferred actions go to the inbox first. The lint covers overlapping `valid_from`/`valid_to` windows for the same subject, active nodes that a newer node contradicts, and orphan hubs. Neither change needs an A/B; both need tests.

Step 6 matters for all the others. A ±2-point move on 282 multi-hop questions is inside sampling noise. Base versus `also` moved strict J from 77.3% to 81.6% ([evaluation.md](../docs/evaluation.md)), and only paired per-question comparison with an explicit interval separates that from noise. Cortex-AI's Wilson-gated winner rule is the template. Rewire already has the replay cache that Cortex-AI's harness depends on.

## What not to adopt, and why

The rejections are as important as the roadmap.

- **No graph database or second store.** Not CortexaDB's bincode WAL and in-RAM maps, which are opaque, not diffable and duplicate SQLite. Not Hindsight's Postgres/pgvector. Not Kuzu or Neo4j. Rewire's design already rejects server graph DBs, notes that "a 2-hop traversal is easily handled with a recursive CTE in SQLite" ([synaptic-memory.md](../docs/design/synaptic-memory.md)), and no study in the evidence isolates the storage engine as a driver of scores.
- **No materialised entity edges in files or index.** Hindsight dropped them because they were half the link storage and unread at recall.
- **No general multi-hop spreading beyond today's two hops.** Hindsight cut its own to one hop, and Mem0's graph lost multi-hop accuracy.
- **No LLM-assigned confidence numbers or Platt calibration.** Hindsight deleted confidence. Cortex-AI's calibration constants are placeholders (slope 1.0, intercept 0.0) ([calibrate.py](https://github.com/Junebugg1214/Cortex-AI/blob/ef05a57/cortex/extraction/stages/calibrate.py)). Rewire's `explicit|inferred` plus provenance plus event counts is more honest.
- **None of Cortex-AI's identity and extraction machinery.** That means the regex extractor (relation F1 0.0), `sha256(label)` IDs that collapse two people called "Alex", transitive union-find dedup, whole-graph JSON snapshots that reimplement git, and the product surface (federation, Brainpacks, a 40-plus-tool MCP list).
- **None of CortexaDB's retrieval shortcuts.** That means the prefix-dimension "ANN", HNSW at a scale where its own docs recommend exact search, min-max recency within the candidate set, and its broken intent anchors.
- **No precomputed semantic kNN edges (Hindsight's cosine ≥ 0.7, top 20).** WordLlama vectors are far weaker than bge-small: R@5 57% against 90% for BM25 plus keywords ([index.py](../src/rewire/index.py)). The edges would also duplicate the semantic arm.
- **No new causal or temporal relations.** They would grow a vocabulary the design keeps fixed and approval-gated. Temporal questions already score 94.1% J with `valid_from` plus window boosting.
- **No Hindsight-style agentic reflect loop at read time.** It breaks rewire's zero-LLM recall.

## Conclusion

The four projects tell one story from different angles. Graph structure earns its place in agent memory as provenance and as a candidate generator, not as a ranking signal or a storage engine. Hindsight, the most benchmarked of them, has spent 2026 deleting graph weight: opinion confidence, stored entity edges, multi-hop spreading. It kept observations with source lists, and those now render to markdown. Rewire already has the parts that this convergence favours: typed, reviewable, bitemporal edges in git, and supersession that forwards activation instead of overwriting. What it lacks is evidence about its own spreading, and a way to bridge category words to instances. So the next graph work in rewire is an experiment, not a feature.

The practical implication is that the first two steps are worth doing even if the result is unwelcome. If spreading-off matches spreading-on in full mode, rewire can drop the association weights, keep only supersedes forwarding, and spend its effort on the encoder, reranker and observation work. If re-scored widening wins, rewire gets a better graph for a few days of work and no format change. Either way, LoCoMo full mode is currently the only place a graph can be measured. That makes a cheaper full-mode path for LongMemEval, or a leak-safe one for BEAM, a prerequisite for trusting any graph result beyond one benchmark.
