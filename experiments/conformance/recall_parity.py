"""Recall parity: the Rust binary must return what the Python reference returns (keyword mode).

For every LoCoMo question of the given conversations, compare `ops.recall(k=50, episodes=10, scope=all)`
from Python (REWIRE_SEMANTIC=0) with `rewire recall --json` from Rust, on two identical copies of the
same brain (each implementation builds its own index). No LLM involved.

Usage: python experiments/conformance/recall_parity.py --data locomo10.json --brains DIR --convs conv-26
       --rust rust/target/release/rewire [--semantic]   (DIR holds py-<conv>/ and rs-<conv>/ copies)
"""

import argparse
import json
import os
import subprocess
import sys

# --semantic compares hybrid ranking (Python with wordllama, Rust with the model from `rewire semantic install`)
os.environ["REWIRE_SEMANTIC"] = "1" if "--semantic" in sys.argv else "0"
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "..", "src"))

from rewire import ops  # noqa: E402
from rewire.brain import Brain  # noqa: E402
from rewire.index import Index  # noqa: E402


def key(hit: dict):
    if hit["type"] == "episode":
        return ("ep", hit["session"], hit["text"])
    return ("node", hit["id"])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", required=True)
    ap.add_argument("--brains", required=True)
    ap.add_argument("--convs", default="conv-26")
    ap.add_argument("--rust", required=True)
    ap.add_argument("--k", type=int, default=50)
    ap.add_argument("--episodes", type=int, default=10)
    ap.add_argument("--semantic", action="store_true")
    ap.add_argument("--expansions", help="JSON {question: [extra queries]}: also compare recall with `also`")
    args = ap.parse_args()
    data = {c["sample_id"]: c for c in json.load(open(args.data))}
    expansions = json.load(open(args.expansions)) if args.expansions else {}
    total = same_order = same_set = facts_equal = 0
    examples = []
    for conv in args.convs.split(","):
        py_brain = Brain(os.path.join(args.brains, f"py-{conv}"))
        rs_root = os.path.join(args.brains, f"rs-{conv}")
        index = Index(py_brain)
        index.refresh()
        for qa in data[conv]["qa"]:
            if qa.get("category") not in (1, 2, 3, 4):
                continue
            q = qa["question"]
            also = expansions.get(q, []) if expansions else []
            if expansions and not also:
                continue
            py = ops.recall(py_brain, index, q, k=args.k, scope="all", episodes=args.episodes, also=also)
            out = subprocess.run([args.rust, "--brain", rs_root, "recall", "--json", "-k", str(args.k),
                                  "--episodes", str(args.episodes), "--scope", "all",
                                  *[a for e in also for a in ("--also", e)], "--", q],
                                 capture_output=True, text=True, check=True).stdout
            rs = json.loads(out)
            pk, rk = [key(h) for h in py], [key(h) for h in rs]
            total += 1
            same_order += pk == rk
            same_set += set(pk) == set(rk)
            facts_equal += [k for k in pk if k[0] == "node"] == [k for k in rk if k[0] == "node"]
            if pk != rk and len(examples) < 5:
                examples.append((q, [k for k in pk if k not in rk][:3], [k for k in rk if k not in pk][:3]))
    print(f"questions: {total}")
    print(f"identical results (same items, same order): {same_order} ({same_order / total:.1%})")
    print(f"same items, any order:                      {same_set} ({same_set / total:.1%})")
    print(f"identical fact ranking:                     {facts_equal} ({facts_equal / total:.1%})")
    for q, only_py, only_rs in examples:
        print(f"- {q}\n    only python: {only_py}\n    only rust:   {only_rs}")


if __name__ == "__main__":
    main()
