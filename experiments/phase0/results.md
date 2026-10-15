Simulation of 180 days, project switch on day 90, core=15 slots, 20 seeds

| Policy | Core hit rate (all) | Hit rate after project switch | Days until 80% of project slots = new project | Constraints in core on day 180 (of 5) |
|---|---|---|---|---|
| ACT-R, exposure=0, decay exemption (DESIGN) | 51.9% | 44.4% | 79.8 (9/20 seeds never) | 5.0 |
| ACT-R, exposure counted as use | 25.1% | 1.6% | never | 5.0 |
| ACT-R, WITHOUT decay exemption | 67.7% | 60.9% | 81.8 (14/20 seeds never) | 0.0 |
| Pure frequency (count) | 44.5% | 29.2% | never | 5.0 |
| Pure recency (last used) | 49.6% | 50.3% | 1.9 | 5.0 |
| WORKAROUND 1: ACT-R d=0.8 (faster decay for project nodes) | 54.9% | 52.1% | 33.9 | 5.0 |
| WORKAROUND 2: ACT-R + active project scope | 60.1% | 60.8% | 1.0 | 5.0 |
| WORKAROUND 2+3: + separate constraint budget (DESIGN REVISION) | 72.9% | 73.8% | 1.0 | 5.0 |
| Recency + scope + separate budget (baseline) | 65.3% | 66.4% | 1.0 | 5.0 |
| Frequency + scope + separate budget (baseline) | 74.7% | 75.1% | 1.0 | 5.0 |

Scenario 2: the focus within a project shifts every 15 days

| Policy | Core hit rate (all) | Hit rate after project switch |
|---|---|---|
| WORKAROUND 2+3: + separate constraint budget (DESIGN REVISION) | 61.4% | 62.3% |
| Recency + scope + separate budget (baseline) | 63.2% | 64.2% |
| Frequency + scope + separate budget (baseline) | 57.1% | 57.0% |

Spacing effect (evaluated on day 60): massed 5x/5 hours B=-2.02, spaced 5x/30 days B=-1.88
