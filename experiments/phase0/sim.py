"""Fase 0: simulasi model activation synaptic memory (tanpa LLM).

Menguji klaim desain dengan event log sintetis selama 180 hari:
  H1  Eksposur (masuk snapshot) yang dihitung sebagai "pakai" -> rich-get-richer.
  H2  Tanpa pengecualian decay, constraint penting hilang dari core.
  H3  ACT-R beradaptasi ke pergantian proyek (abrupt drift) lebih cepat dari frekuensi murni.
  H4  Spacing effect: pemakaian tersebar lebih awet dari pemakaian massal.

Ini memvalidasi PERILAKU matematika, bukan kualitas retrieval di dunia nyata.
Jalankan: python3 experiments/phase0/sim.py
"""

import math
import random

DAY = 24.0  # jam
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
    """Node yang BENAR-BENAR dibutuhkan sesi pada hari itu (ground truth)."""
    proj = "projA" if day < SWITCH_DAY else "projB"
    proj_nodes = [n["id"] for n in nodes if n["kind"] == proj]
    if INNER_DRIFT["on"]:
        # fokus di dalam proyek bergeser tiap 15 hari (fitur/modul berbeda)
        shift = (day // 15) * 5 % len(proj_nodes)
        proj_nodes = proj_nodes[shift:] + proj_nodes[:shift]
    used = zipf_pick(rng, proj_nodes, k=6)
    prefs = [n["id"] for n in nodes if n["kind"] == "preference"]
    if rng.random() < 0.3:
        used.add(rng.choice(prefs))
    # constraint jarang dibutuhkan, tapi saat dibutuhkan harus ada
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
            continue  # harness tahu proyek aktif (cwd/repo) -> hanya node proyek itu
        if exemptions and n["exempt"] and n["importance"] >= 4:
            fixed.append(n["id"])
            continue
        if exemptions and n["exempt"]:
            # preference: tidak decay -> skor = jumlah pakai, ditambah bonus importance
            ranked.append((1e6 + n["importance"] * 10 + len(events[n["id"]]), n["id"]))
            continue
        ranked.append((score_fn(policy, n, events[n["id"]], now, d), n["id"]))
    ranked.sort(reverse=True)
    # separate_budget: constraint adalah baris pendek dengan budget sendiri,
    # tidak memakan slot konteks kerja
    slots = CORE_N if separate_budget else CORE_N - len(fixed)
    # preference hanya boleh mengisi maks 3 slot agar konteks proyek tetap muat
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
    # semua constraint dan preference diajarkan di hari 0
    for n in nodes:
        if n["kind"] in ("constraint", "preference"):
            events[n["id"]].append((0.0, 1.0))
    log = []
    for day in range(1, DAYS + 1):
        now = day * DAY
        scope = ("projA" if day < SWITCH_DAY else "projB") if scoped else None
        core = build_core(policy, nodes, events, now, exemptions, scope, separate_budget, d)
        used = true_usage(rng, day, nodes)
        # hit rate dihitung atas kebutuhan non-constraint agar budget terpisah adil
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
    massed = [(1 * DAY + h, 1.0) for h in range(5)]  # 5x dalam 5 jam, hari 1
    spaced = [(d * DAY, 1.0) for d in (1, 5, 10, 20, 30)]  # 5x tersebar
    return base_level(massed, now), base_level(spaced, now)


def main():
    configs = [
        ("ACT-R, eksposur=0, pengecualian decay (DESAIN)", dict(policy="actr")),
        ("ACT-R, eksposur dihitung sebagai pakai", dict(policy="actr", count_exposure=True)),
        ("ACT-R, TANPA pengecualian decay", dict(policy="actr", exemptions=False)),
        ("Frekuensi murni (count)", dict(policy="frequency")),
        ("Recency murni (terakhir dipakai)", dict(policy="recency")),
        ("WORKAROUND 1: ACT-R d=0.8 (decay lebih cepat untuk node proyek)", dict(policy="actr", d=0.8)),
        ("WORKAROUND 2: ACT-R + scope proyek aktif", dict(policy="actr", scoped=True)),
        ("WORKAROUND 2+3: + budget constraint terpisah (REVISI DESAIN)", dict(policy="actr", scoped=True, separate_budget=True)),
        ("Recency + scope + budget terpisah (pembanding)", dict(policy="recency", scoped=True, separate_budget=True)),
        ("Frekuensi + scope + budget terpisah (pembanding)", dict(policy="frequency", scoped=True, separate_budget=True)),
    ]
    print(f"Simulasi {DAYS} hari, ganti proyek di hari {SWITCH_DAY}, core={CORE_N} slot, {len(SEEDS)} seed\n")
    print("| Kebijakan | Hit rate core (semua) | Hit rate setelah ganti proyek | Hari sampai 80% slot proyek = proyek baru | Constraint di core hari 180 (dari 5) |")
    print("|---|---|---|---|---|")
    for name, kw in configs:
        r = avg(kw)
        lag = "tidak pernah" if r["lag80"] is None else f"{r['lag80']:.1f}"
        if r["never"] and r["lag80"] is not None:
            lag += f" ({r['never']}/{len(SEEDS)} seed tidak pernah)"
        print(f"| {name} | {r['hit_all']:.1%} | {r['hit_after']:.1%} | {lag} | {r['constraints_end']:.1f} |")
    INNER_DRIFT["on"] = True
    print("\nSkenario 2: fokus di dalam proyek bergeser tiap 15 hari\n")
    print("| Kebijakan | Hit rate core (semua) | Hit rate setelah ganti proyek |")
    print("|---|---|---|")
    for name, kw in configs[-3:]:
        r = avg(kw)
        print(f"| {name} | {r['hit_all']:.1%} | {r['hit_after']:.1%} |")
    INNER_DRIFT["on"] = False
    m, s = spacing()
    print(f"\nSpacing effect (dinilai di hari 60): massal 5x/5 jam B={m:.2f}, tersebar 5x/30 hari B={s:.2f}")


if __name__ == "__main__":
    main()
