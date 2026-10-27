# Cortex-AI (Junebugg1214/Cortex-AI): code-grounded review for rewire

Source: local clone /home/user/junebugg1214/cortex-ai, commit ef05a57 (2026-05-15). Citation base URL: `https://github.com/Junebugg1214/Cortex-AI/blob/ef05a57/` + path (abbreviated below as `GH/<path>`). Unless marked "README claim", each item below was checked against source code. I also ran the CLI heuristic extractor once on a throwaway copy in scratch, and the clone was left unmodified. The test suite was not run because pytest is not installed in this environment.

## 1. What it is, language, license, maturity, dependencies

### Takeaway
Cortex-AI is a Python (>=3.10) CLI, library, local HTTP/ASGI server and MCP server, MIT-licensed, and packaged as `cortex-identity` v1.6.0. It calls itself "Git for AI memory". It stores a JSON property graph with commit, branch, merge and retraction semantics. The codebase is large (about 73.5k LOC of Python) and has CI, about 1.5k tests and a coverage gate. The shallow clone has only one commit and the extraction-quality evidence is very thin, so maturity is mixed: the engineering is broad but the memory quality is unproven.

### Cited Findings
- README positioning: "Git for AI memory. Local-first, MCP-native." It is described as "a local memory layer for AI agents" with commits, branches, merge, blame and source-backed retraction, and an MCP server. The README says there is no Cortex cloud or API key and that model extraction is opt-in with the user's own key (README claim). — [README.md](GH/README.md)
- Package `cortex-identity` version 1.6.0, license MIT, `requires-python >=3.10`, classifier "Development Status :: 4 - Beta". The only hard runtime dependency is `tomli` on Python <3.11. Optional extras: `asgi` (starlette, uvicorn), `crypto` (pynacl), `embeddings` (sentence-transformers), `fast` (numpy), `model` (pydantic). Console scripts: `cortex`, `cortex-hook`, `cortexd`, `cortex-mcp`, `cortex-bench`. — [pyproject.toml](GH/pyproject.toml); [LICENSE](GH/LICENSE) (MIT, "Copyright (c) 2025 Junebugg1214")
- Size: `find cortex -name '*.py' | xargs wc -l` reports 73,504 lines in total. The largest files are `cortex/mcp/mcp_tools.py` (1,893), `cortex/service/webapp_shell_js.py` (1,787), `cortex/extraction/heuristic_rules.py` (1,697), `cortex/service/openapi.py` (1,561) and `cortex/extraction/model_backend.py` (1,354). The project also ships a web UI, a TypeScript SDK (`sdk/typescript`) and an OpenClaw plugin (`examples/openclaw-plugin`). — local wc over [cortex/](GH/cortex)
- Tests: 121 test `.py` files with about 1,556 `def test_` functions (grep count). CI runs ruff lint and format checks, pytest, coverage with `--cov-fail-under=75`, bandit, pip-audit, a CycloneDX SBOM and sigstore signing of the wheel. — [.github/workflows/ci.yml](GH/.github/workflows/ci.yml)
- Activity: the local clone is shallow with a single commit, `ef05a57 2026-05-15 "fix: write absolute Cortex MCP config path for Hermes"`, so commit cadence cannot be measured locally. The CHANGELOG lists PRs #232–#251 under Unreleased and a "v1.4.1-beta — 2026-03-26" first public beta. — [CHANGELOG.md](GH/CHANGELOG.md)
- Self-assessed status: the README describes the project as "production-fit for disciplined local-first and small-team self-hosted workflows… not a default LLM extractor". The README names its strongest core as versioned graph storage, branch/merge/review, source-backed retraction, tool mounts, audience filtering and MCP (README claim). — [README.md](GH/README.md)
- Many root-level modules (e.g. `cortex/dedup.py`, `cortex/claims.py`, `cortex/merge.py`) are compatibility shims for subpackages, added in PR #239. — [CHANGELOG.md](GH/CHANGELOG.md); directory listing of [cortex/](GH/cortex)

### Inferences
- The project has broad surface area: versioning, federation, signed remotes, a web UI, SDKs, Brainpacks, "Minds", audience policies, channel runtimes for Hermes and OpenClaw, and an agent monitor. Measured extraction quality is very low next to that (see section 4). It reads as an infrastructure-first project whose memory "intelligence" lags its plumbing.
- The single-commit clone means I could not verify contributor count or commit velocity. Treat "maturity" claims with caution.

