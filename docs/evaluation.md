# Evaluasi rewire

Dokumen ini menetapkan **apa yang kita ukur, bagaimana, dengan biaya berapa**, dan bagaimana angka rewire
dibandingkan dengan sistem memory lain. Runner-nya: [`experiments/eval/run.py`](../experiments/eval/run.py)
(OpenRouter, atau endpoint lain yang kompatibel dengan OpenAI).

## 1. Dua tingkat evaluasi

| Tingkat | Kapan | LLM | Biaya | Yang dijawab |
|---|---|---|---|---|
| **Regresi retrieval** | setiap perubahan recall/index | tidak ada | 0 token | Apakah bukti jawaban ikut terambil, dan berapa besar konteksnya? |
| **Skor end-to-end (J)** | milestone, atau sebelum klaim angka publik | penjawab + judge (+ ingest bila `--mode full`) | lihat §5 | Apakah jawaban akhir benar, dan bisa dibandingkan dengan sistem lain? |

Skor publik (Mem0, Zep, dll.) selalu **end-to-end**: retrieval → LLM menjawab → LLM judge menilai. Metrik
retrieval tidak ada padanannya di leaderboard, tapi gratis dan stabil, jadi dipakai sehari-hari.

## 2. Benchmark yang dipakai

| Benchmark | Isi | Yang dinilai | Protokol kita |
|---|---|---|---|
| **LoCoMo** | 10 percakapan panjang (~27 sesi), 1.540 pertanyaan kategori 1–4 | single-hop, multi-hop, temporal, open-domain | Prompt jawaban + judge resmi Mem0 (`experiments/locomo/official_prompts.py`); kategori 5 (adversarial) dikecualikan seperti Mem0 |
| **LongMemEval_S** | 500 pertanyaan, tiap pertanyaan punya riwayat ~50 sesi (~115k token) | single-session (user / assistant / preference), multi-session, knowledge-update, temporal-reasoning, abstention (30 soal `_abs`) | Template jawaban + prompt judge resmi per tipe (`experiments/longmemeval/lme_prompts.py`) |
| **BEAM** (ICLR 2026) | 100 chat (100K: 20, 500K: 35, 1M: 35, 10M: 10), 2.000 soal, 10 kategori × 2 soal per chat | abstention, contradiction resolution, event ordering, information extraction, instruction following, knowledge update, multi-session reasoning, preference following, summarization, temporal reasoning | Prompt jawaban RAG + judge per butir rubrik (1 / 0,5 / 0), disalin verbatim (`experiments/eval/beam_prompts.py`, MIT); skor soal = rata-rata rubrik. Ukuran 10M belum didukung; Kendall tau untuk event ordering belum dihitung (skor rubrik saja) |
| **PersonaMem** (COLM 2025) | 180+ riwayat persona, sampai 60 sesi; versi konteks 32k / 128k / 1M | 7 tipe pertanyaan, mis. mengingat fakta pengguna, mengikuti perubahan preferensi, rekomendasi baru yang sesuai preferensi | Pilihan ganda (a–d) dengan instruksi dan pencocokan jawaban resmi: **tanpa judge LLM**. Satu brain per konteks bersama; potongan setelah posisi pertanyaan dibuang supaya memory tidak berisi "masa depan" |

Mode ingest:
- `episodes`: transcript disimpan dan diindeks persis seperti hasil konsolidasi, **tanpa** ekstraksi LLM. Gratis.
- `full`: setiap sesi dikonsolidasi oleh LLM (`--ingest-model`), inbox diterima semua. Paling realistis, paling mahal.
  BEAM dan PersonaMem hanya mendukung `episodes` (konsolidasi chat 1M token terlalu mahal; untuk PersonaMem,
  fakta hasil ekstraksi bisa membocorkan giliran setelah pertanyaan).

Sumber data:
- LoCoMo: `locomo10.json`, langsung dari GitHub:
  `https://raw.githubusercontent.com/snap-research/locomo/main/data/locomo10.json`.
- LongMemEval_S: `longmemeval_s_cleaned.json` (Hugging Face; di sesi ini lewat GCS karena HF diblokir proxy).
- BEAM: folder `chats/` di repo `github.com/mohammadtavakoli78/BEAM` (di-clone lewat git; divalidasi pada
  commit `b2da22e`). Divalidasi:
  100K/500K/1M = 20/35/35 chat dengan 400/700/700 soal (20 per chat, 10 kategori), ukuran chat ≈146k / 538k /
  1,09M token (median), cocok dengan paper; skrip `download_dataset.py` resmi mengubah data Hugging Face ke format
  yang sama. Perbandingan byte dengan salinan Hugging Face belum dilakukan (HF diblokir).
