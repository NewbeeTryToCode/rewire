"""Why each BEAM question scored what it did, from a run.py results file and its reply cache (no LLM).

    python experiments/eval/diagnose_beam.py OUT/results-beam-<label>.json BEAM/chats OUT/diagnosis-<label> \\
        [--judge-model claude-cli/haiku] [--sizes 100K]

Writes <prefix>.csv (one row per question) and <prefix>.md: counts per category and cause, then every question
with its score, the judge's reason for each rubric item (why it is right or wrong) and what would improve it.
The judge reasons come from OUT/cache/llm.jsonl, keyed exactly as run.py asked them.

Causes of lost points (score < 1):
  abstain_extra    abstention question, the answer abstained but added details the judge counted against it
  hallucinated     abstention question, the answer gave content although the chat never says it
  terse_judged     the answer is a bare value (as BEAM's answer prompt asks) and the judge marked it non-responsive
  answer_missed    all evidence was in the context but the answer said it is not there
  answer_or_judge  all evidence was in the context; the answer was incomplete/wrong or the judge was strict
  retrieval_partial some evidence turns reached the context, others did not
  retrieval_miss   no evidence turn reached the context
  no_evidence_ids  the dataset lists no source turn for the question (scored by rubric only)
"""

import argparse
import csv
import json
import os
import re
import sys
from collections import Counter, defaultdict
from types import SimpleNamespace

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import importlib.util  # noqa: E402

_spec = importlib.util.spec_from_file_location("eval_run", os.path.join(HERE, "run.py"))
RUN = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(RUN)

ABSTAIN = re.compile(r"(?i)(not (mentioned|specified|stated|provided|included|in the (provided )?context)|"
                     r"no (information|mention|record|details?)|(does|do)n'?t (say|mention|specify|include|contain)|"
                     r"(does|do) not (say|mention|specify|include|contain)|(cannot|can't|unable to) (be )?(determin|answer|find|tell|confirm)|"
                     r"there is no|isn't (mentioned|specified))")
TERSE = re.compile(r"(?i)(no surrounding context|unclear what it answers|question is not provided|"
                   r"original question is not|non-responsive|bare|lacks? context)")

