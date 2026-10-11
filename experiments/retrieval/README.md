# Kualitas retrieval

Mengukur apakah `recall` mengembalikan memory yang benar, bukan seberapa cepat (itu di [`../perf`](../perf/README.md)).

## Data

- [`dataset.json`](dataset.json): dibuat oleh LLM terpisah ([`make_dataset.py`](make_dataset.py)) — persona developer Indonesia yang menulis campuran Indonesia/Inggris, 3 proyek (Go, Flutter, Python), **70 memory**, 22 relasi, 6 fakta yang sudah digantikan, dan **60 pertanyaan** dengan jawaban yang diketahui, 10 per kategori:

| Kategori | Yang diuji |
|---|---|
| lexical | pertanyaan memakai kata yang sama dengan memory |
| paraphrase | makna sama, **tidak ada kata penting yang sama** |
| cross_lingual | pertanyaan Indonesia, memory Inggris (atau sebaliknya) |
| relational | jawabannya memory yang **terhubung** ke memory yang disebut pertanyaan |
| stale | fakta sudah berubah: harus dapat yang baru, bukan yang lama |
| scope | memory mirip ada di proyek lain: harus dapat yang di proyek aktif |

- [`keywords.json`](keywords.json): kata kunci per memory, dibuat dengan instruksi **yang sama persis** dengan konsolidasi (`KEYWORDS_RULE`); generatornya tidak melihat pertanyaan.
- [`agent_queries.json`](agent_queries.json): query yang ditulis LLM-agent untuk tiap pertanyaan, mengikuti panduan di deskripsi tool MCP (`QUERY_GUIDE`); generatornya tidak melihat memory.

Jalankan: `python experiments/retrieval/eval.py`

## Hasil (R@5 / MRR)

| Konfigurasi | lexical | paraphrase | cross_lingual | relational | stale | scope | **semua** |
|---|---|---|---|---|---|---|---|
| Awal iterasi (teks + graph, v0.3) | 100% / 1.00 | 70%* / 0.52 | 80% / 0.57 | 60% / 0.28 | 80% / 0.70 | 70% / 0.70 | **77% / 0.63** |
| + stopword, stemming EN/ID, fakta basi → penggantinya | 100% / 1.00 | 30% / 0.25 | 70% / 0.71 | 90% / 0.39 | 100% / 1.00 | 80% / 0.68 | **78% / 0.67** |
| + kata kunci dari konsolidasi | 100% / 1.00 | 50% / 0.45 | 100% / 0.86 | 90% / 0.47 | 100% / 1.00 | 100% / 1.00 | **90% / 0.80** |
| + query ditulis agent sesuai panduan tool | 100% / 1.00 | 100% / 1.00 | 100% / 1.00 | 100% / 0.63 | 100% / 1.00 | 100% / 0.85 | **100% / 0.91** |

\* Angka parafrase awal **semu**: jawaban ikut terambil karena kebetulan cocok di kata umum ("should", "use", "the"). Setelah stopword dibuang terlihat angka aslinya (30%).

R@5 = jawaban benar ada di 5 teratas. MRR = rata-rata 1/peringkat jawaban benar (1.0 = selalu di posisi pertama). Tidak ada kasus fakta basi atau memory proyek lain yang muncul di 5 teratas pada konfigurasi mana pun.

## Yang diubah karena eval ini

1. **Stopword + stemming ringan** Inggris & Indonesia di sisi query (`deployed` → `deploy*`, `menyimpan` → `simpan*`).
2. **Memory basi mengarahkan ke penggantinya**: dicari juga, tidak pernah tampil, tapi energinya diteruskan lewat `supersedes` (stale 80% → 100%, MRR 0.70 → 1.00).
3. **Kata kunci** (sinonim + terjemahan ID/EN) ditulis LLM saat konsolidasi, disimpan di frontmatter (`keywords:`, terlihat juga di Obsidian) dan diindeks.
4. **Panduan query** di deskripsi tool MCP dan skill: agent diminta menulis kata kunci + sinonim dalam dua bahasa.

## Batasan — baca sebelum percaya angka 100%

- **Satu keluarga model** membuat dataset, kata kunci, dan query agent. Kosakatanya cenderung sama, jadi hasil ini **optimis**. Data nyata (memory dan pertanyaan milik user) hampir pasti lebih rendah.
- **Kecil**: 10 pertanyaan per kategori, jadi satu pertanyaan = 10 poin persen.
- Activation (bobot pemakaian) **tidak diuji** di sini: dataset tidak punya riwayat pemakaian.
- Langkah berikut yang lebih jujur: E2 di [mvp-spec](../../docs/design/mvp-spec.md) — 50 pertanyaan dari riwayat user sendiri, dan/atau subset benchmark publik (LongMemEval/LoCoMo, dengan catatan audit soal judge LoCoMo).
