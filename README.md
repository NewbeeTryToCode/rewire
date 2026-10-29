# rewire

**Own the memory, take it anywhere.** Long-term memory for coding agents (Claude Code, Codex) that lives in a git
repo of markdown files you own, and that you can read, edit, review, sync and undo.

- **Episodic memory with no LLM bill.** Every session's transcript is kept (secrets redacted, `<private>` parts
  removed) and searched with hybrid retrieval: SQLite FTS5 keywords plus optional offline static vectors. Recall
  makes no LLM calls, and transcripts are kept even when the LLM step fails or is not set up (once consolidation
  succeeds, LLM-written questions per turn are indexed with the excerpts too, which the benchmarks' episodes-only
  arm did not have). The benchmark gains
  measured so far all come from this layer (`docs/evaluation.md`), but on chat benchmarks that keep every speaker's
  words; the default coding profile keeps only the user's turns (`transcripts = "all"` keeps the agent's too), and
  no coding-agent memory benchmark has been run.
- **A reviewed layer of facts, rules and skills on top.** After a session, an LLM proposes atomic notes and reusable
  procedures. What the user explicitly said is saved; everything else waits in an inbox. Rules (constraints) always
  go through review. Its value is readable, editable, reviewable memory: on answer accuracy full consolidation (facts,
  session summaries and per-turn questions together) has not been shown better than the episodes alone (+1.7 points,
  interval [−4.2, +7.5], 80 questions, earlier Python engine).
- **Files are the source of truth.** A note is markdown with frontmatter (Obsidian-friendly); usage is an
  append-only event log; the index is derived and rebuilt from the files at any time.
- **Safe by default.** Provenance comes from the transcript and the cited turn must share the claim's content words (a word-overlap
  check, not entailment); tool
  output never becomes a rule on its own; secrets are refused; memory is fenced as data and re-scanned when shown;
  every session can be taken back with `rewire undo`. What runs the LLM is set per device, never synced.
- **Experimental, off by default:** ordering by usage (ACT-R decay over the event log) for recall (`recall_beta`) and the
  session-start snapshot (`snapshot_use`), never measured, and spreading activation over every note link
  (`spread = "full"`: +0.0 points, interval [−4.2, +4.6], 80 questions, earlier Python engine; a null result on a
  small sample, not proof of no effect). By default recall only follows a link from an outdated note to its
  replacement; that default itself has not been measured end to end.

Architecture: [`docs/design/architecture.md`](docs/design/architecture.md) · File format: [`docs/design/format.md`](docs/design/format.md) · Design: [`docs/design/mvp-spec.md`](docs/design/mvp-spec.md) · Background and hypotheses: [`docs/design/synaptic-memory.md`](docs/design/synaptic-memory.md) · Iteration log: [`docs/iterations.md`](docs/iterations.md) · Evaluation: [`docs/evaluation.md`](docs/evaluation.md)

## Installation

The release binary (Linux x86_64, macOS arm64/x86_64; one file, SQLite bundled), checked against its SHA-256, into
`~/.local/bin`:

```bash
curl -fsSL https://raw.githubusercontent.com/NewbeeTryToCode/rewire/main/scripts/install.sh | sh
rewire init                     # creates ~/brain (or set REWIRE_BRAIN)
```

Or build it: `cargo install --locked --git https://github.com/NewbeeTryToCode/rewire rewire`. Everything at once
(binary, brain, Claude Code and Codex connected): `bash scripts/setup-local.sh`.

Recall over MCP, with the server warm, takes a few ms on a small brain and tens of ms on a ~50-session,
~115k-token history (measured: [`experiments/eval/latency.py`](experiments/eval/latency.py)).

**Consolidation (the only LLM use)** runs in the background after a session, or once a session has been idle for 10
minutes, about two calls each time. A long session is read in parts, and each part is given the summaries (and the
claims) of the parts before it, so a later part can correct an earlier one. It is
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
itself refuses approving, `--apply`, `forget`, `undo`, `remote set` and `install cloud` inside an agent session (detected by the agent's environment
variables or its process among the command's ancestors; test suites and benchmark harnesses started from an agent
session set `REWIRE_NOT_AN_AGENT=1`). These raise the bar, they are not a sandbox: an agent that may run arbitrary
shell commands acts as you, and could do the same.

