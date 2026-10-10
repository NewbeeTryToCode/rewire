# Review kelayakan build: Synaptic Memory (rev 1)

> Menilai [synaptic-memory.md](synaptic-memory.md) rev 1. Penilaian subjektif, dibuat 2026-10-10.
> Bagian 3 berisi proyek serupa yang dicek langsung (README/paper), bukan hanya dari daftar.

## 1. Skor

Skala 1–10. Bobot menunjukkan seberapa penting dimensi itu untuk *keputusan mulai build*.

| # | Dimensi | Bobot | Skor | Alasan singkat |
|---|---|---|---|---|
| 1 | Tujuan & prinsip | 10% | **9** | P1–P5 jelas, saling konsisten, dan sudah diuji terhadap kritik (temuan Link). |
| 2 | Model penyimpanan | 15% | **8** | Event sourcing + file + index turunan adalah keputusan kuat. Celah: dua device mengedit *file node yang sama* belum diatur. |
| 3 | Model activation / synapse | 10% | **7** | Matematika berdasar (ACT-R, fan effect) dan sudah dibatasi perannya. Celah: "co-use = dipakai di sesi yang sama" terlalu kasar — satu sesi bisa membahas 5 topik, menghasilkan edge palsu. Belum ada normalisasi co-occurrence. |
| 4 | Ontology & granularity node | 10% | **5** | Tipe ada, tapi **ukuran satu node** belum diputuskan (fakta atomik seperti Mem0, atau halaman seperti LLM Wiki?). Ini memengaruhi ekstraksi, entity resolution, dan activation. |
| 5 | Pipeline konsolidasi | 10% | **6** | Langkahnya lengkap, tapi belum ada kontrak input/output tiap langkah, idempotensi (sesi dikonsolidasi dua kali?), dan estimasi biaya. |
| 6 | **Integrasi harness (janji inti produk)** | 15% | **4** | Adapter hanya ada di dokumen riset. Belum ada: daftar tool MCP + skemanya, pemetaan hook per harness, dan **dari mana transcript diambil** di tiap harness (tanpa transcript, konsolidasi tidak bisa jalan). |
| 7 | Keamanan & privasi | 10% | **5** | Disebut di banyak tempat, belum ada threat model (memory poisoning, secrets di transcript, brain yang di-push ke remote publik). |
| 8 | Rencana evaluasi | 10% | **6** | Ablation dan metrik ada; dataset konkret dan baseline belum ada. |
| 9 | Scope MVP | 10% | **4** | Semua fitur setara pentingnya. Tidak ada garis tegas "v0.1 = ini saja". Risiko terbesar: overbuild. |

**Skor tertimbang: 6.0 / 10**

### Verdict

> **Belum layak full build. Layak untuk mulai Fase 0 (simulasi) dan spike storage.**

Fondasi konseptualnya (storage, prinsip, model activation) sudah solid — bagian yang paling sulit diubah belakangan sudah benar. Yang kurang adalah bagian *produk*: kontrak integrasi harness, granularity node, dan batas MVP. Ketiganya bisa diselesaikan di atas kertas tanpa menulis kode produksi. Target sebelum build: **≥ 7.5**.

---

## 2. Yang harus ditambahkan untuk naik ke ≥ 7.5

| Prioritas | Tambahan | Menaikkan dimensi |
|---|---|---|
| P0 | **Definisi MVP** (usulan di §4) | 9 |
| P0 | **Kontrak integrasi**: 5–6 tool MCP dengan skema, tabel hook per harness, sumber transcript per harness (Claude Code `~/.claude/projects/*.jsonl`, Codex, OpenCode DB, dll) | 6 |
| P0 | **Granularity node**: usulan = *satu klaim/preferensi per node* untuk `fact/preference/decision`, *satu halaman* untuk `project/concept/person` (hub yang merangkum) | 4 |
| P1 | **Co-occurrence yang lebih tajam**: co-use hanya untuk node yang dipakai dalam *jendela tugas yang sama* (mis. dalam N turn), dinormalisasi NPMI, dengan homeostasis per node (pola Ori-Mnemos) | 3 |
| P1 | **Konflik edit file node**: node file hanya ditulis oleh konsolidasi; konflik git diselesaikan dengan "versi dengan `recorded_at` terbaru menang + versi kalah disimpan sebagai `retracted`" | 2 |
| P1 | **Eval konkret**: dataset dari sesi nyata user + subset publik (STALE untuk fakta basi, ForgetEval untuk forgetting, LongMemEval/LoCoMo hanya sebagai sanity check) dan **Ori-Mnemos sebagai baseline** | 8 |
| P2 | Threat model 1 halaman | 7 |
| P2 | Kontrak tiap langkah konsolidasi + idempotensi (key = session id + hash transcript) | 5 |