### Gaps
- I could not see GitHub stars, issues, contributors or release cadence as of October 2026 because only the local shallow clone was used.
- The test pass rate was not verified because pytest is not installed here.

## 2. Graph model: nodes, edges, extraction, entity resolution, temporal/confidence fields, contradictions, export

### Takeaway
The graph is a single in-memory `CortexGraph` (dicts of `Node` and `Edge`) that is serialized as a whole JSON document ("v5"). Nodes are untyped entities. They carry free-form tags drawn from roughly 19 profile-style categories, plus confidence, mention count, timestamps (`first_seen`/`last_seen`, `valid_from`/`valid_to`, `status`), aliases, a `canonical_id`, per-source `provenance` and embedded `snapshots`. Edges are `(source, target, relation)` with confidence, qualifiers and provenance. Extraction defaults to large regex/heuristic rules. An opt-in Anthropic tool-call pipeline exists. Entity resolution is lexical (SequenceMatcher, Jaccard and neighbor overlap) with transitive union-find merging. Contradictions are detected by rule-based detectors and flagged for review, not auto-resolved.

### Cited Findings
**Node and edge schema**
- `Node` fields: `id, label, tags, aliases, confidence(0.5), properties, brief, full_description, mention_count, extraction_method, metrics, timeline, source_quotes, first_seen, last_seen, valid_from, valid_to, status, canonical_id, provenance: list[dict], relationship_type, snapshots: list[dict]`. All text fields pass through an `InputValidator` with length caps (for example label ≤512 and full_description ≤50,000). — [cortex/graph/graph.py](GH/cortex/graph/graph.py) (class `Node`, ~L210–330)
- `Edge` fields: `id, source_id, target_id, relation, confidence, properties, qualifiers, provenance, first_seen, last_seen`. — [cortex/graph/graph.py](GH/cortex/graph/graph.py) (class `Edge`, ~L330–380)
- IDs are deterministic. A node id is `sha256(normalized_label)[:16]` (optionally with the tag appended to the hash input), and an edge id is `sha256(f"{src}:{tgt}:{relation}")[:16]`. — [cortex/graph/graph.py](GH/cortex/graph/graph.py) (`make_node_id`, `make_node_id_with_tag`, `make_edge_id`)
- There is no node-type enum. "Type" is expressed through tags from `CATEGORY_ORDER`: identity, professional_context, business_context, active_priorities, work_history, education_history, relationships, technical_expertise, domain_knowledge, market_context, metrics, constraints, values, negations, user_preferences, communication_preferences, correction_history, history, mentions. — [cortex/graph/graph.py](GH/cortex/graph/graph.py) L25–45
- `CortexGraph` holds `nodes: dict`, `edges: dict`, `schema_version "5.0"` and `meta` (for example `meta["retractions"]`). It has a lazily built adjacency cache, BFS `shortest_path`, `k_hop_neighborhood` and centrality boosting. — [cortex/graph/graph.py](GH/cortex/graph/graph.py)

