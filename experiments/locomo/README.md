# LoCoMo: rewire vs Mem0, Zep dan lainnya

Harness: [`run.py`](run.py) · Prompt resmi: [`official_prompts.py`](official_prompts.py) (disalin verbatim dari [mem0ai/memory-benchmarks](https://github.com/mem0ai/memory-benchmarks)) · Hasil per pertanyaan: [`results.json`](results.json)

## Setup

- **Data:** [LoCoMo](https://github.com/snap-research/locomo) (ACL 2024). Iterasi dilakukan pada 3 percakapan (`conv-26`, `conv-30`, `conv-49`, 389 pertanyaan); konfigurasi akhir dijalankan pada **seluruh 10 percakapan: 272 sesi, 1.540 pertanyaan** kategori 1–4. Kategori 5 (adversarial) dikeluarkan, sama seperti harness Mem0.
- **Ingest:** setiap sesi dikonsolidasi ke brain rewire secara berurutan, persis seperti transcript harness (`claude -p`). Inbox di-approve semua (benchmark tidak punya reviewer manusia).
- **Jawaban:** `recall` mengambil top-K memory → LLM menjawab dengan **prompt jawaban resmi Mem0** (memory diurutkan kronologis dengan tanggal). Perbedaan: 8 pertanyaan per panggilan (hemat biaya), Mem0 1 per panggilan.
- **Penilaian:** **judge resmi Mem0** (CORRECT/WRONG) = skor J, plus token F1 sebagai pembanding yang ketat.

## Hasil penuh: 10 percakapan, 1.540 pertanyaan

Konfigurasi akhir (profil `general`, tanggal kejadian, 10 episode, top-50, ~4.7k token konteks/pertanyaan):

| Konfigurasi | single-hop (841) | multi-hop (282) | temporal (321) | open-domain (96) | **J keseluruhan** | F1 |
|---|---|---|---|---|---|---|
| rewire, hybrid (BM25 + semantik WordLlama) | 95.8% | 95.7% | 94.1% | 75.0% | **94.2% ± 1.2** | 0.454 |
| rewire, kata kunci saja (`REWIRE_SEMANTIC=0`) | 96.9% | 96.5% | 94.1% | 76.0% | **94.9% ± 1.1** | 0.451 |
| Mem0 platform (dipublikasikan, judge sama) | 91.2 | 91.3 | 92.0 | 72.7 | **92.5** | – |

Per percakapan (hybrid): 88.8% (conv-43) sampai 97.4% (conv-26, conv-41).

**Ablasi semantik:** semantik **tidak membantu** di LoCoMo. Dari 1.540 pertanyaan, 17 hanya benar dengan hybrid, 29 hanya benar dengan kata kunci (McNemar p ≈ 0.10, tidak signifikan). Konteks yang diambil 87% identik. Alasannya: episode mentah + kata kunci dwibahasa dari konsolidasi sudah menutup sebagian besar kasus parafrase, dan embedding statis (WordLlama) lebih lemah dari BM25 untuk nama dan detail spesifik. Di eval sintetis ([experiments/retrieval](../retrieval/README.md)) semantik membantu query parafrase, jadi ia tetap **opsional** (`pip install 'rewire[semantic]'`), bukan wajib.

**Open-domain (75%) adalah sisa kelemahan terbesar — dan sebagian besar bukan masalah memory.** Dari 24 jawaban salah, mayoritas pertanyaan spekulatif yang jawaban emasnya butuh pengetahuan dunia atau tebakan spesifik ("brand outdoor mana yang mungkin meng-endorse John?" → *Under Armour*; "taman nasional mana?" → *Voyageurs*; "kartu permainan apa?" → *Exploding Kittens*). Memory-nya memberi petunjuk yang benar; jawabannya ditentukan penalaran penjawab. Mem0 juga paling rendah di kategori ini (72.7%).

## Iterasi 7: dari analisis kegagalan ke perbaikan retrieval

LoCoMo menandai giliran bukti () untuk setiap pertanyaan. [](evidence.py) mengukur apakah giliran itu masuk ke konteks yang diambil rewire: **tanpa LLM, ~20 detik** untuk 1.540 pertanyaan. Ini sekarang loop iterasi utama; skor J berubah ±3 poin antar-run (46 pertanyaan berganti benar/salah dengan konteks 87% sama), terlalu berisik untuk perubahan kecil.

**Diagnosis 90 jawaban salah (run hybrid):** 57% sudah punya *semua* bukti di konteks (salah di penjawab), 21% punya sebagian (hampir semua multi-hop), 22% bukti tidak terambil.

| Perubahan | Semua bukti di konteks (hybrid / kata kunci) | J LoCoMo penuh | Keputusan |
|---|---|---|---|
| Sebelum | 75.5% / 78.0% | 94.2% / 94.9% | – |
| **Fakta menarik potongan transcript asalnya** (sumber klaim → sesi + giliran) | **79.5% / 79.2%** (temporal 81 → 90%) | **94.0%** (23 diperbaiki, 25 rusak vs hybrid) | Dipertahankan: retrieval lebih baik, J tidak berubah |
| Pertanyaan hitungan ("how many") dapat 2× potongan | multi-hop +0.8 poin | 28/35 vs 29–31/35 | **Dibuang**: kejadian yang sama di beberapa sesi dihitung dua kali |
| Maksimal N potongan per sesi (diversitas) | turun (single-hop 94 → 84% di N=1) | – | **Dibuang** |
| Aturan timeline di ekstraksi (tanggal mulai untuk durasi) | butuh ingest ulang | belum diukur | Dipertahankan (berlaku untuk konsolidasi baru) |

Bug yang ditemukan: sumber klaim menyimpan id sesi terpotong 8 karakter (), jadi tautan fakta → transcript putus. Sekarang id penuh disimpan; brain eval diperbaiki dari riwayat git ().

**Kesimpulan:** di LoCoMo retrieval sudah bukan bottleneck. Bukti yang lebih lengkap tidak menaikkan J karena sisa kesalahan ada di penjawab (salah pilih orang/peristiwa, salah hitung) dan di pertanyaan spekulatif. LoCoMo jenuh di ~94% untuk setup ini; perbaikan berikutnya perlu benchmark yang menguji hal lain (LongMemEval: *knowledge update*, *abstention*).

## Iterasi pada subset 3 percakapan

| Konfigurasi | single-hop | multi-hop | temporal | open-domain | **J keseluruhan** | F1 | konteks/pertanyaan |
|---|---|---|---|---|---|---|---|
| rewire v0.4 apa adanya (profil `coding`) | 73.8% | 90.0% | 38.5% | 92.3% | **69.7% ± 4.6** | 0.303 | ~1.8k token |
| + profil `general` + tanggal kejadian | 88.2% | 95.0% | 93.8% | 92.3% | **91.3% ± 2.8** | 0.392 | ~1.9k token |
| + episode (potongan transcript mentah) | 95.2% | 100% | 96.9% | 96.2% | **96.7% ± 1.8** | 0.456 | ~4.4k token |
| + semantik, jendela waktu, intent sejarah (v4) | – | – | – | – | **95.4%** | 0.443 | ~4.7k token |

v3 → v4: 0 diperbaiki, 5 rusak, konteks 83–93% sama: selisih adalah variasi jawaban/judge, bukan regresi retrieval. ± = interval kepercayaan 95% (binomial). n: single 187, multi 80, temporal 96, open 26.

### Angka yang dipublikasikan pihak lain (tidak dijalankan ulang di sini)

| Sistem | J LoCoMo | Sumber & catatan |
|---|---|---|
| **Mem0 platform** (2026) | 92.5 (single 91.2 · multi 91.3 · temporal 92.0 · open 72.7) | [Dokumentasi Mem0](https://github.com/mem0ai/mem0/blob/main/docs/core-concepts/memory-evaluation.mdx) — judge & prompt yang **sama** dengan run ini, top-200 memory, ~7k token, model jawaban tidak disebut, platform berbayar |
| Full-context (semua percakapan di prompt) | 72.9 | Paper Mem0 2025 ([arXiv 2504.19413](https://arxiv.org/abs/2504.19413)), judge lama yang lebih ketat |
| Mem0g / Mem0 (2025) | 68.4 / 66.9 | Paper Mem0 2025, judge lama |
| Zep | ~66 menurut paper Mem0; **75.1** menurut koreksi Zep sendiri; klaim terbaru 94.7 | Sengketa metodologi antara kedua vendor |
| LangMem / OpenAI Memory | ~58 / ~53 | Paper Mem0 2025 |

**Perbandingan yang adil hanya dengan baris pertama** (judge dan prompt jawaban identik). Angka 2025 memakai judge yang jauh lebih ketat dan tidak bisa dibandingkan langsung.

## Jadi, apakah mereka lebih baik?

Pada setup ini **tidak** — di seluruh 1.540 pertanyaan rewire mendapat 94.2–94.9% vs 92.5% Mem0 platform, dengan konteks lebih kecil (~4.7k vs ~7k token). (Subset 3 percakapan sempat menunjukkan 96.7%; angka penuh lebih rendah, seperti yang diperkirakan dari interval ±1.8.) Tapi **jangan dibaca sebagai "rewire mengalahkan Mem0"**:

1. **Selisih kecil**: +1.7 sampai +2.4 poin, sedikit di atas interval ±1.1–1.2 kita, dan tanpa interval di sisi Mem0.
2. **Model jawaban & judge berbeda**: kita memakai Claude untuk konsolidasi, jawaban, **dan** judge. Model jawaban Mem0 tidak dipublikasikan. Judge sekeluarga dengan penjawab bisa lebih murah hati.
3. **Judge sangat longgar** (satu item benar dari daftar = benar, tanggal ±14 hari, durasi ±50%); spot-check kami menemukan "About seven years" dinilai benar untuk "Since 2016". F1 yang ketat (0.456) menunjukkan jawaban masih sering tidak persis.
4. **LoCoMo kecil**: seluruh percakapan muat di context window. Zep dan Mem0 sendiri menyatakan benchmark ini mudah dan bisa "dibeli" dengan konteks besar; BEAM (1M–10M token) lebih relevan untuk skala produksi.
5. Kita belum menjalankan Mem0/Zep di harness yang sama.

Yang valid: **perbaikan relatif** di dalam rewire (69.7 → 96.7) diukur dengan setup yang identik.

## Kenapa baseline rewire tertinggal — dan insight yang diadopsi

| Penyebab (dari analisis jawaban yang salah) | Cara pesaing menanganinya | Yang diimplementasikan di rewire |
|---|---|---|
| **Tidak ada tanggal kejadian.** Memory dicatat dengan tanggal konsolidasi; "kemarin" tidak dikonversi. Temporal 38.5% | Mem0: pass *temporal reasoning* terpisah yang mencatat kapan kejadian terjadi. Zep: fakta dengan jendela validitas (bi-temporal) | Klaim diberi `valid_from` = tanggal sesi; ekstraktor wajib mengubah tanggal relatif jadi absolut; jawaban diberi memory berurutan dengan tanggal. **Temporal 38.5 → 93.8%** |
| **Ekstraksi terlalu jarang.** Profil coding sengaja membuang "obrolan" (3–5 klaim/sesi) | Mem0: ekstraksi semua fakta, *ADD-only* (tidak pernah menimpa) | Profil `general` (9–15 klaim/sesi, nama orang selalu disebut); profil `coding` tetap default untuk agent coding |
| **Detail halus hilang saat diringkas** (kata persis, isi foto, tulisan poster) | Zep: **episode** mentah disimpan sebagai sumber setiap fakta; baseline full-context (72.9) mengalahkan Mem0 (66.9) di paper Mem0 | Transcript sesi disimpan (secret disamarkan), dipotong per 6 giliran, diindeks, dan dikembalikan `recall` setelah fakta. **+23 jawaban benar, −2** |
| Pencarian kata saja | Hybrid: semantik + BM25 + entitas/graph + reranker | Sudah ada: BM25 + stemming EN/ID + kata kunci dwibahasa + spreading activation. **Belum**: embedding & reranker (lihat di bawah) |

### Insight yang belum diimplementasikan

- ~~**Embedding / semantic search**~~ → **diimplementasikan** (WordLlama offline, fusi aditif). Tidak membantu di LoCoMo (lihat ablasi), membantu parafrase di eval sintetis; tetap opsional.
- ~~**ADD-only vs supersede**~~ → **diimplementasikan**: pertanyaan sejarah ("dulu", "used to", "first") mengembalikan fakta `superseded` dengan label.
- ~~**Jendela waktu**~~ → **diimplementasikan**: "in July 2023" memberi bonus pada memory dari bulan itu.
- **Reranker** (Zep: cross-encoder / jarak graph). Belum; dengan embedding statis tidak membantu, reranker kemungkinan butuh model yang lebih berat.
- **Hitungan dan selisih waktu** (sisa kegagalan multi-hop/temporal) butuh penalaran di sisi penjawab, bukan memory.

## Biaya

Subset: ingest ~63 sesi × (ekstraksi + judge) ≈ 125 panggilan `claude -p` (~10–25 detik/sesi, berjalan di background setelah sesi selesai di pemakaian nyata). Jawaban ~49 panggilan, judge ~20 panggilan per konfigurasi.

Run penuh: 272 sesi ≈ 550 panggilan ingest, lalu ~195 panggilan jawaban + ~80 panggilan judge per konfigurasi. Run terputus beberapa kali oleh batas pemakaian; harness melanjutkan dari cache (`locomo_done.json`, `answers-*.json`, `judged-*.json`). Run penuh juga menemukan satu bug nyata: `time_window` crash pada rentang tahun seperti "2022-23" (diperbaiki, ada test).
