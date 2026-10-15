# Design: Synaptic Memory for rewire

> Status: **draft for review**. Nothing has been implemented yet.
> Follows on from the [memory approaches research](../research/memory-approaches.md) and [Karpathy LLM Wiki](../research/karpathy-llm-wiki.md).
>
> **Rev 2:** the MVP specification is in [mvp-spec.md](mvp-spec.md). The Phase 0 results changed the defaults: `d = 0.7`, an **active project scope** and a **separate constraint budget** are mandatory, and Hebbian learning is deferred to v0.2 ([details](../../experiments/phase0/README.md)).
>
> **Revision 1 (after the LLM Wiki review):** activation only controls ranking, constraints do not decay, bitemporal records, a `sources/` layer, contextual push memory, and incremental lint. These changes are marked *(rev 1)*.

## 0. Goals & principles

1. **Like human memory**: memories that are used often grow stronger, memories that are rarely used fade (but are not lost), and memories that are often used *together* become more strongly connected (Hebbian: *neurons that fire together, wire together*).
2. **Update after the session** (the Hermes pattern): the session writes quickly, and then a consolidation phase updates the brain.
3. **Graph/ontology structure**: memories are typed nodes connected by typed relations.
4. **Own the brain, transfer anywhere**: the brain remains a set of files owned by the user. It can be put in git, it can be read by humans, and it does not require a server.

Principles that bind every decision below:

- **P1 — Store *events*, not *weights*.** Weights (synapse strength) are always *computed* from the event log. This is the most important design decision in this document (see §4).
- **P2 — Salience ≠ truth.** How often something is used (activation) is separate from whether a fact still holds (validity). A fact that is used often may already be stale.
- **P3 — Forgetting = dropping a tier, not deleting.** A weak memory leaves the context, but it can still be "recalled again".
- **P4 — Files are the source of truth; the database is only an index** that can be rebuilt at any time.
- **P5 — Activation controls *order*, not *existence* or *truth*.** *(rev 1)* A durable constraint does not become less true just because it is rarely read. Activation is a component that can be switched off (`β = 0`), and it must win in evals to be kept.

---

## 1. Mental model: three kinds of memory

| Kind (brain analogy) | In rewire | Example | Properties |
|---|---|---|---|
| **Episodic** (hippocampus) | `sessions/` — per-session log | "Oct 10: we researched memory harnesses" | Append-only, raw, cheap to write |
| **Semantic** (neocortex) | `nodes/` — graph of facts/concepts | "User prefers a mature design before coding" | Result of consolidation, curated |
| **Procedural** | `skills/` — SKILL.md | "How to release project X" | Invoked when relevant |
| **External sources** *(rev 1)* | `sources/` — articles, papers, documents | Karpathy's gist | Immutable; *ingested* into `nodes/` (the LLM Wiki pattern) |

Consolidation = moving from episodic to semantic, exactly as sleep moves memories from the hippocampus to the neocortex.

---

## 2. Graph & ontology

### 2.1 Node types (core ontology — deliberately small)

| Type | Meaning | Example |
|---|---|---|
| `person` | A person (including the user themselves) | `@me`, a coworker |
| `project` | Project / repo / initiative | `rewire` |
| `concept` | Idea, technology, term | "Hebbian learning", "SQLite" |
| `preference` | How the user wants to work | "answer in Indonesian" |
| `decision` | A decision + its rationale | "use files as the source of truth" |
| `fact` | A factual statement that can change | "API X rate limit is 100/minute" |
| `procedure` | Pointer to a skill | `skills/release/` |
| `artifact` | External document/URL/file | dashboard link, document |

### 2.2 Relation types (fixed vocabulary of ~12)

`about`, `part_of`, `depends_on`, `decided_in`, `prefers`, `works_on`, `knows`, `uses`, `supersedes`, `contradicts`, `derived_from`, `related_to` (fallback).

**Ontology rules:**
- The vocabulary is stored in `brain.yaml`. Consolidation **may propose** new types/relations, but they go into `inbox/` for approval. This prevents *ontology sprawl* (the classic problem of LLM-built graphs: 300 relation types that all mean the same thing).
- Full OWL/RDF is not used. It is too heavy for this need; all that is needed is typed nodes + typed edges + a few constraints (e.g. `supersedes` only between nodes of the same type).

### 2.3 Two classes of edge (important)

