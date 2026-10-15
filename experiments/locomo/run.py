"""LoCoMo end-to-end eval for rewire, using Mem0's official answer + judge prompts.

Stages (each cached on disk, so the run can be resumed):
  ingest  : every session of a conversation is consolidated into its own brain,
            in order, exactly like a harness transcript would be.
  answer  : for each question, rewire recall returns the top-K memories; an LLM
            answers with Mem0's ANSWER_GENERATION_PROMPT (memories in chronological
            order with dates).
  judge   : Mem0's unified judge prompt (CORRECT/WRONG); also token F1 as a strict check.

Categories 1-4 are scored, 5 (adversarial) excluded - same as Mem0's harness.

Usage: python experiments/locomo/run.py --data locomo10.json --out DIR --convs conv-26,conv-30,conv-49
"""

import argparse
import glob
import json
import os
import re
import string
import subprocess
import sys
from collections import Counter, defaultdict
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "..", "..", "src"))
sys.path.insert(0, HERE)

from official_prompts import ANSWER_GENERATION_PROMPT, JUDGE_PROMPT  # noqa: E402
from rewire.brain import Brain  # noqa: E402
from rewire.consolidate import Consolidator  # noqa: E402
from rewire.index import Index  # noqa: E402
from rewire.llm import CommandProvider, LLMError, extract_json  # noqa: E402

CATS = {1: "multi-hop", 2: "temporal", 3: "open-domain", 4: "single-hop"}
INNER = 4  # concurrent LLM calls per conversation (see --parallel)


def claude(prompt: str, timeout: int = 900, attempts: int = 6) -> str:
    """claude -p with retries: usage limits and transient failures back off instead of killing the run."""
    import time

    for attempt in range(attempts):
        try:
            r = subprocess.run(["claude", "-p", "--output-format", "text"], input=prompt, capture_output=True,
                               text=True, timeout=timeout, cwd=HERE, env={**os.environ, "REWIRE_INTERNAL": "1"})
            if r.returncode == 0 and r.stdout.strip():
                return r.stdout
            err = (r.stderr or r.stdout)[:300]
        except subprocess.TimeoutExpired:
            err = "timeout"
        if attempt < attempts - 1:
            time.sleep(min(60 * 2 ** attempt, 900))
    raise RuntimeError(f"claude -p failed after {attempts} attempts: {err}")


def claude_json(prompt: str, attempts: int = 3) -> dict:
    """claude() whose reply must parse as a JSON object; a malformed reply is asked again."""
    for attempt in range(attempts):
        try:
            out = extract_json(claude(prompt))
            if isinstance(out, dict):
                return out
        except LLMError:
            if attempt == attempts - 1:
                raise
    raise ValueError("model did not return a JSON object")


def session_date(raw: str) -> str:
    """'1:56 pm on 8 May, 2023' -> '2023-05-08'."""
    m = re.search(r"(\d{1,2}) (\w+),? (\d{4})", raw)
    return datetime.strptime(" ".join(m.groups()), "%d %B %Y").strftime("%Y-%m-%d") if m else ""


def sessions_of(conv: dict):
    c = conv["conversation"]
    keys = sorted((k for k in c if re.fullmatch(r"session_\d+", k)), key=lambda k: int(k.split("_")[1]))
    for k in keys:
        turns = []
        for i, t in enumerate(c[k], 1):
            text = t["text"]
            if t.get("blip_caption"):
                text += f" [shares a photo: {t['blip_caption']}]"
            turns.append({"n": i, "role": "user", "text": f"{t['speaker']}: {text}", "ts": ""})
        yield k, session_date(c.get(f"{k}_date_time", "")), turns


# ---- ingest -------------------------------------------------------------

def ingest(conv: dict, root: str, profile: str) -> None:
    done_path = os.path.join(root, ".state", "locomo_done.json")
    if not os.path.exists(os.path.join(root, "brain.toml")):
        Brain.init(root, device="eval")
        toml = os.path.join(root, "brain.toml")
        text = open(toml).read().replace('profile = "coding"', f'profile = "{profile}"')
        open(toml, "w").write(text)
    brain = Brain(root)
    done = json.load(open(done_path)) if os.path.exists(done_path) else []
    class RetryingProvider(CommandProvider):
        def complete_json(self, system, prompt, schema):
            for attempt in range(6):
                try:
                    return super().complete_json(system, prompt, schema)
                except Exception:
                    if attempt == 5:
                        raise
                    import time
                    time.sleep(min(60 * 2 ** attempt, 900))

    con = Consolidator(brain, provider=RetryingProvider("claude -p --output-format text",
                                                        os.path.join(root, ".state", "llm")),
                       log=lambda *_: None)
    for key, date, turns in sessions_of(conv):
        if key in done:
            continue
        meta = {"session": f"{conv['sample_id']}-{key}", "cwd": None, "date": date}
        with brain.lock(timeout=600):
            r = con._consolidate("locomo", meta, turns)
        done.append(key)
        json.dump(done, open(done_path, "w"))
        print(f"[{conv['sample_id']}] {key} {date}: +{len(r['applied'])} applied, {len(r['queued'])} queued",
              flush=True)


