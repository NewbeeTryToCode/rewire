# rewire baseline: what a storage migration must preserve, and where the measured bottlenecks are (as of 2026-10-17)

All sources are repository files under `/home/user/rewire/` (paths given relative to that root). Nothing was run; numbers are copied from committed reports.

## 1. Architecture: source of truth vs derived data

### Takeaway
The brain is a folder of plain files in a private git repo (markdown nodes, append-only JSONL events, session markdown, skills, inbox); the SQLite database (FTS5 + vector BLOBs + activation cache) is explicitly derived, gitignored, never synced, and rebuildable at any time. The on-disk file format is the frozen contract; the index is "not part of the contract", so the index backend is already designed to be swappable without touching the brain.

### Cited Findings
- Layout: `nodes/<type>/<slug>-<id>.md` (memory), `events/<device>/YYYY-MM.jsonl` (usage log, `merge=union`, never conflicts), `sessions/YYYY-MM-DD/*.md` (session summaries), `skills/<name>/SKILL.md`, `inbox/*.md`, `.index/` ("SQLite FTS5, rebuilt automatically (gitignored)"), `.state/` (per-device watermark & queue, gitignored) — [README.md](README.md)
- "Every rewire implementation reads and writes exactly this. The SQLite index (`.index/`) is **not** part of the contract: it is derived and rebuilt from the files below." `.index/brain.db  derived (not synced)` — [docs/design/format.md](docs/design/format.md)
- Layers: L0 working memory (owned by the harness), L1 episodic (`sessions/<date>/<harness>-<session>.md`: summary + secret-redacted transcript cut into excerpts), L2 semantic (atomic typed claims, graph-linked, bitemporal, superseded instead of overwritten, ACT-R activation) and L2 procedural (skills, always reviewed). "The SQLite index in `.index/` is derived from L1 and L2 and can be deleted at any time." — [docs/design/architecture.md](docs/design/architecture.md)
- Design principles P1 "Store events, not weights" (weights always computed from the event log), P4 "Files are the source of truth; the database is only an index that can be rebuilt at any time", P5 activation controls order, not existence or truth — [docs/design/synaptic-memory.md](docs/design/synaptic-memory.md)
- Index schema (SCHEMA "9"): tables `nodes`, `edges`, `act` (activation cache), `files`/`evfiles`/`epfiles` (incremental-refresh bookkeeping: mtime/size, event-log byte offsets), `uses`, FTS5 virtual tables `fts` (nodes: title, body, keywords; `unicode61 remove_diacritics 2`) and `fts_ep` (episode excerpts), `episodes` (path, date, text, turn range, scope), `vec` and `epvec` (embedding BLOBs) — [src/rewire/index.py](src/rewire/index.py) lines 19–47
- Index is kept fresh incrementally: only changed node files re-parsed, only appended event-log bytes read (a rewritten log, e.g. after `git revert`, triggers a full rebuild), activation recomputed for touched nodes, full recompute every 5 minutes — [src/rewire/index.py](src/rewire/index.py) docstring; [experiments/perf/README.md](experiments/perf/README.md)
- Embeddings: WordLlama static embeddings, 256 dimensions, offline, "No server, no API, no download at run time: ~0.2 s to load, ~10 µs per text"; vectors are L2-normalised float32 stored as BLOBs and searched by brute-force matrix product in memory (numpy in Python) — [src/rewire/semantic.py](src/rewire/semantic.py); [src/rewire/index.py](src/rewire/index.py) `_matrix`, `_semantic_ranks`, `search_episodes`
- Storage-option analysis in the design: Markdown + JSONL + SQLite index "✅ Recommended" (portable, git, human-readable, zero servers, rebuildable); graph DB server (Neo4j/Memgraph) "❌ conflicts with 'own the brain'" (requires a server, brain locked in DB, hard to put in git); embedded graph DB (Kuzu) "⚠️ could be an optional index backend"; "Because the index is derived, **the backend can be swapped later** ... without moving the brain." Also: "A personal graph (thousands to tens of thousands of nodes) is far below the point where a dedicated graph DB is needed"; 2-hop traversal via recursive CTE in SQLite — [docs/design/synaptic-memory.md](docs/design/synaptic-memory.md) §4.4
- Token cost profile: `prime`, per-prompt hook, `recall`, `get`, index refresh = 0 LLM calls (SQLite FTS5 + activation + graph spreading + optional local WordLlama); consolidation ~2 LLM calls per session — [docs/design/architecture.md](docs/design/architecture.md)

