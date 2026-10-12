"""LLM providers for consolidation. Each returns a parsed JSON object.

- command:   any CLI that reads a prompt on stdin and prints text (e.g. `claude -p`).
             Runs with REWIRE_INTERNAL=1 inside the brain's .state/ dir so hooks
             skip it and its own transcript is never consolidated (no recursion).
- anthropic: the Anthropic SDK with structured outputs.
"""

import json
import os
import shlex
import subprocess

from .transcripts import INTERNAL_MARKER


class LLMError(Exception):
    pass


def extract_json(text: str) -> dict:
    text = text.strip()
    if text.startswith("```"):
        text = text.strip("`")
        text = text[text.find("\n") + 1:] if "\n" in text else text
    start, end = text.find("{"), text.rfind("}")
    if start == -1 or end <= start:
        raise LLMError(f"no JSON object in model output: {text[:200]!r}")
    try:
        return json.loads(text[start:end + 1])
    except json.JSONDecodeError as exc:
        try:  # an object followed by more text (or a second object): keep the first complete one
            return json.JSONDecoder().raw_decode(text[start:])[0]
        except json.JSONDecodeError:
            raise LLMError(f"invalid JSON from model: {exc}") from exc


class CommandProvider:
    def __init__(self, command: str, workdir: str, timeout: int = 900):
        self.argv = shlex.split(command)
        self.workdir = workdir
        self.timeout = timeout

    def complete_json(self, system: str, prompt: str, schema: dict) -> dict:
        os.makedirs(self.workdir, exist_ok=True)
        full = (f"{INTERNAL_MARKER}\n{system}\n\nRespond with ONLY a JSON object matching this JSON schema, "
                f"no prose, no code fences:\n{json.dumps(schema)}\n\n{prompt}")
        env = {**os.environ, "REWIRE_INTERNAL": "1"}
        try:
            r = subprocess.run(self.argv, input=full, capture_output=True, text=True,
                               cwd=self.workdir, env=env, timeout=self.timeout)
        except (OSError, subprocess.TimeoutExpired) as exc:
            raise LLMError(f"command provider failed: {exc}") from exc
        if r.returncode != 0:
            raise LLMError(f"command exited {r.returncode}: {r.stderr.strip()[:300]}")
        return extract_json(r.stdout)


class AnthropicProvider:
    def __init__(self, model: str, effort: str):
        try:
            import anthropic
        except ImportError as exc:
            raise LLMError("pip install 'rewire[anthropic]' to use the anthropic provider") from exc
        self.anthropic = anthropic
        self.client = anthropic.Anthropic()
        self.model = model
        self.effort = effort

    def complete_json(self, system: str, prompt: str, schema: dict) -> dict:
        try:
            response = self.client.beta.messages.create(
                model=self.model,
                max_tokens=16000,
                system=system,
                messages=[{"role": "user", "content": prompt}],
                output_config={"effort": self.effort,
                               "format": {"type": "json_schema", "schema": schema}},
                betas=["server-side-fallback-2026-07-01"],
                fallbacks="default",
            )
        except self.anthropic.APIConnectionError as exc:
            raise LLMError(f"connection error: {exc}") from exc
        except self.anthropic.APIStatusError as exc:
            raise LLMError(f"API error {exc.status_code}: {exc.message}") from exc
        if response.stop_reason == "refusal":
            raise LLMError("model declined the request")
        if response.stop_reason == "max_tokens":
            raise LLMError("output truncated (max_tokens)")
        text = next((b.text for b in response.content if b.type == "text"), "")
        return extract_json(text)


def make(config, brain_root: str):
    if config.provider == "command":
        return CommandProvider(config.command, os.path.join(brain_root, ".state", "llm"))
    if config.provider == "anthropic":
        return AnthropicProvider(config.model, config.effort)
    raise LLMError(f"unknown provider {config.provider}")
