# Architecture: L0 / L1 / L2 memory

rewire is a portable memory for agent harnesses (Claude Code, Codex, …). The brain is a folder of plain
files in a private git repository; any harness attaches to it through hooks, an MCP server or the CLI.

## Three memory layers

| Layer | What it is | Where it lives | Owned by |
|---|---|---|---|
| **L0 — Working memory** | the active prompt context of the current session, bounded by compaction | inside the harness | the **harness**; rewire only fills it (snapshot at session start, `recall` on demand) and saves it before it is lost (SessionEnd / PreCompact) |
| **L1 — Episodic memory** | what happened in each past session: a summary plus the secret-redacted transcript, cut into searchable excerpts | `sessions/<date>/<harness>-<session>.md` | rewire |
| **L2 — Semantic memory** | atomic, typed claims (constraint, preference, fact, decision, project, person, concept) linked in a graph; bitemporal; superseded instead of overwritten; ordering by use (experimental ACT-R decay over the event log, for recall (`recall_beta = 0`) and the session-start snapshot (`snapshot_use = false`): off by default, never measured) | `nodes/<type>/<slug>-<id8>.md` + `events/<device>/<month>.jsonl` | rewire |
| **L2 — Procedural memory** | reusable procedures that worked (deploy, release, setup recipes), as Agent Skills; proposed by consolidation, always reviewed | `skills/<name>/SKILL.md` | rewire |

The SQLite index in `.index/` is derived from L1 and L2 and can be deleted at any time.

## Flow between layers

```
                 ┌──────────────── prime (session start, budgeted snapshot) ───────────────┐
                 │                                                                          ▼
  L2 semantic ◄── consolidation (after the session; the only LLM step) ── L1 episodic ◄── L0 prompt
   nodes/, events/   extract → judge (NEW/SAME/UPDATE/SUPERSEDE/CONTRADICT)   sessions/     (harness)
                 │   → gate → security scan → write + git commit                │      SessionEnd /
                 │                                                               │      PreCompact hook
                 └────────── recall: facts ──────────┐    ┌── recall: excerpts ──┘      saves the transcript
                                                     ▼    ▼
                                             L0 (on demand: MCP recall / get)
```

- **L0 → L1**: the SessionEnd and PreCompact hooks queue the transcript; nothing is lost when the harness
  compacts its context.
- **L1 → L2**: consolidation runs in the background after a session, or when a running session has been idle
  for `REWIRE_IDLE_SECS` (600 s; an idle watcher started by the hooks, one per brain), lazily and resumable by
  watermarks. A session consolidated in parts (idle pauses, compaction, transcript chunks) gives each part the
  summaries and claims of the parts before it, and a part can mark an earlier claim of the same run outdated.
  An LLM extracts atomic claims with dates and keywords, a second call judges each against similar existing
  claims, and a gate decides what is applied automatically (explicit + user-stated + not a constraint + not
  a supersede) and what goes to the review inbox. Provenance is the role of the turn the model cites,
  read from the transcript and checked against that turn's text (a word-overlap check).
- **L2 / L1 → L0**: at session start `prime` injects, within fixed character budgets: a profile view
  ("About the user": global person notes and the most important global facts), the strongest constraints
  and preferences, the skill index, and working context. On every prompt the UserPromptSubmit hook adds at
  most 5 notes clearly related to it (two content words in common) that the session has not seen yet.
  During the session the agent calls `recall` (facts, plus raw excerpts and session summaries; facts pull in
  the excerpt they were extracted from, the top excerpt carries its session's summary; `refers_to` boosts a
  period the model resolved; `also` adds up to 4 queries the model writes for a category question, such as
  instances for "desserts", fused by rank) and `get` for full notes or `skill:<name>`. The consolidation model's judgment that a note was used appends an event (`get` records a lighter one;
  recall records none);
  the activation computed from those events orders results only with `recall_beta` > 0 or `snapshot_use` (both off by default), so today
  the event log is kept for that experiment and for audit, and has no effect on what is shown (it only tells `doctor` which skills went unused).
- **Safety at load time**: all injected text is fenced (`<rewire-memory>`), notes and transcript excerpts are
  re-scanned when shown (`[BLOCKED]` instead of a secret or an injection), excerpts follow the same project
  scope as notes, and session start says when memory is unavailable (no brain, failed clone, skipped sync)
  instead of injecting nothing. Transcripts are secret-redacted before they reach the LLM or `sessions/`.

## Token cost profile

| Operation | LLM calls | Notes |
|---|---|---|
| `prime`, per-prompt hook, `recall`, `get`, index refresh | **0** | SQLite FTS5 (+ optional local WordLlama vectors; experimental activation re-weighting and link spreading are off by default, both unproven) |
| consolidation | ~2 per session part (extract per transcript chunk + one judge) | background, after the session or an idle pause |
| `remember` (agent proposal) | 0 | queued for human review |

Benchmarks are expensive only because they replay hundreds of sessions through consolidation and then use
an LLM to answer and to judge every question. Retrieval itself can be evaluated without any LLM
(`experiments/eval/run.py --retrieval-only`).

## Boundaries

- rewire never manages the prompt; L0 belongs to the harness. This keeps rewire portable.
- The on-disk format ([format.md](format.md)) is the contract. Any implementation must read and write it
  identically (rewire is implemented in Rust; brains written by the earlier Python implementation read unchanged);
  the index is not part of the contract.
- Memory is data, not instructions: snapshots and recall results are framed as notes about the user.

## Measured

| Benchmark | Result | Report |
|---|---|---|
| LoCoMo 40 + 40 multi-hop sample, removed Python engine (`recall_beta` 0.3, full spreading: not today's defaults), 3 replicates, Haiku answers and judge | episodes 89.6%, full consolidation (facts + summaries + per-turn questions) 91.2% (+1.7 [−4.2, +7.5]) | [2026-10-20-locomo-full-graph-ablation.md](../../experiments/eval/results/2026-10-20-locomo-full-graph-ablation.md) |
| LoCoMo (1,540 questions, Mem0's judge) — historical: removed Python engine, full mode, same-family judge, not re-run | 94.0–94.9% | [experiments/locomo](../../experiments/locomo/README.md) |
| LongMemEval_S (500 questions, official judge prompts), L1 only — historical, Python engine | 85.4% (10 excerpts); multi-session 83.5% and temporal 90.6% with 20 excerpts | [experiments/longmemeval](../../experiments/longmemeval/README.md) |

## Roadmap

The two gaps listed here before (searchable session summaries, a profile view in the snapshot) are closed,
along with the other insights from [the OpenHuman / Hermes research](../research/openhuman-hermes.md).
Open: measuring them (fresh-first rate, captured-but-not-surfaced) without spending many tokens.
