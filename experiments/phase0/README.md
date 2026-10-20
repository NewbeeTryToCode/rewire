# Phase 0 — Activation model simulation

> **Historical.** This experiment ran on rewire's earlier Python implementation, which has been removed; its script is in git history (before the commit that removed `src/`). Current benchmarks run through [`experiments/eval`](../eval) on the Rust binary.

Raw results: [`results.md`](results.md)

## Setup

- 180 days, one session per day, **project switch on day 90** (project A → B).
- 65 nodes: 5 constraints (importance 5), 10 preferences, 25 project A nodes, 25 project B nodes.
- Each session needs ~6 project nodes (Zipf distribution), sometimes a preference, and very rarely a constraint (2%).
- Core snapshot = 15 slots. Main metric: **hit rate** = the share of a session's needs that are already in core.
- Scenario 2: the focus within a project shifts every 15 days (features/modules change).
- 20 seeds, averaged.

**Limitations:** synthetic data. This proves that the *behavior* of the math matches the design's claims, not real-world quality. The absolute numbers are not meaningful; what is meaningful is the comparison between policies.

## Findings

| # | Hypothesis | Result | Design consequence |
|---|---|---|---|
| H1 | Counting exposure as "use" causes rich-get-richer | **Confirmed, severely.** The hit rate after the project switch drops to **1.6%**; core freezes forever | Exposure = 0 is a mandatory rule (already in the design) |
| H2 | Without decay exemptions, constraints disappear | **Confirmed.** 0 of 5 constraints remain in core on day 180 | Decay exemption is mandatory (rev 1) |
| H3 | ACT-R adapts to project switches | **Failed with design rev 1.** Global ACT-R needs ~80 days and in 9/20 seeds never adapts; old project nodes are too strong | **Workaround: active project scope** → adaptation in 1 day |
| — | Constraints take up working-context slots | Constraints still occupy 5/15 slots → the hit rate drops | **Workaround: a separate constraint budget** → hit rate 60% → 73% |
| H4 | ACT-R is the best model | **No absolute winner.** Stable focus: frequency wins narrowly (74.7% vs 72.9%). Shifting focus: recency wins narrowly (63.2% vs 61.4%). ACT-R is **never far from the best**, whereas frequency (57%) and recency (65%) each clearly lose in one scenario | ACT-R is kept as the *robust* choice, not because it is optimal |
| — | Decay parameter `d` | Stable: d=0.3 is best (73.3%); shifting: d=1.0 is best (63.9%). **d≈0.7** is within ~1.5 points of the best in both scenarios | Default `d = 0.7`, can be tuned from real event logs |
| H5 | Spacing effect | **Weak.** Standard ACT-R distinguishes only slightly (B −2.02 massed vs −1.88 spaced) | Do not claim a spacing effect in v0.1. If needed: Pavlik & Anderson (2005) decay, where `d` depends on the activation at the time of use |

## Summary of the configuration chosen for the MVP

```
activation:   ACT-R base-level, d = 0.7, exposure = 0
scope:        project nodes filtered to the active project (from repo/cwd)
budget:       constraints & preferences have their own budget, outside the working-context slots
decay exempt: constraint, preference
hebbian:      off (not yet tested; deferred to v0.2)
```
