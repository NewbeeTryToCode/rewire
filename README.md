# rewire

**Own the memory, take it anywhere.** Long-term memory for coding agents (Claude Code, Codex) that lives in a git
repo of markdown files you own, and that you can read, edit, review, sync and undo.

- **Episodic memory with no LLM bill.** Every session's transcript is kept (secrets redacted, `<private>` parts
  removed) and searched with hybrid retrieval: SQLite FTS5 keywords plus optional offline static vectors. Recall
  makes no LLM calls; the benchmark gains measured so far all come from this layer (`docs/evaluation.md`).
- **A reviewed layer of facts, rules and skills on top.** After a session, an LLM proposes atomic notes and reusable
  procedures. What the user explicitly said is saved; everything else waits in an inbox. Rules (constraints) always
  go through review.
- **Files are the source of truth.** A note is markdown with frontmatter (Obsidian-friendly); usage is an
  append-only event log; the index is derived and rebuilt from the files at any time.
- **Safe by default.** Provenance comes from the transcript and the cited turn must actually say the claim; tool
  output never becomes a rule on its own; secrets are refused; memory is fenced as data and re-scanned when shown;
  every session can be taken back with `rewire undo`. What runs the LLM is set per device, never synced.
- **Experimental, off by default:** ordering by usage (ACT-R decay over the event log, `recall_beta`) and spreading
  activation over every note link (`spread = "full"`). Spreading measured +0.0 points; usage ordering has not been
  measured at all. By default recall only follows a link from an outdated note to its replacement.

Architecture: [`docs/design/architecture.md`](docs/design/architecture.md) · File format: [`docs/design/format.md`](docs/design/format.md) · Design: [`docs/design/mvp-spec.md`](docs/design/mvp-spec.md) · Background and hypotheses: [`docs/design/synaptic-memory.md`](docs/design/synaptic-memory.md) · Iteration log: [`docs/iterations.md`](docs/iterations.md) · Evaluation: [`docs/evaluation.md`](docs/evaluation.md)

## Installation

Download a release binary (Linux x86_64, macOS arm64/x86_64) from the GitHub releases page and put `rewire` on
your PATH, or build it:

```bash
cargo install --locked --git https://github.com/NewbeeTryToCode/rewire rewire   # one binary, SQLite bundled
rewire init                     # creates ~/brain (or set REWIRE_BRAIN)
```

Recall over MCP, with the server warm, takes a few ms on a small brain and tens of ms on a ~50-session,
~115k-token history (measured: [`experiments/eval/latency.py`](experiments/eval/latency.py)).

**Consolidation (the only LLM use)** runs after a session, in the background, about two calls per session. It is
configured per device in `~/brain/local.toml` (not synced): by default Claude Code's `claude -p` with Sonnet and
your existing login, with no tools, no MCP servers and no settings or hooks. To use the API directly, set
`provider = "anthropic"` there and export `ANTHROPIC_API_KEY`. Failed consolidations are reported by `rewire doctor`
and the session-start snapshot, and retried with `rewire consolidate --retry`.

Semantic search (optional, offline): `rewire semantic install` copies the WordLlama model from a local `wordllama`
Python package (`pip install wordllama`) or from `--from <dir>`. Without it recall uses keywords only (BM25 +
stemming); with it, recall is hybrid.

The benchmark harness in [`experiments/eval`](experiments/eval) is Python (dataset loading, LLM answering and
judging); it drives the Rust binary and has no implementation of its own.

## Connect to a harness

```bash
rewire install claude-code --write   # hooks + permission rules; an existing settings.json is backed up first
claude mcp add --scope user rewire -- rewire mcp

rewire install codex                 # writes an Agent Skill + prints an AGENTS.md snippet
codex mcp add rewire -- rewire mcp
```

The agent proposes memories and you approve them. `install claude-code --write` adds Claude Code permission
rules that deny the agent `rewire review approve`, `rewire remember --apply` and direct edits of the brain, and rewire
itself refuses approving, `--apply` and `forget` inside an agent session (detected by the agent's environment
variables or its process among the command's ancestors). These raise the bar, they are not a sandbox: an agent that
may run arbitrary shell commands acts as you.

With sync, rules, preferences and skills that arrive from another device are shown to agents on this device only
after you approve them here (`rewire review`); a pull that contains symlinks, submodules or device-local files is
refused, and a brain.toml change to what this device keeps (`transcripts`, `profile`) is not taken until you set it
in this device's `local.toml`.

Harnesses without hooks can still be used: the agent runs `rewire prime` via instructions in AGENTS.md/the skill, and its transcripts are consolidated *lazily* on the next hook or with `rewire consolidate --pending`.

## Day-to-day usage

```bash
rewire prime                      # snapshot injected at the start of a session
rewire recall "storage decision"  # search memory
rewire recall "deploy" --from 2026-09-01 --to 2026-09-30   # prefer a specific period
rewire get <id>                   # or: rewire get skill:<name>
rewire remember --type preference "Always write tests before features"   # goes to the inbox (scope: this project, or --scope global)
rewire review                     # view the inbox
rewire review approve <id|all>    # --force if the target note has changed since it was proposed
rewire forget <id>                # retract a note (it stays in git history)
rewire consolidate --pending      # process pending transcripts (--retry: the ones that failed)
rewire import sessions.jsonl      # import chat history (JSON lines); --llm also consolidates it
rewire sessions                   # list sessions, newest first
rewire undo <session-id>          # revert one session: its notes, proposals and searchable transcript
rewire doctor                     # check brain health (exit status 1 when something needs fixing)
rewire upgrade                    # rewrite memory to the latest format
rewire remote set <url>           # sync the brain through a PRIVATE git repo
rewire sync                       # manual pull + push
```

## Reading the brain as a human

- Open `~/brain/README.md` (on GitHub or in any editor): an always up-to-date home page with rules, preferences, projects, recent memory, the latest sessions, and proposals waiting for review.
- Open the `~/brain` folder as a vault in **Obsidian**: every memory is one note, relations show up in graph view and backlinks, and properties such as `status`, `type`, `scope` can be filtered.
- Every memory file can be edited directly; the index adjusts itself.

## Brain structure

```
~/brain/
├── README.md                     # home page for humans (generated automatically)
├── brain.toml                    # parameters, budgets, project map (synced)
├── local.toml                    # which LLM consolidates, on this device (never synced; a pull or
│                                 #   clone that tracks it is refused); can pin `transcripts` too
├── nodes/<type>/<slug>-<id>.md   # memory (constraint, preference, fact, decision, project, person, concept)
├── events/<device>/YYYY-MM.jsonl # usage log (merge=union, never conflicts)
├── sessions/YYYY-MM-DD/*.md      # summary of each consolidated session
├── skills/<name>/SKILL.md        # procedures (Agent Skills), via review
├── inbox/*.md                    # proposals waiting for review
├── .index/                       # SQLite FTS5, rebuilt automatically (gitignored)
└── .state/                       # per-device watermark & queue (gitignored)
```

## Development

```bash
cd rust && cargo test && cargo clippy --all-targets   # the implementation
python experiments/eval/run.py --help                 # benchmarks (LoCoMo, LongMemEval, BEAM, PersonaMem); see docs/evaluation.md
```
