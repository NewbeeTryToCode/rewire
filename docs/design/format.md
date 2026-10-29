# On-disk format (frozen contract)

Every rewire implementation reads and writes exactly this. The SQLite index (`.index/`) is **not** part of
the contract: it is derived and rebuilt from the files below.

```
<brain>/
  brain.toml            shared configuration (synced)
  README.md             generated home page (deterministic)
  .gitattributes        events/**/*.jsonl merge=union
  .gitignore            .index/  .state/  .obsidian/workspace*.json
  nodes/<type>/<slug>-<id8>.md     L2 claims and hubs
  events/<device>/<YYYY-MM>.jsonl  usage log, append-only
  sessions/<YYYY-MM-DD>/<harness>-<session-slug>.md   L1 episodes
  skills/<name>/SKILL.md           procedures (Agent Skills), added through review
  inbox/<ulid>.md       proposals waiting for review
  .index/brain.db       derived (not synced)
  .state/*.json         device-local state (not synced)
```

## Identifiers

- **Node id**: 26-char ULID, Crockford base32, upper case (48-bit ms timestamp + 80 random bits).
- **Short id**: last 8 chars of the id, lower case. File names end with it: `<slug>-<id8>.md`.
- **Slug**: `lower(text)`, every run of characters outside `[a-z0-9]` → `-`, trimmed of `-`, cut to 48
  chars (session file names: 40), trailing `-` removed; empty → `node`.

## Frontmatter

```
---
key: <JSON value>
...
---
<body>
```

- One key per line; every value is JSON (also valid YAML), written like Python
  `json.dumps(value, ensure_ascii=False)`: separators `", "` and `": "`, non-ASCII kept, standard escapes.
- Body follows the closing `---` line; the file ends with exactly one `\n`.
- Readers also accept hand-written plain YAML scalars (a value that is not valid JSON is kept as a string).

## Node file

Key order: `id, type, title, status, aliases, tags, scope, importance, confidence, provenance, valid_from,
valid_to, recorded_at, retracted_at, source, keywords`, then relation keys in the order edges first use
them, then any other keys in their original order.

| Key | Value |
|---|---|
| `type` | `constraint` · `preference` · `fact` · `decision` · `project` · `person` · `concept` |
| `title` | 1–160 chars |
| `status` | derived: `superseded` if `valid_to`, else `retracted` if `retracted_at`, else `active` |
| `aliases` | derived: `[title]` |
| `tags` | derived: `["rewire/<type>"]` |
| `scope` | `global` or `project:<slug>` |
| `importance` | int 1–5 |
| `confidence` | `explicit` · `inferred` |
| `provenance` | `user` · `assistant` · `tool` |
| `valid_from` / `valid_to` | `YYYY-MM-DD` or `null` (when the claim held in the world) |
| `recorded_at` / `retracted_at` | `YYYY-MM-DDTHH:MM:SSZ` or `null` (when it was written / withdrawn) |
| `source` | `"<harness> · <full session id> · turn <n>"` (any part may be absent) |
| `keywords` | list of lowercase search terms (optional) |
| relation keys | `about`, `part_of`, `depends_on`, `related_to`, `supersedes`, `contradicts`: list of `"[[<target file stem>]]"` |

Derived keys (`status`, `aliases`, `tags`) are regenerated on every write and ignored on read. A claim body
(constraint, preference, fact, decision) is at most 600 characters.

After the body, relations are mirrored for readers:

```
<body>

## Related
- <rel>: [<target title>](<relative path to target file>)
- <rel>: <stem> (missing)          ← target file not found
```

The `## Related` section is generated and never part of the body. Legacy v0.1 nodes stored
`edges: [{"rel", "to"}]`; readers still accept it.

### Project pages

A project hub's body ends with a generated block, rewritten (only when it changes) whenever the home page is:

```
<what a person wrote, kept as is; may be empty>

<!-- rewire:generated (rewritten from this project's notes and sessions: edit those, not this block) -->

## Rules | Decisions | Facts | Preferences | Concepts | People      (active notes of the project, newest first)
- [<title>](<relative path>) · <valid_from or recorded_at date>

## Recent sessions                                                   (the latest 5 session records of the project)
- [<date> · <file stem>](<relative path>): <summary, cut at 160 characters>

<details><summary>No longer valid (<n>)</summary>                    (superseded notes)
- ~~[<title>](<relative path>)~~ · until <valid_to>
</details>

<!-- /rewire:generated -->
```

Members are the notes with the hub's scope, and notes of another scope with an `about` link to the hub. The index
leaves the block out: search, recall snippets and the session-start snapshot read only what a person wrote.

Where the project stands lives in that text as a `## Now (as of <date>)` section: 2-6 lines (focus, next steps,
blockers, open questions). Consolidation shows the model the current section (`PROJECT NOW`) and, when a session
changes it, queues the new text as an inbox item with `kind: "hub_section"`, `section: "Now"` and `target` = the
hub id; at most one waits per project (a newer one replaces it). Approving it sets the section in place, or adds
it after the rest of the text; nothing else on the page changes.

## Events

One JSON object per line, keys sorted, `ensure_ascii=False`, Python default separators (`", "`, `": "`).

