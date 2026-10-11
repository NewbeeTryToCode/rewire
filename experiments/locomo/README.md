# LoCoMo: rewire vs Mem0, Zep dan lainnya

Harness: [`run.py`](run.py) · Prompt resmi: [`official_prompts.py`](official_prompts.py) (disalin verbatim dari [mem0ai/memory-benchmarks](https://github.com/mem0ai/memory-benchmarks)) · Hasil per pertanyaan: [`results.json`](results.json)

## Setup

- **Data:** [LoCoMo](https://github.com/snap-research/locomo) (ACL 2024), 3 dari 10 percakapan (`conv-26`, `conv-30`, `conv-49`): 63 sesi, **389 pertanyaan** kategori 1–4. Kategori 5 (adversarial) dikeluarkan, sama seperti harness Mem0.
- **Ingest:** setiap sesi dikonsolidasi ke brain rewire secara berurutan, persis seperti transcript harness (`claude -p`). Inbox di-approve semua (benchmark tidak punya reviewer manusia).
- **Jawaban:** `recall` mengambil top-K memory → LLM menjawab dengan **prompt jawaban resmi Mem0** (memory diurutkan kronologis dengan tanggal). Perbedaan: 8 pertanyaan per panggilan (hemat biaya), Mem0 1 per panggilan.
- **Penilaian:** **judge resmi Mem0** (CORRECT/WRONG) = skor J, plus token F1 sebagai pembanding yang ketat.

## Hasil

| Konfigurasi | single-hop | multi-hop | temporal | open-domain | **J keseluruhan** | F1 | konteks/pertanyaan |
|---|---|---|---|---|---|---|---|
| rewire v0.4 apa adanya (profil `coding`) | 73.8% | 90.0% | 38.5% | 92.3% | **69.7% ± 4.6** | 0.303 | ~1.8k token |
| + profil `general` + tanggal kejadian | 88.2% | 95.0% | 93.8% | 92.3% | **91.3% ± 2.8** | 0.392 | ~1.9k token |
| + episode (potongan transcript mentah) | 95.2% | 100% | 96.9% | 96.2% | **96.7% ± 1.8** | 0.456 | ~4.4k token |

± = interval kepercayaan 95% (binomial). n: single 187, multi 80, temporal 96, open 26.

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

Pada setup ini **tidak** — rewire v3 (96.7%) setara atau di atas angka Mem0 platform (92.5%), dengan konteks lebih kecil (~4.4k vs ~7k token). Tapi **jangan dibaca sebagai "rewire mengalahkan Mem0"**:

1. **Subset**: 3 dari 10 percakapan (389 dari 1.540 pertanyaan). Interval ±1.8 poin.
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

- **Embedding / semantic search** (Mem0, Zep). Kita menutupinya dengan kata kunci dari konsolidasi + query dari agent. Kandidat berikutnya sebagai plugin opsional — tetap tanpa dependensi wajib.
- **Reranker** (Zep: cross-encoder / jarak graph).
- **ADD-only vs supersede**: Mem0 menyimpan fakta lama dan baru, lalu menyerahkan ke penjawab. rewire menandai fakta lama `superseded`; untuk pertanyaan sejarah ("dulu pakai apa?") fakta lama sebaiknya tetap bisa dikembalikan dengan label.
- **Hitungan dan selisih waktu** (sisa kegagalan multi-hop/temporal) butuh penalaran di sisi penjawab, bukan memory.

## Biaya

Ingest ~63 sesi × (ekstraksi + judge) ≈ 125 panggilan `claude -p` (~10–25 detik/sesi, berjalan di background setelah sesi selesai di pemakaian nyata). Jawaban ~49 panggilan, judge ~20 panggilan per konfigurasi.
