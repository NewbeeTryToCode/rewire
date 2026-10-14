"""One evaluation runner for rewire: LoCoMo and LongMemEval, answered and judged through OpenRouter.

What it measures (definitions in docs/evaluation.md):
  quality   : J (LLM judge, official prompts), token F1 and BLEU-1 (LoCoMo), abstention (LongMemEval)
  retrieval : evidence recall (turn / session level), no LLM
  cost      : context tokens per question, prompt / completion / reasoning tokens per response and per
              stage (ingest, answer, judge), USD cost, latency (retrieval and LLM, p50 / p95)

Token savers built in:
  - every LLM reply is cached by (model, prompt): a rerun only pays for questions whose retrieved
    context changed (or for a new model);
  - --sample N takes N questions per category (stratified, seeded) instead of the whole set;
  - --retrieval-only and --dry-run never call an LLM (dry-run estimates tokens and cost);
  - --max-cost stops the run when the budget is spent (resume later from the cache).

Usage (OPENROUTER_API_KEY must be set as an environment secret):
  python experiments/eval/run.py --bench locomo --data locomo10.json --out OUT --sample 20 \\
      --answer-model openai/gpt-4o-mini --judge-model openai/gpt-4o-mini --dry-run
  python experiments/eval/run.py --bench longmemeval --data longmemeval_s_cleaned.json --out OUT \\
      --sample 20 --answer-model openai/gpt-4o-mini --judge-model openai/gpt-4o --max-cost 2
"""

import argparse
import hashlib
import importlib.util
import json
import math
import os
import random
import re
import sys
import threading
import time
from collections import Counter, defaultdict
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.join(HERE, "..", "..")
sys.path.insert(0, os.path.join(ROOT, "src"))
sys.path.insert(0, HERE)

from openrouter import BudgetExceeded, Client, JsonProvider, Ledger, model_prices  # noqa: E402
from rewire import ops  # noqa: E402
from rewire.brain import Brain  # noqa: E402
from rewire.consolidate import Consolidator  # noqa: E402
from rewire.index import Index  # noqa: E402


def _load(name: str, path: str):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


# The benchmark-specific pieces (official prompts, ingestion, evidence) live in the existing harnesses.
LOCOMO = _load("locomo_run", os.path.join(ROOT, "experiments", "locomo", "run.py"))
LME = _load("lme_run", os.path.join(ROOT, "experiments", "longmemeval", "run.py"))
LOCOMO_PROMPTS = _load("locomo_prompts", os.path.join(ROOT, "experiments", "locomo", "official_prompts.py"))

CHARS_PER_TOKEN = 4  # estimate for context sizes; LLM calls report exact token counts


def est_tokens(text: str) -> int:
    return math.ceil(len(text) / CHARS_PER_TOKEN)


def pct(values: list[float], q: float) -> float:
    v = sorted(values)
    return v[min(len(v) - 1, int(len(v) * q))] if v else 0.0


def norm(s: str) -> str:
    return re.sub(r"\W+", " ", str(s).lower()).strip()


def bleu1(pred: str, gold: str) -> float:
    """Unigram BLEU with brevity penalty (LoCoMo reports BLEU-1 next to F1)."""
    p, g = norm(pred).split(), norm(gold).split()
    if not p or not g:
        return 0.0
    overlap = sum((Counter(p) & Counter(g)).values())
    bp = 1.0 if len(p) > len(g) else math.exp(1 - len(g) / len(p))
    return bp * overlap / len(p)


def ranking(excerpts: list[str], evidence: list[str]) -> dict:
    """Rank-aware retrieval over the ranked excerpts: reciprocal rank of the first excerpt holding an
    evidence turn (MRR when averaged), and context precision (share of excerpts holding any evidence)."""
    keys = [norm(t)[:50] for t in evidence]
    holds = [any(k and k in norm(e) for k in keys) for e in excerpts]
    first = next((i for i, h in enumerate(holds) if h), None)
    return {"rr": 0.0 if first is None else 1.0 / (first + 1),
            "precision": sum(holds) / len(holds) if holds else 0.0}


