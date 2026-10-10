Simulasi 180 hari, ganti proyek di hari 90, core=15 slot, 20 seed

| Kebijakan | Hit rate core (semua) | Hit rate setelah ganti proyek | Hari sampai 80% slot proyek = proyek baru | Constraint di core hari 180 (dari 5) |
|---|---|---|---|---|
| ACT-R, eksposur=0, pengecualian decay (DESAIN) | 51.9% | 44.4% | 79.8 (9/20 seed tidak pernah) | 5.0 |
| ACT-R, eksposur dihitung sebagai pakai | 25.1% | 1.6% | tidak pernah | 5.0 |
| ACT-R, TANPA pengecualian decay | 67.7% | 60.9% | 81.8 (14/20 seed tidak pernah) | 0.0 |
| Frekuensi murni (count) | 44.5% | 29.2% | tidak pernah | 5.0 |
| Recency murni (terakhir dipakai) | 49.6% | 50.3% | 1.9 | 5.0 |
| WORKAROUND 1: ACT-R d=0.8 (decay lebih cepat untuk node proyek) | 54.9% | 52.1% | 33.9 | 5.0 |
| WORKAROUND 2: ACT-R + scope proyek aktif | 60.1% | 60.8% | 1.0 | 5.0 |
| WORKAROUND 2+3: + budget constraint terpisah (REVISI DESAIN) | 72.9% | 73.8% | 1.0 | 5.0 |
| Recency + scope + budget terpisah (pembanding) | 65.3% | 66.4% | 1.0 | 5.0 |
| Frekuensi + scope + budget terpisah (pembanding) | 74.7% | 75.1% | 1.0 | 5.0 |

Skenario 2: fokus di dalam proyek bergeser tiap 15 hari

| Kebijakan | Hit rate core (semua) | Hit rate setelah ganti proyek |
|---|---|---|
| WORKAROUND 2+3: + budget constraint terpisah (REVISI DESAIN) | 61.4% | 62.3% |
| Recency + scope + budget terpisah (pembanding) | 63.2% | 64.2% |
| Frekuensi + scope + budget terpisah (pembanding) | 57.1% | 57.0% |

Spacing effect (dinilai di hari 60): massal 5x/5 jam B=-2.02, tersebar 5x/30 hari B=-1.88
