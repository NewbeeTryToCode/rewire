# How the rewire brain works today (source code + live brain), and where it falls short

Scope: rewire source at `/home/user/rewire` (Rust, `rust/src`), its design docs, and the live brain repo at
`/home/user/brain` (9 commits, 7 active nodes, 5 inbox items, 1 session file, as of 2026-10-28).
All citations are local file paths with line numbers. Nothing was modified.

## 1. Folder layout, node types and frontmatter fields

### Takeaway
The brain is a flat, type-partitioned store: `nodes/<type>/<slug>-<id8>.md`, one memory per file, plus `inbox/`,
`sessions/<date>/`, `events/`, `skills/`, a generated `README.md` and `brain.toml`. There are 7 node types (4 "claim"
types, 3 "hub" types) and 6 relation kinds; projects exist only as a `scope` string (`project:<slug>`) plus an
auto-created, empty `project` hub node, not as folders.

### Cited Findings
- On-disk contract ("frozen"): `brain.toml`, `README.md` (generated), `.gitattributes`, `.gitignore`,
  `nodes/<type>/<slug>-<id8>.md` ("L2 claims and hubs"), `events/<device>/<YYYY-MM>.jsonl`,
  `sessions/<YYYY-MM-DD>/<harness>-<session-slug>.md` ("L1 episodes"), `skills/<name>/SKILL.md`, `inbox/<ulid>.md`,
  derived `.index/brain.db`, device-local `.state/*.json` — [docs/design/format.md:1-19](/home/user/rewire/docs/design/format.md)
- The code creates exactly these dirs: `DIRS = ["nodes","events","sessions","skills","inbox",".index",".state"]`;
  synced ones are `nodes, events, sessions, skills, inbox` — [rust/src/brain.rs:142-145](/home/user/rewire/rust/src/brain.rs)
- Types: `CLAIM_TYPES = constraint, preference, fact, decision`; `HUB_TYPES = project, person, concept`;
  `RELATIONS = about, part_of, depends_on, related_to, supersedes, contradicts`; `CONFIDENCE = explicit|inferred`;
  `PROVENANCE = user|assistant|tool` — [rust/src/schema.rs:6-36](/home/user/rewire/rust/src/schema.rs)
- Required fields: `id, type, title, scope, importance, confidence, provenance, recorded_at`
  — [rust/src/schema.rs:37](/home/user/rewire/rust/src/schema.rs). Allowed fields list (22): `id, type, title, scope,
  importance, confidence, provenance, status, recorded_at, valid_from, valid_to, retracted_at, retracted_reason,
  source, edges, keywords, aliases, tags, created, about, project, person` — [rust/src/schema.rs:8-32](/home/user/rewire/rust/src/schema.rs)
- Validation rules: importance int 1-5; scope must be `global` or `project:<slug>`; edges must use a known relation;
  **claim bodies (constraint/preference/fact/decision) max 600 chars** ("claims must be atomic"); title 1-160 chars;
  hub types have no body limit — [rust/src/schema.rs:58-104](/home/user/rewire/rust/src/schema.rs)
- Field meanings (table): `status` derived (`superseded` if `valid_to`, else `retracted` if `retracted_at`, else
  `active`); `aliases` derived `[title]`; `tags` derived `["rewire/<type>"]`; `valid_from/valid_to` = when the claim
  held in the world; `recorded_at/retracted_at` = when written/withdrawn (bitemporal); `source` =
  `"<harness> · <session id> · turn <n>"`; `keywords` = lowercase search terms; relation keys hold
  `"[[<target file stem>]]"` lists — [docs/design/format.md:43-67](/home/user/rewire/docs/design/format.md)
- Derived keys are regenerated on every write and ignored on read; relations are mirrored after the body in a
  generated `## Related` section of relative markdown links — [docs/design/format.md:66-80](/home/user/rewire/docs/design/format.md);
  implementation [rust/src/nodefmt.rs:13-32, 141-251](/home/user/rewire/rust/src/nodefmt.rs)
- Internally edges are `meta["edges"] = [{"rel","to": <8-char id suffix>}]`; on disk they become frontmatter
  properties like `about: ["[[user-4tnc2n12]]"]` — [rust/src/nodefmt.rs:1-3, 215-235](/home/user/rewire/rust/src/nodefmt.rs)
- Unknown frontmatter keys a hand-edit adds are kept on rewrite (`for (k, v) in meta { if !ORDER.contains ... }`)
  — [rust/src/nodefmt.rs:236-240](/home/user/rewire/rust/src/nodefmt.rs); but MCP `get` returns only `schema::FIELDS`
  — [rust/src/ops.rs:208-214](/home/user/rewire/rust/src/ops.rs)
- Frontmatter values are JSON on one line each (no YAML dependency); hand-written plain YAML scalars are accepted
  — [rust/src/fm.rs:1-46](/home/user/rewire/rust/src/fm.rs); [docs/design/format.md:28-41](/home/user/rewire/docs/design/format.md)
- IDs: 26-char ULID; filename ends with the last 8 chars lower-cased; slug cut to 48 chars
  — [docs/design/format.md:21-26](/home/user/rewire/docs/design/format.md); path built as
  `nodes/<type>/<slug>-<id8>.md` in [rust/src/brain.rs:637-647](/home/user/rewire/rust/src/brain.rs)
