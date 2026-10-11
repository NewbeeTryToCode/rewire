"""High-level operations shared by the CLI, MCP server and consolidation."""

import glob
import os

from . import project as project_mod
from . import fm, schema, security
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
            with open(path, encoding="utf-8") as fh:
                meta, body = fm.loads(fh.read())
            if str(meta.get("id", "")).lower().endswith(ref):
                hits.append(Node(meta, body, path))
        return hits[0] if len(hits) == 1 else None
    matches = [n for n in brain.iter_nodes() if n.id.lower().endswith(ref)]
    return matches[0] if len(matches) == 1 else None


def recall(brain: Brain, index: Index, query: str, k: int = 8, cwd: str | None = None,
           scope: str | None = None, beta: float | None = None) -> list[dict]:
    if scope:
        scopes = [scope] if scope != "all" else None
    else:
        scopes = project_mod.scopes_for(project_mod.detect(cwd, brain.config.projects))
    return index.search(query, scopes=scopes, k=k, beta=beta)


def get(brain: Brain, refs: list[str], session: str, harness: str = "mcp") -> list[dict]:
    out, events = [], []
    for ref in refs[:10]:
        node = resolve_id(brain, ref)
        if node is None:
            out.append({"ref": ref, "error": "not found"})
            continue
        out.append({"id": node.id, **{k: v for k, v in node.meta.items() if k != "id"}, "body": node.body})
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
        with open(path, encoding="utf-8") as fh:
            meta, body = fm.loads(fh.read())
        n = Node(meta, body, path)
        if meta.get("scope") == f"project:{slug}" and n.active:
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
    fields = {k: change[k] for k in ("importance", "confidence", "provenance") if change.get(k) is not None}
    edges = []
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


def approve(brain: Brain, inbox_id: str) -> Node:
    path = brain.path("inbox", f"{inbox_id}.md")
    item = next((i for i in brain.iter_inbox() if i.path == path), None)
    if item is None:
        raise OpError(f"inbox item not found: {inbox_id}")
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
        if "secret" in security.scan(node.body + " " + node.meta["title"]):
            report.append(f"SECRET? {node.path}")
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
