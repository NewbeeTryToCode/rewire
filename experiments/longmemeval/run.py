"""LongMemEval for rewire: official answer template and official per-type judge prompts.

Every question has its own chat history (LongMemEval_S: ~50 sessions, ~115k tokens), so every
question gets its own brain. Two ingest modes:

  episodes : transcripts are stored and indexed exactly as consolidation stores them, but no
             LLM extraction runs. Free, so it runs on all 500 questions.
  full     : every session is consolidated (LLM extraction + judge, as in real use). ~2 LLM
             calls per session, so it is meant for a subset (--per-type).

Stages (cached on disk, resumable):
  retrieval : did recall return the evidence sessions/turns (`answer_session_ids`, `has_answer`)?
              No LLM. Printed for every run.
  answer    : rewire recall -> official "history chats + user facts, step by step" template.
  judge     : official LongMemEval prompts (one per question type, plus abstention).

Usage:
  python experiments/longmemeval/run.py --data longmemeval_s_cleaned.json --out DIR --mode episodes
  python experiments/longmemeval/run.py ... --mode full --per-type 5 --label full5
  python experiments/longmemeval/run.py ... --retrieval-only
"""

import argparse
import json
import os
import random
import re
import sys
from collections import defaultdict
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "..", "src"))
sys.path.insert(0, HERE)
sys.path.append(os.path.join(HERE, "..", "locomo"))

from lme_prompts import ANSWER_TEMPLATE, judge_prompt  # noqa: E402
from rewire import ops  # noqa: E402
from rewire.brain import Brain  # noqa: E402
from rewire.consolidate import Consolidator  # noqa: E402
from rewire.index import Index  # noqa: E402
from rewire.llm import CommandProvider  # noqa: E402
from rewire.util import slugify  # noqa: E402


def _locomo_harness():
    import importlib.util

    spec = importlib.util.spec_from_file_location("locomo_run", os.path.join(HERE, "..", "locomo", "run.py"))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


claude_json = _locomo_harness().claude_json  # claude -p with retries, shared with the LoCoMo harness

HARNESS = "lme"
TYPES = ["single-session-user", "single-session-assistant", "single-session-preference",
         "multi-session", "knowledge-update", "temporal-reasoning"]


def iso(date: str) -> str:
    """'2023/05/20 (Sat) 02:21' -> '2023-05-20'."""
    m = re.match(r"(\d{4})/(\d{2})/(\d{2})", date or "")
    return "-".join(m.groups()) if m else ""


def turns_of(session: list[dict]) -> list[dict]:
    return [{"n": i, "role": t["role"], "text": t["content"], "ts": ""} for i, t in enumerate(session, 1)]


def session_file(sid: str) -> str:
    return f"{HARNESS}-{slugify(sid, 40)}.md"


# ---- ingest -------------------------------------------------------------

def new_brain(root: str) -> Brain:
    if not os.path.exists(os.path.join(root, "brain.toml")):
        Brain.init(root, device="eval")
        toml = os.path.join(root, "brain.toml")
        text = open(toml).read().replace('profile = "coding"', 'profile = "general"')
        with open(toml, "w") as fh:
            fh.write(text)
    return Brain(root)


def ingest_episodes(q: dict, root: str) -> None:
    """Transcripts only: what consolidation writes to sessions/, without the LLM extraction."""
    brain = new_brain(root)
    if os.path.exists(os.path.join(root, ".state", "lme_done")):
        return
    con = Consolidator.__new__(Consolidator)  # only the transcript writer: no LLM
    con.brain = brain
    for sid, date, session in zip(q["haystack_session_ids"], q["haystack_dates"], q["haystack_sessions"]):
        con._write_session(HARNESS, sid, None, [], [], [], [], [], set(), iso(date), con._transcript(turns_of(session)))
    os.makedirs(os.path.join(root, ".state"), exist_ok=True)
    open(os.path.join(root, ".state", "lme_done"), "w").close()


def ingest_full(q: dict, root: str) -> None:
    """Every session consolidated in order, as a harness would; inbox accepted in bulk (no reviewer)."""
    brain = new_brain(root)
    done_path = os.path.join(root, ".state", "lme_sessions.json")
    done = json.load(open(done_path)) if os.path.exists(done_path) else []
    con = Consolidator(brain, provider=CommandProvider("claude -p --output-format text",
                                                       os.path.join(root, ".state", "llm")), log=lambda *_: None)
    for sid, date, session in zip(q["haystack_session_ids"], q["haystack_dates"], q["haystack_sessions"]):
        if sid in done:
            continue
        with brain.lock(timeout=600):
            con._consolidate(HARNESS, {"session": sid, "cwd": None, "date": iso(date)}, turns_of(session))
        done.append(sid)
        with open(done_path, "w") as fh:
            json.dump(done, fh)
    for item in list(brain.iter_inbox()):
        try:
            ops.approve(brain, item.meta["inbox_id"])
        except Exception as exc:
            print("approve failed:", exc)
    open(os.path.join(root, ".state", "lme_done"), "w").close()