- Defaults for a new node: scope `global`, importance 3, confidence `inferred`, provenance `assistant`,
  `valid_from` today, `valid_to` null, `retracted_at` null — [rust/src/brain.rs:614-635](/home/user/rewire/rust/src/brain.rs)
- Inbox item keys: `inbox_id, action (NEW|UPDATE|SUPERSEDE|CONTRADICT), target, type, title, scope, importance,
  confidence, provenance, keywords, edges, reason, flags (secret|injection), source, created, target_hash`
  — [docs/design/format.md:142-149](/home/user/rewire/docs/design/format.md). Live inbox items also carry
  `evidence_turn` and `valid_from` — [/home/user/brain/inbox/01M60Z44SPSZWK639AFD4YW8JS.md:1-18](/home/user/brain/inbox/01M60Z44SPSZWK639AFD4YW8JS.md)
- Session file format: header, `- date / - project / - consolidated / - memory shown / - memory used`, then
  `## Summary`, `## Applied`, `## Queued for review`, `## Rejected`, `## Questions`, `## Transcript`; several runs of
  one session separated by `---` — [docs/design/format.md:98-140](/home/user/rewire/docs/design/format.md)
- Live brain contents: `nodes/decision/` (2), `nodes/fact/` (2), `nodes/preference/` (1), `nodes/project/` (2:
  `rewire-ntgx8mpe.md`, `user-4tnc2n12.md`), `inbox/` (5), `sessions/2026-10-28/` (1 file with 3 runs); no
  `events/` or `skills/` content committed — directory listing of `/home/user/brain`.
- Layers: L0 working (harness), L1 episodic (`sessions/`), L2 semantic (`nodes/` + `events/`), L2 procedural
  (`skills/`) — [docs/design/architecture.md:6-15](/home/user/rewire/docs/design/architecture.md)

### Inferences
- "Project" is not a container in the file system: a project's memories are scattered across `nodes/fact/`,
  `nodes/decision/`, etc., linked to the hub only by `scope: "project:<slug>"` and an `about: [[hub]]` edge.
- The schema already allows extra hub-like types (`person`, `concept`) to hold long bodies; only claims are capped.

### Gaps
- `docs/design/format.md` does not list `evidence_turn`/`valid_from` for inbox items although the live files have
  them (minor doc drift).

## 2. How a memory is created: consolidation → judge → gate → inbox/nodes; scope and project mapping

### Takeaway
After a session (or after 10 min idle), one LLM call per transcript chunk extracts atomic claims, a second LLM call
judges each against ≤5 similar existing notes (NEW/SAME/UPDATE/SUPERSEDE/CONTRADICT), and a deterministic gate
auto-applies only explicit, user-stated, non-constraint NEW/SAME/UPDATE claims; everything else is written to
`inbox/` and waits for `rewire review approve|reject`. Scope is chosen by the extraction LLM as a binary
`global|project`, and "project" is resolved to a slug from `[projects]` patterns, else the git top-level directory
name, else (consolidation only) the basename of the working directory.

### Cited Findings
- Pipeline: "normalize -> filter -> extract (LLM) -> resolve (FTS + LLM judge) -> gate -> scan -> write + git
  commit -> advance watermark" — [rust/src/consolidate.rs:1-5](/home/user/rewire/rust/src/consolidate.rs)
- Triggers: SessionEnd/PreCompact hooks, or a running session idle for `REWIRE_IDLE_SECS` (600 s) via one watcher
  per brain; long sessions are consolidated in parts that see earlier parts' summaries/claims
  — [docs/design/architecture.md:43-52](/home/user/rewire/docs/design/architecture.md)
- Extraction prompt is given `SESSION DATE`, `SESSION PROJECT`, `KNOWN MEMORY` (id + title of up to 200 active
  notes in `global` + current project scope), `AWAITING REVIEW`, known skills, and the transcript chunk
  — [rust/src/consolidate.rs:579-631](/home/user/rewire/rust/src/consolidate.rs)
- Extraction rules: "One atomic, self-contained claim per item (max 2 sentences)"; absolute dates; "Describe what a
  project is or does as a 'fact' with scope 'project' (project pages are managed separately)"; "scope = 'project' if
  it only matters for the session's project, else 'global'"; importance 1-5; 8-15 keywords; optional `links`
  (`part_of|depends_on|related_to`) — [rust/src/consolidate.rs:70-106](/home/user/rewire/rust/src/consolidate.rs)
- The extraction JSON schema excludes type `project` and restricts scope to `["global","project"]`
  — [rust/src/consolidate.rs:147-199](/home/user/rewire/rust/src/consolidate.rs); a claim typed `project` is
  rewritten to a project-scoped `fact` ("hubs are system-managed") — [rust/src/consolidate.rs:867-872](/home/user/rewire/rust/src/consolidate.rs)
- Scope resolution: `project:<slug>` only if the LLM said "project" and a project was detected; otherwise
  `global` — [rust/src/consolidate.rs:883-886](/home/user/rewire/rust/src/consolidate.rs)
