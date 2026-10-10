# Riset: Karpathy "LLM Wiki" dan relevansinya dengan rewire

> Sumber: https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f (gist + thread komentar komunitas). Dibaca 2026-10-10.
> Terkait: [memory approaches](memory-approaches.md), [design synaptic memory](../design/synaptic-memory.md).

## 1. Ringkasan gist

**Ide inti:** daripada RAG yang mengambil potongan dokumen mentah setiap kali ditanya (pengetahuan "ditemukan ulang" tiap query), LLM **membangun dan merawat wiki markdown yang saling tertaut** secara bertahap. Setiap sumber baru diintegrasikan ke halaman yang sudah ada, kontradiksi ditandai, sintesis diperbarui. Wiki-nya *compounding*; manusia hampir tidak menulisnya sendiri.

**Tiga lapisan:**
1. **Raw sources** — dokumen asli, immutable. LLM membaca, tidak mengubah.
2. **Wiki** — halaman markdown buatan LLM (ringkasan, halaman entitas/konsep, perbandingan, overview). Dimiliki LLM.
3. **Schema** — dokumen konvensi (CLAUDE.md/AGENTS.md) yang mendefinisikan struktur & workflow; disempurnakan bersama manusia.

**Tiga operasi:**
- **Ingest** — baca sumber → diskusi → tulis ringkasan → update index + 10–15 halaman terkait → append log.
- **Query** — baca index → baca halaman relevan → jawab dengan sitasi; jawaban bagus disimpan balik sebagai halaman baru.
- **Lint** — health check berkala: kontradiksi, klaim basi, halaman yatim, konsep/tautan yang hilang.

**File navigasi:** `index.md` (katalog per kategori, satu baris per halaman, dibaca pertama kali) dan `log.md` (kronologis, append-only, prefix konsisten agar bisa di-grep).

**Kenapa berhasil:** manusia meninggalkan wiki karena biaya perawatan > nilainya. LLM tidak bosan merawat cross-reference, jadi biaya perawatan ≈ 0.

## 2. Pelajaran dari thread komentar (yang paling penting untuk kita)

| Temuan komunitas | Relevansi untuk rewire |
|---|---|
| **Link (gowtham0992)** menguji *usage-weighting, recency decay, dan MMR* — dan **tidak merilis satu pun**. Alasannya: constraint yang tahan lama *"do not become less true for going unread."* | ⚠️ **Langsung menantang konsep synapse kita.** Lihat §4. |
| **post-graph (crajah)**: resolusi kontradiksi **saat menulis** (bukan dicari belakangan), klaim lama ditutup bukan dihapus, dan **dua sumbu waktu terpisah**: *valid-time* (kapan fakta berlaku di dunia) vs *belief-time* (kapan brain mempercayainya). | ✅ Adopsi: bitemporal + supersede saat konsolidasi. |
| Error ekstraksi awal **meracuni store secara permanen**; output model harus diperlakukan sebagai *untrusted input*. | ✅ Menguatkan `confidence: inferred`, inbox, dan validasi skema sebelum tulis. |
| Pipeline capture **gagal diam-diam** selama dua minggu. | ✅ Tambahkan health check & laporan per tahap konsolidasi. |
| **Agent tidak membaca memory di tengah tugas** — mendiagnosis ulang masalah yang pelajarannya sudah tertulis. orcosto-lab menaruh prefix `LESSON:` di deskripsi tool supaya terlihat sebelum tiap pemanggilan. | ⚠️ Masalah *pull vs push*: memory yang hanya bisa diambil (pull) sering tidak diambil. Perlu injeksi kontekstual (push). |
| Lint berbasis LLM **kuadratik** terhadap jumlah halaman. | ✅ Lint harus inkremental (hanya node yang berubah + tetangganya). |
| Klaim pola ini **belum di-benchmark** secara terkontrol. | ✅ Menguatkan rencana eval kita (Fase 0 dan metrik di §7 desain). |
| Curated Thoughts: SQLite "brain" + MCP + traversal graph + **review sebelum merge**. | ✅ Validasi arah storage kita (SQLite + review inbox). |
| Cortexes: BM25 + vector dengan RRF, dedup sadar-provenance. | ✅ Opsi retrieval hybrid untuk fase embedding. |

