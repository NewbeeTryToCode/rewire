# Fase 0 — Simulasi model activation

Script: [`sim.py`](sim.py) · Hasil mentah: [`results.md`](results.md) · Jalankan: `python3 experiments/phase0/sim.py` (stdlib saja)

## Setup

- 180 hari, satu sesi per hari, **ganti proyek di hari 90** (proyek A → B).
- 65 node: 5 constraint (importance 5), 10 preference, 25 node proyek A, 25 node proyek B.
- Setiap sesi membutuhkan ~6 node proyek (distribusi Zipf), kadang preference, constraint sangat jarang (2%).
- Core snapshot = 15 slot. Metrik utama: **hit rate** = porsi kebutuhan sesi yang sudah ada di core.
- Skenario 2: fokus di dalam proyek bergeser tiap 15 hari (fitur/modul berganti).
- 20 seed, dirata-rata.

**Batasan:** data sintetis. Ini membuktikan *perilaku* matematika sesuai klaim desain, bukan kualitas di dunia nyata. Angka absolut tidak bermakna; yang bermakna adalah perbandingan antar kebijakan.

## Temuan

| # | Hipotesis | Hasil | Konsekuensi desain |
|---|---|---|---|
| H1 | Menghitung eksposur sebagai "pakai" menyebabkan rich-get-richer | **Terbukti, parah.** Hit rate setelah ganti proyek jatuh ke **1.6%**; core membeku selamanya | Eksposur = 0 adalah aturan wajib (sudah di desain) |
| H2 | Tanpa pengecualian decay, constraint hilang | **Terbukti.** 0 dari 5 constraint tersisa di core hari 180 | Pengecualian decay wajib (rev 1) |
| H3 | ACT-R beradaptasi ke pergantian proyek | **Gagal pada desain rev 1.** ACT-R global butuh ~80 hari dan di 9/20 seed tidak pernah beradaptasi; node proyek lama terlalu kuat | **Workaround: scope proyek aktif** → adaptasi 1 hari |
| — | Constraint memakan slot konteks kerja | Constraint tetap menempati 5/15 slot → hit rate turun | **Workaround: budget constraint terpisah** → hit rate 60% → 73% |
| H4 | ACT-R model terbaik | **Tidak ada pemenang mutlak.** Fokus stabil: frekuensi menang tipis (74.7% vs 72.9%). Fokus bergeser: recency menang tipis (63.2% vs 61.4%). ACT-R **tidak pernah jauh dari yang terbaik**, sedangkan frekuensi (57%) dan recency (65%) masing-masing kalah jelas di satu skenario | ACT-R dipertahankan sebagai pilihan *robust*, bukan karena optimal |
| — | Parameter decay `d` | Stabil: d=0.3 terbaik (73.3%); bergeser: d=1.0 terbaik (63.9%). **d≈0.7** berada dalam ~1.5 poin dari yang terbaik di kedua skenario | Default `d = 0.7`, bisa di-tuning dari event log nyata |
| H5 | Spacing effect | **Lemah.** ACT-R standar hanya membedakan sedikit (B −2.02 massal vs −1.88 tersebar) | Jangan klaim spacing effect di v0.1. Jika dibutuhkan: decay Pavlik & Anderson (2005), di mana `d` bergantung pada aktivasi saat pemakaian |

## Ringkasan konfigurasi yang dipilih untuk MVP

```
activation:   ACT-R base-level, d = 0.7, eksposur = 0
scope:        node proyek difilter ke proyek aktif (dari repo/cwd)
budget:       constraint & preference punya budget sendiri, di luar slot konteks kerja
decay exempt: constraint, preference
hebbian:      off (belum diuji; ditunda ke v0.2)
```
