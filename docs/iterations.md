# MVP iteration log

Each iteration: what was tested, with what data, what was found, and what was changed.

## Iteration 1 — 2026-10-10: build MVP v0.1 + first real test

**Built:** M1–M4 from the [mvp-spec](design/mvp-spec.md): brain format, FTS5 index, ACT-R activation, `prime/recall/get/remember`, MCP server (3 tools), Claude Code & Codex parsers, consolidation (extract → judge → gate → scan → write → commit), inbox, `undo`, Claude Code hooks, Agent Skill for Codex. 29 unit tests.

**Real tests (not fixtures):**

| Test | Data | Result |
|---|---|---|
| Consolidating a real transcript | The transcript of this rewire design session (Claude Code, 6 user turns, 131 turns in total, ~33 thousand characters after filtering) via `claude -p` | 1 LLM call, ~10 seconds. 5 claims auto-applied, 1 to the inbox |
| Real Claude Code hook | `claude -p --settings hooks.json` in another project folder | SessionStart injects the snapshot; the agent mentions preferences from memory. Project scope works (preferences of project `rewire` do not appear in other projects) |
| Fully automatic cycle | In that session the user said "untuk proyek ini saya selalu ingin test ditulis sebelum fitur" ("for this project I always want tests written before features") | SessionEnd → background consolidation → saved as a `preference` with scope `project:proj`, without intervention |
| Recursion protection | The consolidation's own `claude -p` session | Detected as `internal`, not consolidated |

**Findings → changes:**

| Finding | Change |
|---|---|
| Claims were written in English even though the user wrote in Indonesian | Extraction prompt: claims are written in the user's language. Retested: all claims were in Indonesian |
| The LLM created its own `project` nodes → duplicate project hubs | The `project` type was removed from the extraction schema; project descriptions become project-scoped `fact`s |
| Empty project hubs ("Project rewire.") consumed snapshot budget | Hubs are created without a body and are not displayed while empty |
| A hook could be missed if another consolidation was running (lock) | The process holding the lock rechecks the queue before finishing (+ test) |
| Self-review: truncated snapshot lines lost their ID; `remember` > 600 characters failed on approve; `review approve` without arguments approved everything | Fixed (+ tests) |

**Extraction quality (manual assessment of 1 session):** 6 of 6 claims were relevant and correct. 1 claim was not atomic (it combined two preferences), but it was held in the inbox because it was `inferred`. One session is not yet enough for metric E1 (target: 20 transcripts, precision ≥ 0.85).

## Iteration 2 — 2026-10-11: installed in a Codespace, sync, performance

**Real installation:** the user's Codespace, through a `claude remote-control` session instructed from a cloud session. 29 tests passed on Python 3.14, hooks + MCP registered, `rewire doctor` clean. The Codespace session found a bug: rerunning `install claude-code --write` overwrote the original settings backup → fixed (the first backup is never overwritten) + test.

**Cross-machine sync:** `rewire remote set`, `rewire init --from`, `rewire sync`, automatic pull at session start (8-second budget), automatic push after background consolidation, automatic clone from `REWIRE_REMOTE` on ephemeral machines. Per-device events never conflict; conflicting nodes → the newest version wins, other versions go to the inbox. The device name is now local per machine (previously it was synced in `brain.toml` → two machines could write to the same event folder). Tested with two brains + a bare remote (5 tests) and a real E2E (`claude -p` consolidation → push → clone on a second device → snapshot appears).

**Performance:** session-start hook 1,240 → 168 ms and MCP recall 908 → 34 ms at 10,000 memories, via an incremental index. Details in [experiments/perf](../experiments/perf/README.md).

## Iteration 3 — 2026-10-11: a human-readable brain + graph

Trigger: a comparison with Obsidian and graph DBs. The foundation (files as the source, derived index) was kept, but three shortcomings were fixed.