## 3. Pemetaan LLM Wiki ↔ rewire

| LLM Wiki | rewire (desain saat ini) | Catatan |
|---|---|---|
| Raw sources (immutable) | `sessions/` (transcript/ringkasan) | **Gap:** rewire baru menangkap sesi agent, belum *sumber eksternal* (artikel, paper, dokumen). |
| Wiki pages | `nodes/*.md` | Sama: markdown buatan LLM, saling tertaut. Kita menambah tipe & relasi bertipe. |
| Schema (AGENTS.md) | `brain.yaml` + ontology | Sama. Kita lebih formal (tipe tetap). |
| `index.md` | `snapshot/CORE.md` + `MEMORY.md` | Bedanya: index Karpathy berisi **semua** halaman; CORE kita dipilih berdasarkan activation & budget. |
| `log.md` | `events/*.jsonl` + `sessions/dreams/` | Kita memisahkan log mesin (event) dan log manusia. |
| Ingest | Konsolidasi SessionEnd | Sama polanya: satu input → update banyak node. |
| Query → simpan balik jawaban | `remember()` | Adopsi eksplisit: jawaban/sintesis bagus jadi node `concept`/`decision`. |
| Lint | "Tidur" periodik | Sama. Kita tambah decay/tier. |
| Obsidian + graph view | — | **Bonus gratis:** karena node kita markdown, brain bisa dibuka di Obsidian (pakai wikilink di body). |

**Kesimpulan:** LLM Wiki adalah **lapisan semantic** dari rewire, hampir 1:1. Yang ditambahkan rewire di atasnya: (a) siklus hidup per sesi agent dan adapter ke banyak harness, (b) ontology bertipe, (c) model activation/synapse, (d) event sourcing untuk multi-device.

## 4. Pertanyaan kritis: apakah synapse masih layak?

Temuan Link bukan alasan membuang konsep synapse, tapi **membatasi perannya**. Ada dua jenis memory yang berperilaku berbeda:

| | **Constraint / identitas** | **Asosiasi / konteks kerja** |
|---|---|---|
| Contoh | "Jangan pernah push ke main", "user alergi kacang", "jawab dalam Bahasa Indonesia" | "Saat bahas rewire biasanya juga bahas SQLite", proyek yang sedang aktif |
| Apakah berkurang benarnya kalau jarang dipakai? | **Tidak** | Relevansinya memang turun |
| Peran decay | ❌ Tidak boleh | ✅ Masuk akal |

Jadi revisi desain:
1. **Activation hanya mengatur *ranking & penempatan di context*, tidak pernah mengatur *keberadaan* atau *kebenaran*.** Tidak ada yang dihapus karena decay.
2. **Node bertipe `preference` dan constraint (`importance ≥ 4`) dikecualikan dari decay** — selalu `core` atau `active` sesuai budget.
3. **Synapse paling berguna di *associative edges*** (memory apa yang ikut dipanggil saat X aktif) dan pemilihan snapshot proyek — di situ pola penggunaan memang sinyal relevansi.
4. **Wajib dibuktikan dengan eval** di Fase 0/2: bandingkan retrieval *dengan* vs *tanpa* activation. Kalau tidak menang, matikan (sesuai pengalaman Link). Desain harus membuat activation sebagai komponen yang bisa di-*toggle* (`β = 0`).

## 5. Revisi desain yang diusulkan (sudah diterapkan ke dokumen desain)

1. **Bitemporal**: `valid_from/valid_to` (dunia) + `recorded_at/retracted_at` (keyakinan brain).
2. **Constraint tidak decay**; activation hanya untuk ranking.
3. **Kontradiksi diselesaikan saat konsolidasi** (write-time), bukan hanya saat lint.
4. **Layer `sources/`** untuk sumber eksternal yang immutable + operasi `ingest`.
5. **Push memory kontekstual**: hook sebelum tool call / saat membuka file menyuntikkan pelajaran relevan (pola `LESSON:`), tidak hanya mengandalkan agent memanggil `recall`.
6. **Lint inkremental** + **health check per tahap** konsolidasi.
7. **Kompatibel Obsidian**: wikilink `[[...]]` di body node.
