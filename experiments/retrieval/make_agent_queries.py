"""Rewrite eval questions the way an agent calls `recall`, following the tool description.

The agent (an LLM) writes the query string. We give it the same guidance the MCP
tool description gives (QUERY_GUIDE) and only the question - never the memories.
"""

import json
import os
import subprocess
import sys

HERE = os.path.dirname(__file__)
sys.path.insert(0, os.path.join(HERE, "..", "..", "src"))

from rewire.mcp import QUERY_GUIDE  # noqa: E402


def main():
    with open(os.path.join(HERE, "dataset.json"), encoding="utf-8") as fh:
        queries = json.load(fh)["queries"]
    listing = "\n".join(f"{i}: {q['q']}" for i, q in enumerate(queries))
    prompt = (f"You are an AI coding agent about to call a memory search tool. Tool guidance: {QUERY_GUIDE}\n\n"
              "For each numbered question below, write the query string you would send. Return ONLY a JSON "
              f"object mapping the number to the query string.\n\n{listing}")
    r = subprocess.run(["claude", "-p", "--output-format", "text"], input=prompt, capture_output=True, text=True,
                       timeout=900, cwd=HERE, env={**os.environ, "REWIRE_INTERNAL": "1"})
    if r.returncode != 0:
        sys.exit(r.stderr)
    raw = json.loads(r.stdout[r.stdout.find("{"): r.stdout.rfind("}") + 1])
    with open(os.path.join(HERE, "agent_queries.json"), "w", encoding="utf-8") as fh:
        json.dump({int(k): v for k, v in raw.items()}, fh, ensure_ascii=False, indent=1)
    print(f"{len(raw)} agent queries")


if __name__ == "__main__":
    main()
