# Research: Karpathy's "LLM Wiki" and its relevance to rewire

> Source: https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f (the gist + the community comment thread). Read 2026-10-10.
> Related: [memory approaches](memory-approaches.md), [synaptic memory design](../design/synaptic-memory.md).

## 1. Summary of the gist

**Core idea:** instead of RAG that retrieves raw document chunks every time a question is asked (knowledge is "rediscovered" on every query), the LLM **incrementally builds and maintains an interlinked markdown wiki**. Each new source is integrated into existing pages, contradictions are flagged, and syntheses are updated. The wiki *compounds*; humans hardly write it themselves.

**Three layers:**
1. **Raw sources** — the original documents, immutable. The LLM reads them and does not change them.
2. **Wiki** — LLM-written markdown pages (summaries, entity/concept pages, comparisons, overviews). Owned by the LLM.
3. **Schema** — a conventions document (CLAUDE.md/AGENTS.md) that defines the structure & workflow; refined together with the human.

**Three operations:**
- **Ingest** — read the source → discuss → write a summary → update the index + 10–15 related pages → append to the log.
- **Query** — read the index → read relevant pages → answer with citations; good answers are saved back as new pages.
- **Lint** — periodic health check: contradictions, stale claims, orphan pages, missing concepts/links.

**Navigation files:** `index.md` (a catalog by category, one line per page, read first) and `log.md` (chronological, append-only, with consistent prefixes so it can be grepped).

**Why it works:** humans abandon wikis because the maintenance cost > their value. LLMs do not get bored maintaining cross-references, so the maintenance cost ≈ 0.

## 2. Lessons from the comment thread (the most important ones for us)

| Community finding | Relevance for rewire |
|---|---|
| **Link (gowtham0992)** tested *usage-weighting, recency decay, and MMR* — and **shipped none of them**. The reason: durable constraints *"do not become less true for going unread."* | ⚠️ **Directly challenges our synapse concept.** See §4. |
| **post-graph (crajah)**: contradiction resolution **at write time** (not searched for later), old claims are closed rather than deleted, and **two separate time axes**: *valid-time* (when a fact holds in the world) vs *belief-time* (when the brain believed it). | ✅ Adopt: bitemporal + supersede during consolidation. |
| Early extraction errors **permanently poison the store**; model output must be treated as *untrusted input*. | ✅ Reinforces `confidence: inferred`, the inbox, and schema validation before writing. |
| A capture pipeline **failed silently** for two weeks. | ✅ Add health checks & per-stage consolidation reports. |
| **Agents do not read memory mid-task** — they re-diagnose problems whose lessons were already written down. orcosto-lab puts a `LESSON:` prefix in tool descriptions so it is seen before every call. | ⚠️ A *pull vs push* problem: memory that can only be pulled often is not pulled. Contextual injection (push) is needed. |
| LLM-based lint is **quadratic** in the number of pages. | ✅ Lint must be incremental (only changed nodes + their neighbors). |
| The claims of this pattern have **not been benchmarked** in a controlled way. | ✅ Reinforces our eval plan (Phase 0 and the metrics in §7 of the design). |
| Curated Thoughts: a SQLite "brain" + MCP + graph traversal + **review before merge**. | ✅ Validates our storage direction (SQLite + inbox review). |
| Cortexes: BM25 + vector with RRF, provenance-aware dedup. | ✅ A hybrid retrieval option for the embedding phase. |

## 3. Mapping LLM Wiki ↔ rewire

| LLM Wiki | rewire (current design) | Notes |
|---|---|---|
| Raw sources (immutable) | `sessions/` (transcripts/summaries) | **Gap:** rewire only captures agent sessions so far, not *external sources* (articles, papers, documents). |
| Wiki pages | `nodes/*.md` | Same: LLM-written, interlinked markdown. We add types & typed relations. |
| Schema (AGENTS.md) | `brain.yaml` + ontology | Same. Ours is more formal (fixed types). |
| `index.md` | `snapshot/CORE.md` + `MEMORY.md` | The difference: Karpathy's index contains **all** pages; our CORE is selected by activation & budget. |
| `log.md` | `events/*.jsonl` + `sessions/dreams/` | We separate the machine log (events) from the human log. |
| Ingest | SessionEnd consolidation | Same pattern: one input → updates many nodes. |
| Query → save the answer back | `remember()` | Adopted explicitly: good answers/syntheses become `concept`/`decision` nodes. |
| Lint | Periodic "sleep" | Same. We add decay/tiers. |
| Obsidian + graph view | — | **Free bonus:** because our nodes are markdown, the brain can be opened in Obsidian (using wikilinks in the body). |

**Conclusion:** the LLM Wiki is the **semantic layer** of rewire, almost 1:1. What rewire adds on top of it: (a) a per-agent-session lifecycle and adapters for many harnesses, (b) a typed ontology, (c) an activation/synapse model, (d) event sourcing for multi-device use.

## 4. Critical question: is the synapse still worth it?

Link's finding is not a reason to throw away the synapse concept, but it **limits its role**. There are two kinds of memory that behave differently:

| | **Constraints / identity** | **Associations / working context** |
|---|---|---|
| Examples | "Never push to main", "the user is allergic to peanuts", "answer in Indonesian" | "When discussing rewire we usually also discuss SQLite", the currently active project |
| Does it become less true if rarely used? | **No** | Its relevance does drop |
| Role of decay | ❌ Not allowed | ✅ Makes sense |

So the design revision:
1. **Activation only governs *ranking & placement in the context*, never *existence* or *truth*.** Nothing is deleted because of decay.
2. **Nodes of type `preference` and constraints (`importance ≥ 4`) are exempt from decay** — always `core` or `active` depending on the budget.
3. **Synapses are most useful for *associative edges*** (which memories get recalled along with X when X is active) and for choosing the project snapshot — there, usage patterns really are a relevance signal.
4. **Must be proven with an eval** in Phase 0/2: compare retrieval *with* vs *without* activation. If it does not win, turn it off (in line with Link's experience). The design must make activation a component that can be *toggled* (`β = 0`).

## 5. Proposed design revisions (already applied to the design document)

1. **Bitemporal**: `valid_from/valid_to` (world) + `recorded_at/retracted_at` (brain belief).
2. **Constraints do not decay**; activation is only for ranking.
3. **Contradictions are resolved during consolidation** (write-time), not only during lint.
4. **A `sources/` layer** for immutable external sources + an `ingest` operation.
5. **Contextual memory push**: a hook before a tool call / when opening a file injects relevant lessons (the `LESSON:` pattern), instead of relying only on the agent calling `recall`.
6. **Incremental lint** + **per-stage health checks** for consolidation.
7. **Obsidian-compatible**: `[[...]]` wikilinks in the node body.