---

## 3. Proyek serupa yang bisa dipelajari

### 3.1 Ori-Mnemos — paling mirip, wajib dipelajari
[github.com/aayoawoyemi/Ori-Mnemos](https://github.com/aayoawoyemi/Ori-Mnemos) · Apache-2.0 · ~330 stars · v0.7

**Kemiripan:** markdown sebagai source of truth, wikilink sebagai edge, index SQLite turunan, decay ACT-R, Hebbian co-occurrence, spreading activation (Personalized PageRank), MCP server, hook lifecycle untuk Claude Code, Hermes, dan OpenCode; MCP-only untuk Cursor/Codex. Pada dasarnya ~70% dari desain kita sudah ada di sini.

**Yang layak diadopsi:**
- **Normalisasi NPMI** untuk co-occurrence + **homeostasis per node** (Turrigiano) agar hub tidak menyerap semua bobot.
- **Bootstrap edge hari-0 dari wikilink** yang sudah ada — mengatasi cold start.
- **Tarjan (articulation points)**: node yang secara struktural penting (jika dihapus graph terputus) dilindungi dari arsip.
- **Multiplier decay per ruang**: `self/` 0.1×, `notes/` 1×, `ops/` 3× — padanan dari pengecualian decay per tipe kita, tapi lebih halus.
- **Sinyal hasil sesi**: forward citation (+1.0), re-recall (+0.4) — padanan dari tabel §3.4 kita.
- **Capture → promote** (`ori add` → `ori promote`) — sama dengan inbox kita.

**Pelajaran dari kesalahan mereka:**
- State yang dipelajari (Q-values, bandit) **tidak bisa dibangun ulang** dari file — penulis sendiri mengoreksi klaim bahwa `.ori/` disposable. Prinsip P1 kita (event log sebagai sumber) menghindari masalah ini → **ini pembeda utama rewire**.
- 3 tool MCP eksplorasi **dihapus setelah 6 bulan karena tidak pernah dipakai**. Jaga permukaan tool tetap kecil.
- Retrieval multi-hop dan temporal lemah (LoCoMo recall 0.53 / 0.57); benchmark self-reported dan pernah dikoreksi.

**Implikasi strategis:** pertanyaan *build vs adopt* harus dijawab. Usulan: jangan fork, tapi jadikan Ori **baseline eval**. rewire baru layak ada jika menang di: portabilitas multi-device (event sourcing), validitas temporal (bitemporal + supersede), dan ontology bertipe.

### 3.2 FERNme — disiplin LLM-free dan eval yang jujur
[github.com/mirkofr/FERNme](https://github.com/mirkofr/FERNme) · Apache-2.0 · research preview v0.4

- **Mode `pure`: nol pemanggilan LLM untuk write dan recall.** Enrichment LLM hanya *mengusulkan* ke antrean review manusia. → Adopsi: activation/Hebbian/tier dihitung tanpa LLM; LLM hanya di ekstraksi konsolidasi.
- Append-only event log (Cabinet) — validasi P1.
- **Eval retensi**: fakta permanen tetap di atas ambang pada hari ke-700, fakta volatil memudar cepat — persis metrik yang kita butuhkan untuk membuktikan pengecualian decay.
- Temuan jujur: pada *abrupt drift*, **recency murni mengalahkan** model mereka (1.000 vs 0.833). Masukkan skenario ini ke simulasi Fase 0.

### 3.3 AIAgentLocalMemory — formula konkret & sync via op-log
[github.com/jackieju/AIAgentLocalMemory](https://github.com/jackieju/AIAgentLocalMemory) · plugin OpenCode · tahap awal (1 star)

- Hebbian `Δw = η(1 − w)`, decay `w ← w·exp(−λΔt)`, pruning edge lemah; skor `w_fts·BM25 + w_act·spread`.
- **Git sync hanya mengirim `operations.jsonl`**, bukan `graph.db` — implementasi nyata dari pola event sourcing kita.
- **Working memory** (antrean LRU-frequency, skor `freq·exp(−0.01·jam)`) — konsep yang belum ada di desain kita; berguna untuk memory jangka sangat pendek *dalam* sesi.

### 3.4 Riset yang relevan

| Paper | Ide yang bisa diambil | Catatan |
|---|---|---|
| **SYNAPSE** ([arXiv 2601.02744](https://arxiv.org/abs/2601.02744), Findings ACL 2026) | Spreading activation dengan **lateral inhibition** (node teraktif menekan pesaing) + fan effect + sigmoid, stabil dalam 3 iterasi. Lebih baik di multi-hop & temporal (LoCoMo). | Upgrade langsung untuk §3.3 desain kita. Kode belum dirilis per pencarian. |
| **HeLa-Mem** ([arXiv 2604.16839](https://arxiv.org/abs/2604.16839)) | **Hebbian distillation**: hub episodic yang padat diubah menjadi pengetahuan semantic. | Cocok untuk fase "tidur". Self-reported. |
| **RD-Forget** ([arXiv 2609.10263](https://arxiv.org/abs/2609.10263)) | Lupa = *pandangan saat query*, bukan penghapusan; arsip sumber tetap utuh. | Validasi P3/P5. |
| **ScrubJay-MEM** ([arXiv 2608.04746](https://arxiv.org/abs/2608.04746)) | Decay per tipe memory (perishability). | Gain menyempit di model kuat dan **berbalik** pada tugas konsolidasi fakta — peringatan untuk decay. |
| **Filesystem-Based Memory** ([arXiv 2607.26637](https://arxiv.org/abs/2607.26637)) | Memory markdown dalam pohon direktori: organisasi menghemat biaya pencarian, **bukan akurasi**, dan **terkikis** saat store membesar. | Lint/tidur wajib, bukan opsional. |
| **MemDelta** ([arXiv 2606.29914](https://arxiv.org/abs/2606.29914)) | Pilihan embedding model bisa menggelembungkan klaim arsitektur. | Ablation harus mengunci embedder. |
| **Letta sleep-time compute** | Agent terpisah di background yang menulis ulang memory block saat agent utama idle. | Padanan "tidur" kita yang sudah produksi. |

**Benchmark untuk eval:** STALE (apakah agent sadar memory sudah tidak berlaku; model frontier terbaik 55.2%), ForgetEval (forgetting, MIT), MemoryAgentBench (ICLR 2026, empat kompetensi). Catatan: audit independen menemukan judge LoCoMo menerima hingga 63% jawaban yang sengaja salah — jangan jadikan LoCoMo metrik utama.

> Sebagian besar angka di bagian ini **self-reported** oleh pembuatnya. Paper di tabel 3.4 (kecuali SYNAPSE) dirangkum dari [awesome-agent-memory](https://github.com/shimo4228/awesome-agent-memory) dan belum saya baca langsung.

---

## 4. Usulan MVP (v0.1)

Masuk:
- Format brain: `nodes/`, `events/`, `sessions/`, `brain.yaml`; index SQLite FTS5 yang bisa di-rebuild.
- Activation ACT-R + pengecualian decay per tipe; **Hebbian ditunda** sampai simulasi membuktikan manfaatnya.
- CLI: `prime`, `recall`, `get`, `remember`, `consolidate`, `rebuild`.
- MCP: `recall`, `get`, `remember` (3 tool saja).
- Konsolidasi manual (`rewire consolidate <session>`) dengan LLM, keluaran ke inbox + review.
- 1 harness dengan hook (Claude Code) + 1 tanpa hook (AGENTS.md + CLI).

Ditunda: embedding, Hebbian/associative edge, tidur otomatis, `sources/` ingest, push hook PreToolUse, multi-device, Obsidian.

## 5. Langkah berikut yang disarankan

1. Tambahkan ke desain: MVP (§4), kontrak integrasi, keputusan granularity, co-occurrence NPMI → skor diperkirakan naik ke ~7.5.
2. Jalankan **Fase 0 simulasi**, termasuk skenario abrupt drift (FERNme) dan hub (Ori), dengan ablation activation on/off.
3. Pasang Ori-Mnemos di satu proyek nyata selama ±1 minggu sebagai baseline pengalaman pengguna.
