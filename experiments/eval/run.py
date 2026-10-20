"""One evaluation runner for rewire: LoCoMo, LongMemEval, BEAM and PersonaMem, answered and judged through
OpenRouter (or the local Claude CLI). rewire runs as its Rust binary (rewire_cli.py); build it first with
`cargo build --release --manifest-path rust/Cargo.toml`.

What it measures (definitions in docs/evaluation.md):
  quality   : J (LLM judge, official prompts), token F1 and BLEU-1 (LoCoMo), abstention (LongMemEval)
  retrieval : evidence recall (turn / session level), no LLM
  cost      : context tokens per question (GPT-4o tokenizer when available, mean / median / p95), prompt /
              completion / reasoning tokens per response and per stage (ingest, expand, answer, judge), USD cost,
              latency (recall net of process start, LLM calls, and end to end per question, p50 / p95)
  baseline  : --mode fullctx sends the whole history (up to the question) instead of recalled memory

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
import glob
import hashlib
import json
import math
import os
import platform
import random
import shlex
import string
import re
import subprocess
import sys
import threading
from collections import Counter, defaultdict
from datetime import datetime, timezone
from concurrent.futures import ThreadPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.normpath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, HERE)

import beam_prompts as BEAM_PROMPTS  # noqa: E402
import lme_prompts as LME_PROMPTS  # noqa: E402
import locomo_prompts as LOCOMO_PROMPTS  # noqa: E402
import rewire_cli as rw  # noqa: E402
import tokens  # noqa: E402
from openrouter import BudgetExceeded, Client, Ledger, model_prices, normalize_model  # noqa: E402


def count_tokens(text: str) -> int:
    """Context and prompt sizes: GPT-4o tokenizer when available (tokens.name()), else characters / 4."""
    return tokens.count(text)


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


REPLICATE = 0


def ask(client: Client | None, cache: Cache, stage: str, model: str, prompt: str, max_tokens: int) -> dict:
    """Cached LLM call. Returns the call record plus "cached" (True when nothing was spent now)."""
    # --replicate N: answers and verdicts are drawn afresh (own cache entries); extra queries stay shared, so the
    # recalled context is the same in every replicate and only answer/judge sampling varies
    key = Cache.key(f"{stage}#{REPLICATE}" if REPLICATE and stage != "expand" else stage, model, prompt)
    hit = cache.get(key)
    # a reply cut off by max_tokens is reused only if the new limit is no larger than the one that cut it
    if hit and not (hit.get("finish_reason") == "length" and max_tokens > hit.get("max_tokens", 0)):
        return {**hit, "cached": True}
    call = client.chat(model, [{"role": "user", "content": prompt}], stage=stage, max_tokens=max_tokens)
    cache.put(key, {**call, "max_tokens": max_tokens})
    return {**call, "cached": False}


# ---- clock ---------------------------------------------------------------
# Recall ranks by activation, which decays with the time since each memory was written. Questions answered hours
# or days after ingest would see different rankings (and miss the answer cache), so each question is recalled as
# of the moment its brain finished ingesting (REWIRE_NOW, read by the rewire binary).

def frozen_now(root: str) -> str:
    """When the brain at root finished ingesting: the time written in eval_done, else its mtime (RFC 3339)."""
    done = os.path.join(root, ".state", "eval_done")
    text = open(done).read().strip() if os.path.exists(done) else ""
    at = datetime.fromisoformat(text) if text else datetime.fromtimestamp(os.path.getmtime(done), timezone.utc)
    return at.astimezone(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def mark_done(root: str) -> None:
    done = os.path.join(root, ".state", "eval_done")
    os.makedirs(os.path.dirname(done), exist_ok=True)
    with open(done, "w") as fh:  # the brain's clock for recall (frozen_now)
        fh.write(datetime.now(timezone.utc).isoformat())


# ---- ingest (shared) -------------------------------------------------------
# --mode episodes: transcripts only, no LLM (`rewire import`). --mode full: every session consolidated in order
# (`rewire import --llm`), through llm_cmd.py so every ingest call is logged with its tokens; the inbox is then
# accepted in bulk (no reviewer in a benchmark).

INGEST_COMMAND = None  # set in main() for --mode full


def ingest(root: str, sessions: list[dict], mode: str) -> None:
    """Build the brain at root from sessions {harness, session, date, turns}; resumes a stopped full ingest."""
    if os.path.exists(os.path.join(root, ".state", "eval_done")):
        return
    command = None
    if mode == "full":
        command = INGEST_COMMAND.replace("{ledger}", os.path.join(os.path.abspath(root), ".state", "ingest_calls.jsonl"))
    rw.init(root, command=command)
    if mode == "episodes":
        rw.import_sessions(root, sessions)
    else:
        log = os.path.join(root, ".state", "eval_ingest.jsonl")
        os.makedirs(os.path.dirname(log), exist_ok=True)
        done_sessions = {r["session"] for r in _jsonl(log) if r.get("ok")}  # a stopped ingest resumes here
        todo = [x for x in sessions if x["session"] not in done_sessions]
        records = rw.import_sessions(root, todo, llm=True)
        with open(log, "a", encoding="utf-8") as fh:
            for rec in records:
                fh.write(json.dumps(rec) + "\n")
        failed = [r for r in records if not r.get("ok")]
        if failed or len(records) < len(todo):
            raise RuntimeError(f"ingest stopped at {failed[0]['session'] if failed else '?'}: "
                               f"{failed[0].get('error', '') if failed else 'no record'}")
        rw.approve_all(root)
    mark_done(root)


def _jsonl(path: str) -> list[dict]:
    if not os.path.exists(path):
        return []
    with open(path, encoding="utf-8") as fh:
        return [json.loads(line) for line in fh if line.strip()]


def ingest_stats(roots: list[str]) -> dict | None:
    """What --mode full consolidation produced and cost, over the given brains."""
    recs = [r for root in roots for r in _jsonl(os.path.join(root, ".state", "eval_ingest.jsonl"))]
    if not recs:
        return None
    ok = [r for r in recs if r.get("ok")]
    calls = [c for root in roots for c in _jsonl(os.path.join(root, ".state", "ingest_calls.jsonl"))]
    types, questions = Counter(), 0
    for root in roots:
        types.update(rw.active_node_types(root))
        for path in glob.glob(os.path.join(root, "sessions", "*", "*.md")):
            with open(path, encoding="utf-8") as fh:
                part = fh.read()
            m = re.search(r"\n## Questions\n(.*?)(?:\n## |\Z)", part, re.S)
            questions += len(re.findall(r"(?m)^- \[\d+\] ", m.group(1))) if m else 0
    turns = sum(r["turns"] for r in ok)
    n = max(1, len(ok))
    return {"sessions": len(ok), "failed_attempts": sum(not r.get("ok") for r in recs),
            "bad_json_replies": sum(not c.get("json", True) for c in calls),
            "applied": sum(r.get("applied", 0) for r in ok), "queued_then_approved": sum(r.get("queued", 0) for r in ok),
            "rejected": sum(r.get("rejected", 0) for r in ok), "nodes_by_type": dict(types.most_common()),
            "facts_per_session": sum(r.get("applied", 0) + r.get("queued", 0) for r in ok) / n,
            "questions": questions, "questions_per_turn": questions / max(1, turns),
            "seconds_per_session_p50": pct([r["seconds"] for r in ok], 0.5),
            "seconds_per_session_p95": pct([r["seconds"] for r in ok], 0.95),
            "seconds_total": sum(r["seconds"] for r in ok),
            "llm_calls_per_session": sum(r.get("llm_calls", 0) for r in ok) / n,
            # exact provider counts from llm_cmd.py's log (absent for brains ingested before it existed)
            "llm_calls_logged": len(calls),
            "prompt_tokens_per_session": sum(c.get("prompt_tokens") or 0 for c in calls) / n if calls else None,
            "completion_tokens_per_session": sum(c.get("completion_tokens") or 0 for c in calls) / n if calls else None,
            "cost_usd": sum(c.get("cost") or 0 for c in calls) if calls else None,
            "prompt_chars_per_session": sum(r.get("prompt_chars", 0) for r in ok) / n}


# ---- LoCoMo --------------------------------------------------------------

LOCOMO_CATS = {1: "multi-hop", 2: "temporal", 3: "open-domain", 4: "single-hop"}  # category 5 excluded, as Mem0 does


def locomo_session_date(raw: str) -> str:
    """'1:56 pm on 8 May, 2023' -> '2023-05-08'."""
    m = re.search(r"(\d{1,2}) (\w+),? (\d{4})", raw)
    return datetime.strptime(" ".join(m.groups()), "%d %B %Y").strftime("%Y-%m-%d") if m else ""


def locomo_sessions(conv: dict) -> list[dict]:
    c = conv["conversation"]
    keys = sorted((k for k in c if re.fullmatch(r"session_\d+", k)), key=lambda k: int(k.split("_")[1]))
    out = []
    for k in keys:
        turns = []
        for i, t in enumerate(c[k], 1):
            text = t["text"]
            if t.get("blip_caption"):
                text += f" [shares a photo: {t['blip_caption']}]"
            turns.append({"n": i, "role": "user", "text": f"{t['speaker']}: {text}", "ts": ""})
        out.append({"harness": "locomo", "session": f"{conv['sample_id']}-{k}",
                    "date": locomo_session_date(c.get(f"{k}_date_time", "")), "turns": turns})
    return out


def locomo_brain(conv: dict, root: str, mode: str) -> None:
    ingest(root, locomo_sessions(conv), mode)


def f1(pred: str, gold: str) -> float:
    """LoCoMo's token F1 (punctuation and articles removed)."""
    def toks(s):
        s = "".join(ch for ch in str(s).lower() if ch not in string.punctuation)
        return [w for w in s.split() if w not in ("a", "an", "the", "and")]
    p, g = toks(pred), toks(gold)
    common = sum((Counter(p) & Counter(g)).values())
    if not common:
        return 0.0
    prec, rec = common / len(p), common / len(g)
    return 2 * prec * rec / (prec + rec)


