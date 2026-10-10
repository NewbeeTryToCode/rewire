"""ACT-R base-level activation computed from the event log.

B(i) = ln( sum_j w_j * (now - t_j)^(-d) )

Only real use counts (exposure in a snapshot or a recall listing does not),
capped at one event per node per session (the highest weight wins).
Node creation counts as the first presentation. See experiments/phase0.
"""

import math
from collections import defaultdict
from datetime import datetime, timezone

from .util import parse_iso

USE_EVENTS = ("use", "confirm")
MIN_AGE_H = 0.5
FLOOR = -20.0


def collect_uses(events, nodes: dict) -> dict[str, list[tuple[datetime, float]]]:
    per_session: dict[tuple[str, str], tuple[datetime, float]] = {}
    for ev in events:
        if ev.get("ev") not in USE_EVENTS or ev.get("node") not in nodes:
            continue
        key = (ev["node"], ev.get("session") or ev["t"])
        when, weight = parse_iso(ev["t"]), float(ev.get("w", 1.0))
        if key not in per_session or weight > per_session[key][1]:
            per_session[key] = (when, weight)
    uses = defaultdict(list)
    for (node_id, _), use in per_session.items():
        uses[node_id].append(use)
    for node_id, node in nodes.items():
        uses[node_id].append((parse_iso(node.meta["recorded_at"]), 1.0))
    return uses


def base_level(uses: list[tuple[datetime, float]], d: float, now: datetime | None = None) -> float:
    now = now or datetime.now(timezone.utc)
    total = 0.0
    for when, weight in uses:
        age_h = max((now - when).total_seconds() / 3600.0, MIN_AGE_H)
        total += weight * age_h ** (-d)
    return math.log(total) if total > 0 else FLOOR


def compute(events, nodes: dict, d: float, now: datetime | None = None) -> dict[str, float]:
    uses = collect_uses(events, nodes)
    return {node_id: base_level(uses[node_id], d, now) for node_id in nodes}


def squash(b: float) -> float:
    """Map activation to 0..1 for mixing with text relevance."""
    return 1.0 / (1.0 + math.exp(-(b + 2.0)))
