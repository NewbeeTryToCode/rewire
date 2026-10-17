# rewire's existing graph and memory structure (as of 2026-10-17): where a graph layer could plug in

All sources are repository files under /home/user/rewire (paths are repo-relative). Line numbers refer to the files as of 2026-10-17.

## Q1. Current schema, edge storage and how recall uses the graph (Python and Rust)

### Takeaway
rewire already has a small typed property graph. It has 7 node types and 6 relations. Edges live in each node file's frontmatter as `[[wikilink]]` lists, and a derived SQLite `edges(src, dst, rel)` table copies them. Recall runs a 2-hop spreading activation over that table, with per-relation weights, a fan effect, superseded-to-replacement forwarding and a `via` explanation. Rust mirrors all of this, and parity is tested. The graph is only reached through *facts* (L2 nodes). Episodes (L1 transcript excerpts) have no graph at all, apart from the "seed" link from a fact to its source excerpt.

### Cited Findings
**Node types and fields**
- Types: claim types `constraint, preference, fact, decision`; hub types `project, person, concept`. Relations: `about, part_of, depends_on, related_to, supersedes, contradicts`. `DECAY_EXEMPT = constraint, preference`. `CONFIDENCE = explicit|inferred`. `PROVENANCE = user|assistant|tool`. — [src/rewire/schema.py](src/rewire/schema.py) L3-9
- Required fields: `id, type, title, scope, importance, confidence, provenance, recorded_at`. A claim body is at most 600 chars. `keywords` is an optional list of strings. Every edge must have `rel` in RELATIONS and a non-empty `to`. — [src/rewire/schema.py](src/rewire/schema.py) L11-42
- The frozen key order is `id, type, title, status, aliases, tags, scope, importance, confidence, provenance, valid_from, valid_to, recorded_at, retracted_at, source, keywords`, followed by relation keys. `valid_from/valid_to` are `YYYY-MM-DD` (world time). `recorded_at/retracted_at` are ISO timestamps (belief time). `source` is `"<harness> · <session id> · turn <n>"`. `status` is derived: `superseded` if `valid_to` is set, else `retracted` if `retracted_at` is set, else `active`. — [docs/design/format.md](docs/design/format.md) L44-80
- Relation keys hold `"[[<target file stem>]]"` lists, and a generated `## Related` section mirrors them as relative markdown links. Internally `meta["edges"] = [{"rel","to": <last 8 id chars>}]`. Legacy `edges:` lists are still accepted on read. — [src/rewire/nodefmt.py](src/rewire/nodefmt.py) docstring and `parse()`/`render()`; [docs/design/format.md](docs/design/format.md)
- `ensure_project_hub` creates or reuses a `project` hub node per scope. `apply_change` adds an `about` edge to it for every project-scoped node. — [src/rewire/ops.py](src/rewire/ops.py) L150-173
- SUPERSEDE and CONTRADICT both create a *new* node with a `supersedes` or `contradicts` edge to the target. The old node gets `retracted_at` either way, and `valid_to` only for SUPERSEDE. A contradicted node therefore also becomes inactive (status `retracted`). — [src/rewire/ops.py](src/rewire/ops.py) L184-196
- ACT-R base-level activation `B = ln Σ w_j (now - t_j)^-d` is computed from `events/<device>/*.jsonl` `use`/`confirm` events, capped at one event per node per session. Node creation counts as the first use. — [src/rewire/activation.py](src/rewire/activation.py)

**Index (derived SQLite, schema "9")**
- DDL: `nodes(id, type, title, body, scope, importance, active, recorded_at, path, meta JSON, short)`; `edges(src TEXT, dst TEXT, rel TEXT)` with src = full id and dst = 8-char suffix, indexed both ways; `act(id, b)`; `fts(id, title, body, keywords)`; `episodes(path, date, text, first, last, scope)` + `fts_ep`; `vec`, `epvec`. — [src/rewire/index.py](src/rewire/index.py) L31-59
- Any schema-version mismatch deletes and rebuilds the DB, so the index is fully derived. Edges are re-inserted from frontmatter on every node re-parse. — [src/rewire/index.py](src/rewire/index.py) L111-131, L245-247
- `neighbors()` is bidirectional: outgoing edges are joined via `nodes.short = e.dst`, and incoming edges come from `e.dst = short`. — [src/rewire/index.py](src/rewire/index.py) L623-629

