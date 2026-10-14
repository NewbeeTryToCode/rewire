"""Minimal OpenAI-compatible chat client (OpenRouter by default) with token and cost accounting.

No dependencies: urllib only (it honours HTTPS_PROXY). Every call is recorded in a Ledger under a
stage name (ingest / answer / judge), so a run can report tokens per stage, tokens per response,
cost and latency.

Environment:
  OPENROUTER_API_KEY   required for real calls (store it as an environment secret, never in code)
  OPENROUTER_BASE_URL  default https://openrouter.ai/api/v1 (any OpenAI-compatible endpoint works)
"""

import json
import os
import random
import threading
import time
import urllib.error
import urllib.request

BASE_URL = os.environ.get("OPENROUTER_BASE_URL", "https://openrouter.ai/api/v1").rstrip("/")


class BudgetExceeded(RuntimeError):
    pass


class Ledger:
    """Thread-safe record of every LLM call: tokens, cost, latency, per stage."""

    def __init__(self, max_cost: float | None = None):
        self.calls: list[dict] = []
        self.max_cost = max_cost
        self._lock = threading.Lock()

    def add(self, call: dict) -> None:
        with self._lock:
            self.calls.append(call)

    def cost(self) -> float:
        with self._lock:
            return sum(c.get("cost") or 0.0 for c in self.calls)

    def check_budget(self) -> None:
        if self.max_cost is not None and self.cost() >= self.max_cost:
            raise BudgetExceeded(f"budget of ${self.max_cost:.2f} reached (${self.cost():.4f} spent)")

    def summary(self) -> dict:
        out: dict[str, dict] = {}
        with self._lock:
            calls = list(self.calls)
        for c in calls:
            s = out.setdefault(c["stage"], {"calls": 0, "prompt_tokens": 0, "completion_tokens": 0,
                                            "reasoning_tokens": 0, "cost": 0.0, "latency_ms": []})
            s["calls"] += 1
            s["prompt_tokens"] += c.get("prompt_tokens") or 0
            s["completion_tokens"] += c.get("completion_tokens") or 0
            s["reasoning_tokens"] += c.get("reasoning_tokens") or 0
            s["cost"] += c.get("cost") or 0.0
            s["latency_ms"].append(c.get("latency_ms") or 0)
        for s in out.values():
            lat = sorted(s.pop("latency_ms"))
            s["latency_p50_ms"] = lat[len(lat) // 2] if lat else 0
            s["latency_p95_ms"] = lat[min(len(lat) - 1, int(len(lat) * 0.95))] if lat else 0
            s["prompt_tokens_per_call"] = s["prompt_tokens"] / s["calls"]
            s["completion_tokens_per_call"] = s["completion_tokens"] / s["calls"]
        return out


class Client:
    def __init__(self, ledger: Ledger, api_key: str | None = None, base_url: str = BASE_URL, timeout: int = 300):
        self.ledger = ledger
        self.api_key = api_key or os.environ.get("OPENROUTER_API_KEY", "")
        self.base_url = base_url
        self.timeout = timeout

    def chat(self, model: str, messages: list[dict], stage: str, max_tokens: int = 2000,
             temperature: float = 0.0, attempts: int = 6) -> dict:
        """One chat completion -> the ledger record plus "text". Retries 429/5xx/network errors."""
        if not self.api_key:
            raise RuntimeError("OPENROUTER_API_KEY is not set")
        self.ledger.check_budget()
        body = {"model": model, "messages": messages, "max_tokens": max_tokens, "temperature": temperature,
                "usage": {"include": True}}  # OpenRouter: return the exact cost of the call
        data = json.dumps(body).encode()
        err = ""
        for attempt in range(attempts):
            req = urllib.request.Request(f"{self.base_url}/chat/completions", data=data, method="POST", headers={
                "Authorization": f"Bearer {self.api_key}", "Content-Type": "application/json",
                "X-Title": "rewire-eval"})
            start = time.time()
            try:
                with urllib.request.urlopen(req, timeout=self.timeout) as resp:
                    out = json.loads(resp.read())
                latency = int((time.time() - start) * 1000)
                if out.get("error"):
                    raise RuntimeError(str(out["error"])[:300])
                text = out["choices"][0]["message"].get("content") or ""
                usage = out.get("usage") or {}
                details = usage.get("completion_tokens_details") or {}
                call = {"stage": stage, "model": model, "prompt_tokens": usage.get("prompt_tokens") or 0,
                        "completion_tokens": usage.get("completion_tokens") or 0,
                        "reasoning_tokens": details.get("reasoning_tokens") or 0,
                        "cost": usage.get("cost") or 0.0, "latency_ms": latency}
                self.ledger.add(call)
                return {**call, "text": text}
            except urllib.error.HTTPError as exc:
                err = f"HTTP {exc.code}: {exc.read()[:300]!r}"
                if exc.code not in (408, 409, 429) and exc.code < 500:
                    raise RuntimeError(err) from exc  # bad request / auth: retrying will not help
            except (urllib.error.URLError, TimeoutError, KeyError, json.JSONDecodeError, RuntimeError) as exc:
                err = str(exc)[:300]
            time.sleep(min(2 ** attempt + random.random(), 60))
        raise RuntimeError(f"{model}: failed after {attempts} attempts: {err}")


def model_prices(client: Client) -> dict[str, tuple[float, float]]:
    """model id -> (USD per 1M input tokens, USD per 1M output tokens), from OpenRouter's catalog."""
    req = urllib.request.Request(f"{client.base_url}/models")
    with urllib.request.urlopen(req, timeout=60) as resp:
        data = json.loads(resp.read())["data"]
    return {m["id"]: (float(m["pricing"]["prompt"]) * 1e6, float(m["pricing"]["completion"]) * 1e6) for m in data}


class JsonProvider:
    """rewire's consolidation Provider interface (complete_json) on top of Client, for --mode full."""

    def __init__(self, client: Client, model: str, stage: str = "ingest"):
        self.client, self.model, self.stage = client, model, stage

    def complete_json(self, system: str, prompt: str, schema: dict) -> dict:
        from rewire.llm import extract_json

        instruction = ("\n\nReturn ONLY a JSON object that matches this JSON schema, with no other text:\n"
                       + json.dumps(schema))
        call = self.client.chat(self.model, [{"role": "system", "content": system},
                                             {"role": "user", "content": prompt + instruction}],
                                stage=self.stage, max_tokens=8000)
        return extract_json(call["text"])