def locomo_items(args) -> list[dict]:
    data = json.load(open(args.data))
    if args.convs != "all":
        want = set(args.convs.split(","))
        data = [c for c in data if c["sample_id"] in want]
    errata = json.load(open(os.path.join(HERE, "locomo_errata.json")))["questions"]
    items = []
    for conv in data:
        dates = [locomo_session_date(v) for k, v in conv["conversation"].items() if k.endswith("_date_time")]
        turns = {t["dia_id"]: t["text"] for k, v in conv["conversation"].items()
                 if re.fullmatch(r"session_\d+", k) for t in v}
        raw_tokens = count_tokens(" ".join(turns.values()))  # the whole history, as full-context would send it
        for i, qa in enumerate(conv["qa"]):
            if qa.get("category") not in LOCOMO_CATS:  # category 5 (adversarial) excluded, as Mem0 does
                continue
            gold = qa.get("answer", qa.get("adversarial_answer", ""))
            if qa["category"] == 3 and ";" in str(gold):
                gold = str(gold).split(";")[0].strip()
            ev_ids = [e for raw in qa.get("evidence", []) for e in re.split(r"[;, ]+", raw) if e in turns]
            ev = [turns[e] for e in ev_ids]
            items.append({"bench": "locomo", "qid": f"{conv['sample_id']}-{i}", "conv": conv,
                          "category": LOCOMO_CATS[qa["category"]], "question": qa["question"], "gold": str(gold),
                          "ref_date": max(d for d in dates if d), "evidence_turns": ev, "evidence_ids": ev_ids,
                          "abstention": False, "errata": errata.get(f"{conv['sample_id']}-{i}"),
                          "raw_tokens": raw_tokens})
    return items


OVERHEAD_MS = 0.0  # median wall time of a CLI recall on an empty brain (process start, model load); set in main()
_rebuilt: set[str] = set()
_rebuilt_lock = threading.Lock()


def timed_recall(root: str, query: str, k: int, episodes: int, also=None, rebuild: bool = False) -> tuple[list[dict], float, float]:
    """rewire recall as of the brain's ingest time -> (hits, net ms, wall ms). The index is built (and, for brains
    with memory nodes, activation recomputed as of that time) once per brain, outside the timed call."""
    now = frozen_now(root)
    with _ROOT_LOCKS_GUARD:
        lock = _ROOT_LOCKS[root]
    with lock:  # questions on the same history: one builds the index, the others wait
        with _rebuilt_lock:
            first = root not in _rebuilt
            _rebuilt.add(root)
        if first:
            if rebuild:
                rw.run(root, "rebuild", now=now)
            rw.warm_up(root, now=now)
    hits, wall = rw.recall(root, query, k, episodes, also=also, now=now)
    return hits, max(0.0, wall - OVERHEAD_MS), wall


def locomo_memories(hits: list[dict]) -> list[dict]:
    """rewire's recall hits as Mem0-style memory rows (what the official answer prompt expects)."""
    out = []
    for h in hits:
        if h["type"] == "episode":
            out.append({"memory": "Conversation excerpt: " + " / ".join(h["text"].splitlines()), "created_at": h["date"]})
            continue
        body = (h.get("body") or "").strip()
        m = re.search(r"session_(\d+) · turn (\d+)", str(h.get("source") or ""))
        out.append({"memory": f"{h['title']}: {body}" if body and body != h["title"] else h["title"],
                    "created_at": h.get("valid_from") or str(h.get("recorded_at") or "")[:10],
                    "source": f"D{m.group(1)}:{m.group(2)}" if m else None})  # the LoCoMo turn (dia_id) it came from
    return out


def locomo_full_history(conv: dict) -> str:
    """--mode fullctx: the whole conversation, session by session, in place of recalled memories."""
    return "\n\n".join(f"Session on {s['date'] or 'unknown date'}:\n" + "\n".join(t["text"] for t in s["turns"])
                       for s in locomo_sessions(conv))


def locomo_prepare(it: dict, args) -> dict:
    if args.mode == "fullctx":
        context = locomo_full_history(it["conv"])
        mems, ms, wall, root = [], 0.0, 0.0, None
    else:
        root = os.path.join(args.out, "locomo", f"brains-{args.mode}", it["conv"]["sample_id"])
        locomo_brain(it["conv"], root, args.mode)
        hits, ms, wall = timed_recall(root, it["question"], args.k, args.episodes, also=it.get("also"),
                                      rebuild=args.mode == "full")
        mems = locomo_memories(hits)
        lines = ["The following memories are presented in chronological order (oldest to newest)."]
        lines += [f"({m['created_at'] or 'unknown date'}) {m['memory']}" for m in sorted(mems, key=lambda m: m["created_at"])]
        context = "\n".join(lines) if mems else "(No relevant memories found)"
    ranked = [m["memory"] for m in mems if m["memory"].startswith("Conversation excerpt")]
    excerpts = norm(" ".join(ranked) if args.mode != "fullctx" else context)
    found = [norm(t)[:50] in excerpts for t in it["evidence_turns"]]
    hits_n = sum(found)
    # a turn also counts when a fact in the context was extracted from it (mode full)
    sources = {m.get("source") for m in mems if m.get("source")}
    with_facts = sum(f or i in sources for f, i in zip(found, it["evidence_ids"]))
    prompt = (LOCOMO_PROMPTS.ANSWER_GENERATION_PROMPT.replace("{reference_date}", it["ref_date"])
              .replace("{memories}", context).replace("{question}", it["question"]))
    return {"prompt": prompt, "context": context, "context_tokens": count_tokens(context), "retrieval_ms": ms,
            "retrieval_wall_ms": wall, "brain": root, "n_memories": len(mems),
            "raw_tokens": it["raw_tokens"],
            "evidence": {"turns": hits_n, "n_turns": len(it["evidence_turns"]), "all": hits_n == len(it["evidence_turns"]),
                         "turns_with_facts": with_facts, "all_with_facts": with_facts == len(it["evidence_turns"]),
                         "ids": it["evidence_ids"],  # per turn: found as an excerpt, as a fact, or missing
                         "via": ["excerpt" if f else "fact" if i in sources else "missing"
                                 for f, i in zip(found, it["evidence_ids"])],
                         **ranking(ranked, it["evidence_turns"])} if it["evidence_turns"] else None}


def locomo_answer(text: str) -> str:
    """The final answer after the last "ANSWER:" (any case, markdown such as **Answer:** or "Final answer:").
    Without the marker the whole reply is kept, reasoning included, which F1 then penalises."""
    parts = re.split(r"(?i)\**\s*(?:final\s+)?answer\s*\**\s*:\s*\**", text)
    return (parts[-1] if len(parts) > 1 else text).strip().strip("*").strip()


def locomo_judge_prompt(it: dict, response: str) -> str:
    return (LOCOMO_PROMPTS.JUDGE_PROMPT.replace("{question}", it["question"]).replace("{answer}", it["gold"])
            .replace("{response}", response))


def locomo_verdict(text: str) -> bool | None:
    """True / False from the judge's label; None when the reply has no label (counted as wrong, and reported)."""
    m = re.search(r'"label"\s*:\s*"(CORRECT|WRONG)"', text, re.I)
    if m:
        return m.group(1).upper() == "CORRECT"
    up = text.upper()
    if ("WRONG" in up) != ("CORRECT" in up):
        return "CORRECT" in up
    return None


EXPAND_PROMPT = """You search a person's long chat history with a keyword search engine (BM25: it matches words, not meanings).
Question: {question}

The words in the question may not appear in the messages that answer it. A question about a category ("desserts",
"cities", "activities", "pets") is answered by messages that name specific instances ("ice cream", "Tampa",
"pottery", "dog"). Write 4 short keyword queries (3-8 words each) with the specific words such messages are likely
to contain: likely instances, synonyms, and related verbs. Keep the person's name out unless it is needed.

Reply with one line of JSON and nothing else: {"queries": ["...", "...", "...", "..."]}"""


# Questions whose evidence is spread over several memories or that hinge on when things happened: extra queries
# brought their evidence (LongMemEval all evidence 91% -> 98%) but hurt preference and advice questions in every
# sampled benchmark (2026-10-17-ab-four-benchmarks.md).
TIME_QUESTION = re.compile(r"(?i)\b(when|how long|ago|before|after|since|until|earliest|latest|first|last|most recent|"
                           r"what (date|day|time|year|month))\b")


# Only questions: PersonaMem "questions" are often the user's statements ("During my trip last weekend, I ..."),
# whose time words are narrative; expanding them hurt 3 of 3 (2026-10-17 A/B, variant C).
QUESTION_FORM = re.compile(r"\?\s*$|^\s*(how|what|when|which|who|where|why|did|do|does|is|are|was|were|can|could|have|has)\b",
                           re.I)


# rewire's own rule for counting / total / ordering questions (rust/src/text.rs `aggregate`), which recall also
# uses to widen the excerpt budget
AGGREGATE = re.compile(r"(?i)\b(how many|in total|total|altogether|combined|sum of|all (the|my)|each of|how often|"
                       r"in what order|order of|which\b.{0,80}\b(first|last|most|least|earliest|latest|more|fewer))\b")


def wants_expansion(question: str) -> bool:
    q = question.strip()
    return bool(QUESTION_FORM.search(q)) and (bool(AGGREGATE.search(q)) or bool(TIME_QUESTION.search(q)))


