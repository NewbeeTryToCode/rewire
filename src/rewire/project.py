"""Resolve the active project from a working directory."""

import os
import subprocess

from .util import slugify


def _git(cwd: str, *args: str) -> str:
    try:
        r = subprocess.run(["git", "-C", cwd, *args], capture_output=True, text=True, timeout=5)
        return r.stdout.strip() if r.returncode == 0 else ""
    except (OSError, subprocess.TimeoutExpired):
        return ""


def detect(cwd: str | None, projects: dict) -> str | None:
    """Return a project slug, or None when cwd is not inside a project."""
    if not cwd or not os.path.isdir(cwd):
        return None
    top = _git(cwd, "rev-parse", "--show-toplevel")
    remote = _git(cwd, "remote", "get-url", "origin")
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
