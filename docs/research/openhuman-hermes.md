# OpenHuman and Hermes Agent: memory, and what rewire should adopt

Sources: [tinyhumansai/openhuman](https://github.com/tinyhumansai/openhuman) (memory engine: its `tinymemory`
submodule + the closed CortexDB server) and [NousResearch/hermes-agent](https://github.com/NousResearch/hermes-agent)
(commit 8f817911), read on 2026-10-13. Key claims were spot-checked against the code.

## How they work, in one table

| | OpenHuman | Hermes Agent | rewire |
|---|---|---|---|
| Storage | nothing local; everything in CortexDB (closed server: events → facts → beliefs, 3072-d vectors) | `MEMORY.md` (2,200 chars) + `USER.md` (1,375 chars), entries split by `§`; SQLite FTS5 of all messages; skills as files | markdown nodes + events + sessions in git |
| Write | every turn logged; extraction inside CortexDB (67% of their model cost); "beliefs" built every 10 turns | agent edits memory with a tool during the turn; background review fork every 10 turns / 10 tool loops; weekly skill curator | after the session: extract → judge → gate → inbox |
| Read | per-turn "memory pack" (1,200 tokens, sectioned, prefetched, injected outside the transcript) | frozen snapshot at session start (stable prompt cache); `session_search` without LLM | snapshot at session start + MCP `recall` |
| Procedural memory | – | **skills** (`SKILL.md`), created and patched by the review, curated (stale 14 d, archive 30 d) | – |
| Conflicts | CortexDB; flagged 0% of planted conflicts in their own eval | exact-duplicate removal only | NEW/SAME/UPDATE/SUPERSEDE/CONTRADICT judge + review |

## Insights worth adopting (prioritised)

Status (2026-10-13): 1–13 are implemented in both implementations (iterations 9–10 in
[docs/iterations.md](../iterations.md)); 14 is part of the benchmark work.

| # | Insight | From | Value | Cost |
|---|---|---|---|---|
| 1 | **Date on every injected item** (`valid_from`, session date on excerpts): stale-first answers on contradictions went to 6/6 correct once turns carried timestamps | OpenHuman eval (`tinymemory/docs/evals/agent-memory.md`) | high | trivial |
| 2 | **Loop guard**: strip injected memory (snapshot / recall blocks) from transcripts before extraction and indexing, so memory never returns as a "user-stated" claim | OpenHuman 500-turn echo test | high | low |
| 3 | **Do-not-capture list + declarative phrasing** in the extraction prompt: no env-dependent failures, no "tool X is broken", no transient errors or unresolved dead ends; preferences/facts declarative ("User prefers…"), only constraints imperative | Hermes `background_review.py`, `prompt_builder.py` | high | trivial |
| 4 | **User profile view** rendered from top constraint/preference/person nodes, injected first with its own budget; derived summaries help only if retrieval reads them | Hermes `USER.md`; OpenHuman "beliefs built but +0pp until surfaced" | high | low-medium |
| 5 | **Per-prompt recall** (UserPromptSubmit hook): small block (300–600 tokens) of the most relevant nodes, skipping what the snapshot already showed; local FTS takes milliseconds | OpenHuman `pre_turn` pack | high | low-medium |
| 6 | **Procedural memory (skills)**: `skills/<class>/SKILL.md` in the brain, proposed by consolidation, patched before created, reviewed, archived when unused (activation) | Hermes skills + curator | high | medium |
| 7 | **Index session summaries**, and give the top excerpt hit its session's goal/resolution | Hermes `session_search` bookends | medium | low |
| 8 | **Load-time security re-scan**: brain files are hand-edited and synced; a flagged node shows as `[BLOCKED]` in the snapshot instead of silently vanishing | Hermes `threat_patterns.py` | medium | low |
| 9 | **Pin staged edits**: an inbox UPDATE/SUPERSEDE records the target's hash and refuses to apply if the node changed meanwhile | Hermes `_pin_matched_entries` | medium | low |
| 10 | **Say when memory is unavailable** (no brain, locked, sync failed) instead of injecting nothing | OpenHuman `TurnPack::refused` | medium | trivial |
| 11 | **Recalled content as untrusted data**: escaped `<memory-context>` / `<untrusted-source>` fence | both | medium | low |
| 12 | **Model-resolved time hints** (`refers_to` on `recall`, boost not filter; cover both readings of ambiguous words like "kal") | OpenHuman `date_hint.rs` | medium | low |
| 13 | Snapshot header shows budget use and what was cut ("[67% 1,474/2,200]") | Hermes | low-medium | trivial |
| 14 | Eval KPIs: fresh-first rate, captured-but-not-surfaced | OpenHuman | medium | low |

## What rewire already does as well or better

- Local, plain files, git history and undo (OpenHuman needs a hosted closed engine; Hermes stores flat text).
- Typed atomic claims with a conflict judge, bitemporal history and review-by-default (Hermes: exact dedup,
  background writes on by default; CortexDB: 0% planted conflicts flagged).
- Retrieval: stemming, time/history intent, spreading activation, fact → source-excerpt (Hermes session search
  is plain BM25).
- Multi-device: per-device append-only events + git sync (Hermes warns against two writers on one home).
- Standard, larger benchmarks (LoCoMo, LongMemEval) vs their smoke-sized evals.

Not worth copying: Hermes's turn-count nudges and "memory full, consolidate now" errors (artefacts of having no
retrieval layer and never-ending chats); OpenHuman's team scopes, credits and job queue.
