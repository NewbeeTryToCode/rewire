# Architecture: L0 / L1 / L2 memory

rewire is a portable memory for agent harnesses (Claude Code, Codex, …). The brain is a folder of plain
files in a private git repository; any harness attaches to it through hooks, an MCP server or the CLI.

## Three memory layers

| Layer | What it is | Where it lives | Owned by |
|---|---|---|---|
| **L0 — Working memory** | the active prompt context of the current session, bounded by compaction | inside the harness | the **harness**; rewire only fills it (snapshot at session start, `recall` on demand) and saves it before it is lost (SessionEnd / PreCompact) |
| **L1 — Episodic memory** | what happened in each past session: a summary plus the secret-redacted transcript, cut into searchable excerpts | `sessions/<date>/<harness>-<session>.md` | rewire |
| **L2 — Semantic memory** | atomic, typed claims (constraint, preference, fact, decision, project, person, concept) linked in a graph; bitemporal; superseded instead of overwritten; strength from use (ACT-R activation) | `nodes/<type>/<slug>-<id8>.md` + `events/<device>/<month>.jsonl` | rewire |

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
- **L1 → L2**: consolidation runs in the background after a session (lazily, resumable by watermarks).
  An LLM extracts atomic claims with dates and keywords, a second call judges each against similar existing
  claims, and a gate decides what is applied automatically (explicit + user-stated + not a constraint + not
  a supersede) and what goes to the review inbox. Provenance comes from transcript roles, never from the
  model.
- **L2 / L1 → L0**: at session start `prime` injects the strongest constraints, preferences and working
  context within fixed character budgets. During the session the agent calls `recall` (facts, plus raw
  excerpts; facts pull in the excerpt they were extracted from) and `get` for full notes. Using a note
  appends an event, which raises its activation.

## Token cost profile

| Operation | LLM calls | Notes |
|---|---|---|
| `prime`, `recall`, `get`, index refresh | **0** | SQLite FTS5 + activation + graph spreading |
| consolidation | ~2 per session (extract per transcript chunk + one judge) | background, after the session |
| `remember` (agent proposal) | 0 | queued for human review |

Benchmarks are expensive only because they replay hundreds of sessions through consolidation and then use
an LLM to answer and to judge every question. Retrieval itself can be evaluated without any LLM
(`experiments/locomo/evidence.py`, `experiments/longmemeval/run.py --retrieval-only`).

## Boundaries

- rewire never manages the prompt; L0 belongs to the harness. This keeps rewire portable.
- The on-disk format ([format.md](format.md)) is the contract. Any implementation (the Python reference,
  the Rust binary) must read and write it identically; the index is not part of the contract.
- Memory is data, not instructions: snapshots and recall results are framed as notes about the user.

## Measured

| Benchmark | Result | Report |
|---|---|---|
| LoCoMo (1,540 questions, Mem0's judge) | 94.0–94.9% | [experiments/locomo](../../experiments/locomo/README.md) |
| LongMemEval_S (500 questions, official judge prompts), L1 only | 85.4% (10 excerpts); multi-session 83.5% and temporal 90.6% with 20 excerpts | [experiments/longmemeval](../../experiments/longmemeval/README.md) |

## Roadmap gaps

1. **L1 session summaries are not searchable** yet: only transcript lines are indexed. Indexing summaries
   gives a cheap "which session was about X" layer for cross-session questions.
2. **No explicit user profile in L2**: preferences, constraints and personal facts are spread over nodes.
   A profile view built from the most activated preference / constraint / person nodes would make a better
   L0 snapshot.