### Inferences
- A migration that moves the *index* (FTS/vector/graph layer) to another engine is architecturally permitted by the contract; a migration that moves the *source of truth* (nodes/events/sessions) into a hosted DB would break P4, the frozen file contract, git history/undo, Obsidian readability and the merge-by-git sync model.
- Any replacement index must reproduce: BM25 with custom field weights (3.0, 1.0, 2.0), unicode61 diacritic folding + the project's own stemming/stopwords (`text.match_expression`), deterministic tie-breaks (path, turn) for Rust/Python parity, scope filtering, JSON-field filtering on `valid_from`, and incremental refresh from files.

### Gaps
- No committed measurement of `.index/brain.db` file size at any scale was found (grep of results/perf/docs found none).

## 2. Consolidation (write path)

### Takeaway
The only LLM step: extract atomic claims -> judge each against similar existing notes (NEW/SAME/UPDATE/SUPERSEDE/CONTRADICT) -> gate -> security scan -> write files + git commit; anything not explicitly user-stated, any constraint, and any supersede goes to the human-review inbox. Similar-note lookup for the judge uses the local FTS index.

### Cited Findings
- Pipeline "normalize -> filter -> extract (LLM) -> resolve (FTS + LLM judge) -> gate" — [src/rewire/consolidate.py](src/rewire/consolidate.py) line 3
- Judge actions: NEW, SAME, UPDATE (refines), SUPERSEDE (truth changed), CONTRADICT (unclear which is right); targets that don't resolve to an existing id fall back to NEW — [src/rewire/consolidate.py](src/rewire/consolidate.py) lines 194–199, 513–516
- Gate: auto-apply only if `confidence == "explicit"` and `provenance == "user"` and action in NEW/SAME/UPDATE and type != constraint; otherwise inbox — [src/rewire/consolidate.py](src/rewire/consolidate.py) `gate`
- "Provenance comes from transcript roles, never from the model"; consolidation runs in background after a session, lazily, resumable by watermarks — [docs/design/architecture.md](docs/design/architecture.md)
- Every consolidation can be reverted with `rewire undo <session-id>` (git revert per session) — [README.md](README.md); [docs/design/mvp-spec.md](docs/design/mvp-spec.md) §9
- Consolidation cost dominated by LLM (~10–17 s per session, background); non-LLM part 385 ms at 10,000 nodes — [experiments/perf/README.md](experiments/perf/README.md)
- Contrast documented in-repo: OpenHuman stores "nothing local; everything in CortexDB (closed server: events → facts → beliefs, 3072-d vectors)", extraction inside CortexDB is "67% of their model cost", and CortexDB "flagged 0% of planted conflicts in their own eval"; rewire claims as advantages "Local, plain files, git history and undo (OpenHuman needs a hosted closed engine...)" — [docs/research/openhuman-hermes.md](docs/research/openhuman-hermes.md)
- "OpenHuman v2 itself no longer has local retrieval: ranking and multi-hop run on the closed CortexDB server; only the client layer is open" — [experiments/eval/results/2026-10-14-locomo-nemotron-s20-analysis.md](experiments/eval/results/2026-10-14-locomo-nemotron-s20-analysis.md) §8

### Inferences
- A CortexDB-style backend that performs extraction server-side would replace rewire's reviewed, provenance-capped gate with server-side automatic writes; that conflicts with the human-review principle unless the server only stores/derives.

## 3. Recall (read path)

### Takeaway
Recall is hybrid and entirely local: nodes ranked by BM25 magnitudes plus a small semantic bonus (weight 0.15), episodes by RRF of BM25 and semantic (weight 0.25, k=60), plus time windows, ACT-R activation, 2-step spreading activation over edges, fact->source-excerpt seeding, query-aware condensing, session opener, doubled budget for aggregate questions, and optional `also` extra queries fused by rank.

