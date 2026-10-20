# rewire

**Own the brain, transfer it anywhere.** Portable memory for agent harnesses: one "brain" that you own (a git repo of markdown files) that can be plugged into Claude Code, Codex, and other harnesses.

- **Files are the source of truth.** A memory node is markdown with frontmatter; it can be read, edited, diffed, and opened in Obsidian.
- **Synapses are computed, not stored.** Usage is recorded as append-only events; memory strength (ACT-R, `d=0.7`) is recomputed from the log. Rules and preferences never decay.
- **Learns after each session.** Transcripts are consolidated by an LLM into atomic claims; things the user stated explicitly are saved immediately, and the rest go to the inbox for review.
- **Safe by default.** Provenance is taken from the transcript (not from the model's claims), tool output never automatically becomes a rule, secrets are rejected, and every consolidation can be reverted with `undo`. Injected memory is always fenced in `<rewire-memory>` and rescanned when displayed (`[BLOCKED]` if it contains a secret or an injection).
- **Knows procedures, not just facts.** Procedures that worked (deploy, release, setup) are proposed as *skills* (`skills/<name>/SKILL.md`) and go to the inbox for review.

Architecture (L0/L1/L2): [`docs/design/architecture.md`](docs/design/architecture.md) · File format: [`docs/design/format.md`](docs/design/format.md) · Design: [`docs/design/mvp-spec.md`](docs/design/mvp-spec.md) · Concept: [`docs/design/synaptic-memory.md`](docs/design/synaptic-memory.md) · Simulation evidence: [`experiments/phase0`](experiments/phase0/README.md) · Iteration log: [`docs/iterations.md`](docs/iterations.md) · Evaluation: [`docs/evaluation.md`](docs/evaluation.md)

## Installation

```bash
cargo install --git https://github.com/NewbeeTryToCode/rewire rewire   # a single Rust binary, SQLite bundled
rewire init                     # creates ~/brain (or set REWIRE_BRAIN)
```

rewire is a single Rust binary ([`rust/`](rust/), no runtime). Recall over MCP, with the server warm, takes a few ms on a
small brain and tens of ms on a ~50-session, ~115k-token history (measured: [`experiments/eval/latency.py`](experiments/eval/latency.py)). Consolidation uses `claude -p` by default (your existing Claude Code login); to use the API directly, set `provider = "anthropic"` in `brain.toml` and set `ANTHROPIC_API_KEY`.

The benchmark harness in [`experiments/eval`](experiments/eval) is Python (dataset loading, LLM answering and judging); it drives the Rust binary and has no implementation of its own.

Semantic search (optional, offline): `rewire semantic install` copies the WordLlama model from a local `wordllama` Python package (`pip install wordllama`) or from `--from <dir>`. Without the model, recall uses keywords only (BM25 + stemming); with it, recall is hybrid (keywords + vectors).

## Connect to a harness

```bash
rewire install claude-code --write   # SessionStart/UserPromptSubmit/SessionEnd/PreCompact hooks (settings are backed up first)
claude mcp add --scope user rewire -- rewire mcp

rewire install codex                 # writes an Agent Skill + prints an AGENTS.md snippet
codex mcp add rewire -- rewire mcp
```

Harnesses without hooks can still be used: the agent runs `rewire prime` via instructions in AGENTS.md/the skill, and its transcripts are consolidated *lazily* on the next hook or with `rewire consolidate --pending`.

## Day-to-day usage

```bash
rewire prime                      # snapshot injected at the start of a session
rewire recall "storage decision"  # search memory
rewire recall "deploy" --from 2026-09-01 --to 2026-09-30   # prefer a specific period
rewire get <id>                   # or: rewire get skill:<name>
rewire remember --type preference "Always write tests before features"   # goes to the inbox
rewire review                     # view the inbox
rewire review approve <id|all>    # --force if the target note has changed since it was proposed
rewire consolidate --pending      # process pending transcripts
rewire import sessions.jsonl      # import chat history (JSON lines); --llm also consolidates it
rewire undo <session-id>          # revert the consolidation result of one session
rewire doctor                     # check brain health
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
├── brain.toml                    # parameters, budget, provider, project map
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
