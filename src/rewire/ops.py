"""High-level operations shared by the CLI, MCP server and consolidation."""

import glob
import hashlib
import os
import re

from . import project as project_mod
from . import schema, security
from .brain import Brain, Node
from .index import Index
from .util import now_iso


class OpError(Exception):
    pass


def resolve_id(brain: Brain, ref: str) -> Node | None:
    ref = ref.strip().strip("[]")
    if len(ref) == 26:
        return brain.get_node(ref.upper())
    ref = ref.lower()
    if len(ref) == 8:  # file names end with the last 8 id chars: no full scan needed
        hits = []
        for path in glob.glob(brain.path("nodes", "*", f"*-{ref}.md")):
            node = brain.read_node(path)
            if node and node.id.lower().endswith(ref):
                hits.append(node)
        return hits[0] if len(hits) == 1 else None
    matches = [n for n in brain.iter_nodes() if n.id.lower().endswith(ref)]
    return matches[0] if len(matches) == 1 else None


def recall(brain: Brain, index: Index, query: str, k: int = 8, cwd: str | None = None,
           scope: str | None = None, beta: float | None = None, episodes: int = 0,
           refers_to: dict | None = None) -> list[dict]:
    """`refers_to` {from, to}: the time the question is about, resolved by the calling model
    ("last week", "kemarin"); it boosts notes and excerpts dated inside it (a boost, not a filter)."""
    window = time_range(refers_to)
    if scope:
        scopes = [scope] if scope != "all" else None
    else:
        scopes = project_mod.scopes_for(project_mod.detect(cwd, brain.config.projects))
    hits = index.search(query, scopes=scopes, k=k, beta=beta, window=window)
    for hit in hits:  # re-scanned when shown: files can be edited by hand or arrive by sync
        why = security.blocked(hit["title"] + " " + hit["snippet"])
        if why:
            hit["title"], hit["snippet"] = f"[BLOCKED: {why}; check this note with `rewire doctor`]", ""
    if episodes:
        # raw excerpts after the distilled facts; the facts found pull in the excerpts they came from
        hits += index.search_episodes(query, k=episodes, seeds=hits, window=window)
    return hits


def time_range(refers_to: dict | None) -> tuple[str, str] | None:
    if not refers_to:
        return None
    if not isinstance(refers_to, dict):
        raise OpError("refers_to must be an object {from, to}")
    start, end = str(refers_to.get("from") or "")[:10], str(refers_to.get("to") or "")[:10]
    start, end = start or end, end or start
    if not re.fullmatch(r"\d{4}-\d{2}-\d{2}", start) or not re.fullmatch(r"\d{4}-\d{2}-\d{2}", end):
        raise OpError("refers_to needs ISO dates: {\"from\": \"YYYY-MM-DD\", \"to\": \"YYYY-MM-DD\"}")
    return (start, end) if start <= end else (end, start)


def get(brain: Brain, refs: list[str], session: str, harness: str = "mcp") -> list[dict]:
    out, events = [], []
    for ref in refs[:10]:
        node = resolve_id(brain, ref)
        if node is None:
            out.append({"ref": ref, "error": "not found"})
            continue
        item = {"id": node.id, **{k: v for k, v in node.meta.items() if k != "id"}, "body": node.body}
        why = security.blocked(node.meta["title"] + " " + node.body)
        if why:
            item["title"], item["body"] = f"[BLOCKED: {why}; check this note with `rewire doctor`]", ""
        out.append(item)
        events.append({"ev": "use", "node": node.id, "w": 0.5, "session": session, "harness": harness})
    brain.append_events(events)
    return out


def remember(brain: Brain, text: str, type_: str = "fact", scope: str = "global",
             provenance: str = "assistant", source: dict | None = None) -> str:
    text = security.strip_private(text).strip()
    if not text:
        raise OpError("empty memory")
    if type_ not in schema.TYPES:
        raise OpError(f"unknown type {type_}; use one of {', '.join(schema.TYPES)}")
    if type_ in schema.CLAIM_TYPES and len(text) > 600:
        raise OpError("a memory must be one self-contained claim (max 600 characters)")
    problems = security.scan(text)
    if "secret" in problems:
        raise OpError("refused: text looks like it contains a secret")
    meta = {"action": "NEW", "type": type_, "title": text[:160], "scope": scope,
            "importance": 3, "confidence": "explicit" if provenance == "user" else "inferred",
            "provenance": provenance, "reason": "remember()", "flags": problems,
            "source": source or {}, "created": now_iso()}
    return brain.write_inbox(meta, text)


# ---- applying changes -----------------------------------------------------

def ensure_project_hub(brain: Brain, slug: str) -> Node:
    for path in glob.glob(brain.path("nodes", "project", "*.md")):  # hubs only, not every node
        n = brain.read_node(path)
        if n and n.meta.get("scope") == f"project:{slug}" and n.active:
            return n
    meta = brain.new_meta("project", slug, scope=f"project:{slug}", importance=3,
                          confidence="explicit", provenance="user")
    return brain.write_node(meta, "")


