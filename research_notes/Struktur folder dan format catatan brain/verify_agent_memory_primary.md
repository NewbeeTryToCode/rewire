# Primary-source verification: LLM Wiki and file-based agent memory claims

Access date for every source below: **2026-10-28**. Scope: claims in `reports/Struktur folder dan format catatan brain.md` (the report) and `research_notes/Struktur folder dan format catatan brain/llm_wiki_agent_memory.md` / `obsidian_claude.md` (the notes) that are sourced to Karpathy's LLM Wiki gist, Cline Memory Bank, Basic Memory, Letta, A-MEM, Zep/Graphiti and MemGPT.

Method. Every source was downloaded with curl and read in full as text (no summarizing fetch tool):
- Gist: the rendered page at gist.github.com, with the `<article class="markdown-body">` body extracted in full (11,764 characters, created `2026-04-04T16:25:13Z`). gist.githubusercontent.com (raw) and api.github.com are still blocked (proxy 403). The comment thread was not re-read.
- Docs: Mintlify/Markdown `.md` endpoints of docs.cline.bot, docs.basicmemory.com and docs.letta.com.
- Papers: the full arXiv HTML versions. The current versions are 2502.12110v11, 2501.13956v1 and 2310.08560v2.

Status legend:
- **CONFIRMED**: the primary source says it.
- **CORRECTED**: the claim is wrong or misquoted; the correct fact and a verbatim quote are given.
- **NOT FOUND**: the primary source does not contain it.
- **NEW**: a primary-source fact the notes and report lack.

## 1. Karpathy, "LLM Wiki" gist

### Takeaway
The notes' summary of the gist holds up: three layers, ingest/query/lint, index.md + log.md, ~100 sources, qmd, and filing answers back. Three quotes are wrong, though, and one matters for the recommendation. **The phrase "with close supervision" does not appear in the gist.** The gist calls one-at-a-time ingest a personal preference ("stay involved") and explicitly allows batch ingest "with less supervision". The report's argument for human gates G2/G3 (line 201) cites a quote that does not exist and has to be reworded.

### Cited Findings
All findings in this section come from the [Karpathy gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f) (accessed 2026-10-28).

- **CONFIRMED: identity, date, purpose.** Title "LLM Wiki"; the page timestamp is 2026-04-04T16:25:13Z. Quote: "A pattern for building personal knowledge bases using LLMs." / "This is an idea file, it is designed to be copy pasted to your own LLM Agent (e.g. OpenAI Codex, Claude Code, OpenCode / Pi, or etc.). Its goal is to communicate the high level idea, but your agent will build out the specifics in collaboration with you."
- **CONFIRMED: section headings**, in order: LLM Wiki / The core idea / Architecture / Operations / Indexing and logging / Optional: CLI tools / Tips and tricks / Why this works / Note.
- **CONFIRMED: "persistent, compounding artifact"** (report line 61). Quote: "This is the key difference: the wiki is a persistent, compounding artifact. The cross-references are already there. The contradictions have already been flagged."
- **CORRECTED: the "compiled once" quote** (notes §4, "compiled once and kept up to date, not re-derived per query"). Verbatim: "The knowledge is compiled once and then kept current, not re-derived on every query."
- **CONFIRMED: contradiction noting during ingest.** Quote: "updating entity pages, revising topic summaries, noting where new data contradicts old claims, strengthening or challenging the evolving synthesis."
- **CONFIRMED: three layers.** Quotes:
  - "Raw sources — your curated collection of source documents. Articles, papers, images, data files. These are immutable — the LLM reads from them but never modifies them. This is your source of truth."
  - "The wiki — a directory of LLM-generated markdown files. Summaries, entity pages, concept pages, comparisons, an overview, a synthesis. The LLM owns this layer entirely."
  - "The schema — a document (e.g. CLAUDE.md for Claude Code or AGENTS.md for Codex) … You and the LLM co-evolve this over time as you figure out what works for your domain."
