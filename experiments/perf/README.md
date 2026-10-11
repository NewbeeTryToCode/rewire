# Performa jalur kritis

Script: [`bench.py`](bench.py) · Jalankan: `python experiments/perf/bench.py [ukuran...]`

Brain sintetis dengan N node dan ~10 event pemakaian per node (10.000 node = 100.000 event). Median dalam milidetik, diukur di container cloud (Python 3.13). Angka CLI termasuk ~70 ms startup Python.

| Operasi | 500 node (sebelum → sesudah) | 2.000 node | 10.000 node |
|---|---|---|---|
| **hook session-start (CLI)** — yang ditunggu harness | 197 → **77** | 359 → **83** | 1.240 → **168** |
| prime (CLI) | 209 → **72** | 266 → **84** | 1.099 → **140** |
| recall (CLI) | 190 → **87** | 268 → **81** | 999 → **95** |
| recall (MCP, server hangat) | 94 → **6** | 188 → **8** | 908 → **34** |
| get dengan ID pendek (MCP) | 61 → **3** | 176 → **5** | 910 → **25** |
| rebuild penuh (sekali, saat init/clone) | 48 → 76 | 177 → 281 | 907 → 1.300–1.600 |
| konsolidasi tanpa waktu LLM | — | — | 385 (500 node: 140) |

## Yang diubah

1. **Index inkremental** (penyebab utama). Sebelumnya setiap `prime`/`recall`/hook membangun ulang index dari nol. Sekarang hanya file node yang mtime/ukurannya berubah yang dibaca ulang, hanya byte baru di event log yang diproses (offset + verifikasi 64 byte terakhir; file yang ditulis ulang, misalnya oleh `git revert`, memicu rebuild penuh), dan activation dihitung ulang hanya untuk node yang tersentuh (penuh setiap 5 menit, karena activation berubah dalam satuan jam).
2. **Lewati scan direktori** bila mtime folder tidak berubah dan scan penuh terakhir < 60 detik (rewire selalu menulis dengan rename atomik, yang mengubah mtime folder).
3. **ID pendek → glob nama file** (`*-<8 karakter>.md`), bukan parse semua node.
4. **Frontmatter diparse sekali per file** (bukan satu `json.loads` per baris); event diparse dari string, bukan bytes, dan disisipkan dengan `executemany`.
5. **Konsolidasi tanpa scan penuh**: hub proyek dicari di `nodes/project/` saja; ID memory yang dipakai di-resolve satu per satu.
6. `importlib.resources` hanya diimport saat `install` (−16 ms startup).

## Trade-off

- Rebuild penuh ~1,6× lebih lambat dari sebelumnya (WAL + upsert per event). Terjadi sekali per mesin; dipilih karena semua operasi harian jadi 3–30× lebih cepat.
- Edit manual *in-place* pada file node (tanpa rename) bisa terlambat terlihat hingga 60 detik. Editor biasa dan rewire sendiri menulis dengan rename, jadi langsung terlihat.
- Biaya dominan konsolidasi tetap panggilan LLM (~10–17 detik per sesi, berjalan di background setelah sesi selesai).