**Spreading activation in `Index.search`**
- `REL_WEIGHT = part_of 0.8, depends_on 0.8, related_to 0.6, about 0.5, contradicts 0.4, supersedes 0.2`; `HOP_DECAY = 0.5`; any other relation gets 0.3. — [src/rewire/index.py](src/rewire/index.py) L33-35, L743
- Algorithm: seeds are the top 10 of BM25 (+ semantic bonus 0.15 + time-window bonus). There are 2 hops. `fan = 1 + ln(1 + degree)` and `gain = amount * REL_WEIGHT * 0.5^hop / fan`. Gains below 0.02 are dropped. An inactive source with a `supersedes` link forwards 0.9 of its amount to its replacement. Each hop keeps the top 10 as the next frontier. `via` records `"<rel> ↔ <src title>"` or `"replaces outdated: <title>"` for nodes not in the text hits. — [src/rewire/index.py](src/rewire/index.py) L727-753
- Final score is `energy * (1 + beta * squash(B)) * (1.0 active | 0.6 superseded)`. Superseded nodes are returned only for history-intent queries or with `include_inactive`. Project hubs reached only through links are excluded. — [src/rewire/index.py](src/rewire/index.py) L754-771
- The judge's candidate search uses `spread=False, beta=0.0`, so consolidation dedup ignores the graph. — [src/rewire/consolidate.py](src/rewire/consolidate.py) L502
- Episodes link to facts only through `_seed_episodes`: a fact's `source` (harness, session, turn) boosts the excerpt containing that turn by `SEED_WEIGHT 0.35`. — [src/rewire/index.py](src/rewire/index.py) L518-540
- `ops.recall` runs `index.search` (facts, with spreading) and `index.search_episodes` (seeded by those facts) for each of the query and up to 4 `also` queries, then fuses them by RRF. — [src/rewire/ops.py](src/rewire/ops.py) L41-83

**Rust equivalents and parity**
- `rust/src/schema.rs` has the same TYPES and RELATIONS arrays (L6-9). `rust/src/index.rs` has `SCHEMA = "9"` (L23), `HOP_DECAY = 0.5` (L24), a `rel_weight` match including `"supersedes" => 0.2` (~L37), the same `CREATE TABLE edges` (L47), `fn neighbors` (L1042), and the spreading loop with `fan = 1 + ln(1+len)` and supersedes forwarding (L1165-1196), plus a `via` field on hits (L172, L200, L1231). — [rust/src/index.rs](rust/src/index.rs), [rust/src/schema.rs](rust/src/schema.rs)
- The README says that node files are byte-for-byte identical and recall results identical across Python and Rust, tested by `experiments/conformance/recall_parity.py`. — [README.md](README.md) L22, L82
- Parity was run on conv-26 and conv-30 (233 questions, k=50, 10 excerpts): 233/233 identical. — [experiments/eval/results/2026-10-14-locomo-trim.md](experiments/eval/results/2026-10-14-locomo-trim.md) §3
- architecture.md: "The on-disk format (format.md) is the contract. Any implementation … must read and write it identically; the index is not part of the contract." — [docs/design/architecture.md](docs/design/architecture.md) Boundaries
- Tests cover depends_on links from consolidation, legacy edge upgrade, superseded status, superseded-only-for-history, and a `via` containing `depends_on` vs `spread=False`. — [tests/test_rewire.py](tests/test_rewire.py) L474-475, L620-645, L844-873

**Sessions (L1)**
- `sessions/<date>/<harness>-<session>.md` holds a header (date, project), `## Summary`, `## Applied`, `## Queued for review`, `## Questions` (hypothetical questions per turn, indexed and never shown) and `## Transcript`. The index chunks it into 6-turn windows with stride 4. — [src/rewire/consolidate.py](src/rewire/consolidate.py) L530-551; [src/rewire/index.py](src/rewire/index.py) L255-302