def sample(items: list[dict], per_cat: int, seed: int) -> list[dict]:
    if not per_cat:
        return items
    rng = random.Random(seed)
    by = defaultdict(list)
    for it in items:
        by[it["category"]].append(it)
    out = []
    for cat in sorted(by):
        out += rng.sample(by[cat], min(per_cat, len(by[cat])))
    return out


# ---- reply cache ---------------------------------------------------------

class Cache:
    """(stage, model, prompt) -> the LLM call (text + usage). A rerun pays only for new prompts."""

    def __init__(self, path: str):
        self.path, self.data, self._lock = path, {}, threading.Lock()
        if os.path.exists(path):
            for line in open(path, encoding="utf-8"):
                if line.strip():
                    row = json.loads(line)
                    self.data[row["key"]] = row
        os.makedirs(os.path.dirname(path), exist_ok=True)

    @staticmethod
    def key(stage: str, model: str, prompt: str) -> str:
        return hashlib.sha256(f"{stage}\0{model}\0{prompt}".encode()).hexdigest()

    def get(self, key: str) -> dict | None:
        return self.data.get(key)

    def put(self, key: str, call: dict) -> None:
        row = {"key": key, **call}
        with self._lock:
            self.data[key] = row
            with open(self.path, "a", encoding="utf-8") as fh:
                fh.write(json.dumps(row, ensure_ascii=False) + "\n")


def ask(client: Client | None, cache: Cache, stage: str, model: str, prompt: str, max_tokens: int) -> dict:
    """Cached LLM call. Returns the call record plus "cached" (True when nothing was spent now)."""
    key = Cache.key(stage, model, prompt)
    hit = cache.get(key)
    if hit:
        return {**hit, "cached": True}
    call = client.chat(model, [{"role": "user", "content": prompt}], stage=stage, max_tokens=max_tokens)
    cache.put(key, call)
    return {**call, "cached": False}


# ---- LoCoMo --------------------------------------------------------------

def locomo_brain(conv: dict, root: str, mode: str, provider) -> None:
    done = os.path.join(root, ".state", "eval_done")
    if os.path.exists(done):
        return
    if not os.path.exists(os.path.join(root, "brain.toml")):
        Brain.init(root, device="eval")
        toml = os.path.join(root, "brain.toml")
        text = open(toml).read().replace('profile = "coding"', 'profile = "general"')
        open(toml, "w").write(text)
    if mode == "episodes":
        LOCOMO.backfill_transcripts(conv, root)  # transcripts only, no LLM
    else:
        brain = Brain(root)
        con = Consolidator(brain, provider=provider, log=lambda *_: None)
        for key, date, turns in LOCOMO.sessions_of(conv):
            with brain.lock(timeout=600):
                con._consolidate("locomo", {"session": f"{conv['sample_id']}-{key}", "cwd": None, "date": date},
                                 turns)
        LOCOMO.approve_all(root)  # no human reviewer in a benchmark: proposals accepted in bulk
    os.makedirs(os.path.dirname(done), exist_ok=True)
    open(done, "w").close()