| Field | Meaning |
|---|---|
| `ev` | `use` (a note was used; weight 1.0 from consolidation, 0.5 from `get`) or `confirm` (consolidation saw the same claim again) |
| `node` | full node id |
| `w` | weight |
| `session`, `harness` | where it happened |
| `t` | `YYYY-MM-DDTHH:MM:SSZ` |

Only `use` and `confirm` count toward activation, at most one per (node, session) — the highest weight.
Node creation (`recorded_at`) counts as the first use. Activation: `B = ln Σ wᵢ · max(ageᵢ_h, 0.5)^(-d)`,
`d = 0.7`. Files are appended only; git merges them with `merge=union`.

## Session file (L1)

```
# <harness> session <session id>

- date: <YYYY-MM-DD>              ← when the session happened
- project: <slug or ->
- consolidated: <timestamp>
- memory shown at consolidation: <n>
- memory used: <comma-separated short ids or ->

## Summary

<summary paragraphs or ->

## Outputs                                 (only if the session published something)

- [<n>] <artifact | pull request>: [<title>](<url>)

## Applied
- <ACTION> [<title>](<relative path>)      (or "-")

## Queued for review
- <ACTION> <title> (inbox <id>)            (or "-")

## Rejected                                (only if any)
- <title> (<why>)

## Questions                               (only if transcripts are kept and the LLM wrote any)

- [<n>] <question a later search might ask, answered by turn n>

## Transcript                              (only if transcripts are kept)

[<n> <role>] <text on one line>
```

Outputs are read from the tool calls themselves (an Artifact publish, a pull request opened through the
GitHub tools or `gh pr create`), so their links are kept whatever `transcripts` keeps; turn `n` is the tool
result that reported the link. Each run's outputs are indexed as one more searchable row, like its summary.

A file may hold several consolidation runs of the same session, separated by `\n\n---\n\n`; each later run
is given the earlier runs' summaries. Transcript lines are secret-redacted and kept whole. The extraction
prompt condenses agent replies longer than `assistant_chars` (default 1,500) to their opening and their end,
joined by ` …[condensed]… `.
For search, transcript lines are cut into excerpts of 6 lines with a stride of 4, and each run's summary
is one more searchable excerpt (`Summary: <text>`, whitespace collapsed, at most 600 characters).
Questions (secret-redacted, one line, at most 200 characters) are written by the extraction call. Each
one is added to the search text of every excerpt whose turn range holds its turn; the excerpt shown in
recall is unchanged. On LoCoMo this raised evidence recall at the same context size
(experiments/eval/results/2026-10-14-locomo-nemotron-s20-analysis.md §11–12).

## Inbox item

Frontmatter (same JSON rules) + the claim text as body. Keys: `inbox_id`, `action` (`NEW` · `UPDATE` ·
`SUPERSEDE` · `CONTRADICT`), `target` (node id, for non-NEW), `type`, `title`, `scope`, `importance`,
`confidence`, `provenance`, `keywords`, `edges`, `reason`, `flags` (`secret` · `injection`), `source`,
`created`, and `target_hash` when the change targets an existing note: the first 16 hex characters of
`sha256(title + "\n" + body)` of that note when the item was written (body as read: stripped, one
trailing newline). Approving refuses while the note's current hash differs (`--force` overrides).

A skill proposal has `kind: "skill"`, `action: "SKILL"`, `type: "skill"`, `title: "skill <name>"`,
`scope: "global"`, `name` (kebab-case, at most 40 characters), `description`, `skill_action`
(`create` · `append`), `confidence`, `provenance`, `reason`, `flags`, `source`, `created`; the body is
the steps. Skills are never applied without review.

## Skill file

`skills/<name>/SKILL.md`: frontmatter (same JSON rules, which are also valid Agent Skills YAML) with
`name`, `description`, `updated` (timestamp), `sources` (list of `harness · session · turn N`), then the
steps. An approved `append` adds `\n\n## Update <YYYY-MM-DD>\n\n<steps>` to the body. Reading one
(`get skill:<name>`) logs a `use` event with `node: "skill:<name>"`.

## brain.toml

Sections `[activation]` (`d`, `recall_beta`), `[budgets]` (`profile`, `constraints`, `preferences`,
`skills`, `working`; characters),
`[consolidation]` (`profile`, `transcripts`, `provider`, `command`, `model`, `effort`, `max_chunk_chars`,
`assistant_chars`),
`[sync]` (`auto`), `[projects.<slug>]` (`match = [...]`). Unknown keys are ignored.

## Device-local state (`.state/`, never synced)

JSON files (indent 1, keys sorted): `device.json` (`name`), `scan.json` (`since`, `since_ts`),
`pending.json` (transcript path → harness), `watermarks.json` (path → last consolidated line),
`processed.json`, `failed.json`, `prompt_shown.json` (session id → `{t, ids}`: short ids the session has
seen, for the per-prompt hook; latest 50 sessions); plus `brain.lock`, `watch.lock` (held by the running
idle watcher) and `consolidate.log`.

## Injected text

Everything rewire puts in an agent's context (snapshot, per-prompt block, MCP `recall`/`get`, CLI lists)
is wrapped in `<rewire-memory>` … `</rewire-memory>`; the tags inside note content are escaped as
`&lt;rewire-memory` / `&lt;/rewire-memory`. Transcript parsing drops fenced blocks before consolidation.
Notes are re-scanned when shown; one holding a secret or an injection pattern is replaced by
`[BLOCKED: <why>; check this note with \`rewire doctor\`]`.
