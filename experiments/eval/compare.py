"""Paired comparison of eval arms over replicated runs (run.py --replicate N).

Each arm is a run.py output directory holding, per question set, results-<bench>-<label>.json for replicate 0 and
results-<bench>-<label>-r<N>.json for the replicates. A question's score is its mean J over the replicates found;
an arm's J is the mean over questions. The difference to the baseline is paired per question, with a 95% bootstrap
interval over questions (replicate noise is averaged inside each question's score first).

    python experiments/eval/compare.py --base W=/tmp/W --arm k12=/tmp/k12 --sets e2e-locomo e2e-lme
"""
import argparse
import glob
import json
import os
import random
import re


def load(root: str, label: str) -> dict[str, list[float]]:
    """qid -> J of every replicate of one question set."""
    scores: dict[str, list[float]] = {}
    for path in sorted(glob.glob(os.path.join(root, f"results-*-{label}.json"))
                       + glob.glob(os.path.join(root, f"results-*-{label}-r[0-9]*.json"))):
        if not re.search(rf"-{re.escape(label)}(-r\d+)?\.json$", path):
            continue
        for row in json.load(open(path))["rows"]:
            if row.get("correct") is not None:
                scores.setdefault(row["qid"], []).append(float(row["correct"]))
    return scores


def mean(xs) -> float:
    xs = list(xs)
    return sum(xs) / len(xs) if xs else float("nan")


def interval(diffs: list[float], n: int = 10000, seed: int = 0) -> tuple[float, float]:
    rng = random.Random(seed)
    boots = sorted(mean(rng.choice(diffs) for _ in diffs) for _ in range(n))
    return boots[int(0.025 * n)], boots[int(0.975 * n) - 1]


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--base", required=True, help="NAME=DIR of the baseline arm")
    ap.add_argument("--arm", action="append", default=[], help="NAME=DIR of an arm to compare (repeatable)")
    ap.add_argument("--sets", nargs="+", required=True, help="run labels of the question sets, e.g. e2e-locomo")
    ap.add_argument("--total", nargs="*", default=None, help="sets pooled into the overall row (default: all)")
    args = ap.parse_args()
    arms = [a.split("=", 1) for a in [args.base] + args.arm]
    data = {name: {s: load(root, s) for s in args.sets} for name, root in arms}
    base = arms[0][0]
    pooled = args.total if args.total is not None else args.sets
    print("| set | n | replicates | " + " | ".join(f"J {name}" for name, _ in arms)
          + " | " + " | ".join(f"{name} − {base} [95% CI]" for name, _ in arms[1:]) + " |")
    print("|" + "---|" * (3 + len(arms) + len(arms) - 1))
    for s in args.sets + ["overall"]:
        sets = pooled if s == "overall" else [s]
        qs = [(x, q) for x in sets for q in data[base][x]
              if all(q in data[name][x] for name, _ in arms)]
        if not qs:
            continue
        per = {name: [mean(data[name][x][q]) for x, q in qs] for name, _ in arms}
        reps = "/".join(str(round(mean(len(data[name][x][q]) for x, q in qs), 1)) for name, _ in arms)
        cells = [f"{100 * mean(per[name]):.1f}%" for name, _ in arms]
        for name, _ in arms[1:]:
            diffs = [a - b for a, b in zip(per[name], per[base])]
            lo, hi = interval(diffs)
            mark = " *" if lo > 0 or hi < 0 else ""
            cells.append(f"{100 * mean(diffs):+.1f} [{100 * lo:+.1f}, {100 * hi:+.1f}]{mark}")
        print(f"| {s} | {len(qs)} | {reps} | " + " | ".join(cells) + " |")
    print("\n* the 95% interval excludes 0")


if __name__ == "__main__":
    main()