def approve_all(root: str) -> None:
    """The benchmark has no human reviewer: inbox proposals are accepted, as a user would in bulk."""
    from rewire import nodefmt, ops

    brain = Brain(root)
    for item in list(brain.iter_inbox()):
        try:
            ops.approve(brain, item.meta["inbox_id"])
        except Exception as exc:  # a failed apply stays out of memory, as it would for a user
            print("approve failed:", exc)


# ---- answer -------------------------------------------------------------

def backfill_transcripts(conv: dict, root: str) -> None:
    """Brains ingested before episodes existed: write the transcripts consolidation now keeps."""
    brain = Brain(root)
    brain.config.transcripts = "all"
    con = Consolidator.__new__(Consolidator)  # only the transcript writer is needed: no LLM
    con.brain = brain
    for key, date, turns in sessions_of(conv):
        session = f"{conv['sample_id']}-{key}"
        con._write_session("locomo", session, None, [], [], [], [], [], set(), date, con._transcript(turns))


def memories_for(index: Index, question: str, k: int, episodes: int = 0, also: list[str] | None = None) -> list[dict]:
    """What rewire's recall returns (the same path MCP and the CLI use), as Mem0-style memory rows."""
    from rewire import nodefmt, ops

    out = []
    for h in ops.recall(index.brain, index, question, k=k, scope="all", episodes=episodes, also=also):
        if h["type"] == "episode":
            out.append({"memory": "Conversation excerpt: " + " / ".join(h["text"].splitlines()),
                        "created_at": h["date"]})
            continue
        row = index.rows("id = ?", (h["id"],))[0]
        meta = json.loads(row["meta"])
        src = nodefmt.parse_source(meta.get("source"))
        m = re.search(r"session_(\d+)$", src[1]) if src else None
        out.append({"memory": f"{row['title']}: {row['body'].strip()}" if row["body"].strip() != row["title"]
                    else row["title"], "created_at": meta.get("valid_from") or meta.get("recorded_at", "")[:10],
                    "source": f"D{m.group(1)}:{src[2]}" if m else None})  # the LoCoMo turn (dia_id) it came from
    return out


def link_sources(root: str) -> int:
    """Brains ingested before sources kept the full session id ('conv-26-'): restore it from git.

    Each consolidation is one commit ("consolidate locomo/<session>") adding that session's nodes.
    """
    from rewire import nodefmt

    brain = Brain(root)
    log = subprocess.run(["git", "-C", root, "log", "--reverse", "--name-status", "--format=@@%s"],
                         capture_output=True, text=True).stdout
    first: dict[str, str] = {}
    session = None
    for line in log.splitlines():
        if line.startswith("@@"):
            m = re.match(r"@@consolidate locomo/(\S+)", line)
            session = m.group(1) if m else None
        elif session and line.startswith("A\t") and line.endswith(".md") and "/nodes/" in "/" + line[2:]:
            first.setdefault(line[2:].split("/")[-1], session)
    fixed = 0
    for node in brain.iter_nodes():
        src = node.meta.get("source")
        if not isinstance(src, str) or nodefmt.parse_source(src) and not src.split(" · ")[1].endswith("-"):
            continue
        session = first.get(os.path.basename(node.path))
        m = re.search(r"turn (\d+)", src)
        if session and m:
            brain.write_node({**node.meta, "source": {"harness": "locomo", "session": session,
                                                      "turn": int(m.group(1))}}, node.body, node.path)
            fixed += 1
    return fixed


def answer_batch(batch: list[dict], ref_date: str) -> dict:
    blocks = []
    for q in batch:
        lines = ["The following memories are presented in chronological order (oldest to newest)."]
        for m in sorted(q["memories"], key=lambda m: m["created_at"]):
            lines.append(f"({m['created_at'] or 'unknown date'}) {m['memory']}")
        if not q["memories"]:
            lines = ["(No relevant memories found)"]
        blocks.append(f"### QUESTION {q['qid']}\n" + "\n".join(lines) + f"\n\nQuestion: {q['question']}")
    head = ANSWER_GENERATION_PROMPT.split("{memories}")[0].replace("{reference_date}", ref_date)
    prompt = (head + "\n\nYou will answer SEVERAL independent questions. Each has its OWN memory list; use only "
              "that list for that question.\n\n" + "\n\n".join(blocks) +
              "\n\nWork through Steps 1-7 for each question privately. Then output ONLY a JSON object mapping "
              "each QUESTION id to its final concise answer string.")
    return {str(k): str(v) for k, v in claude_json(prompt).items()}


def judge_batch(batch: list[dict]) -> dict:
    items = "\n\n".join(
        f"### ITEM {q['qid']}\n" + JUDGE_PROMPT.split("## Question")[1].strip()
        .replace("{question}", q["question"]).replace("{answer}", str(q["gold"])).replace("{response}", q["response"])
        .split("Return JSON")[0].strip() for q in batch)
    rules = JUDGE_PROMPT.split("## Question")[0]
    prompt = (rules + "\nJudge EACH item below independently with these rules.\n\n" + items +
              '\n\nReturn ONLY a JSON object mapping each ITEM id to "CORRECT" or "WRONG".')
    return {str(k): str(v).upper() for k, v in claude_json(prompt).items()}