### Cited Findings
- Node search: "BM25 keeps its magnitudes (they carry real signal); the semantic ranker adds a bonus that decays with its rank. Plain RRF flattened BM25 and hurt MRR"; `SEM_WEIGHT = 0.15`, `SEM_TOP = 10`; "The bundled static embeddings are much weaker than BM25 + keywords on their own (R@5 57% vs 90%)"; time-window bonus +0.5; superseded memories hand activation to replacement, shown only for history questions — [src/rewire/index.py](src/rewire/index.py) lines 610–700
- Episode search: RRF (`RRF_K = 60`) of FTS5 BM25 ranks (weight 1.0) and semantic ranks (`EP_SEM_WEIGHT = 0.25`), seeds from matched facts boost their source excerpt, overlapping excerpts merged, condensing, session opener once per session, top excerpt carries session summary — [src/rewire/index.py](src/rewire/index.py) `search_episodes`
- Why 0.25: at weight 1.0 "the static vectors pushed the best keyword match out of the top 10 (single-session-user all evidence 100% -> 95%)" — [experiments/eval/results/2026-10-16-longmemeval-improvements.md](experiments/eval/results/2026-10-16-longmemeval-improvements.md)
- Spreading activation: relation weights part_of/depends_on 0.8, related_to 0.6, about 0.5, contradicts 0.4, supersedes 0.2, hop decay 0.5 — [src/rewire/index.py](src/rewire/index.py) lines 20–23; iteration 3 in [docs/iterations.md](docs/iterations.md)
- Condensing is keyword-only now: "embedding every sentence at recall time cost ~4x the latency for the same evidence" — [src/rewire/index.py](src/rewire/index.py) line ~411; [experiments/eval/results/2026-10-17-personamem-haiku55.md](experiments/eval/results/2026-10-17-personamem-haiku55.md) ("sentence vectors in condensing, since dropped for latency")
- `also`: up to 4 extra queries written by the calling model, fused by rank; `refers_to` boosts a model-resolved period; prime injects budgeted snapshot; UserPromptSubmit adds ≤5 related unseen notes — [docs/design/architecture.md](docs/design/architecture.md)
- Iteration 6: on full LoCoMo hybrid 94.2% vs keywords-only 94.9%; "semantics does not help on LoCoMo ... so it stays optional" — [docs/iterations.md](docs/iterations.md)

### Inferences
- Vector quality is the acknowledged weak component (static WordLlama, 256-d); this is a model choice, not a storage limit — better embeddings could be stored in SQLite BLOBs just as well.

### Gaps
- No experiment in the repo compares a different vector store / ANN index; brute-force in-memory cosine is used.

## 4. Python and Rust implementations, parity, MCP, sync, security

### Takeaway
Rust is the main shipped implementation (single ~9 MB binary, bundled SQLite); Python is the reference. Both must produce byte-identical node files and identical recall, verified by conformance tests and, end to end, by identical A/B verdicts. Sync is plain git; security is enforced at write time and again at display time.

