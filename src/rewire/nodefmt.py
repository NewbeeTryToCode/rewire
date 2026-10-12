"""On-disk node format: readable by people, Obsidian and GitHub alike.

    ---
    id: "01M4..."
    type: "preference"
    title: "Desain solid sebelum build"
    aliases: ["Desain solid sebelum build"]
    tags: ["rewire/preference"]
    scope: "project:rewire"
    ...
    source: "claude-code · e824c8b5-6b94-52f5-9188-fd31fa5ca77d · turn 338"
    about: ["[[rewire-tp7kc0q7]]"]
    ---
    Pengguna ingin desain dicek solid sebelum mulai membangun.

    ## Related
    - about: [rewire](../project/rewire-tp7kc0q7.md)

Relations are frontmatter properties holding [[wikilinks]] (Obsidian links them
and shows them in the graph) and are mirrored as relative markdown links under
"## Related" (clickable on GitHub too). Internally a node keeps
meta["edges"] = [{"rel", "to": <last 8 id chars>}]; the Related section is
generated and never part of the body.
"""

import json
import os
import re

from . import fm
from .schema import RELATIONS

RELATED = "## Related"
DERIVED = ("aliases", "tags", "status")  # regenerated on every write
_STEM_ID = re.compile(r"-([0-9a-z]{8})$")
_LINK = re.compile(r"^\[\[([^\]|#]+)(?:[|#][^\]]*)?\]\]$")
ORDER = ("id", "type", "title", "status", "aliases", "tags", "scope", "importance", "confidence", "provenance",
         "valid_from", "valid_to", "recorded_at", "retracted_at", "source", "keywords")


def short(ref: str) -> str:
    """Normalize an id, short id, file stem or [[link]] to the 8-char id suffix."""
    ref = str(ref).strip()
    m = _LINK.match(ref)
    if m:
        ref = m.group(1).strip()
    ref = os.path.basename(ref)
    if ref.endswith(".md"):
        ref = ref[:-3]
    m = _STEM_ID.search(ref.lower())
    if m:
        return m.group(1)
    return ref[-8:].lower() if len(ref) == 26 else ref.lower()


def source_text(source) -> str | None:
    if not source:
        return None
    if isinstance(source, str):
        return source
    parts = [source.get("harness")]
    if source.get("session"):
        parts.append(str(source["session"]))  # in full: it names the session file the claim came from
    if source.get("turn"):
        parts.append(f"turn {source['turn']}")
    return " · ".join(str(p) for p in parts if p)


def parse_source(source) -> tuple[str, str, int] | None:
    """(harness, session, turn) of a claim, from either stored form; None if incomplete."""
    if isinstance(source, dict):
        h, s, t = source.get("harness"), source.get("session"), source.get("turn")
    elif isinstance(source, str):
        parts = source.split(" · ")
        if len(parts) != 3 or not parts[2].startswith("turn "):
            return None
        h, s, t = parts[0], parts[1], parts[2][5:]
    else:
        return None
    try:
        return (str(h), str(s), int(t)) if h and s and t else None
    except ValueError:
        return None


def parse(text: str) -> tuple[dict, str]:
    meta, body = fm.loads(text)
    idx = body.find("\n" + RELATED + "\n")
    if body.startswith(RELATED + "\n"):
        idx = 0
    if idx != -1:
        body = body[:idx]
    edges = []
    for e in meta.pop("edges", None) or []:  # legacy v0.1 format
        if isinstance(e, dict) and e.get("rel") and e.get("to"):
            edges.append({"rel": e["rel"], "to": short(e["to"])})
    for rel in RELATIONS:
        values = meta.pop(rel, None)
        if values is None:
            continue
        for v in values if isinstance(values, list) else [values]:
            if v:
                edges.append({"rel": rel, "to": short(v)})
    for key in DERIVED:
        meta.pop(key, None)
    meta["edges"] = edges
    return meta, body.strip() + "\n" if body.strip() else ""


def render(meta: dict, body: str, path: str, locate) -> str:
    """locate(short_id) -> path of the target node file, or None."""
    out = {}
    title = meta.get("title", "")
    status = "superseded" if meta.get("valid_to") else "retracted" if meta.get("retracted_at") else "active"
    derived = {"aliases": [title] if title else [], "tags": [f"rewire/{meta.get('type')}"], "status": status}
    rest = {k: v for k, v in meta.items() if k not in ORDER and k != "edges" and k not in RELATIONS}
    for key in ORDER:
        if key in derived:
            out[key] = derived[key]
        elif key == "source":
            if meta.get("source"):
                out["source"] = source_text(meta["source"])
        elif key in meta:
            out[key] = meta[key]
    related_lines = []
    by_rel: dict[str, list[str]] = {}
    for e in meta.get("edges") or []:
        target = locate(short(e["to"]))
        stem = os.path.basename(target)[:-3] if target else short(e["to"])
        by_rel.setdefault(e["rel"], []).append(f"[[{stem}]]")
        if target:
            rel_path = os.path.relpath(target, os.path.dirname(path))
            label = (_title_of(target) or stem).replace("[", "(").replace("]", ")")
            related_lines.append(f"- {e['rel']}: [{label}]({rel_path})")
        else:
            related_lines.append(f"- {e['rel']}: {stem} (missing)")
    out.update(by_rel)
    out.update(rest)
    text = body.strip()
    if related_lines:
        text += ("\n\n" if text else "") + RELATED + "\n" + "\n".join(related_lines)
    return fm.dumps(out, text)


def _title_of(path: str) -> str | None:
    try:
        with open(path, encoding="utf-8") as fh:
            head = fh.read(2048)
    except OSError:
        return None
    m = re.search(r'(?m)^title:\s*(.+)$', head)
    if not m:
        return None
    raw = m.group(1).strip()
    if raw.startswith('"') and raw.endswith('"'):
        try:
            return json.loads(raw)
        except ValueError:
            return raw.strip('"')
    return raw