- Provenance is the role of the cited `evidence_turn`, never the model's say-so; confidence is downgraded to
  `inferred` unless the cited user turn shares the claim's content words
  — [rust/src/consolidate.rs:873-882](/home/user/rewire/rust/src/consolidate.rs), [350-355](/home/user/rewire/rust/src/consolidate.rs)
- Judge actions: NEW, SAME, UPDATE (refines without changing truth), SUPERSEDE (truth changed), CONTRADICT (unclear
  which is right) — [rust/src/consolidate.rs:201-208](/home/user/rewire/rust/src/consolidate.rs). Candidates are an
  FTS search with `k: 5` restricted to `global` + current project scopes; if no candidate exists at all, every claim
  is NEW without an LLM call — [rust/src/consolidate.rs:911-932](/home/user/rewire/rust/src/consolidate.rs)
- Gate: auto-apply only if `confidence == explicit && provenance == user && action in [NEW,SAME,UPDATE] && type !=
  constraint && no flags` — [rust/src/consolidate.rs:368-375](/home/user/rewire/rust/src/consolidate.rs). An
  auto-UPDATE is further refused if the target is a constraint or project hub or has a different scope
  ("notes of another scope change through review only") — [rust/src/consolidate.rs:357-366](/home/user/rewire/rust/src/consolidate.rs)
- Everything not auto-applied becomes an inbox file (with resolved edges and a `target_hash` pin)
  — [rust/src/consolidate.rs:699-786](/home/user/rewire/rust/src/consolidate.rs); skills are always proposals
  — [rust/src/consolidate.rs:788-823](/home/user/rewire/rust/src/consolidate.rs)
- Applying: `apply_change` handles NEW/SAME/UPDATE/SUPERSEDE/CONTRADICT; for any `project:<slug>` scope it calls
  `ensure_project_hub` and adds an `about` edge to the hub; SUPERSEDE writes the new note, then sets `retracted_at`
  and `valid_to` on the old one; CONTRADICT sets only `retracted_at` — [rust/src/ops.rs:291-375](/home/user/rewire/rust/src/ops.rs)
- `approve` reads the inbox file's own frontmatter (including `scope`), forces `confidence = explicit` ("a human
  approved it"), applies it and deletes the inbox file; `reject` just deletes the file
  — [rust/src/ops.rs:419-464](/home/user/rewire/rust/src/ops.rs). CLI: `rewire review [list|approve|reject] [--force] ids`
  — [rust/src/cli.rs:129-137](/home/user/rewire/rust/src/cli.rs)
- Agent/CLI proposals: `remember` writes an inbox item (`action NEW`, importance 3, `reason "remember()"`), refuses
  claim text > 600 chars — [rust/src/ops.rs:233-261](/home/user/rewire/rust/src/ops.rs); default scope = project
  of the cwd, else global — [rust/src/ops.rs:15-19](/home/user/rewire/rust/src/ops.rs)
- Project mapping: `[projects.<slug>] match = [...]` substrings tested against git remote URL, git top-level and
  absolute cwd; without a match, slug = git top-level directory name; outside a git repo `detect` returns None
  — [rust/src/project.rs:145-168](/home/user/rewire/rust/src/project.rs); config parsing of `[projects]`
  [rust/src/config.rs:284-294](/home/user/rewire/rust/src/config.rs)
- Consolidation alone adds a fallback: if `detect` returns None, the project is the **basename of the cwd**
  — [rust/src/consolidate.rs:556-563](/home/user/rewire/rust/src/consolidate.rs)
- The live `brain.toml` has no `[projects]` entries (only the commented example)
  — [/home/user/brain/brain.toml:33-36](/home/user/brain/brain.toml)
- Live result: the session record shows run 1 with `project: rewire`, runs 2-3 with `project: user`
  — [/home/user/brain/sessions/2026-10-28/claude-code-7314e139-87ff-5047-80c5-229658e75304.md:4, 41, 91](/home/user/brain/sessions/2026-10-28/claude-code-7314e139-87ff-5047-80c5-229658e75304.md);
  a hub `nodes/project/user-4tnc2n12.md` with title "user" and scope `project:user` exists
  — [/home/user/brain/nodes/project/user-4tnc2n12.md:1-16](/home/user/brain/nodes/project/user-4tnc2n12.md); and a
  decision about rewire's idle consolidation is scoped `project:user`, `about: [[user-4tnc2n12]]`
  — [/home/user/brain/nodes/decision/konsolidasi-rewire-saat-session-idle-10-menit-fg531z4v.md:8, 18](/home/user/brain/nodes/decision/konsolidasi-rewire-saat-session-idle-10-menit-fg531z4v.md)
- Live inbox: 5 items, all `NEW`: two near-duplicate global preferences about writing in Indonesian (from runs 1 and
  2), one global fact about the GitHub identity (provenance `assistant`), two global constraints (always reviewed)
  — [/home/user/brain/README.md:41-49](/home/user/brain/README.md) and the files in `/home/user/brain/inbox/`

