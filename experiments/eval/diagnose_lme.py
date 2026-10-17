"""Why each LongMemEval question was answered right or wrong, and what would improve it.

    python experiments/eval/diagnose_lme.py OUT/results-longmemeval-<label>.json DATA/longmemeval_s_cleaned.json \\
        OUT OUT/diagnosis-<label> --analyst-model claude-cli/claude-sonnet-5-5

Per question it combines three sources:
  1. a rule-based diagnosis (no LLM), from the evidence the runner recorded;
  2. where the evidence sessions rank in a deep recall (60 excerpts, same brain and frozen clock): a session
     ranked 11-20 was cut by the excerpt budget, one ranked past 60 (or never) is a recall miss;
  3. an analyst LLM that reads the question, gold, answer, judge reply, the evidence turns and (for wrong
     answers) the whole context, and writes why the verdict holds and one concrete improvement.

Writes <prefix>.json / .csv (one row per question) and <prefix>.md (counts, then every question).

Rule diagnoses:
  ok_full          right, every evidence session in the context
  ok_partial       right with an evidence session missing (another session or inference was enough)
  ok_abstain       abstention question, correctly said the information is not there
  wrong_answer     wrong although every evidence session was in the context (answering, judge or gold)
  wrong_retrieval  wrong, an evidence session was missing from the context
  wrong_abstain    wrong: abstention question answered with a made-up fact
"""

import argparse
import csv
import importlib.util
import json
import os
import re
import sys
from collections import Counter, defaultdict
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("eval_run", os.path.join(HERE, "run.py"))
RUN = importlib.util.module_from_spec(spec)
spec.loader.exec_module(RUN)
LME = RUN.LME

DEEP = 60  # excerpts in the deep recall that locates missing evidence

ROOT_CAUSES = {
    "correct_grounded": "right, from evidence in the context",
    "correct_partial": "right although some evidence was missing (inference, another session, or a lenient judge)",
    "correct_abstain": "abstention question, rightly said it is not in the history",
    "retrieval_cutoff": "evidence session ranked just below the excerpt budget",
    "retrieval_miss": "evidence session not recalled at all (vocabulary gap, buried in a long session)",
    "excerpt_trim": "evidence session recalled, but the excerpt cut out the turn holding the answer",
    "temporal_arithmetic": "dates in context, wrong date or interval arithmetic / wrong anchor date",
    "aggregation_count": "multi-session counting or summing: missed, double-counted or wrongly included items",
    "stale_update": "used an older value when a newer one exists (knowledge update)",
    "wrong_detail": "picked or misread the wrong detail though the right one was in context",
    "preference_generic": "preference question: generic advice that ignores the user's stated preference",
    "over_abstain": "said it does not know although the evidence was in the context",
    "hallucination": "abstention question: invented an answer instead of saying it was not mentioned",
    "judge_error": "the judge's verdict is wrong (answer actually matches / does not match the gold)",
    "gold_dispute": "the gold answer itself is questionable",
}

ANALYST_PROMPT = """You audit one question from LongMemEval_S, a benchmark for long-term chat memory. The memory system
(rewire) recalled excerpts from ~50 past chat sessions; an answering model wrote an answer from them; a judge model
graded it against the gold answer with the official LongMemEval prompt.

Question type: {qtype}{abs_note}
Question (asked on {qdate}): {question}
Gold answer: {gold}

Answer given:
<<<
{response}
>>>

Judge verdict: {verdict} (judge reply: {judge_text})

Retrieval facts (measured, trust them):
- rule diagnosis: {diag}
- evidence sessions and their rank among recalled excerpts (excerpt budget = {budget}; "-" = not in the top {deep}):
{ranks}
- evidence turns (the dataset's has_answer turns), and whether each reached the context:
{turns}
{context_block}
Decide:
1. why: 2-4 sentences explaining WHY the verdict is right or wrong. Be concrete: name the fact, date, item or
   step that made it right, or where it went wrong (missing evidence, which detail was misread, which arithmetic
   step failed, which item was missed or double counted). If you think the judge is wrong, say so.
2. root_cause: exactly one of {causes}.
3. owner: one of retrieval, answerer, judge, dataset, none.
4. key_insight: 1-3 sentences: the concrete change to rewire (recall ranking, excerpt size/trim, indexing,
   query expansion, fact extraction, answer prompt, temporal tooling, ...) that would make THIS question right,
   or, if it is already right, what made it work or what makes it fragile. Generalise one step: name the
   pattern, not only this question.
5. judge_ok: true if you agree with the judge's verdict, else false.

Reply with ONLY one line of JSON:
{{"why": "...", "root_cause": "...", "owner": "...", "key_insight": "...", "judge_ok": true}}"""


def norm(s: str) -> str:
    return re.sub(r"\W+", " ", str(s).lower()).strip()


