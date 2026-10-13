"""Procedural memory: reusable procedures as Agent Skills in the brain (skills/<name>/SKILL.md).

Consolidation proposes them (create, or append to an existing one); they always go through review,
since a skill is followed as instructions. The snapshot lists them, `get skill:<name>` reads one, and
doctor reports skills nobody has used for a while.
"""

import glob
import os
from datetime import datetime, timezone

from . import fm, security
from .util import atomic_write, now_iso, slugify

STALE_DAYS = 90


def normalize_name(name: str) -> str:
    return slugify(str(name or ""), 40) if str(name or "").strip() else ""


def path(brain, name: str) -> str:
    return brain.path("skills", name, "SKILL.md")


def read(brain, name: str) -> dict | None:
    p = path(brain, normalize_name(name))
    if not os.path.exists(p):
        return None
    with open(p, encoding="utf-8") as fh:
        meta, body = fm.loads(fh.read())
    return {"name": str(meta.get("name") or normalize_name(name)), "description": str(meta.get("description") or ""),
            "updated": str(meta.get("updated") or ""), "path": p, "body": body}


def all_skills(brain) -> list[dict]:
    names = sorted(os.path.basename(os.path.dirname(p)) for p in glob.glob(brain.path("skills", "*", "SKILL.md")))
    return [s for s in (read(brain, n) for n in names) if s]


def apply(brain, item: dict, steps: str) -> str:
    """Create the skill, or append the new steps to the existing one. Returns the file path."""
    name = normalize_name(item.get("name"))
    if not name or not steps.strip():
        raise ValueError("a skill needs a name and steps")
    src = item.get("source") or {}
    source = f"{src.get('harness', '?')} · {src.get('session', '?')} · turn {src.get('turn', 0)}"
    current = read(brain, name)
    if current is None:
        meta = {"name": name, "description": str(item.get("description") or ""), "updated": now_iso(),
                "sources": [source]}
        body = steps.strip()
    else:
        with open(current["path"], encoding="utf-8") as fh:
            meta, _ = fm.loads(fh.read())
        meta["updated"] = now_iso()
        if item.get("description") and not meta.get("description"):
            meta["description"] = str(item["description"])
        meta["sources"] = list(meta.get("sources") or []) + [source]
        body = current["body"].strip() + f"\n\n## Update {now_iso()[:10]}\n\n" + steps.strip()
    atomic_write(path(brain, name), fm.dumps(meta, body))
    return path(brain, name)


def blocked(skill: dict) -> str:
    return security.blocked(f"{skill['name']} {skill['description']} {skill['body']}")


def stale(brain, now: datetime | None = None) -> list[tuple[str, int]]:
    """(name, days since last use or update) for skills unused for STALE_DAYS."""
    now = now or datetime.now(timezone.utc)
    last = {}
    for ev in brain.iter_events():
        node = str(ev.get("node") or "")
        if ev.get("ev") == "use" and node.startswith("skill:"):
            last[node[6:]] = max(last.get(node[6:], ""), str(ev.get("t") or ""))
    out = []
    for s in all_skills(brain):
        when = max(last.get(s["name"], ""), s["updated"])
        try:
            days = (now - datetime.fromisoformat(when.replace("Z", "+00:00"))).days
        except ValueError:
            continue
        if days >= STALE_DAYS:
            out.append((s["name"], days))
    return out
