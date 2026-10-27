"""Why each LoCoMo question was answered right or wrong, from a run.py results file (no LLM).

    python experiments/eval/diagnose.py OUT/results-locomo-<label>.json OUT/locomo/brains-full OUT/diagnosis-<label>

Writes <prefix>.csv (one row per question) and <prefix>.md (counts per category and diagnosis, with examples).
Each evidence turn reached the context as an excerpt, as a fact extracted from it, or not at all; a missing turn
is "retrieval" when the brain holds a fact from it (or its excerpt, which is always indexed) but recall ranked
it out, and "ingest" when no fact was ever extracted from it.

Diagnoses:
  ok_full        right, every evidence turn in the context
  ok_partial     right with some evidence missing (other memories, inference, or a lenient judge)
  ok_list_part   right by the judge, but the answer names only part of the gold list
  wrong_answer   wrong although every evidence turn was in the context (answering, gold or judge)
  wrong_abstain  wrong, the answer says the memories do not tell (and evidence was missing)
  wrong_retrieval wrong, evidence missing although a fact from the turn exists (ranked out)
  wrong_ingest   wrong, evidence missing and no fact was extracted from that turn (only its excerpt could help)
  no_evidence    the dataset lists no evidence turn for the question
"""

import csv
import glob
import json
import os
import re
import sys
from collections import Counter, defaultdict

ABSTAIN = re.compile(r"(?i)\b(not (mentioned|specified|stated|in the memories|clear)|no (information|mention|record)|"
                     r"(does|do) not (say|mention|specify|include)|cannot (be )?determin|unknown|don't have)\b")


def fact_sources(root: str) -> set[str]:
    """dia_ids ("D3:7") that at least one active fact in the brain was extracted from (read from the node files'
    front matter: source "locomo · conv-26-session_3 · turn 7", not retracted, not superseded)."""
    out = set()
    for path in glob.glob(os.path.join(root, "nodes", "*", "*.md")):
        head = open(path, encoding="utf-8").read(4000).split("\n---", 1)[0]
        if re.search(r'(?m)^(retracted_at|valid_to): "', head):
            continue
        m = re.search(r'(?m)^source: "[^"]*session_(\d+) · turn (\d+)"', head)
        if m:
            out.add(f"D{m.group(1)}:{m.group(2)}")
    return out


def diagnose(row: dict, facts: set[str]) -> tuple[str, list[str]]:
    ev = row.get("evidence")
    if not ev:
        return "no_evidence", []
    missing = [i for i, v in zip(ev.get("ids", []), ev.get("via", [])) if v == "missing"]
    if row.get("correct"):
        if row.get("item_coverage", 1) < 1:
            return "ok_list_part", missing
        return ("ok_partial" if missing else "ok_full"), missing
    if not missing:
        return "wrong_answer", missing
    if ABSTAIN.search(row.get("response", "")):
        return "wrong_abstain", missing
    return ("wrong_retrieval" if any(i in facts for i in missing) else "wrong_ingest"), missing


def main():
    results, brains, prefix = sys.argv[1:4]
    rows = json.load(open(results))["rows"]
    facts = {}
    out, counts, examples = [], defaultdict(Counter), defaultdict(list)
    for r in rows:
        conv = r["qid"].rsplit("-", 1)[0]
        if conv not in facts:
            facts[conv] = fact_sources(os.path.join(brains, conv))
        d, missing = diagnose(r, facts[conv])
        counts[r["category"]][d] += 1
        counts["overall"][d] += 1
        if len(examples[d]) < 6:
            examples[d].append(r)
        ev = r.get("evidence") or {}
        out.append({"qid": r["qid"], "category": r["category"], "diagnosis": d, "correct": r.get("correct"),
                    "question": r["question"], "gold": r["gold"], "response": r.get("response", "").replace("\n", " "),
                    "evidence_via": " ".join(f"{i}={v}" for i, v in zip(ev.get("ids", []), ev.get("via", []))),
                    "missing_with_fact": " ".join(i for i in missing if i in facts[conv]),
                    "missing_without_fact": " ".join(i for i in missing if i not in facts[conv]),
                    "item_coverage": r.get("item_coverage", ""), "context_tokens": r.get("context_tokens")})
    with open(prefix + ".csv", "w", newline="", encoding="utf-8") as fh:
        w = csv.DictWriter(fh, fieldnames=list(out[0]))
        w.writeheader()
        w.writerows(out)
    kinds = ["ok_full", "ok_partial", "ok_list_part", "wrong_answer", "wrong_abstain", "wrong_retrieval",
             "wrong_ingest", "no_evidence"]
    lines = [f"# Diagnosis per question: {os.path.basename(results)}", "",
             "| category | n | " + " | ".join(kinds) + " |", "|---|---|" + "---|" * len(kinds)]
    for cat in sorted(counts, key=lambda c: (c == "overall", c)):
        n = sum(counts[cat].values())
        lines.append(f"| {cat} | {n} | " + " | ".join(f"{counts[cat][k]} ({counts[cat][k] / n:.0%})" for k in kinds) + " |")
    lines += ["", __doc__.split("Diagnoses:")[1].rstrip(), ""]
    for k in kinds:
        if examples[k] and k != "ok_full":
            lines += [f"## {k} (examples)", ""]
            for r in examples[k]:
                lines.append(f"- {r['qid']} ({r['category']}): {r['question']} | gold: {r['gold']} | "
                             f"answer: {r.get('response', '')[:160].replace(chr(10), ' ')}")
            lines.append("")
    with open(prefix + ".md", "w", encoding="utf-8") as fh:
        fh.write("\n".join(lines))
    print("\n".join(lines[:4 + len(counts)]))


if __name__ == "__main__":
    main()