### Cited Findings
- Rust binary "~9 MB, no runtime; `prime` ~20 ms, recall ~20 ms", SQLite bundled; "node files are byte-for-byte identical and recall results are identical" (`experiments/conformance/recall_parity.py`) — [README.md](README.md)
- Iteration 9/10: recall identical 304/304 LoCoMo questions (keyword and hybrid) after deterministic tie-break; `upgrade` byte-identical on 509 nodes; prime 11 ms vs 97 ms, recall 10 ms vs 84 ms (Rust vs Python) — [docs/iterations.md](docs/iterations.md)
- "Rust C2 gives the same verdict as Python C2 on all 190 questions (72.2% both; 189/190 identical context sizes)" — [experiments/eval/results/2026-10-17-ab-rust-gated-expansion.md](experiments/eval/results/2026-10-17-ab-rust-gated-expansion.md)
- Code size: Python `src/rewire` ≈4,220 lines (index.py 758); Rust `rust/src` ≈7,020 lines incl. `index.rs`, `semantic.rs`, `mcp.rs`, `sync.rs` — counted with `wc -l` on the repo
- MCP server exposes `recall`/`get` (and `remember`); Claude Code hooks SessionStart/UserPromptSubmit/SessionEnd/PreCompact; Codex via skill + AGENTS.md — [README.md](README.md)
- Sync: events one file per device per month with `merge=union` (never conflict); nodes conflict -> most recently changed wins, the other version to inbox; index never synced, every device rebuilds — [src/rewire/sync.py](src/rewire/sync.py); [docs/design/mvp-spec.md](docs/design/mvp-spec.md) §8
- Security: secret regexes + high-entropy token redaction of transcripts before LLM or `sessions/`; `<private>` spans stripped; injection patterns (ignore previous instructions, exfiltrate, curl/wget, invisible/bidi chars) — [src/rewire/security.py](src/rewire/security.py). Injected memory fenced in `<rewire-memory>`, rescanned at display (`[BLOCKED]`) — [README.md](README.md). Threat model: provenance cap (tool-only evidence never auto-applied), `doctor` warns on a public remote, "no remote by default", encryption at rest deferred — [docs/design/mvp-spec.md](docs/design/mvp-spec.md) §1, §9

### Inferences
- Any index migration must be done twice (Rust + Python) and pass the parity suite; a hosted backend would add a network dependency to the session-start hook (which has an 8-second sync budget) and to MCP recall.

## 5. Stated design principles and non-goals

### Takeaway
Local-first, user-owned, serverless, git-versioned, human-readable/editable, human review by default; prompt management belongs to the harness.

### Cited Findings
- "Own the brain, transfer anywhere: the brain remains a set of files owned by the user. It can be put in git, it can be read by humans, and it does not require a server." — [docs/design/synaptic-memory.md](docs/design/synaptic-memory.md)
- "Files are the source of truth"; "Safe by default"; Obsidian-readable; "Every memory file can be edited directly; the index adjusts itself" — [README.md](README.md)
- Boundaries: "rewire never manages the prompt; L0 belongs to the harness"; "The on-disk format is the contract ... the index is not part of the contract"; "Memory is data, not instructions" — [docs/design/architecture.md](docs/design/architecture.md)
- v0.1 scope deferred embeddings ("FTS5 is enough for a personal brain's size; add them when recall proves insufficient") and encryption at rest — [docs/design/mvp-spec.md](docs/design/mvp-spec.md) §1
- Evaluation plan wants Mem0/Graphiti compared run locally "without their cloud services, so the versions are pinned and the runs are repeatable"; notes Graphiti needs Neo4j/FalkorDB, Mem0 a vector store — [docs/evaluation.md](docs/evaluation.md) §4

### Gaps
- No explicit "non-goals" heading exists; non-goals are inferred from the storage-options table, the Boundaries section and the v0.1 deferral table.

## 6. Measured results (end to end and retrieval)

### Takeaway
LongMemEval_S (500 q, Haiku 5.5 answerer / Haiku 4.5 judge, `episodes` mode) went 78.4% -> 83.6% (A) -> 85.6% (B) purely through recall pipeline changes, with all-evidence rising 85.7% -> 91.1% -> 98.3%. In sampled four-benchmark A/Bs the best arm (C2, selective extra queries) is 72.2% over 190 questions; Rust reproduces Python exactly.

