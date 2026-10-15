"""Phase 0: simulation of the synaptic memory activation model (no LLM).

Tests the design claims with a synthetic event log over 180 days:
  H1  Exposure (being included in the snapshot) counted as "use" -> rich-get-richer.
  H2  Without decay exemptions, important constraints disappear from core.
  H3  ACT-R adapts to a project switch (abrupt drift) faster than pure frequency.
  H4  Spacing effect: spaced usage lasts longer than massed usage.

This validates the BEHAVIOR of the math, not real-world retrieval quality.
Run: python3 experiments/phase0/sim.py
"""

import math
import random

DAY = 24.0  # hours
D = 0.5  # ACT-R decay
CORE_N = 15
DAYS = 180
SWITCH_DAY = 90
SEEDS = range(20)


def base_level(uses, now, d=D):
    s = sum(w * max(now - t, 0.5) ** (-d) for t, w in uses if t <= now)
    return math.log(s) if s > 0 else -99.0


def make_nodes():
    nodes = []
    for i in range(5):
        nodes.append({"id": f"C{i}", "kind": "constraint", "exempt": True, "importance": 5})
    for i in range(10):
        nodes.append({"id": f"P{i}", "kind": "preference", "exempt": True, "importance": 3})
    for i in range(25):
        nodes.append({"id": f"A{i}", "kind": "projA", "exempt": False, "importance": 2})
    for i in range(25):
        nodes.append({"id": f"B{i}", "kind": "projB", "exempt": False, "importance": 2})
    return nodes


def zipf_pick(rng, items, k, s=1.1):
    weights = [1.0 / (r + 1) ** s for r in range(len(items))]
    return set(rng.choices(items, weights=weights, k=k))


INNER_DRIFT = {"on": False}


