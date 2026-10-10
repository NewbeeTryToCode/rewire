"""The brain on disk: markdown nodes, append-only events, git history.

Source of truth is files; .index/ is derived and .state/ is device-local.
"""

import glob
import json
import os
import subprocess
import time
from dataclasses import dataclass

from . import config as config_mod
from . import fm, schema
from .util import atomic_write, now_iso, slugify, ulid

GITATTRIBUTES = "events/**/*.jsonl merge=union\n"
GITIGNORE = ".index/\n.state/\n"


def default_path() -> str:
    return os.path.expanduser(os.environ.get("REWIRE_BRAIN", "~/brain"))


@dataclass
class Node:
    meta: dict
    body: str
    path: str

    @property
    def id(self) -> str:
        return self.meta["id"]

    @property
    def type(self) -> str:
        return self.meta["type"]

    @property
    def active(self) -> bool:
        return not self.meta.get("retracted_at") and not self.meta.get("valid_to")


class Brain:
    def __init__(self, root: str | None = None):
        self.root = os.path.abspath(os.path.expanduser(root or default_path()))
        self.config = config_mod.load(self.path("brain.toml"))

    def path(self, *parts: str) -> str:
        return os.path.join(self.root, *parts)

    @property
    def exists(self) -> bool:
        return os.path.exists(self.path("brain.toml"))

    # ---- lifecycle -------------------------------------------------------

    @classmethod
    def init(cls, root: str | None = None, device: str | None = None) -> "Brain":
        root = os.path.abspath(os.path.expanduser(root or default_path()))
        if os.path.exists(os.path.join(root, "brain.toml")):
            raise FileExistsError(f"brain already exists at {root}")
        for sub in ("nodes", "events", "sessions", "inbox", ".index", ".state"):
            os.makedirs(os.path.join(root, sub), exist_ok=True)
        atomic_write(os.path.join(root, "brain.toml"), config_mod.render_default(device))
        atomic_write(os.path.join(root, ".gitattributes"), GITATTRIBUTES)
        atomic_write(os.path.join(root, ".gitignore"), GITIGNORE)
        brain = cls(root)
        # Only transcripts written after init are consolidated automatically;
        # older history can be imported explicitly with `rewire consolidate <file>`.
        brain.write_state("scan", {"since": now_iso(), "since_ts": time.time()})
        if not os.path.isdir(os.path.join(root, ".git")):
            subprocess.run(["git", "init", "-q", root], check=False, capture_output=True)
        brain.commit("init brain")
        return brain

    # ---- nodes -----------------------------------------------------------

    def iter_nodes(self):
        for path in sorted(glob.glob(self.path("nodes", "*", "*.md"))):
            with open(path, encoding="utf-8") as fh:
                meta, body = fm.loads(fh.read())
            if meta.get("id"):
                yield Node(meta, body, path)

    def nodes(self) -> dict[str, Node]:
        return {n.id: n for n in self.iter_nodes()}

    def get_node(self, node_id: str) -> Node | None:
        hits = glob.glob(self.path("nodes", "*", f"*-{node_id[-8:].lower()}.md"))
        for path in hits:
            with open(path, encoding="utf-8") as fh:
                meta, body = fm.loads(fh.read())
            if meta.get("id") == node_id:
                return Node(meta, body, path)
        return self.nodes().get(node_id)

    def new_meta(self, type_: str, title: str, **fields) -> dict:
        meta = {
            "id": ulid(),
            "type": type_,
            "title": title.strip(),
            "scope": fields.pop("scope", "global"),
            "importance": int(fields.pop("importance", 3)),
            "confidence": fields.pop("confidence", "inferred"),
            "provenance": fields.pop("provenance", "assistant"),
            "valid_from": fields.pop("valid_from", now_iso()[:10]),
            "valid_to": None,
            "recorded_at": now_iso(),
            "retracted_at": None,
            "edges": fields.pop("edges", []),
        }
        meta.update(fields)
        return meta

    def write_node(self, meta: dict, body: str, path: str | None = None) -> Node:
        schema.validate(meta, body)
        if path is None:
            slug = slugify(meta["title"])
            path = self.path("nodes", meta["type"], f"{slug}-{meta['id'][-8:].lower()}.md")
        atomic_write(path, fm.dumps(meta, body))
        return Node(meta, body, path)

    # ---- inbox -----------------------------------------------------------

    def iter_inbox(self):
        for path in sorted(glob.glob(self.path("inbox", "*.md"))):
            with open(path, encoding="utf-8") as fh:
                meta, body = fm.loads(fh.read())
            yield Node(meta, body, path)

    def write_inbox(self, meta: dict, body: str) -> str:
        item_id = meta.setdefault("inbox_id", ulid())
        path = self.path("inbox", f"{item_id}.md")
        atomic_write(path, fm.dumps(meta, body))
        return item_id

    # ---- events ----------------------------------------------------------

    def append_events(self, events: list[dict]) -> None:
        if not events:
            return
        month = now_iso()[:7]
        path = self.path("events", self.config.device, f"{month}.jsonl")
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "a", encoding="utf-8") as fh:
            for ev in events:
                ev.setdefault("t", now_iso())
                fh.write(json.dumps(ev, ensure_ascii=False, sort_keys=True) + "\n")

    def iter_events(self):
        for path in sorted(glob.glob(self.path("events", "*", "*.jsonl"))):
            with open(path, encoding="utf-8") as fh:
                for line in fh:
                    line = line.strip()
                    if line:
                        try:
                            yield json.loads(line)
                        except json.JSONDecodeError:
                            continue  # tolerate a torn line from a crash

    # ---- device-local state ---------------------------------------------

    def read_state(self, name: str) -> dict:
        path = self.path(".state", f"{name}.json")
        if not os.path.exists(path):
            return {}
        with open(path, encoding="utf-8") as fh:
            return json.load(fh)

    def write_state(self, name: str, data: dict) -> None:
        atomic_write(self.path(".state", f"{name}.json"), json.dumps(data, indent=1, sort_keys=True))

    # ---- git -------------------------------------------------------------

    def git(self, *args: str, check: bool = False) -> subprocess.CompletedProcess:
        cmd = ["git", "-C", self.root, "-c", "user.name=rewire", "-c", "user.email=rewire@localhost", *args]
        return subprocess.run(cmd, capture_output=True, text=True, check=check)

    def commit(self, message: str) -> bool:
        if not os.path.isdir(self.path(".git")):
            return False
        self.git("add", "-A")
        result = self.git("commit", "-q", "-m", message)
        return result.returncode == 0
