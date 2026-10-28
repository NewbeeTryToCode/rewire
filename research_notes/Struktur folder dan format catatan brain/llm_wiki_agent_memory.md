# LLM Wiki and File-Based LLM/Agent Memory Designs (as candidates for a personal markdown git "brain")

Research date: 2026-10-28. Network limits during research: arxiv.org, docs.cline.bot, docs.basicmemory.com, docs.letta.com, gist.githubusercontent.com and mpt.solutions could not be reached from this session. Those sources were read through GitHub pages or search snippets instead, or recorded as gaps. The Karpathy gist was read through gist.github.com with a summarizing fetch tool, so the quotes below are short excerpts, not the full text.

## 1. Karpathy's "LLM Wiki": architecture, layout, operations, index/log, lint, filing queries back

### Takeaway
Karpathy's "LLM Wiki" is a short, deliberately abstract "idea file" gist created April 4, 2026. It has three layers: immutable raw sources, an LLM-owned markdown wiki, and a schema file (CLAUDE.md or AGENTS.md). There are three operations: ingest, query and lint. Two special files hold it together: index.md, a content catalog the LLM reads first, and log.md, an append-only chronological log with grep-able prefixes. The gist says this works without embeddings at about 100 sources and a few hundred pages. Good query answers are filed back into the wiki as new pages, so knowledge "compounds".

