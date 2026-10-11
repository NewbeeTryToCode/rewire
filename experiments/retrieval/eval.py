"""Retrieval quality eval: does `recall` return the right memory?

Builds a brain from dataset.json (memories, links, superseded facts), runs every
query through Index.search with several configurations and reports, per query
category: R@1, R@5, MRR, stale@5 (an outdated memory shows up in the top 5)
and leak@5 (another project's memory shows up in the top 5).

Run: python experiments/retrieval/eval.py
"""

import json
import os
import shutil
import sys
import tempfile
from collections import defaultdict

HERE = os.path.dirname(__file__)
sys.path.insert(0, os.path.join(HERE, "..", "..", "src"))

from rewire import ops  # noqa: E402
from rewire import project as project_mod  # noqa: E402
from rewire.brain import Brain  # noqa: E402
from rewire.index import Index  # noqa: E402
from rewire.util import now_iso  # noqa: E402

CATEGORIES = ["lexical", "paraphrase", "cross_lingual", "relational", "stale", "scope"]


def build(data: dict, root: str, keywords: dict | None = None) -> tuple[Brain, dict]:
    brain = Brain.init(root, device="eval")
    ids = {}
    for m in data["memories"]:
        change = {"action": "NEW", "type": m["type"], "text": m["text"], "title": m.get("title") or m["text"][:80],
                  "scope": m["scope"], "confidence": "explicit", "provenance": "user", "importance": 3}
        if keywords and keywords.get(m["id"]):
            change["keywords"] = keywords[m["id"]]
        ids[m["id"]] = ops.apply_change(brain, change).id
    for m in data["memories"]:
        node = brain.get_node(ids[m["id"]])
        edges = list(node.meta["edges"])
        edges += [{"rel": l["rel"], "to": ids[l["to"]]} for l in m.get("links") or [] if l.get("to") in ids]
        meta = dict(node.meta, edges=edges)
        if m.get("superseded_by") in ids:
            meta.update(valid_to=now_iso()[:10], retracted_at=now_iso())
            new = brain.get_node(ids[m["superseded_by"]])
            brain.write_node(dict(new.meta, edges=new.meta["edges"] + [{"rel": "supersedes", "to": node.id}]),
                             new.body, new.path)
            node = brain.get_node(node.id)
            meta = dict(node.meta, edges=meta["edges"], valid_to=meta["valid_to"], retracted_at=meta["retracted_at"])
        brain.write_node(meta, node.body, node.path)
    return brain, ids


def evaluate(brain: Brain, ids: dict, queries: list, rewrites: dict | None = None, **search_kw) -> dict:
    index = Index(brain)
    index.refresh()
    back = {v: k for k, v in ids.items()}
    per = defaultdict(lambda: defaultdict(float))
    for n, q in enumerate(queries):
        scopes = project_mod.scopes_for(q["project"]) if q.get("project") else None
        text = (rewrites or {}).get(str(n)) or q["q"]
        hits = [back[h["id"]] for h in index.search(text, scopes=scopes, k=10, **search_kw) if h["id"] in back]
        gold = set(q["gold"])
        rank = next((i + 1 for i, h in enumerate(hits) if h in gold), None)
        s = per[q["category"]]
        s["n"] += 1
        s["r1"] += rank == 1
        s["r5"] += rank is not None and rank <= 5
        s["mrr"] += 1 / rank if rank else 0
        s["stale5"] += bool(set(q.get("stale") or []) & set(hits[:5]))
        s["leak5"] += bool(set(q.get("wrong_scope") or []) & set(hits[:5]))
    index.close()
    total = defaultdict(float)
    for s in per.values():
        for k, v in s.items():
            total[k] += v
    per["ALL"] = total
    return per


def table(results: dict) -> str:
    cats = CATEGORIES + ["ALL"]
    lines = ["| config | " + " | ".join(cats) + " |", "|---|" + "---|" * len(cats)]
    for name, per in results.items():
        cells = []
        for c in cats:
            s = per[c]
            n = s["n"] or 1
            cell = f"{s['r5'] / n:.0%} / {s['mrr'] / n:.2f}"
            if c in ("stale", "ALL") and s["stale5"]:
                cell += f" ⚠stale {s['stale5'] / n:.0%}"
            if c in ("scope", "ALL") and s["leak5"]:
                cell += f" ⚠leak {s['leak5'] / n:.0%}"
            cells.append(cell)
        lines.append(f"| {name} | " + " | ".join(cells) + " |")
    return "\n".join(lines) + "\n\ncell = R@5 / MRR"


def main():
    with open(os.path.join(HERE, "dataset.json"), encoding="utf-8") as fh:
        data = json.load(fh)
    kw_path = os.path.join(HERE, "keywords.json")
    keywords = json.load(open(kw_path, encoding="utf-8")) if os.path.exists(kw_path) else None
    tmp = tempfile.mkdtemp()
    try:
        brain, ids = build(data, os.path.join(tmp, "brain"))
        results = {
            "text only": evaluate(brain, ids, data["queries"], spread=False, beta=0.0),
            "text + graph (current default)": evaluate(brain, ids, data["queries"]),
        }
        if keywords:
            kbrain, kids = build(data, os.path.join(tmp, "kbrain"), keywords)
            results["text + graph + keywords"] = evaluate(kbrain, kids, data["queries"])
            aq_path = os.path.join(HERE, "agent_queries.json")
            if os.path.exists(aq_path):
                rewrites = json.load(open(aq_path, encoding="utf-8"))
                results["+ agent-written queries (MCP guide)"] = evaluate(kbrain, kids, data["queries"], rewrites)
        print(table(results))
    finally:
        shutil.rmtree(tmp)


if __name__ == "__main__":
    main()