- PersonaMem: Hugging Face `bowen-upenn/PersonaMem`, disalin ke GCS (`bucket_for_claude`):
  `questions_32k.csv`, `shared_contexts_32k.jsonl`, `questions_1M.csv`, `shared_contexts_1M.jsonl`.
  Divalidasi: 32k = 589 soal, 37 konteks bersama, ≈27k token per konteks; 1M = 2.674 soal, 33 konteks,
  ≈925k token per konteks (median); 15 kolom sesuai README, semua referensi konteks ada, kunci jawaban
  tersebar rata (a)–(d). Nama tipe pertanyaan berbeda antara 32k dan 1M (mis. `track_full_preference_evolution`
  vs `track_full_preference_updates`; 1M punya `acknowledge_latest_user_preferences`), jadi skor per tipe
  32k dan 1M tidak dibandingkan langsung.

## 3. Metrik

| Metrik | Definisi | LLM? | Dilaporkan oleh |
|---|---|---|---|
| **J (accuracy)** | Persen jawaban yang dinilai benar oleh judge dengan prompt resmi (BEAM: rata-rata skor rubrik 0–1; PersonaMem: akurasi pilihan ganda tanpa judge) | ya (PersonaMem: tidak untuk penilaian) | run.py |
| **F1** (LoCoMo) | F1 token antara jawaban dan gold (metrik asli paper LoCoMo) | jawaban saja | run.py |
| **BLEU-1** (LoCoMo) | Presisi unigram dengan brevity penalty | jawaban saja | run.py |
| **Abstention** (LongMemEval) | Persen soal tak-terjawab yang dikenali sebagai tak-terjawab | ya | run.py |
| **Evidence turn recall** | Persen giliran bukti yang muncul di konteks hasil recall | tidak | run.py, `--retrieval-only` |
| **All evidence** | Persen soal yang **semua** buktinya ada di konteks | tidak | run.py |
| **MRR** | 1 / peringkat potongan pertama yang memuat bukti (0 bila tidak ada), dirata-rata: bukti ada di urutan atas atau terkubur | tidak | run.py |
| **Context precision** | Persen potongan terambil yang memuat bukti; sisanya *noise* yang mencemari konteks (noise ratio = 1 − precision) | tidak | run.py |
| **Context tokens** | Besar konteks memory per soal (mean, p95; ≈ karakter/4) | tidak | run.py |
| **Compression ratio** | Token riwayat penuh ÷ token konteks rewire: berapa kali lebih hemat dari mengirim seluruh riwayat (*full-context*) | tidak | run.py |
| **Prompt / completion / reasoning tokens per response** | Dari `usage` API, per tahap (ingest / answer / judge) | ya | run.py |
| **Biaya (USD)** | `usage.cost` dari OpenRouter, per tahap dan per soal | ya | run.py |
| **Latency** | Retrieval (ms, p50/p95/p99) dan panggilan LLM (p50/p95) | — | run.py |
| **Panjang jawaban** | Rata-rata kata per jawaban (jawaban panjang cenderung dinilai benar oleh judge yang longgar) | — | results JSON |

Hasil retrieval yang sudah diukur ada di §8. Context precision rendah (≈11–20%) karena satu potongan berisi
6 giliran: ini sasaran perbaikan yang jelas (potongan lebih kecil atau pemangkasan) tanpa menurunkan recall.

### Dimensi kemampuan → kategori benchmark

| Dimensi | Diukur oleh |
|---|---|
| Fact recall (*needle-in-a-haystack*) | LoCoMo single-hop; LongMemEval single-session-user / -assistant |
| Belief update (fakta lama diganti yang baru) | LongMemEval knowledge-update; uji unit supersede di repo |
| Temporal reasoning | LoCoMo temporal; LongMemEval temporal-reasoning |
| Cross-session synthesis | LoCoMo multi-hop; LongMemEval multi-session |
| Konsistensi preferensi | LongMemEval single-session-preference; PersonaMem; BEAM preference following |
| Kontradiksi, urutan kejadian, instruksi jangka panjang, ringkasan | BEAM contradiction resolution, event ordering, instruction following, summarization |
| Skala sangat panjang (500K–10M token) | BEAM |
| Tahu kapan tidak tahu | LongMemEval abstention |