### Inferences
- The `edges` table already uses an open string `rel` column, and `REL_WEIGHT.get(rel, 0.3)` has a default. A new relation (e.g. `mentions`, `before`/`after`) therefore needs no DDL change in the index. It does need changes to `schema.RELATIONS` (Python and Rust), format.md, and the `nodefmt` relation-key parsing (which iterates RELATIONS), so that it round-trips through files.
- Entity nodes could reuse the existing hub types `person`/`concept`. Hubs reached only via links are already filtered for `project` only, not for `person`/`concept`, so entity hubs would show up as results unless that filter is extended.
- The fan effect `1 + ln(1+degree)` is mild. A `person` hub with hundreds of `about`/`mentions` edges would still pass `amount*0.5*0.5/ (1+ln(N))` ≈ 0.04 per neighbor at N=300, above the 0.02 cutoff, but the per-hop top-10 frontier limits the blow-up.
- Because dst is stored as an 8-char suffix and joined on `nodes.short`, edges to non-node targets (e.g. episodes `ep:<rowid>` or session files) cannot be expressed without a new table or id scheme.

### Gaps
- I did not read the Rust spreading code line by line beyond the grep hits, so exact equivalence of thresholds (0.02, top-10 frontier) in Rust is assumed from the parity tests, not verified by reading.
- No measurement exists of graph density (edges per node, share of `related_to` vs `about`) in any benchmark brain.

## Q2. Consolidation pipeline: extraction → judge → gate → inbox, and the cost of an extra step

### Takeaway
Consolidation is the only LLM stage. It makes one extraction call per transcript chunk and one judge call per session (skipped when no candidates exist). Links (`part_of/depends_on/related_to`) are already produced by the extraction call at no extra LLM cost. An "entity" or "belief" step would be a third call type or an extension of the extraction schema, and its writes would have to pass the same gate and inbox.

### Cited Findings
- Pipeline: "normalize -> filter -> extract (LLM) -> resolve (FTS + LLM judge) -> gate -> scan -> write + git commit -> advance watermark", idempotent per (harness, session, content hash). — [src/rewire/consolidate.py](src/rewire/consolidate.py) L1-8
- Extraction makes one `complete_json` call per chunk of `max_chunk_chars`. The schema returns `summary`, `claims[]` (text, title, type, scope, importance, confidence, evidence_turn, keywords, `links[{rel∈part_of|depends_on|related_to, to}]`), `used_memory_ids`, `questions[]`, and `skills` for the coding profile. — [src/rewire/consolidate.py](src/rewire/consolidate.py) L27, L142-190, L370-389
- The "general" profile tells the model to extract `person / concept: people, pets, groups or things that are introduced and matter` and to "Name the person in every claim". — [src/rewire/consolidate.py](src/rewire/consolidate.py) L57-66
- The timeline rule says: "Link a finish or change to the claim about its start with related_to". Temporal links are therefore currently encoded as `related_to`. — [src/rewire/consolidate.py](src/rewire/consolidate.py) L76-79
- Judge: one call per session over all claims, each with up to 5 FTS candidates (`spread=False`). Actions are NEW/SAME/UPDATE/SUPERSEDE/CONTRADICT. It is skipped if no claim has any candidate. — [src/rewire/consolidate.py](src/rewire/consolidate.py) L192-221, L498-518
- Gate: auto-apply only if `confidence == explicit and provenance == user and action in (NEW, SAME, UPDATE) and type != constraint and no flags`. Everything else goes to `inbox/`. — [src/rewire/consolidate.py](src/rewire/consolidate.py) L271-275
- Links are resolved after every claim has a node, so claims can link to each other (`claim:<n>`). They are written back into node frontmatter. Inbox items carry resolved `edges`. — [src/rewire/consolidate.py](src/rewire/consolidate.py) L417-444
- Provenance comes from transcript roles, never from the model. — [src/rewire/consolidate.py](src/rewire/consolidate.py) L484-489
- Approve re-checks a pinned `target_hash` and refuses if the target note changed since the proposal. Approved items become `confidence: explicit`. — [src/rewire/ops.py](src/rewire/ops.py) L204-236
- Each consolidation is one git commit `consolidate <harness>/<session>`, and `undo` reverts it. — [src/rewire/consolidate.py](src/rewire/consolidate.py) L474; [src/rewire/ops.py](src/rewire/ops.py) L246-255
- Token cost: recall/prime/get use 0 LLM calls. Consolidation uses "~2 per session (extract per transcript chunk + one judge)". — [docs/design/architecture.md](docs/design/architecture.md) Token cost profile
- Measured ingest on full LoCoMo (Haiku 5.5): 272 sessions, 15.8 facts/session (4,120 applied, 191 queued then approved), 3,345 hypothetical questions, p50 42 s/session. Active nodes: fact 2,401, preference 553, decision 449, **person 35, concept 8**. — [experiments/eval/results/2026-10-15-locomo-haiku-full.md](experiments/eval/results/2026-10-15-locomo-haiku-full.md) "Ingest (mode full)"
- In benchmarks there is no human reviewer: `LOCOMO.approve_all(root)` accepts proposals in bulk. — [experiments/eval/run.py](experiments/eval/run.py) ~L261