With sync, rules, preferences and skills that arrive from another device (or come with `rewire init --from`) are left
out of the snapshot, recall and the per-prompt hook on this device until you approve them here (`rewire review`;
`review reject` forgets one everywhere); proposals that arrive in the inbox are labelled and never approved by
`approve all`. Facts, decisions and session transcripts flow between devices without a review, and the sender
chooses a note's type, so this protects against unreviewed rules, not against someone who controls your remote:
keep the remote private. The review is per device (each device approves a rule once, and a new or ephemeral clone
starts with all of them waiting, unless `REWIRE_REMOTE_TRUSTED` says otherwise: see Cloud sessions); a signed approval ledger shared across devices is not built. A pull that contains
symlinks, submodules or device-local files, or a `.gitignore` that un-ignores them, is refused, and a brain.toml change
to what this device keeps (`transcripts`, `profile`) is not taken until you set it in this device's `local.toml`.
Other brain.toml settings from the remote apply as they come (budgets and chunk size have floors; recall weights,
link following and the project map are not checked), so keep the remote private.

### Review from the chat

No terminal at hand (Claude Code on the web, the desktop app, the phone)? Run `/rewire-review`: the agent shows the
inbox, grouped and with its recommendation, and ends with lines to copy. Send them back as a message of their own:

```
rewire approve 01M61NFF 01M61NFG0
rewire reject 01M60TX5
```

The `UserPromptSubmit` hook carries that out, not the agent: it acts only on a message made of nothing but review
commands, only when the session's transcript shows the same text as your latest message with no agent turn after it,
and it replies with each item's text as its file said, the outcome, and one commit in the brain. `rewire install
claude-code --write` installs the skill and a rule that denies the agent `rewire hook`. As with the other checks,
this raises the bar and is not a sandbox: an agent that writes the transcript file as you could forge a message.

Harnesses without hooks can still be used: the agent runs `rewire prime` via instructions in AGENTS.md/the skill, and its transcripts are consolidated *lazily* on the next hook or with `rewire consolidate --pending`.

### Cloud sessions (Claude Code on the web)

A cloud session starts in a fresh VM, so the brain lives in a private git repository and each session fetches it.
Setup is one paste:

1. Create a private repository for the brain (empty, or with GitHub's README and LICENSE) and let the Claude GitHub
   App reach it (or connect with `/web-setup`). The cloud's GitHub proxy authenticates git: no token goes into the VM.
2. In the cloud environment's settings (environment menu > Edit), add that repository to the environment's
   repositories: a session reaches only the private repositories attached to it, and rewire tells you in the snapshot
   when the brain is out of its reach.
3. Paste [`scripts/setup-cloud.sh`](scripts/setup-cloud.sh) as the setup script, with `BRAIN` set to that
   repository and `AUTHOR` to your git identity (a cloud VM commits as the agent otherwise; rewire's commits to the
   brain use `REWIRE_GIT_AUTHOR`, which `--author` sets). It downloads the release binary (no Rust build, no login) and runs `rewire install cloud --remote <url>`:
   the hooks, `REWIRE_REMOTE` and `REWIRE_REMOTE_TRUSTED=1` in Claude Code's settings, and the MCP server. The
   environment caches the result.

Each session then clones the brain into `~/brain` at start (the session-start hook or the MCP server, whichever
starts first), shows the snapshot, and pushes what consolidation learns back. The first session turns a new
repository into a brain (`rewire init --from <url>` does the same on your own machine; a repository with anything
else in it is refused). `REWIRE_REMOTE` turns auto sync on for the brain it names. Each VM syncs as its own device
(`cloud-<session id>`), so parallel sessions never write one event log.

`REWIRE_REMOTE_TRUSTED=1` makes what comes from the repository count as reviewed in the VM: a VM's review state ends
with the session, so without it every rule, preference and skill would wait for a review that never comes. The review
gate therefore does not apply in cloud sessions: review the inbox from the chat (see Review from the chat) or on your
own machine, and keep the repository private.
`rewire install cloud --review` leaves it off. An agent in the VM can change these settings for its own commands,
like everything else it can do as you there.

While a session runs, an idle watcher (started by the hooks, one per brain) consolidates its transcript once it has
been quiet for `REWIRE_IDLE_SECS` (600 s), without waiting for SessionEnd, which a closed tab or a reclaimed VM never
sends. When the session goes on, the rest is consolidated the same way, as a later part. Limit: whatever was said
after the last such pause is lost if the VM is reclaimed sooner; `rewire consolidate` before you leave captures it.

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