def apply_change(brain: Brain, change: dict, source: dict | None = None) -> Node:
    """Apply NEW / UPDATE / SUPERSEDE / CONTRADICT. Returns the written node."""
    action = change.get("action", "NEW")
    text = change["text"].strip()
    title = (change.get("title") or text)[:160]
    scope = change.get("scope") or "global"
    fields = {k: change[k] for k in ("importance", "confidence", "provenance", "keywords", "valid_from")
              if change.get(k)}
    edges = [{"rel": e["rel"], "to": e["to"]} for e in change.get("edges") or []
             if isinstance(e, dict) and e.get("rel") in schema.RELATIONS and e.get("to")]
    if scope.startswith("project:"):
        hub = ensure_project_hub(brain, scope.split(":", 1)[1])
        if change.get("type") != "project":
            edges.append({"rel": "about", "to": hub.id})

    if action == "UPDATE":
        target = resolve_id(brain, change["target"])
        if target is None:
            raise OpError(f"update target not found: {change['target']}")
        meta = dict(target.meta)
        meta.update(fields, title=title, recorded_at=now_iso())
        meta["source"] = source or meta.get("source")
        return brain.write_node(meta, text, path=target.path)

    meta = brain.new_meta(change["type"], title, scope=scope, edges=edges, source=source or {}, **fields)
    if action in ("SUPERSEDE", "CONTRADICT"):
        target = resolve_id(brain, change["target"])
        if target is None:
            raise OpError(f"target not found: {change['target']}")
        rel = "supersedes" if action == "SUPERSEDE" else "contradicts"
        meta["edges"].append({"rel": rel, "to": target.id})
        old = dict(target.meta)
        old["retracted_at"] = now_iso()
        if action == "SUPERSEDE":
            old["valid_to"] = now_iso()[:10]
        brain.write_node(old, target.body, path=target.path)
    return brain.write_node(meta, text)


def content_hash(node: Node) -> str:
    """Fingerprint of a note's content, pinned on inbox items that change an existing note."""
    return hashlib.sha256((node.meta["title"] + "\n" + node.body).encode()).hexdigest()[:16]


def pin_target(brain: Brain, item: dict) -> dict:
    """Record what the target looked like when the change was proposed (approve refuses if it moved)."""
    node = resolve_id(brain, item["target"]) if item.get("target") else None
    if node:
        item["target_hash"] = content_hash(node)
    return item


def approve(brain: Brain, inbox_id: str, force: bool = False) -> Node:
    path = brain.path("inbox", f"{inbox_id}.md")
    item = next((i for i in brain.iter_inbox() if i.path == path), None)
    if item is None:
        raise OpError(f"inbox item not found: {inbox_id}")
    pinned = item.meta.get("target_hash")
    if pinned and not force:
        target = resolve_id(brain, item.meta["target"])
        if target is not None and content_hash(target) != pinned:
            raise OpError(f"{inbox_id}: the note it changes was edited after this was proposed; "
                          "check it, then approve with --force or reject")
    change = dict(item.meta)
    change["text"] = item.body
    change["confidence"] = "explicit"  # a human approved it
    node = apply_change(brain, change, source=item.meta.get("source"))
    os.remove(path)
    return node


def reject(brain: Brain, inbox_id: str) -> None:
    path = brain.path("inbox", f"{inbox_id}.md")
    if not os.path.exists(path):
        raise OpError(f"inbox item not found: {inbox_id}")
    os.remove(path)


def undo(brain: Brain, session: str) -> str:
    log = brain.git("log", "--format=%H %s", f"--grep=consolidate .*{session}").stdout.strip()
    if not log:
        raise OpError(f"no consolidation commit found for session {session}")
    sha = log.splitlines()[0].split()[0]
    r = brain.git("revert", "--no-edit", sha)
    if r.returncode != 0:
        brain.git("revert", "--abort")
        raise OpError(f"git revert failed: {r.stderr.strip()}")
    return sha


def doctor(brain: Brain) -> list[str]:
    report = []
    bad = 0
    for node in brain.iter_nodes():
        try:
            schema.validate(node.meta, node.body)
        except schema.SchemaError as exc:
            bad += 1
            report.append(f"INVALID {node.path}: {exc}")
        why = security.blocked(node.meta["title"] + " " + node.body)
        if why:
            report.append(f"BLOCKED ({why}) {node.path}: hidden from sessions until edited")
    inbox = list(brain.iter_inbox())
    report.append(f"inbox: {len(inbox)} item(s) awaiting review")
    remotes = brain.git("remote", "-v").stdout.strip()
    if remotes:
        report.append(f"sync: {'auto' if brain.config.sync_auto else 'manual'} · device {brain.device} · "
                      "make sure the brain repository is PRIVATE:\n  " + remotes.splitlines()[0])
    else:
        report.append(f"sync: no remote (this brain lives only on this machine) · device {brain.device}")
    failed = brain.read_state("failed")
    if failed:
        report.append(f"failed consolidations: {len(failed)} (see .state/failed.json)")
    pending = brain.read_state("pending")
    if pending:
        report.append(f"pending transcripts: {len(pending)}")
    conflicts = brain.git("diff", "--name-only", "--diff-filter=U").stdout.strip()
    if conflicts:
        report.append("unresolved git conflicts:\n  " + conflicts.replace("\n", "\n  "))
    report.insert(0, f"nodes valid: {'yes' if bad == 0 else f'NO ({bad} invalid)'}")
    return report