def true_usage(rng, day, nodes):
    """Nodes the session REALLY needs on that day (ground truth)."""
    proj = "projA" if day < SWITCH_DAY else "projB"
    proj_nodes = [n["id"] for n in nodes if n["kind"] == proj]
    if INNER_DRIFT["on"]:
        # the focus within the project shifts every 15 days (different features/modules)
        shift = (day // 15) * 5 % len(proj_nodes)
        proj_nodes = proj_nodes[shift:] + proj_nodes[:shift]
    used = zipf_pick(rng, proj_nodes, k=6)
    prefs = [n["id"] for n in nodes if n["kind"] == "preference"]
    if rng.random() < 0.3:
        used.add(rng.choice(prefs))
    # constraints are rarely needed, but when needed they must be present
    if rng.random() < 0.02:
        used.add(rng.choice([n["id"] for n in nodes if n["kind"] == "constraint"]))
    return used


def score_fn(policy, node, uses, now, d=D):
    if policy == "frequency":
        return float(len(uses))
    if policy == "recency":
        return -(now - max(t for t, _ in uses)) if uses else -1e9
    return base_level(uses, now, d)


def build_core(policy, nodes, events, now, exemptions, scope=None, separate_budget=False, d=D):
    fixed, ranked = [], []
    for n in nodes:
        if scope and n["kind"].startswith("proj") and n["kind"] != scope:
            continue  # the harness knows the active project (cwd/repo) -> only that project's nodes
        if exemptions and n["exempt"] and n["importance"] >= 4:
            fixed.append(n["id"])
            continue
        if exemptions and n["exempt"]:
            # preference: no decay -> score = number of uses, plus an importance bonus
            ranked.append((1e6 + n["importance"] * 10 + len(events[n["id"]]), n["id"]))
            continue
        ranked.append((score_fn(policy, n, events[n["id"]], now, d), n["id"]))
    ranked.sort(reverse=True)
    # separate_budget: constraints are short lines with their own budget,
    # they do not take up working-context slots
    slots = CORE_N if separate_budget else CORE_N - len(fixed)
    # preferences may fill at most 3 slots so the project context still fits
    pref_taken, chosen = 0, []
    for sc, nid in ranked:
        if len(chosen) >= slots:
            break
        if nid.startswith("P"):
            if pref_taken >= 3:
                continue
            pref_taken += 1
        if sc <= -99:
            continue
        chosen.append(nid)
    return fixed + chosen


def run(policy="actr", count_exposure=False, exemptions=True, seed=0,
        scoped=False, separate_budget=False, d=D):
    rng = random.Random(seed)
    nodes = make_nodes()
    events = {n["id"]: [] for n in nodes}
    # all constraints and preferences are taught on day 0
    for n in nodes:
        if n["kind"] in ("constraint", "preference"):
            events[n["id"]].append((0.0, 1.0))
    log = []
    for day in range(1, DAYS + 1):
        now = day * DAY
        scope = ("projA" if day < SWITCH_DAY else "projB") if scoped else None
        core = build_core(policy, nodes, events, now, exemptions, scope, separate_budget, d)
        used = true_usage(rng, day, nodes)
        # hit rate is computed over non-constraint needs so the separate budget is fair
        need = {u for u in used if not u.startswith("C")} or used
        hit = len(need & set(core)) / len(need)
        proj_slots = [c for c in core if c[0] in "AB"]
        cur = "B" if day >= SWITCH_DAY else "A"
        cur_share = sum(c[0] == cur for c in proj_slots) / max(len(proj_slots), 1)
        constraints_in = sum(c.startswith("C") for c in core)
        log.append((day, hit, cur_share, constraints_in, set(core)))
        for nid in used:
            events[nid].append((now + 1, 1.0))
        if count_exposure:
            for nid in core:
                if nid not in used:
                    events[nid].append((now + 1, 1.0))
    return log


def summarize(log):
    after = [l for l in log if l[0] > SWITCH_DAY]
    lag = next((l[0] - SWITCH_DAY for l in after if l[2] >= 0.8), None)
    hit_after = sum(l[1] for l in after) / len(after)
    hit_all = sum(l[1] for l in log) / len(log)
    constraints_end = log[-1][3]
    return {"lag80": lag, "hit_all": hit_all, "hit_after": hit_after, "constraints_end": constraints_end}


def avg(policy_kwargs):
    rows = [summarize(run(seed=s, **policy_kwargs)) for s in SEEDS]
    lags = [r["lag80"] for r in rows]
    never = sum(l is None for l in lags)
    lag_vals = [l for l in lags if l is not None]
    return {
        "lag80": (sum(lag_vals) / len(lag_vals)) if lag_vals else None,
        "never": never,
        "hit_all": sum(r["hit_all"] for r in rows) / len(rows),
        "hit_after": sum(r["hit_after"] for r in rows) / len(rows),
        "constraints_end": sum(r["constraints_end"] for r in rows) / len(rows),
    }


def spacing():
    now = 60 * DAY
    massed = [(1 * DAY + h, 1.0) for h in range(5)]  # 5x within 5 hours, day 1
    spaced = [(d * DAY, 1.0) for d in (1, 5, 10, 20, 30)]  # 5x spaced
    return base_level(massed, now), base_level(spaced, now)


def main():
    configs = [
        ("ACT-R, exposure=0, decay exemption (DESIGN)", dict(policy="actr")),
        ("ACT-R, exposure counted as use", dict(policy="actr", count_exposure=True)),
        ("ACT-R, WITHOUT decay exemption", dict(policy="actr", exemptions=False)),
        ("Pure frequency (count)", dict(policy="frequency")),
        ("Pure recency (last used)", dict(policy="recency")),
        ("WORKAROUND 1: ACT-R d=0.8 (faster decay for project nodes)", dict(policy="actr", d=0.8)),
        ("WORKAROUND 2: ACT-R + active project scope", dict(policy="actr", scoped=True)),
        ("WORKAROUND 2+3: + separate constraint budget (DESIGN REVISION)", dict(policy="actr", scoped=True, separate_budget=True)),
        ("Recency + scope + separate budget (baseline)", dict(policy="recency", scoped=True, separate_budget=True)),
        ("Frequency + scope + separate budget (baseline)", dict(policy="frequency", scoped=True, separate_budget=True)),
    ]
    print(f"Simulation of {DAYS} days, project switch on day {SWITCH_DAY}, core={CORE_N} slots, {len(SEEDS)} seeds\n")
    print("| Policy | Core hit rate (all) | Hit rate after project switch | Days until 80% of project slots = new project | Constraints in core on day 180 (of 5) |")
    print("|---|---|---|---|---|")
    for name, kw in configs:
        r = avg(kw)
        lag = "never" if r["lag80"] is None else f"{r['lag80']:.1f}"
        if r["never"] and r["lag80"] is not None:
            lag += f" ({r['never']}/{len(SEEDS)} seeds never)"
        print(f"| {name} | {r['hit_all']:.1%} | {r['hit_after']:.1%} | {lag} | {r['constraints_end']:.1f} |")
    INNER_DRIFT["on"] = True
    print("\nScenario 2: the focus within a project shifts every 15 days\n")
    print("| Policy | Core hit rate (all) | Hit rate after project switch |")
    print("|---|---|---|")
    for name, kw in configs[-3:]:
        r = avg(kw)
        print(f"| {name} | {r['hit_all']:.1%} | {r['hit_after']:.1%} |")
    INNER_DRIFT["on"] = False
    m, s = spacing()
    print(f"\nSpacing effect (evaluated on day 60): massed 5x/5 hours B={m:.2f}, spaced 5x/30 days B={s:.2f}")


if __name__ == "__main__":
    main()