### Inferences
- The "project:user" hub is an artifact: the session ran with cwd `/home/user` (not a git repo), so
  `project::detect` returned None and consolidation fell back to the cwd basename "user". Recall and the snapshot use
  `project::detect` without that fallback ([rust/src/ops.rs:86-90](/home/user/rewire/rust/src/ops.rs),
  [rust/src/snapshot.rs:207-212](/home/user/rewire/rust/src/snapshot.rs)), so from the same cwd they search only
  `global`: notes scoped `project:user` are invisible to default recall and snapshot unless `scope: "all"` or
  `project:user` is passed. The decision "Konsolidasi rewire saat session idle 10 menit" is about rewire but landed
  in a phantom project.
- There is no "promote to project" step in the flow: the project assignment happens once, at extraction, as a
  binary LLM choice combined with cwd detection. The inbox is a review gate, not a staging area for routing.
- A reviewer can only approve/reject; changing scope during review is possible only by hand-editing the inbox
  file's `scope` before `approve` (approve reads `item.meta`, [rust/src/ops.rs:446-449](/home/user/rewire/rust/src/ops.rs)) — not a documented workflow.
- The two duplicate Indonesian-language preferences show the AWAITING REVIEW de-dup instruction is advisory only
  (LLM-enforced), not a deterministic check.

### Gaps
- I did not confirm the actual cwd of the cloud session from the transcript; the "cwd = /home/user" explanation is
  inferred from the slug "user" plus the fallback code path.

## 3. Promotion to global/shared knowledge and richer per-project pages; README generation

### Takeaway
There is no mechanism to promote a project-scoped memory to global, to merge memories across projects, or to
aggregate memories into a richer project page. Project hubs are created with an empty body by design and only
gain content if a human edits them; the README is a generated index (titles + links), not a synthesis.

### Cited Findings
- `ensure_project_hub` searches `nodes/project/` for an active node with the scope, else creates one with title =
  slug and **empty body** (`brain.write_node(&meta, "", None)`) — [rust/src/ops.rs:265-285](/home/user/rewire/rust/src/ops.rs)
- Design history: "The LLM created its own `project` nodes → duplicate project hubs" → `project` removed from the
  extraction schema; "Empty project hubs ('Project rewire.') consumed snapshot budget" → "Hubs are created without a
  body and are not displayed while empty" — [docs/iterations.md:20-24](/home/user/rewire/docs/iterations.md)
- The original spec intended something richer: `project` = "**hub page** (summary ≤ 300 words, created by
  consolidation)"; `person`/`concept` also hub pages — [docs/design/mvp-spec.md:67-75](/home/user/rewire/docs/design/mvp-spec.md);
  the design review's P0 recommendation was "one claim/preference per node for fact/preference/decision, one page for
  project/concept/person (summarizing hubs)" — [docs/design/design-review.md:19, 42](/home/user/rewire/docs/design/design-review.md)
- The planned periodic "SLEEP" phase included "re-summarize hub nodes that have grown too long", "merge duplicates",
  incremental lint, and an `ingest` of external `sources/` that updates 5-15 related nodes (LLM Wiki pattern)
  — [docs/design/synaptic-memory.md:294-305](/home/user/rewire/docs/design/synaptic-memory.md); none of `sources/`,
  sleep, lint or ingest exist in the code (no such dirs in `DIRS`, [rust/src/brain.rs:142](/home/user/rewire/rust/src/brain.rs);
  no such CLI commands, [rust/src/cli.rs:39-160](/home/user/rewire/rust/src/cli.rs))
- Snapshot uses a project hub only if its body is non-empty: `type = 'project' AND scope = ? AND body != ''`
  — [rust/src/snapshot.rs:232-238](/home/user/rewire/rust/src/snapshot.rs); recall drops hubs unless reached via a
  link ("hubs reached only through links") — [rust/src/index.rs:1300-1302](/home/user/rewire/rust/src/index.rs); the
  per-prompt hook excludes `type == project` — [rust/src/integrations.rs:153-157](/home/user/rewire/rust/src/integrations.rs)
- Scope changes: auto-UPDATE across scopes is forbidden ([rust/src/consolidate.rs:357-366](/home/user/rewire/rust/src/consolidate.rs));
  the judge only sees candidates from `global` + current project ([rust/src/consolidate.rs:911-919](/home/user/rewire/rust/src/consolidate.rs)),
  so the same fact learned in two projects is never compared/merged; CLI commands are init, prime, recall, get,
  remember, consolidate, review, import, rebuild, doctor, semantic, upgrade, undo, sessions, forget, remote, sync,
  mcp, hook, install — no promote/move/rescope/merge command — [rust/src/cli.rs:39-160](/home/user/rewire/rust/src/cli.rs)
- README generator `write_home`: deterministic; sections "Rules (always applied)" (constraints), "Preferences",
  "Projects" (per hub: link + count + up to 10 member titles whose `scope == hub.scope`), "Recently learned"
  (10 newest), "General knowledge" (global fact/decision/concept/person, up to 20), "Recent sessions" (10), "Waiting
  for review" (20), "How this brain works" — [rust/src/ops.rs:945-1066](/home/user/rewire/rust/src/ops.rs)
- The rendered node page shows only outgoing relations in `## Related`; nothing writes backlinks/members into the
  hub file — [rust/src/nodefmt.rs:215-250](/home/user/rewire/rust/src/nodefmt.rs)