### Cited Findings
- LongMemEval_S all 500: baseline 78.4% (abstention 76.7%), A 83.6%, B 85.6% (abstention 73.3%); all evidence 85.7% / 91.1% / 98.3%; evidence turns 86.2% / 94.4% / 98.1%; MRR 0.857 / 0.892 / 0.853; context 8.8k / 12.5k / 19.1k tokens; answer prompt 13.6k / 18.4k / 27.4k tokens; retrieval p50/p95 21/56 ms, 235/651 ms, 1,119/3,404 ms; cost $3.37 / $3.89 / $4.96; J when all evidence present 88.1% / 89.3% / 87.7% — [experiments/eval/results/2026-10-16-longmemeval-improvements.md](experiments/eval/results/2026-10-16-longmemeval-improvements.md)
- Per category (baseline -> A -> B): multi-session 65.4 -> 75.2 -> 79.7; temporal 78.9 -> 86.5 -> 88.0; single-session-preference 43.3 -> 50.0 -> 56.7; knowledge-update 93.6 -> 92.3 -> 89.7 — same source
- "Retrieval is no longer the bottleneck in B: 98.3% of questions have every evidence session, yet J is 85.6%... The remaining errors are answering and judging... A stronger answerer, or a judge with the official GPT-4o, is the next lever, not more recall." "More context costs abstention and knowledge-update" — same source
- Four-benchmark sampled A/B (190 q): base 70.2%, A 70.9%, B 67.3%; LoCoMo 88/88/88, LongMemEval 82/90/82, BEAM 100K 47/39/54, PersonaMem 32k 57/54/47; B "is not a safe default" (hurts preference/generative questions) — [experiments/eval/results/2026-10-17-ab-four-benchmarks.md](experiments/eval/results/2026-10-17-ab-four-benchmarks.md)
- Selective expansion: A 70.9%, C 70.6%, D (oldest-first) 70.4%, C2 72.2% (4 better / 1 worse vs A); expansion fires on 13/40 LoCoMo, 30/60 LME, 6/20 BEAM, 0/70 PersonaMem — [experiments/eval/results/2026-10-17-ab-selective-expansion.md](experiments/eval/results/2026-10-17-ab-selective-expansion.md)
- Rust C2/C3/C4: 72.2% / 71.8% / 72.3% on 190 q; LoCoMo extra multi-hop sample all evidence 32% -> 50% with model-gated expansion, J 92% -> 95%; "What would move J now is not more recall: LongMemEval has 97–98% all evidence in every arm with J 88–90%; the remaining misses are answering (counting slips, self-contradicting answers) and BEAM's retrieval-heavy categories (information extraction 0% evidence in every arm)" — [experiments/eval/results/2026-10-17-ab-rust-gated-expansion.md](experiments/eval/results/2026-10-17-ab-rust-gated-expansion.md)
- Retrieval tokens (Rust C2, 190 q): memory context mean 11,562 tokens, answer prompt mean 17,220; expansion LLM tokens ~67+18 per question (rule) vs ~390+151 (model-gated); per benchmark C2 memory context mean/p95: LoCoMo 2,769/5,294; LME 15,062/29,025; BEAM 17,945/48,547; PersonaMem 11,762/18,506 — same source
- PersonaMem: 32k all 589 = 59.3%, 128k (140 q) 55.7%, 1M (140 q) 51.4%, context ~11k tokens at 3.1× / 14.5× / 99.4× compression; retrieval p50/p95 42/181, 57/214, 77/220 ms; weak on `suggest_new_ideas` (20–26%) and `generalizing_to_new_scenarios` (30–46%), which "need the user's whole preference profile ... the case for `--mode full` ... or a profile summary, not for more excerpts" — [experiments/eval/results/2026-10-17-personamem-haiku55.md](experiments/eval/results/2026-10-17-personamem-haiku55.md)
- LoCoMo: full 1,540 q J 94.0–94.9% (Claude via `claude -p`, Mem0 prompt; not same model as Mem0); retrieval-only all evidence 77.5% overall, multi-hop 32.3%, open-domain 40.2%, p50 4 ms / p99 16 ms — [docs/evaluation.md](docs/evaluation.md) §4, §8
- Older reference: LongMemEval 85.4% at 10 excerpts in iteration 8 (Claude, model not recorded); "the 7-point gap comes from the answerer/judge pair, not from memory" since retrieval was identical — [experiments/eval/results/2026-10-16-longmemeval-haiku55-ep10-analysis.md](experiments/eval/results/2026-10-16-longmemeval-haiku55-ep10-analysis.md)
- Comparability caveat: numbers use Haiku judge, not official GPT-4o; judge variance ±1–3 points; 95% CI on 190 q ≈ ±6.5 points — [docs/evaluation.md](docs/evaluation.md) §6; four-benchmark report