GATED_EXPAND_PROMPT = """You search a person's long chat history with a keyword search engine (BM25: it matches words, not meanings).
Question: {question}

First decide whether extra queries would help. They help when the answer is spread over several messages or hinges
on when things happened: counts, totals, lists of things, comparisons, the order of events, "when" or "how long ago"
questions, and questions naming a category ("activities", "books", "places") whose answer names specific instances.
They do not help, and pull in stale or off-topic messages, when the question asks for one specific fact that its own
words will find, asks for advice, a recommendation or an opinion, or is not a question but the person telling something.

If they would not help, reply {"queries": []}. Otherwise write 4 short keyword queries (3-8 words each) with the
specific words the answering messages are likely to contain: likely instances, synonyms, and related verbs, one query
per item when several things are counted. Keep the person's name out unless it is needed.

Reply with one line of JSON and nothing else: {"queries": [...]}"""


def expand_prompt(question: str, gated: bool = False) -> str:
    if gated:  # the model decides whether the question needs extra queries at all
        return GATED_EXPAND_PROMPT.replace("{question}", question)
    return EXPAND_PROMPT.replace("{question}", question)


def expand_queries(text: str) -> list[str]:
    """The extra queries from the expansion reply (none when it is unreadable)."""
    m = re.search(r"\{.*\}", text, re.S)
    try:
        return [q for q in json.loads(m.group(0))["queries"] if isinstance(q, str)][:4]
    except (AttributeError, ValueError, KeyError, TypeError):
        return []


LIST_JUDGE_PROMPT = """The gold answer to a question is a list. Split the gold answer into its items, then count how
many of them the response names. A paraphrase, a synonym or a more specific term counts as named; a vaguer term,
a different item or an item only hinted at does not. Extra items in the response do not matter.

Question: {question}
Gold answer: {gold}
Response: {response}

Reply with one line of JSON and nothing else: {"items": <number of gold items>, "named": <number named>}"""


def list_judge_prompt(it: dict, response: str) -> str:
    return (LIST_JUDGE_PROMPT.replace("{question}", it["question"]).replace("{gold}", it["gold"])
            .replace("{response}", response))


def list_coverage(text: str) -> float | None:
    """Share of the gold list the response names, from the list judge's reply; None when unreadable."""
    m = re.search(r'"items"\s*:\s*(\d+)\s*,\s*"named"\s*:\s*(\d+)', text)
    if not m or int(m.group(1)) == 0:
        return None
    return min(1.0, int(m.group(2)) / int(m.group(1)))


def gold_items(gold: str) -> list[str]:
    """The items of a list answer ("a, b and c"); one item for anything else."""
    parts = [p for p in re.split(r",|;|\band\b", str(gold)) if norm(p)]
    # a list is short items without numbers; dates ("24 June, 2022") and sentences stay one item, including
    # short reasons and purposes ("because it relaxes and calms him", "to help others and grow")
    is_list = (len(parts) > 1 and not re.search(r"\d", str(gold)) and all(len(norm(p).split()) <= 4 for p in parts)
               and not re.match(r"(?i)\s*(because|since|so|to|by|for|that|when|if)\b", str(gold)))
    return parts if is_list else [str(gold)]


def item_coverage(gold: str, response: str) -> float:
    """Share of the gold items whose content words all appear in the response (no LLM). The official judge
    accepts a list answer when one item is right; this shows how much of the list was found."""
    have = set(norm(response).split())
    items = [set(norm(i).split()) for i in gold_items(gold)]
    return sum(bool(i) and i <= have for i in items) / len(items)


# ---- LongMemEval ---------------------------------------------------------
# Every question has its own chat history (~50 sessions, ~115k tokens in _S), so every question gets its own brain.

LME_HARNESS = "lme"


def lme_iso(date: str) -> str:
    """'2023/05/20 (Sat) 02:21' -> '2023-05-20'."""
    m = re.match(r"(\d{4})/(\d{2})/(\d{2})", date or "")
    return "-".join(m.groups()) if m else ""


def lme_turns(session: list[dict]) -> list[dict]:
    return [{"n": i, "role": t["role"], "text": t["content"], "ts": ""} for i, t in enumerate(session, 1)]


def lme_session_file(sid: str) -> str:
    return f"{LME_HARNESS}-{rw.slugify(sid, 40)}.md"


def lme_brain(q: dict, root: str, mode: str) -> None:
    ingest(root, [{"harness": LME_HARNESS, "session": sid, "date": lme_iso(date), "turns": lme_turns(session)}
                  for sid, date, session in zip(q["haystack_session_ids"], q["haystack_dates"], q["haystack_sessions"])],
           mode)


def lme_items(args) -> list[dict]:
    items = []
    for q in json.load(open(args.data)):
        items.append({"bench": "longmemeval", "qid": q["question_id"], "raw": q, "category": q["question_type"],
                      "question": q["question"], "gold": str(q["answer"]),
                      "abstention": q["question_id"].endswith("_abs")})
    return items


def lme_coverage(turn: str, context_norm: str) -> float:
    """Share of an evidence turn's sentences that reached the context (normalised text). Matching only the turn's
    start counts a turn cut short as found; this sees what was cut."""
    sents = [s for s in (re.sub(r"\W+", " ", p.lower()).strip() for p in re.split(r"(?<=[.!?:])\s+|\n+", turn))
             if len(s) >= 20]
    return sum(s in context_norm for s in sents) / len(sents) if sents else 1.0


def lme_evidence(q: dict, episodes: list[dict]) -> dict | None:
    """Session- and turn-level recall of the evidence; None for abstention questions (no evidence)."""
    if q["question_id"].endswith("_abs"):
        return None
    want = {lme_session_file(s) for s in q["answer_session_ids"]}
    got = {e["session"] for e in episodes}
    text = norm(" ".join(e["text"] for e in episodes))
    turns = [t["content"] for s in q["haystack_sessions"] for t in s if t.get("has_answer")]
    return {"sess_any": bool(want & got), "sess_all": want <= got,
            "turns": sum(norm(t)[:60] in text for t in turns), "n_turns": len(turns),
            "turn_coverage": [lme_coverage(t, text) for t in turns]}


def lme_history(facts: list[dict], episodes: list[dict]) -> str:
    """Official layout: one block per session, oldest first; rewire facts appended as 'user facts'."""
    by: dict[tuple, list] = defaultdict(list)
    for e in episodes:
        by[(e["date"], e["session"])].append(e["text"])
    blocks = [f"### Session Date: {date}\nSession Content:\n" + "\n...\n".join(texts)
              for (date, _), texts in sorted(by.items())]
    lines = "\n".join(f"- ({f['date'] or 'undated'}) {f['text']}" for f in sorted(facts, key=lambda f: f["date"]))
    return "\n\n".join(blocks) + ("\n\nUser facts extracted from the chat history:\n" + lines if lines else "")


def lme_prepare(it: dict, args) -> dict:
    q = it["raw"]
    if args.mode == "fullctx":  # every session of the history, whole, in the official layout
        episodes = [{"session": lme_session_file(sid), "date": lme_iso(date),
                     "text": "\n".join(f"{t['role']}: {t['content']}" for t in session)}
                    for sid, date, session in zip(q["haystack_session_ids"], q["haystack_dates"], q["haystack_sessions"])]
        facts, ms, wall, root = [], 0.0, 0.0, None
    else:
        root = os.path.join(args.out, "longmemeval", f"brains-{args.mode}", q["question_id"])
        lme_brain(q, root, args.mode)
        hits, ms, wall = timed_recall(root, q["question"], args.k, args.episodes, also=it.get("also"),
                                      rebuild=args.mode == "full")
        episodes = [{"session": os.path.basename(h["session"]), "date": h["date"], "text": h["text"]}
                    for h in hits if h["type"] == "episode"]
        facts = [{"date": str(h.get("valid_from") or "")[:10], "text": (h.get("body") or "").strip() or h["title"]}
                 for h in hits if h["type"] != "episode"]
    history = lme_history(facts, episodes)
    ev = lme_evidence(q, episodes)
    evidence_turns = [t["content"] for s in q["haystack_sessions"] for t in s if t.get("has_answer")]
    return {"prompt": LME_PROMPTS.ANSWER_TEMPLATE.format(history, q["question_date"], q["question"]),
            "context": history, "context_tokens": count_tokens(history), "retrieval_ms": ms, "retrieval_wall_ms": wall, "brain": root,
            "raw_tokens": count_tokens(" ".join(t["content"] for s in q["haystack_sessions"] for t in s)),
            "n_memories": len(facts) + len(episodes),
            # for per-question diagnosis: excerpt sessions in rank order, and the sessions holding the answer
            "trace": {"ranked_sessions": [e["session"] for e in episodes],
                      "answer_sessions": [lme_session_file(s) for s in q.get("answer_session_ids", [])],
                      "context": history},
            "evidence": {"turns": ev["turns"], "n_turns": ev["n_turns"], "all": ev["sess_all"],
                         "turn_coverage": ev["turn_coverage"],
                         "session_any": ev["sess_any"], **ranking([e["text"] for e in episodes], evidence_turns)}
            if ev else None}


def lme_judge_prompt(it: dict, response: str) -> str:
    return LME_PROMPTS.judge_prompt({**it["raw"], "response": response})


def lme_verdict(text: str) -> bool:
    """The official judge answers "yes" or "no" only."""
    return text.strip().lower().lstrip("*\"' ").startswith("yes")


# ---- shared helpers for the transcript-only benchmarks -------------------

_ROOT_LOCKS: dict[str, threading.Lock] = defaultdict(threading.Lock)
_ROOT_LOCKS_GUARD = threading.Lock()