- **CORRECTED: "The user reads it; the LLM writes it."** (notes §1). Verbatim: "You read it; the LLM writes it."
- **CORRECTED (minor): the `raw/` + `wiki/` directory names** (report table line 52 "`raw/` immutable"; notes §1 inferences). The gist names the *layers* "Raw sources", "The wiki" and "The schema". It does not prescribe directory names. The only path it shows is an example attachment folder: "set "Attachment folder path" to a fixed directory (e.g. raw/assets/)". `wiki/` never appears. The notes correctly mark the `raw/` + `wiki/` layout as coming from secondary write-ups.
- **CONFIRMED: the LLM owns the writing; the human curates.** Quote: "You never (or rarely) write the wiki yourself — the LLM writes and maintains all of it. You're in charge of sourcing, exploration, and asking the right questions." And: "Obsidian is the IDE; the LLM is the programmer; the wiki is the codebase."
- **CONFIRMED: ingest flow and the 10–15 pages figure.** Quote: "the LLM reads the source, discusses key takeaways with you, writes a summary page in the wiki, updates the index, updates relevant entity and concept pages across the wiki, and appends an entry to the log. A single source might touch 10-15 wiki pages."
- **CORRECTED: "close supervision" (gap filled).** This affects report line 201 ("Karpathy sendiri memilih ingest satu sumber "with close supervision"") and notes §1 and §6. The words "close supervision" are NOT FOUND anywhere in the gist; "supervision" appears once. Verbatim: "Personally I prefer to ingest sources one at a time and stay involved — I read the summaries, check the updates, and guide the LLM on what to emphasize. But you could also batch-ingest many sources at once with less supervision. It's up to you to develop the workflow that fits your style and document it in the schema for future sessions."
- **NEW: human review mentioned only for teams.** In the use-case list: "Business/team: an internal wiki maintained by LLMs, fed by Slack threads, meeting transcripts, project documents, customer calls. Possibly with humans in the loop reviewing updates."
- **CONFIRMED: filing answers back.** Quote: "The important insight: good answers can be filed back into the wiki as new pages. A comparison you asked for, an analysis, a connection you discovered — these are valuable and shouldn't disappear into chat history." Output formats named: "a markdown page, a comparison table, a slide deck (Marp), a chart (matplotlib), a canvas."
- **CONFIRMED: lint targets.** Quote: "Lint. Periodically, ask the LLM to health-check the wiki. Look for: contradictions between pages, stale claims that newer sources have superseded, orphan pages with no inbound links, important concepts mentioned but lacking their own page, missing cross-references, data gaps that could be filled with a web search."
- **CONFIRMED: index.md and scale (gap filled, verbatim).** "index.md is content-oriented. It's a catalog of everything in the wiki — each page listed with a link, a one-line summary, and optionally metadata like date or source count. Organized by category (entities, concepts, sources, etc.). The LLM updates it on every ingest. When answering a query, the LLM reads the index first to find relevant pages, then drills into them. This works surprisingly well at moderate scale (~100 sources, ~hundreds of pages) and avoids the need for embedding-based RAG infrastructure."
- **CORRECTED (nuance): "teruji sampai ±100 sumber"** (report line 52, "tested up to about 100 sources"). The gist reports experience ("works surprisingly well"). It does not describe a test, and it does not say the approach fails beyond that scale.
- **CONFIRMED: log.md.** Quote: "log.md is chronological. It's an append-only record of what happened and when — ingests, queries, lint passes. A useful tip: if each entry starts with a consistent prefix (e.g. ## [2026-04-02] ingest | Article Title), the log becomes parseable with simple unix tools — grep "^## \[" log.md | tail -5 gives you the last 5 entries."
- **CONFIRMED: qmd (gap filled, verbatim).** "A search engine over the wiki pages is the most obvious one — at small scale the index file is enough, but as the wiki grows you want proper search. qmd is a good option: it's a local search engine for markdown files with hybrid BM25/vector search and LLM re-ranking, all on-device. It has both a CLI (so the LLM can shell out to it) and an MCP server (so the LLM can use it as a native tool). You could also build something simpler yourself — the LLM can help you vibe-code a naive search script as the need arises."
- **CONFIRMED: tips.** Obsidian Web Clipper, downloading images locally, graph view, Marp, Dataview. Git quote: "The wiki is just a git repo of markdown files. You get version history, branching, and collaboration for free."
- **CONFIRMED: rationale and Memex.** Quote: "Humans abandon wikis because the maintenance burden grows faster than the value. LLMs don't get bored, don't forget to update a cross-reference, and can touch 15 files in one pass." Memex: "related in spirit to Vannevar Bush's Memex (1945)".
- **CONFIRMED: deliberate abstraction.** Quote: "This document is intentionally abstract. It describes the idea, not a specific implementation. The exact directory structure, the schema conventions, the page formats, the tooling — all of that will depend on your domain".
- **NOT FOUND in the gist body: "quadratic lint" and "permanently poison the store"** (report line 61). The report correctly attributes both to rewire's own `docs/research/karpathy-llm-wiki.md`, which takes them from the **comment thread**, not from Karpathy's text. The comment thread was not re-verified in this pass.

### Inferences
- **Report line 201 needs rewording.** Karpathy does not choose supervision on principle; he states a personal preference ("I prefer … stay involved") and says less supervision is a legitimate option. The case for human gates G2/G3 should rest on rewire's own evidence (the duplicated inbox preferences, "permanently poison the store" from the comments) and on the gist's team line "Possibly with humans in the loop reviewing updates". The 10–15 pages per source figure is still accurate.
- **Possible tension with the recommendation.** The gist's wiki layer is *entirely* LLM-written ("You never (or rarely) write the wiki yourself"). The report's hub content is "di-generate atau dikurasi" (generated or curated). This does not conflict, but rewire's human curation of hubs goes beyond what Karpathy describes.
- **The scale claim is softer than the report says.** The gist itself names the next step once the index stops being enough (qmd: BM25 + vector + LLM re-rank). That matches the report's point that rewire's existing FTS/recall covers the scale issue.

### Gaps
- The raw gist endpoint and the GitHub API are still blocked, so the revision history could not be checked. The text above is the rendered current revision.
- The comment thread (the source of the "quadratic lint" and "poison the store" lessons) was not re-read against the primary page.

## 2. Cline Memory Bank (docs.cline.bot)

### Takeaway
Every Cline claim in the notes and the report is confirmed by the official docs page, which carries the same custom-instructions text as the GitHub `cline/prompts` file the notes cited. Nothing here changes the recommendation. The docs add explicit update cadence: activeContext after each session, a full update "every few sessions".