def deep_ranks(q: dict, out: str) -> dict:
    """Rank (1-based, by first excerpt) of every evidence session in a DEEP recall on the same brain and clock."""
    root = os.path.join(out, "longmemeval", "brains-episodes", q["question_id"])
    RUN._now.at = datetime.fromtimestamp(os.path.getmtime(os.path.join(root, ".state", "lme_done")), timezone.utc)
    try:
        ctx = LME.recall(q, root, 30, DEEP)
    finally:
        RUN._now.at = None
    first = {}
    for i, e in enumerate(ctx["episodes"], 1):
        first.setdefault(e["session"], i)
    return {LME.session_file(s): first.get(LME.session_file(s)) for s in q["answer_session_ids"]}


def rule(row: dict) -> str:
    ev = row.get("evidence")
    if row["abstention"]:
        return "ok_abstain" if row["correct"] else "wrong_abstain"
    if row["correct"]:
        return "ok_full" if ev and ev["all"] else "ok_partial"
    return "wrong_answer" if ev and ev["all"] else "wrong_retrieval"


def parse(text: str) -> dict:
    m = re.search(r"\{.*\}", text, re.S)
    try:
        d = json.loads(m.group(0))
    except (AttributeError, ValueError):
        return {"why": text.strip()[:600], "root_cause": "unparsed", "owner": "?", "key_insight": "", "judge_ok": None}
    if d.get("root_cause") not in ROOT_CAUSES:
        d["root_cause_raw"], d["root_cause"] = d.get("root_cause"), "unparsed"
    return d


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("results")
    ap.add_argument("data")
    ap.add_argument("out", help="the run's --out (brains and LLM cache)")
    ap.add_argument("prefix")
    ap.add_argument("--analyst-model", default="claude-cli/claude-sonnet-5-5")
    ap.add_argument("--parallel", type=int, default=6)
    ap.add_argument("--budget", type=int, default=10, help="the run's --episodes")
    args = ap.parse_args()

    res = json.load(open(args.results))
    rows, summary = res["rows"], res["summary"]
    data = {q["question_id"]: q for q in json.load(open(args.data))}
    cache = RUN.Cache(os.path.join(args.out, "cache", "llm.jsonl"))
    client = RUN.Client(RUN.Ledger(None), api_key="unused")
    judge_model = summary["judge_model"]

    def one(row: dict) -> dict:
        q = data[row["qid"]]
        judge_text = (cache.get(RUN.Cache.key("judge", judge_model,
                                              LME.judge_prompt({**q, "response": row["response"]}))) or {}).get("text", "")
        diag = rule(row)
        ranks = deep_ranks(q, args.out) if not row["abstention"] else {}
        context = (row.get("trace") or {}).get("context", "")
        ctx_norm = norm(context)
        turns = []
        for sid, sess in zip(q["haystack_session_ids"], q["haystack_sessions"]):
            for t in sess:
                if t.get("has_answer"):
                    turns.append({"session": LME.session_file(sid), "role": t["role"],
                                  "in_context": norm(t["content"])[:60] in ctx_norm, "text": t["content"]})
        rank_lines = "\n".join(f"  {s}: rank {r or '-'}{' (in context)' if r and r <= args.budget else ''}"
                               for s, r in ranks.items()) or "  (abstention question: no evidence)"
        turn_lines = "\n".join(f"  [{t['role']}, {'IN context' if t['in_context'] else 'NOT in context'}] "
                               f"{t['text'][:700]}" for t in turns) or "  (none)"
        context_block = ("" if row["correct"] and not row["abstention"] else
                         f"\nFull context the answering model saw:\n<<<\n{context}\n>>>\n")
        prompt = ANALYST_PROMPT.format(
            qtype=row["category"], abs_note=" (abstention: the asked fact was never said; the right answer says so)"
            if row["abstention"] else "", qdate=q["question_date"], question=row["question"], gold=row["gold"],
            response=row["response"][:6000], verdict="CORRECT" if row["correct"] else "WRONG",
            judge_text=judge_text.strip()[:400] or "-", diag=diag, budget=args.budget, deep=DEEP, ranks=rank_lines,
            turns=turn_lines, context_block=context_block, causes=", ".join(ROOT_CAUSES))
        a = parse(RUN.ask(client, cache, "analyze", args.analyst_model, prompt, 1500)["text"])
        return {"qid": row["qid"], "category": row["category"], "abstention": row["abstention"],
                "question": row["question"], "question_date": q["question_date"], "gold": row["gold"],
                "response": row["response"], "correct": bool(row["correct"]), "judge_reply": judge_text.strip(),
                "diagnosis": diag, "evidence_ranks": ranks, "evidence_all": (row.get("evidence") or {}).get("all"),
                "evidence_turns_in_context": f"{sum(t['in_context'] for t in turns)}/{len(turns)}",
                "context_tokens": row["context_tokens"], "answer_words": len(row["response"].split()),
                "f1": RUN.LOCOMO.f1(row["response"], row["gold"]), "bleu1": RUN.bleu1(row["response"], row["gold"]),
                **{k: a.get(k) for k in ("why", "root_cause", "owner", "key_insight", "judge_ok")}}

    out = []
    with ThreadPoolExecutor(args.parallel) as pool:
        for i, r in enumerate(pool.map(one, rows), 1):
            out.append(r)
            if i % 25 == 0:
                print(f"{i}/{len(rows)}", flush=True)

    json.dump(out, open(args.prefix + ".json", "w"), indent=1, ensure_ascii=False)
    cols = ["qid", "category", "correct", "diagnosis", "root_cause", "owner", "judge_ok", "evidence_all",
            "evidence_turns_in_context", "evidence_ranks", "context_tokens", "answer_words", "question", "gold",
            "why", "key_insight"]
    with open(args.prefix + ".csv", "w", newline="", encoding="utf-8") as fh:
        w = csv.DictWriter(fh, cols, extrasaction="ignore")
        w.writeheader()
        for r in out:
            w.writerow({**r, "evidence_ranks": json.dumps(r["evidence_ranks"])})
    write_md(out, args.prefix + ".md", args.results, args.analyst_model, args.budget)