def locomo_items(args) -> list[dict]:
    data = json.load(open(args.data))
    if args.convs != "all":
        want = set(args.convs.split(","))
        data = [c for c in data if c["sample_id"] in want]
    items = []
    for conv in data:
        dates = [LOCOMO.session_date(v) for k, v in conv["conversation"].items() if k.endswith("_date_time")]
        turns = {t["dia_id"]: t["text"] for k, v in conv["conversation"].items()
                 if re.fullmatch(r"session_\d+", k) for t in v}
        raw_tokens = est_tokens(" ".join(turns.values()))  # the whole history, as full-context would send it
        for i, qa in enumerate(conv["qa"]):
            if qa.get("category") not in LOCOMO.CATS:  # category 5 (adversarial) excluded, as Mem0 does
                continue
            gold = qa.get("answer", qa.get("adversarial_answer", ""))
            if qa["category"] == 3 and ";" in str(gold):
                gold = str(gold).split(";")[0].strip()
            ev = [turns[e] for raw in qa.get("evidence", []) for e in re.split(r"[;, ]+", raw) if e in turns]
            items.append({"bench": "locomo", "qid": f"{conv['sample_id']}-{i}", "conv": conv,
                          "category": LOCOMO.CATS[qa["category"]], "question": qa["question"], "gold": str(gold),
                          "ref_date": max(d for d in dates if d), "evidence_turns": ev, "abstention": False,
                          "raw_tokens": raw_tokens})
    return items


def locomo_prepare(it: dict, args, provider) -> dict:
    root = os.path.join(args.out, "locomo", f"brains-{args.mode}", it["conv"]["sample_id"])
    locomo_brain(it["conv"], root, args.mode, provider)
    index = Index(Brain(root))
    index.refresh()
    start = time.time()
    mems = LOCOMO.memories_for(index, it["question"], args.k, args.episodes)
    retrieval_ms = (time.time() - start) * 1000
    index.close()
    lines = ["The following memories are presented in chronological order (oldest to newest)."]
    lines += [f"({m['created_at'] or 'unknown date'}) {m['memory']}" for m in sorted(mems, key=lambda m: m["created_at"])]
    context = "\n".join(lines) if mems else "(No relevant memories found)"
    ranked = [m["memory"] for m in mems if m["memory"].startswith("Conversation excerpt")]
    excerpts = norm(" ".join(ranked))
    hits = sum(norm(t)[:50] in excerpts for t in it["evidence_turns"])
    prompt = (LOCOMO_PROMPTS.ANSWER_GENERATION_PROMPT.replace("{reference_date}", it["ref_date"])
              .replace("{memories}", context).replace("{question}", it["question"]))
    return {"prompt": prompt, "context_tokens": est_tokens(context), "retrieval_ms": retrieval_ms,
            "n_memories": len(mems),
            "raw_tokens": it["raw_tokens"],
            "evidence": {"turns": hits, "n_turns": len(it["evidence_turns"]), "all": hits == len(it["evidence_turns"]),
                         **ranking(ranked, it["evidence_turns"])} if it["evidence_turns"] else None}


def locomo_answer(text: str) -> str:
    return text.rsplit("ANSWER:", 1)[1].strip() if "ANSWER:" in text else text.strip()


def locomo_judge_prompt(it: dict, response: str) -> str:
    return (LOCOMO_PROMPTS.JUDGE_PROMPT.replace("{question}", it["question"]).replace("{answer}", it["gold"])
            .replace("{response}", response))


def locomo_verdict(text: str) -> bool:
    m = re.search(r'"label"\s*:\s*"(CORRECT|WRONG)"', text, re.I)
    return (m.group(1) if m else ("WRONG" if "WRONG" in text.upper() else "CORRECT")).upper() == "CORRECT"


# ---- LongMemEval ---------------------------------------------------------

def lme_brain(q: dict, root: str, mode: str, provider) -> None:
    if mode == "episodes":
        LME.ingest_episodes(q, root)  # transcripts only, no LLM
        return
    if os.path.exists(os.path.join(root, ".state", "lme_done")):
        return
    brain = LME.new_brain(root)
    con = Consolidator(brain, provider=provider, log=lambda *_: None)
    for sid, date, session in zip(q["haystack_session_ids"], q["haystack_dates"], q["haystack_sessions"]):
        with brain.lock(timeout=600):
            con._consolidate(LME.HARNESS, {"session": sid, "cwd": None, "date": LME.iso(date)}, LME.turns_of(session))
    for item in list(brain.iter_inbox()):
        try:
            ops.approve(brain, item.meta["inbox_id"])
        except Exception as exc:
            print("approve failed:", exc)
    open(os.path.join(root, ".state", "lme_done"), "w").close()