### Inferences
- **Cheapest entity layer (0 extra calls):** add an `entities`/`about_entity` field to the existing extraction schema, or a rule-based extractor at index time (derived, so no format change). Entity *resolution* across sessions would piggyback on the judge prompt (add entity candidates) or use deterministic matching.
- **Belief/observation summaries** (per-entity "what we know about X") would be a new call type, probably at most 1 per touched entity per session, or batched in a periodic job. They would need to be written as files (e.g. `person`/`concept` hub bodies, which have no 600-char limit) and gated: they are `inferred`/`assistant`, so the current gate would send every one to the inbox.
- The gate as written would route all LLM-derived graph structure (non-user provenance) to the inbox. This could flood human review unless derived structure is kept in the index (not files) or a separate gate class is defined.
- The extraction prompt yields few hub nodes (35 person + 8 concept across 10 LoCoMo conversations), so today's graph is mostly claim-to-claim `related_to/part_of/depends_on` plus `about` to project hubs, not an entity graph.

### Gaps
- No repository data on how many `links` per claim extraction actually produces, or the distribution of edge types in benchmark brains.
- The Rust `consolidate.rs` (843 lines) was not read; its LLM-call structure is assumed to match from the README parity claim.

## Q3. Evaluation evidence: what runs exercised the graph, entity/second-hop experiments, full-mode results, what is unmeasured

### Takeaway
Most result files (48 of about 60 with a mode header) are `--mode episodes`. That mode stores only transcripts and has no fact nodes and hence no graph. The graph was exercised only in the `--mode full` LoCoMo runs (1,540-question full run at J 93.0%, plus A/B samples). There it is never ablated. Entity routes and entity second-hop over *excerpts* were neutral or worse. Facts plus excerpts (full mode) helped. Graph-specific levers (query-relevant spreading, PPR, community summaries) remain untested.

### Cited Findings
**Which mode each run used**
- `--mode episodes` = "transcripts only (no LLM ingest)"; `full` = "real consolidation through --ingest-model". BEAM and PersonaMem refuse `full` mode (cost; "facts could leak turns after the question"). — [experiments/eval/run.py](experiments/eval/run.py) L993-994, L674-675, L735-736
- Result headers: `mode full` appears only in 2026-10-15-locomo-ab-A-base, ab-B-also, ab80-A, ab80-B and haiku-full. All 2026-10-16 LongMemEval files and all 2026-10-17 A/B files (base/A/B/C/C2/C2rs/C3rs/C4rs/D across LoCoMo, LME, BEAM, PersonaMem) and PersonaMem runs are `mode episodes`. — header lines of files in [experiments/eval/results/](experiments/eval/results/)
- "`episodes` mode has no graph": the candidate "spreading activation only through neighbors relevant to the query, with a cap on the graph bonus" "needs `full` mode (facts) to measure". — [experiments/eval/results/2026-10-14-locomo-trim.md](experiments/eval/results/2026-10-14-locomo-trim.md) "Other candidates"
- The four-benchmark A/B of 2026-10-17 used `--mode episodes`, k=30, 10 excerpts. Overall over 190 questions: base 70.2%, A 70.9%, B 67.3%. — [experiments/eval/results/2026-10-17-ab-four-benchmarks.md](experiments/eval/results/2026-10-17-ab-four-benchmarks.md)