- Live hub pages are frontmatter only, body empty: `nodes/project/rewire-ntgx8mpe.md` lines 1-16 end with `---`
  and an empty line 17 — [/home/user/brain/nodes/project/rewire-ntgx8mpe.md](/home/user/brain/nodes/project/rewire-ntgx8mpe.md);
  same for `user-4tnc2n12.md` — [/home/user/brain/nodes/project/user-4tnc2n12.md](/home/user/brain/nodes/project/user-4tnc2n12.md)
- Live README "Projects": `rewire — 1 memories` (one fact) and `user — 1 memories` (one decision); career and
  learning notes appear under "General knowledge" — [/home/user/brain/README.md:14-35](/home/user/brain/README.md)
- Research notes adopting "Capture → promote" (Ori-Mnemos `ori add` → `ori promote`) equate promotion with the
  existing inbox — [docs/design/design-review.md:64](/home/user/rewire/docs/design/design-review.md); "Capture ≠
  promote ... promotion to core with a gate" — [docs/research/memory-approaches.md:96](/home/user/rewire/docs/research/memory-approaches.md)

### Inferences
- "Promotion" in rewire today means only inbox → `nodes/` (review approval); there is no second promotion tier
  (project → global/area knowledge). A project-scoped note becomes global only by hand-editing `scope` in the node
  file (the index re-reads it), or by `remember --scope global` + `forget` of the old one.
- The intended "project page" (≤300-word summary maintained by consolidation) was dropped because the LLM made
  duplicate hubs; the fix removed the content instead of making hub maintenance deterministic. This is the direct
  cause of the empty project bodies the user sees.
- A hand-written hub body would be used (snapshot "Current context"), so a richer, generated-or-curated project page
  is compatible with the existing code paths, provided it stays `type: project` in `nodes/project/`.

### Gaps
- No design doc found that specifies how a hub body should be generated now, or a plan for cross-project merging.

## 4. Size/budget limits and coding vs general profile: how they constrain richness

### Takeaway
Every write path is tuned for atomic, short claims: ≤2 sentences, ≤600 chars per claim body, titles ≤160 chars,
snapshot lines ≤380 chars, recall snippets 160 chars. The live brain uses the `coding` profile, which tells the LLM
to return "few, high-quality claims; zero is fine", skip task progress and narratives, and keep only the user's
turns in the stored transcript, so substantive assistant output (e.g. the learning roadmap) is not stored anywhere.

### Cited Findings
- Budgets (chars, ~4 chars/token): profile 600, constraints 1500, preferences 800, working 2500, skills 400 (defaults)
  with synced minimums 200/400/200/200/400 — [rust/src/config.rs:15-19, 99, 226-240](/home/user/rewire/rust/src/config.rs);
  live values [/home/user/brain/brain.toml:12-16](/home/user/brain/brain.toml)
- The original budget goal: total snapshot ≤ ~4,800 chars (~1,200 tokens), below Claude Code's 10,000-char
  `additionalContext` limit — [docs/design/mvp-spec.md:50-59](/home/user/rewire/docs/design/mvp-spec.md)
- Snapshot sections: "About the user" (global person + global facts with importance ≥4), "Rules the user set",
  "Preferences", "Skills", "Current context" (non-empty project hub + fact/decision/person/concept by importance then
  recency); each line truncated to 380 chars — [rust/src/snapshot.rs:56-79, 207-292](/home/user/rewire/rust/src/snapshot.rs)
- Claim body cap 600 chars on write — [rust/src/schema.rs:92-94](/home/user/rewire/rust/src/schema.rs); remember cap
  — [rust/src/ops.rs:241-243](/home/user/rewire/rust/src/ops.rs); MCP `remember` "max 2 sentences"
  — [rust/src/mcp.rs:53-66](/home/user/rewire/rust/src/mcp.rs); extraction "max 2 sentences"
  — [rust/src/consolidate.rs:74](/home/user/rewire/rust/src/consolidate.rs)
- Rationale for atomicity: Dense X Retrieval (propositions beat paragraphs at 100-200 word budgets), clean
  supersede/contradict, meaningful activation — [docs/design/mvp-spec.md:79-81](/home/user/rewire/docs/design/mvp-spec.md)
- Recall hit snippet = first 160 chars of body — [rust/src/index.rs:1313](/home/user/rewire/rust/src/index.rs); MCP
  recall says "Returns short summaries with ids; call `get` for full text" — [rust/src/mcp.rs:24-25](/home/user/rewire/rust/src/mcp.rs)
- `coding` profile prompt: extract ONLY durable knowledge; "Skip: chit-chat, task progress, things derivable from the
  code/repo ... one-off narratives"; "Return few, high-quality claims; zero is fine"
  — [rust/src/consolidate.rs:45-57](/home/user/rewire/rust/src/consolidate.rs)
- `general` profile prompt: "Extract everything a friend would want to remember later about each person"; "Prefer
  many small, specific claims over few general ones" — [rust/src/consolidate.rs:59-68](/home/user/rewire/rust/src/consolidate.rs);
  skills only in non-general profiles — [rust/src/consolidate.rs:115-122](/home/user/rewire/rust/src/consolidate.rs)
