# Build readiness review: Synaptic Memory (rev 1)

> Assesses [synaptic-memory.md](synaptic-memory.md) rev 1. A subjective assessment, written 2026-10-10.
> Section 3 covers similar projects that were checked directly (README/paper), not just taken from a list.

## 1. Scores

Scale of 1–10. The weight shows how important each dimension is for the *decision to start building*.

> **Pre-build self-assessment.** These scores were given to the design before it was built and measured. The
> evaluations they lean on (E1, E4) were not run (see [iterations](../iterations.md)); later independent reviews
> scored the built system lower. Read them as the design's intent, not its result.

| # | Dimension | Weight | Score | Short rationale |
|---|---|---|---|---|
| 1 | Goals & principles | 10% | **9** | P1–P5 are clear, mutually consistent, and have already been tested against criticism (the Link finding). |
| 2 | Storage model | 15% | **8** | Event sourcing + files + a derived index is a strong decision. Gap: two devices editing *the same node file* is not yet handled. |
| 3 | Activation / synapse model | 10% | **7** | The math is grounded (ACT-R, fan effect) and its role is already limited. Gap: "co-use = used in the same session" is too coarse — one session can cover 5 topics and produce spurious edges. There is no co-occurrence normalization yet. |
| 4 | Ontology & node granularity | 10% | **5** | The types exist, but **the size of a single node** has not been decided (atomic facts like Mem0, or pages like LLM Wiki?). This affects extraction, entity resolution, and activation. |
| 5 | Consolidation pipeline | 10% | **6** | The steps are complete, but there is no input/output contract for each step, no idempotency (what if a session is consolidated twice?), and no cost estimate. |
| 6 | **Harness integration (the product's core promise)** | 15% | **4** | Adapters exist only in the research document. Still missing: the list of MCP tools + their schemas, the hook mapping per harness, and **where the transcript comes from** in each harness (without a transcript, consolidation cannot run). |
| 7 | Security & privacy | 10% | **5** | Mentioned in many places, but there is no threat model (memory poisoning, secrets in transcripts, a brain pushed to a public remote). |
| 8 | Evaluation plan | 10% | **6** | Ablation and metrics exist; a concrete dataset and baseline do not. |
| 9 | MVP scope | 10% | **4** | All features are treated as equally important. There is no firm line saying "v0.1 = only this". Biggest risk: overbuilding. |

**Weighted score: 6.0 / 10**

### Verdict

> **Not yet ready for a full build. Ready to start Phase 0 (simulation) and a storage spike.**

The conceptual foundation (storage, principles, activation model) is already solid — the parts that are hardest to change later are already right. What is missing is the *product* part: the harness integration contract, node granularity, and the MVP boundary. All three can be settled on paper without writing production code. Target before building: **≥ 7.5**.

---

## 2. What must be added to reach ≥ 7.5

| Priority | Addition | Raises dimension |
|---|---|---|
| P0 | **MVP definition** (proposal in §4) | 9 |
| P0 | **Integration contract**: 5–6 MCP tools with schemas, a hook table per harness, the transcript source per harness (Claude Code `~/.claude/projects/*.jsonl`, Codex, the OpenCode DB, etc.) | 6 |
| P0 | **Node granularity**: proposal = *one claim/preference per node* for `fact/preference/decision`, *one page* for `project/concept/person` (summarizing hubs) | 4 |
| P1 | **Sharper co-occurrence**: co-use only for nodes used within *the same task window* (e.g. within N turns), normalized with NPMI, with per-node homeostasis (the Ori-Mnemos pattern) | 3 |
| P1 | **Node file edit conflicts**: node files are written only by consolidation; git conflicts are resolved with "the version with the latest `recorded_at` wins + the losing version is kept as `retracted`" | 2 |
| P1 | **Concrete eval**: a dataset from the user's real sessions + public subsets (STALE for stale facts, ForgetEval for forgetting, LongMemEval/LoCoMo only as a sanity check) and **Ori-Mnemos as the baseline** | 8 |
| P2 | A 1-page threat model | 7 |
| P2 | A contract for each consolidation step + idempotency (key = session id + transcript hash) | 5 |

---

## 3. Similar projects to learn from

### 3.1 Ori-Mnemos — the most similar, must be studied
[github.com/aayoawoyemi/Ori-Mnemos](https://github.com/aayoawoyemi/Ori-Mnemos) · Apache-2.0 · ~330 stars · v0.7

**Similarities:** markdown as the source of truth, wikilinks as edges, a derived SQLite index, ACT-R decay, Hebbian co-occurrence, spreading activation (Personalized PageRank), an MCP server, lifecycle hooks for Claude Code, Hermes, and OpenCode; MCP-only for Cursor/Codex. Essentially ~70% of our design already exists here.

**Worth adopting:**
- **NPMI normalization** for co-occurrence + **per-node homeostasis** (Turrigiano) so that hubs do not absorb all the weight.
- **Day-0 edge bootstrap from existing wikilinks** — solves the cold start.
- **Tarjan (articulation points)**: structurally important nodes (whose removal would disconnect the graph) are protected from archiving.
- **Per-space decay multipliers**: `self/` 0.1×, `notes/` 1×, `ops/` 3× — the equivalent of our per-type decay exceptions, but finer-grained.
- **Session outcome signals**: forward citation (+1.0), re-recall (+0.4) — the equivalent of our §3.4 table.
- **Capture → promote** (`ori add` → `ori promote`) — the same as our inbox.

**Lessons from their mistakes:**
- Learned state (Q-values, bandits) **cannot be rebuilt** from files — the author themselves corrected the claim that `.ori/` is disposable. Our principle P1 (the event log as the source) avoids this problem → **this is rewire's main differentiator**.
- 3 exploration MCP tools were **removed after 6 months because they were never used**. Keep the tool surface small.
- Multi-hop and temporal retrieval are weak (LoCoMo recall 0.53 / 0.57); the benchmarks are self-reported and have been corrected before.

**Strategic implication:** the *build vs adopt* question must be answered. Proposal: do not fork, but make Ori the **eval baseline**. rewire only deserves to exist if it wins on: multi-device portability (event sourcing), temporal validity (bitemporal + supersede), and a typed ontology.

### 3.2 FERNme — LLM-free discipline and honest evals
[github.com/mirkofr/FERNme](https://github.com/mirkofr/FERNme) · Apache-2.0 · research preview v0.4

- **`pure` mode: zero LLM calls for write and recall.** LLM enrichment only *proposes* into a human review queue. → Adopt: activation/Hebbian/tiers are computed without an LLM; the LLM is used only for consolidation extraction.
- Append-only event log (Cabinet) — validates P1.
- **Retention eval**: permanent facts stay above the threshold on day 700, while volatile facts fade quickly — exactly the metric we need to prove the decay exceptions.
- Honest finding: under *abrupt drift*, **pure recency beats** their model (1.000 vs 0.833). Add this scenario to the Phase 0 simulation.

### 3.3 AIAgentLocalMemory — concrete formulas & sync via an op-log
[github.com/jackieju/AIAgentLocalMemory](https://github.com/jackieju/AIAgentLocalMemory) · OpenCode plugin · early stage (1 star)

- Hebbian `Δw = η(1 − w)`, decay `w ← w·exp(−λΔt)`, pruning of weak edges; score `w_fts·BM25 + w_act·spread`.
- **Git sync sends only `operations.jsonl`**, not `graph.db` — a real implementation of our event sourcing pattern.
- **Working memory** (an LRU-frequency queue, score `freq·exp(−0.01·hours)`) — a concept that does not yet exist in our design; useful for very short-term memory *within* a session.

### 3.4 Relevant research

| Paper | Idea worth taking | Notes |
|---|---|---|
| **SYNAPSE** ([arXiv 2601.02744](https://arxiv.org/abs/2601.02744), Findings ACL 2026) | Spreading activation with **lateral inhibition** (the most active nodes suppress competitors) + fan effect + sigmoid, stable within 3 iterations. Better at multi-hop & temporal (LoCoMo). | A direct upgrade for §3.3 of our design. Code not yet released as of the search. |
| **HeLa-Mem** ([arXiv 2604.16839](https://arxiv.org/abs/2604.16839)) | **Hebbian distillation**: dense episodic hubs are turned into semantic knowledge. | Fits the "sleep" phase. Self-reported. |
| **RD-Forget** ([arXiv 2609.10263](https://arxiv.org/abs/2609.10263)) | Forgetting = *a view at query time*, not deletion; the source archive stays intact. | Validates P3/P5. |
| **ScrubJay-MEM** ([arXiv 2608.04746](https://arxiv.org/abs/2608.04746)) | Decay per memory type (perishability). | The gain narrows on strong models and **reverses** on fact consolidation tasks — a warning for decay. |
| **Filesystem-Based Memory** ([arXiv 2607.26637](https://arxiv.org/abs/2607.26637)) | Markdown memory in a directory tree: organization saves search cost, **not accuracy**, and **erodes** as the store grows. | Lint/sleep is mandatory, not optional. |
| **MemDelta** ([arXiv 2606.29914](https://arxiv.org/abs/2606.29914)) | The choice of embedding model can inflate architecture claims. | Ablations must lock the embedder. |
| **Letta sleep-time compute** | A separate background agent that rewrites memory blocks while the main agent is idle. | A production equivalent of our "sleep". |

**Benchmarks for eval:** STALE (whether the agent realizes a memory no longer holds; the best frontier model scores 55.2%), ForgetEval (forgetting, MIT), MemoryAgentBench (ICLR 2026, four competencies). Note: an independent audit found that the LoCoMo judge accepts up to 63% of deliberately wrong answers — do not make LoCoMo the primary metric.

> Most numbers in this section are **self-reported** by their authors. The papers in table 3.4 (except SYNAPSE) are summarized from [awesome-agent-memory](https://github.com/shimo4228/awesome-agent-memory) and I have not read them directly.

---

## 4. MVP proposal (v0.1)

In:
- Brain format: `nodes/`, `events/`, `sessions/`, `brain.yaml`; a rebuildable SQLite FTS5 index.
- ACT-R activation + per-type decay exceptions; **Hebbian deferred** until the simulation proves its benefit.
- CLI: `prime`, `recall`, `get`, `remember`, `consolidate`, `rebuild`.
- MCP: `recall`, `get`, `remember` (only 3 tools).
- Manual consolidation (`rewire consolidate <session>`) with an LLM, output to the inbox + review.
- 1 harness with hooks (Claude Code) + 1 without hooks (AGENTS.md + CLI).

Deferred: embeddings, Hebbian/associative edges, automatic sleep, `sources/` ingest, the PreToolUse push hook, multi-device, Obsidian.

## 5. Recommended next steps

1. Add to the design: the MVP (§4), the integration contract, the granularity decision, NPMI co-occurrence → the score is expected to rise to ~7.5.
2. Run the **Phase 0 simulation**, including the abrupt drift (FERNme) and hub (Ori) scenarios, with activation on/off ablation.
3. Install Ori-Mnemos on one real project for about 1 week as a user experience baseline.

---

## 6. Reassessment after rev 2 (MVP spec + Phase 0)

Changes: [mvp-spec.md](mvp-spec.md) and the [Phase 0](../../experiments/phase0/README.md) simulation.

| # | Dimension | Weight | Rev 1 | **Rev 2** | What changed it | Remaining gap |
|---|---|---|---|---|---|---|
| 1 | Goals & principles | 10% | 9 | **9** | — | — |
| 2 | Storage model | 15% | 8 | **9** | Node conflict rule, `merge=union` for events, single-device in v0.1 | — |
| 3 | Activation model | 10% | 7 | **8.5** | Phase 0: exposure=0 & decay exceptions mandatory in the synthetic simulation; project scope + separate budgets discovered and adopted; `d=0.7` tuned; Hebbian deferred | Evidence is still synthetic (closed by E4) |
| 4 | Ontology & granularity | 10% | 5 | **9** | 7 types / 5 relations; atomic claims vs hub pages, supported by Dense X Retrieval | — |
| 5 | Consolidation | 10% | 6 | **8** | 8-step contract, idempotency, watermark, gate rules | Extraction prompt quality not yet tested (closed by E1) |
| 6 | Harness integration | 15% | 4 | **8.5** | Hooks & transcripts per harness verified; lazy consolidation workaround; Agent Skill as a universal adapter | Codex hook/rollout details come from third-party sources and are still experimental |
| 7 | Security | 10% | 5 | **8.5** | Threat model, provenance cap, secret scan, `undo` | Mitigations not yet tested against real attacks |
| 8 | Evaluation | 10% | 6 | **9** | E0 done; E1–E4 with data, metrics, go thresholds | — |
| 9 | MVP scope | 10% | 4 | **9.5** | Firm in/deferred line, 5 milestones | — |

**Rev 2 weighted score: 8.8 / 10** (rev 1: 6.0)

### Verdict

> **Ready to start building M1–M2** (storage, activation, CLI/MCP). This part does not depend on anything that is still unproven.

### Path to ≥ 9

The remaining gaps **cannot be closed by writing documents** — they need real data, and the transcripts are on the user's machine, not in this environment:

1. **E1 earlier (a spike before M3):** run the extraction prompt on 20 of the user's real Claude Code + Codex transcripts, labeled manually. Pass → Consolidation 8 → 9.
2. **Verify the Codex format from real source/files** (`~/.codex/sessions/…/rollout-*.jsonl` and the hook payload in the Codex version in use). → Integration 8.5 → 9.

With those two spikes the score becomes **≈ 9.0**. Activation (8.5) and security (8.5) can only rise through E4 dogfooding — it is reasonable to close those *after* the build, not before.