def write_sessions(root: str, harness: str, sessions: list[tuple[str, str, list[dict]]]) -> None:
    """Store sessions exactly as consolidation stores transcripts, without LLM extraction (mode episodes).
    Questions on the same history run in parallel: the first one writes the brain, the others wait."""
    with _ROOT_LOCKS_GUARD:
        lock = _ROOT_LOCKS[root]
    with lock:
        ingest(root, [{"harness": harness, "session": sid, "date": date, "turns": turns} for sid, date, turns in sessions],
               "episodes")


def recall_context(root: str, query: str, k: int, episodes: int, keep=None, also: list[str] | None = None,
                   order: str = "rank") -> tuple[str, list[str], float, float]:
    """rewire recall -> (context text, ranked excerpt texts, net ms, wall ms). `keep(excerpt_line)` drops lines
    (e.g. the future). `order` "chrono" shows the excerpts oldest first (date, then turn) instead of best match
    first; the returned list stays in rank order for the retrieval metrics."""
    hits, ms, wall = timed_recall(root, query, k, episodes, also=also)
    facts, ranked = [], []
    for h in hits:
        if h["type"] == "episode":
            lines = [l for l in h["text"].splitlines() if keep is None or keep(l)]
            if lines:
                ranked.append(f"({h['date']}) " + "\n".join(lines))
        else:
            facts.append(f"({h.get('date') or 'undated'}) {h['title']}: {h['snippet']}")
    shown, heading = ranked, "Excerpts from earlier conversations (best match first):"
    if order == "chrono":  # a preference stated later replaces an earlier one: show which came last
        shown = sorted(ranked, key=lambda e: (e[1:11], max(map(int, re.findall(r"(?m)^\[(\d+) ", e)), default=0)))
        heading = "Excerpts from earlier conversations, in the order they happened (oldest first):"
    parts = (["Facts remembered about the user:"] + facts if facts else []) + \
            ([heading] + shown if shown else [])
    return "\n".join(parts) or "(no relevant memories found)", ranked, ms, wall


# ---- BEAM (github.com/mohammadtavakoli78/BEAM; chats/<size>/<id>/) ---------

def beam_date(anchor: str | None) -> str:
    from datetime import datetime
    try:
        return datetime.strptime(anchor or "", "%B-%d-%Y").strftime("%Y-%m-%d")
    except ValueError:
        return ""


def beam_ids(src) -> list[int]:
    if isinstance(src, dict):
        return [i for v in src.values() for i in beam_ids(v)]
    if isinstance(src, list):
        return [i for v in src for i in beam_ids(v)]
    return [src] if isinstance(src, int) else []


def beam_items(args) -> list[dict]:
    """--data is BEAM's chats/ directory (repository copy or download_dataset.py output)."""
    items = []
    for size in args.beam_sizes.split(","):
        for chat_dir in sorted(glob.glob(os.path.join(args.data, size, "*")), key=lambda p: (len(p), p)):
            qpath = os.path.join(chat_dir, "probing_questions", "probing_questions.json")
            if not os.path.exists(qpath) or not os.path.exists(os.path.join(chat_dir, "chat.json")):
                continue
            for qtype, qs in json.load(open(qpath)).items():
                for i, q in enumerate(qs):
                    gold = next((q[k] for k in ("answer", "ideal_answer", "ideal_response", "ideal_summary") if k in q), "")
                    items.append({"bench": "beam", "qid": f"{size}-{os.path.basename(chat_dir)}-{qtype}-{i}",
                                  "chat_dir": chat_dir, "size": size, "category": qtype, "question": q["question"],
                                  "gold": json.dumps(gold) if not isinstance(gold, str) else gold,
                                  "rubric": q.get("rubric") or [], "evidence_ids": beam_ids(q.get("source_chat_ids")),
                                  "ordering": q.get("ordering_tested") or [],
                                  "abstention": qtype == "abstention"})
    return items


def beam_messages(chat_dir: str) -> list[tuple[str, list[dict]]]:
    """(date, turns) per batch; turn n = the message id, as BEAM's source_chat_ids refer to it."""
    out = []
    for batch in json.load(open(os.path.join(chat_dir, "chat.json"))):
        msgs = [m for turn in batch["turns"] for m in turn]
        date = next((beam_date(m.get("time_anchor")) for m in msgs if beam_date(m.get("time_anchor"))), "")
        out.append((date, [{"n": m["id"], "role": "assistant" if m["role"] == "assistant" else "user",
                            "text": m["content"], "ts": ""} for m in msgs]))
    return out


def beam_prepare(it: dict, args) -> dict:
    if args.mode == "full":
        raise SystemExit("BEAM is supported in --mode episodes or fullctx (a 1M-token chat costs too much to consolidate)")
    batches = beam_messages(it["chat_dir"])
    by_id = {t["n"]: t["text"] for _, turns in batches for t in turns}
    if args.mode == "fullctx":
        context = "\n\n".join(f"({date or 'undated'})\n" + "\n".join(f"[{t['n']} {t['role']}] {t['text']}" for t in turns)
                               for date, turns in batches)
        ranked, ms, wall, root = [context], 0.0, 0.0, None
    else:
        root = os.path.join(args.out, "beam", "brains", it["size"], os.path.basename(it["chat_dir"]))
        name = f"beam-{it['size']}-{os.path.basename(it['chat_dir'])}"
        write_sessions(root, "beam", [(f"{name}-b{i + 1}", date or "2024-01-01", turns) for i, (date, turns) in enumerate(batches)])
        context, ranked, ms, wall = recall_context(root, it["question"], args.k, args.episodes, also=it.get("also"),
                                                   order=args.excerpt_order)
    evidence = [by_id[i] for i in it["evidence_ids"] if i in by_id]
    text = norm(" ".join(ranked))
    hits = sum(norm(e)[:50] in text for e in evidence)
    return {"prompt": BEAM_PROMPTS.ANSWER_GENERATION_FOR_RAG.replace("<context>", context).replace("<question>", it["question"]),
            "context": context, "context_tokens": count_tokens(context), "retrieval_ms": ms, "retrieval_wall_ms": wall, "brain": root,
            "n_memories": len(ranked), "raw_tokens": count_tokens(" ".join(by_id.values())),
            "evidence": {"turns": hits, "n_turns": len(evidence), "all": hits == len(evidence),
                         **ranking(ranked, evidence)} if evidence else None}


ORDER_ALIGN_PROMPT = """A response lists events in some order. Match each item of the response, in the response's order,
to the reference event it describes.

Reference events (numbered):
{reference}

Response:
{response}

Reply with one line of JSON and nothing else: {"order": [n, ...]} where n is the reference number of each item the
response lists, in the response's order, and 0 for an item that matches no reference event."""


def kendall_tau_b(xs: list[float], ys: list[float]) -> float:
    """Kendall's tau-b between two rankings of the same items (1.0 for one item or none)."""
    conc = disc = tx = ty = 0
    for i in range(len(xs)):
        for j in range(i + 1, len(xs)):
            dx, dy = xs[i] - xs[j], ys[i] - ys[j]
            if dx == 0 and dy == 0:
                continue
            if dx == 0:
                tx += 1
            elif dy == 0:
                ty += 1
            elif dx * dy > 0:
                conc += 1
            else:
                disc += 1
    denom = math.sqrt((conc + disc + tx) * (conc + disc + ty))
    return (conc - disc) / denom if denom else 1.0


def beam_order_score(it: dict, response: str, judge) -> tuple[float | None, dict | None]:
    """BEAM's event-ordering metric, (tau_b + 1) / 2 x F1: the response's items are aligned to the reference
    events (by the judge model), F1 counts the matched events, tau_b compares their order with the reference."""
    ref = [re.sub(r"^\s*\d+(st|nd|rd|th)?\s*:\s*", "", r) for r in it.get("ordering") or []]
    if len(ref) < 2:
        return None, None
    call = judge(ORDER_ALIGN_PROMPT.replace("{reference}", "\n".join(f"{i}. {r}" for i, r in enumerate(ref, 1)))
                 .replace("{response}", response))
    m = re.search(r"\{.*\}", call["text"], re.S)
    try:
        order = [int(x) for x in json.loads(m.group(0))["order"]]
    except (AttributeError, ValueError, KeyError, TypeError):
        return 0.0, call
    seen, matched = set(), []
    for pos, n in enumerate(order):
        if 1 <= n <= len(ref) and n not in seen:
            seen.add(n)
            matched.append((pos, n))
    if not matched:
        return 0.0, call
    precision, recall = len(matched) / max(1, len(order)), len(matched) / len(ref)
    f1_score = 2 * precision * recall / (precision + recall)
    tau = kendall_tau_b([p for p, _ in matched], [n for _, n in matched])
    return (tau + 1) / 2 * f1_score, call


def beam_judge(it: dict, response: str, judge) -> tuple[float, list[dict]]:
    """Official BEAM scoring: one judge call per rubric item (1.0 / 0.5 / 0.0), averaged. Event ordering adds the
    order metric (beam_order_score), reported next to the rubric score as row["order_score"]."""
    scores, calls = [], []
    for item in it["rubric"]:
        call = judge(BEAM_PROMPTS.UNIFIED_LLM_JUDGE_BASE_PROMPT.replace("<rubric_item>", item)
                     .replace("<llm_response>", response))
        calls.append(call)
        m = re.search(r'"score"\s*:\s*"?([01](?:\.\d+)?)', call["text"])
        scores.append(float(m.group(1)) if m else 0.0)
    if it["category"] == "event_ordering":
        it["order_score"], call = beam_order_score(it, response, judge)
        if call:
            calls.append(call)
    return (sum(scores) / len(scores) if scores else 0.0), calls


# ---- PersonaMem (huggingface.co/datasets/bowen-upenn/PersonaMem) -------------

PERSONAMEM_INSTRUCTIONS = ("Find the most appropriate model response and give your final answer (a), (b), (c), or (d) "
                           "after the special token <final_answer>.")  # verbatim from the official inference code