**Entity / second-hop experiments (all on excerpts, no LLM, 2026-10-14)**
- Baseline all-evidence k=10/20: overall 77.5/82.9, multi-hop 32.3/44.3. Entity route (capitalized question words, df ≤5%) in RRF: 77.5/82.8, 32.3/44.3 (**neutral**). Second hop through entities from top-5 excerpts, weight 0.5: 71.3/80.4, multi-hop 25.5/40.8 (**worse**); weight 1.0: 70.2/76.5, 23.4/35.8 (**worse**). Max 1 or 2 excerpts per session were also worse. — [experiments/eval/results/2026-10-14-locomo-nemotron-s20-analysis.md](experiments/eval/results/2026-10-14-locomo-nemotron-s20-analysis.md) §8
- Stated reason: failing multi-hop questions are aggregation questions with vocabulary unlike their evidence ("countries" ↔ "Rio de Janeiro"). The entities in the question are usually just the speaker's name, which is in almost every excerpt, and new entities from top results "bring in other topics, not the second piece of evidence". — same, §8
- What helped: full mode, 30 facts + 20 excerpts (4.7k ctx): multi-hop 51.4%, overall 86.4%, vs episodes 20 excerpts 44.3%/82.9%. "100 facts only" gave 66.4% overall. Conclusion: "none of the entity-based variants was adopted." — same, §8
- The idea map lists: "Lightweight entity graph: entity route / entity hop | neutral / worse (§8)"; "Relation graph + spreading activation (`full` mode) | already present"; "Not yet tested: cross-encoder (model unreachable), Personalized PageRank, GraphRAG community summaries". Graph RAG/LightRAG/GraphRAG are "partial: relations + spreading activation; entity hop tested: worse (§8)". — same, L240, L305-309
- evaluation.md §10 says multi-hop in `episodes` mode is 32–44% all-evidence. "Already tried, and did not help: 1-turn units, line trimming, pseudo-relevance feedback, an entity path, a second hop via entities, a per-session excerpt limit. What helped: `full` mode." — [docs/evaluation.md](docs/evaluation.md) §10

**Full-mode LoCoMo**
- 1,540 questions, k=30, episodes=20, Haiku 5.5 for ingest, answer and judge: J overall 93.0% (94.8% without disputed gold); multi-hop 92.2%, open-domain 66.7%, single-hop 95.8%, temporal 94.1%. All-evidence (excerpts) 84.5%, multi-hop 47.9%. "Excerpts or a fact from the turn" gives multi-hop 56.0%, overall 87.2%. Context ≈5.0k tokens. Retrieval p50 9 ms. — [experiments/eval/results/2026-10-15-locomo-haiku-full.md](experiments/eval/results/2026-10-15-locomo-haiku-full.md)
- Caveat: same model as answerer and judge; "J may read higher than with an independent judge". — same
- Diagnosis: 26 retrieval errors, of which "category words in the question do not match the specific names in memory (12), evidence just missed the k cutoff (9), and paraphrase or spelling (5)". There are 11 ingest errors (relative durations, photo captions, nicknames). — [experiments/eval/results/2026-10-15-locomo-haiku-full-recommendations.md](experiments/eval/results/2026-10-15-locomo-haiku-full-recommendations.md) L6
- "51% of OD evidence turns share no content word with the question. The figure is 38% for multi-hop, 19% for temporal, and 15% for single-hop." — same, L87
- Example of a missed link: conv-48-19 "How many times has Jolene been to France?": "a year ago in Paris" is not linked to "France", rank 104. — same, L272
- "Retrieval is only BM25 and graph spreading, with excerpts seeded from facts. There is no reranker." — same, L302
- Full-mode `also` A/B (multi-hop, Haiku): strict J 77.3% → 81.6%, all evidence 56% → 71%. — [docs/evaluation.md](docs/evaluation.md) runbook notes (~L429)
- Earlier full LoCoMo: 94.2% hybrid, 94.9% keywords only. "semantics does not help on LoCoMo … because episodes + keywords already cover it". — [docs/iterations.md](docs/iterations.md) L61
- Iteration 3 introduced spreading activation because "The relations table existed but was not used by recall". — [docs/iterations.md](docs/iterations.md) L46