# What would raise the score, per category and cause. Phrased as changes to rewire, not to the benchmark.
INSIGHT = {
    ("instruction_following", "retrieval"):
        "The instruction was stated once, sessions earlier, and shares no words with the question, so recall cannot "
        "find it. Consolidate user instructions into `constraint`/`preference` nodes that `prime` always injects "
        "(they never decay), rather than relying on per-question recall (needs --mode full for BEAM).",
    ("instruction_following", "answer"):
        "The instruction was in context but the answer did not apply it. Put standing instructions in a separate "
        "'always follow' block above the excerpts so the answerer treats them as rules, not as history.",
    ("preference_following", "retrieval"):
        "The preference is phrased differently from the request (no shared keywords). Extract preferences as "
        "`preference` nodes with topic keywords and inject them via `prime`; semantic recall (WordLlama) could also "
        "bridge the paraphrase.",
    ("preference_following", "answer"):
        "The preference was retrieved but not applied. Label preference excerpts explicitly (\"user prefers ...\") "
        "so the answer adapts to them instead of only reporting facts.",
    ("summarization", "retrieval"):
        "A summary needs the whole thread across many sessions; 10 excerpts of 6 turns cannot cover it. Recall "
        "per-session summaries (`sessions/*.md`) for 'summarize/overview' questions, ordered by date, instead of "
        "raw excerpts.",
    ("summarization", "answer"):
        "The summary missed rubric points that were in context. Order excerpts chronologically and ask for one "
        "line per milestone; a larger --episodes budget for summary intents.",
    ("event_ordering", "retrieval"):
        "Ordering needs every event of the thread; only some were retrieved. Add an 'order/sequence' intent that "
        "widens recall (more excerpts, one per session) and sorts the context by date.",
    ("event_ordering", "answer"):
        "Events were present but listed out of order or incompletely. Sort the context by session date and turn "
        "and show the dates, so ordering is read off rather than inferred.",
    ("contradiction_resolution", "retrieval"):
        "Only one side of the contradiction was retrieved. When a fact has a superseded or conflicting version, "
        "recall should return both with dates (history intent) so the answer can point out the conflict.",
    ("contradiction_resolution", "answer"):
        "Both statements were in context but the answer picked one instead of flagging the contradiction. Mark "
        "conflicting memories as such in the context (e.g. 'conflicts with ...') at consolidation time.",
    ("knowledge_update", "retrieval"):
        "The newest value was not retrieved. Supersede the old fact on update (`valid_to`) and boost the newest "
        "version of a fact in recall.",
    ("knowledge_update", "answer"):
        "Old and new values were both in context and the answer used the wrong one. Show dates on every excerpt "
        "and keep superseded facts out unless the question asks for history.",
    ("multi_session_reasoning", "retrieval"):
        "Aggregating across sessions needs every mention; some were missed. Raise --episodes for counting/total "
        "questions (the LongMemEval result: 10 -> 20 excerpts fixed most undercounts).",
    ("multi_session_reasoning", "answer"):
        "All mentions were present but the aggregation was wrong. Facts from consolidation (one per event) are "
        "easier to count than raw turns; --mode full would help here.",
    ("temporal_reasoning", "retrieval"):
        "A date or one end of the interval was not retrieved. The time-window bonus needs the dates in the question "
        "to be parsed; make sure relative phrases in the chat are converted to absolute dates at ingest.",
    ("temporal_reasoning", "answer"):
        "The dates were in context but the computation or the format was off. Give each excerpt its absolute date "
        "and let the answer show the arithmetic.",
    ("information_extraction", "retrieval"):
        "The detail sits in a long turn that the question paraphrases. Smaller excerpts (fewer turns, long assistant "
        "turns split) and semantic recall would raise its rank.",
    ("information_extraction", "answer"):
        "The detail was in context but the answer missed part of it. Long assistant turns are truncated to "
        "1,500 chars in transcripts; check whether the detail was cut.",
    ("abstention", "hallucinated"):
        "The chat never says this, but the answer filled it in from related excerpts. The answer prompt should "
        "require an explicit statement of the asked detail before answering; otherwise say it is not in memory.",
    ("abstention", "abstain_extra"):
        "The answer correctly said the detail is missing but added nearby facts, which the judge counted against "
        "it. Answer abstentions with the abstention alone.",
}
GENERIC = {
    "terse_judged": "The answer gave the right value as a bare phrase (BEAM's own answer prompt asks for that) and "
                    "the judge, which does not see the question, called it non-responsive. Judge artefact; answering "
                    "in a full sentence that restates the question would avoid it.",
    "answer_missed": "Evidence was in context but the answer said it is not there: likely buried among 10 excerpts "
                     "(low rank) or in a truncated long turn. Rank the evidence excerpt higher or shorten excerpts.",
    "no_evidence_ids": "No source turn is listed, so retrieval cannot be checked; read the judge reasons.",
    "ok": "Nothing to fix.",
}


def bucket(cause: str) -> str:
    if cause.startswith("retrieval"):
        return "retrieval"
    return "answer" if cause in ("answer_or_judge", "answer_missed") else cause


def judge_items(it: dict, response: str, model: str, cache) -> list[dict]:
    out = []
    for item in it["rubric"]:
        prompt = (RUN.BEAM_PROMPTS.UNIFIED_LLM_JUDGE_BASE_PROMPT.replace("<rubric_item>", item)
                  .replace("<llm_response>", response))
        hit = cache.get(RUN.Cache.key("judge", model, prompt)) or {}
        text = hit.get("text", "")
        m = re.search(r'"score"\s*:\s*"?([01](?:\.\d+)?)', text)
        r = re.search(r'"reason"\s*:\s*"((?:[^"\\]|\\.)*)', text, re.S)
        out.append({"rubric": item, "score": float(m.group(1)) if m else None,
                    "reason": (r.group(1).encode().decode("unicode_escape", "ignore") if r else text).strip()})
    return out


def diagnose(row: dict, items: list[dict]) -> str:
    score = row.get("correct", 0.0)
    if score >= 1:
        return "ok"
    resp = row.get("response", "")
    if row["category"] == "abstention":
        return "abstain_extra" if ABSTAIN.search(resp) else "hallucinated"
    if len(resp.split()) <= 6 and any(TERSE.search(i["reason"]) for i in items):
        return "terse_judged"
    ev = row.get("evidence")
    if not ev:
        return "no_evidence_ids"
    if ev["all"]:
        return "answer_missed" if ABSTAIN.search(resp) else "answer_or_judge"
    return "retrieval_partial" if ev["turns"] else "retrieval_miss"