### Cited Findings
All findings in this section come from [Cline docs: Memory Bank](https://docs.cline.bot/best-practices/memory-bank.md) (accessed 2026-10-28).

- **CONFIRMED: six core files, fixed per project.** The tree shows `memory-bank/` with projectbrief.md ("Foundation document"), productContext.md, activeContext.md, systemPatterns.md, techContext.md and progress.md. The table entry for activeContext.md reads "Current focus, recent changes, next steps (updates most frequently)".
- **CONFIRMED: hierarchy.** Quote: "projectbrief.md at the top flowing into productContext, systemPatterns, and techContext, which feed into activeContext and progress". projectbrief.md is the "Source of truth for project scope".
- **CONFIRMED: read everything on every task.** Quote: "I MUST read ALL memory bank files at the start of EVERY task - this is not optional."
- **CONFIRMED: update triggers.** Quote: "Memory Bank updates occur when: 1. Discovering new project patterns 2. After implementing significant changes 3. When user requests with **update memory bank** (MUST review ALL files) 4. When context needs clarification".
- **CONFIRMED: additional files.** Quote: "Create additional files/folders within memory-bank/ when they help organize: Complex feature documentation, Integration specifications, API documentation, Testing strategies, Deployment procedures".
- **CONFIRMED: stable versus volatile split** (report line 61). progress.md covers "What works / What's left to build / Current status / Known issues / Evolution of project decisions".
- **NEW: cadence.** Quotes: "`activeContext.md` changes most frequently; update it after each session" / "`progress.md` tracks milestones; review it when resuming work" / "After significant milestones or direction changes. For active development, every few sessions." / "Start with a basic project brief and let structure evolve".
- **CONFIRMED (with nuance): "K3: tidak ada global"** (no global layer). The memory bank lives in the project ("regular markdown files in your project"). The *instructions* can be global: "Custom instructions apply globally across all projects. A Cline Rules file is project-specific and stored in your repo". No global memory content is described.

### Inferences
- The "update after each session" volatile file plus a stable brief directly supports a hub template with a stable summary section and a volatile "current state" section. This is consistent with the report.

### Gaps
- None for the claims in scope.

## 3. Basic Memory (docs.basicmemory.com)

### Takeaway
The format claims are confirmed: `- [category] content #tag (context)` observations, `relation_type [[Target]]` relations, and YAML frontmatter. Two corrections: the frontmatter also has `created`/`modified`, and the current docs name the MCP tools `write_note`/`read_note`/`search_notes`/`build_context`, not the `bm_*` names in the notes. v0.24 (October 2026) adds facts the notes lack that bear on the recommendation:
- **valid-time qualifiers on observation lines** (`@effective[2026-06-10,2026-07-27)`);
- `locked: true` notes that AI tools cannot edit;
- similar-note advisories with "Nothing is merged automatically";
- schema validation that "warns rather than blocking writes".

### Cited Findings
- **CONFIRMED: observation pattern.** Quote: "The pattern is: `- [category] content #optional-tags (optional context)`" / "Categories can be anything that makes sense … There's no fixed list." — [Knowledge Format](https://docs.basicmemory.com/raw/concepts/knowledge-format.md)
- **CONFIRMED: observations are indexed one by one.** Quote: "each one is indexed individually. When you search your knowledge base, Basic Memory can surface the specific fact you need rather than just pointing you at a whole document." — [Knowledge Format](https://docs.basicmemory.com/raw/concepts/knowledge-format.md)
- **CONFIRMED: relations and `links_to`.** Quotes: "The word before the link becomes the relationship type." / "a bare or prose list item like `- [[Caching Layer]]` or `- see also [[Caching Layer]]` is indexed as a generic `links_to` connection. (Changed in v0.21.0)". Multi-word types must be quoted: `"depends on" [[Caching Layer]]`. — [Observations and relations](https://docs.basicmemory.com/raw/concepts/observations-and-relations.md)
- **CORRECTED (incomplete): frontmatter fields** (notes: "`title`, `type`, `permalink`, `tags`"). The documented example also has `created` and `modified`: "**created** and **modified** — Optional authoritative timestamps for the note. When present, Basic Memory uses them for recency ordering instead of filesystem timestamps." The permalink is "Generated automatically from the file's path (and prefixed with the project name by default) … Stays the same even if you later move the file." — [Knowledge Format](https://docs.basicmemory.com/raw/concepts/knowledge-format.md)
- **CONFIRMED: the coffee example from the README.** The docs use a different example ("Authentication Design", with `- [decision] Using JWT tokens for stateless authentication #security`, `- implements [[API Security Requirements]]`); the README example was not re-checked. — [Knowledge Format](https://docs.basicmemory.com/raw/concepts/knowledge-format.md)
- **CORRECTED: MCP tool names** (notes: "`bm_write`, `bm_read`, … `bm_man`"). The current docs never use `bm_*` MCP names (0 occurrences on the local MCP tools page). They name `write_note`, `read_note`, `edit_note`, `search_notes`, `build_context`, `schema_validate` and the v0.24 POSIX tools. Quotes: "`write_note` to create or update a note"; v0.24: "Six POSIX-style read tools now ship **on by default** next to the rich tools: `cat`, `grep`, `ls`, `find`, `tail`, and `man`." The CLI uses `bm …` verbs (`bm cat`, `bm grep`, `bm schema validate`), which may explain the notes' confusion. — [MCP tools (local)](https://docs.basicmemory.com/raw/local/mcp-tools-local.md); [v0.24](https://docs.basicmemory.com/raw/whats-new/v0-24.md)
- **CONFIRMED (notes gap closed): the `memory://` URL scheme exists.** Quote: "Every note in Basic Memory has an address — a `memory://` URL that works like a web link for your knowledge base." — [Memory URLs](https://docs.basicmemory.com/raw/concepts/memory-urls.md)
- **CONFIRMED: SQLite local index.** Quote: "[tool] Basic Memory uses SQLite for local indexing" (example line); v0.24: "FastEmbed and sqlite-vec are core dependencies". — [Observations and relations](https://docs.basicmemory.com/raw/concepts/observations-and-relations.md); [v0.24](https://docs.basicmemory.com/raw/whats-new/v0-24.md)
- **NEW: valid time on observations (v0.24).** Quotes: "As of v0.24, an observation can say when it holds with a qualifier placed right after the category: `- [decision] @effective:2026-06-10 The cache layer will use Redis.` / `- [decision] @effective[2026-06-10,2026-07-27) Redis served cache traffic until the cutover.`" / "The qualifier is indexed as **valid time** — when the note *says* something was true — which is separate from when the note was written or edited." Kinds: `effective`, `valid` (default), `occurred`, `due`, `mentioned`. "Relative dates aren't accepted." `search_notes` gains "`valid_at`, `valid_overlaps`, and `time_kind` filters". — [Observations and relations](https://docs.basicmemory.com/raw/concepts/observations-and-relations.md); [v0.24](https://docs.basicmemory.com/raw/whats-new/v0-24.md)
- **NEW: locked notes.** Quote: "Set the YAML boolean `locked: true` … writes, edits, overwrites, and deletes through the note API — which is what AI tools and `bm tool` commands use — are refused with HTTP `423` … That makes it a guard against accidental AI edits, not access control." — [Knowledge Format](https://docs.basicmemory.com/raw/concepts/knowledge-format.md)
- **NEW: no automatic merge; revision guards.** Quotes: "**Similar-note advisory.** When a new note looks like notes that already exist, `write_note` lists them and asks whether you meant to update one of them. … Nothing is merged automatically" / "Pass `expected_checksum` to `edit_note` … The write applies only while the note is still the revision you read." — [v0.24](https://docs.basicmemory.com/raw/whats-new/v0-24.md)
- **NEW: schema validation warns, does not block.** Quote: "You can then validate notes on demand … and the report flags anything that's missing. Validation warns rather than blocking writes." — [Observations and relations](https://docs.basicmemory.com/raw/concepts/observations-and-relations.md)
- **NEW: deleting a note keeps inbound links.** Quote: "Deleting a note doesn't erase the relations that pointed at it. They go back to the unresolved state … (Before v0.24, deleting a note silently removed its inbound relations.)" — [Observations and relations](https://docs.basicmemory.com/raw/concepts/observations-and-relations.md)
- **NEW: the folder tree does not matter to the graph.** Quote: "Organize your files however you want — the knowledge graph is built from the content of your notes, not from where they sit on disk." — [Knowledge Format](https://docs.basicmemory.com/raw/concepts/knowledge-format.md)

### Inferences
- **Refines, but does not change, the recommendation.** The report's "Observasi" (observations) format on concept and hub pages can carry rewire's valid time inline, e.g. `- [decision] @effective[2026-06-10,) …` (v0.24 syntax). A compiled page would then keep each atomic claim's `valid_from`/`valid_to` instead of flattening them. This strengthens the "compiled page rebuildable from atomic layer" argument.
- **`locked: true` is prior art for protecting human-curated hub sections** from LLM rewrites.
- **"Nothing is merged automatically" is prior art for human-gated de-dup and promotion**, a better citation for gates G2/G3 than the misquoted Karpathy line.
- **"Built from content, not where they sit on disk"** independently supports the report's core point: do not reorganize folders.

### Gaps
- The basic-memory GitHub README (the source of the notes' `bm_*` names and coffee example) was not re-fetched. If the README really lists `bm_*` names, the README and the docs disagree; the docs are newer (v0.24, October 2026).

## 4. Letta (docs.letta.com): memory blocks, MemFS, git sync, dreaming

### Takeaway
The MemFS/git, dreaming and scope claims are confirmed by the docs and the letta-code README. The memory-block schema gap is now filled. Classic blocks (labelled **legacy** in the docs) have `label`, `description`, `value`, `limit` (characters) and `read_only`, and the agent sees `chars_current`/`chars_limit` metadata in context; the example limit is 5000. In current Letta, each memory is a Markdown file with `name`/`description` frontmatter in a git repo: root files are always in context, and subdirectories with their own `MEMORY.md` index load on demand. Dreaming (the sleep phase) runs as background subagents in git worktrees. Its optional review step is done by the agent and "does not ask you for approval".

### Cited Findings
- **CONFIRMED + gap filled: memory-block schema (legacy API).** The page is titled "Memory blocks (core memory)", with front-matter `status: legacy`. Quotes: "A `label`, which is a unique identifier for the block / A `description`, which describes the purpose of the block / A `value`, which is the contents/data of the block / A `limit`, which is the size limit (in characters) of the block". Read-only: "Memory blocks are read-write by default … but can be set to read-only by setting the `read_only` field to `true`." In-context rendering: "memory blocks are simply prepended to the agent's prompt in an XML-like format", with `<metadata> - chars_current=128 - chars_limit=5000`. On descriptions: "The `description` is the main information used by the agent to determine how to read and write to that block." — [Letta: Memory blocks (legacy)](https://docs.letta.com/v1-sdk/memory/memory-blocks/index.md)
- **NEW: shared blocks are legacy.** Quote: "Older Letta API guides use **shared memory blocks** for one in-context memory block attached to multiple agents. The legacy memory block API still supports this pattern, but users should migrate to shared memory repositories instead." Shared repositories are Git repos "hosted in Letta Cloud and owned by your organization"; "The agent must commit and push its changes." — [Letta: Shared memory](https://docs.letta.com/concepts/shared-memory/index.md)
- **CONFIRMED: MemFS is git-tracked and synced.** Quotes: "Every memory edit is committed to the MemFS git repository. This provides version history, conflict resolution, and a clear boundary between saved memory and uncommitted changes." / "Local-only agents commit to a repository on the current machine, so you are responsible for backing it up." — [Letta: MemFS](https://docs.letta.com/concepts/memfs/index.md). README: "All context (including memory blocks) is tracked via git. Sync context to a custom GitHub repository by setting `/memory-repository set git@github.com:...`" — [letta-code README](https://github.com/letta-ai/letta-code)
- **NEW: the MemFS file format and in-context rule.** Quotes:
  - "Each memory except a `MEMORY.md` index is projected as Markdown with YAML frontmatter" (fields `name`, `description`).
  - "Files at the memory root are loaded into the agent's system prompt on every turn. Use them for the agent's identity, important user preferences, durable project facts, and critical workflow rules."
  - "Directories with their own `MEMORY.md` index stay out of context until they are needed. The root `MEMORY.md` lists them, so directory and file names act as signposts … This keeps the active context lean while preserving deeper reference material."
  - SDK: "Files under `system/` are in the system prompt every turn. Everything else stays out of context — the agent sees the file tree and reads what it needs."
  — [Letta: MemFS](https://docs.letta.com/concepts/memfs/index.md); [Letta: Memory (SDK)](https://docs.letta.com/agent-sdk/memory/index.md)
- **NEW: no vector index by default; qmd as an option.** Quote: "MemFS does not include a semantic or vector index by default. Agents find memory in its Markdown files with normal file-search and read tools." / "Semantic and hybrid modes require QMD to be installed". This is the same tool Karpathy names. — [Letta: MemFS](https://docs.letta.com/concepts/memfs/index.md)
- **CONFIRMED: dreaming / sleep-time.** Quotes:
  - "Dreaming uses background subagents to review recent conversations, consolidate useful lessons, and update memory without interrupting your active work."
  - "Configure dreaming with `/sleeptime` in the CLI … Choose when it runs: after a set number of completed agent steps or when the context window is compacted."
  - SDK: `trigger: "step-count" // "off" | "step-count" | "compaction-event"`, `behavior: "auto-launch" // "reminder" | "auto-launch"`.
  - Concurrency: "Memory subagents such as dreaming and memory doctor use git worktrees so they can update memory concurrently without blocking the main agent."
  — [Letta: Memory & dreaming](https://docs.letta.com/configuration/memory/index.md); [Letta: Memory (SDK)](https://docs.letta.com/agent-sdk/memory/index.md); [Letta: MemFS](https://docs.letta.com/concepts/memfs/index.md)
- **NEW: the review step is done by the agent, with no human approval.** Quote: "Select **Agent reviews before applying** to have your agent review and revise proposed memory updates in a second background conversation. This uses more model tokens and does not ask you for approval." — [Letta: Memory & dreaming](https://docs.letta.com/configuration/memory/index.md)
- **NEW: lint/doctor and reorganization.** Quotes: "If the memory hierarchy has drifted or grown too large, run `/doctor` to audit placement, duplication, and system-prompt token usage." / "The memory workflow backs up the current repository before splitting large files, merging duplicates, or restructuring the hierarchy." / "`/remember always use pnpm in this repo` … The agent decides where the lesson belongs and commits the update to MemFS." — [Letta: Memory & dreaming](https://docs.letta.com/configuration/memory/index.md)
- **NEW: storage limits.** Quote: "The Free plan includes **5 MiB per memory repository**; Pro includes **50 MiB per memory repository**." — [Letta: Memory & dreaming](https://docs.letta.com/configuration/memory/index.md)
- **CONFIRMED: README commands and scopes.** Quotes: "Configure periodic dreaming with `/sleeptime`, investigate agent behavior with `/doctor [symptom]`, and view memory with `/palace`" / "Manual `/dream` and `/reflect` commands remain available." / "Loads global skills (`~/.letta`), project-scoped skills (`.agents/skills`), and agent-scoped skills (stored in MemFS)". — [letta-code README](https://raw.githubusercontent.com/letta-ai/letta-code/main/README.md)
- **NEW (naming): "Letta Code" is now "the Letta Harness".** Quote: "**The Letta Harness** (formerly Letta Code) — an open source harness designed for stateful, self-evolving agents. … It provides a git-versioned memory filesystem (MemFS), multi-conversation memory, skills, schedules, and sleep-time subagents for reflection and memory organization." — [docs.letta.com/llms.txt](https://docs.letta.com/llms.txt)

### Inferences
- **Supports the recommendation's sleep/lint phase.** Letta ships one: dreaming on a step-count or compaction trigger, `/doctor` auditing placement, duplication and token usage, and a reorganization step that backs up first. Running it in a git worktree is a concrete design for rewire to run consolidation without blocking sessions.
- **Supports the two-layer hub/on-demand split.** Letta's "root files always in context, subdirectory files on demand via MEMORY.md signposts" is the same as the report's plan: the snapshot takes only the hub summary, and detail loads through recall.
- **Contrast for the human gates.** Letta's only review option is agent-on-agent and explicitly "does not ask you for approval". Rewire's human gates G2/G3 are therefore a deliberate design choice beyond Letta's practice, not something Letta validates. The report's table line 56 ("Validasi bahwa 'brain git ditulis agen' itu layak") remains accurate.
- **Hub length cap.** Classic block `limit` (characters) with `chars_current`/`chars_limit` shown to the model is prior art for showing a hub's size budget to the LLM when enforcing the ≤300-word hub cap.

### Gaps
- The default value of a block's `limit` is not stated on the legacy page; 5000 appears only in examples.
- The "formerly MemGPT" lineage was not re-checked on letta-ai/letta in this pass (not fetched).

## 5. A-MEM (arXiv 2502.12110)

### Takeaway
The architecture is confirmed: Zettelkasten-style notes with LLM-generated keywords, tags and context, link generation, and memory evolution. Two corrections:
1. The "45.85 vs 25.52" figure is **one category (the Temporal column of Table 1, GPT-4o-mini)**, not an overall LoCoMo F1.
2. A-MEM is **not append-only**: memory evolution rewrites existing notes' attributes. Report table line 58 lumps "Mem0, A-MEM" under "Ekstraksi atomik ADD-only" (ADD-only atomic extraction); ADD-only applies to Mem0 only.

### Cited Findings
All findings in this section come from [A-MEM, arXiv 2502.12110 (v11 HTML)](https://arxiv.org/html/2502.12110), first submitted 2025-02-17, accessed 2026-10-28.

- **CONFIRMED: note construction.** Quote: "When a new memory is added, we generate a comprehensive note containing multiple structured attributes, including contextual descriptions, keywords, and tags." Note fields: "$c_i$ represents the original interaction content, $t_i$ is the timestamp of the interaction, $K_i$ denotes LLM-generated keywords …, $G_i$ contains LLM-generated tags …, $X_i$ represents the LLM-generated contextual description …, and $L_i$ maintains the set of linked memories". Atomicity: "Following the Zettelkasten principle of atomicity, each note captures a single, self-contained unit of knowledge."
- **CONFIRMED: link generation with an LLM step.** Quote: "The link generation process first retrieves the most relevant historical memories and then employs an LLM to determine whether connections should be established between them."
- **CORRECTED: "ADD-only" grouping.** A-MEM mutates existing notes. Quote: "After creating links for the new memory, A-Mem evolves the retrieved memories … For each memory $m_j$ in the nearest neighbor set … the system determines whether to update its context, keywords, and tags." Abstract: "new memories … can trigger updates to the contextual representations and attributes of existing historical memories".
- **CORRECTED: benchmark figure** (notes: "LoCoMo F1 of 45.85 versus MemGPT's 25.52 with GPT-4o-mini"). In Table 1 (GPT-4o-mini rows) the column order is Multi Hop | Temporal | Open Domain | Single Hop | Adversial, each with F1 and BLEU:
  - MemGPT row: "26.65 | 17.72 | 25.52 | 19.44 | 9.15 | 7.44 | 41.04 | 34.34 | 43.29 | 42.73"
  - A-Mem row: "27.02 | 20.09 | 45.85 | 36.67 | 12.14 | 12.00 | 44.65 | 37.06 | 50.03 | 49.47"

  So 45.85 vs 25.52 is the **Temporal** F1. Multi-Hop F1 is 27.02 vs 26.65. The prose elsewhere says A-Mem is "particularly doubling performance on complex multi-hop reasoning tasks", which is inconsistent with the column header as rendered in v11. Treat the per-category labels with caution.
- **NEW: cost.** Quote: "The system requires approximately 1,200 tokens per memory operation, achieving an 85-93% reduction in token usage compared to baseline methods (LoCoMo and MemGPT with 16,900 tokens) through our selective top-k retrieval mechanism." Retrieval uses "$k$=10 for top-$k$ memory selection" and the "all-minilm-l6-v2" embedding model.
- **NEW: ablation.** Quote: "When both LG and ME modules are removed, the system exhibits substantial performance degradation, particularly in Multi Hop reasoning and Open Domain tasks."

### Inferences
- **The report's use of A-MEM stands** ("Konfirmasi bahwa lapis atomik tetap perlu", confirming that the atomic layer is still needed), since A-MEM is atomic-note based. But A-MEM is evidence *for* LLM-driven updates to existing notes (link + evolve), which matches a sleep/consolidation phase. It is not evidence for append-only storage. The table cell should say "ADD-only (Mem0); evolving Zettelkasten notes (A-MEM)".

### Gaps
- Which column is really Temporal versus Multi-Hop could not be settled from the HTML alone; the PDF was not cross-checked.

## 6. Zep / Graphiti (arXiv 2501.13956)

### Takeaway
The bitemporal model and invalidate-not-delete are confirmed from the paper. Graphiti tracks four timestamps, and a contradicted edge gets `t_invalid` set to the new edge's `t_valid` while history is kept. The benchmark numbers (gap) are DMR 94.8% vs MemGPT 93.4% (gpt-4-turbo) and LongMemEval 63.8%/71.2% vs full-context 55.4%/60.2%. The paper itself warns that DMR conversations are only 60 messages. **NEW:** Zep has a third, compiled tier, the *community subgraph* of "high-level summarizations", above the atomic entity/fact tier. This is direct prior art for the report's "compiled pages above atomic claims".

### Cited Findings
All findings in this section come from [Zep, arXiv 2501.13956v1 (HTML)](https://arxiv.org/html/2501.13956), submitted 2025-01-20, accessed 2026-10-28.

- **CONFIRMED: bi-temporal model.** Quote: "Zep implements a bi-temporal model, where timeline $T$ represents the chronological ordering of events, and timeline $T'$ represents the transactional order of Zep's data ingestion." Four timestamps: "the system tracks four timestamps: $t'_{created}$ and $t'_{expired} \in T'$ monitor when facts are created or invalidated in the system, while $t_{valid}$ and $t_{invalid} \in T$ track the temporal range during which facts held true. These temporal data points are stored on edges alongside other fact information."
- **CONFIRMED: invalidation, not deletion.** Quote: "The system employs an LLM to compare new edges against semantically related existing edges to identify potential contradictions. When the system identifies temporally overlapping contradictions, it invalidates the affected edges by setting their $t_{invalid}$ to the $t_{valid}$ of the invalidating edge. Following the transactional timeline $T'$, Graphiti consistently prioritizes new information when determining edge invalidation." Result: "maintaining both current relationship states and historical records of relationship evolution over time." The word "delete" is not used; "not deleted" is implied by keeping the historical records.
- **CONFIRMED: episodes as non-lossy provenance.** Quotes: "Episodes serve as a non-lossy data store from which semantic entities and relations are extracted." / "semantic artifacts can be traced to their sources for citation or quotation". This supports report line 161 (`sessions/` = Graphiti episodes).
- **NEW: three tiers, including a compiled summary tier.** Quotes: "This graph comprises three hierarchical tiers of subgraphs: an episode subgraph, a semantic entity subgraph, and a community subgraph." / "Community nodes … represent clusters of strongly connected entities. Communities contain high-level summarizations of these clusters" / "from episodes to facts to entities to communities".
- **NEW: anti-hallucination step and entity dedup at write time.** Quotes: "we employ a reflection technique inspired by reflexion to minimize hallucinations and enhance extraction coverage." / "When the system identifies a duplicate entity, it generates an updated name and summary."
- **Gap filled: DMR numbers.** Quotes: "The MemGPT framework currently leads performance metrics with 93.4% accuracy using gpt-4-turbo, a significant improvement over the 35.3% baseline achieved through recursive summarization." / "Using gpt-4-turbo, the full-conversation baseline achieved 94.4% accuracy … session summary baseline achieved 78.6%. When using gpt-4o-mini … 98.0% for full-conversation and 88.0% for session summaries." / "Zep achieved 94.8% accuracy with gpt-4-turbo and 98.2% with gpt-4o-mini". Caveat: "these results must be contextualized: each conversation contains only 60 messages, easily fitting within current LLM context windows."
- **Gap filled: LongMemEval numbers.** Table 2 (LongMemEval_s, about 115k-token contexts):
  | Setup | Score | Latency | Context tokens |
  |---|---|---|---|
  | Full-context, gpt-4o-mini | 55.4% | 31.3 s | 115k |
  | Zep, gpt-4o-mini | 63.8% | 3.20 s | 1.6k |
  | Full-context, gpt-4o | 60.2% | 28.9 s | 115k |
  | Zep, gpt-4o | 71.2% | 2.58 s | 1.6k |

  Quote: "Using gpt-4o-mini, Zep achieved a 15.2% accuracy improvement over the baseline, while gpt-4o showed an 18.5% improvement." Regression: "The decrease in performance for single-session-assistant questions—17.7% for gpt-4o and 9.06% for gpt-4o-mini". The improvements are relative percentages; the absolute gains are 8.4 and 11.0 points.
- **CONFIRMED: hybrid retrieval.** The paper describes cosine similarity, full-text (BM25) and breadth-first graph search, plus rerankers (RRF, MMR, episode-mentions, node distance, cross-encoder).

### Inferences
- **Strengthens the recommendation.** Zep's own architecture is exactly "atomic facts (entity edges with validity windows) + a compiled summary tier (communities) + raw episodes", the three-layer shape the report proposes (`sessions/` → atomic claims → hub/concept pages).
- Rewire's `recorded_at` (transaction time) + `valid_from`/`valid_to` (valid time) in `rust/src/schema.rs` matches Graphiti's bi-temporal split. This supports the report's "bitemporal ala Graphiti" (line 216). Graphiti additionally records `t'_expired` (when the system invalidated a fact). Rewire's SUPERSEDE fills `valid_to`; whether it also records when the supersession was made is not checked here.
- Graphiti detects contradictions at **write time**, by comparing new edges with semantically related existing edges. This argues for contradiction checks during consolidation, not only in a later lint pass.

### Gaps
- None for the claims in scope.

## 7. MemGPT (arXiv 2310.08560)

### Takeaway
The gap is filled, with one terminology correction. The paper's split is **main context** (prompt tokens) versus **external context**:
- Main context has three parts: read-only system instructions, a fixed-size read/write **working context**, and a FIFO message queue whose first entry is a recursive summary.
- External context has two stores: **archival storage** (a read/write database of arbitrary-length text, vector-searched) and **recall storage** (the full message history).

The notes' "core memory" is later Letta terminology. In the paper the term appears only inside an appendix prompt; the architecture section calls it "working context".

### Cited Findings
All findings in this section come from [MemGPT, arXiv 2310.08560v2 (HTML)](https://arxiv.org/html/2310.08560), first submitted 2023-10-12, accessed 2026-10-28.

- **CORRECTED (terminology): main versus external context.** Quotes: "MemGPT's OS-inspired multi-level memory architecture delineates between two primary memory types: main context (analogous to main memory/physical memory/RAM) and external context (analogous to disk memory/disk storage)." / "MemGPT uses functions to move data between main context and external context (the archival and recall storage databases)."
- **Gap filled: main context composition.** Quote: "The prompt tokens in MemGPT are split into three contiguous sections: the system instructions, working context, and FIFO Queue. The system instructions are read-only (static) … Working context is a fixed-size read/write block of unstructured text, writeable only via MemGPT function calls. In conversational settings, working context is intended to be used to store key facts, preferences, and other important information about the user and the persona the agent is adopting".
- **Gap filled: FIFO queue, memory pressure, recall storage.** Quotes:
  - "The first index in the FIFO queue stores a system message containing a recursive summary of messages that have been evicted from the queue."
  - "When the prompt tokens exceed the 'warning token count' of the underlying LLM's context window (e.g. 70% of the context window), the queue manager inserts a system message into the queue warning the LLM of an impending queue eviction (a 'memory pressure' warning) to allow the LLM to use MemGPT functions to store important information contained in the FIFO queue to working context or archival storage (a read/write database storing arbitrary length text objects)."
  - "the evicted messages are no longer in-context … however they are stored indefinitely in recall storage and readable via MemGPT function calls."
- **CONFIRMED: archival storage is vector-searched.** Quote: "the retriever naturally emerges via the archival storage search functionality (which performs vector search based on cosine similarity)."
- **CONFIRMED: self-directed memory edits.** Quote: "MemGPT provides function calls that the LLM processor to manage its own memory without any user intervention."
- **NEW: "core memory" appears only in an appendix prompt.** Quote: "I should reply with a best guess using the information in core memory and conversation …" (DMR instructions). Letta's legacy docs title memory blocks "Memory blocks (core memory)" — [Letta legacy](https://docs.letta.com/v1-sdk/memory/memory-blocks/index.md).
- **NEW: DMR results (as cited by Zep).** "GPT-4 Turbo | 35.3% | 0.359 | + MemGPT | 93.4% | 0.827" (accuracy, ROUGE-L). Also "GPT-4 | 32.1% … + MemGPT | 92.5%" and "GPT-3.5 Turbo | 38.7% … + MemGPT | 66.9%".

### Inferences
- MemGPT's fixed-size, always-in-context working context, written only through function calls, is the prototype for a size-capped hub/summary in the rewire snapshot. Its "memory pressure" warning at about 70% is prior art for warning the consolidator as a hub nears its ≤300-word cap. None of this changes the recommendation.

### Gaps
- None for the claims in scope.

## 8. Facts that would change or adjust the report's recommendation

### Takeaway
No primary source contradicts the core recommendation: atomic claims below, compiled hub/concept pages above, inside `nodes/`, with promotion gates and a sleep/lint phase. Several sources strengthen it (Zep's community tier, Letta's root-in-context plus on-demand directories and dreaming/doctor, Basic Memory's "built from content, not where they sit on disk"). One citation must be fixed (Karpathy's "close supervision"), and two small adjustments are worth making.

### Cited Findings
- **Fix: Karpathy "close supervision" (report line 201).** The quote does not exist. Verbatim: "Personally I prefer to ingest sources one at a time and stay involved … But you could also batch-ingest many sources at once with less supervision." — [Karpathy gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
- **Better support for human gates:**
  - Karpathy's team line: "Possibly with humans in the loop reviewing updates." — [Karpathy gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
  - Basic Memory: "Nothing is merged automatically" — [Basic Memory v0.24](https://docs.basicmemory.com/raw/whats-new/v0-24.md)
  - Counter-example: Letta's reviewer step "does not ask you for approval" — [Letta Memory & dreaming](https://docs.letta.com/configuration/memory/index.md)
- **Strengthens two-layer compiled-over-atomic:** "Communities contain high-level summarizations of these clusters" above the entity/fact tier — [Zep paper](https://arxiv.org/html/2501.13956). Also "Files at the memory root are loaded into the agent's system prompt on every turn … Directories with their own `MEMORY.md` index stay out of context until they are needed." — [Letta MemFS](https://docs.letta.com/concepts/memfs/index.md)
- **Strengthens the sleep/lint phase:** dreaming triggers on `"step-count" | "compaction-event"`, runs in git worktrees, and `/doctor` audits "placement, duplication, and system-prompt token usage" — [Letta SDK Memory](https://docs.letta.com/agent-sdk/memory/index.md); [Letta MemFS](https://docs.letta.com/concepts/memfs/index.md); [Letta Memory & dreaming](https://docs.letta.com/configuration/memory/index.md). Graphiti instead does contradiction checks at write time — [Zep paper](https://arxiv.org/html/2501.13956).
- **Adjustment 1, hub "Observasi" lines:** use Basic Memory v0.24 valid-time qualifiers (`- [decision] @effective[2026-06-10,) …`) so compiled pages keep each claim's validity window — [Basic Memory: Observations and relations](https://docs.basicmemory.com/raw/concepts/observations-and-relations.md).
- **Adjustment 2, table line 58:** "ADD-only" applies to Mem0, not A-MEM. A-MEM "determines whether to update its context, keywords, and tags" of existing notes — [A-MEM](https://arxiv.org/html/2502.12110).
- **Minor wording fixes:**
  - Table line 52: "`raw/` immutable" → Karpathy names a "Raw sources" layer, not a directory.
  - "teruji sampai ±100 sumber" → "works surprisingly well at moderate scale (~100 sources, ~hundreds of pages)".
  - "Letta Code" → "Letta Harness (formerly Letta Code)".
  - Notes' `bm_*` MCP names → `write_note`/`search_notes`/`build_context`.

  Sources: [Karpathy gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f); [docs.letta.com/llms.txt](https://docs.letta.com/llms.txt); [Basic Memory MCP tools](https://docs.basicmemory.com/raw/local/mcp-tools-local.md)

### Inferences
- **Report line 67 is out of date.** It says docs.cline.bot, docs.basicmemory.com, docs.letta.com and arxiv.org were blocked and the gist was read only as short excerpts. For this section's sources that limitation no longer applies; the claims are now verified against full primary text, with the corrections above.

### Gaps
- gist.githubusercontent.com (raw gist) and api.github.com are still blocked (403), so gist revision history and the comment thread were not verified.
- mpt.solutions was not in scope and was not retried.