def lme_items(args) -> list[dict]:
    items = []
    for q in json.load(open(args.data)):
        items.append({"bench": "longmemeval", "qid": q["question_id"], "raw": q, "category": q["question_type"],
                      "question": q["question"], "gold": str(q["answer"]),
                      "abstention": q["question_id"].endswith("_abs"),
                      "raw_tokens": est_tokens(" ".join(t["content"] for s in q["haystack_sessions"] for t in s))})
    return items


def lme_prepare(it: dict, args, provider) -> dict:
    q = it["raw"]
    root = os.path.join(args.out, "longmemeval", f"brains-{args.mode}", q["question_id"])
    lme_brain(q, root, args.mode, provider)
    start = time.time()
    ctx = LME.recall(q, root, args.k, args.episodes)
    retrieval_ms = (time.time() - start) * 1000
    history = LME.history_string(ctx)
    ev = LME.evidence(q, ctx)
    evidence_turns = [t["content"] for s in q["haystack_sessions"] for t in s if t.get("has_answer")]
    return {"prompt": LME.ANSWER_TEMPLATE.format(history, q["question_date"], q["question"]),
            "context_tokens": est_tokens(history), "retrieval_ms": retrieval_ms, "raw_tokens": it["raw_tokens"],
            "n_memories": len(ctx["facts"]) + len(ctx["episodes"]),
            "evidence": {"turns": ev["turns"], "n_turns": ev["n_turns"], "all": ev["sess_all"],
                         "session_any": ev["sess_any"], **ranking([e["text"] for e in ctx["episodes"]], evidence_turns)}
            if ev else None}


def lme_judge_prompt(it: dict, response: str) -> str:
    return LME.judge_prompt({**it["raw"], "response": response})


def lme_verdict(text: str) -> bool:
    """The official judge answers "yes" or "no" only."""
    return text.strip().lower().lstrip("*\"' ").startswith("yes")


BENCH = {
    "locomo": {"items": locomo_items, "prepare": locomo_prepare, "answer": locomo_answer,
               "judge_prompt": locomo_judge_prompt, "verdict": locomo_verdict},
    "longmemeval": {"items": lme_items, "prepare": lme_prepare, "answer": lambda t: t.strip(),
                    "judge_prompt": lme_judge_prompt, "verdict": lme_verdict},
}


# ---- report --------------------------------------------------------------

