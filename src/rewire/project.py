"""Resolve the active project from a working directory."""

import os
import subprocess
import time

from .util import slugify


def _git(cwd: str, *args: str) -> str:
    try:
        r = subprocess.run(["git", "-C", cwd, *args], capture_output=True, text=True, timeout=5)
        return r.stdout.strip() if r.returncode == 0 else ""
    except (OSError, subprocess.TimeoutExpired):
        return ""


# cwd -> (checked at, git toplevel, origin url). Two git processes cost ~3 ms, which was most of a warm MCP
# recall; a long-running server asks for the same cwd on every call, and a repo's root and remote rarely move.
_GIT_TTL = 60.0
_git_cache: dict[str, tuple[float, str, str]] = {}


def _repo(cwd: str) -> tuple[str, str]:
    hit = _git_cache.get(cwd)
    if hit and time.monotonic() - hit[0] < _GIT_TTL:
        return hit[1], hit[2]
    top, remote = _git(cwd, "rev-parse", "--show-toplevel"), _git(cwd, "remote", "get-url", "origin")
    _git_cache[cwd] = (time.monotonic(), top, remote)
    return top, remote


def detect(cwd: str | None, projects: dict) -> str | None:
    """Return a project slug, or None when cwd is not inside a project."""
    if not cwd or not os.path.isdir(cwd):
        return None
    top, remote = _repo(cwd)
    candidates = [c for c in (remote, top, os.path.abspath(cwd)) if c]
    for slug, spec in projects.items():
        for pattern in spec.get("match", []):
            pat = os.path.expanduser(pattern)
            if any(pat in c for c in candidates):
                return slug
    if top:
        return slugify(os.path.basename(top))
    return None


def scopes_for(project: str | None) -> list[str]:
    return ["global"] + ([f"project:{project}"] if project else [])
