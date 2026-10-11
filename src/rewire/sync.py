"""Sync a brain across machines through an ordinary git remote.

Merge rules (docs/design/mvp-spec.md §8):
- events/: one file per device per month + `merge=union`, so they never conflict.
- nodes/: on a conflict the version changed most recently wins; the other
  version goes to the inbox so nothing is lost.
- anything else: keep ours, and say so.
"""

import os
import re
import subprocess

from . import fm
from .util import now_iso

GIT_TIMEOUT = 30


class SyncError(Exception):
    pass


def _git(brain, *args, timeout: int = GIT_TIMEOUT) -> subprocess.CompletedProcess:
    cmd = ["git", "-C", brain.root, "-c", "user.name=rewire", "-c", "user.email=rewire@localhost", *args]
    env = {**os.environ, "GIT_TERMINAL_PROMPT": "0"}  # never hang on a credential prompt in a hook
    try:
        return subprocess.run(cmd, capture_output=True, text=True, timeout=timeout, env=env)
    except subprocess.TimeoutExpired as exc:
        raise SyncError(f"git {args[0]} timed out") from exc


def remote_url(brain) -> str:
    r = _git(brain, "remote", "get-url", "origin")
    return r.stdout.strip() if r.returncode == 0 else ""


def branch(brain) -> str:
    return _git(brain, "rev-parse", "--abbrev-ref", "HEAD").stdout.strip() or "main"


def set_remote(brain, url: str) -> list[str]:
    notes = []
    if remote_url(brain):
        _git(brain, "remote", "set-url", "origin", url)
    else:
        _git(brain, "remote", "add", "origin", url)
    _enable_auto(brain)
    notes.append(f"origin = {url}; auto sync enabled (pull at session start, push after consolidation)")
    if re.search(r"github\.com|gitlab\.com|bitbucket\.org", url):
        notes.append("WARNING: your brain contains personal memory. Make sure this repository is PRIVATE.")
    return notes


def _enable_auto(brain) -> None:
    path = brain.path("brain.toml")
    _ = brain.device  # pin the device name locally before the legacy shared value is removed
    with open(path, encoding="utf-8") as fh:
        text = fh.read()
    text = re.sub(r'(?m)^device\s*=.*\n', "", text)  # legacy: device must not be shared
    if re.search(r"(?m)^\[sync\]", text):
        text = re.sub(r"(?ms)(^\[sync\][^\[]*?^auto\s*=\s*)false", r"\1true", text)
        if not re.search(r"(?ms)^\[sync\][^\[]*?^auto\s*=", text):
            text = re.sub(r"(?m)^\[sync\]\s*$", "[sync]\nauto = true", text)
    else:
        text = text.rstrip() + "\n\n[sync]\nauto = true\n"
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(text)
    brain.config.sync_auto = True
    brain.commit("enable sync")


def _stamp(meta: dict) -> str:
    return max(str(meta.get("recorded_at") or ""), str(meta.get("retracted_at") or ""))


def _stage(brain, n: int, path: str) -> str | None:
    r = _git(brain, "show", f":{n}:{path}")
    return r.stdout if r.returncode == 0 else None


def _resolve(brain) -> list[str]:
    """Resolve merge conflicts in place. Returns human-readable notes."""
    notes = []
    conflicted = _git(brain, "diff", "--name-only", "--diff-filter=U").stdout.split()
    for path in conflicted:
        full = brain.path(path)
        if path.startswith("nodes/"):
            ours, theirs = _stage(brain, 2, path), _stage(brain, 3, path)
            if ours is None or theirs is None:  # deleted on one side: keep the surviving content
                keep = ours if ours is not None else theirs
                with open(full, "w", encoding="utf-8") as fh:
                    fh.write(keep or "")
                notes.append(f"{path}: kept the side that was not deleted")
            else:
                m_ours, b_ours = fm.loads(ours)
                m_theirs, b_theirs = fm.loads(theirs)
                if _stamp(m_theirs) > _stamp(m_ours):
                    winner, (m_lose, b_lose) = theirs, (m_ours, b_ours)
                else:
                    winner, (m_lose, b_lose) = ours, (m_theirs, b_theirs)
                with open(full, "w", encoding="utf-8") as fh:
                    fh.write(winner)
                item = {k: m_lose.get(k) for k in ("type", "title", "scope", "importance", "confidence", "provenance")}
                item.update(action="UPDATE", target=m_lose.get("id"), created=now_iso(),
                            reason="sync conflict: this version lost to a newer edit from another device",
                            source={"harness": "sync"})
                brain.write_inbox(item, b_lose)
                notes.append(f"{path}: kept the newest version; the other one is in the inbox")
        else:
            _git(brain, "checkout", "--ours", "--", path)
            notes.append(f"{path}: conflict, kept this device's version")
        _git(brain, "add", "--", path)
    _git(brain, "add", "-A", "inbox")
    r = _git(brain, "commit", "--no-edit", "-m", "sync: merge")
    if r.returncode != 0:
        raise SyncError(f"could not complete merge: {r.stderr.strip() or r.stdout.strip()}")
    return notes


def pull(brain, timeout: int = GIT_TIMEOUT) -> list[str]:
    if not remote_url(brain):
        return ["no remote configured (rewire remote set <url>)"]
    brain.commit("sync: local changes")
    r = _git(brain, "fetch", "origin", timeout=timeout)
    if r.returncode != 0:
        raise SyncError(f"fetch failed: {r.stderr.strip()}")
    ref = f"origin/{branch(brain)}"
    if _git(brain, "rev-parse", "--verify", "-q", ref).returncode != 0:
        return ["remote has no brain yet"]
    behind = _git(brain, "rev-list", "--count", f"HEAD..{ref}").stdout.strip()
    if behind == "0":
        return ["already up to date"]
    r = _git(brain, "merge", "--no-edit", "--allow-unrelated-histories", ref)
    notes = [f"pulled {behind} commit(s)"]
    if r.returncode != 0:
        notes += _resolve(brain)
    return notes


def push(brain, timeout: int = GIT_TIMEOUT) -> list[str]:
    if not remote_url(brain):
        return ["no remote configured (rewire remote set <url>)"]
    brain.commit("sync: local changes")
    if _git(brain, "rev-parse", "--verify", "-q", f"origin/{branch(brain)}").returncode == 0 and \
            _git(brain, "rev-list", "--count", f"origin/{branch(brain)}..HEAD").stdout.strip() == "0":
        return ["nothing to push"]
    r = _git(brain, "push", "-u", "origin", "HEAD", timeout=timeout)
    if r.returncode != 0 and ("rejected" in r.stderr or "fetch first" in r.stderr):
        notes = pull(brain)
        r = _git(brain, "push", "-u", "origin", "HEAD")
        if r.returncode == 0:
            return notes + ["pushed"]
    if r.returncode != 0:
        raise SyncError(f"push failed: {r.stderr.strip()}")
    return ["pushed"]


def sync(brain) -> list[str]:
    return pull(brain) + push(brain)
