# Research: Memory Approaches in Agent Harnesses

> rewire's goal: **own the brain, transfer it anywhere** — one "second brain" owned by the user that can be attached to any harness (Claude Code, Codex, Gemini CLI, OpenCode, Cursor, Hermes, OpenClaw, etc.).
>
> Main source: [best-of-agent-harnesses](https://github.com/ryanalberts/best-of-agent-harnesses) (the *Memory and state* section, `comparisons/memory-layers.md`, `comparisons/progressive-disclosure.md`, and the `context_memory` ratings in `attributes/*.json`), then verified against each project's documentation. Researched 2026-10-10.

---

## 1. The big picture: 4 shapes of memory

That list divides memory into several "shapes". The key question: **who decides what is remembered?**

| Shape | Examples | Who decides | Portable across harnesses? |
|---|---|---|---|
| **Instruction files** (written by humans) | AGENTS.md, CLAUDE.md, GEMINI.md | Humans | ✅ AGENTS.md is read by 20+ tools |
| **Memory files written by the agent** | Claude Code auto memory, Hermes `MEMORY.md`/`USER.md`, OpenClaw `MEMORY.md` + `memory/YYYY-MM-DD.md`, Qwen Code auto-memory | The agent (via a file-writing tool) | ⚠️ the formats are similar, but locations & rules differ per harness |
| **Harness plugins** (auto-capture) | claude-mem, context-mode, beads | Automatic via hooks | ✅ claude-mem & beads are deliberately multi-harness |
| **Memory layer / engine** (called via API/MCP) | Mem0/OpenMemory, Graphiti (Zep), cognee | Application code | ✅ but needs a server + DB |
| **Memory-based runtime** | Letta (MemGPT) | The agent itself, as part of how it thinks | ❌ the agent has to live inside Letta (there is an `.af` export) |

---

## 2. Approach breakdown per harness

### Claude Code — instruction files + auto memory
- Layered **CLAUDE.md** (org/managed → user `~/.claude/CLAUDE.md` → project → subdirectory), supports `@path` imports (max 4 hops), and `.claude/rules/` that can be scoped per path. Since v2.1.277 it reads **AGENTS.md** directly.
- **Auto memory** in `~/.claude/projects/<project>/memory/`:
  - `MEMORY.md` = **index**, one line per memory; only the **first 200 lines / 25KB** are loaded each session.
  - Details are stored in **topic files** (`feedback_testing.md`, `user_role.md`) that are read **on demand**.
  - Frontmatter has a `type`: `user`, `feedback`, `project`, `reference`; the harness adds a `modified` timestamp.
  - Important rule: **do not store what can be derived from code/git**, and do not duplicate CLAUDE.md.
  - Weakness for our vision: **machine-local, per-repo**, not synced across machines/harnesses. But its location can be moved via `autoMemoryDirectory` → this is a good attach point.

### Hermes (Nous Research) — small memory, frozen, written via a tool
- Two files in `~/.hermes/memories/`: `MEMORY.md` (agent notes, capped at **2,200 characters**) and `USER.md` (user profile, capped at **1,375 characters**). Entries are separated by `§`.
- Loaded once at session start as a **frozen snapshot** → preserves the LLM's **prefix cache**. Mid-session writes go straight to disk but only appear in the next session.
- `memory` tool: `add`, `replace` (finds via an `old_text` substring), `remove`. There is no `read` (it is already in the prompt).
- **Hard cap**: when full, writes fail and the agent is *forced* to consolidate. Nothing is silently discarded.
- **Security scan**: entries are checked for prompt injection, exfiltration, and invisible unicode.
- Optional `write_approval` (review before saving).
- Episodic: all sessions in SQLite FTS5 (`state.db`), searched via `session_search` (raw, without LLM summaries).
- **Provider plugins**: Honcho, Mem0, Hindsight, Supermemory, etc. run *alongside* the built-in files.

### OpenClaw — Markdown as the source of truth + "dreaming"
- All memory = plain Markdown in the workspace. "The model stores nothing except what is written to disk."
  - `MEMORY.md`: long-term facts & decisions, injected at session start (truncated if it exceeds the budget; the file on disk stays intact).
  - `USER.md`: stable preferences, smaller budget.
  - `memory/YYYY-MM-DD.md`: **daily notes** (episodic); today + yesterday are loaded on `/new`, the rest are only indexed.
  - `DREAMS.md`: a consolidation log for human review.
- Retrieval: `memory_search` (hybrid vector + keyword, SQLite index) and `memory_get` (reads a file/line range).
- **Pre-compaction flush**: before the context is compressed, the harness runs a silent turn that tells the agent to save important things.
- **Dreaming**: a background cron that promotes candidates from short-term to `MEMORY.md` with gates on score, recall frequency, and query diversity; candidates from untrusted sources are never promoted.

### claude-mem — auto-capture via hooks, progressive retrieval
- 5 lifecycle hooks (SessionStart, UserPromptSubmit, PostToolUse, Stop, SessionEnd) — all **observers**, fire-and-forget.
- A local worker (Bun) + SQLite (FTS5) + Chroma (vector) → hybrid search.
- **3-step progressive disclosure** via MCP: `search` (compact index ~50–100 tokens/result) → `timeline` (surrounding context) → `get_observations` (details ~500–1000 tokens, only for selected IDs). Claims ~10x token savings.
- Multi-harness: installers for Claude Code, Codex, OpenCode, Gemini, Copilot, OpenClaw, Pi, etc.; harnesses without hooks are handled by **watching the chat log file**.
- Privacy: content inside `<private>…</private>` is not stored.

### beads — portable memory that lives outside the context window
- Store: **Dolt** (versioned SQL) in `.beads/`; git is used as the transport (`refs/dolt/data`), JSONL only for export.
- `bd remember "insight"` stores a fact; `bd prime` prints the workflow + memories to inject into the agent.
- Integration: `bd init` writes a section in **AGENTS.md**; `bd setup claude|codex|cursor` installs hooks/a skill; for other agents an AGENTS.md snippet pointing to `bd prime` is enough. There is `beads-mcp`.
- Lesson: **CLI + an AGENTS.md snippet is the cheapest universal adapter** — every harness that can run a shell can use it.

### Letta (MemGPT) — memory as an OS
- **Core memory blocks** (label, value, limit, description, read_only) are always in context; the agent edits them itself via tools.
- **Archival memory** (unlimited, semantic search on demand) + **recall memory** (conversation history).
- Blocks can be **shared between agents**.
- **Agent File (`.af`)**: an open format for exporting an agent (system prompt, memory blocks, tools, message history, model config). Archival is not yet supported; secrets are exported as `null`. This is the closest precedent for "transferring a brain".

### Mem0 / OpenMemory — a universal memory layer
- Automatic fact extraction, scoped per user/agent/session, hybrid retrieval (semantic + BM25 + entity).
- **OpenMemory** = a local MCP server shared by many clients (Claude, Cursor, etc.) — a "one brain, many clients" model.

### Graphiti (Zep) — temporal knowledge graph
- Facts are stored along with **when they are valid** (valid_at / invalid_at); old facts are not overwritten but *invalidated*. It can answer "what was true at time X".
- Lesson: a long-term brain needs **supersession, not overwriting**.

### Other relevant ones
- **Qwen Code**: per-project auto-memory + a daily "dream" pass to dedupe & discard stale entries.
- **oh-my-pi (Hindsight)**: `retain` / `recall` / `reflect`, and each session is compressed into a project "mental model" that auto-loads on the first turn of the next session.
- **gsd-core / planning-with-files**: work state is stored in files such as `.planning/STATE.md`, `PLAN.md`, etc. — crash-proof short-term working memory.
- **Codex / OpenInterpreter / Pi**: `PreCompact`/`PostCompact`/`SessionStart(resume)` hooks — standard points for flush & restore.
- **Anthropic memory tool (API)**: a `/memories` directory operated by the model through file commands (`view`, `create`, `str_replace`, `insert`, `delete`, `rename`) — its storage is provided by the application, so it can be pointed at our brain.

---

## 3. Converging patterns (that we should adopt)

1. **Plain Markdown on disk = source of truth.** Claude Code, Hermes, OpenClaw, Qwen Code, and the Anthropic memory tool all end up here. Indexes (FTS/vector) are only derivatives that can be rebuilt. This is the key to *own the brain*: human-readable, diffable, git-able.
2. **Two tiers: a small index that is always loaded + details on demand.** The 200-line `MEMORY.md` (Claude Code), the 2,200-char cap (Hermes), the bootstrap budget (OpenClaw), `search → get` (claude-mem). Context is expensive; a good brain is a **map**, not an encyclopedia.
3. **Explicit memory types.** At minimum: `user` (who I am), `feedback` (corrections/work preferences), `project` (decisions & state), `reference` (where information lives), plus `episodic` (daily log) and `procedure` (skills).
4. **A frozen snapshot at session start** to preserve the prefix cache; mid-session writes go to disk and appear in the next session.
5. **Capture ≠ promote.** Write quickly to an inbox/daily note → background consolidation ("dreaming") → promotion to core with a gate. A hard cap forces curation.
6. **Time & supersession.** Store `modified`, `valid_from`/`valid_to`, `supersedes`. Do not overwrite facts that change.
7. **Lifecycle hooks as integration points:** `SessionStart` (inject), `PreCompact` (flush), `Stop`/`SessionEnd` (summarize). For harnesses without hooks: an AGENTS.md snippet + CLI (`bd prime`), or watching logs.
8. **Memory security:** scan for injection/exfiltration before promotion (Hermes, OpenClaw), a `<private>` tag (claude-mem), optional approval, and untrusted sources must not be promoted automatically. Persistent memory = a persistent prompt-injection attack surface.
9. **Do not store what can be derived** from code/git/other documents.

### Gaps nobody has solved yet (rewire's opportunity)
- All of the memory above is **tied to one harness** (location, format, loading rules) or **needs a server/DB** (Mem0, Graphiti, Letta). There is no **neutral brain format** yet that is file-first, git-syncable, schema-backed, and has a thin adapter for each harness.
- Claude Code's and Hermes's auto memory are **machine-local**; there is no sync across machines/harnesses.
- There is no standard like AGENTS.md **for personal memory** (as opposed to repo instructions).

---

## 4. Proposed rewire architecture

```
~/brain/                      # a git repo owned by the user — THIS is the brain
├── brain.yaml                # manifest: schema version, identity, budget, policy
├── core/
│   ├── USER.md               # who I am (always loaded, tight cap)
│   └── MEMORY.md             # index: 1 line per memory → link to topic file
├── memories/                 # topic files, 1 memory per file, schema-backed frontmatter
│   ├── feedback/…            # corrections & work preferences
│   ├── project/<slug>/…      # decisions & state per project
│   └── reference/…
├── journal/YYYY-MM-DD.md     # episodic, append-only (fast capture)
├── inbox/                    # candidates from harnesses, not yet promoted
├── skills/<name>/SKILL.md    # procedural memory (Agent Skills format)
└── .index/                   # SQLite FTS5 (+ optional embeddings) — gitignored, rebuildable
```

**Memory frontmatter (draft):**
```yaml
---
id: 01JABC...            # ULID, stable across renames
type: feedback           # user | feedback | project | reference | procedure
scope: global            # global | project:<slug>
created: 2026-10-10T14:00:00Z
modified: 2026-10-10T14:00:00Z
valid_to: null           # set when the fact is no longer valid
supersedes: []           # ids of memories this one replaces
source: claude-code      # originating harness/session
confidence: confirmed    # confirmed | inferred
---
```

**Three adapter layers (from cheapest to richest):**

| Layer | Mechanism | Harnesses covered |
|---|---|---|
| 1. **Render** | `rewire render <harness>` writes core into the native format: an `@~/brain/core/MEMORY.md` import in `~/.claude/CLAUDE.md`, a section in AGENTS.md/GEMINI.md, `~/.hermes/memories/*.md`, `~/.openclaw/workspace/MEMORY.md`; set Claude Code's `autoMemoryDirectory` to the brain | Everything that reads instruction files |
| 2. **CLI + MCP** | `rewire prime` (prints context, `bd prime` style), `rewire remember`, `rewire recall`; an MCP server with progressive `search → get` + `remember` | Every harness that has a shell or MCP |
| 3. **Hooks** | SessionStart → inject snapshot; PreCompact → flush to `inbox/`; SessionEnd → summary to `journal/` | Claude Code, Codex, Gemini CLI, OpenCode, Pi, etc. |

Plus a two-way **importer**: pull `~/.claude/projects/*/memory`, `~/.hermes/memories`, and the OpenClaw workspace into `inbox/` → consolidation → brain. And **export** to formats such as Letta `.af` when needed.

**Write pipeline:** harnesses write to `inbox/`/`journal/` (fast, no gate) → `rewire consolidate` (manual or a "dream" cron: dedupe, merge, supersede, injection scan, budget check) → promotion to `memories/` + update `MEMORY.md` → `git commit` (history = audit trail, sync = `git push/pull`).

---

## 5. Decisions to make next

1. **Implementation language for the CLI/MCP** (e.g. TypeScript/Bun following claude-mem, or Go/Rust for a single binary).
2. **Embeddings or not in v1?** Recommendation: start with FTS5 only (zero-dependency; Hermes proves it is enough), with embeddings as a plugin.
3. **Target harnesses for v1.** Recommendation: Claude Code + Codex + one harness without hooks (via AGENTS.md + CLI) to prove portability.
4. **Whose LLM does consolidation use?** Use the model of the currently active harness, or the user's own API key.
5. **Privacy policy**: what must never leave the machine (`<private>` tag, encrypted files for secrets, public vs private brain).

---

## References
- best-of-agent-harnesses — https://github.com/ryanalberts/best-of-agent-harnesses
- Claude Code memory — https://code.claude.com/docs/en/memory
- Hermes memory — https://github.com/NousResearch/hermes-agent (docs `user-guide/features/memory`)
- OpenClaw memory — https://github.com/openclaw/openclaw (docs `concepts/memory`)
- claude-mem — https://github.com/thedotmack/claude-mem
- beads — https://github.com/gastownhall/beads
- Letta memory blocks & Agent File — https://github.com/letta-ai/letta, https://github.com/letta-ai/agent-file
- Mem0 / OpenMemory — https://github.com/mem0ai/mem0
- Graphiti — https://github.com/getzep/graphiti
- AGENTS.md — https://agents.md