- Transcripts kept: `auto` = user turns for coding, everything for general
  — [rust/src/consolidate.rs:991-995](/home/user/rewire/rust/src/consolidate.rs); [/home/user/brain/brain.toml:18-27](/home/user/brain/brain.toml)
  sets `profile = "coding"`, `transcripts = "auto"`
- Extraction sees assistant replies condensed above `assistant_chars` (default 1,500: opening + end)
  — [rust/src/config.rs:31-33](/home/user/rewire/rust/src/config.rs); [docs/design/format.md:131-134](/home/user/rewire/docs/design/format.md)
- Live session transcript keeps only user lines, e.g. "[229 user] bisakah buatkan agar menjadi sebuah roadmap? ..."
  with no assistant turns — [/home/user/brain/sessions/2026-10-28/claude-code-7314e139-87ff-5047-80c5-229658e75304.md:71-83](/home/user/brain/sessions/2026-10-28/claude-code-7314e139-87ff-5047-80c5-229658e75304.md)
- The roadmap the assistant built is stored only as a 2-sentence decision ("tiga jalur karier ... 12–14 bulan ...
  16–18 bulan ... 22–24 bulan ... 15 jam per minggu") — [/home/user/brain/nodes/decision/roadmap-belajar-ai-engineering-dan-chip-design-3-r5t189wq.md:20](/home/user/brain/nodes/decision/roadmap-belajar-ai-engineering-dan-chip-design-3-r5t189wq.md);
  the learning-source research is one sentence "Pengguna lebih suka sumber belajar yang gratis dan sudah menjadi
  standar industri." — [/home/user/brain/nodes/preference/preferensi-sumber-belajar-gratis-dan-standar-ind-yafrehew.md:20](/home/user/brain/nodes/preference/preferensi-sumber-belajar-gratis-dan-standar-ind-yafrehew.md)

### Inferences
- Concern (a) "one md gives too little information" is the designed behavior, not a bug: claims are propositions;
  context is meant to come from links, the session file and recall excerpts. With `coding` + `auto`, the assistant's
  content (resources list, roadmap phases, key insights) is in neither nodes nor sessions, so it is unrecoverable
  from the brain.
- The user's brain mixes a coding project (rewire) with personal second-brain content (career, learning), which the
  `coding` profile is explicitly told to discard or minimize; `general` would keep more, but is tuned for
  people/events (LoCoMo-style) and disables skills, so neither profile fits a mixed "projects + personal knowledge"
  brain well.
- Budgets limit only what is injected at session start, not what can be stored; a longer hub/page body is allowed
  by the schema but would be shown truncated (380 chars/line) in the snapshot.

### Gaps
- No measurement in the repo of how much information is lost by the coding profile on real coding sessions
  (README notes "no coding-agent memory benchmark has been run" — [README.md:10-13](/home/user/rewire/README.md)).

## 5. Recall/MCP dependence on files, and what a PARA-like reorganization would break

### Takeaway
Recall, get and remember work off a derived SQLite index rebuilt from files, keyed by frontmatter (`id`, `type`,
`scope`, `importance`, `valid_to`/`retracted_at`) and by the `-<id8>.md` filename suffix. Folder names under
`nodes/` are not semantically used for reading, but the code assumes exactly one directory level under `nodes/`,
writes new notes to `nodes/<type>/`, finds hubs only in `nodes/project/`, and keys review-trust and sync conflict
handling on paths starting with `nodes/`. A PARA layout outside `nodes/` or nested deeper would silently drop notes
from the index.

### Cited Findings
- MCP exposes three tools: `recall` (query, k, episodes, scope `global|project:<slug>|all`, refers_to, also), `get`
  (ids or `skill:<name>`), `remember` (text, type, scope) — [rust/src/mcp.rs:21-68](/home/user/rewire/rust/src/mcp.rs)
- Default recall scope = `global` + project of the server's cwd (`project::detect`) — [rust/src/ops.rs:86-90](/home/user/rewire/rust/src/ops.rs);
  `scopes_for` = `["global", "project:<p>"]` — [rust/src/project.rs:170-176](/home/user/rewire/rust/src/project.rs)
- Index tables: `nodes(id, type, title, body, scope, importance, active, recorded_at, path, meta, short)`, FTS5 over
  title/body/keywords, `edges`, `episodes(path, date, text, first, last, scope)` — [rust/src/index.rs:48-66](/home/user/rewire/rust/src/index.rs)
- Index scan reads only immediate subdirectories of `nodes/` and `*.md` directly inside them (no recursion); `active`
  = not `retracted_at` and not `valid_to`; scope from frontmatter, default `global` — [rust/src/index.rs:764-875](/home/user/rewire/rust/src/index.rs).
  `Brain::node_files` has the same one-level walk — [rust/src/brain.rs:561-582](/home/user/rewire/rust/src/brain.rs)
- Lookup by short id relies on the filename ending `-<id8>.md` (`locate`, `iter_node_files_with_suffix`)
  — [rust/src/brain.rs:584-612](/home/user/rewire/rust/src/brain.rs); `resolve_id` — [rust/src/ops.rs:34-56](/home/user/rewire/rust/src/ops.rs);
  `nodefmt::short` regex `-([0-9a-z]{8})$` — [rust/src/nodefmt.rs:34-62](/home/user/rewire/rust/src/nodefmt.rs)
- New nodes are always written to `nodes/<type>/<slug>-<id8>.md`; updates keep the existing path
  — [rust/src/brain.rs:637-656](/home/user/rewire/rust/src/brain.rs); `upgrade` rewrites every node in place
  (recomputing `## Related` relative links) — [rust/src/cli.rs:944-956](/home/user/rewire/rust/src/cli.rs)
- Hub lookup reads only `nodes/project/` — [rust/src/ops.rs:265-277](/home/user/rewire/rust/src/ops.rs)
- Review/trust keys are paths: "nodes/constraint/x.md" ... "by path, whatever folder: what decides is the type the
  file declares" — [rust/src/brain.rs:250-264](/home/user/rewire/rust/src/brain.rs)
- Sync conflict resolution (newest wins, loser to inbox) applies to paths starting `nodes/`; pulled-path checks use
  `:(icase)nodes`, `skills`, `inbox` — [rust/src/sync.rs:1-6, 143-160, 348-406](/home/user/rewire/rust/src/sync.rs)
- `undo` finds touched notes via `git log ... -- nodes` — [rust/src/ops.rs:493](/home/user/rewire/rust/src/ops.rs)
- Sessions are indexed from `sessions/<day>/*.md` (two levels) and get scope from the `- project:` line
  — [rust/src/index.rs:136-146, 877-890](/home/user/rewire/rust/src/index.rs); README lists them the same way
  — [rust/src/ops.rs:1018-1035](/home/user/rewire/rust/src/ops.rs)
- Relations on disk are `[[<file stem>]]` wikilinks plus relative links; the format was chosen so Obsidian graph view
  and backlinks work — [docs/iterations.md:44](/home/user/rewire/docs/iterations.md); README promises opening `~/brain`
  as an Obsidian vault — [README.md:149-153](/home/user/rewire/README.md); `.gitignore` already ignores
  `.obsidian/workspace*.json` — [/home/user/brain/.gitignore:4](/home/user/brain/.gitignore)

### Inferences
- Safe without code changes: renaming/moving a node file to another first-level folder under `nodes/` (e.g.
  `nodes/projects/x-ab12cd34.md`) keeps it indexed, recallable and `get`-able, because type/scope come from
  frontmatter and lookup from the id suffix. Wikilinks keep working in Obsidian (stem-based); `## Related` relative
  links break until `rewire upgrade`. But new notes would still go to `nodes/<type>/`, and a moved hub would cause a
  duplicate hub to be created on the next project-scoped write.
- Would break: (1) nesting deeper (`nodes/projects/rewire/decision/...`) — not indexed at all; (2) PARA folders at
  the brain root (`projects/`, `areas/`, `resources/`, `archive/`) — not indexed, not covered by sync conflict
  handling, review trust, or undo; (3) dropping the `-<id8>` filename suffix — short-id lookup and link resolution
  fail; (4) moving session files out of `sessions/<day>/`.
- A PARA-like view is therefore most compatible as (a) frontmatter (e.g. an extra key, which survives rewrites but
  is ignored by `get` and the index columns) plus (b) generated index/hub pages, rather than physically moving notes;
  "Archive" already exists semantically as `status: superseded/retracted` (kept files).

### Gaps
- Did not test a moved file end-to-end (read-only constraint); the effects above are read from the code paths.
- The untrusted/review state after a path rename on another device was not traced in detail (path-keyed state in
  `.state/untrusted.json` suggests a move looks like a new file to review).

## 6. Concrete thin-note examples from /home/user/brain

### Takeaway
All 7 active nodes are one or two sentences; both project hubs are empty; one project fact is already stale; and the
project grouping contains a phantom "user" project.

### Cited Findings
- `nodes/project/rewire-ntgx8mpe.md`: 15 frontmatter lines, empty body, no Related section
  — [/home/user/brain/nodes/project/rewire-ntgx8mpe.md:1-17](/home/user/brain/nodes/project/rewire-ntgx8mpe.md)
- `nodes/project/user-4tnc2n12.md`: same, title "user" — [/home/user/brain/nodes/project/user-4tnc2n12.md:1-17](/home/user/brain/nodes/project/user-4tnc2n12.md)
- Stale project fact: "As of 2026-10-28 it contained only LICENSE and a README.md with no brain.toml, so it had not
  yet been initialized as a brain." (the brain was initialized the same day: commits `b6996ca init brain`,
  `1477793 enable sync`) — [/home/user/brain/nodes/fact/remote-brain-rewire-newbeetrytocode-brain-vt34xd89.md:20](/home/user/brain/nodes/fact/remote-brain-rewire-newbeetrytocode-brain-vt34xd89.md); `git log` of `/home/user/brain`
