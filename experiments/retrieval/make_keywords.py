"""Generate keywords for the dataset memories exactly as consolidation would.

Uses the same KEYWORDS_RULE text the consolidation prompt uses. The generator
sees only the memories, never the eval queries.
"""

import json
import os
import subprocess
import sys

HERE = os.path.dirname(__file__)
sys.path.insert(0, os.path.join(HERE, "..", "..", "src"))

from rewire.consolidate import KEYWORDS_RULE, normalize_keywords  # noqa: E402


def main():
    with open(os.path.join(HERE, "dataset.json"), encoding="utf-8") as fh:
        memories = json.load(fh)["memories"]
    listing = "\n".join(f"{m['id']}: {m['text']}" for m in memories)
    prompt = (f"For each memory below, write {KEYWORDS_RULE}\n\nReturn ONLY a JSON object mapping memory id "
              f"to its list of keywords.\n\n{listing}")
    r = subprocess.run(["claude", "-p", "--output-format", "text"], input=prompt, capture_output=True, text=True,
                       timeout=900, cwd=HERE, env={**os.environ, "REWIRE_INTERNAL": "1"})
    if r.returncode != 0:
        sys.exit(r.stderr)
    raw = json.loads(r.stdout[r.stdout.find("{"): r.stdout.rfind("}") + 1])
    keywords = {k: normalize_keywords(v) for k, v in raw.items()}
    with open(os.path.join(HERE, "keywords.json"), "w", encoding="utf-8") as fh:
        json.dump(keywords, fh, ensure_ascii=False, indent=1)
    print(f"keywords for {len(keywords)} memories")


if __name__ == "__main__":
    main()