def summarize(rows: list[dict], ledger: Ledger, args, est: dict | None) -> dict:
    by = defaultdict(list)
    for r in rows:
        by[r["category"]].append(r)
        by["overall"].append(r)

    def group(g: list[dict]) -> dict:
        judged = [r for r in g if r.get("correct") is not None]
        ev = [r["evidence"] for r in g if r.get("evidence")]
        out = {"n": len(g),
               "J": sum(r["correct"] for r in judged) / len(judged) if judged else None,
               "evidence_turn_recall": sum(e["turns"] for e in ev) / max(1, sum(e["n_turns"] for e in ev)) if ev else None,
               "evidence_all": sum(e["all"] for e in ev) / len(ev) if ev else None,
               "context_tokens_mean": sum(r["context_tokens"] for r in g) / len(g),
               "context_tokens_p95": pct([r["context_tokens"] for r in g], 0.95),
               "retrieval_ms_p50": pct([r["retrieval_ms"] for r in g], 0.5),
               "retrieval_ms_p95": pct([r["retrieval_ms"] for r in g], 0.95),
               "retrieval_ms_p99": pct([r["retrieval_ms"] for r in g], 0.99),
               "MRR": sum(e["rr"] for e in ev) / len(ev) if ev else None,
               "context_precision": sum(e["precision"] for e in ev) / len(ev) if ev else None,
               "compression": sum(r["raw_tokens"] for r in g) / max(1, sum(r["context_tokens"] for r in g))}
        if args.bench == "locomo" and judged:
            out["F1"] = sum(r["f1"] for r in judged) / len(judged)
            out["BLEU1"] = sum(r["bleu1"] for r in judged) / len(judged)
        answered = [r for r in g if r.get("answer_call")]
        if answered:
            out["answer_prompt_tokens_per_response"] = sum(r["answer_call"]["prompt_tokens"] for r in answered) / len(answered)
            out["answer_completion_tokens_per_response"] = sum(r["answer_call"]["completion_tokens"] for r in answered) / len(answered)
            out["answer_words_mean"] = sum(len(r["response"].split()) for r in answered) / len(answered)
        return out

    summary = {"bench": args.bench, "mode": args.mode, "k": args.k, "episodes": args.episodes,
               "answer_model": args.answer_model, "judge_model": args.judge_model,
               "groups": {name: group(g) for name, g in by.items()},
               "llm_spent_this_run": ledger.summary()}
    abst = [r for r in rows if r["abstention"] and r.get("correct") is not None]
    if abst:
        summary["abstention_accuracy"] = sum(r["correct"] for r in abst) / len(abst)
    all_calls = [r[c] for r in rows for c in ("answer_call", "judge_call") if r.get(c)]
    if all_calls:  # per-response totals including cached replies (what the full run costs from scratch)
        summary["tokens_per_question_incl_cached"] = {
            "prompt": sum(c["prompt_tokens"] for c in all_calls) / len(rows),
            "completion": sum(c["completion_tokens"] for c in all_calls) / len(rows),
            "cost_usd": sum(c.get("cost") or 0 for c in all_calls) / len(rows)}
    if est:
        summary["dry_run_estimate"] = est
    return summary


def write_report(summary: dict, path: str) -> None:
    f = lambda v, p=False: "-" if v is None else (f"{v:.1%}" if p else (f"{v:,.0f}" if v >= 10 else f"{v:.3f}"))  # noqa: E731
    lines = [f"# {summary['bench']} · mode {summary['mode']} · k={summary['k']} · episodes={summary['episodes']}",
             f"answer model `{summary['answer_model']}` · judge model `{summary['judge_model']}`", "",
             "| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |",
             "|---|---|---|---|---|---|---|---|---|---|"]
    for name, g in sorted(summary["groups"].items(), key=lambda x: (x[0] == "overall", x[0])):
        lines.append(
            f"| {name} | {g['n']} | {f(g['J'], True)} | {f(g.get('F1'))} | {f(g.get('BLEU1'))} | "
            f"{f(g['evidence_turn_recall'], True)} | {f(g['evidence_all'], True)} | "
            f"{f(g['context_tokens_mean'])} / {f(g['context_tokens_p95'])} | "
            f"{f(g.get('answer_prompt_tokens_per_response'))} / {f(g.get('answer_completion_tokens_per_response'))} | "
            f"{g['retrieval_ms_p50']:.0f} / {g['retrieval_ms_p95']:.0f} |")
    lines += ["", "| category | MRR (first evidence excerpt) | context precision | compression vs full history | retrieval ms p99 |",
              "|---|---|---|---|---|"]
    for name, g in sorted(summary["groups"].items(), key=lambda x: (x[0] == "overall", x[0])):
        lines.append(f"| {name} | {f(g['MRR'])} | {f(g['context_precision'], True)} | {g['compression']:.1f}× | "
                     f"{g['retrieval_ms_p99']:.0f} |")
    if "abstention_accuracy" in summary:
        lines += ["", f"Abstention accuracy: {summary['abstention_accuracy']:.1%}"]
    lines += ["", "## LLM usage this run (cached replies cost nothing)", "",
              "| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |",
              "|---|---|---|---|---|---|---|---|"]
    for stage, s in summary["llm_spent_this_run"].items():
        lines.append(f"| {stage} | {s['calls']} | {s['prompt_tokens']:,} | {s['completion_tokens']:,} | "
                     f"{s['reasoning_tokens']:,} | {s['prompt_tokens_per_call']:,.0f} / {s['completion_tokens_per_call']:,.0f} | "
                     f"{s['cost']:.4f} | {s['latency_p50_ms']} / {s['latency_p95_ms']} |")
    if "tokens_per_question_incl_cached" in summary:
        t = summary["tokens_per_question_incl_cached"]
        lines += ["", f"Per question, answer + judge, including cached replies: {t['prompt']:,.0f} prompt tokens, "
                      f"{t['completion']:,.0f} completion tokens, ${t['cost_usd']:.5f}."]
    if "dry_run_estimate" in summary:
        e = summary["dry_run_estimate"]
        lines += ["", "## Dry-run estimate (no LLM called)", "", "```", json.dumps(e, indent=1), "```"]
    lines += ["", "Published scores of other memory systems, and why they are not directly comparable: "
                  "docs/evaluation.md."]
    with open(path, "w", encoding="utf-8") as fh:
        fh.write("\n".join(lines) + "\n")


