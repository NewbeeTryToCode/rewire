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

## Applied
- <ACTION> [<title>](<relative path>)      (or "-")

## Queued for review
- <ACTION> <title> (inbox <id>)            (or "-")

## Rejected                                (only if any)
- <title> (<why>)

## Transcript                              (only if transcripts are kept)

[<n> <role>] <text on one line>
```

A file may hold several consolidation runs of the same session, separated by `\n\n---\n\n`. Transcript
lines are secret-redacted; assistant turns longer than 1,500 characters are cut and end with ` …[condensed]`.
For search, transcript lines are cut into excerpts of 6 lines with a stride of 4.

## Inbox item

Frontmatter (same JSON rules) + the claim text as body. Keys: `inbox_id`, `action` (`NEW` · `UPDATE` ·
`SUPERSEDE` · `CONTRADICT`), `target` (node id, for non-NEW), `type`, `title`, `scope`, `importance`,
`confidence`, `provenance`, `keywords`, `edges`, `reason`, `flags` (`secret` · `injection`), `source`,
`created`.

## brain.toml

Sections `[activation]` (`d`, `recall_beta`), `[budgets]` (`constraints`, `preferences`, `working`; characters),
`[consolidation]` (`profile`, `transcripts`, `provider`, `command`, `model`, `effort`, `max_chunk_chars`),
`[sync]` (`auto`), `[projects.<slug>]` (`match = [...]`). Unknown keys are ignored.

## Device-local state (`.state/`, never synced)

JSON files (indent 1, keys sorted): `device.json` (`name`), `scan.json` (`since`, `since_ts`),
`pending.json` (transcript path → harness), `watermarks.json` (path → last consolidated line),
`processed.json`, `failed.json`; plus `brain.lock` and `consolidate.log`.
