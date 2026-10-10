import os
import re
import secrets
import time
from datetime import datetime, timezone

_CROCKFORD = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"


def ulid() -> str:
    """26-char ULID: 48-bit millisecond timestamp + 80 random bits."""
    value = (int(time.time() * 1000) << 80) | secrets.randbits(80)
    out = []
    for _ in range(26):
        out.append(_CROCKFORD[value & 31])
        value >>= 5
    return "".join(reversed(out))


def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def parse_iso(value: str) -> datetime:
    return datetime.fromisoformat(value.replace("Z", "+00:00"))


def hours_since(value: str, now: datetime | None = None) -> float:
    now = now or datetime.now(timezone.utc)
    return (now - parse_iso(value)).total_seconds() / 3600.0


def slugify(text: str, max_len: int = 48) -> str:
    slug = re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-")
    return slug[:max_len].rstrip("-") or "node"


def atomic_write(path: str, content: str) -> None:
    os.makedirs(os.path.dirname(path), exist_ok=True)
    tmp = f"{path}.tmp-{os.getpid()}"
    with open(tmp, "w", encoding="utf-8") as fh:
        fh.write(content)
    os.replace(tmp, path)
