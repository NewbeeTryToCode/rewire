"""Write-time scanning: secrets, injection patterns, private spans."""

import math
import re

PRIVATE = re.compile(r"<private>.*?</private>", re.S | re.I)

SECRET_PATTERNS = [
    re.compile(r"sk-ant-[A-Za-z0-9_\-]{20,}"),
    re.compile(r"sk-[A-Za-z0-9]{32,}"),
    re.compile(r"gh[pousr]_[A-Za-z0-9]{30,}"),
    re.compile(r"github_pat_[A-Za-z0-9_]{40,}"),
    re.compile(r"AKIA[0-9A-Z]{16}"),
    re.compile(r"xox[baprs]-[A-Za-z0-9-]{10,}"),
    re.compile(r"AIza[0-9A-Za-z_\-]{35}"),
    re.compile(r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
    re.compile(r"(?i)\b(password|passwd|secret|api[_-]?key|token)\s*[:=]\s*\S{8,}"),
]

INJECTION_PATTERNS = [
    re.compile(r"(?i)ignore (all |any )?(previous|prior|above) (instructions|prompts)"),
    re.compile(r"(?i)\b(always|must) (send|upload|post|exfiltrate|forward)\b.*\b(to|at)\b"),
    re.compile(r"(?i)\b(curl|wget)\s+https?://"),
    re.compile(r"(?i)disregard (the )?(system|developer) (prompt|message)"),
    re.compile(r"(?i)you are now\b"),
    re.compile(r"[​-‏‪-‮⁠-⁤﻿]"),  # invisible / bidi control chars
]


def strip_private(text: str) -> str:
    return PRIVATE.sub("[private]", text)


def _high_entropy_token(text: str) -> bool:
    for tok in re.findall(r"[A-Za-z0-9+/_\-]{32,}", text):
        counts = {c: tok.count(c) for c in set(tok)}
        entropy = -sum(n / len(tok) * math.log2(n / len(tok)) for n in counts.values())
        if entropy > 4.3 and any(c.isdigit() for c in tok) and any(c.isalpha() for c in tok):
            return True
    return False


def scan(text: str) -> list[str]:
    """Return a list of problems; empty means safe to store."""
    problems = []
    if any(p.search(text) for p in SECRET_PATTERNS) or _high_entropy_token(text):
        problems.append("secret")
    if any(p.search(text) for p in INJECTION_PATTERNS):
        problems.append("injection")
    return problems