- Mixed languages: the rewire fact body is English while later claims are Indonesian
  — [/home/user/brain/nodes/fact/remote-brain-rewire-newbeetrytocode-brain-vt34xd89.md:20](/home/user/brain/nodes/fact/remote-brain-rewire-newbeetrytocode-brain-vt34xd89.md) vs
  [/home/user/brain/nodes/fact/tujuan-karier-bekerja-di-perusahaan-ai-gmwxfsqt.md:19](/home/user/brain/nodes/fact/tujuan-karier-bekerja-di-perusahaan-ai-gmwxfsqt.md)
- Career goal is one sentence with 13 keywords and no links — [/home/user/brain/nodes/fact/tujuan-karier-bekerja-di-perusahaan-ai-gmwxfsqt.md:17-19](/home/user/brain/nodes/fact/tujuan-karier-bekerja-di-perusahaan-ai-gmwxfsqt.md)
- Session summaries are the richest text in the brain (3 runs, each with Summary, Applied, Queued, Questions,
  user-only Transcript) — [/home/user/brain/sessions/2026-10-28/claude-code-7314e139-87ff-5047-80c5-229658e75304.md:1-129](/home/user/brain/sessions/2026-10-28/claude-code-7314e139-87ff-5047-80c5-229658e75304.md)

