"""Retrieval-only LoCoMo metric: does recall return the turns that hold the answer?

LoCoMo marks the evidence turns (`dia_id`) for every question. This checks how many
of them appear in the episodes rewire retrieves, plus how much context it costs.
No LLM calls: runs in seconds on brains already ingested by run.py, so it is the
inner loop for retrieval changes (the LLM judge varies by +-3 points between runs).

Usage: python experiments/locomo/evidence.py --data locomo10.json --out DIR [--k 50 --episodes 10]
       [--failed results-full-hybrid.json]   # also report on the questions that run got wrong
       [--facts]   # mode full: a turn also counts when a retrieved fact was extracted from it
"""

import argparse
import json
import os
import re
import sys
from collections import defaultdict

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "..", "src"))
sys.path.insert(0, HERE)

from rewire.brain import Brain  # noqa: E402
from rewire.index import Index  # noqa: E402
from run import CATS, link_sources, memories_for  # noqa: E402


def norm(s: str) -> str:
    return re.sub(r"\W+", " ", s.lower()).strip()


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--k", type=int, default=50)
    ap.add_argument("--episodes", type=int, default=10)
    ap.add_argument("--failed", help="results-*.json from run.py: also score its WRONG questions")
    ap.add_argument("--seed-weight", type=float, help="override Index.SEED_WEIGHT (0 disables)")
    ap.add_argument("--dump", help="write per-question rows here (JSON)")
    ap.add_argument("--facts", action="store_true",
                    help="count an evidence turn as found when a retrieved fact's source is that turn")
    args = ap.parse_args()
    if args.seed_weight is not None:
        Index.SEED_WEIGHT = args.seed_weight
    wrong = set()
    if args.failed:
        wrong = {q["qid"] for q in json.load(open(args.failed)) if q["label"] != "CORRECT"}
    rows = []
    for conv in json.load(open(args.data)):
        sid = conv["sample_id"]
        root = os.path.join(args.out, "brains", sid)
        if not os.path.exists(root):
            continue
        turns = {t["dia_id"]: t["text"] for k, v in conv["conversation"].items()
                 if re.fullmatch(r"session_\d+", k) for t in v}
        link_sources(root)
        index = Index(Brain(root))
        index.refresh()
        for i, qa in enumerate(conv["qa"]):
            if qa.get("category") not in CATS:
                continue
            ev = [e for raw in qa.get("evidence", []) for e in re.split(r"[;, ]+", raw) if e in turns]
            if not ev:
                continue
            mems = memories_for(index, qa["question"], args.k, args.episodes)
            eps = norm(" ".join(m["memory"] for m in mems if m["memory"].startswith("Conversation excerpt")))
            srcs = {m["source"] for m in mems if m.get("source")} if args.facts else set()
            hit = sum(norm(turns[e])[:50] in eps or e in srcs for e in ev)
            rows.append({"qid": f"{sid}-{i}", "cat": CATS[qa["category"]], "n_ev": len(ev), "hit": hit,
                         "ctx": sum(len(m["memory"]) for m in mems)})
    if args.dump:
        json.dump(rows, open(args.dump, "w"), indent=0)

    def report(name, group):
        if not group:
            return
        turn = sum(r["hit"] for r in group) / sum(r["n_ev"] for r in group)
        full = sum(r["hit"] == r["n_ev"] for r in group) / len(group)
        ctx = sum(r["ctx"] for r in group) / len(group) / 4  # ~4 chars per token
        print(f"| {name} | {len(group)} | {turn:.1%} | {full:.1%} | {ctx:,.0f} |")

    by = defaultdict(list)
    for r in rows:
        by[r["cat"]].append(r)
        by["overall"].append(r)
        if r["qid"] in wrong:
            by["previously WRONG"].append(r)
    print(f"k={args.k} episodes={args.episodes}")
    print("| group | n | evidence turns found | all evidence found | context tokens |")
    print("|---|---|---|---|---|")
    for name in ["single-hop", "multi-hop", "temporal", "open-domain", "overall", "previously WRONG"]:
        report(name, by[name])


if __name__ == "__main__":
    main()
