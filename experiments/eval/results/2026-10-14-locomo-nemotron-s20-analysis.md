# Analisis: LoCoMo 80 soal, Nemotron 3 Ultra (free)

Hasil run: [2026-10-14-locomo-nemotron-s20.md](2026-10-14-locomo-nemotron-s20.md). J 88,8%, F1 0,232, BLEU-1 0,163,
semua bukti ada di konteks untuk 67,5% soal. Analisis ini tanpa LLM: retrieval 80 soal yang sama direproduksi
persis (seed sama, 67,5%), lalu diubah satu variabel per kali.

## 1. Mana yang sebenarnya rendah

| Angka | Nilai | Bacaan |
|---|---|---|
| J | 88,8% (±6,9 poin, n=80) | Tidak rendah; dekat 94% iterasi 6–7 (Claude, mode `full`, k=50). Penjawab = judge, jadi cenderung longgar (contoh "Banff" untuk gold "Rockies, Jasper" dinilai benar). |
| F1 / BLEU-1 | 0,23 / 0,16 | Rendah. Jawaban rata-rata 69 kata (temporal 150 kata) sedangkan gold 1–5 kata. Penyebab terbesar: parser `ANSWER:` peka huruf besar dan markdown, jadi `**Answer:**` atau `Final answer:` membuat seluruh penalaran ikut dihitung sebagai jawaban. Diperbaiki (lihat §3). |
| Semua bukti ada | 67,5% | Rendah untuk multi-hop (35%) dan open-domain (50%). Inilah batas atas yang sesungguhnya. |

## 2. Kenapa retrieval multi-hop / open-domain gagal

26 dari 80 soal buktinya tidak lengkap: multi-hop 13, open-domain 10, temporal 2, single-hop 1. Polanya:

- **Soal agregasi / daftar.** "What causes has John done events for?", "What people has Maria met...?",
  "What writing classes has Maria taken?" perlu 3–5 giliran dari sesi berbeda. Dengan 10 potongan, potongan
  teratas sering berasal dari satu sesi yang sama.
- **Kesenjangan kata (lexical gap) dan pengetahuan dunia.** "France" vs bukti "Paris"; "roadtrips" vs "a trip
  with my family"; "stress reliever" vs "yoga ... sunsets". BM25 tidak bisa menjembatani ini.