| | **Semantic edge** | **Associative edge (synapse)** |
|---|---|---|
| Meaning | A meaningful relation: "rewire `uses` SQLite" | "These two memories often appear together" |
| Created by | Consolidation (LLM) or the user | **Automatically** from co-activation |
| Stored in | The node file's frontmatter (curated, in git) | **Not stored** — computed from the event log |
| Weight | Per-relation-type prior + Hebbian | Pure Hebbian |

This mimics the brain: there is structured knowledge, and there are associations that grow on their own from experience.

---

## 3. The "synapse" model: the math

Foundation: **ACT-R** (Anderson's cognitive model), which has been validated for decades against human memory data, plus **Hebbian learning** for edges and **spreading activation** for retrieval.

### 3.1 Node strength — base-level activation

```
B(i) = ln( Σ_j  (now − t_j)^(−d) )        d ≈ 0.5
```

`t_j` = each time node *i* was actually used. The effects:
- **Frequency** raises the value (more terms are summed).
- **Recency** raises the value (recent terms are much larger).
- **Decay** naturally follows a power law, matching the Ebbinghaus forgetting curve.
- **Spacing effect**: use spread over many days lasts longer than 10 uses within one hour.

Example (time in hours):

| Usage pattern | Calculation | B |
|---|---|---|
| Used 1 hour, 1 day, and 7 days ago | ln(1 + 0.204 + 0.077) | **+0.25** |
| Used once, 30 days ago | ln(720^−0.5) | **−3.29** |
| Used 5x within 1 hour, 30 days ago | ln(5 · 0.037) | **−1.68** |

For efficiency, store the last *k* events exactly plus the Petrov approximation for the rest: `ln(n/(1−d)) − d·ln(L)` (n = number of uses, L = age since the first use).

### 3.2 Synapse strength — Hebbian with decay

Every time nodes *a* and *b* are used in the same session → a `co_use(a,b)` event. Edge strength is computed with the same formula as 3.1 over the co-use events, then normalized to 0..1:

```
H(a,b)   = σ( ln Σ_j (now − t_j)^(−d) )       # σ = sigmoid
W(a,b)   = prior(rel) + (1 − prior(rel)) · H(a,b)   # for semantic edges
W(a,b)   = H(a,b)                                  # for associative edges
```

An associative edge with `W < θ_edge` is treated as nonexistent (natural pruning).

### 3.3 Retrieval — spreading activation

When the agent needs memory (a query or session context):

1. **Seed**: find nodes that match textually (FTS5) and optionally by embedding → relevance score `R(i)`.
2. **Spread** at most 2 hops from the seeds:
   ```
   A(j) = B(j) + Σ_i  R(i) · W(i,j) · (S − ln fan(i))
   ```
   `fan(i)` = the number of edges of node *i*. ACT-R's **fan effect**: a "hub" node connected to everything spreads weaker activation. This prevents nodes like "@me" or "coding" from dominating.
3. **Final score**:
   ```
   score(j) = α·R(j) + β·norm(A(j)) + γ·importance(j)
   ```
   with a filter: `superseded` nodes / nodes with `valid_to < now` are dropped unless the query is historical.
4. Take the top-k until the **token budget** runs out; return summaries + IDs (claude-mem's progressive pattern: `search → get`).

This approach is close to **HippoRAG** (Personalized PageRank over a knowledge graph, inspired by the hippocampus) and the *recency × importance × relevance* score from the **Generative Agents** paper. Both are proven in research.

### 3.4 What counts as "used"? (the most fragile part)

If the definition of "used" is wrong, the system falls into a **rich-get-richer loop**: memories in the snapshot are always visible → treated as used → grow stronger → always in the snapshot.

| Signal | Event weight | Notes |
|---|---|---|
| Present in the session-start snapshot | **0** | Only exposed, not necessarily used |
| Appears in `search` results | **0** | Only a candidate |
| Agent calls `get(id)` | 0.5 | There is an intent to use it |
| Consolidation judges that this memory influenced an answer/action | **1.0** | The main signal |
| User confirms ("yes, that's right") | 2.0 + `confidence: confirmed` | |
| User corrects / contradicts | `contradict` event | Does not raise B; triggers a validity review |

Weighted events → the terms in §3.1 are multiplied by their weight.

Additional safeguards: a maximum number of events per node per session (e.g. 1), and a little *exploration* (occasionally putting a relevant dormant node into the snapshot).

### 3.5 Tiers (human-like forgetting)

| Tier | Condition | Behavior |
|---|---|---|
| **core** | top-N activation within the token budget | Goes into the session-start snapshot (frozen) |
| **active** | B ≥ θ_active | Takes part in spreading activation |
| **dormant** | B < θ_active | Does not take part in spreading; only appears via explicit search |
| **archived** | dormant > X months or superseded | Only for historical queries |

A dormant memory that is used again immediately moves up a tier: it is "recalled again".

Besides activation, each node has an `importance` (1–5) from consolidation/the user. An important memory that is rarely used (e.g. an allergy, a password policy) must not disappear just because it rarely comes up: `importance ≥ 4` → never drops below `active`.

**Decay exceptions** *(rev 1)*: `preference` nodes, constraints (`importance ≥ 4`), and the `person: @me` node are **not subject to decay at all**. They are ordered by `importance` and are always prioritized for `core`. Decay only applies to `fact`, `concept`, `project`, `artifact`, and associative edges, which are memories whose relevance genuinely depends on context.

---

## 4. Storage (the answer to the main concern)

### 4.1 Why weights should not be stored in the main files/database

If every use updated `weight: 0.73` in a file:
- Every session would change dozens of files → **noisy git diffs** and a useless history.
- Two machines/harnesses using the brain at the same time → **merge conflicts** over numbers that mean nothing to humans.
- Parameters (`d`, `θ`) could not be retuned without losing data.

If what is stored is **events** (`use`, `co_use`, `confirm`, `contradict`) in an append-only log:
- Merging across devices = **concatenating logs** (union). There is never a conflict.
- Weights can be recomputed at any time with new parameters, which makes it possible to experiment and *replay* history.
- This is event sourcing, a mature pattern.

### 4.2 Layout

```
~/brain/                              # git repo = brain
├── brain.yaml                        # schema version, ontology, parameters (d, θ, budget)
├── nodes/                            # SEMANTIC — 1 node = 1 markdown file
│   ├── preference/
│   │   └── design-before-build.md
│   ├── project/rewire.md
│   └── concept/hebbian-learning.md
├── events/                           # SYNAPSE — append-only log, 1 file per device per month
│   └── <device-id>/2026-10.jsonl     #   → never conflicts on merge
├── sessions/                         # EPISODIC — per-session summaries
│   └── 2026-10-10/<session-id>.md
├── sources/                          # immutable EXTERNAL SOURCES (rev 1)
│   └── 2026-10-10-karpathy-llm-wiki.md
├── inbox/                            # candidates awaiting approval
├── skills/<name>/SKILL.md            # PROCEDURAL
├── snapshot/CORE.md                  # generated output, injected into the harness
└── .index/brain.db                   # SQLite, gitignored, can be rebuilt
```

**Node file:**

```markdown
---
id: 01JA8Z3K7Q...            # ULID, stable even if the file is renamed
type: preference
title: Mature design before coding
aliases: [design-first]
importance: 4
confidence: confirmed        # confirmed | inferred
valid_from: 2026-10-10       # valid-time: when it holds in the world
valid_to: null
recorded_at: 2026-10-10T15:02:11Z   # belief-time: when the brain started believing it (rev 1)
retracted_at: null                   # when the brain stopped believing it
source: { harness: claude-code, session: 01JA8Z... }
edges:
  - { rel: about, to: 01JA8Z...rewire }
  - { rel: derived_from, to: 01JA8Y...session }
---
The user wants to validate the design (especially storage) before implementation starts.
Related: [[rewire]], [[event-sourcing]].
```

The body may use `[[slug]]` wikilinks *(rev 1)* so the brain can be opened directly in Obsidian (graph view, backlinks) without additional tools. Official edges stay in the frontmatter; wikilinks in the body are treated as low-prior `related_to` edges.

**Event (JSONL):**

```json
{"t":"2026-10-10T15:02:11Z","ev":"use","node":"01JA8Z3K7Q","w":1.0,"session":"01JA8Z...","harness":"claude-code"}
{"t":"2026-10-10T15:02:11Z","ev":"co_use","a":"01JA8Z3K7Q","b":"01JA8Z9RWX","session":"01JA8Z..."}
{"t":"2026-10-10T15:40:00Z","ev":"contradict","node":"01JA7...","session":"01JA8Z..."}
```

### 4.3 Index (derived): SQLite

```
nodes(id, type, title, body, importance, valid_from, valid_to, tier, path)
edges(src, dst, rel, prior)              -- from frontmatter
events(t, ev, node, a, b, w, session)    -- from events/*.jsonl
activation(node, B, updated_at)          -- cache, recomputed
assoc(a, b, H, updated_at)               -- associative edge cache
nodes_fts  (FTS5)
nodes_vec  (optional, sqlite-vec)
```

A 2-hop traversal is easily handled with a recursive CTE in SQLite. A personal graph (thousands to tens of thousands of nodes) is far below the point where a dedicated graph DB is needed.

### 4.4 Comparison of storage options

| Option | Pros | Cons | Verdict |
|---|---|---|---|
| **Markdown + JSONL + SQLite index** | Portable, can be put in git, human-readable, zero servers, can be rebuilt | Complex graph queries must be written by hand | ✅ **Recommended** |
| Graph DB server (Neo4j, Memgraph) | Cypher, ready-made graph algorithms | Requires a server, brain locked in the DB, hard to put in git | ❌ conflicts with "own the brain" |
| Embedded graph DB (Kuzu, etc.) | Cypher without a server | Binary format, project maintenance risk, still cannot be diffed | ⚠️ could be an optional index backend |
| RDF triple store (Oxigraph) + OWL | Formal ontology, SPARQL | Complex, overkill for a personal graph | ❌ for v1 |
| Markdown + wikilinks only (Obsidian style) | Very simple | No place for weights/events | ⚠️ insufficient for the synapse concept |

Because the index is derived, **the backend can be swapped later** (e.g. exporting to Neo4j for visualization) without moving the brain.

---

## 5. Session lifecycle (the Hermes pattern, extended)

```
SessionStart ──► generate snapshot/CORE.md (core tier + spreading from the active project node)
                 → injected & FROZEN (preserves the prefix cache)
     │
During ────────► recall(query)   → search + spread → summaries + IDs   (writes no event)
                 get(id)          → node details                        (use event w=0.5)
                 remember(text)   → write to sessions/ + inbox/         (fast, no gate)
                 PUSH (rev 1)     → PreToolUse hook / when a file is opened: inject 1–3
                                    `feedback`/`decision` nodes related to that tool/path
                                    ("LESSON:" pattern), because agents often forget to call recall
     │
PreCompact ────► flush: ask the agent to save important things before the context is compacted
     │
SessionEnd ────► SESSION CONSOLIDATION (async, does not block the user)
                 1. Summarize transcript → sessions/<date>/<id>.md
                 2. Extract candidates: entities, facts, preferences, decisions
                 3. Entity resolution: match against existing nodes (FTS + embedding + LLM judge)
                 4. Decide an action per candidate: CREATE | UPDATE | SUPERSEDE | CONTRADICT | IGNORE
                    — contradictions are resolved HERE (write-time), not left for lint (rev 1):
                      the old claim is closed (valid_to / retracted_at), not deleted
                 5. Judge which nodes were actually used → use/co_use events
                 6. Security scan (injection, exfiltration, hidden unicode)
                 7. Validate schema, write files + events, git commit "session <id>"
                 8. Per-stage health report (rev 1) — a failed stage = a report, not a silent failure
     │
Ingest (rev 1) ► `rewire ingest <url|file>`: save to sources/ (immutable) → summarize →
                 update/create related nodes (usually 5–15) → `derived_from` events (LLM Wiki pattern)
     │
Periodic ──────► "SLEEP" (e.g. daily)
                 - recompute activation & tiers
                 - INCREMENTAL LINT (rev 1): only nodes changed since the last sleep + their
                   1-hop neighbors (a full lint is quadratic in the number of nodes): contradictions,
                   orphan nodes, missing links, stale claims
                 - merge duplicates, prune weak associative edges
                 - re-summarize hub nodes that have grown too long
                 - new ontology proposals → inbox/
                 - change report → sessions/dreams/<date>.md (for human review)
```

Safety rule: destructive actions (SUPERSEDE, merging nodes, changing `importance` ≥ 4) from an `inferred` source go to `inbox/` first; from a `confirmed` source they may be applied directly.

---

## 6. Risks & mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| **Rich-get-richer** | The snapshot freezes on the same memories | Exposure = 0 (§3.4), per-session cap, exploration |
| **Decay hides important constraints** *(rev 1)* | The agent violates rules that are rarely read | P5 + decay exceptions (§3.5); a finding from the Link project in the LLM Wiki thread |
| **Agent does not call recall** *(rev 1)* | Lessons that were already written down are not used | Contextual push via hooks (§5) |
| **Pipeline fails silently** *(rev 1)* | The brain stops learning without anyone noticing | Per-stage health report |
| **Duplicate nodes** (entity resolution fails) | Activation is split, the graph gets dirty | `aliases`, staged resolution, merging during "sleep" (old events are moved to the winning node) |
| **Wrong LLM extraction** | An incorrect fact becomes permanent | `confidence: inferred` vs `confirmed`, inbox, git history for reverts |
| **Stale but frequently used facts** | The agent trusts old information | P2: separate validity; `contradict` → review; `valid_to` |
| **Ontology sprawl** | Inconsistent relations | Fixed vocabulary + approval |
| **Hub nodes dominate** | Irrelevant retrieval | Fan effect (§3.3) |
| **Persistent prompt injection** | A malicious memory enters every session | Scanning, untrusted sources are not promoted automatically, `<private>` |
| **Event log bloat** | Slow index | Monthly compaction: events older than N months are summarized as (n, L) per node (Petrov approximation) |
| **Consolidation cost** | Expensive per session | Async, small model, skip trivial sessions |
| **Multi-device** | Conflicts | Per-device events (union), node files rarely change, ULIDs |

---

## 7. How to validate the design before building it fully

Recommended order (cheap → expensive):

1. **Phase 0 — Mathematical simulation** (no LLM, no harness). A script that generates a synthetic event log (frequently used memories, rarely used ones, spaced patterns, hub nodes, superseded facts) and plots B, H, and tiers over time. Goal: choose `d`, `θ`, and the `α/β/γ` weights, and prove there is no rich-get-richer effect. **This is the fastest way to prove the design is solid.**
2. **Phase 1 — Storage + index + CLI**: file format, event log, SQLite rebuild, `recall/get/remember`, `CORE.md` generation. Test: delete `.index/` → rebuild → identical results.
3. **Phase 2 — Consolidation**: the SessionEnd pipeline with replay of real transcripts. Metrics: duplicate rate, extraction precision, how many candidates go to the inbox.
4. **Phase 3 — Harness adapters**: Claude Code (hooks + MCP), Codex, and one harness without hooks (AGENTS.md + CLI).

**Mandatory ablation** *(rev 1)*: every eval is run with and without activation (`β = 0`). Activation is kept only if it wins measurably.

**Success metrics** (built as an eval set): recall@k of the facts a task needs, snapshot tokens, rate of stale facts retrieved, duplicate rate, snapshot stability across sessions.

---

## 8. Open decisions (need the user's answer)

1. **Who decides the "used" event?** Proposal: a combination of the `get` tool call + LLM judgment during consolidation. Cheaper alternative: tool calls only (but biased toward agents that call tools diligently).
2. **Embeddings in v1?** Proposal: no; FTS5 + spreading activation first, with embeddings as a plugin for entity resolution.
3. **How automatic should consolidation be?** Fully automatic, automatic + inbox for destructive actions (proposed), or everything through review.
4. **Brain scope**: one global brain with `project` nodes, or a brain per context (work/personal) that can be mounted together?
5. **Implementation language** for the CLI/MCP.

---

## Concept references

- ACT-R base-level learning & fan effect — Anderson & Schooler (1991), *Reflections of the environment in memory*; Anderson (2007) *How Can the Human Mind Occur in the Physical Universe?*
- Base-level approximation — Petrov (2006), *Computationally efficient approximation of the base-level learning equation in ACT-R*
- Hebbian learning — Hebb (1949), *The Organization of Behavior*
- Generative Agents (recency × importance × relevance, reflection) — Park et al. (2023)
- HippoRAG (graph + Personalized PageRank, inspired by the hippocampus) — Gutiérrez et al. (2024)
- A-MEM (Zettelkasten-style memory that links to itself & evolves) — Xu et al. (2025)
- Graphiti / Zep — temporal knowledge graph with fact invalidation
- Hermes (post-session consolidation, frozen snapshot), OpenClaw (gated dreaming) — see the research documents
- Karpathy, *LLM Wiki* (ingest / query / lint, index + log) and its community thread (Link, post-graph) — see the [research](../research/karpathy-llm-wiki.md)