# ---- human-readable home page --------------------------------------------

HOME_INTRO = (
    "> Generated by [rewire](https://github.com/NewbeeTryToCode/rewire) from the notes in `nodes/`.\n"
    "> Edit those notes (or use `rewire review`), not this page: it is rewritten after every change.\n"
)


def _md_link(brain: Brain, row) -> str:
    rel = os.path.relpath(row["path"], brain.root)
    title = row["title"].replace("[", "(").replace("]", ")")
    return f"[{title}]({rel})"


def write_home(brain: Brain, index) -> str:
    """Write README.md: an overview a person can read on GitHub or in Obsidian.

    Deterministic (no timestamps, no activation-based order) so it only changes
    when the memory itself changes and never causes needless sync conflicts.
    """
    index.refresh()
    active = index.rows("active = 1 ORDER BY importance DESC, recorded_at DESC, id")
    by_type: dict[str, list] = {}
    for r in active:
        by_type.setdefault(r["type"], []).append(r)
    inbox = list(brain.iter_inbox())
    counts = " · ".join(f"{len(by_type.get(t, []))} {t}" for t in schema.TYPES if by_type.get(t))
    lines = ["# Brain", "", HOME_INTRO, f"**{len(active)} active memories** — {counts or 'none yet'}"]
    if inbox:
        lines.append(f"\n**{len(inbox)} proposal(s) waiting for review** — run `rewire review` (see below).")

    def section(title, rows, limit=None, describe=None):
        if not rows:
            return
        lines.extend(["", f"## {title}", ""])
        for r in rows[:limit]:
            extra = describe(r) if describe else ""
            lines.append(f"- {_md_link(brain, r)}{extra}")
        if limit and len(rows) > limit:
            lines.append(f"- … {len(rows) - limit} more in `nodes/`")

    scope_of = lambda r: "" if r["scope"] == "global" else f" · {r['scope'].split(':', 1)[1]}"  # noqa: E731
    section("Rules (always applied)", by_type.get("constraint", []), describe=scope_of)
    section("Preferences", by_type.get("preference", []), describe=scope_of)

    hubs = [r for r in by_type.get("project", [])]
    if hubs:
        lines.extend(["", "## Projects"])
        for hub in sorted(hubs, key=lambda r: r["title"]):
            scope = hub["scope"]
            members = [r for r in active if r["scope"] == scope and r["type"] != "project"]
            lines.extend(["", f"### {_md_link(brain, hub)} — {len(members)} memories", ""])
            for r in members[:10]:
                lines.append(f"- {_md_link(brain, r)} · {r['type']}")
            if len(members) > 10:
                lines.append(f"- … {len(members) - 10} more")

    recent = sorted((r for r in active if r["type"] != "project"), key=lambda r: (r["recorded_at"], r["id"]),
                    reverse=True)
    section("Recently learned", recent, limit=10, describe=lambda r: f" · {r['type']} · {r['recorded_at'][:10]}")
    globals_ = [r for r in by_type.get("fact", []) + by_type.get("decision", []) + by_type.get("concept", [])
                + by_type.get("person", []) if r["scope"] == "global"]
    section("General knowledge", globals_, limit=20, describe=lambda r: f" · {r['type']}")

    sessions = sorted(glob.glob(brain.path("sessions", "*", "*.md")), reverse=True)
    if sessions:
        lines.extend(["", "## Recent sessions", ""])
        for path in sessions[:10]:
            day = os.path.basename(os.path.dirname(path))
            lines.append(f"- [{day} · {os.path.basename(path)[:-3]}]({os.path.relpath(path, brain.root)})")

    if inbox:
        lines.extend(["", "## Waiting for review", ""])
        for item in inbox[:20]:
            m = item.meta
            target = f" → `{m['target'][-8:].lower()}`" if m.get("target") else ""
            lines.append(f"- `{m.get('inbox_id')}` {m.get('action', 'NEW')}{target} · {m.get('type')}: {m.get('title')}")
        lines.append("\nApprove with `rewire review approve <id>` or reject with `rewire review reject <id>`.")

    lines.extend(["", "## How this brain works", "",
                  "- `nodes/<type>/` — one memory per file; relations are links you can click.",
                  "- `events/` — when memories were used; how strongly each memory is recalled is computed from this.",
                  "- `sessions/` — a summary of every consolidated agent session.",
                  "- Superseded memories are kept (status `superseded`) so history is never lost.", ""])
    text = "\n".join(lines)
    path = brain.path("README.md")
    old = open(path, encoding="utf-8").read() if os.path.exists(path) else None
    if text != old:
        with open(path, "w", encoding="utf-8") as fh:
            fh.write(text)
    return path