def personamem_items(args) -> list[dict]:
    """--data holds questions_<size>.csv and shared_contexts_<size>.jsonl (size from --personamem-size)."""
    import csv

    size = args.personamem_size
    contexts = {}
    with open(os.path.join(args.data, f"shared_contexts_{size}.jsonl"), encoding="utf-8") as fh:
        for line in fh:
            if line.strip():
                contexts.update(json.loads(line))
    items = []
    with open(os.path.join(args.data, f"questions_{size}.csv"), encoding="utf-8") as fh:
        for row in csv.DictReader(fh):
            items.append({"bench": "personamem", "qid": row["question_id"], "category": row["question_type"],
                          "question": row["user_question_or_message"], "gold": row["correct_answer"],
                          "options": row["all_options"], "context_id": row["shared_context_id"],
                          "end": int(float(row["end_index_in_shared_context"])), "abstention": False,
                          "messages": contexts[row["shared_context_id"]]})
    return items


def personamem_prepare(it: dict, args) -> dict:
    """One brain per shared context, holding all of it; excerpts after the question's position are dropped,
    so memory never contains the future (questions on the same context sit at different positions)."""
    if args.mode == "full":
        raise SystemExit("PersonaMem is supported in --mode episodes or fullctx (facts could leak turns after the question)")
    turns = [{"n": i, "role": "assistant" if m.get("role") == "assistant" else "user",
              "text": (f"[{m['role']}] " if m.get("role") == "system" else "") + str(m.get("content", "")), "ts": ""}
             for i, m in enumerate(it["messages"])]
    if args.mode == "fullctx":  # the conversation up to the question
        context = "\n".join(f"[{t['n']} {t['role']}] {t['text']}" for t in turns[:it["end"]])
        ranked, ms, wall, root = [], 0.0, 0.0, None
    else:
        root = os.path.join(args.out, "personamem", args.personamem_size, "brains", re.sub(r"\W+", "-", it["context_id"])[:60])
        write_sessions(root, "personamem", [(f"pm-{it['context_id']}"[:80], "2024-01-01", turns)])

        def before(line: str) -> bool:
            m = re.match(r"\[(\d+) ", line)
            return bool(m) and int(m.group(1)) < it["end"]

        context, ranked, ms, wall = recall_context(root, it["question"], args.k, args.episodes, keep=before,
                                                   also=it.get("also"), order=args.excerpt_order)
    prompt = (f"{context}\n\n{it['question']}\n\n{PERSONAMEM_INSTRUCTIONS}\n\n{it['options']}")
    return {"prompt": prompt, "context": context, "context_tokens": count_tokens(context), "retrieval_ms": ms,
            "retrieval_wall_ms": wall, "brain": root, "n_memories": len(ranked),
            "raw_tokens": count_tokens(" ".join(t["text"] for t in turns[:it["end"]])), "evidence": None}


def personamem_score(it: dict, response: str, judge) -> tuple[float, list[dict]]:
    """Official scoring: the option letter after <final_answer> (or anywhere) must be exactly the correct one."""
    def options(text: str) -> set:
        text = text.lower()
        found = re.findall(r"\(([a-d])\)", text)
        return set(found) if found else set(re.findall(r"\b([a-d])\b", text))

    correct = it["gold"].lower().strip("() ")
    final = response.split("<final_answer>")[-1].strip().removesuffix("</final_answer>").strip()
    return (1.0 if options(final) == {correct} or options(response) == {correct} else 0.0), []


def single_judge(prompt_fn, verdict_fn):
    def run(it, response, judge):
        call = judge(prompt_fn(it, response))
        return (1.0 if verdict_fn(call["text"]) else 0.0), [call]
    return run


BENCH = {
    "locomo": {"items": locomo_items, "prepare": locomo_prepare, "answer": locomo_answer,
               "judge": single_judge(locomo_judge_prompt, locomo_verdict)},
    "longmemeval": {"items": lme_items, "prepare": lme_prepare, "answer": lambda t: t.strip(),
                    "judge": single_judge(lme_judge_prompt, lme_verdict)},
    "beam": {"items": beam_items, "prepare": beam_prepare, "answer": lambda t: t.strip(), "judge": beam_judge},
    "personamem": {"items": personamem_items, "prepare": personamem_prepare, "answer": lambda t: t.strip(),
                   "judge": personamem_score},
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
               "evidence_all_with_facts": (sum(e["all_with_facts"] for e in ev if "all_with_facts" in e)
                                           / max(1, sum("all_with_facts" in e for e in ev)))
               if any("all_with_facts" in e for e in ev) else None,
               "context_tokens_mean": sum(r["context_tokens"] for r in g) / len(g),
               "context_tokens_p95": pct([r["context_tokens"] for r in g], 0.95),
               "context_tokens_median": pct([r["context_tokens"] for r in g], 0.5),
               "retrieval_wall_ms_p50": pct([r.get("retrieval_wall_ms", r["retrieval_ms"]) for r in g], 0.5),
               "retrieval_ms_p50": pct([r["retrieval_ms"] for r in g], 0.5),
               "retrieval_ms_p95": pct([r["retrieval_ms"] for r in g], 0.95),
               "retrieval_ms_p99": pct([r["retrieval_ms"] for r in g], 0.99),
               "MRR": sum(e["rr"] for e in ev) / len(ev) if ev else None,
               "context_precision": sum(e["precision"] for e in ev) / len(ev) if ev else None,
               "evidence_session_any": (sum(e["session_any"] for e in ev if "session_any" in e)
                                        / max(1, sum("session_any" in e for e in ev))) if any("session_any" in e for e in ev) else None,
               "compression": sum(r["raw_tokens"] for r in g) / max(1, sum(r["context_tokens"] for r in g))}
        cov = [c for e in ev for c in e.get("turn_coverage", [])]
        if cov:  # LongMemEval: how much of each evidence turn reached the context, not only its first words
            out["evidence_turn_coverage"] = sum(cov) / len(cov)
            out["evidence_turns_complete"] = sum(c >= 0.9 for c in cov) / len(cov)
        # retrieval vs answering: accuracy when every evidence turn was in the context, and when some were not
        with_ev = [r for r in judged if r.get("evidence")]
        hit = [r for r in with_ev if r["evidence"]["all"]]
        miss = [r for r in with_ev if not r["evidence"]["all"]]
        out["J_all_evidence"] = sum(r["correct"] for r in hit) / len(hit) if hit else None
        out["J_missing_evidence"] = sum(r["correct"] for r in miss) / len(miss) if miss else None
        clean = [r for r in judged if not r.get("errata")]
        out["errata_n"] = len(judged) - len(clean)
        out["J_clean"] = sum(r["correct"] for r in clean) / len(clean) if clean and out["errata_n"] else None
        listed = [r for r in judged if "item_coverage" in r]
        if listed:  # list answers: the judge accepts one right item, this is the share of items found
            out["list_n"] = len(listed)
            out["list_J"] = sum(r["correct"] for r in listed) / len(listed)
            out["list_item_coverage"] = sum(r["item_coverage"] for r in listed) / len(listed)
            llm = [r for r in listed if r.get("list_items_llm") is not None]
            if llm:
                out["list_items_llm"] = sum(r["list_items_llm"] for r in llm) / len(llm)
                out["list_complete"] = sum(r["list_items_llm"] == 1 for r in llm) / len(llm)
                # strict J: a list answer is right only when the judge accepts it and it names every gold item
                strict = [bool(r["correct"]) and (r.get("list_items_llm") in (None, 1.0)) if "item_coverage" in r
                          else bool(r["correct"]) for r in judged]
                out["J_strict"] = sum(strict) / len(strict)
        if args.bench == "locomo" and judged:
            out["F1"] = sum(r["f1"] for r in judged) / len(judged)
            out["BLEU1"] = sum(r["bleu1"] for r in judged) / len(judged)
        timed = [r["latency"] for r in g if r.get("latency")]
        if timed:  # end to end: extra queries + recall + answer, per question
            out["e2e_ms_p50"] = pct([t["e2e_ms"] for t in timed], 0.5)
            out["e2e_ms_p95"] = pct([t["e2e_ms"] for t in timed], 0.95)
            out["answer_ms_p50"] = pct([t["answer_ms"] for t in timed], 0.5)
        ordered = [r["order_score"] for r in g if r.get("order_score") is not None]
        if ordered:  # BEAM event ordering: (tau_b + 1) / 2 x F1
            out["order_score"] = sum(ordered) / len(ordered)
        answered = [r for r in g if r.get("answer_call")]
        if answered:
            billed = [r["answer_call"]["prompt_tokens"] or 0 for r in answered]
            out["answer_prompt_tokens_median"] = pct(billed, 0.5)
            out["answer_prompt_tokens_p95"] = pct(billed, 0.95)
            out["answer_prompt_tokens_per_response"] = sum(r["answer_call"]["prompt_tokens"] for r in answered) / len(answered)
            out["answer_completion_tokens_per_response"] = sum(r["answer_call"]["completion_tokens"] for r in answered) / len(answered)
            out["answer_words_mean"] = sum(len(r["response"].split()) for r in answered) / len(answered)
        return out

    summary = {"bench": args.bench, "mode": args.mode, "k": args.k, "episodes": args.episodes,
               "answer_model": args.answer_model, "judge_model": args.judge_model,
               "expand_model": getattr(args, "expand_model", None),
               "groups": {name: group(g) for name, g in by.items()},
               "llm_spent_this_run": ledger.summary()}
    answered = [r for r in rows if "response" in r]
    if answered:  # a reasoning model that runs out of max_tokens returns no text: raise the limit or lower effort
        summary["empty_answers"] = sum(not r["response"].strip() for r in answered)
        summary["truncated_answers"] = sum((r["answer_call"].get("finish_reason") == "length") for r in answered)
    if any("judge_unparsed" in r for r in rows):
        summary["judge_unparsed"] = sum(r.get("judge_unparsed", False) for r in rows)
    abst = [r for r in rows if r["abstention"] and r.get("correct") is not None]
    if abst:
        summary["abstention_accuracy"] = sum(r["correct"] for r in abst) / len(abst)
    all_calls = [r[c] for r in rows for c in ("answer_call", "judge_call") if r.get(c)]
    if all_calls:  # per-response totals including cached replies (what the full run costs from scratch)
        summary["tokens_per_question_incl_cached"] = {
            "prompt": sum(c["prompt_tokens"] for c in all_calls) / len(rows),
            "completion": sum(c["completion_tokens"] for c in all_calls) / len(rows),
            "cost_usd": sum(c.get("cost") or 0 for c in all_calls) / len(rows)}
    if rows:  # what retrieval costs in tokens: the memory context it returns, and the LLM calls writing extra queries
        exp = [r["expand_call"] for r in rows if r.get("expand_call")]
        wrote = [e for e in exp if e["queries"]]
        summary["retrieval_tokens"] = {
            "context_mean": sum(r["context_tokens"] for r in rows) / len(rows),
            "context_p95": pct([r["context_tokens"] for r in rows], 0.95),
            "expand_prompt": sum(e["prompt_tokens"] for e in exp) / len(rows),
            "expand_completion": sum(e["completion_tokens"] for e in exp) / len(rows),
            "expand_prompt_when": sum(e["prompt_tokens"] for e in wrote) / max(1, len(wrote)),
            "expand_completion_when": sum(e["completion_tokens"] for e in wrote) / max(1, len(wrote)),
            "expanded": len(wrote), "expand_calls": sum(e["calls"] for e in exp)}
    if est:
        summary["dry_run_estimate"] = est
    return summary