def insight(row: dict, cause: str, items: list[dict]) -> str:
    if cause in GENERIC:
        return GENERIC[cause]
    base = INSIGHT.get((row["category"], bucket(cause)), "")
    lost = [i["rubric"] for i in items if (i["score"] or 0) < 1]
    if lost and cause.startswith("retrieval"):
        base += f" Missing from the answer: {'; '.join(x[:90] for x in lost[:3])}."
    return base


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("results")
    ap.add_argument("chats")
    ap.add_argument("prefix")
    ap.add_argument("--judge-model", default="claude-cli/haiku")
    ap.add_argument("--sizes", default="100K")
    args = ap.parse_args()
    rows = json.load(open(args.results))["rows"]
    cache = RUN.Cache(os.path.join(os.path.dirname(os.path.abspath(args.results)), "cache", "llm.jsonl"))
    by_qid = {it["qid"]: it for it in RUN.beam_items(SimpleNamespace(data=args.chats, beam_sizes=args.sizes, convs="all"))}

    out, counts, scores = [], defaultdict(Counter), defaultdict(list)
    for row in rows:
        if "correct" not in row:
            continue
        it = by_qid[row["qid"]]
        items = judge_items(it, row.get("response", ""), args.judge_model, cache)
        cause = diagnose(row, items)
        counts[row["category"]][cause] += 1
        counts["overall"][cause] += 1
        scores[row["category"]].append(row["correct"])
        ev = row.get("evidence") or {}
        why = " | ".join(f"[{i['score']}] {i['rubric']}: {i['reason']}" for i in items)
        out.append({"qid": row["qid"], "category": row["category"], "score": round(row["correct"], 3),
                    "cause": cause, "question": it["question"], "gold": it["gold"],
                    "response": row.get("response", "").replace("\n", " "),
                    "evidence": f"{ev.get('turns', '-')}/{ev.get('n_turns', '-')}" if ev else "-",
                    "mrr": round(ev["rr"], 3) if ev else "", "context_tokens": row.get("context_tokens"),
                    "why": why, "insight": insight(row, cause, items), "_items": items})

    fields = [k for k in out[0] if not k.startswith("_")]
    with open(args.prefix + ".csv", "w", newline="", encoding="utf-8") as fh:
        w = csv.DictWriter(fh, fieldnames=fields, extrasaction="ignore")
        w.writeheader()
        w.writerows(out)

    causes = ["ok", "abstain_extra", "hallucinated", "terse_judged", "answer_missed", "answer_or_judge",
              "retrieval_partial", "retrieval_miss", "no_evidence_ids"]
    lines = [f"# BEAM diagnosis per question: {os.path.basename(args.results)}", "",
             f"Judge reasons from the cache of `{args.judge_model}`. Score = mean over rubric items (1 / 0.5 / 0).", "",
             "| category | n | mean score | " + " | ".join(causes) + " |", "|---|---|---|" + "---|" * len(causes)]
    for cat in sorted(counts, key=lambda c: (c == "overall", c)):
        n = sum(counts[cat].values())
        s = [x for v in scores.values() for x in v] if cat == "overall" else scores[cat]
        lines.append(f"| {cat} | {n} | {sum(s) / len(s):.1%} | "
                     + " | ".join(str(counts[cat][k]) if counts[cat][k] else "" for k in causes) + " |")
    lines += ["", "Causes:" + __doc__.split("Causes of lost points (score < 1):")[1].rstrip(), ""]
    lines += ["## Improvement insights, by where the points were lost", ""]
    for (cat, b), text in sorted(INSIGHT.items()):
        n = sum(1 for o in out if o["category"] == cat and bucket(o["cause"]) == b)
        if n:
            lines.append(f"- **{cat} / {b}** ({n} questions): {text}")
    for c in ("terse_judged", "answer_missed"):
        if counts["overall"][c]:
            lines.append(f"- **{c}** ({counts['overall'][c]} questions): {GENERIC[c]}")
    lines += ["", "## Every question", ""]
    for cat in sorted(scores):
        lines += [f"### {cat}", ""]
        for o in sorted((o for o in out if o["category"] == cat), key=lambda o: (o["score"], o["qid"])):
            lines += [f"#### {o['qid']}: score {o['score']:.2f} ({o['cause']})", "",
                      f"- **Question:** {o['question']}",
                      f"- **Gold:** {o['gold'][:400]}",
                      f"- **Answer:** {o['response'][:500]}",
                      f"- **Evidence in context:** {o['evidence']} turns · MRR {o['mrr']} · {o['context_tokens']} context tokens",
                      "- **Why (judge, per rubric item):**"]
            for i in o["_items"]:
                lines.append(f"  - [{i['score']}] *{i['rubric'][:200]}*: {i['reason'][:400]}")
            lines += [f"- **Insight:** {o['insight']}", ""]
    with open(args.prefix + ".md", "w", encoding="utf-8") as fh:
        fh.write("\n".join(lines) + "\n")
    print("\n".join(lines[:6 + len(counts)]))


if __name__ == "__main__":
    main()