### Inferences
- Gains of +7.2 points on LME came entirely from recall-pipeline logic (excerpt windowing, condensing, budgets, query expansion) running on the unchanged SQLite storage.

## 7. Where errors come from (per-question diagnoses) and whether storage is implicated

### Takeaway
At the 78.4% baseline, 63% of wrong answers were attributed to retrieval, but the causes were ranking/budget/excerpt-windowing/vocabulary mismatch — all subsequently addressed in pipeline code, after which retrieval reached 97–98% all-evidence on LME. No diagnosis attributes errors to storage scale, database limits, or lack of a server-side graph/temporal model; remaining errors are answering, judging, vocabulary gaps (embedding quality), and BEAM categories with 0% evidence.

### Cited Findings
- Baseline 108 wrong answers: retrieval 68 (63%) = not recalled 37, ranked just below budget 20, answer turn trimmed/condensed away 10; answerer 30 (28%) = over-abstaining 9, wrong detail 8, generic preference advice 8, date arithmetic 2, counting 2, invented `_abs` 1; judge 8 (7%); dataset 2 — [experiments/eval/results/2026-10-16-longmemeval-haiku55-ep10-analysis.md](experiments/eval/results/2026-10-16-longmemeval-haiku55-ep10-analysis.md)
- J 88.1% when every evidence session in context vs 20.9% when one missing; 32 missing evidence sessions ranked 11–20, 23 ranked 21–60, 8 outside top 60 — same source
- Ranked fixes: query-aware excerpt selection (≈15–20 q), larger budget for aggregation (≈19–30 q), category->instance query expansion (≈12–15 q), self-attributes buried in off-topic sessions (≈6 q; "`--mode full` fact extraction ... is the structural fix"), preference questions, answerer behaviour, judge strictness — same source
- LoCoMo multi-hop: entity route in RRF neutral; second hop via entities worse (71.3/80.4 overall vs 77.5/82.9); "Why it does not work on LoCoMo: almost all of the failing multi-hop questions are aggregation questions whose vocabulary differs from their evidence"; what helps is `full` mode facts + source excerpts (multi-hop 51.4% at 4.7k tokens); next lever "A small contextual encoder (bge-small / e5-small, ONNX) replacing WordLlama, and/or a small cross-encoder" blocked because models could not be downloaded — [experiments/eval/results/2026-10-14-locomo-nemotron-s20-analysis.md](experiments/eval/results/2026-10-14-locomo-nemotron-s20-analysis.md) §8
- Evidence depth table: LoCoMo multi-hop all evidence 32.3% at 10 excerpts, 79.8% at 100, 99.6% at 200 — evidence exists, ranked deep — same source
- CortexDB's 79.8% multi-hop is "J ... a vendor claim, not all evidence in context" — same source §9
- 2026-10-17 conclusion: LME 97–98% all evidence, misses are answering; BEAM information extraction 0% evidence in every arm, event ordering and summarization 0% all-evidence — [experiments/eval/results/2026-10-17-ab-rust-gated-expansion.md](experiments/eval/results/2026-10-17-ab-rust-gated-expansion.md)
- Knowledge-update degrades with larger context (92.3 -> 89.7, "more stale values next to the new one") — [experiments/eval/results/2026-10-16-longmemeval-improvements.md](experiments/eval/results/2026-10-16-longmemeval-improvements.md)
- Benchmarks were run in `episodes` mode (no LLM extraction, so no supersede/contradict graph in play); BEAM and PersonaMem support only `episodes` — [docs/evaluation.md](docs/evaluation.md) §2

### Inferences
- The "lack of graph/temporal model" hypothesis is only weakly tested: benchmarks ran with transcripts only, bypassing rewire's own L2 graph/bitemporal nodes; the one graph-like experiment (entity second hop) hurt. Temporal/knowledge-update errors are attributed to context size and answerer, not missing storage features.
- BEAM's 0%-evidence categories (information extraction, summarization, event ordering) on a 20-question sample are the only clear remaining retrieval gap; the reports do not attribute them to storage.