- **Bukti tanpa nama entitas.** Giliran bukti sering tidak menyebut nama orangnya ("I took a dog grooming
  course"); yang menyebut nama ada di giliran lain.
- **Noise.** Context precision 15,6%: satu potongan berisi 6 giliran, jadi sebagian besar konteks tidak relevan.

## 3. Ablasi (80 soal yang sama, tanpa LLM; persen soal dengan semua bukti di konteks)

| Konfigurasi | single | temporal | multi-hop | open | **overall** | konteks |
|---|---|---|---|---|---|---|
| Run ini: episodes, keyword, k=30, 10 potongan | 95 | 90 | 35 | 50 | **67,5** | 2,5k |
| + 20 potongan | 95 | 95 | 65 | 60 | **78,8** | 4,9k |
| + semantic (WordLlama), 10 potongan | 90 | 90 | 60 | 50 | **72,5** | 2,3k |
| + semantic, 20 potongan | 95 | 95 | 65 | 65 | **80,0** | 4,5k |
| mode `full` (fakta L2), k=30, 10 potongan | 95 | 90 | 50 | 50 | **71,2** | 3,3k |
| mode `full`, k=50, 20 potongan | 95 | 95 | 65 | 65 | **80,0** | 6,0k |

n=20 per kategori, jadi selisih per kategori di bawah ±15 poin belum bermakna. Untuk mode `full`, metrik ini
hanya menghitung potongan transcript; fakta yang sudah memuat informasinya (mis. "Jolene bought a pendant in
Paris") tidak ikut dihitung, jadi manfaat `full` di sini terlalu rendah.

Kesimpulan: **jumlah potongan adalah tuas termurah** (multi-hop 35 → 65%) dan konteksnya tetap ≈5× lebih kecil
dari riwayat penuh (≈26k token). Semantic membantu bila potongannya sedikit. Run ini juga tidak menguji fitur
inti rewire (konsolidasi menjadi fakta + kata kunci lintas bahasa), karena memakai mode `episodes`.

## 4. Yang dilakukan sistem lain (riset)

- **Hindsight** (91,4% LongMemEval, 89,6% LoCoMo menurut paper-nya): retrieval multi-strategi (vektor + BM25 +
  graph entitas + filter waktu) digabung RRF lalu di-rerank cross-encoder; saat menyimpan, entitas
  di-resolve dan dihubungkan. Keunggulannya paling besar di multi-session dan open-domain.
- **Mastra observational memory** (94,9% dengan GPT-5-mini, 84,2% dengan GPT-4o): agen Observer menulis
  observasi ringkas terus-menerus, Reflector menggabungkannya; riwayat mentah diganti log observasi.
- **T-Mem**: rute "bridge" khusus untuk multi-hop yang menarik bukti lintas adegan, terpisah dari rute entitas.
- **Second-hop retrieval** (usulan PRISM): untuk soal daftar dan multi-hop, query lanjutan dari hasil pertama,
  digabung RRF. Motivasinya sama dengan temuan kita: bukti multi-hop berperingkat rendah.

## 5. Rekomendasi, diurutkan dari biaya terendah

1. **Perbaiki parser jawaban** (selesai): F1/BLEU sekarang mengukur jawaban, bukan penalaran.
2. **Potongan adaptif:** 20 potongan untuk soal daftar/agregasi/multi-hop ("what … has X", "how many", bentuk
   jamak), 10 untuk sisanya. rewire sudah punya intent hitungan; perluas ke soal daftar.
3. **Judge berbeda dari penjawab** untuk J yang kredibel (model murah lain), dan kalibrasi pada ±100 soal.
4. **Second-hop berbasis entitas:** ambil nama dan kata benda dari hasil pertama, cari ulang, gabung RRF.
   Menjembatani "Evan" → giliran tanpa nama, dan sebagian kesenjangan kata.
5. **Potongan lebih kecil + perluasan tetangga:** giliran tunggal sebagai unit ranking, lalu ±1 giliran di
   sekitarnya: precision naik, lebih banyak bukti berbeda dalam anggaran yang sama.
6. **Evaluasi mode `full`** (fakta + kata kunci lintas bahasa dari konsolidasi), karena itu cara rewire dipakai
   sebenarnya. Dengan model gratis: ±2 panggilan × 272 sesi LoCoMo; batas harian jadi kendala.
7. Lebih jauh: indeks entitas (siapa → giliran mana) dan reranker; keduanya terbukti di Hindsight.

Sumber: [Hindsight (arXiv 2512.12818)](https://arxiv.org/html/2512.12818v1) ·
[Mastra observational memory](https://mastra.ai/research/observational-memory) ·
[T-Mem (arXiv 2606.15405)](https://arxiv.org/pdf/2606.15405) ·
[PRISM issue #94: second-hop retrieval](https://github.com/dwamianm/prism/issues/94) ·
[QueryLink (ACL Findings 2026)](https://aclanthology.org/2026.findings-acl.765.pdf) ·
[LoCoMo overview](https://www.emergentmind.com/topics/locomo).

## 6. Ablasi pada seluruh 1.540 soal (tanpa LLM, 2026-10-14)

Persen soal dengan semua bukti di konteks (mode `episodes`, k=30):

| Konfigurasi | single | temporal | multi-hop | open | **overall** | konteks |
|---|---|---|---|---|---|---|
| keyword, 10 potongan | 94,5 | 83,4 | 32,3 | 40,2 | **77,5** | 2,5k |
| keyword, 20 potongan | 97,5 | 88,1 | 44,3 | 50,0 | **82,9** | 4,9k |
| semantic, 10 potongan | 92,6 | 80,9 | 28,4 | 40,2 | **75,2** | 2,4k |
| semantic, 20 potongan | 97,1 | 90,0 | 42,6 | 47,8 | **82,7** | 4,6k |

Pada sampel penuh, semantic (WordLlama) **tidak** membantu LoCoMo (sama dengan temuan iterasi 6); kenaikan di
sampel 80 soal tadi adalah noise. 20 potongan memberi +5,4 poin overall dan +12 poin multi-hop, tapi multi-hop
tetap di 44%: menambah jumlah saja tidak cukup, perlu perubahan cara mencari (entitas, unit lebih kecil).

## 7. Perbaikan retrieval: bentuk potongan (tanpa LLM, 2026-10-14)

Pertanyaan: dengan anggaran token yang sama, bisakah konteks memuat lebih banyak bukti? Semua angka: seluruh
1.540 soal LoCoMo, mode `episodes`, keyword, k=30 fakta; "bukti" = persen soal dengan semua bukti di konteks.

| Varian | k=10 | k=15 | k=20 | k=25 | Hasil |
|---|---|---|---|---|---|
| Baseline (jendela 6 giliran, langkah 4) | 77,5% / 2.458 tok | – | 82,9% / 4.795 tok | – | acuan |
| Unit 1 giliran ± 1 tetangga (lalu digabung) | – | – | – | 74,4% / 2.655 tok | **lebih buruk** pada token sama |
| **Gabung potongan yang tumpang tindih** | 77,5% / 2.196 tok | 80,4% / 3.223 tok | 82,9% / 4.219 tok | 84,8% / 5.174 tok | **bukti sama, token −11–12%** |
| Pangkas baris yang tidak memuat kata pertanyaan (±1/±2 baris) | – | – | – | – | bukti −0,2 s.d. −2 poin |
| Pseudo-relevance feedback (3/5/10 potongan teratas) + gabung | 77,1–77,7% | 80,5–80,6% | 82,9–83,2% | – | netral (dalam noise) |

Kenapa gabung berhasil: jendela 6 giliran bergeser 4, jadi dua potongan bersebelahan berbagi 2 giliran; bila
keduanya terambil, giliran yang sama dikirim dua kali. Potongan satu sesi yang bertumpuk atau bersambung kini
dijadikan satu potongan (tiap baris sekali), di peringkat potongan terbaiknya. Ringkasan sesi tidak pernah
digabung.

Kenapa unit kecil dan pemangkasan gagal: bukti LoCoMo sering ada di giliran yang tidak memuat kata pertanyaan
(jawaban ada di giliran sebelum/sesudah yang cocok); memotong konteks di sekitar kata kunci membuang bukti itu.
PRF menambah kata dari potongan teratas, tapi kata yang sering muncul di sana kebanyakan topik obrolan umum, bukan
jembatan ke bukti kedua.

**LongMemEval_S (120 soal, 20 per tipe, keyword):** bukti identik (giliran 87,6%, semua bukti 85,5%, sesi 94,9%);
konteks 11.090 → 9.780 token (−11,8%), MRR 0,709 → 0,794, context precision 16,9% → 21,3%; latensi retrieval
sama (p50 ≈170 ms).

**Diterapkan:** penggabungan jadi perilaku bawaan di Python (`Index._merge`) dan Rust (`merge_excerpts`); parity
recall Rust–Python tetap 304/304 (keyword) dan 298/304 (semantic, sama dengan sebelumnya). Varian lain tidak
dipertahankan.

**Yang belum terpecahkan:** multi-hop (32–44%) dan open-domain. Transcript mentah saja tidak cukup; langkah
berikutnya yang paling menjanjikan adalah hop kedua lewat entitas/fakta (mode `full`, yang dulu mencapai J 94%
pada k=50), diukur pada anggaran token tetap (≈2,5k), bukan menambah jumlah potongan.

**Evaluasi ulang dengan LLM** (gpt-4o-mini sebagai penjawab dan judge) dijalankan di sesi evaluasi, sesuai
runbook `docs/evaluation.md` §9: sesi ini tidak punya `OPENROUTER_API_KEY`. Karena balasan di-cache per prompt,
hanya soal yang konteksnya berubah yang dibayar ulang.

## 8. Ide dari sistem lain: entitas, hop kedua, keragaman sesi (tanpa LLM, 2026-10-14)

Sumber ide: Mem0 2026 (entity boost via spaCy, diredam untuk entitas yang sering muncul), Hindsight dan
CortexDB/OpenHuman (jalur entitas di RRF, hasil diselang-seling antar sesi/scope), SmartSearch dan paper
Entity–Memory Graph (perluasan entitas berbasis aturan). OpenHuman v2 sendiri tidak lagi punya retrieval lokal:
peringkat dan multi-hop berjalan di server CortexDB yang tertutup; yang terbuka hanya lapisan klien (anggaran
1.200 token per bagian, tanggal sebagai boost, bukan filter; rewire sudah begitu).

**Diagnosis dulu:** bukti multi-hop ada, tapi peringkatnya dalam. Mode `episodes`, keyword:

| Potongan | Konteks | Multi-hop | Open-domain | Overall |
|---|---|---|---|---|
| 10 | 2,2k | 32,3% | 40,2% | 77,5% |
| 20 | 4,2k | 44,3% | 50,0% | 82,9% |
| 50 | 9,7k | 61,3% | 57,6% | 89,1% |
| 100 | 17,9k | 79,8% | 78,3% | 94,5% |
| 200 | 26k (≈ seluruh percakapan) | 99,6% | 100% | 99,9% |

**Hasil (persen soal dengan semua bukti, k=10 / k=20):**

| Varian | Overall | Multi-hop | Hasil |
|---|---|---|---|
| Baseline | 77,5 / 82,9 | 32,3 / 44,3 | acuan |
| Maks. 1 potongan per sesi (sisanya menyusul) | 70,2 / 73,2 | 27,3 / 33,7 | lebih buruk |
| Maks. 2 potongan per sesi | 75,8 / 79,9 | 31,6 / 42,2 | lebih buruk |
| Jalur entitas (kata berhuruf kapital di pertanyaan, df ≤ 5%) di RRF | 77,5 / 82,8 | 32,3 / 44,3 | netral |
| Hop kedua lewat entitas dari 5 potongan teratas, bobot 0,5 | 71,3 / 80,4 | 25,5 / 40,8 | lebih buruk |
| Hop kedua, bobot 1,0 | 70,2 / 76,5 | 23,4 / 35,8 | lebih buruk |

Kenapa tidak berhasil di LoCoMo: soal multi-hop yang gagal hampir semuanya soal agregasi dengan kosakata
berbeda dari buktinya ("Which countries has Deborah traveled to?" ↔ "I was in Rio de Janeiro"; "favorite
desserts" ↔ "coconut milk icecream"; "as a child" ↔ "as a kid"). Entitas di pertanyaan biasanya hanya nama
pembicara, yang muncul di hampir semua potongan; entitas baru dari hasil teratas membawa topik lain, bukan bukti
kedua. Bukti biasanya tersebar di banyak sesi, tetapi membatasi potongan per sesi justru membuang bukti yang
sesinya memang relevan.

**Yang membantu: fakta (mode `full`) pada anggaran token yang sama.** Bukti dihitung juga bila fakta yang
terambil diekstrak dari giliran bukti (`experiments/locomo/evidence.py --facts`; 1.535 soal, brain mode full
dari iterasi sebelumnya):

| Isi konteks | Konteks | Multi-hop | Open-domain | Overall |
|---|---|---|---|---|
| episodes, 15 potongan | 3,2k | 36,9% | 45,7% | 80,4% |
| full: 30 fakta + 10 potongan | 3,0k | 41,5% | 44,6% | 82,5% |
| full: 50 fakta + 10 potongan | 3,7k | 45,4% | 50,0% | 83,8% |
| episodes, 20 potongan | 4,2k | 44,3% | 50,0% | 82,9% |
| **full: 30 fakta + 20 potongan** | **4,7k** | **51,4%** | **52,2%** | **86,4%** |
| full: 100 fakta saja | 3,4k | 35,1% | 47,8% | 66,4% |

Fakta saja kehilangan detail (single-hop hanya 74%), tetapi fakta + potongan sumbernya memberi +3,5 poin overall
dan +7 poin multi-hop dibanding potongan saja pada konteks ≈4,5k. Biayanya ada di ingest (satu ekstraksi LLM per
sesi), bukan di query.

**Kesimpulan:** tidak ada varian berbasis entitas yang diadopsi. Tuas berikutnya yang tersisa:
1. Mode `full` untuk LoCoMo/LongMemEval di sesi evaluasi (biaya ingest gpt-4o-mini, sekali per brain).
2. Encoder kontekstual kecil (bge-small / e5-small, ONNX) menggantikan WordLlama, dan/atau cross-encoder kecil
   untuk mengurutkan ulang 50 kandidat teratas. Ini yang paling mungkin menutup selisih kosakata, tetapi
   modelnya tidak bisa diunduh di sesi ini (Hugging Face dan mirror GCS fastembed ditolak proxy, 403).

## 9. Menuju multi-hop 80% (2026-10-14)

Angka 79,8% multi-hop dari CortexDB/OpenHuman adalah **J** (jawaban LLM yang dinilai judge, klaim vendor),
bukan "semua bukti ada di konteks". J lebih longgar: jawaban bisa benar dari bukti parsial. Di run Nemotron
(80 soal), multi-hop J 95% padahal bukti lengkap hanya 35%; run lama mode `full` k=50 (885 soal, judge Mem0):
multi-hop J 96,5%. Jadi target 80% harus dibuktikan dengan J gpt-4o-mini di sesi evaluasi.

Untuk ukuran bukti (tanpa LLM, 1.535 soal, mode `full`, keyword kecuali disebut):

| Fakta + potongan | Konteks | Multi-hop | Open-domain | Overall |
|---|---|---|---|---|
| 30 + 20 | 4,7k | 51,4% | 52,2% | 86,4% |
| 30 + 20, semantic | 4,2k | 52,8% | 54,3% | 86,4% |
| 50 + 20 | 5,3k | 56,0% | 57,6% | 87,8% |
| 30 + 40 | 7,6k | 61,7% | 59,8% | 89,2% |
| 100 + 20 | 7,1k | 64,2% | 68,5% | 90,3% |
| 100 + 20, semantic | 6,8k | 66,0% | 66,3% | 90,0% |
| 200 + 20 | 10,0k | 71,6% | 81,5% | 92,7% |
| 100 + 40 | 10,2k | 73,8% | 71,7% | 92,7% |
| episodes saja, 100 potongan | 17,9k | 79,8% | 78,3% | 94,5% |

Per token, fakta lebih efisien daripada potongan (100+20 di 7k token mengalahkan 30+40 di 7,6k). Bukti multi-hop
lengkap 80% butuh ±12k token; itu bukan tujuan, karena konteks besar menurunkan presisi jawaban. Konfigurasi yang
diuji dengan J: 30+20 (≈4,7k) dan 100+20 (≈7k, setara Mem0 2026 dengan 200 fakta ≈7k token, J 91,6% pakai
GPT-5). Perintahnya ada di runbook `docs/evaluation.md` §9.

## 10. Posisi retrieval rewire dan teknik RAG (NirDiamant/rag_techniques), 2026-10-14

**Kuat:** single-hop 94–99% dan temporal 83–95% bukti lengkap pada 2–5k token; query tanpa LLM; retrieval
dalam milidetik (LoCoMo p50 4 ms); potongan bertumpuk digabung.
**Lemah:** multi-hop dan open-domain. Bukti yang kosakatanya berbeda dari pertanyaan berperingkat dalam (MRR
multi-hop 0,55); embedding statis (WordLlama) tidak menutup celah itu, dan tidak ada reranker. Sistem tanpa LLM
saat query yang skornya tinggi (SmartSearch, Hindsight) memakai cross-encoder dan encoder kontekstual.

| Teknik (nomor di repo) | Status di rewire |
|---|---|
| Proposition chunking (5) | ada: fakta mode `full` |
| Choose chunk size (4), semantic chunking (12) | diuji: 6 giliran / langkah 4 lebih baik dari unit 1 giliran |
| Relevant segment extraction (10), context window enrichment (11) | ada: penggabungan potongan (§7) |
| Contextual compression (13) | diuji (pangkas baris tanpa LLM): lebih buruk |
| Fusion retrieval (15) | ada: BM25 + WordLlama, RRF |
| Intelligent reranking (16) | belum: cross-encoder perlu file model (diblokir proxy di sesi ini) |
| Multi-faceted filtering (17) | ada: jendela waktu sebagai boost |
| Hierarchical indices (18), RAPTOR (32) | sebagian: ringkasan sesi diindeks; brain LoCoMo belum punya ringkasan per sesi |
| Contextual chunk headers (9) | sebagian: tanggal dan ringkasan ikut di hasil; belum diuji ke indeks |
| Dartboard / diversity (20) | diuji (batas per sesi): lebih buruk |
| Feedback loops (22) | ada untuk fakta: aktivasi ACT-R dari event `use` |
| Adaptive retrieval (23) | pernah (count intent): dicabut, jawaban kelebihan hitung |
| Graph RAG, LightRAG, GraphRAG (29–31, 36) | sebagian: relasi + spreading activation; hop entitas diuji: lebih buruk (§8) |
| Query transformation, HyDE, HyPE-query, Self/Corrective RAG, agentic (6–8, 28, 33–34) | tidak cocok: LLM saat query (agen pemanggil sudah bisa menulis ulang query sendiri) |
| **Document augmentation / question generation (14), HyPE (8)** | **paling menjanjikan**: LLM saat ingest, query tetap tanpa LLM |

**Uji augmentasi dokumen dengan fakta yang sudah ada** (indeks potongan ditambah teks fakta yang diekstrak dari
gilirannya; teks yang dikirim tidak berubah; 1.535 soal, keyword):

| Konfigurasi | Konteks | Multi-hop | Overall |
|---|---|---|---|
| 10 potongan, indeks biasa | 1,7k | 25,9% | 75,8% |
| 10 potongan, indeks + fakta | 2,0k | 31,9% | 80,2% |
| 20 potongan, indeks biasa | 3,3k | 38,3% | 81,2% |
| 20 potongan, indeks + fakta | 3,9k | 43,6% | 84,6% |
| 30 fakta + 20 potongan, indeks biasa | 4,7k | 51,4% | 86,4% |
| 30 fakta + 20 potongan, indeks + fakta | 5,0k | 50,0% | 86,4% |

Kosakata hasil ekstraksi membuat potongan lebih mudah ditemukan (+3–4 poin pada token setara bila konteks hanya
potongan), tetapi bila fakta sudah ikut dikirim, seeding fakta → potongan sumber sudah memberi efek yang sama.
Tidak diadopsi. Ini mengisyaratkan bahwa pertanyaan hipotetis per potongan (teknik 14), yang memakai bentuk
pertanyaan ("Which countries has Deborah visited?"), bisa memberi lebih; perlu LLM saat ingest untuk diuji.