# ---- recall + retrieval metric -----------------------------------------

def recall(q: dict, root: str, k: int, episodes: int) -> dict:
    brain = Brain(root)
    index = Index(brain)
    index.refresh()
    facts, eps = [], []
    for h in ops.recall(brain, index, q["question"], k=k, scope="all", episodes=episodes):
        if h["type"] == "episode":
            eps.append({"session": os.path.basename(h["session"]), "date": h["date"], "text": h["text"]})
        else:
            row = index.rows("id = ?", (h["id"],))[0]
            date = json.loads(row["meta"]).get("valid_from") or ""
            facts.append({"date": date[:10], "text": row["body"].strip() or row["title"]})
    return {"facts": facts, "episodes": eps}


def coverage(turn: str, context_norm: str) -> float:
    """Share of an evidence turn's sentences that reached the context (normalised text). Matching only the turn's
    start counts a turn cut after 1,500 characters as found; this sees what was cut."""
    sents = [s for s in (re.sub(r"\W+", " ", p.lower()).strip() for p in re.split(r"(?<=[.!?:])\s+|\n+", turn))
             if len(s) >= 20]
    return sum(s in context_norm for s in sents) / len(sents) if sents else 1.0


def evidence(q: dict, ctx: dict) -> dict | None:
    """Session- and turn-level recall of the evidence; None for abstention questions (no evidence)."""
    if q["question_id"].endswith("_abs"):
        return None
    want = {session_file(s) for s in q["answer_session_ids"]}
    got = {e["session"] for e in ctx["episodes"]}
    norm = lambda s: re.sub(r"\W+", " ", s.lower()).strip()  # noqa: E731
    text = norm(" ".join(e["text"] for e in ctx["episodes"]))
    turns = [t["content"] for s in q["haystack_sessions"] for t in s if t.get("has_answer")]
    hit_turns = sum(norm(t)[:60] in text for t in turns)
    return {"sess_any": bool(want & got), "sess_all": want <= got,
            "turns": hit_turns, "n_turns": len(turns), "turn_coverage": [coverage(t, text) for t in turns],
            "ctx": sum(len(e["text"]) for e in ctx["episodes"]) +
            sum(len(f["text"]) for f in ctx["facts"])}


# ---- answer + judge ----------------------------------------------------

def history_string(ctx: dict) -> str:
    """Official layout: one block per session, oldest first; rewire facts appended as 'user facts'."""
    by: dict[str, list] = defaultdict(list)
    for e in ctx["episodes"]:
        by[(e["date"], e["session"])].append(e["text"])
    blocks = [f"### Session Date: {date}\nSession Content:\n" + "\n...\n".join(texts)
              for (date, _), texts in sorted(by.items())]
    facts = "\n".join(f"- ({f['date'] or 'undated'}) {f['text']}" for f in sorted(ctx["facts"], key=lambda f: f["date"]))
    return "\n\n".join(blocks) + ("\n\nUser facts extracted from the chat history:\n" + facts if facts else "")


def answer_batch(batch: list[dict]) -> dict:
    blocks = [f"### QUESTION {q['question_id']}\n" + ANSWER_TEMPLATE.format(history_string(q["ctx"]),
                                                                          q["question_date"], q["question"])
              for q in batch]
    prompt = ("You will answer SEVERAL independent questions. Each has its OWN chat history; use only that "
              "history for that question.\n\n" + "\n\n".join(blocks) +
              "\n\nReason step by step for each question privately. Then output ONLY a JSON object mapping each "
              "QUESTION id to its final answer string (concise, but keep the reasoning facts that justify it).")
    return {str(k): str(v) for k, v in claude_json(prompt).items()}


def judge_batch(batch: list[dict]) -> dict:
    items = "\n\n".join(f"### ITEM {q['question_id']}\n" + judge_prompt(q) for q in batch)
    prompt = ("Grade EACH item below independently, following its own instructions.\n\n" + items +
              '\n\nReturn ONLY a JSON object mapping each ITEM id to "yes" or "no".')
    return {str(k): str(v).strip().lower() for k, v in claude_json(prompt).items()}


# ---- driver --------------------------------------------------------------

def select(data: list[dict], per_type: int, seed: int, ids: str | None) -> list[dict]:
    if ids:
        want = set(ids.split(","))
        return [q for q in data if q["question_id"] in want]
    if not per_type:
        return data
    rng = random.Random(seed)
    out = []
    for t in TYPES:
        group = [q for q in data if q["question_type"] == t]
        out += rng.sample(group, min(per_type, len(group)))
    return out


def safe(fn):
    """A batch that keeps failing (usage limit) is skipped, not fatal: a rerun resumes it from the cache."""
    def run(batch):
        try:
            return fn(batch)
        except Exception as exc:
            print(f"batch of {len(batch)} FAILED: {exc}", flush=True)
            return {}
    return run