**Extraction**
- The public contract is `ExtractionPipeline.run(Document, ExtractionContext) -> ExtractionResult` with typed `ExtractedFact` (attribute_name/value), `ExtractedClaim` (assertion and stance, default "asserts") and `ExtractedRelationship` (source_label defaults to "self", relation defaults to "related_to", target_label, qualifiers). — [docs/EXTRACTION.md](GH/docs/EXTRACTION.md); [cortex/extraction/extract_memory_context.py](GH/cortex/extraction/extract_memory_context.py) L211–278
- The default backend is heuristic: regex and pattern rules in `heuristic_rules.py` (1,697 LOC). README: "Default extraction is local and rule-based." — [README.md](GH/README.md); [cortex/extraction/heuristic_rules.py](GH/cortex/extraction/heuristic_rules.py)
- The heuristic confidence model is fixed tables. `BASE_CONFIDENCE` maps explicit_statement 0.85, self_reference 0.8, direct_description 0.75, contextual 0.6, mentioned 0.4 and inferred 0.3. `MENTION_COUNT_BOOST` runs up to +0.3 at 20 mentions. `TIME_DECAY` multipliers are 1.0 (≤7d), 0.9 (≤30d), 0.7 (≤90d), 0.5 (≤180d), 0.3 (≤365d) and 0.1 beyond. — [cortex/extraction/extract_memory_context.py](GH/cortex/extraction/extract_memory_context.py) L11–31
- The model backend is a staged pipeline over immutable `PipelineState`: `split_document` (chunking aware of source type: chat turns, markdown headings, code symbols, transcript timestamps) → `generate_candidates` (schema-constrained tool call; Anthropic is the built-in default provider) → `refine_types` (second pass for low-confidence items) → `link_to_graph` → `link_relations` (drops dangling edges) → `calibrate_confidence` → `contradictions.detect`. On schema-validation failure it retries up to 2 times with the error fed back, then returns zero items plus a `schema_violation` warning. — [docs/EXTRACTION.md](GH/docs/EXTRACTION.md); [cortex/extraction/stages/](GH/cortex/extraction/stages)
- The prompts are short. `candidates.v1.md` says: "You are a knowledge graph extractor. Call the extraction tool exactly once… Do not invent facts… Use fact for stable attributes… claim for user assertions, denials, corrections… If relationship direction is ambiguous, return both directions… with confidence below 0.6. confidence reflects how clearly the fact is stated, not how likely it is to be true." `canonicalize.v1.md` asks the model to reuse existing node IDs from retrieval hints via `entity_resolution`. Each prompt has YAML front matter (version, schema_ref, inputs, outputs, test_fixture). — [cortex/extraction/prompts/](GH/cortex/extraction/prompts)
- Confidence "calibration" is a no-op today. `_BOOTSTRAP_PLATT_SLOPE = 1.0` and `_BOOTSTRAP_PLATT_INTERCEPT = 0.0`, so the sigmoid of the logit returns the input unchanged. The docs admit "The constants are placeholders". — [cortex/extraction/stages/calibrate.py](GH/cortex/extraction/stages/calibrate.py); [docs/EXTRACTION.md](GH/docs/EXTRACTION.md)
- The hybrid "cost router" runs the heuristic backend first. It escalates every `ExtractedClaim` and any item with confidence <0.6 (`_ESCALATION_CONFIDENCE_THRESHOLD = 0.6`) to a single model call over only those spans, then merges. Diagnostics record `heuristic_kept`, `escalated` and `cost_saved_usd`. — [cortex/extraction/hybrid_backend.py](GH/cortex/extraction/hybrid_backend.py); [docs/EXTRACTION.md](GH/docs/EXTRACTION.md)
- Every extraction appends a JSON line (tokens_in/out, cost_usd, latency_ms, stage_timings, model, prompt_version, warnings, cache_hit, router_decision) to `.cortex/logs/extractions.jsonl`. A replay cache keyed by `sha256(prompt_version + input + model_id)` makes model evals deterministic. — [docs/EXTRACTION.md](GH/docs/EXTRACTION.md)
- Smoke test (my run, heuristic backend, scratch copy). I gave it the input "Alice is a data engineer at Acme. She prefers concise answers. We use Python and Postgres. We no longer use MySQL." Output: 4 nodes (Python, Postgres and **Mysql**, each tagged `technical_expertise` with confidence 0.8, plus Alice tagged `mentions` with 0.4) and 0 edges. The negation "no longer use MySQL" was stored as expertise. The role, employer and preference were missed. Each node carried provenance `{source:"extract:in.md", source_id:<sha256>, method:"extract", …}`, and 4 claim events were appended to `.cortex/claims.jsonl`. — local run of `python -m cortex.cli extract run` at ef05a57