| Shortcoming | Change |
|---|---|
| When opened in Obsidian, the graph view was empty; relations were long IDs | Relations are written as `[[wikilink]]` properties (Obsidian) **and** relative markdown links in a `## Related` section (clickable on GitHub). Added `status`, `tags`, `aliases`; `source` became plain text. The old format is still readable; `rewire upgrade` rewrites it |
| No place to "see what's in the head" | A `README.md` at the brain root, regenerated automatically: rules, preferences, projects with their memories, what was recently learned, the latest sessions, and the inbox. Deterministic: it does not change if memory does not change |
| The relations table existed but was not used by recall | 2-step spreading activation from the text search results, with per-relation weights and a fan effect; results include `via` (the reason that memory also appeared). Project hubs do not flood the results |
| Consolidation only created `about` relations | The LLM may link claims to existing memories or to other claims in the same session (`part_of`, `depends_on`, `related_to`) |

Validation: frontmatter parsed with PyYAML (a standard YAML parser), all links in the README point to existing files, 43 tests passed, a real consolidation produced Indonesian claims + relations + a home page, and MCP recall stayed at 16 ms at 10,000 memories with 20,000 relations.

## Iteration 4 — 2026-10-11: measuring retrieval quality

New eval ([experiments/retrieval](../experiments/retrieval/README.md)): 70 memories + 60 questions created by a separate LLM, 6 categories. Overall R@5 77% → **100%**, MRR 0.63 → **0.91**, via stopwords + EN/ID stemming, stale facts that redirect to their replacements, bilingual keywords from consolidation, and query guidance for the agent. Important finding: the initial paraphrase number (70%) was spurious. The final result is optimistic because one model family created the data and the queries; the next validation must use user data.

## Iteration 5 — 2026-10-11: LoCoMo, compared with Mem0 and Zep

An end-to-end public benchmark ([experiments/locomo](../experiments/locomo/README.md)) with **Mem0's official answer prompt and judge**, 389 questions from 3 conversations. J: **69.7% → 91.3% → 96.7%** (Mem0 platform publishes 92.5% with the same judge). Three changes, each taken from how competitors work: claims are given an event date (temporal 38.5 → 93.8%), a `general` extraction profile for a personal second brain, and episodes (searchable transcripts, with secrets masked). Limitations: a subset, a lenient judge from the same family as the answering model, Mem0/Zep were not rerun.

## Iteration 6 — 2026-10-12: competitor insights + full LoCoMo

Implemented from the Mem0/Zep analysis: optional semantic search (WordLlama, offline, additive fusion with BM25), a time-window bonus ("in July 2023"), and history questions return labeled `superseded` facts. **Full** LoCoMo (10 conversations, 1,540 questions, official Mem0 judge): **94.2%** hybrid, **94.9%** keywords only, vs 92.5% for the Mem0 platform. Ablation: semantics does not help on LoCoMo (17 vs 29 differing questions, not significant) because episodes + keywords already cover it, so it stays optional. Remaining weakness: open-domain 75% — mostly speculative questions that need world knowledge, not memory. Bug found by the full run: `time_window` crashed on "2022-23".

## Iteration 7 — 2026-10-13: LoCoMo failure analysis

A new metric without an LLM ([evidence.py](../experiments/locomo/evidence.py)): whether LoCoMo's evidence turns make it into the context. Diagnosis: 57% of wrong answers already had all the evidence, so the error is in the answerer. Implemented: facts pull in the transcript chunks they came from (required a bug fix: the session id in the claim source was truncated to 8 characters). All-evidence-in-context 75.5 → 79.5%, temporal 81 → 90%, but full J stayed at 94.0% (noise ±3). Tried and discarded: 2× chunks for counting questions (caused miscounting) and a per-session chunk limit (lowered recall). Timeline rules were added to extraction, not yet measured. Conclusion: LoCoMo is saturated; next is LongMemEval.

## Iteration 8 — 2026-10-13: LongMemEval

LongMemEval_S (500 questions, official answer prompt and judge) in *episodes* mode (transcripts only, no LLM extraction): **85.4%**; retrieval without an LLM found the evidence sessions for 97% of questions with ~11k tokens of context out of ~115k. Weakness: multi-session counting questions *under*count because not all evidence sessions are retrieved (the opposite of LoCoMo). With 20 chunks: multi-session 73.6 → 83.5%, temporal 82.7 → 90.6%, combined ≈89.8%. MCP `recall` now allows up to 30 chunks and tells the agent when to request ~20.