Metrik produk (bukan benchmark, diukur dari pemakaian nyata): fresh-first rate (nilai terbaru dipakai
saat fakta berubah), captured-but-not-surfaced (fakta tersimpan tapi tidak muncul saat dibutuhkan),
item inbox per sesi, hit rate snapshot (`memory used` di `sessions/*.md`).

## 4. Skor sistem lain (dipublikasikan)

**Peringatan:** angka di bawah tidak sebanding satu sama lain. Model penjawab, model judge, prompt, dan
subset berbeda-beda, dan banyak yang dilaporkan oleh vendornya sendiri. Bandingkan hanya bila protokolnya
sama (§6).

### LoCoMo (J, kategori 1–4)

| Sistem | J | Token / latency | Sumber, catatan |
|---|---|---|---|
| Full-context (semua percakapan di prompt) | ≈72,9% | ≈26k token per percakapan, p95 ≈17 s | Paper Mem0 (2025) |
| Mem0 | ≈66,9% (Mem0ᵍ ≈68,4%) | ≈7k token, p95 ≈1,44 s | Paper Mem0 (2025) |
| Zep | ≈66,0% | | Paper Mem0 (2025); Zep melaporkan 94,7% sendiri, pihak ketiga 75,1% |
| LangMem | ≈58,1% | latency ≈60 s | Paper Mem0 (2025) |
| OpenAI Memory | ≈52,9% | | Paper Mem0 (2025) |
| A-Mem | ≈48,4% | | Paper Mem0 (2025) |
| Mem0 (platform 2026) | 92,5% | median 6,7–7k token per retrieval, p50 0,88–1,09 s | Laporan vendor |
| Evaluasi Memori (pihak ketiga) | Zep 79,1% · LangMem 78,1% · Mem0 62,5% | Mem0 ≈1.764 token per soal | Memori Labs |
| **rewire** | **94,0–94,9%** | konteks ≈2,7k token per soal (dry-run conv-26, k=30, 10 potongan) | Iterasi 6–7 (k=50); penjawab + judge Claude via `claude -p`, prompt Mem0; **belum** dengan model yang sama dengan Mem0 |

### LongMemEval_S (akurasi, 500 soal)

| Sistem | Akurasi | Sumber, catatan |
|---|---|---|
| Zep | 71,2% | Judge GPT-4o; angka yang paling konsisten di beberapa sumber |
| Supermemory | 85,2% | Dirangkum di paper Agent Zero Memory (2026) |
| EmergenceMem | 86,0% | idem |
| Hindsight | 91,4% | idem |
| ByteRover | 92,8% | Laporan vendor |
| Mem0 | 94,4% | Laporan vendor |
| Mastra (observational memory) | 94,9% (GPT-5-mini); 84,2% dengan GPT-4o | Laporan vendor; menunjukkan besarnya pengaruh model penjawab |
| Agent Zero Memory | 95,6% | Paper Agent Zero Memory (2026) |
| **rewire** (`episodes`, tanpa ekstraksi) | **85,4%** (10 potongan); ≈89,8% (20 potongan) | Iterasi 8; penjawab + judge Claude via `claude -p`, prompt resmi |

Retrieval rewire (tanpa LLM, LongMemEval_S, 470 soal non-abstention): sesi bukti ditemukan 97,0%, semua
sesi bukti 87,9%, konteks rata-rata ≈11,2k token dari ≈115k.

