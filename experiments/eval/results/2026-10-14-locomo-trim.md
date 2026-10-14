# LoCoMo: memangkas potongan transcript dan mempercepat jalur panas

Tanggal 2026-10-14 · LoCoMo 10 percakapan, 1.540 soal kategori 1–4 · mode `episodes`, `--retrieval-only`
(tanpa LLM), kata kunci saja (`REWIRE_SEMANTIC=0`), k=30. Sumber ide: review performa
[styler-ai/ProjectAtlas](https://github.com/styler-ai/ProjectAtlas) (MIT) dan
[Bpolat0/atlasmemory](https://github.com/Bpolat0/atlasmemory) (GPL-3.0, hanya ide yang diambil, tanpa kode).

```bash
python experiments/eval/run.py --bench locomo --data locomo10.json --out OUT --sample 0 --retrieval-only \
  --episodes 10 --label trim-e10
```

## 1. Potongan transcript hanya menyimpan giliran yang relevan

Ide dari atlasmemory (snippet per file diurutkan menurut kata kueri, sisanya dibuang). Di rewire: dua
potongan teratas tetap utuh; potongan berikutnya hanya menyimpan giliran yang punya kata isi yang sama dengan
pertanyaan (tanpa awalan pembicara `Nama:`), plus satu giliran di kiri-kanannya. Potongan tanpa kata yang cocok
(datang lewat vektor, jendela waktu, atau sumber fakta) tetap utuh.

| konfigurasi | evidence turns | semua bukti | MRR | token konteks (mean / p95) | retrieval p50 / p95 ms |
|---|---|---|---|---|---|
| sebelum, 8 potongan | 70.6% | 75.2% | 0.745 | 1.816 / 2.295 | 3 / 8 |
| **sebelum, 10 potongan (baseline)** | 73.1% | 77.3% | 0.750 | 2.238 / 2.822 | 4 / 9 |
| sebelum, 12 potongan | 75.5% | 79.1% | 0.754 | 2.655 / 3.301 | 6 / 12 |
| sebelum, 15 potongan | 77.4% | 80.1% | 0.755 | 3.274 / 4.052 | 7 / 16 |
| **trim, 10 potongan** | 72.5% | 77.0% | 0.749 | **1.993** / 2.652 | 6 / 14 |
| **trim, 12 potongan** | **74.7%** | **78.7%** | 0.753 | 2.332 / 3.084 | 8 / 16 |

- Dengan jumlah potongan sama (10): token −11%, evidence −0,6 poin, semua bukti −0,3 poin.
- Dengan **anggaran token sama** (≈2.330 token; kurva "sebelum" diinterpolasi = 73,6% / 77,7%): trim + 12
  potongan **+1,1 poin evidence turns, +1,0 poin semua bukti**. Multi-hop semua bukti 32,3% → 34,0%.
- Biaya: retrieval p50 Python naik ~2 ms (cek kata per baris).

Yang **dicoba dan tidak dipakai**: bonus koordinasi (berapa kata pertanyaan yang ada di potongan, ide dari
query frasa + OR atlasmemory): bobot 0,1 / 0,3 / 0,6 memberi +0,1–0,4 poin evidence dan +0,01 MRR, hilang saat
digabung dengan trim, dan menggandakan latensi. Trim dengan 2 giliran konteks: sama dengan 1 giliran, token lebih
banyak.

## 2. Latensi jalur panas

`python experiments/perf/bench.py 2000 10000` (Python, median ms, container cloud, 3 ulangan; angka CLI
termasuk startup Python):

| operasi | 2.000 node sebelum → sesudah | 10.000 node sebelum → sesudah |
|---|---|---|
| hook session-start (CLI) | 200 → **155** | 491 → **331** |
| prime (CLI) | 101 → **90** | 271 → **235** |
| recall (CLI) | 108 → **60** | 68 → 67 |
| recall (MCP, server hangat) | 6 → **3** | 14 → **10** |

Perubahan:
- **Snapshot tidak memformat baris yang pasti tidak muat** (ide ProjectAtlas: setiap baca punya batas). Profil
  hook di 2.000 node: 60% waktu habis memformat dan memindai ±1.600 baris yang tidak pernah tampil. Sekarang baris
  yang judulnya (batas bawah panjang baris) sudah melebihi sisa anggaran dihitung tanpa diformat. Output identik
  byte per byte (dicek dengan `prime` sebelum/sesudah).
- **Cache git per cwd selama 60 detik**: dua proses `git` (toplevel + remote) per recall memakan ~3 ms, sebagian
  besar waktu recall MCP yang hangat.

## 3. Paritas

`experiments/conformance/recall_parity.py` pada conv-26 dan conv-30 (233 soal, k=50, 10 potongan): hasil Python
dan Rust identik 233/233 (item dan urutan sama).

## Kandidat lain dari review (belum dikerjakan)

- Spreading activation hanya lewat tetangga yang relevan dengan kueri, dengan batas bonus graf (ProjectAtlas
  `projectatlas-service/src/lib.rs` ~2451). Perlu mode `full` (fakta) untuk diukur; mode `episodes` tidak punya
  graf.
- Output recall MCP berbentuk tabel (gaya TOON) dan tanpa `score`/`b`: perkiraan −8–15% token.
- Hash isi file di samping mtime, supaya `git checkout`/sync tidak memicu parse dan embedding ulang.
- Klasifikasi tiap soal yang meleset: tidak jadi kandidat, terpotong di k, atau hilang saat merge/trim.
