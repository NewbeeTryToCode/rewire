"""Consolidation command for --mode full: rewire's command provider pipes the prompt to this script, which answers
through the harness's client (OpenRouter, or the local Claude CLI for claude-cli/<model>) and logs every call.

    python llm_cmd.py --model claude-cli/claude-haiku-5-5 --ledger BRAIN/.state/ingest_calls.jsonl < prompt

A reply without a JSON object is asked again (up to 3 times), with a larger token limit when it was cut off.
"""

import argparse
import json
import os
import re
import sys
import threading

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from openrouter import Client, Ledger, normalize_model  # noqa: E402

_LOCK = threading.Lock()


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--model", required=True)
    ap.add_argument("--ledger", required=True, help="JSON lines: one record per LLM call (tokens, cost, latency)")
    ap.add_argument("--max-tokens", type=int, default=16000)
    args = ap.parse_args()
    prompt = sys.stdin.read()
    client = Client(Ledger())
    model, max_tokens = normalize_model(args.model), args.max_tokens
    for attempt in range(3):
        call = client.chat(model, [{"role": "user", "content": prompt}], stage="ingest", max_tokens=max_tokens)
        ok = bool(re.search(r"\{.*\}", call["text"], re.S))
        rec = {k: call.get(k) for k in ("prompt_tokens", "completion_tokens", "reasoning_tokens", "cost", "latency_ms",
                                         "finish_reason")}
        rec.update(model=model, attempt=attempt, json=ok, prompt_chars=len(prompt), reply_chars=len(call["text"]))
        os.makedirs(os.path.dirname(os.path.abspath(args.ledger)), exist_ok=True)
        with _LOCK, open(args.ledger, "a", encoding="utf-8") as fh:
            fh.write(json.dumps(rec) + "\n")
        if ok:
            sys.stdout.write(call["text"])
            return
        if call.get("finish_reason") == "length":
            max_tokens *= 2
    sys.stdout.write(call["text"])  # rewire reports the unreadable reply


if __name__ == "__main__":
    main()