### Inferences
- In this brain the knowledge about a "project" lives mostly in the session summaries, not in the project node;
  the project node is a pure anchor for `about` links.
- Without a project/area page, "learning AI engineering" (a personal long-running project) has no hub at all: its
  three notes are global and only weakly linked via `related_to`.

### Gaps
- Only one session has been consolidated; the evidence on how the structure scales to many projects is from code,
  not from a large live brain.

## 7. Design docs on future structure, wiki pages, Obsidian, promotion

### Takeaway
The docs already contain the ingredients the user is asking for (hub pages, LLM-Wiki-style synthesis, `sources/`
ingest, periodic re-summarization/lint, Obsidian compatibility), but only Obsidian-compatible linking and the
generated README were implemented; hub pages and the "sleep"/ingest phases were deferred or dropped.

### Cited Findings
- Design review scored "Ontology & node granularity" 5/10 because "the size of a single node has not been decided
  (atomic facts like Mem0, or pages like LLM Wiki?)" — [docs/design/design-review.md:19](/home/user/rewire/docs/design/design-review.md);
  re-score 9 with "atomic claims vs hub pages" — [docs/design/design-review.md:135](/home/user/rewire/docs/design/design-review.md)
- Deferred from MVP: "embeddings, Hebbian/associative edges, automatic sleep, `sources/` ingest, the PreToolUse push
  hook, multi-device, Obsidian" — [docs/design/design-review.md:116](/home/user/rewire/docs/design/design-review.md)
  (multi-device and Obsidian were later implemented: [docs/iterations.md:30-49](/home/user/rewire/docs/iterations.md))
- Karpathy LLM Wiki research: the LLM "incrementally builds and maintains an interlinked markdown wiki", with
  `index.md` + `log.md`, ingest/query/lint — [docs/research/karpathy-llm-wiki.md:8-22](/home/user/rewire/docs/research/karpathy-llm-wiki.md);
  mapping says Wiki pages = `nodes/*.md` and the key difference is that Karpathy's index lists all pages while
  rewire's core is budget-selected; gap: rewire captures only agent sessions, not external sources
  — [docs/research/karpathy-llm-wiki.md:38-52](/home/user/rewire/docs/research/karpathy-llm-wiki.md)
- Lessons: LLM lint is quadratic so lint must be incremental; early extraction errors "permanently poison the store"
  — [docs/research/karpathy-llm-wiki.md:24-36](/home/user/rewire/docs/research/karpathy-llm-wiki.md)
- Synaptic-memory design: `sources/` immutable layer, `snapshot/CORE.md`, body `[[slug]]` wikilinks treated as
  low-prior `related_to` edges — [docs/design/synaptic-memory.md:186-229](/home/user/rewire/docs/design/synaptic-memory.md);
  "Markdown + wikilinks only (Obsidian style) ... insufficient for the synapse concept" — [docs/design/synaptic-memory.md:261](/home/user/rewire/docs/design/synaptic-memory.md)
- Memory-approaches research: "Capture ≠ promote. Write quickly to an inbox/daily note → background consolidation
  ('dreaming') → promotion to core with a gate. A hard cap forces curation." — [docs/research/memory-approaches.md:96](/home/user/rewire/docs/research/memory-approaches.md);
  proposed write pipeline inbox/journal → consolidate → `memories/` + `MEMORY.md` — [docs/research/memory-approaches.md:152](/home/user/rewire/docs/research/memory-approaches.md)
- Architecture roadmap lists only measurement work as open — [docs/design/architecture.md:85-89](/home/user/rewire/docs/design/architecture.md)

### Inferences
- A recommendation to add generated project/area pages (LLM-Wiki style) or a second "promotion" tier would revive
  documented-but-deferred design (mvp-spec hub pages; synaptic-memory sleep: re-summarize hubs, merge duplicates)
  rather than contradict the design.
- The documented reason against LLM-made hubs (duplicates) suggests hub creation stays deterministic
  (`ensure_project_hub`) while only the hub *body* is LLM-maintained or generated from member notes.

### Gaps
- No doc found discussing PARA, areas/resources, or a multi-level scope (e.g. `area:<slug>`); the scope validator
  only accepts `global` and `project:*` ([rust/src/schema.rs:79-82](/home/user/rewire/rust/src/schema.rs)), so any
  new scope kind needs a schema change.