### Inferences
- No result in the repository isolates the contribution of spreading activation. There is no `spread=False` ablation in full mode, so the graph's current value on benchmarks is unknown, not shown to be zero.
- The failing multi-hop cases (category ↔ instance, e.g. France ↔ Paris, desserts ↔ ice cream) are a semantic/hypernym gap. An entity graph built from capitalized names would not close it, which is consistent with §8. A graph whose nodes are *categories/concepts* with `part_of`/instance edges, or entity summaries carrying category words, targets the observed failure more directly. This is untested.
- Temporal performance is already high in full mode (94.1% J, 91.6% all-evidence). Temporal edges would have to beat `valid_from` + `time_window` boosting, which already exist.

### Gaps
- Unmeasured: spreading on/off in full mode; edge density; query-relevant spreading; Personalized PageRank; GraphRAG-style community/entity summaries; LongMemEval/BEAM/PersonaMem in full mode (BEAM and PersonaMem are blocked by the runner by design); an independent judge for the full-mode run.
- Why ~865 of the 4,311 written nodes are not active (superseded/contradicted vs UPDATE/SAME bookkeeping) cannot be determined from the report.

## Q4. Design-doc constraints: allowed and rejected graph features

### Takeaway
The design explicitly wants a typed graph with a small, fixed, inbox-gated relation vocabulary. The files and git are the source of truth. The SQLite index is derived, and graph DBs are rejected as primary storage. Hebbian/associative edges and an automatic "sleep" (reflection/lint/merge) were designed but deferred for lack of evidence. None is implemented: the CLI has no sleep, lint or reflect command.

### Cited Findings
- The L2 semantic layer is "atomic, typed claims … linked in a graph; bitemporal; superseded instead of overwritten; strength from use (ACT-R activation)". `.index/` "is derived from L1 and L2 and can be deleted at any time". — [docs/design/architecture.md](docs/design/architecture.md)
- The original concept had 8 types (incl. `procedure`, `artifact`) and a ~12-relation vocabulary (`about, part_of, depends_on, decided_in, prefers, works_on, knows, uses, supersedes, contradicts, derived_from, related_to`). "Consolidation **may propose** new types/relations, but they go into `inbox/` for approval. This prevents *ontology sprawl*". It rejects full OWL/RDF and keeps "typed nodes + typed edges + a few constraints (e.g. `supersedes` only between nodes of the same type)". — [docs/design/synaptic-memory.md](docs/design/synaptic-memory.md) §2.1-2.2
- Two edge classes: semantic edges (in frontmatter, curated, in git) and associative "synapse" edges, which are "**Not stored** — computed from the event log" (Hebbian co-use). — [docs/design/synaptic-memory.md](docs/design/synaptic-memory.md) §2.3, §3.2
- Wikilinks in the body "are treated as low-prior `related_to` edges" (design intent). — [docs/design/synaptic-memory.md](docs/design/synaptic-memory.md) L226
- Storage: "Markdown + JSONL + SQLite index ✅ Recommended". Graph DB server (Neo4j, Memgraph) "❌ conflicts with 'own the brain'". Embedded graph DB (Kuzu) "⚠️ could be an optional index backend". RDF/OWL "❌ for v1". "A 2-hop traversal is easily handled with a recursive CTE in SQLite." — [docs/design/synaptic-memory.md](docs/design/synaptic-memory.md) L248-257
- The designed "SLEEP" (periodic) phase covers recomputing activation, incremental lint (changed nodes + 1-hop neighbors: contradictions, orphans, missing links, stale claims), merging duplicates, pruning weak associative edges, re-summarizing hub nodes, ontology proposals to the inbox, and a report to `sessions/dreams/<date>.md` "for human review". Also designed: entity resolution "FTS + embedding + LLM judge" at consolidation. — [docs/design/synaptic-memory.md](docs/design/synaptic-memory.md) L281-300
- Safety rule: destructive actions (SUPERSEDE, merging nodes, importance ≥4 changes) from an `inferred` source go to `inbox/` first. — [docs/design/synaptic-memory.md](docs/design/synaptic-memory.md) L302
- Risk table: "Duplicate nodes (entity resolution fails)" → mitigation "`aliases`, staged resolution, merging during 'sleep' (old events are moved to the winning node)". "Ontology sprawl" → "Fixed vocabulary + approval". — [docs/design/synaptic-memory.md](docs/design/synaptic-memory.md) §6
- The MVP defers "Hebbian / associative edges (no evidence of benefit yet)", the "Full ontology of 8 types + 12 relations", "Scheduled automatic 'sleep', lint", and "Exploration tools (Ori removed similar tools because they were not used)". — [docs/design/mvp-spec.md](docs/design/mvp-spec.md) §1
- design-review: "co-use = used in the same session" is "too coarse … can produce spurious edges". The size of a node "affects extraction, entity resolution, and activation". SYNAPSE (lateral inhibition + fan effect) is noted as "a direct upgrade for §3.3". HeLa-Mem (Hebbian distillation of episodic hubs into semantic knowledge) "Fits the 'sleep' phase". Tarjan articulation points to protect structural nodes from archiving. — [docs/design/design-review.md](docs/design/design-review.md) L14-15, L56-57, L88-89
- Boundaries: rewire never manages the prompt; "Memory is data, not instructions". Recall output is re-scanned for secrets and injection at show time. — [docs/design/architecture.md](docs/design/architecture.md); [src/rewire/ops.py](src/rewire/ops.py) L74-82
- No `sleep`/`lint`/`dream`/`reflect` command exists in [src/rewire/cli.py](src/rewire/cli.py) or [rust/src/cli.rs](rust/src/cli.rs) (grep found no matches).
- `write_home` is deterministic ("no timestamps, no activation-based order") to avoid sync conflicts. — [src/rewire/ops.py](src/rewire/ops.py) L311-316
- `events/**/*.jsonl merge=union` in `.gitattributes`. `.index/` and `.state/` are git-ignored. — [src/rewire/brain.py](src/rewire/brain.py) L19-20

