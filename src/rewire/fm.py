"""Markdown frontmatter without a YAML dependency.

Every value is written as JSON, which is also valid YAML, so files stay
readable by Obsidian and other YAML tools while we parse them with json.
"""

import json

DELIM = "---"


def dumps(meta: dict, body: str) -> str:
    lines = [DELIM]
    for key, value in meta.items():
        lines.append(f"{key}: {json.dumps(value, ensure_ascii=False)}")
    lines.append(DELIM)
    return "\n".join(lines) + "\n" + body.strip() + "\n"


def loads(text: str) -> tuple[dict, str]:
    if not text.startswith(DELIM + "\n"):
        return {}, text
    end = text.find("\n" + DELIM, len(DELIM))
    if end == -1:
        return {}, text
    block = text[len(DELIM) + 1 : end]
    body = text[end + len(DELIM) + 1 :].lstrip("\n")
    # Fast path for files rewire wrote (every value is JSON): one parse per file.
    pairs = []
    for line in block.splitlines():
        key, sep, raw = line.partition(":")
        if not sep or not raw.strip() or line.lstrip().startswith("#"):
            pairs = None
            break
        pairs.append(f"{json.dumps(key.strip())}:{raw.strip()}")
    if pairs is not None:
        try:
            return json.loads("{" + ",".join(pairs) + "}"), body
        except json.JSONDecodeError:
            pass
    meta = {}
    for line in block.splitlines():
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        key, sep, raw = line.partition(":")
        if not sep:
            raise ValueError(f"invalid frontmatter line: {line!r}")
        raw = raw.strip()
        try:
            meta[key.strip()] = json.loads(raw) if raw else None
        except json.JSONDecodeError:
            meta[key.strip()] = raw  # hand-written plain YAML scalar
    return meta, body