## Iteration 9 — 2026-10-13: L0/L1/L2 architecture, frozen format, Rust port

- [architecture.md](design/architecture.md): L0 working memory (owned by the harness), L1 episodic (`sessions/`), L2 semantic (`nodes/` + `events/`), and where tokens are spent (only in consolidation).
- [format.md](design/format.md): the frozen file contract.
- [`rust/`](../rust): a module-by-module port, which became the main implementation. Conformance: 32 porting tests pass; `upgrade` writes node files and a home page byte-for-byte identical to Python (509 nodes); recall is identical for 304/304 LoCoMo questions (50 facts + 10 chunks, same order) after the tie-break was made deterministic in both implementations. `prime` 11 ms vs 97 ms, `recall` 10 ms vs 84 ms. Semantic vectors (WordLlama) were not ported: on LoCoMo keywords alone are just as good.

## Iteration 10 — 2026-10-13: OpenHuman & Hermes insights

All insights from the [OpenHuman/Hermes research](research/openhuman-hermes.md) except the benchmark KPI (#14), in Python and Rust:
a date on every snapshot line, a `<rewire-memory>` fence + loop guard in the transcript parser, extraction rules (do not store temporary failures / "tool X is broken"; preferences written declaratively), an **About the user** section with its own budget, session summaries indexed and attached to the top chunks, `refers_to` (a period resolved by the model, as a boost), a **per-prompt** hook (UserPromptSubmit, at most 5 notes the session has not seen yet), **skills** (procedural memory, always via review), rescanning on display (`[BLOCKED]`), `target_hash` on inbox items, a notice when memory is unavailable, escaping of the fence tag, and a "… N more not shown" line. Semantic search is now also in Rust (the `semantic` feature, with the model from `rewire semantic install`). Conformance: recall identical 304/304 (keyword **and** hybrid), `prime` and `upgrade` identical, consolidation prompts identical; 61 Python tests, 54 Rust tests.

## Next (iteration 11)

0. **Token-efficient evaluation**: measure the impact of iteration 10 (fresh-first rate, captured-but-not-surfaced) without a full consolidation replay.

1. **Full E1:** consolidate 20 of the user's real transcripts (Claude Code + Codex), label them manually, and measure precision & duplicates.
2. **Verify the Codex parser** with real `~/.codex/sessions/**/rollout-*.jsonl` files (the current fixture was built from third-party documentation).
3. **E4 dogfooding:** install on the user's machine for 2 weeks; monitor the snapshot hit rate (from `used_memory_ids` in `sessions/*.md`) and the number of inbox items per session.
4. Consider automatically splitting combined claims during judging.

## Iteration: BEAM (2026-10-16)

BEAM ([ICLR 2026](https://github.com/mohammadtavakoli78/BEAM)) tests what LoCoMo does not: instruction and preference following, contradictions, event order, summaries, and scale (100K–1M tokens). Retrieval only, all 1,800 questions: all evidence in context 44.8% at 100K, 30.5% at 500K/1M. End to end on 100K (Haiku answers): the Haiku judge is unusable for BEAM (its prompt has no question, so correct bare answers score 0; 20.6% vs 37.7% with Sonnet on the same answers). Implemented: transcript excerpts capped at 1,200 characters as well as 6 turns (BEAM all-evidence 47.0 → 54.1% at the same budget, LoCoMo unchanged, Python/Rust recall identical), end to end 37.7 → 40.4%; and BEAM `--mode full`, which scores 43.0% vs 36.5% for episodes on the same 80 questions. Per-question diagnoses with the judge's reason per rubric item: [`diagnose_beam.py`](../experiments/eval/diagnose_beam.py), results in [`experiments/eval/results/2026-10-16-beam-100k-summary.md`](../experiments/eval/results/2026-10-16-beam-100k-summary.md).

