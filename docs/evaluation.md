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

Mode ingest:
- `episodes`: transcript disimpan dan diindeks persis seperti hasil konsolidasi, **tanpa** ekstraksi LLM. Gratis.
- `full`: setiap sesi dikonsolidasi oleh LLM (`--ingest-model`), inbox diterima semua. Paling realistis, paling mahal.

## 3. Metrik

| Metrik | Definisi | LLM? | Dilaporkan oleh |
|---|---|---|---|
| **J (accuracy)** | Persen jawaban yang dinilai benar oleh judge dengan prompt resmi | ya | run.py |
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

Contoh dry-run (sampel kecil, hanya ilustrasi): LongMemEval MRR 0,90, context precision 20%, kompresi ≈10,7×;
LoCoMo MRR 0,62, precision 11%, kompresi ≈5,4×. Precision rendah karena potongan berisi 6 giliran: ini sasaran
perbaikan yang jelas (potongan lebih kecil atau pemangkasan) tanpa menurunkan recall.

### Dimensi kemampuan → kategori benchmark

| Dimensi | Diukur oleh |
|---|---|
| Fact recall (*needle-in-a-haystack*) | LoCoMo single-hop; LongMemEval single-session-user / -assistant |
| Belief update (fakta lama diganti yang baru) | LongMemEval knowledge-update; uji unit supersede di repo |
| Temporal reasoning | LoCoMo temporal; LongMemEval temporal-reasoning |
| Cross-session synthesis | LoCoMo multi-hop; LongMemEval multi-session |
| Konsistensi preferensi | LongMemEval single-session-preference; kandidat: PersonaMem |
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

## 8. Cara menjalankan

Prasyarat: `openrouter.ai` diizinkan di network policy environment, dan `OPENROUTER_API_KEY` dipasang sebagai
secret environment (jangan ditempel di chat atau di-commit).

```bash
# 0 token: retrieval saja
python experiments/eval/run.py --bench longmemeval --data longmemeval_s_cleaned.json --out OUT --sample 0 --retrieval-only

# 0 token: perkiraan token dan biaya run sebelum dijalankan
python experiments/eval/run.py --bench locomo --data locomo10.json --out OUT --sample 20 --dry-run \
    --answer-model openai/gpt-4o-mini --judge-model openai/gpt-4o-mini

# regresi murah: 20 soal per kategori, dengan batas biaya
python experiments/eval/run.py --bench longmemeval --data longmemeval_s_cleaned.json --out OUT --sample 20 \
    --answer-model openai/gpt-4o-mini --judge-model openai/gpt-4o-mini --max-cost 1

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

## 9. Berikutnya

- Judge alternatif yang lebih murah: Jev (keputusan ya/tidak dengan probabilitas) dengan cascade ke judge LLM
  untuk kasus ragu; perlu dikalibrasi dulu terhadap judge resmi pada ~100 soal.
- Head-to-head dengan Mem0 dan Graphiti di harness yang sama (§4).
- Benchmark tambahan bila relevan: PersonaMem, BEAM.
- Faithfulness (apakah jawaban hanya memakai isi konteks): judge tambahan, hanya bila halusinasi jadi masalah.
- Metrik produk dari dogfooding (§3).