def f1(pred: str, gold: str) -> float:
    def norm(s):
        s = str(s).lower()
        s = "".join(ch for ch in s if ch not in string.punctuation)
        return [w for w in s.split() if w not in ("a", "an", "the", "and")]
    p, g = norm(pred), norm(gold)
    common = sum((Counter(p) & Counter(g)).values())
    if not common:
        return 0.0
    prec, rec = common / len(p), common / len(g)
    return 2 * prec * rec / (prec + rec)


def run_conv(conv, out, k, label, skip_ingest, profile="coding", episodes=0):
    root = os.path.join(out, "brains", conv["sample_id"])
    if not skip_ingest:
        ingest(conv, root, profile)
        approve_all(root)
    if episodes and not glob.glob(os.path.join(root, "sessions", "*", "locomo-*session-*.md")):
        backfill_transcripts(conv, root)
    link_sources(root)
    brain = Brain(root)
    index = Index(brain)
    index.refresh()
    dates = [session_date(v) for kk, v in conv["conversation"].items() if kk.endswith("_date_time")]
    ref = max(d for d in dates if d)
    qs = []
    for i, qa in enumerate(conv["qa"]):
        if qa.get("category") not in CATS:
            continue
        gold = qa.get("answer", qa.get("adversarial_answer", ""))
        if qa["category"] == 3 and ";" in str(gold):
            gold = str(gold).split(";")[0].strip()
        qs.append({"qid": f"{conv['sample_id']}-{i}", "question": qa["question"], "gold": gold,
                   "category": qa["category"], "memories": memories_for(index, qa["question"], k, episodes)})
    cache = os.path.join(out, f"answers-{label}-{conv['sample_id']}.json")
    answers = json.load(open(cache)) if os.path.exists(cache) else {}
    todo = [q for q in qs if q["qid"] not in answers]
    with ThreadPoolExecutor(INNER) as pool:
        for res in pool.map(lambda b: answer_batch(b, ref), [todo[i:i + 8] for i in range(0, len(todo), 8)]):
            answers.update(res)
            json.dump(answers, open(cache, "w"), indent=1)
    for q in qs:
        q["response"] = answers.get(q["qid"], "")
    jcache = os.path.join(out, f"judged-{label}-{conv['sample_id']}.json")
    judged = json.load(open(jcache)) if os.path.exists(jcache) else {}
    todo = [q for q in qs if q["qid"] not in judged]
    with ThreadPoolExecutor(INNER) as pool:
        for res in pool.map(judge_batch, [todo[i:i + 20] for i in range(0, len(todo), 20)]):
            judged.update(res)
            json.dump(judged, open(jcache, "w"), indent=1)
    for q in qs:
        q["label"] = judged.get(q["qid"], "WRONG")
        q["f1"] = f1(q["response"], q["gold"])
        q["n_memories"] = len(q["memories"])
    return qs


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--data", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--convs", default="conv-26,conv-30,conv-49")
    ap.add_argument("--k", type=int, default=50)
    ap.add_argument("--label", default="run")
    ap.add_argument("--skip-ingest", action="store_true")
    ap.add_argument("--profile", default="coding")
    ap.add_argument("--episodes", type=int, default=0, help="raw transcript excerpts added to the answer context")
    ap.add_argument("--parallel", type=int, default=3, help="conversations processed at once")
    ap.add_argument("--inner", type=int, default=4, help="concurrent LLM calls per conversation")
    args = ap.parse_args()
    global INNER
    INNER = args.inner
    data = {c["sample_id"]: c for c in json.load(open(args.data))}
    convs = list(data.values()) if args.convs == "all" else [data[c] for c in args.convs.split(",")]
    os.makedirs(args.out, exist_ok=True)
    def safe(c):
        try:
            return run_conv(c, args.out, args.k, args.label, args.skip_ingest, args.profile, args.episodes)
        except Exception as exc:  # one conversation failing must not lose the others; rerun resumes it
            print(f"[{c['sample_id']}] FAILED: {exc}", flush=True)
            return []

    with ThreadPoolExecutor(min(args.parallel, len(convs))) as pool:
        results = list(pool.map(safe, convs))
    qs = [q for r in results for q in r]
    json.dump(qs, open(os.path.join(args.out, f"results-{args.label}.json"), "w"), indent=1, ensure_ascii=False)
    by = defaultdict(list)
    for q in qs:
        by[CATS[q["category"]]].append(q)
        by["overall"].append(q)
    print(f"\n{args.label}: k={args.k}, {len(qs)} questions")
    print("| category | n | J (Mem0 judge) | F1 |")
    print("|---|---|---|---|")
    for name in ["single-hop", "multi-hop", "temporal", "open-domain", "overall"]:
        g = by[name]
        if g:
            j = sum(q["label"] == "CORRECT" for q in g) / len(g)
            print(f"| {name} | {len(g)} | {j:.1%} | {sum(q['f1'] for q in g) / len(g):.3f} |")


if __name__ == "__main__":
    main()