def cached(path: str) -> dict:
    return json.load(open(path)) if os.path.exists(path) else {}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--mode", choices=["episodes", "full"], default="episodes")
    ap.add_argument("--per-type", type=int, default=0, help="stratified sample per question type (0 = all)")
    ap.add_argument("--ids", help="comma-separated question ids")
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--k", type=int, default=30, help="memories (facts) recalled")
    ap.add_argument("--episodes", type=int, default=10, help="transcript excerpts recalled")
    ap.add_argument("--label", default="run")
    ap.add_argument("--retrieval-only", action="store_true")
    ap.add_argument("--parallel", type=int, default=4)
    args = ap.parse_args()

    qs = select(json.load(open(args.data)), args.per_type, args.seed, args.ids)
    brains = os.path.join(args.out, f"brains-{args.mode}")
    ingest = ingest_episodes if args.mode == "episodes" else ingest_full

    def prepare(q):
        root = os.path.join(brains, q["question_id"])
        try:
            ingest(q, root)
            q["ctx"] = recall(q, root, args.k, args.episodes)
            q["ev"] = evidence(q, q["ctx"])
        except Exception as exc:  # one question failing must not lose the rest; a rerun resumes it
            print(f"[{q['question_id']}] FAILED: {exc}", flush=True)
            q["ctx"] = None
        return q

    workers = args.parallel if args.mode == "full" else 1  # episodes mode is CPU-bound and fast
    with ThreadPoolExecutor(workers) as pool:
        qs = [q for q in pool.map(prepare, qs) if q["ctx"] is not None]
    report_retrieval(qs, args)
    if args.retrieval_only:
        return

    acache = os.path.join(args.out, f"answers-{args.label}.json")
    answers = cached(acache)
    todo = [q for q in qs if q["question_id"] not in answers]
    with ThreadPoolExecutor(args.parallel) as pool:
        for res in pool.map(safe(answer_batch), [todo[i:i + 4] for i in range(0, len(todo), 4)]):
            answers.update(res)
            with open(acache, "w") as fh:
                json.dump(answers, fh, indent=1)
    for q in qs:
        q["response"] = answers.get(q["question_id"], "")
    jcache = os.path.join(args.out, f"judged-{args.label}.json")
    judged = cached(jcache)
    todo = [q for q in qs if q["question_id"] not in judged]
    with ThreadPoolExecutor(args.parallel) as pool:
        for res in pool.map(safe(judge_batch), [todo[i:i + 20] for i in range(0, len(todo), 20)]):
            judged.update(res)
            with open(jcache, "w") as fh:
                json.dump(judged, fh, indent=1)
    rows = [{"qid": q["question_id"], "type": q["question_type"], "abs": q["question_id"].endswith("_abs"),
             "question": q["question"], "answer": q["answer"], "response": q["response"],
             "correct": judged.get(q["question_id"], "no").startswith("yes"), "ev": q["ev"]} for q in qs]
    with open(os.path.join(args.out, f"results-{args.label}.json"), "w") as fh:
        json.dump(rows, fh, indent=1, ensure_ascii=False)
    report_qa(rows, args.label)


def report_retrieval(qs, args):
    by = defaultdict(list)
    for q in qs:
        if q["ev"]:
            by[q["question_type"]].append(q["ev"])
            by["overall"].append(q["ev"])
    print(f"\nretrieval ({args.mode}, k={args.k}, episodes={args.episodes}), {len(qs)} questions")
    print("| type | n | evidence session found | all evidence sessions | evidence turns | context tokens |")
    print("|---|---|---|---|---|---|")
    for t in TYPES + ["overall"]:
        g = by[t]
        if g:
            turns = sum(e["turns"] for e in g) / max(1, sum(e["n_turns"] for e in g))
            print(f"| {t} | {len(g)} | {sum(e['sess_any'] for e in g) / len(g):.1%} | "
                  f"{sum(e['sess_all'] for e in g) / len(g):.1%} | {turns:.1%} | "
                  f"{sum(e['ctx'] for e in g) / len(g) / 4:,.0f} |")


def report_qa(rows, label):
    by = defaultdict(list)
    for r in rows:
        by["abstention" if r["abs"] else r["type"]].append(r["correct"])
        by["overall"].append(r["correct"])
    print(f"\n{label}: {len(rows)} questions (official LongMemEval judge prompts)")
    print("| type | n | accuracy |")
    print("|---|---|---|")
    for t in TYPES + ["abstention", "overall"]:
        if by[t]:
            print(f"| {t} | {len(by[t])} | {sum(by[t]) / len(by[t]):.1%} |")
    per_type = [sum(by[t]) / len(by[t]) for t in TYPES if by[t]]
    print(f"| task-averaged | | {sum(per_type) / len(per_type):.1%} |")


if __name__ == "__main__":
    main()