### Gaps
- No per-question diagnosis for PersonaMem (multiple choice, no evidence annotation) or BEAM beyond 2 questions per category.
- No `full`-mode (extracted facts + supersede) run on LongMemEval/BEAM at scale, so the contribution of the L2 graph to these benchmarks is unmeasured.

## 8. Latency, index size and scaling

### Takeaway
Recall is tens to a few hundred ms locally; the regressions seen came from recall-time embedding work (condensing) and multiple-query expansion, not from SQLite. Rust CLI overhead of 96 ms is process start + WordLlama model load, avoided by a warm MCP server. Perf tested up to 10,000 nodes / 100,000 events / 20,000 relations.

### Cited Findings
- Rust latency method: wall time minus "a fixed 96 ms (process start + WordLlama load, measured on an empty brain; 6 ms without semantic). A long-running MCP server loads the model once"; Rust recall p50 7–165 ms in every arm; C2 p50: LoCoMo 14 ms, LME 156 ms, BEAM 73 ms, PersonaMem 10 ms; p95 up to 404–549 ms on BEAM — [experiments/eval/results/2026-10-17-ab-rust-gated-expansion.md](experiments/eval/results/2026-10-17-ab-rust-gated-expansion.md)
- Prefix matching by binary search cut extra-query recall p50 169 -> 134 ms — git log commit 8eb9bfa (`git log --oneline`)
- A-arm latency cause: "condensing embeds every sentence of every long line at recall time (A: p50 21 -> 235 ms), and B runs it for five queries (1.1 s). Fix: cache sentence vectors in the index ... or condense by keywords only" — [experiments/eval/results/2026-10-16-longmemeval-improvements.md](experiments/eval/results/2026-10-16-longmemeval-improvements.md); later fixed by keyword-only condensing — [src/rewire/index.py](src/rewire/index.py)
- Perf at 10,000 nodes (Python): session-start hook 168 ms, recall CLI 95 ms, recall via warm MCP 34 ms, get 25 ms, full rebuild 1,300–1,600 ms; 16 ms MCP recall at 10,000 memories with 20,000 relations (iteration 3); before incremental index, hook took 1,240 ms — [experiments/perf/README.md](experiments/perf/README.md); [docs/iterations.md](docs/iterations.md)
- Scale reached in evals: LongMemEval_S ≈115k tokens per question history; PersonaMem 1M ≈925k tokens per context, retrieval p50 77 ms / p95 220 ms; BEAM 100K chats ≈146k tokens; BEAM 500K/1M exist in the runner but only 100K was run; BEAM 10M "not supported yet" — [docs/evaluation.md](docs/evaluation.md) §2, §8; [experiments/eval/results/2026-10-17-personamem-haiku55.md](experiments/eval/results/2026-10-17-personamem-haiku55.md)
- Accuracy vs scale: "Accuracy falls slowly with history size (59% → 56% → 51%) while the context stays at ~11k tokens: retrieval keeps up with a 99× compression at 1M" — same PersonaMem source
- Design statement on scale: personal graph of thousands to tens of thousands of nodes "far below the point where a dedicated graph DB is needed" — [docs/design/synaptic-memory.md](docs/design/synaptic-memory.md)
- LLM latencies dwarf retrieval: answer p50 8.3 s / p95 13.5 s, judge p50 7.4 s (via `claude -p`); consolidation ~10–17 s/session — [experiments/eval/results/2026-10-16-longmemeval-haiku55-ep10-analysis.md](experiments/eval/results/2026-10-16-longmemeval-haiku55-ep10-analysis.md); [experiments/perf/README.md](experiments/perf/README.md)

### Inferences
- A hosted backend would add network round-trips to a path that currently costs ~10–160 ms p50; the dominant costs (LLM answer/judge seconds, model load 90 ms) are not storage.
- Brute-force vector search is fine at measured scales; it would become a concern only at far larger corpora (e.g. BEAM 10M), which is untested.

### Gaps
- No measured `.index/brain.db` size, no perf test beyond 10,000 nodes, and no BEAM 500K/1M/10M runs; scaling of brute-force vector search beyond these is unmeasured.
- No measurement of hosted/remote-backend latency or of CortexDB itself in the repo (CortexDB is closed; only vendor claims are cited).