def write_report(summary: dict, path: str) -> None:
    f = lambda v, p=False: "-" if v is None else (f"{v:.1%}" if p else (f"{v:,.0f}" if v >= 10 else f"{v:.3f}"))  # noqa: E731
    lines = [f"# {summary['bench']} · mode {summary['mode']} · k={summary['k']} · episodes={summary['episodes']}",
             f"answer model `{summary['answer_model']}` · judge model `{summary['judge_model']}`"
             + (f" · extra queries by `{summary['expand_model']}`" if summary.get("expand_model") else ""), "",
             "| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |",
             "|---|---|---|---|---|---|---|---|---|---|"]
    for name, g in sorted(summary["groups"].items(), key=lambda x: (x[0] == "overall", x[0])):
        lines.append(
            f"| {name} | {g['n']} | {f(g['J'], True)} | {f(g.get('F1'))} | {f(g.get('BLEU1'))} | "
            f"{f(g['evidence_turn_recall'], True)} | {f(g['evidence_all'], True)} | "
            f"{f(g['context_tokens_mean'])} / {f(g['context_tokens_p95'])} | "
            f"{f(g.get('answer_prompt_tokens_per_response'))} / {f(g.get('answer_completion_tokens_per_response'))} | "
            f"{g['retrieval_ms_p50']:.0f} / {g['retrieval_ms_p95']:.0f} |")
    lines += ["", "| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |",
              "|---|---|---|---|---|---|---|"]
    for name, g in sorted(summary["groups"].items(), key=lambda x: (x[0] == "overall", x[0])):
        lines.append(f"| {name} | {f(g['MRR'])} | {f(g['context_precision'], True)} | "
                     f"{f(g['evidence_session_any'], True)} | {g['compression']:.1f}× | {g['retrieval_ms_p99']:.0f} | "
                     f"{f(g.get('answer_words_mean'))} |")
    if any(g.get("evidence_turn_coverage") is not None for g in summary["groups"].values()):
        lines += ["", "| category | evidence turn coverage (share of its sentences in the context) | evidence turns complete (≥90%) |",
                  "|---|---|---|"]
        for name, g in sorted(summary["groups"].items(), key=lambda x: (x[0] == "overall", x[0])):
            lines.append(f"| {name} | {f(g.get('evidence_turn_coverage'), True)} | {f(g.get('evidence_turns_complete'), True)} |")
    if any(g.get("J_clean") is not None for g in summary["groups"].values()):
        lines += ["", "| category | J | disputed gold excluded | J without them |", "|---|---|---|---|"]
        for name, g in sorted(summary["groups"].items(), key=lambda x: (x[0] == "overall", x[0])):
            lines.append(f"| {name} | {f(g['J'], True)} | {g['errata_n']} | {f(g.get('J_clean'), True)} |")
        lines += ["", "Disputed gold: experiments/eval/locomo_errata.json (speaker swapped in the question, gold "
                      "contradicts the conversation, answer only in a photo). J is the official, comparable number."]
    if any(g.get("J_all_evidence") is not None for g in summary["groups"].values()):
        lines += ["", "| category | J when all evidence retrieved | J when evidence missing | share with all evidence |",
                  "|---|---|---|---|"]
        for name, g in sorted(summary["groups"].items(), key=lambda x: (x[0] == "overall", x[0])):
            lines.append(f"| {name} | {f(g.get('J_all_evidence'), True)} | {f(g.get('J_missing_evidence'), True)} | "
                         f"{f(g['evidence_all'], True)} |")
        lines += ["", "Low J with all evidence = the answering model; low share with all evidence = retrieval."]
    if any(g.get("evidence_all_with_facts") is not None for g in summary["groups"].values()):
        lines += ["", "| category | all evidence (excerpts) | all evidence (excerpts or a fact from the turn) | list answers n | list J | list items found |",
                  "|---|---|---|---|---|---|"]
        for name, g in sorted(summary["groups"].items(), key=lambda x: (x[0] == "overall", x[0])):
            lines.append(f"| {name} | {f(g['evidence_all'], True)} | {f(g.get('evidence_all_with_facts'), True)} | "
                         f"{g.get('list_n', '-')} | {f(g.get('list_J'), True)} | {f(g.get('list_item_coverage'), True)} |")
        lines += ["", "List J follows the official judge (one right item is enough); list items found is the share of "
                      "gold items present in the answer, without an LLM."]
    if any(g.get("J_strict") is not None for g in summary["groups"].values()):
        lines += ["", "| category | J | strict J (lists must be complete) | list answers n | list items named (LLM) | "
                      "list answers complete |", "|---|---|---|---|---|---|"]
        for name, g in sorted(summary["groups"].items(), key=lambda x: (x[0] == "overall", x[0])):
            lines.append(f"| {name} | {f(g['J'], True)} | {f(g.get('J_strict', g['J']), True)} | {g.get('list_n', 0)} | "
                         f"{f(g.get('list_items_llm'), True)} | {f(g.get('list_complete'), True)} |")
        lines += ["", "The list judge (same model as the judge) splits the gold list and counts the items the answer "
                      "names, paraphrases included."]
    if summary.get("judge_unparsed"):
        lines += ["", f"WARNING: {summary['judge_unparsed']} judge replies had no CORRECT/WRONG label (counted as wrong)."]
    if "abstention_accuracy" in summary:
        lines += ["", f"Abstention accuracy: {summary['abstention_accuracy']:.1%}"]
    if summary.get("empty_answers") or summary.get("truncated_answers"):
        lines += ["", f"WARNING: {summary['empty_answers']} empty and {summary['truncated_answers']} truncated answers "
                      "(max_tokens reached): raise --answer-max-tokens or pass --reasoning-effort low."]
    lines += ["", "## LLM usage this run (cached replies cost nothing)", "",
              "| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |",
              "|---|---|---|---|---|---|---|---|"]
    for stage, s in summary["llm_spent_this_run"].items():
        lines.append(f"| {stage} | {s['calls']} | {s['prompt_tokens']:,} | {s['completion_tokens']:,} | "
                     f"{s['reasoning_tokens']:,} | {s['prompt_tokens_per_call']:,.0f} / {s['completion_tokens_per_call']:,.0f} | "
                     f"{s['cost']:.4f} | {s['latency_p50_ms']} / {s['latency_p95_ms']} |")
    if "retrieval_tokens" in summary:
        t = summary["retrieval_tokens"]
        lines += ["", f"Retrieval tokens per question: context {t['context_mean']:,.0f} (p95 {t['context_p95']:,.0f}); "
                      f"extra-query LLM calls {t['expand_prompt']:,.0f} prompt + {t['expand_completion']:,.0f} completion "
                      f"on average over all questions, {t['expand_prompt_when']:,.0f} + {t['expand_completion_when']:,.0f} "
                      f"on the {t['expanded']} questions given extra queries."]
    if "tokens_per_question_incl_cached" in summary:
        t = summary["tokens_per_question_incl_cached"]
        lines += ["", f"Per question, answer + judge, including cached replies: {t['prompt']:,.0f} prompt tokens, "
                      f"{t['completion']:,.0f} completion tokens, ${t['cost_usd']:.5f}."]
    if summary.get("answer_model") == summary.get("judge_model"):
        lines += ["", f"NOTE: answers are judged by the same model (`{summary['judge_model']}`); a model tends to "
                      "accept its own answers, so J may read higher than with an independent judge."]
    if "stopped" in summary:
        lines += ["", f"PARTIAL RUN: {summary['stopped']}. Rerun the same command to resume from the cache."]
    if "ingest" in summary:
        i = summary["ingest"]
        lines += ["", "## Ingest (mode full)", "",
                  f"{i['sessions']} sessions consolidated ({i['failed_attempts']} failed attempts retried, "
                  f"{i['bad_json_replies']} replies that were not JSON). Facts per session: {i['facts_per_session']:.1f} "
                  f"({i['applied']} applied, {i['queued_then_approved']} queued then approved, {i['rejected']} rejected). "
                  f"Hypothetical questions: {i['questions']} ({i['questions_per_turn']:.2f} per turn). "
                  f"Seconds per session p50 / p95: {i['seconds_per_session_p50']:.0f} / {i['seconds_per_session_p95']:.0f} "
                  f"({i['seconds_total'] / 3600:.1f} h serial). LLM calls per session: {i['llm_calls_per_session']:.1f}"
                  + (f"; tokens per session {i['prompt_tokens_per_session']:,.0f} prompt + {i['completion_tokens_per_session']:,.0f} "
                     f"completion; cost ${i['cost_usd']:.4f}" if i.get("prompt_tokens_per_session") is not None else
                     f"; prompt {i['prompt_chars_per_session']:,.0f} characters per session") + ".", "",
                  "Active nodes by type: " + ", ".join(f"{k} {v}" for k, v in i["nodes_by_type"].items())]
    lines += ["", "## Tokens and latency", "",
              "| category | context tokens mean / median / p95 | answer prompt tokens (billed) mean / median / p95 | "
              "recall ms net p50 / p95 (wall p50) | end-to-end ms p50 / p95 | answer call ms p50 |",
              "|---|---|---|---|---|---|"]
    for name, g in sorted(summary["groups"].items(), key=lambda x: (x[0] == "overall", x[0])):
        lines.append(f"| {name} | {f(g['context_tokens_mean'])} / {f(g['context_tokens_median'])} / {f(g['context_tokens_p95'])} | "
                     f"{f(g.get('answer_prompt_tokens_per_response'))} / {f(g.get('answer_prompt_tokens_median'))} / "
                     f"{f(g.get('answer_prompt_tokens_p95'))} | {g['retrieval_ms_p50']:.0f} / {g['retrieval_ms_p95']:.0f} "
                     f"({g['retrieval_wall_ms_p50']:.0f}) | {f(g.get('e2e_ms_p50'))} / {f(g.get('e2e_ms_p95'))} | "
                     f"{f(g.get('answer_ms_p50'))} |")
    env = summary.get("env") or {}
    lines += ["", f"Context tokens: {env.get('tokenizer', '?')} tokenizer. Recall net = wall time of one `rewire recall` "
                  f"process minus {env.get('recall_overhead_ms', 0)} ms measured on an empty brain (process start and "
                  "model load, which a running MCP server pays once). End to end = extra-query calls + recall + answer "
                  "call, with the latency of the call that produced each (cached) reply; `claude -p` adds its own start "
                  "time to every call. Runs with --parallel > 1 share the machine, so latencies are upper bounds."]
    if any(g.get("order_score") is not None for g in summary["groups"].values()):
        lines += ["", "BEAM event ordering, (tau_b + 1) / 2 x F1 after aligning the answer's items to the reference: " +
                  ", ".join(f"{n} {g['order_score']:.3f}" for n, g in summary["groups"].items() if g.get("order_score") is not None)]
    if env:
        m = env.get("machine") or {}
        lines += ["", f"Run: {env.get('date')} · commit `{str(env.get('commit'))[:10]}`{' (modified)' if env.get('dirty') else ''} · "
                      f"{env.get('rewire') or 'no rewire (fullctx)'} · data `{env.get('data')}` sha256 {env.get('data_sha256_16')} · "
                      f"{m.get('cpu')} ({m.get('cores')} cores, {m.get('memory_gb')} GB) · {m.get('os')} · "
                      f"parallel {env.get('parallel')} · replicate {env.get('replicate')}"]
    if "dry_run_estimate" in summary:
        e = summary["dry_run_estimate"]
        lines += ["", "## Dry-run estimate (no LLM called)", "", "```", json.dumps(e, indent=1), "```"]
    lines += ["", "Published scores of other memory systems, and why they are not directly comparable: "
                  "docs/evaluation.md."]
    with open(path, "w", encoding="utf-8") as fh:
        fh.write("\n".join(lines) + "\n")