def write_md(out: list[dict], path: str, results: str, analyst: str, budget: int) -> None:
    cats = sorted({r["category"] for r in out})
    diags = ["ok_full", "ok_partial", "ok_abstain", "wrong_answer", "wrong_retrieval", "wrong_abstain"]
    by = defaultdict(Counter)
    for r in out:
        by[r["category"]][r["diagnosis"]] += 1
        by["overall"][r["diagnosis"]] += 1
    L = [f"# Per-question diagnosis: {os.path.basename(results)}", "",
         f"Analyst model `{analyst}` (why / root cause / key insight per question); rule diagnosis and evidence "
         f"ranks without an LLM (deep recall of {DEEP} excerpts, budget {budget}).", "",
         "| category | n | " + " | ".join(diags) + " |", "|---|---|" + "---|" * len(diags)]
    for c in cats + ["overall"]:
        n = sum(by[c].values())
        L.append(f"| {c} | {n} | " + " | ".join(f"{by[c][d]} ({by[c][d] / n:.0%})" for d in diags) + " |")

    rc = defaultdict(Counter)
    for r in out:
        rc[r["root_cause"]][r["category"]] += 1
    L += ["", "## Root causes (analyst)", "", "| root cause | meaning | n | " + " | ".join(cats) + " |",
          "|---|---|---|" + "---|" * len(cats)]
    for k, cnt in sorted(rc.items(), key=lambda kv: -sum(kv[1].values())):
        L.append(f"| {k} | {ROOT_CAUSES.get(k, '')} | {sum(cnt.values())} | " + " | ".join(str(cnt[c]) for c in cats) + " |")
    owners = Counter(r["owner"] for r in out if not r["correct"])
    L += ["", "Owner of the wrong answers: " + ", ".join(f"{k} {v}" for k, v in owners.most_common()),
          "", f"Analyst disagrees with the judge on {sum(r['judge_ok'] is False for r in out)} questions "
              "(listed with ⚠ below)."]

    # missing evidence: how far below the budget it sits
    miss = [rk for r in out if not r["abstention"] for rk in r["evidence_ranks"].values() if not rk or rk > budget]
    L += ["", f"Evidence sessions outside the context: {len(miss)}; ranked {budget + 1}-20: "
              f"{sum(bool(x) and x <= 20 for x in miss)}, 21-{DEEP}: {sum(bool(x) and x > 20 for x in miss)}, "
              f"not in the top {DEEP}: {sum(not x for x in miss)}."]

    for wrong in (True, False):
        L += ["", f"## {'Wrong' if wrong else 'Right'} answers", ""]
        for c in cats:
            rs = [r for r in out if r["category"] == c and r["correct"] != wrong]
            if not rs:
                continue
            L += [f"### {c} ({len(rs)})", ""]
            for r in rs:
                ranks = ", ".join(f"rank {v or f'>{DEEP}'}" for v in r["evidence_ranks"].values()) or "abstention"
                L += [f"**{r['qid']}**{' ⚠ analyst disagrees with the judge' if r['judge_ok'] is False else ''} · "
                      f"{r['diagnosis']} · `{r['root_cause']}` · owner {r['owner']} · evidence {ranks} · "
                      f"turns in context {r['evidence_turns_in_context']}",
                      f"- Q ({r['question_date'][:10]}): {r['question']}",
                      f"- Gold: {r['gold']}",
                      f"- Answer: {short(r['response'])}",
                      f"- Why: {r['why']}",
                      f"- Key insight: {r['key_insight']}", ""]
    with open(path, "w", encoding="utf-8") as fh:
        fh.write("\n".join(L) + "\n")


def short(text: str, n: int = 400) -> str:
    """The final answer part of a step-by-step reply, on one line."""
    tail = re.split(r"(?i)\**\s*(?:final\s+)?answer\s*\**\s*:", text)
    t = " ".join((tail[-1] if len(tail) > 1 else text).split())
    return t if len(t) <= n else "…" + t[-n:]


if __name__ == "__main__":
    main()