**Entity resolution and dedup**
- Dedup scores each pair as `0.7 * text_similarity(label) + 0.3 * neighbor_overlap` with threshold 0.80. It only compares nodes that share at least one tag, which is O(n²) pairwise. Matches are merged by transitive closure (union-find), and the canonical node is the one with the highest confidence, then the earliest `first_seen`. README: "It does not use embeddings… if A matches B and B matches C… all three can collapse… even when A and C are below threshold." — [cortex/graph/dedup.py](GH/cortex/graph/dedup.py); [README.md](GH/README.md)
- `merge_nodes` keeps the max confidence, sums mention_count, unions tags, aliases and provenance, takes the longest brief/description, takes the min `first_seen`/`valid_from` and max `last_seen`/`valid_to`, truncates `source_quotes` to 5, and rewires edges while dropping self-loops. — [cortex/graph/graph.py](GH/cortex/graph/graph.py) `merge_nodes`
- Topic-level similarity in the heuristic extractor uses `max(SequenceMatcher ratio, word Jaccard) >= 0.85`. — [cortex/extraction/extract_memory_context.py](GH/cortex/extraction/extract_memory_context.py) `are_similar`
- An optional `EmbeddingBackend` uses sentence-transformers (`all-MiniLM-L6-v2`) and has a `canonical_match_by_similarity` method. Without the extra it raises `NotImplementedError`. A dependency-free `HashedEmbeddingProvider` (hashed character n-grams) also exists. — [cortex/extraction/embedding_backend.py](GH/cortex/extraction/embedding_backend.py); [cortex/extraction/embeddings.py](GH/cortex/extraction/embeddings.py)

**Temporal fields and contradictions**
- `ContradictionEngine` has 5 detectors:
  - negation conflicts: the same entity is in a positive tag and in `negations`; severity 0.6+0.4·conf, resolution `needs_review`
  - temporal flips: confidence changes direction ≥2 times across ≥3 snapshots; resolution `prefer_newer`
  - source conflicts: same label, different sources, different description hash; resolution `prefer_higher_confidence`
  - tag conflicts: resolution `prefer_newer`
  - temporal claim conflicts: invalid `valid_from`/`valid_to` windows or overlapping incompatible statuses such as active vs historical (severity 0.85); resolution `needs_review`

  Results are sorted by severity. They are recommendations, not automatic rewrites. — [cortex/graph/contradictions.py](GH/cortex/graph/contradictions.py)
- Three-way branch merge raises `field_conflict` (class DIRECT) on label, brief, full_description, status, valid_from, valid_to, relationship_type and canonical_id when both sides changed them. — [cortex/versioning/merge.py](GH/cortex/versioning/merge.py)
- Per-node `snapshots` and `graph_at(timestamp)` give point-in-time views, and `drift_score` uses a weighted Jaccard between graphs. — [cortex/graph/graph.py](GH/cortex/graph/graph.py); [cortex/graph/temporal.py](GH/cortex/graph/temporal.py)

**Provenance, retraction and ledger**
- `retract_source(source)` strips matching provenance and snapshot entries from every node and edge. It then prunes nodes and edges left with no provenance and logs a retraction record to `graph.meta["retractions"]`. PR #233 made provenance mandatory on extraction. — [cortex/graph/graph.py](GH/cortex/graph/graph.py) `retract_source`; [CHANGELOG.md](GH/CHANGELOG.md)
- `ClaimLedger` is append-only JSONL at `<store>/claims.jsonl` (ClaimEvent ops with claim_id and version_id) and is mirrored into a SQLite `claims` table in the SQLite backend. — [cortex/graph/claims.py](GH/cortex/graph/claims.py); [cortex/storage/sqlite_versions.py](GH/cortex/storage/sqlite_versions.py)