# ---- driver --------------------------------------------------------------

def run_environment(args) -> dict:
    """What a reader needs to reproduce or qualify the numbers: code, binary, data, machine, tokenizer."""
    def sh(*cmd):
        try:
            return subprocess.run(cmd, capture_output=True, text=True, cwd=ROOT, timeout=30).stdout.strip()
        except Exception:
            return ""
    cpu = next((l.split(":", 1)[1].strip() for l in open("/proc/cpuinfo") if l.startswith("model name")), "") \
        if os.path.exists("/proc/cpuinfo") else platform.processor()
    mem = next((l.split()[1] for l in open("/proc/meminfo") if l.startswith("MemTotal")), None) \
        if os.path.exists("/proc/meminfo") else None
    data = args.data if os.path.isfile(args.data) else None
    digest = hashlib.sha256(open(data, "rb").read()).hexdigest()[:16] if data else None
    return {"date": datetime.now(timezone.utc).isoformat(timespec="seconds"),
            "commit": sh("git", "rev-parse", "HEAD"), "dirty": bool(sh("git", "status", "--porcelain", "--", "rust", "experiments/eval")),
            "rewire": rw.version() if args.mode != "fullctx" else None,
            "recall_overhead_ms": round(OVERHEAD_MS, 1), "tokenizer": tokens.name(),
            "data": os.path.basename(args.data.rstrip("/")), "data_sha256_16": digest,
            "machine": {"os": platform.platform(), "cpu": cpu, "cores": os.cpu_count(),
                        "memory_gb": round(int(mem) / 1048576, 1) if mem else None},
            "python": platform.python_version(), "parallel": args.parallel, "replicate": args.replicate,
            "command": " ".join(shlex.quote(a) for a in sys.argv)}