Sumber: [paper Mem0](https://arxiv.org/pdf/2504.19413) ·
[Mem0: benchmarks 2026](https://mem0.ai/library/agent-memory/ai-memory-benchmarks-in-2026) ·
[Mem0: state of agent memory 2026](https://mem0.ai/blog/state-of-ai-agent-memory-2026) ·
[Memori LoCoMo results](https://memorilabs.ai/docs/memori-cloud/benchmark/results/) ·
[Agent Zero Memory](https://arxiv.org/pdf/2608.29606) ·
[Mastra observational memory](https://mastra.ai/research/observational-memory) ·
[ZenBrain (catatan soal angka vendor)](https://arxiv.org/pdf/2604.23878) ·
[User as Code (reimplementasi Mem0 29,3%)](https://arxiv.org/pdf/2606.16707).

### Model penjawab dan judge yang dipakai pembanding (dicek dari kode evaluasi mereka, 2026-10-14)

| Angka | Penjawab | Judge | Sumber |
|---|---|---|---|
| Paper Mem0 2025 (LoCoMo: Mem0 ≈66,9%, Mem0ᵍ ≈68,4%, Zep ≈66,0%, full-context ≈72,9%) | `gpt-4o-mini` (env `MODEL`, default README) | `gpt-4o-mini` (tertanam di `metrics/llm_judge.py`) | repo `mem0ai/mem0`, folder `evaluation/` (commit `d85fcda`) |
| Mem0 platform 2026 (LoCoMo 92,5%, LongMemEval 94,4%) | `gpt-5` | `gpt-5` | repo `mem0ai/memory-benchmarks`, `results/platform/*.json` (default runner: `gpt-4o` / `gpt-4o`) |
| Zep LoCoMo 75,14% (sanggahan Zep) | `gpt-4o-mini` | `gpt-4o-mini`, prompt grader milik Zep | repo `getzep/zep-papers`, `locomo_eval/` |
| Zep LongMemEval (paper 2025) | `gpt-4o-mini`, dan `gpt-4o` untuk angka 71,2% | kode: `gpt-4o-mini`, prompt grader milik Zep, bukan prompt resmi LongMemEval | `zep-papers`, `zep_longmem_eval.ipynb` ("models default to gpt-4o-mini") |

Konsekuensinya: untuk dibandingkan dengan paper Mem0 dan angka LoCoMo Zep, pakai `gpt-4o-mini` sebagai
penjawab **dan** judge. Angka Mem0 2026 memakai GPT-5 untuk keduanya, jadi tidak sebanding dengan run yang
memakai model lebih kecil.

### Besar konteks yang dikirim ke penjawab (per soal, LoCoMo)

| Sistem | Konteks / soal | Isi konteks | Skor | Sumber |
|---|---|---|---|---|
| Memori | ≈1.294 token | fakta ringkas | 81,95% | laporan Memori (vendor) |
| Zep | ≈1.602 token | fakta (edge) + ringkasan entitas | 79,1% (Memori), 75,1% (Zep) | paper MemCoT; Memori; Zep |
| Mem0 (paper 2025 / pengukuran pihak ketiga) | ≈1.764 token | 10–30 fakta ringkas | 62,5–68,4% | paper MemCoT; Memori; paper Mem0 |
| Mem0 platform 2026, top-50 | 50 fakta (≈1,7–2k token, perkiraan) | fakta ringkas | 82,7% (GPT-5) | `memory-benchmarks/results/platform/locomo_top50_results.json` |
| Mem0 platform 2026, top-200 | 200 fakta, rata-rata 6.956 token | fakta ringkas | 91,6% (GPT-5) | `locomo_results.json` + laporan Mem0 |
| Full-context | ≈26k token | seluruh percakapan | 72,9% (4o-mini) – 87,5% | paper Mem0; Memori |
| **rewire, 10 potongan** | **≈2,2k token** (≈2,5k sebelum penggabungan) | 10 potongan × 6 giliran transcript mentah | bukti lengkap 77,5% | §8 |
| **rewire, 20 potongan** | **≈4,2k token** (≈4,8k sebelum penggabungan) | 20 potongan transcript mentah | bukti lengkap 82,9% | §8 |

Pelajaran: sistem dengan konteks kecil (≈1,3–1,8k token) mengirim **fakta ringkas**, bukan transcript mentah;
skor tinggi Mem0 2026 justru memakai konteks besar (≈7k token, 200 fakta). rewire mode `episodes` mengirim
transcript mentah dengan context precision hanya 11–20%: tokennya habis untuk giliran yang tidak relevan. Target
yang tepat bukan "konteks lebih besar" tetapi **lebih banyak bukti per token**: KPI retrieval = persen soal dengan
semua bukti pada anggaran token tetap (mis. 2,5k).

### Perbandingan head-to-head (apples-to-apples)

Satu-satunya cara membandingkan secara adil adalah menjalankan Mem0, Zep (Graphiti, open source) dan rewire
**di harness yang sama**: dataset dan sampel sama, model penjawab dan judge sama, embedding lokal yang sama
untuk sistem yang butuh embedding (mis. `bge-small-en-v1.5` atau `all-MiniLM-L6-v2`). Yang perlu diperhatikan:
- **Biaya ingest berbeda jauh.** Mem0 dan Graphiti memanggil LLM saat menulis memory (ekstraksi per pesan /
  per episode); rewire mode `episodes` tidak memanggil LLM sama sekali, mode `full` sekitar 2 panggilan per
  sesi. Token ingest harus dilaporkan terpisah, bukan hanya token saat menjawab.
- **Infrastruktur.** Graphiti butuh graph database (Neo4j/FalkorDB); Mem0 open source butuh vector store.
  Jalankan lokal, tanpa layanan cloud mereka, supaya versinya terkunci dan bisa diulang.
- **Mulai kecil:** 20–50 soal per benchmark untuk ketiga sistem, baru skala penuh bila selisihnya menarik.
Belum diimplementasikan; runner sudah memisahkan token per tahap (ingest / answer / judge), jadi adaptor
Mem0 dan Graphiti tinggal mengisi tahap ingest dan retrieval.

## 5. Token: dari mana datangnya, dan berapa

Per soal ada tiga kemungkinan konsumen token:

| Tahap | Ukuran tipikal per soal | Catatan |
|---|---|---|
| **ingest** (`--mode full` saja) | ~2 panggilan per **sesi** transcript, bukan per soal | LoCoMo: ~272 sesi; LongMemEval: ~50 sesi × 500 soal (setiap soal punya riwayat sendiri), jadi `full` untuk LongMemEval hanya layak untuk sampel kecil |
| **answer** | LoCoMo ≈4,4k prompt (≈1,8k instruksi Mem0 + ≈2,7k konteks); LongMemEval ≈11,4k prompt | **Tahap terbesar**; output 100–500 token (penalaran langkah demi langkah) |
| **judge** | ≈400–600 prompt, ≈10–60 output | ±5% dari total; kandidat untuk model murah atau Jev |
| **judge BEAM** | ≈3 panggilan per soal (satu per butir rubrik), masing-masing ≈1,3k token | judge resmi BEAM tidak murah: ≈20% dari total di dry-run |
| **PersonaMem** | 0 | pilihan ganda, dinilai dengan pencocokan |

Dry-run BEAM 100K (20 soal): ≈13,4k token konteks per soal (potongan berisi jawaban asisten yang panjang),
≈$0,06 per 20 soal dengan harga contoh di atas. Turunkan `--episodes` bila konteks terlalu besar.

Perkiraan dry-run (prompt dihitung dari teks sebenarnya, harga contoh $0,15 / $0,60 per 1M token):
LoCoMo 8 soal ≈ $0,008; LongMemEval 6 soal ≈ $0,012, artinya 500 soal LongMemEval ≈ $1 dengan model sekelas
GPT-4o-mini untuk menjawab dan menilai. Harga sebenarnya diambil dari katalog OpenRouter saat dry-run.

Mengapa run sebelumnya cepat menghabiskan kuota: semua panggilan lewat `claude -p` dengan langganan yang sama
dengan sesi kerja, memakai model default terbesar, menjawab ulang semua soal di setiap run, dan untuk
LongMemEval membawa ≈11–22k token konteks per soal.

## 6. Aturan agar angka bisa dibandingkan

1. **Model sama dengan pembanding.** Catat model penjawab dan judge di setiap hasil (`report-*.md` menulisnya).
   Untuk LongMemEval, judge resmi adalah GPT-4o; untuk LoCoMo pakai model yang dipakai sistem pembanding.
2. **Prompt resmi**, tanpa modifikasi (sudah disalin verbatim di repo).
3. **Subset sama**: LoCoMo kategori 1–4; LongMemEval_S (bukan M, bukan versi penuh).
4. **Ukuran sampel**: interval kepercayaan 95% ≈ ±1,96·√(p(1−p)/n). Untuk p≈0,85: n=120 → ±6,4 poin;
   n=500 → ±3,1 poin. Sampel 20/kategori cukup untuk regresi kasar, bukan untuk klaim publik.
5. **Variasi judge**: J bisa bergeser ±1–3 poin antar run dengan model yang sama; jangan menyimpulkan dari
   selisih kecil.

## 7. Pilihan model

| Peran | Rekomendasi | Alasan |
|---|---|---|
| Judge untuk angka publik | model yang sama dengan pembanding (LongMemEval: GPT-4o) | angka hanya sebanding bila judge-nya sama |
| Judge untuk regresi | model murah yang stabil; cek dulu kecocokannya dengan judge resmi pada ~100 soal | judge hanya ~5% token, tapi harus konsisten antar run |
| Penjawab | satu model tetap untuk semua sistem yang dibandingkan | model penjawab menggeser skor puluhan poin (Mastra: 94,9% vs 84,2% hanya karena model) |
| Ingest (`--mode full`) | model yang andal untuk JSON schema | ekstraksi gagal = memory hilang, bukan sekadar skor turun |

**Model utama (keputusan 2026-10-14, setelah run Nemotron):** `openai/gpt-4o-mini` sebagai penjawab **dan**
judge: sama dengan paper Mem0 dan angka LoCoMo Zep (tabel di §4), judge-nya stabil, bukan model reasoning
(tidak perlu `--reasoning-effort`), dan murah (LoCoMo penuh ≈$1,60–2,30; `openai/gpt-4o-mini:batch` setengahnya).
Untuk LongMemEval, judge resmi adalah GPT-4o: tambahkan run `--judge-model openai/gpt-4o` bila angkanya
akan dibandingkan dengan protokol resmi (jawaban diambil dari cache, hanya judge yang dibayar).

**Model untuk run pertama (2026-10-14):** `nvidia/nemotron-3-ultra-550b-a55b:free` untuk penjawab
dan judge. Bila skornya jauh di bawah batas atas retrieval (§8), atau banyak jawaban kosong / terpotong, coba
`thinkingmachines/inkling:free` pada soal yang sama (seed sama; cache menyimpan balasan per model).
**Catatan 2026-10-14:** `inkling:free` membalas 403 untuk panggilan API langsung (hanya untuk "agentic harness"
terdaftar); pakai `thinkingmachines/inkling` (berbayar) atau model gratis lain. Keduanya
ada di katalog OpenRouter (gratis, konteks ±1M token) dan keduanya model *reasoning*: pakai
`--reasoning-effort low`, karena proses berpikir ikut memakai `max_tokens` dan jawaban bisa kosong
(report memberi peringatan bila itu terjadi). Penjawab dan judge yang sama cenderung menilai dirinya sendiri
lebih longgar: wajar untuk regresi internal, tidak untuk angka publik.

Model gratis (`:free`) di OpenRouter cocok untuk mencoba harness, tapi kurang cocok untuk angka resmi:
- **Rate limit:** batas permintaannya biasanya ketat, jadi run panjang mudah terputus.
- **Ketersediaan:** daftar model gratis bisa berubah, sehingga hasilnya sulit diulang.
- **Privasi data:** cek kebijakan datanya sebelum mengirim riwayat chat sungguhan.
Benchmark publik sendiri tidak berisi data pribadi.
Kunci model (`--answer-model`, `--judge-model`, `--ingest-model`) selalu dicatat di report.

Benchmark lain yang dipertimbangkan:
- **PersonaMem**: relevan untuk preferensi pengguna.
- **BEAM**: memory jangka sangat panjang.
- **GAIA**: tidak cocok, karena menguji asisten umum (tools, web), bukan memory.
- **SWE-bench**: baru relevan bila rewire diukur sebagai memory untuk agent coding jangka panjang.

## 8. Baseline tanpa LLM (sudah diukur)

Batas atas untuk run dengan LLM: bila bukti tidak ada di konteks, model penjawab tidak bisa menjawab benar
(kecuali menebak). Semua angka: mode `episodes`, keyword (tanpa semantic), k=30, 10 potongan. Sejak
2026-10-14 potongan satu sesi yang tumpang tindih digabung (bukti sama, konteks −11–12%; rincian di
`experiments/eval/results/2026-10-14-locomo-nemotron-s20-analysis.md` §7).

**LoCoMo, 1.540 soal (2026-10-14):**

| Kategori | n | Giliran bukti terambil | Semua bukti ada | MRR | Konteks / soal |
|---|---|---|---|---|---|
| single-hop | 841 | 94,5% | 94,5% | 0,81 | ≈2,5k token |
| temporal | 321 | 84,0% | 83,4% | 0,68 | ≈2,5k |
| multi-hop | 282 | 54,9% | 32,3% | 0,55 | ≈2,6k |
| open-domain | 96 | 39,9% | 40,2% | 0,40 | ≈2,4k |
| **overall** | 1.540 | **73,2%** | **77,5%** | 0,71 | ≈2,5k (7,4× lebih kecil dari riwayat penuh); ≈2,2k setelah penggabungan |

Retrieval p50 4 ms, p99 16 ms. Open-domain banyak butuh pengetahuan umum (tidak ada di percakapan); multi-hop
adalah sasaran perbaikan rewire. Catatan: skor J 94% dari iterasi 6–7 memakai k=50.

**LongMemEval_S, 470 soal non-abstention (iterasi 8):** sesi bukti ditemukan 97,0%, semua sesi bukti 87,9%,
konteks ≈11,2k token dari ≈115k (≈10× lebih kecil). Sampel 120 soal setelah penggabungan: bukti sama, konteks
11.090 → 9.780 token, MRR 0,71 → 0,79.

**PersonaMem (dry-run):** 32k, semua 589 soal: konteks ≈12,6k token, kompresi 2,8× (riwayatnya pendek),
perkiraan $1,30 dengan GPT-4o-mini. 1M, 140 soal (20 per tipe): konteks ≈12,7k token dari ≈925k
(kompresi ≈92×), retrieval p50 42 ms / p99 277 ms, perkiraan seluruh 2.674 soal ≈$5,90.

**BEAM 100K (dry-run, 20 soal):** semua bukti ada 52,9%, MRR 0,43, konteks ≈13,4k token (kompresi ≈11×),
judge 52 panggilan untuk 20 soal; information extraction dan preference following 0% bukti di sampel kecil ini.

## 9. Runbook sesi evaluasi

### Prasyarat (sekali, di pengaturan environment)
- `openrouter.ai` di **Allowed domains** (Network access). Sudah dicek terbuka pada 2026-10-14.
- Secret / environment variable: `OPENROUTER_API_KEY` (wajib). Opsional: `OPENROUTER_MODEL` (model default
  untuk penjawab, judge dan ingest), `OPENROUTER_BASE_URL` (default `https://openrouter.ai/api/v1`).
  Secret baru terbaca di **sesi baru**. Jangan tempel key di chat atau di-commit.
- Model ditulis sebagai id (`nvidia/nemotron-3-ultra-550b-a55b:free`); URL halaman model
  (`openrouter.ai/<id>`) juga diterima dan dinormalisasi.
- Python ≥ 3.11 saja; runner tidak butuh paket tambahan (semantic opsional: `pip install -e '.[semantic]'`;
  tanpa itu recall memakai keyword, sama dengan baseline §8).

### Langkah 1: data (sesi baru = container baru, scratchpad lama hilang)

```bash
D=/tmp/eval-data OUT=/tmp/eval-out && mkdir -p $D/personamem $OUT && cd $D
curl -sSfLO https://raw.githubusercontent.com/snap-research/locomo/main/data/locomo10.json
# LongMemEval dan PersonaMem: signed URL baru dari GCS (berlaku 2 jam), satu URL per file
curl -sSf -o longmemeval_s_cleaned.json             '<signed url longmemeval_s_cleaned.json>'
curl -sSf -o personamem/questions_32k.csv           '<signed url questions_32k.csv>'
curl -sSf -o personamem/shared_contexts_32k.jsonl   '<signed url shared_contexts_32k.jsonl>'
curl -sSf -o personamem/questions_1M.csv            '<signed url questions_1M.csv>'
curl -sSf -o personamem/shared_contexts_1M.jsonl    '<signed url shared_contexts_1M.jsonl>'
# BEAM: hanya chat.json + probing_questions.json untuk 100K/500K/1M (tanpa pickle, ±263 MB)
git clone -q --filter=blob:none --no-checkout https://github.com/mohammadtavakoli78/BEAM && cd BEAM
git sparse-checkout init --no-cone
for s in 100K 500K 1M; do printf '/chats/%s/*/chat.json\n/chats/%s/*/probing_questions/probing_questions.json\n' $s $s; done \
  > .git/info/sparse-checkout
git checkout -q main && cd /home/user/rewire
```

Jangan pakai `git ls-tree --long` pada clone `--filter=blob:none`: ia mengunduh setiap blob satu per satu.

### Langkah 2: cek key tanpa menampilkannya

```bash
test -n "$OPENROUTER_API_KEY" && echo "key terbaca (${#OPENROUTER_API_KEY} karakter)"
```

### Langkah 3: run uji paling kecil (memastikan balasan model terbaca dan token tercatat)

```bash
M=nvidia/nemotron-3-ultra-550b-a55b:free
python experiments/eval/run.py --bench locomo --data $D/locomo10.json --out $OUT --sample 2 \
  --answer-model $M --judge-model $M --reasoning-effort low --parallel 2 --max-cost 0.10 --label smoke
```

Periksa di `report-locomo-smoke.md`: tidak ada peringatan jawaban kosong / terpotong, token per tahap terisi,
J masuk akal dibanding §8. Buka beberapa jawaban di `results-locomo-smoke.json`.

### Langkah 4: run sampel, satu benchmark per kali

```bash
# LoCoMo 20 soal per kategori (80 soal), bandingkan dengan §8
python experiments/eval/run.py --bench locomo --data $D/locomo10.json --out $OUT --sample 20 \
  --answer-model $M --judge-model $M --reasoning-effort low --parallel 2 --label nemotron-s20
# bila jauh di bawah batas atas retrieval: model kedua pada soal yang sama
python experiments/eval/run.py ... --answer-model thinkingmachines/inkling:free --judge-model thinkingmachines/inkling:free --label inkling-s20
# lalu benchmark lain dengan flag yang sama:
#   --bench personamem  --data $D/personamem --personamem-size 32k   (lalu 1M; tanpa judge)
#   --bench longmemeval --data $D/longmemeval_s_cleaned.json
#   --bench beam        --data $D/BEAM/chats --beam-sizes 100K
```

Model gratis punya batas per menit dan per hari: `--parallel 2`, dan bila run berhenti (429 / batas harian),
jalankan perintah yang sama lagi nanti: cache melanjutkan dari titik terakhir.

### Langkah 5: simpan hasil ke repo

`$OUT` ada di scratchpad dan hilang bersama sesi. Salin report (bukan cache, bukan brain) ke
`experiments/eval/results/<tanggal>-<bench>-<label>.md`, catat model, sampel, dan seed, lalu commit.

### Opsi runner yang sering dipakai

```bash
# 0 token: retrieval saja
python experiments/eval/run.py --bench longmemeval --data longmemeval_s_cleaned.json --out OUT --sample 0 --retrieval-only
# 0 token: perkiraan token dan biaya run sebelum dijalankan (--price MODEL=IN,OUT bila katalog tidak terjangkau)
python experiments/eval/run.py --bench locomo --data locomo10.json --out OUT --sample 20 --dry-run

```bash
# 0 token: retrieval saja
python experiments/eval/run.py --bench longmemeval --data longmemeval_s_cleaned.json --out OUT --sample 0 --retrieval-only

# 0 token: perkiraan token dan biaya run sebelum dijalankan
python experiments/eval/run.py --bench locomo --data locomo10.json --out OUT --sample 20 --dry-run \
    --answer-model openai/gpt-4o-mini --judge-model openai/gpt-4o-mini

# BEAM (data: folder chats/ dari repo BEAM) dan PersonaMem (data: folder berisi csv + jsonl), dry-run
python experiments/eval/run.py --bench beam --data BEAM/chats --beam-sizes 100K,500K,1M --out OUT --sample 5 --dry-run
python experiments/eval/run.py --bench personamem --data personamem/ --personamem-size 32k --out OUT --sample 20 --dry-run

# angka yang sebanding dengan publik (LongMemEval, judge resmi GPT-4o)
python experiments/eval/run.py --bench longmemeval --data longmemeval_s_cleaned.json --out OUT --sample 0 \
    --answer-model openai/gpt-4o --judge-model openai/gpt-4o --max-cost 20
```

Fitur penghemat token di runner:
- **Cache balasan** per (tahap, model, prompt) di `OUT/cache/llm.jsonl`: run ulang hanya membayar soal yang
  konteks retrieval-nya berubah, atau model baru.
- `--sample N` per kategori (stratified, seed tetap) · `--max-cost` (berhenti, lalu lanjut dari cache) ·
  `--dry-run` dan `--retrieval-only` tanpa LLM.

Output: `OUT/report-<bench>-<label>.md` (tabel per kategori, token per tahap dan per response, biaya, latency)
dan `OUT/results-<bench>-<label>.json` (per soal: konteks, jawaban, putusan, usage).

## 10. Berikutnya

- Retrieval multi-hop (32–44% bukti lengkap): hop kedua lewat entitas/fakta (mode `full`) pada anggaran token
  tetap ≈2,5k. Unit 1 giliran, pemangkasan baris, dan pseudo-relevance feedback sudah dicoba dan tidak membantu.

- Judge alternatif yang lebih murah: Jev (keputusan ya/tidak dengan probabilitas) dengan cascade ke judge LLM
  untuk kasus ragu; perlu dikalibrasi dulu terhadap judge resmi pada ~100 soal.
- Head-to-head dengan Mem0 dan Graphiti di harness yang sama (§4).
- BEAM 10M dan Kendall tau untuk event ordering.
- Faithfulness (apakah jawaban hanya memakai isi konteks): judge tambahan, hanya bila halusinasi jadi masalah.
- Metrik produk dari dogfooding (§3).