**Storage and portability**
- Versioning stores a **full graph JSON per version**: SQLite table `snapshots(version_id, graph_hash, graph_json, node_count, edge_count)`. Other tables are `version_history` (parent_id, merge_parent_ids, branch, signature), `refs`, `claims`, `governance_rules`, `remotes`, `lexical_indices` (a serialized TF-IDF payload per version), `embedding_indices` and `maintenance_audit`. A filesystem backend also exists. Version IDs are Merkle-chained (PR #232). — [cortex/storage/sqlite_versions.py](GH/cortex/storage/sqlite_versions.py) L52–110; [README.md](GH/README.md)
- Export formats: the v5 JSON graph (`export_v5`/`from_v5_json`), a v4 flat-category JSON downgrade (`to_v4_categories`), and per-target renderers. The README portability table says "write/mount" works for Claude, Notion, SystemPrompt, Claude Code, Codex, Cursor, Copilot, Windsurf, Gemini CLI, ChatGPT/Grok export artifacts and Hermes, while round-trip pull works only for Claude, Notion and SystemPrompt (README claim). Writes into AGENTS.md, CLAUDE.md and similar files go inside marker blocks so user content is preserved. Federation bundles are signed exports (pynacl). — [cortex/graph/graph.py](GH/cortex/graph/graph.py); [README.md](GH/README.md); [cortex/federation.py](GH/cortex/federation.py)

### Inferences
- The node model is a user-profile ontology (identity, preferences, expertise), not a general event or episodic graph. It fits the "what does the agent know about me/my project" use case and fits conversational episodic recall poorly.
- Transitive union-find dedup over lexical similarity with a 0.80 threshold risks chain over-merging, and the README admits this. Combined with deterministic `sha256(label)` IDs, two different "Alex" persons collapse by construction.
- `valid_from`/`valid_to`/`status` exist on the schema, but in my smoke test the heuristic extractor did not use negation or "no longer" cues to set them. The temporal schema is ahead of the extractors that are supposed to fill it.
- Storing the whole graph JSON per commit (instead of deltas) is simple but scales O(graph × commits). rewire's git-tracked markdown and JSONL already gives this kind of versioning more cheaply.

### Gaps
- I did not trace every heuristic rule or how the model backend's `link_to_graph` uses `CanonicalEntityRegistry`. That resolution path was not verified beyond the docs and the prompt file.
- Model-backend output quality is unknown: `tests/extraction/corpus/replay/` is empty at ef05a57, so no recorded model responses exist to inspect.

## 3. Retrieval and context assembly; token budgets

### Takeaway
Context assembly mostly does not depend on the user's query. A "Mind" (core graph plus attached "Brainpacks") is filtered by a disclosure policy (tag allowlist plus `min_confidence`), ranked with a fixed heuristic (tag priority, recency, cwd-path term overlap, confidence), and rendered into grouped markdown capped by `max_chars` (default 1,500 characters, not tokens). It is then written into AGENTS.md or CLAUDE.md or injected via a Claude Code SessionStart hook. Query-time search exists separately: a stdlib TF-IDF called "semantic_search", with exact, alias and fuzzy label boosts and optional hashed or sentence-transformer vectors. Graph traversal is offered as explicit tools (paths, k-hop, related), not as automatic context expansion.

### Cited Findings
- `_render_graph_for_target` works in four steps: `apply_disclosure(graph, policy)`, then sort facts by `-confidence`, then per-target render (generic tools use `generate_compact_context` with `max_chars`; Hermes uses `build_hermes_documents`; Claude uses preferences/memories export; ChatGPT/Grok use an "about/respond" instruction pack), then return `context_markdown`. — [cortex/mind_runtime.py](GH/cortex/mind_runtime.py) L334–424
- The `task` string only selects Brainpacks, by substring match against each attachment's `activation.task_terms` and `targets` (`always_on`, `target_match`, `task_match`, …). It does not rank nodes. — [cortex/mind_runtime.py](GH/cortex/mind_runtime.py) `_evaluate_attachment_match`, `_select_brainpacks_for_compose`
- Built-in disclosure policies:
  - `full`: no filter
  - `professional`: identity, professional and business context, expertise, priorities, work history, …
  - `technical`: technical_expertise, domain_knowledge and active_priorities, with min_confidence 0.5
  - `minimal`: identity and communication_preferences, with min_confidence 0.8

  — [cortex/versioning/upai/disclosure.py](GH/cortex/versioning/upai/disclosure.py) L25–66
- `_context_rank` adds +1.0 if any cwd path token appears in the node text, +1.0 for `active_priorities`, +0.8 for preference tags, +0.5 for `technical_expertise`, +0.3 for status "active" and −0.1 for "planned", plus a recency boost from `last_seen`/`first_seen`/`valid_from`. `_format_compact_markdown` groups the result into sections ("Tech Stack", "Projects", "Preferences", …) under "## Shared AI Context" and truncates to `max_chars`. — [cortex/hooks.py](GH/cortex/hooks.py) L166–280
- Claude Code integration: `handle_session_start` returns `hookSpecificOutput.additionalContext` built by the same path, and `install_hook` defaults to `policy="technical", max_chars=1500`. — [cortex/hooks.py](GH/cortex/hooks.py) L372–405
- The budget unit is characters. A `max_chars: int = 1500` default appears across `mind_runtime.py`, `portable_runtime.py`, `pack_runtime.py`, `channel_runtime.prepare_turn` and `mind_mounts.py`. The token-based `ExtractionBudget` (`max_tokens`, `max_latency_ms`, `max_cost_usd`) applies to extraction, not retrieval. — grep over [cortex/](GH/cortex); [docs/EXTRACTION.md](GH/docs/EXTRACTION.md)
- `CortexGraph.semantic_search` is a stdlib TF-IDF cosine index (`TFIDFIndex`), built lazily and invalidated on mutation. `semantic_search_documents` takes the max of the TF-IDF score and label boosts: exact label 1.0, exact alias 0.95, substring 0.55, fuzzy ≥0.75 times 0.45. `search_nodes` is plain case-insensitive substring search. — [cortex/graph/search.py](GH/cortex/graph/search.py); [cortex/graph/graph.py](GH/cortex/graph/graph.py) L710–780
- `vector_search_documents` and `hybrid_search_documents` exist over hashed or sentence-transformer embeddings, and per-version lexical and embedding indices are persisted in SQLite. — [cortex/extraction/embeddings.py](GH/cortex/extraction/embeddings.py); [cortex/storage/sqlite_versions.py](GH/cortex/storage/sqlite_versions.py)
- `QueryEngine` offers `query_category`, `query_path` (shortest path between labels), `query_changed(since)`, `query_related(label, depth=2)`, `query_strongest`/`weakest`, and a small `parse_nl_query`. — [cortex/graph/query.py](GH/cortex/graph/query.py)
- `ask_pack` is not LLM generation. It runs `query_pack(..., mode="hybrid")` and renders a templated note, report or slides markdown from ranked matches, optionally writing the artifact back to disk. — [cortex/pack_query.py](GH/cortex/pack_query.py) L356–413
- The MCP tool surface includes health, meta, portability context/scan/status/audit, agent status/monitor/compile/dispatch/schedule/review_conflicts, mind list/status/ingest/compose/remember/mounts/mount, pack list/status/context/mount/compile/query/ask/lint/export/import, channel prepare_turn/seed_turn_memory, index status/rebuild, prune, and node and edge lookup/get/upsert/delete. — [cortex/mcp/mcp_tools.py](GH/cortex/mcp/mcp_tools.py)
- `mind remember` runs `extract_graph_from_statement` (the heuristic path) on the statement, adopts the result into the Mind graph as a commit and refreshes all mounts. — [cortex/mind_runtime.py](GH/cortex/mind_runtime.py) L142–177

### Inferences
- The context model is "profile injection" (a static, policy-filtered slice written into instruction files), not "retrieve-per-query". That is a different job from rewire's per-query FTS5 plus vector retrieval, and it complements it.
- A 1,500-character cap with tag-priority ranking is cheap and predictable, but it ignores the query and can crowd out relevant facts in large graphs.
- What Cortex calls "semantic search" is lexical TF-IDF. rewire's FTS5 BM25 plus vectors is already stronger on retrieval.

### Gaps
- I did not inspect `channel_runtime.prepare_turn` deeply enough to confirm whether it does per-turn query-based retrieval. Its signature takes `max_chars=1500`, but the ranking logic was not traced.
- No measurements exist of context-assembly quality (for example answer accuracy with vs without the mount).

## 4. Evaluation and benchmark evidence

### Takeaway
The only quality evidence is a 5-case extraction corpus whose committed heuristic baseline scores are very low: node F1 0.25, relation F1 0.0 and contradiction recall 0.0. There is no committed model replay, no end-to-end memory QA benchmark (LoCoMo, LongMemEval or similar), and `cortex-bench` measures storage and service latency on synthetic graphs, not quality.

### Cited Findings
- The corpus has 5 cases: chat_name_role_preference, code_readme_stack_mentions, pathological_empty_garbled_profanity, policy_two_rules and transcript_contradiction. — [tests/extraction/corpus/manifest.yml](GH/tests/extraction/corpus/manifest.yml)
- Committed baseline (heuristic, prompt_version "corpus-v1"):

  | Metric | Value |
  |---|---|
  | node precision | 0.222 (4/18) |
  | node recall | 0.286 (4/14) |
  | node F1 | 0.25 |
  | relation precision | 0 (0/0) |
  | relation recall | 0 (0/9) |
  | relation F1 | 0.0 |
  | contradiction recall | 0/1 |
  | completeness | 0.357 (5/14) |
  | canonicalization accuracy | 0/0 |

  — [tests/extraction/corpus/baseline.json](GH/tests/extraction/corpus/baseline.json)
- The CI `extract-eval` job fails only if a metric regresses by more than 0.01 against that baseline, so the gate prevents regressions but does not require good quality. — [docs/EXTRACTION_EVAL.md](GH/docs/EXTRACTION_EVAL.md); [docs/EXTRACTION.md](GH/docs/EXTRACTION.md)
- `tests/extraction/corpus/replay/` is empty at ef05a57, so the documented model-backend replay eval has no recorded responses in the repo. — local `ls`
- The eval tooling exists and is well designed:
  - failure taxonomy: missed_node, wrong_type, bad_canonicalization, hallucinated_node, missed_relation, hallucinated_relation, missed_contradiction
  - an interactive `cortex extract review` that marks `true_failure` vs `gold_is_wrong`
  - prompt A/B (`cortex extract ab`) that declares a winner only when the F1 delta clears a Wilson-interval threshold

  — [docs/EXTRACTION_EVAL.md](GH/docs/EXTRACTION_EVAL.md); [cortex/extraction/eval/](GH/cortex/extraction/eval)
- `cortex/benchmark.py` builds synthetic chain graphs (`"{prefix} Node {i}"`, `related_to` edges) and times the `MemoryService` on SQLite, so it is a performance benchmark only. — [cortex/benchmark.py](GH/cortex/benchmark.py)
- My smoke test (section 2) agrees with the low baseline: the negation was misclassified and the role, preference and relationships were missed.

### Inferences
- Nothing in the repo shows that Cortex's graph improves agent answers. For rewire, Cortex's extraction quality is not evidence for adopting graph extraction. Its eval harness design (taxonomy, replay cache, regression gate, A/B with significance) is the valuable part.

### Gaps
- I found no published LoCoMo, LongMemEval or other QA results for Cortex-AI. I did not search the web.
- Model-backend precision and recall are unknown.

## 5. What rewire should copy and what it should skip

### Takeaway
Worth borrowing, roughly in priority order:
1. Source-keyed provenance on every derived fact, with whole-source retraction.
2. Typed fact/claim/relationship records with `stance` and `valid_from`/`valid_to`/`status` fields.
3. Rule-based contradiction detectors that flag items for review instead of rewriting them.
4. The extraction diagnostics JSONL, replay cache, failure taxonomy and regression-gated eval harness.
5. The heuristic-first, escalate-uncertain-to-LLM cost router as an optional enrichment path.
6. Policy-filtered "profile mounts" into AGENTS.md or CLAUDE.md with marker blocks, using a token budget instead of a character budget.

Not worth copying:
- whole-graph JSON snapshots per commit
- the regex extractor and its fixed confidence tables
- transitive lexical union-find dedup
- deterministic `sha256(label)` node IDs
- the no-op Platt calibration
- the oversized product surface (federation, web UI, Brainpacks, agent monitor)

### Cited Findings (mechanisms referenced)
- Provenance plus retraction: [cortex/graph/graph.py](GH/cortex/graph/graph.py) `retract_source`; [cortex/graph/claims.py](GH/cortex/graph/claims.py) `build_retraction_plan`, `retract_graph_source`
- Typed items: [cortex/extraction/extract_memory_context.py](GH/cortex/extraction/extract_memory_context.py) L211–278
- Contradiction detectors: [cortex/graph/contradictions.py](GH/cortex/graph/contradictions.py)
- Diagnostics, replay and eval: [docs/EXTRACTION.md](GH/docs/EXTRACTION.md); [docs/EXTRACTION_EVAL.md](GH/docs/EXTRACTION_EVAL.md)
- Cost router: [cortex/extraction/hybrid_backend.py](GH/cortex/extraction/hybrid_backend.py)
- Mounts, disclosure and SessionStart hook: [cortex/hooks.py](GH/cortex/hooks.py); [cortex/versioning/upai/disclosure.py](GH/cortex/versioning/upai/disclosure.py); [README.md](GH/README.md)
- Snapshots as full JSON: [cortex/storage/sqlite_versions.py](GH/cortex/storage/sqlite_versions.py)
- Dedup: [cortex/graph/dedup.py](GH/cortex/graph/dedup.py)
- Calibration placeholder: [cortex/extraction/stages/calibrate.py](GH/cortex/extraction/stages/calibrate.py)
- Baseline quality: [tests/extraction/corpus/baseline.json](GH/tests/extraction/corpus/baseline.json)

### Inferences (recommendations for rewire, with reasons)
**Copy or adapt:**
1. **Source-keyed provenance and retraction (high value, low cost).** rewire already stores JSONL events in git. Give every derived record (facts, links, summaries in the SQLite index) a `source_ids[]` that points back to event or markdown-note IDs, and add `rewire retract <source>` that removes the source and rebuilds the derived index. Since rewire's index is derived, retraction can simply be "delete the source and re-derive", which is simpler and more reliable than Cortex's in-place stripping. Privacy benefit: it gives a real "forget this conversation" path.
2. **Fact vs claim vs relationship, with `stance` and validity windows (medium value, low cost).** Add optional front-matter fields to markdown nodes or event payloads: `kind: fact|claim|relation`, `stance: asserts|denies|corrects`, `valid_from`, `valid_to`, `status: active|historical|planned`. This makes "no longer use MySQL" representable. The schema idea is worth copying even though Cortex's extractors fill it poorly.
3. **Rule-based contradiction flags with `needs_review` (medium value, low cost, no LLM).** The temporal-window overlap and negation-vs-positive detectors are cheap SQL-able checks over the derived index. Surface them as warnings in retrieval output ("conflicting: X active vs historical") instead of silently resolving them.
4. **Eval harness pattern (high value for development).** Cortex's best engineering idea for rewire is:
   - a JSONL diagnostics log per extraction or derivation call
   - a replay cache keyed by `sha256(prompt_version + input + model)` so evals are deterministic and offline
   - a gold corpus with a failure taxonomy
   - a CI gate on regression greater than 0.01
   - prompt A/B with a significance test

   rewire already runs LoCoMo, LongMemEval, BEAM and PersonaMem; adding replay caching and a regression gate would make those runs reproducible and cheap.
5. **Heuristic-first, escalate-uncertain router (optional).** If rewire ever adds LLM-based fact or entity extraction, use the cheap path by default and send only low-confidence or claim spans to the model, recording `cost_saved`. This keeps local-first and offline as the default and bounds API cost and data egress.
6. **Policy-filtered profile mounts (low cost, nice integration).** These are marker-delimited blocks in AGENTS.md or CLAUDE.md plus a SessionStart hook carrying a short "standing context" slice (preferences, active priorities). They complement rewire's per-query MCP retrieval. Use a **token** budget and query or recency ranking instead of Cortex's 1,500-character tag ranking.

**Do not copy:**
- **Whole-graph JSON snapshot per commit.** rewire's git over markdown and JSONL already provides history, diff, blame and branch at file granularity. Cortex's SQLite `snapshots.graph_json` duplicates the full graph on every commit and is a reimplementation of git.
- **Regex heuristic extractor and fixed confidence tables.** The measured baseline is node F1 0.25 and relation F1 0.0, and my smoke test stored a negated technology as expertise. That is not good enough to inject as "memory". If rewire wants entities or relations, it should use an LLM pass over its own events behind an opt-in flag, and evaluate it on rewire's benchmarks first.
- **Transitive lexical dedup and `sha256(label)` IDs.** These cause chain over-merging and homonym collisions (two "Alex"). rewire should keep stable, non-content-derived IDs for notes and treat entity merging as a reversible alias link, not a destructive merge.
- **Placeholder Platt calibration and a confidence scale without evidence.** Do not surface numeric confidences that are not calibrated.
- **Product sprawl:** federation, signed remotes, web UI, Brainpacks, "Minds" and namespaces, agent monitor, a 40+ tool MCP surface. That is high complexity with no demonstrated memory-quality benefit, and a large MCP tool list also costs agent context tokens.
- **TF-IDF "semantic search".** rewire's FTS5 plus vector retrieval is already a superset.

**Possible combination:** rewire could expose a Cortex-compatible export (v5 JSON nodes and edges with provenance) so Cortex users can mount rewire memory into tools. The value is low unless there is demand; the format is simple to emit from rewire's derived index.

### Gaps
- None of these recommendations has been tested in rewire. The expected value (especially of the fact/claim schema and contradiction flags) should be checked against rewire's existing LoCoMo, LongMemEval, BEAM and PersonaMem harness before adoption.
