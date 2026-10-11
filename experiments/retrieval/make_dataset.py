"""Generate the retrieval eval dataset with an independent LLM call (`claude -p`).

The dataset is written once and committed, so every run of eval.py is
reproducible. Regenerate only on purpose: python experiments/retrieval/make_dataset.py
"""

import json
import os
import subprocess
import sys

OUT = os.path.join(os.path.dirname(__file__), "dataset.json")

PROMPT = """You are building a test set for a personal memory system used by AI coding agents.
Invent ONE realistic developer persona who works on 3 projects ("atlas" a payments API in Go,
"kebun" a mobile app in Flutter, "notes" a personal Python CLI). The persona is Indonesian and
mixes Indonesian and English when writing notes.

Produce 70 memories. Each memory is one self-contained claim (1-2 sentences):
- types: constraint, preference, fact, decision, person, concept
- scope: "global" or "project:atlas" / "project:kebun" / "project:notes"
- about 60% written in Indonesian, 40% in English
- relations: some memories have "links" to other memories by id with rel one of
  part_of, depends_on, related_to (about 20 links total)
- 6 memories are OLD facts that were later superseded: include both the old one (with
  "superseded_by": <new id>) and the new one.
- include near-duplicates across projects on purpose (e.g. deploy steps for atlas vs kebun).
Use ids m01..m70.

Then produce 60 queries an agent would ask while working, 10 per category:
- "lexical": shares the key words with the answer memory
- "paraphrase": same meaning but shares NO content words with the answer (synonyms, rephrasing)
- "cross_lingual": query in Indonesian when the answer is English, or the other way round
- "relational": the answer is a memory that is LINKED to a memory the query mentions; the answer
  itself shares no content words with the query
- "stale": asks about something that changed; gold is the NEW memory; list the old one in "stale"
- "scope": asked while working in a given project; a similar memory exists in another project
  (list it in "wrong_scope"); gold is the one in the current project
Each query: {"q": text, "category": ..., "project": "atlas"|"kebun"|"notes"|null,
"gold": [memory ids that answer it], "stale": [ids], "wrong_scope": [ids]}.

Return ONLY JSON: {"memories": [{"id","type","scope","text","title","links":[{"rel","to"}],
"superseded_by": id or null}], "queries": [...]}. Titles are short (max 8 words)."""


def main():
    r = subprocess.run(["claude", "-p", "--output-format", "text"], input=PROMPT, capture_output=True,
                       text=True, timeout=900, cwd=os.path.dirname(OUT),
                       env={**os.environ, "REWIRE_INTERNAL": "1"})
    if r.returncode != 0:
        sys.exit(r.stderr)
    text = r.stdout
    data = json.loads(text[text.find("{"): text.rfind("}") + 1])
    ids = {m["id"] for m in data["memories"]}
    for q in data["queries"]:
        q["gold"] = [g for g in q.get("gold", []) if g in ids]
    data["queries"] = [q for q in data["queries"] if q["gold"]]
    with open(OUT, "w", encoding="utf-8") as fh:
        json.dump(data, fh, ensure_ascii=False, indent=1)
    print(f"{len(data['memories'])} memories, {len(data['queries'])} queries -> {OUT}")


if __name__ == "__main__":
    main()