def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--bench", choices=list(BENCH), required=True,
                    help="locomo | longmemeval | beam (data: BEAM's chats/ dir) | personamem (data: dir with the csv + jsonl)")
    ap.add_argument("--data", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--mode", choices=["episodes", "full", "fullctx"], default="episodes",
                    help="episodes: transcripts only (no LLM ingest); full: real consolidation through --ingest-model; "
                         "fullctx: no memory, the whole history up to the question (the full-context baseline)")
    ap.add_argument("--sample", type=int, default=20, help="questions per category (0 = all)")
    ap.add_argument("--categories", default="all", help="only these categories (comma-separated, e.g. multi-hop)")
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--convs", default="all", help="LoCoMo conversations (comma-separated sample ids or all)")
    ap.add_argument("--beam-sizes", default="100K", help="BEAM chat sizes: 100K,500K,1M")
    ap.add_argument("--personamem-size", default="32k", choices=["32k", "128k", "1M"])
    ap.add_argument("--k", type=int, default=30, help="memories (facts) recalled")
    ap.add_argument("--expand-model", help="this model writes up to 4 extra queries per question, passed to "
                    "recall as `also` (what an agent calling rewire over MCP does)")
    ap.add_argument("--expand-when", choices=["all", "gather", "model", "union"], default="all",
                    help="gather: extra queries only for counting, total, ordering and time questions (a rule); "
                         "model: the expansion model decides, and may write none; union: the model or the rule")
    ap.add_argument("--excerpt-order", choices=["rank", "chrono"], default="rank",
                    help="BEAM and PersonaMem: excerpts best match first, or oldest first")
    ap.add_argument("--episodes", type=int, default=10, help="transcript excerpts recalled")
    default_model = os.environ.get("OPENROUTER_MODEL") or "openai/gpt-4o-mini"
    ap.add_argument("--answer-model", default=default_model,
                    help="default: $OPENROUTER_MODEL or openai/gpt-4o-mini; claude-cli/haiku = local Claude Code CLI")
    ap.add_argument("--judge-model", default=default_model)
    ap.add_argument("--ingest-model", default=default_model, help="consolidation model for --mode full")
    ap.add_argument("--reasoning-effort", choices=["low", "medium", "high"],
                    help="for reasoning models (e.g. Nemotron 3): keeps thinking from using up max_tokens")
    ap.add_argument("--answer-max-tokens", type=int, default=1500)
    ap.add_argument("--judge-max-tokens", type=int, default=200)
    ap.add_argument("--parallel", type=int, default=8)
    ap.add_argument("--max-cost", type=float, help="stop when this many USD were spent in this run")
    ap.add_argument("--skip-judge", action="store_true", help="answer only; rerun without it to judge the cached answers")
    ap.add_argument("--retrieval-only", action="store_true", help="no LLM at all: evidence recall + context size")
    ap.add_argument("--dry-run", action="store_true", help="no LLM at all: estimate tokens and cost of the run")
    ap.add_argument("--est-answer-out", type=int, default=400, help="dry-run: expected answer completion tokens")
    ap.add_argument("--est-judge-out", type=int, default=60, help="dry-run: expected judge completion tokens")
    ap.add_argument("--price", action="append", default=[], metavar="MODEL=IN,OUT",
                    help="USD per 1M tokens for the dry-run when the catalog is unreachable")
    ap.add_argument("--replicate", type=int, default=0,
                    help="N > 0: answer and judge again into separate cache entries (run-to-run noise; see compare.py)")
    ap.add_argument("--ingest-only", action="store_true", help="LoCoMo: build (and consolidate) the brains, then stop")
    ap.add_argument("--save-prompts", action="store_true",
                    help="keep each question's answer prompt and context in the results file (for re-judging or audit)")
    ap.add_argument("--label", default="run")
    args = ap.parse_args()
    global OVERHEAD_MS, INGEST_COMMAND
    if args.mode != "fullctx":
        rw.check()

    no_llm = args.retrieval_only or args.dry_run
    ledger = Ledger(args.max_cost)
    # --retrieval-only still calls --expand-model: the extra queries are part of recall, not of answering
    client = None if no_llm and not args.expand_model else Client(ledger, reasoning_effort=args.reasoning_effort)
    args.answer_model, args.judge_model, args.ingest_model = (
        normalize_model(m) for m in (args.answer_model, args.judge_model, args.ingest_model))
    if args.expand_model:
        args.expand_model = normalize_model(args.expand_model)
    if args.mode == "full":  # consolidation runs in rewire; its command provider calls back into llm_cmd.py
        INGEST_COMMAND = " ".join([shlex.quote(sys.executable), shlex.quote(os.path.join(HERE, "llm_cmd.py")),
                                   "--model", shlex.quote(args.ingest_model), "--ledger", "{ledger}"])
    bench = BENCH[args.bench]
    os.makedirs(args.out, exist_ok=True)
    global REPLICATE
    REPLICATE = args.replicate
    cache = Cache(os.path.join(args.out, "cache", "llm.jsonl"))

    items = bench["items"](args)
    if args.categories != "all":
        items = [it for it in items if it["category"] in args.categories.split(",")]
    items = sample(items, args.sample, args.seed)
    print(f"{args.bench}: {len(items)} questions ({args.sample or 'all'} per category), mode {args.mode}", flush=True)
    all_items = items
    brain_root = lambda conv: os.path.join(args.out, "locomo", f"brains-{args.mode}", conv["sample_id"])  # noqa: E731
    is_done = lambda conv: os.path.exists(os.path.join(brain_root(conv), ".state", "eval_done"))  # noqa: E731
    if args.mode == "full" and no_llm and args.bench == "locomo":  # scoring brains ingested by an earlier run
        missing = sorted({it["conv"]["sample_id"] for it in items if not is_done(it["conv"])})
        if missing:
            sys.exit(f"--mode full needs the LLM for ingestion; not ingested yet: {', '.join(missing)}")
    elif args.mode == "full" and no_llm:
        sys.exit("--mode full needs the LLM for ingestion; use --mode episodes with --dry-run/--retrieval-only")
    if args.mode != "fullctx":
        OVERHEAD_MS = rw.process_overhead()
        print(f"rewire recall process overhead (empty brain, median): {OVERHEAD_MS:.0f} ms", flush=True)

    def run_one(it: dict) -> dict:
        def expansion(gated: bool) -> list[str]:
            prompt = expand_prompt(it["question"], gated=gated)
            if client is None and not cache.get(Cache.key("expand", args.expand_model, prompt)):
                raise SystemExit("--retrieval-only with --expand-model needs the expansions cached by an LLM run")
            call = ask(client, cache, "expand", args.expand_model, prompt, 1000)
            calls.append({k: call.get(k) or 0 for k in ("prompt_tokens", "completion_tokens", "reasoning_tokens", "cost",
                                                        "latency_ms")})
            return expand_queries(call["text"])

        calls = []  # the expansion calls: LLM tokens spent on retrieval, not on answering

        # the calling model writes extra queries (recall `also`), from the question only: for every question (all),
        # when a rule says the question gathers or hinges on time (gather), when the model itself decides (model),
        # or when either does (union)
        if args.expand_model:
            if args.expand_when in ("all", "gather") and (args.expand_when == "all" or wants_expansion(it["question"])):
                it["also"] = expansion(gated=False)
            elif args.expand_when in ("model", "union"):
                it["also"] = expansion(gated=True)
                if not it["also"] and args.expand_when == "union" and wants_expansion(it["question"]):
                    it["also"] = expansion(gated=False)
        prep = bench["prepare"](it, args)
        row = {k: it[k] for k in ("bench", "qid", "category", "question", "gold", "abstention")}
        if it.get("errata"):
            row["errata"] = it["errata"]
        row.update({k: prep[k] for k in ("context_tokens", "raw_tokens", "retrieval_ms", "retrieval_wall_ms",
                                          "n_memories", "evidence", "brain")})
        if it.get("also"):
            row["also"] = it["also"]
        if prep.get("trace"):
            row["trace"] = prep["trace"]
        if args.save_prompts:
            row["prompt"], row["context"] = prep["prompt"], prep["context"]
        row["answer_prompt_tokens_est"] = count_tokens(prep["prompt"])
        if calls:
            row["expand_call"] = {k: sum(c[k] for c in calls) for k in calls[0]}
            row["expand_call"].update(calls=len(calls), queries=len(it.get("also") or []))
        if no_llm:
            return row
        a = ask(client, cache, "answer", args.answer_model, prep["prompt"], args.answer_max_tokens)
        row["response"] = bench["answer"](a["text"])
        row["answer_call"] = {k: a.get(k) for k in ("prompt_tokens", "completion_tokens", "reasoning_tokens", "cost",
                                                    "latency_ms", "cached", "finish_reason")}
        # what a user waits for: extra queries (when written), recall, then the answer. LLM latencies are those of
        # the call that produced the reply (kept in the cache), through the client used (`claude -p` adds its start)
        expand_ms = sum(c.get("latency_ms") or 0 for c in calls)
        row["latency"] = {"expand_ms": expand_ms, "recall_ms": row["retrieval_ms"], "answer_ms": a.get("latency_ms") or 0,
                          "e2e_ms": expand_ms + row["retrieval_ms"] + (a.get("latency_ms") or 0)}
        if args.skip_judge:  # answers only (cached); a later run with the judge reuses them
            return row
        judge = lambda prompt: ask(client, cache, "judge", args.judge_model, prompt, args.judge_max_tokens)  # noqa: E731
        row["correct"], calls = bench["judge"](it, row["response"], judge)  # score in [0, 1]
        if it.get("order_score") is not None:  # BEAM event ordering: (tau_b + 1) / 2 x F1
            row["order_score"] = it["order_score"]
        if calls:  # several calls for rubric judges (BEAM); none for multiple choice (PersonaMem)
            row["judge_call"] = {k: sum(c[k] for c in calls) for k in ("prompt_tokens", "completion_tokens",
                                                                       "reasoning_tokens", "cost", "latency_ms")}
            row["judge_call"]["cached"] = all(c["cached"] for c in calls)
        if args.bench == "locomo":
            row["f1"] = f1(row["response"], row["gold"])
            row["bleu1"] = bleu1(row["response"], row["gold"])
            row["judge_unparsed"] = locomo_verdict(calls[0]["text"]) is None
            if len(gold_items(row["gold"])) > 1:
                row["item_coverage"] = item_coverage(row["gold"], row["response"])
                c = ask(client, cache, "list_judge", args.judge_model, list_judge_prompt(it, row["response"]),
                        args.judge_max_tokens)
                row["list_items_llm"] = list_coverage(c["text"])
        return row

    rows, stopped = [], None
    # one brain per LoCoMo conversation: ingest it once before questions run in parallel
    if args.bench == "locomo":
        convs = list({it["conv"]["sample_id"]: it["conv"] for it in items}.values())
        with ThreadPoolExecutor(args.parallel) as pool:  # conversations are independent brains
            futures = [pool.submit(locomo_brain, conv, brain_root(conv), args.mode) for conv in convs
                       if args.mode != "fullctx"]
            errors = []
            for fut in futures:
                try:
                    fut.result()
                except Exception as exc:  # quota, budget or a failing session
                    errors.append(str(exc))
        if args.ingest_only:
            print(f"ingest-only: {len(convs) - len(errors)} of {len(convs)} conversations ingested"
                  + (f"; first error: {errors[0]}" if errors else ""), flush=True)
            return
        if errors:  # questions still run on the conversations whose ingest finished
            stopped = f"ingest stopped: {errors[0]}"
            items = [it for it in items if is_done(it["conv"])]
            print(f"{stopped}; {len(items)} questions on finished conversations", flush=True)
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
    ingest = None
    if args.mode == "full" and args.bench in ("locomo", "longmemeval"):
        roots = ({brain_root(it["conv"]) for it in all_items} if args.bench == "locomo" else
                 {os.path.join(args.out, "longmemeval", "brains-full", it["qid"]) for it in all_items})
        ingest = ingest_stats(sorted(roots))

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
        if args.bench == "beam":  # one judge call per rubric item, each carrying the full judge prompt
            per_call = count_tokens(BEAM_PROMPTS.UNIFIED_LLM_JUDGE_BASE_PROMPT) + args.est_answer_out
            n_calls = sum(len(it["rubric"]) for it in items if it["qid"] in {r["qid"] for r in rows})
        elif args.bench == "personamem":  # multiple choice: scored by matching the option, no judge
            per_call, n_calls = 0, 0
        else:  # judge prompt (rules + question + gold + answer), roughly
            per_call, n_calls = 400, len(rows)
        judge_in = per_call * n_calls
        est = {"questions": len(rows), "answer_prompt_tokens": ans_in,
               "answer_completion_tokens": len(rows) * args.est_answer_out,
               "judge_calls": n_calls, "judge_prompt_tokens": judge_in, "judge_completion_tokens": n_calls * args.est_judge_out}
        for stage, model, tin, tout in (("answer", args.answer_model, ans_in, len(rows) * args.est_answer_out),
                                        ("judge", args.judge_model, judge_in, n_calls * args.est_judge_out)):
            if model in prices:
                est[f"{stage}_cost_usd"] = round((tin * prices[model][0] + tout * prices[model][1]) / 1e6, 4)

    summary = summarize(rows, ledger, args, est)
    summary["env"] = run_environment(args)
    if ingest:
        summary["ingest"] = ingest
    if stopped:
        summary["stopped"] = stopped
    tag = f"{args.bench}-{args.label}"
    json.dump({"summary": summary, "rows": rows}, open(os.path.join(args.out, f"results-{tag}.json"), "w"),
              indent=1, ensure_ascii=False, default=str)
    write_report(summary, os.path.join(args.out, f"report-{tag}.md"))
    print(open(os.path.join(args.out, f"report-{tag}.md")).read())


if __name__ == "__main__":
    main()