### Cited Findings
- **Identity and date.** The gist is titled "LLM Wiki", created April 4, 2026, and describes itself as "A pattern for building personal knowledge bases using LLMs." It is an "idea file" meant to be pasted into an agent (Claude Code, Codex), which then builds a concrete version together with the user. — [Karpathy gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
- **Section headings, in order:** LLM Wiki / The core idea / Architecture / Operations / Indexing and logging / Optional: CLI tools / Tips and tricks / Why this works / Note. — [Karpathy gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
- **Core idea versus RAG.** RAG re-derives answers from raw chunks on every query, so nothing accumulates. In the wiki pattern, the LLM summarizes each source when it is added, updates entity and topic pages, flags contradictions and keeps cross-references current. In Karpathy's words, "the wiki is a persistent, compounding artifact." — [Karpathy gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
- **Three layers:**
  - *Raw sources:* curated, immutable documents (articles, papers, images, data). The LLM reads them and never modifies them.
  - *The wiki:* LLM-generated, LLM-owned markdown, including summaries, entity pages, concept pages, comparisons, an overview and a synthesis. "The user reads it; the LLM writes it."
  - *The schema:* a config document such as CLAUDE.md or AGENTS.md that defines structure, conventions and workflows. The user and the LLM refine it over time.
  — [Karpathy gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
- **Ingest.** The user adds a source. The LLM discusses the takeaways, writes a summary page, updates the index, revises related pages and appends to the log. One source "might touch 10–15" pages. The gist prefers ingesting one source at a time under close supervision, though batches are possible. — [Karpathy gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
- **Query.** The LLM searches the relevant pages and writes a cited answer, which can be markdown, a table, slides or a chart. "good answers can be filed back into the wiki as new pages." — [Karpathy gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
- **Lint.** A periodic health check. "Look for: contradictions between pages, stale claims that newer sources have superseded", plus orphan pages, missing concept pages, missing cross-references, and data gaps that web search could fill. — [Karpathy gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
- **index.md.** Content-oriented: a categorized catalog of every page, each with a link, a one-line summary and optional metadata. The LLM updates it on every ingest and reads it first when answering queries. "This works surprisingly well at moderate scale (~100 sources, ~hundreds of pages)". At that scale no embedding-based RAG is needed. — [Karpathy gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
- **log.md.** Chronological and append-only; it records ingests, queries and lint passes. Each entry starts with a consistent prefix, for example `## [2026-04-02] ingest | Article Title`, so simple Unix tools can parse it. — [Karpathy gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
- **Optional tooling.** When the index stops being enough, a local markdown search engine (the gist names qmd, which has CLI and MCP interfaces) or a small custom search script. Other tips: a web clipper that saves articles as markdown, downloading images locally so the LLM can view them, Obsidian graph view, Marp for slides, Dataview for frontmatter queries, and git for version history. — [Karpathy gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
- **Rationale.** "Humans abandon wikis because the maintenance burden grows faster than the value." LLMs do the bookkeeping at near-zero cost, while humans curate sources, ask questions and interpret results. The gist links the idea to Vannevar Bush's Memex. — [Karpathy gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
- **Deliberate abstraction.** The gist's final Note says the directory structure, schema, formats and tooling should be adapted to each domain. It does not prescribe a fixed folder tree beyond the raw / wiki / schema split and index.md and log.md. — [Karpathy gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
- **Secondary descriptions (not from the gist itself).** Write-ups describe the layout as `raw/` (immutable) + `wiki/` (LLM-maintained) + CLAUDE.md/AGENTS.md. One describes index.md as "one row per article, grouped by topic, with link + summary + Updated date". — [Starmorph guide](https://blog.starmorph.com/blog/karpathy-llm-wiki-knowledge-base-guide); [Codersera](https://codersera.com/blog/karpathy-llm-knowledge-base-second-brain/amp/)
- **Earlier version.** Daniel Vaughan's blog (2026-04-11) refers to Karpathy's earlier "LLM Knowledge Bases" post and describes running "the same flywheel" for six weeks. — [Daniel Vaughan blog](https://codex.danielvaughan.com/2026/04/11/karpathy-llm-knowledge-bases-codex-resources-flywheel/)

### Inferences
- The gist names page types (source summary, entity, concept, comparison, overview, synthesis) but gives no templates or frontmatter. Any "Karpathy template" with fixed fields is a derivative, not the original.
- For a brain fed by session transcripts, the mapping is: transcripts = `raw/` (immutable, never edited); project and concept pages = `wiki/`; CLAUDE.md/AGENTS.md = schema; index.md = the catalog the agent reads first; log.md = an audit trail of every ingest, query and lint pass.
- The roughly 100 sources / few hundred pages ceiling matters. A brain that ingests every session transcript will pass 100 sources quickly, so it needs either summarized or rolled-up raw material, or a search layer (qmd, ripgrep, embeddings) beyond index.md.

### Gaps
- The full verbatim gist text could not be extracted. The raw endpoint was blocked and the fetch tool would only summarize. The exact wording of the page-type list and the qmd details was not captured.
- The comment thread (more than 140K characters) was not read beyond 100K characters. The summarizer said it was mostly unrelated promotion.
- The original X/Twitter post(s) and their exact dates were not retrieved.

## 2. Implementations and derivatives (repos, Obsidian vaults, "second brain with Claude Code")

### Takeaway
A wave of Claude Code and Obsidian derivatives appeared from April to May 2026. Most keep Karpathy's ingest/query/lint verbs and add the operational pieces the gist leaves out: an inbox, filing modes (PARA, LYT, Zettelkasten), lint rules such as dead links and missing metadata, write journals, and log roll-ups. Some, such as obsidian-second-brain, explicitly rewrite pages in place instead of appending. Star counts and dates come from aggregators and should be treated as approximate.

### Cited Findings
- **AgriciDaniel/claude-obsidian.** "Self-organizing AI second brain for Obsidian + Claude Code". One listing reports about 7.3k stars, 845 forks and a last push on 2026-05-28. — [gittrend listing](https://gittrend.io/repo/AgriciDaniel/claude-obsidian)
  - Sources go into `inbox/`, and originals stay untouched.
  - Skills: `wiki-ingest` turns inbox sources into linked pages with recorded sources; `wiki-query` is read-only; `save` keeps an answer as a note "only when asked"; `wiki-lint` checks for dead links, orphans, missing metadata and empty sections; `wiki-fold` "Rolls up the operation log into short, traceable summaries"; others include `wiki-mode`, `wiki-retrieve`, `autoresearch` and `canvas`.
  - A write journal "keeps backups during the write", and `transaction recover` rolls back interrupted changes.
  - Filing modes: Generic ("Sources, concepts, entities and sessions"), PARA ("Projects, Areas, Resources and Archives"), LYT ("Maps of Content and linked atomic notes") and Zettelkasten (stable IDs, atomic notes). Switching modes "changes where new notes go and never moves your old ones."
  - The vault is located through `CLAUDE_OBSIDIAN_VAULT` or the nearest `.claude-obsidian.json`.
  — [GitHub README](https://github.com/AgriciDaniel/claude-obsidian)
- **eugeniughelbur/obsidian-second-brain.** About 3.9k stars, works across several CLIs, and is described as differing from Karpathy's pattern because it "rewrites existing pages rather than only appending new ones, so stale facts get replaced." (aggregator description) — [gittrend listing](https://gittrend.io/repo/eugeniughelbur/obsidian-second-brain)
- **Other derivatives:**
  - ignromanov/llm-obsidian-wiki: a Claude Code plugin that captures URLs, PDFs, GitHub, YouTube and compiles them into an Obsidian vault with citations. — [skills.cat](https://skills.cat/skills/ignromanov/llm-obsidian-wiki)
  - shannhk/llm-wikid: "Clone, run Claude Code, start building your second brain". — [gittrend](https://gittrend.io/repo/shannhk/llm-wikid)
  - leahyra LLM Wiki plugin: an init command for Claude Code, Codex and Cursor. — [claudepluginhub](https://www.claudepluginhub.com/plugins/leahyra-llm-wiki-engineering-llm-wiki)
  - astro-han/karpathy-llm-wiki skill. — [skills.sh](https://www.skills.sh/astro-han/karpathy-llm-wiki/karpathy-llm-wiki)
  - alirezarezvani/claude-skills llm-wiki skill. — [skills.sh](https://www.skills.sh/alirezarezvani/claude-skills/llm-wiki)
  - coleam00/second-brain-starter, named in a video guide. — [videohighlight summary](https://videohighlight.com/v/HuREI6lks4s)
- **Tutorials** describing Obsidian + Claude Code builds of the pattern. — [MindStudio](https://www.mindstudio.ai/blog/ai-second-brain-obsidian-claude-code-llm-wiki); [The AI Maker](https://aimaker.substack.com/p/llm-wiki-obsidian-knowledge-base-andrej-karphaty)

### Inferences
- Derivatives converge on four additions to the gist:
  1. an explicit inbox or raw staging area;
  2. a filing taxonomy (PARA is the one that directly models "many projects plus shared areas and resources");
  3. lint rules that can be checked mechanically (dead links, orphans, missing frontmatter, empty sections);
  4. log compaction (wiki-fold) so log.md does not grow without bound.
- "Save only when asked" (claude-obsidian) and "file good answers back" (Karpathy) are two opposite policies. For automatic writes from transcripts, this choice decides how much review the human has to do.

### Gaps
- Exact page templates and frontmatter fields for claude-obsidian (its `templates/vault`) and the other repos were not retrieved.
- Star counts come from third-party aggregators and were not verified on GitHub.

## 3. Other file-based agent memory designs (atomic vs compiled)

### Takeaway
The designs fall on a spectrum:
- **Compiled, fixed-schema project docs:** Cline Memory Bank (six files, all read every task); CLAUDE.md/AGENTS.md (human-written instructions).
- **Index + topic files:** Claude Code auto memory (MEMORY.md index of one line per memory, plus topic files); Anthropic memory tool (a free-form /memories directory).
- **Atomic, graph-structured:** Basic Memory (one entity per file, with categorized observations and typed relations); A-MEM (Zettelkasten notes); Mem0 (extracted facts, ADD-only since April 2026); Zep/Graphiti (temporal knowledge-graph edges with validity windows).
Karpathy's wiki sits at the compiled end, but adds an index and a log.

### Cited Findings
- **Cline Memory Bank (compiled, fixed schema).**
  - Six core files: `projectbrief.md` (foundation: requirements, goals, scope); `productContext.md` (why it exists, problems, UX); `activeContext.md` (current focus, recent changes, next steps, active decisions); `systemPatterns.md` (architecture, key decisions, patterns, component relationships); `techContext.md` (technologies, setup, constraints, dependencies); `progress.md` (what works, what's left, status, known issues, "evolution of project decisions").
  - Hierarchy: projectbrief feeds productContext, systemPatterns and techContext; those feed activeContext, which feeds progress. Extra files or folders under `memory-bank/` hold complex features, integrations, API docs, testing and deployment.
  - Rule: "I MUST read ALL memory bank files at the start of EVERY task - this is not optional."
  - Updates happen when new patterns are discovered, after significant changes, when context needs clarifying, or when the user says "update memory bank". That last trigger requires reviewing every file, especially activeContext and progress.
  — [cline/prompts memory-bank.md](https://github.com/cline/prompts/blob/main/.clinerules/memory-bank.md)
- **Claude Code CLAUDE.md hierarchy (human-written instructions):**
  - Locations: managed policy (`/etc/claude-code/CLAUDE.md` on Linux), user (`~/.claude/CLAUDE.md`), project (`./CLAUDE.md` or `./.claude/CLAUDE.md`), and local (`./CLAUDE.local.md`, gitignored).
  - Files in ancestor directories are concatenated, not overridden, ordered root-to-cwd so closer files are read last. Subdirectory CLAUDE.md files load on demand when Claude touches files there.
  - `@path` imports nest up to four hops deep, and imports do not reduce context cost.
  - Recommended size: "target under 200 lines per CLAUDE.md file."
  - Contradictions: "if two instructions contradict each other, Claude may pick one arbitrarily", so review periodically or run `/doctor prompt-audit`.
  - `.claude/rules/*.md` holds one topic per file, optionally path-scoped with `paths:` frontmatter, and supports symlinks for sharing rules across projects. `~/.claude/rules/` holds personal rules that apply to every project.
  - AGENTS.md is read when no CLAUDE.md exists (v2.1.277+), or both are read with the `claude-md-and-agents-md` setting.
  — [Claude Code memory docs](https://code.claude.com/docs/en/memory)
- **Claude Code auto memory (agent-written, index + topic files):**
  - Stored at `~/.claude/projects/<project>/memory/`, where `<project>` is derived from the git repo, so all worktrees share one directory. It is machine-local and not synced.
  - `MEMORY.md` is an "Index, one line per memory, loaded into every session". Only the first 200 lines or 25KB load. Near the limit, Claude Code reminds Claude to "keep one line per entry, move detail into topic files, and merge or drop stale entries". Over the limit, the write succeeds but returns an error asking Claude to rewrite the index.
  - Topic files (e.g. `user_role.md`, `feedback_testing.md`) load on demand.
  - Four memory types are recorded in a frontmatter `type` field: `user`, `feedback`, `project`, `reference`.
  - Claude "skips anything it can derive from the codebase, such as architecture, file paths, or debugging fixes", and anything CLAUDE.md already says.
  - The location can be changed with `autoMemoryDirectory`, and subagents can have their own memory directories.
  — [Claude Code memory docs](https://code.claude.com/docs/en/memory)
- **Anthropic memory tool (API, client-side):**
  - Tool type `memory_20250818`, available on all Claude 4+ models, with no beta header needed. The app maps the `/memories` prefix onto its own storage.
  - Commands: `view` (directories 2 levels deep with sizes; files with line numbers; text truncated beyond 16,000 characters), `create`, `str_replace`, `insert`, `delete`, `rename`.
  - The API auto-injects this system prompt: "IMPORTANT: ALWAYS VIEW YOUR MEMORY DIRECTORY BEFORE DOING ANYTHING ELSE … ASSUME INTERRUPTION: Your context window might be reset at any moment".
  - Suggested reinforcement prompt: "keep its content up-to-date, coherent and organized … Do not create new files unless necessary."
  - Security guidance: path-traversal protection, capping file size, and "Periodically delete memory files that haven't been accessed in a long time."
  - Pairs with context editing and compaction.
  - Recommended multi-session pattern: an initializer session creates a progress log, a feature checklist and an init-script reference; each later session reads them and updates the progress log at the end.
  — [Anthropic memory tool docs](https://platform.claude.com/docs/en/agents-and-tools/tool-use/memory-tool)
- **Basic Memory (atomic entity per file, graph semantics):**
  - Each markdown file is an Entity with YAML frontmatter (`title`, `type`, `permalink`, `tags`).
  - Observations are list items of the form `- [category] content #tag (optional context)`. Relations use `- relation_type [[Target]]`; a bare `[[Target]]` or a link in prose is indexed as `links_to`.
  - Indexed in local SQLite (Postgres also supported), with multi-project support (`basic-memory project add/list`).
  - Current MCP tools are named `bm_write`, `bm_read`, `bm_edit`, `bm_move`, `bm_delete`, `bm_search`, `bm_list`, `bm_find`, `bm_context`, `bm_project`, `bm_schema` (validate/infer/diff) and `bm_man`.
  - Verbatim example from the README:
    ```
    ---
    title: Coffee Brewing Methods
    permalink: coffee-brewing-methods
    tags: [coffee, brewing]
    ---
    # Coffee Brewing Methods
    ## Observations
    - [method] Pour over highlights subtle flavors over body
    - [technique] Water at 205°F (96°C) extracts optimal compounds
    ## Relations
    - relates_to [[Coffee Bean Origins]]
    - requires [[Proper Grinding Technique]]
    ```
  — [basic-memory README](https://github.com/basicmachines-co/basic-memory)
- **Letta / Letta Code (formerly MemGPT):**
  - Letta describes itself as a "Platform for stateful agents" and was formerly MemGPT. Current development is in `letta-ai/letta-code`; the V1 server is archived. — [letta-ai/letta](https://github.com/letta-ai/letta)
  - In Letta Code, agents rewrite their own context, including the system prompt, through memory blocks. Under MemFS, "All context (including memory blocks) is tracked via git" and can be synced to a GitHub repo with `/memory-repository set git@github.com:...`.
  - Sleep-time compute is now called "dreaming" (`/sleeptime`, `/dream`, `/reflect`). Skills load from `~/.letta` (global), `.agents/skills` (project) and MemFS (agent-scoped). `/palace` views memory.
  — [letta-code README](https://github.com/letta-ai/letta-code)
- **Mem0 (atomic extracted memories):**
  - Paper: arXiv 2504.19413 (2025).
  - A new algorithm from April 2026 uses "Single-pass ADD-only extraction". One LLM call per add; memories accumulate rather than being overwritten, and entity linking boosts retrieval.
  - Reported managed-platform scores, old → new: LoCoMo 71.4 → 92.5 (about 7.0K tokens, p50 0.88s); LongMemEval 67.8 → 94.4. These include proprietary optimizations; open source is "directionally similar".
  - Memory is scoped by `user_id`, with User, Session and Agent levels.
  — [mem0 README](https://github.com/mem0ai/mem0)
- **A-MEM (arXiv 2502.12110).** A Zettelkasten-inspired agentic memory: for each new memory, an LLM generates a full note (keywords, tags, context), links it to related notes and updates the linked existing notes ("memory evolution"). Reported LoCoMo F1 of 45.85 versus MemGPT's 25.52 with GPT-4o-mini (secondary summary). — [themoonlight review](https://themoonlight.io/review/a-mem-agentic-memory-for-llm-agents); [arXiv html](https://arxiv.org/html/2502.12110v7)
- **Zep / Graphiti (temporal knowledge graph):**
  - Graphiti is the open-source framework inside the commercial Zep. It builds "temporal context graphs" of entities, facts and episodes.
  - "explicit bi-temporal tracking with automatic fact invalidation": each fact carries a validity window. A contradicted fact is marked invalid, not deleted, so past states stay queryable.
  - Episodes are the ground-truth raw stream, and every derived entity or edge traces back to its source episodes.
  - Hybrid retrieval combines embeddings, BM25 and graph traversal, with no LLM summarization at query time.
  - Paper: arXiv 2501.13956, "Zep: A Temporal Knowledge Graph Architecture for Agent Memory".
  — [graphiti README](https://github.com/getzep/graphiti)
- **Cursor rules.** Cursor rules live in `.cursor/rules/` or the legacy `.cursorrules`. Claude Code `/init` can import them, along with `.windsurf/rules/`, `.devin/rules/` and `.clinerules`. — [Claude Code memory docs](https://code.claude.com/docs/en/memory)

### Inferences
- **Atomic:** Basic Memory (one entity per file, with multiple atomic observations inside), A-MEM, Mem0, Graphiti, and Claude Code auto memory (one memory per topic file, indexed by MEMORY.md).
- **Compiled:** Karpathy's LLM Wiki, Cline Memory Bank, CLAUDE.md/AGENTS.md, and Letta core memory blocks (an always-in-context compiled summary).
- **Basic Memory is a useful hybrid.** A page is a compiled entity page, but its body is made of atomic, categorized observation lines plus typed relation lines. That makes one page information-dense while staying machine-parseable.
- **Graphiti's validity windows and episode provenance** are the clearest prior art for adding `valid_from`/`valid_to`/`supersedes`/`source:` metadata to markdown claims.
- **Letta Code's MemFS** (agent memory tracked in git and syncable to GitHub) is the closest existing system to a "markdown git brain written by agents".

### Gaps
- Letta's memory-block field schema (label, value, char limit, description, read_only), shared blocks, and archival/recall memory semantics could not be verified (docs.letta.com was unreachable). Background knowledge, not verified in this session: MemGPT (Packer et al., arXiv 2310.08560, Oct 2023) splits memory into in-context "core" memory and out-of-context "archival" (vector-searched) and "recall" (conversation history) storage, paged in and out through function calls.
- Mem0's earlier ADD/UPDATE/DELETE/NOOP update logic (described in the 2025 paper) could not be confirmed (arxiv.org was unreachable). The README states only the new ADD-only approach.
- Zep paper benchmark numbers (DMR, LongMemEval) were not retrieved.
- Basic Memory's `memory://` URL scheme and the older `write_note`/`build_context` tool names were not confirmed on the current README, which lists `bm_*` names.
- The AGENTS.md convention's own spec site (agents.md) was not fetched. Only Claude Code's handling of AGENTS.md is cited.

## 4. Trade-offs: atomic memories vs compiled/synthesized pages

### Takeaway
Compiled pages give better human readability, fewer files to load, and the "read index then a few pages" retrieval pattern that works without embeddings at small scale. Their costs are rewrite risk (hallucinated synthesis, lost detail), a heavy review burden on diffs, and weak per-claim provenance. Atomic memories give cleaner provenance, easy supersession and dedup, and append-friendly automation. Their costs are fragmentation, the need for search or embeddings to retrieve, and poor human readability. Production systems increasingly mix both: an atomic or log layer underneath, a compiled index or summary on top.

### Cited Findings
- **Compiled-wiki argument:** knowledge is "compiled once and kept up to date, not re-derived per query", and index-first navigation works at about 100 sources without embeddings. — [Karpathy gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
- **Always-loaded token budgets are hard limits:**
  - Claude Code loads only the first 200 lines or 25KB of MEMORY.md and pushes detail into on-demand topic files.
  - CLAUDE.md should be under 200 lines because "Longer files consume more context and reduce adherence."
  — [Claude Code memory docs](https://code.claude.com/docs/en/memory)
- **Fixed compiled schemas cost tokens on every task.** Cline requires reading all six-plus memory-bank files at the start of every task. — [cline memory-bank.md](https://github.com/cline/prompts/blob/main/.clinerules/memory-bank.md)
- **Atomic, append-only extraction is the current Mem0 direction.** "Single-pass ADD-only extraction"; memories accumulate rather than being overwritten, with about 7K tokens per query reported on LoCoMo and LongMemEval. — [mem0 README](https://github.com/mem0ai/mem0)
- **Atomic facts with validity windows avoid destructive rewrites.** Invalidated facts stay queryable, and every fact traces to source episodes. — [graphiti README](https://github.com/getzep/graphiti)
- **Rewrite versus append is a design axis derivatives argue about.** obsidian-second-brain rewrites pages so stale facts get replaced (aggregator description). — [gittrend](https://gittrend.io/repo/eugeniughelbur/obsidian-second-brain)
- **Atomic-note linking needs an LLM step.** In A-MEM, adding a note triggers an LLM to generate note attributes and update linked notes. — [themoonlight review](https://themoonlight.io/review/a-mem-agentic-memory-for-llm-agents)

### Inferences
- A two-layer design follows from the evidence:
  1. Keep an atomic, append-only, provenance-rich layer, such as dated observation lines with a source link to a transcript, or log.md entries.
  2. Compile from it a per-entity or per-project page that the agent rewrites, plus an index.md one line per page under a hard size cap (as with MEMORY.md).
  Rewrites can then always be checked against, or regenerated from, the atomic layer.
- Basic Memory's `- [category] fact #tag (context)` lines plus `relation [[Target]]` lines are a concrete way to make one compiled page information-dense: many typed facts, each greppable, inside one human-readable page.
- **Token-budget rule of thumb from the Anthropic sources:** always-loaded material should stay under about 200 lines; everything else loads on demand through an index.

### Gaps
- No controlled benchmark was found that directly compares retrieval accuracy of compiled wiki pages against atomic notes for the same corpus. The benchmark numbers above (Mem0, A-MEM) compare memory systems against each other on conversational QA, not file formats.

## 5. Per-project vs global/shared knowledge, and promotion

### Takeaway
Systems separate scopes mainly by directory or file location: Claude Code uses managed, user, project, local and per-repo auto memory; Letta Code uses global `~/.letta`, project `.agents/skills` and agent-scoped MemFS; Basic Memory uses projects; Mem0 uses user_id plus session and agent levels; PARA-mode vaults use Projects, Areas, Resources and Archives. Sharing works through symlinks or imports. None of the primary sources describes an automatic "promote to global when repeated across projects" mechanism. Promotion is manual, or left to the agent's judgment.

### Cited Findings
- **Claude Code scopes:**
  - User `~/.claude/CLAUDE.md` and `~/.claude/rules/` apply to all projects; project CLAUDE.md is shared through git; CLAUDE.local.md is personal per project.
  - Auto memory is per git repo and shared across its worktrees.
  - Shared rules can be symlinked into `.claude/rules/` (for example `ln -s ~/shared-claude-rules .claude/rules/shared`). External symlink targets need one-time approval, and only rules without a `paths` field load from them.
  — [Claude Code memory docs](https://code.claude.com/docs/en/memory)
- **Claude Code "skip what's derivable" rule.** Auto memory excludes what can be derived from code or git history and what CLAUDE.md already says. This is a de-duplication rule between scopes. — [Claude Code memory docs](https://code.claude.com/docs/en/memory)
- **Letta Code scopes:** skills load from `~/.letta` (global), `.agents/skills` (project) and MemFS (agent-scoped). — [letta-code README](https://github.com/letta-ai/letta-code)
- **Basic Memory projects:** `basic-memory project list/add`, and project-scoped MCP tools take a project name, a `workspace/project` path or a UUID. — [basic-memory README](https://github.com/basicmachines-co/basic-memory)
- **Mem0 scoping:** memories scoped by `user_id`, with User, Session and Agent memory levels. — [mem0 README](https://github.com/mem0ai/mem0)
- **claude-obsidian PARA mode** organizes notes as "Projects, Areas, Resources and Archives". — [claude-obsidian README](https://github.com/AgriciDaniel/claude-obsidian)
- **Anthropic memory tool scoping is the app's job.** The `/memories` prefix maps to whatever storage the app chooses, "such as a per-user directory". — [Anthropic memory tool docs](https://platform.claude.com/docs/en/agents-and-tools/tool-use/memory-tool)

### Inferences
- **For a multi-project brain, the closest prior art is the Claude Code layering pattern:**
  - global files (`~/.claude`-like: user profile, cross-project rules and patterns) are always loaded or indexed;
  - per-project pages (`projects/<name>/…`) load on demand;
  - a thin pointer or import joins them.
  - PARA's Projects / Areas / Resources / Archives maps naturally: active projects; ongoing areas such as tools and practices; reference resources; finished work archived.
- **A promotion rule has to be designed.** One option: when lint finds the same observation or decision on two or more project pages, propose moving it to a global concept page and replacing the copies with links. No primary source documents this as a built-in feature.

### Gaps
- No primary source was found describing automated cross-project promotion of memories in any of these systems.

## 6. Known failure modes and mitigations

### Takeaway
The documented failure modes are:
- contradictions and staleness between pages or instructions;
- orphan and missing pages, and broken links;
- index and memory bloat past the load limits;
- cluttered or duplicated memory files;
- interrupted writes.
Documented mitigations are lint passes (Karpathy, claude-obsidian), hard size limits with rewrite prompts (Claude Code MEMORY.md), prompt audits (`/doctor prompt-audit`), invalidate-don't-delete validity windows plus episode provenance (Graphiti), expiry of unaccessed files (Anthropic memory tool), write journals with rollback (claude-obsidian), log roll-ups (wiki-fold), and git history (Karpathy tip, Letta MemFS). No first-hand primary source on "hallucinated synthesis" in LLM wikis was retrieved.

### Cited Findings
- **Lint targets:** "contradictions between pages, stale claims that newer sources have superseded", orphan pages, missing concept pages, missing cross-references, data gaps. Git is recommended for version history. — [Karpathy gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
- **Contradictions are a known instruction-file failure.** "if two instructions contradict each other, Claude may pick one arbitrarily". Mitigation: periodic review and `/doctor prompt-audit`. — [Claude Code memory docs](https://code.claude.com/docs/en/memory)
- **Index bloat** is handled by hard limits (200 lines or 25KB) and automatic reminders to "keep one line per entry, move detail into topic files, and merge or drop stale entries". Content past the limit "is dropped on the next load". — [Claude Code memory docs](https://code.claude.com/docs/en/memory)
- **Memory clutter and duplication:** the suggested prompt is "keep its content up-to-date, coherent and organized. You can rename or delete files that are no longer relevant. Do not create new files unless necessary." Also "Periodically delete memory files that haven't been accessed in a long time", cap file sizes, and strip sensitive data. — [Anthropic memory tool docs](https://platform.claude.com/docs/en/agents-and-tools/tool-use/memory-tool)
- **Progress logs drift when work is marked done too early.** "Mark a feature complete only after end-to-end verification confirms it works, not when the code is written. This keeps the progress log accurate." — [Anthropic memory tool docs](https://platform.claude.com/docs/en/agents-and-tools/tool-use/memory-tool)
- **Staleness and contradiction handling in a graph:** a contradicted fact is marked invalid with a validity window, not deleted, and every derived fact links to its source episodes. — [graphiti README](https://github.com/getzep/graphiti)
- **Write safety and log growth:** a journal "keeps backups during the write" plus `transaction recover`; `wiki-fold` rolls the operation log into "short, traceable summaries"; `wiki-lint` checks dead links, orphans, missing metadata and empty sections. — [claude-obsidian README](https://github.com/AgriciDaniel/claude-obsidian)
- **Review burden.** Karpathy prefers one-source-at-a-time ingest "with close supervision", which implies human review cost grows with automation. A single source can touch 10–15 pages. — [Karpathy gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
- **Production failure modes.** An August 2026 mpt.solutions post claims that six months of running the LLM-wiki pattern in production surfaced "three failure modes he never mentions". The content could not be retrieved; only the search snippet was seen. — [mpt.solutions tag page](https://www.mpt.solutions/tag/knowledge-management/)

### Inferences
- **Mitigations for an automatic transcript-fed brain:**
  1. Raw transcripts stay immutable, and every compiled claim carries a source link (transcript path and date). This is provenance as in Graphiti episodes.
  2. Superseded claims are struck or moved under a dated "Superseded" heading with `valid_to`, not silently deleted.
  3. Every write is a git commit with a log.md entry in the `## [date] op | title` format, so each change can be audited and reverted.
  4. A periodic lint agent checks contradictions, orphans, duplicates across projects (promotion candidates), stale `updated:` dates and index size limits.
  5. A hard cap on index.md, like MEMORY.md's 200 lines.
- **"Hallucinated synthesis" and "page bloat"** are the risks that matter most when no human reviews each ingest, the opposite of Karpathy's "close supervision" default. Diff-based review of git commits is the natural way to control them.

### Gaps
- No primary or HN source was retrieved that empirically documents hallucinated synthesis or wiki drift rates in LLM-maintained wikis. The search for HN criticism returned nothing relevant, and the mpt.solutions post was unreachable.
- No source was found that quantifies review burden (for example, minutes per ingest) for LLM wikis.