### Inferences: concrete integration points and constraints
1. **Entity nodes/edges.** (a) *Derived-only* option: extract entities at index time (rule-based or from `keywords`) into new index tables (`entities`, `node_entities`). No format change, no inbox, and both Python and Rust must implement the same deterministic extractor for recall parity. (b) *File-backed* option: reuse `person`/`concept` hubs plus a relation (`about` already exists; a new `mentions` needs RELATIONS + format.md + Rust). Hubs would be created like `ensure_project_hub` and gated through the inbox for `inferred` sources.
2. **Temporal edges.** `valid_from/valid_to/recorded_at/retracted_at` already give bitemporality, and `time_window` boosts by `valid_from`. Explicit `before/after/started/ended` relations would extend the vocabulary that the design says is fixed and approval-gated. A derived alternative is to compute temporal order in the index from `valid_from` within an entity.
3. **Belief/observation summaries.** The natural home is hub-node bodies (no 600-char limit for hub types) or the designed `sessions/dreams/` report. They cost new LLM calls and, under the current gate, go to the inbox.
4. **Graph traversal in recall.** The hook is `Index.search` L729-753 (Python) and `rust/src/index.rs` ~L1165-1196. The parameters are REL_WEIGHT, HOP_DECAY, the frontier size and the 0.02 cutoff. `search_episodes` has no graph. Extending traversal to excerpts would need fact→episode edges, which today exist only implicitly via `source` (the `_seed_episodes` hook). Any change must keep the deterministic tie-breaks (`ORDER BY …, id`) for cross-implementation parity.
5. **Background reflection ("sleep").** It is designed but unimplemented. A natural slot is after `_consolidate` or as a new CLI command under the existing brain lock (`brain.lock()`), committing once and writing proposals to `inbox/`. The incremental-lint design (changed nodes + 1-hop) bounds its cost.
6. **Evaluation constraint.** Any graph change needs `--mode full` runs to be measurable (episodes mode has no graph), and only LoCoMo supports full mode in the runner. The full LoCoMo ingest costs about 272 sessions × 2+ LLM calls.

### Gaps
- No design doc covers how derived (index-only) graph structure should be surfaced to human review, or whether derived structure may influence recall without being in files. The principle "index is derived, not part of the contract" implies it may, but this is never stated for graph data.
- No doc specifies a fan-effect or lateral-inhibition upgrade beyond naming SYNAPSE as a candidate.