# ---- driver --------------------------------------------------------------

def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--bench", choices=list(BENCH), required=True)
    ap.add_argument("--data", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--mode", choices=["episodes", "full"], default="episodes",
                    help="episodes: transcripts only (no LLM ingest); full: real consolidation through --ingest-model")
    ap.add_argument("--sample", type=int, default=20, help="questions per category (0 = all)")
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--convs", default="all", help="LoCoMo conversations (comma-separated sample ids or all)")
    ap.add_argument("--k", type=int, default=30, help="memories (facts) recalled")
    ap.add_argument("--episodes", type=int, default=10, help="transcript excerpts recalled")
    ap.add_argument("--answer-model", default="openai/gpt-4o-mini")
    ap.add_argument("--judge-model", default="openai/gpt-4o-mini")
    ap.add_argument("--ingest-model", default="openai/gpt-4o-mini", help="consolidation model for --mode full")
    ap.add_argument("--answer-max-tokens", type=int, default=1500)
    ap.add_argument("--judge-max-tokens", type=int, default=200)
    ap.add_argument("--parallel", type=int, default=8)
    ap.add_argument("--max-cost", type=float, help="stop when this many USD were spent in this run")
    ap.add_argument("--retrieval-only", action="store_true", help="no LLM at all: evidence recall + context size")
    ap.add_argument("--dry-run", action="store_true", help="no LLM at all: estimate tokens and cost of the run")
    ap.add_argument("--est-answer-out", type=int, default=400, help="dry-run: expected answer completion tokens")
    ap.add_argument("--est-judge-out", type=int, default=60, help="dry-run: expected judge completion tokens")
    ap.add_argument("--price", action="append", default=[], metavar="MODEL=IN,OUT",
                    help="USD per 1M tokens for the dry-run when the catalog is unreachable")
    ap.add_argument("--label", default="run")
    args = ap.parse_args()

    no_llm = args.retrieval_only or args.dry_run
    if args.mode == "full" and no_llm:
        sys.exit("--mode full needs the LLM for ingestion; use --mode episodes with --dry-run/--retrieval-only")
    ledger = Ledger(args.max_cost)
    client = None if no_llm else Client(ledger)
    provider = JsonProvider(client, args.ingest_model) if client else None
    bench = BENCH[args.bench]
    os.makedirs(args.out, exist_ok=True)
    cache = Cache(os.path.join(args.out, "cache", "llm.jsonl"))

    items = sample(bench["items"](args), args.sample, args.seed)
    print(f"{args.bench}: {len(items)} questions ({args.sample or 'all'} per category), mode {args.mode}", flush=True)

    def run_one(it: dict) -> dict:
        prep = bench["prepare"](it, args, provider)
        row = {k: it[k] for k in ("bench", "qid", "category", "question", "gold", "abstention")}
        row.update({k: prep[k] for k in ("context_tokens", "raw_tokens", "retrieval_ms", "n_memories", "evidence")})
        row["answer_prompt_tokens_est"] = est_tokens(prep["prompt"])
        if no_llm:
            return row
        a = ask(client, cache, "answer", args.answer_model, prep["prompt"], args.answer_max_tokens)
        row["response"] = bench["answer"](a["text"])
        row["answer_call"] = {k: a[k] for k in ("prompt_tokens", "completion_tokens", "reasoning_tokens", "cost",
                                                "latency_ms", "cached")}
        j = ask(client, cache, "judge", args.judge_model, bench["judge_prompt"](it, row["response"]),
                args.judge_max_tokens)
        row["correct"] = bench["verdict"](j["text"])
        row["judge_call"] = {k: j[k] for k in ("prompt_tokens", "completion_tokens", "reasoning_tokens", "cost",
                                               "latency_ms", "cached")}
        if args.bench == "locomo":
            row["f1"] = LOCOMO.f1(row["response"], row["gold"])
            row["bleu1"] = bleu1(row["response"], row["gold"])
        return row

    rows, stopped = [], None
    # one brain per LoCoMo conversation: ingest it once before questions run in parallel
    if args.bench == "locomo":
        for conv in {it["conv"]["sample_id"]: it["conv"] for it in items}.values():
            locomo_brain(conv, os.path.join(args.out, "locomo", f"brains-{args.mode}", conv["sample_id"]),
                         args.mode, provider)
    with ThreadPoolExecutor(args.parallel) as pool:
        futures = [pool.submit(run_one, it) for it in items]
        for fut in futures:
            try:
                rows.append(fut.result())
            except BudgetExceeded as exc:
                stopped = str(exc)
            except Exception as exc:  # one question failing must not lose the others; rerun resumes it
                print(f"question failed: {exc}", flush=True)
    if stopped:
        print(f"STOPPED: {stopped}; rerun to resume from the cache", flush=True)

    est = None
    if args.dry_run:
        prices = {}
        for p in args.price:
            model, vals = p.split("=", 1)
            pin, pout = (float(x) for x in vals.split(","))
            prices[model] = (pin, pout)
        if not prices:
            try:
                prices = model_prices(Client(ledger, api_key="-"))
            except Exception as exc:
                print(f"price catalog unreachable ({exc}); pass --price MODEL=IN,OUT for a cost estimate")
        ans_in = sum(r["answer_prompt_tokens_est"] for r in rows)
        judge_in = len(rows) * 400  # judge prompt (rules + question + gold + answer), roughly
        est = {"questions": len(rows), "answer_prompt_tokens": ans_in,
               "answer_completion_tokens": len(rows) * args.est_answer_out,
               "judge_prompt_tokens": judge_in, "judge_completion_tokens": len(rows) * args.est_judge_out}
        for stage, model, tin, tout in (("answer", args.answer_model, ans_in, len(rows) * args.est_answer_out),
                                        ("judge", args.judge_model, judge_in, len(rows) * args.est_judge_out)):
            if model in prices:
                est[f"{stage}_cost_usd"] = round((tin * prices[model][0] + tout * prices[model][1]) / 1e6, 4)

    summary = summarize(rows, ledger, args, est)
    tag = f"{args.bench}-{args.label}"
    json.dump({"summary": summary, "rows": rows}, open(os.path.join(args.out, f"results-{tag}.json"), "w"),
              indent=1, ensure_ascii=False, default=str)
    write_report(summary, os.path.join(args.out, f"report-{tag}.md"))
    print(open(os.path.join(args.out, f"report-{tag}.md")).read())


if __name__ == "__main__":
    main()
