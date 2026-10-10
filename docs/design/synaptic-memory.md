# Design: Synaptic Memory untuk rewire

> Status: **draf untuk review** — belum ada implementasi.
> Lanjutan dari [riset memory approaches](../research/memory-approaches.md).

## 0. Tujuan & prinsip

1. **Seperti memory manusia**: memory yang sering dipakai makin kuat, yang jarang dipakai memudar (tapi tidak hilang), dan memory yang sering dipakai *bersama* makin terhubung (Hebbian: *neurons that fire together, wire together*).
2. **Update setelah sesi** (pola Hermes): sesi menulis cepat, lalu ada fase konsolidasi yang memperbarui brain.
3. **Struktur graph/ontology**: memory adalah node bertipe yang terhubung oleh relasi bertipe.
4. **Own the brain, transfer anywhere**: brain tetap berupa file milik user, bisa di-git, bisa dibaca manusia, tanpa wajib server.

Prinsip yang mengikat semua keputusan di bawah:

- **P1 — Simpan *kejadian*, bukan *bobot*.** Bobot (kekuatan synapse) selalu *dihitung* dari log kejadian. Ini keputusan desain terpenting di dokumen ini (lihat §4).
- **P2 — Salience ≠ kebenaran.** Seberapa sering dipakai (activation) terpisah dari apakah fakta masih berlaku (validity). Fakta yang sering dipakai bisa saja sudah basi.
- **P3 — Lupa = turun tier, bukan hapus.** Memory yang lemah keluar dari context, tapi masih bisa "teringat kembali".
- **P4 — File adalah sumber kebenaran; database hanyalah index** yang bisa dibangun ulang kapan saja.

---

## 1. Model mental: tiga jenis memory

| Jenis (analogi otak) | Di rewire | Contoh | Sifat |
|---|---|---|---|
| **Episodic** (hippocampus) | `sessions/` — log per sesi | "10 Okt: kita riset memory harness" | Append-only, mentah, murah ditulis |
| **Semantic** (neocortex) | `nodes/` — graph fakta/konsep | "User lebih suka desain matang sebelum coding" | Hasil konsolidasi, terkurasi |
| **Procedural** | `skills/` — SKILL.md | "Cara rilis proyek X" | Dipanggil saat relevan |

Konsolidasi = memindahkan dari episodic ke semantic, persis seperti tidur memindahkan memory dari hippocampus ke neocortex.

---

## 2. Graph & ontology

### 2.1 Node types (ontology inti — sengaja kecil)

| Type | Arti | Contoh |
|---|---|---|
| `person` | Orang (termasuk user sendiri) | `@me`, rekan kerja |
| `project` | Proyek / repo / inisiatif | `rewire` |
| `concept` | Ide, teknologi, istilah | "Hebbian learning", "SQLite" |
| `preference` | Cara user ingin bekerja | "jawab dalam Bahasa Indonesia" |
| `decision` | Keputusan + alasannya | "pakai file sebagai source of truth" |
| `fact` | Pernyataan faktual yang bisa berubah | "API X rate-limit 100/menit" |
| `procedure` | Pointer ke skill | `skills/release/` |
| `artifact` | Dokumen/URL/file eksternal | link dashboard, dokumen |

### 2.2 Relation types (kosakata tetap ~12)

`about`, `part_of`, `depends_on`, `decided_in`, `prefers`, `works_on`, `knows`, `uses`, `supersedes`, `contradicts`, `derived_from`, `related_to` (fallback).

**Aturan ontology:**
- Kosakata disimpan di `brain.yaml`. Konsolidasi **boleh mengusulkan** type/relasi baru, tapi masuk ke `inbox/` untuk disetujui — mencegah *ontology sprawl* (masalah klasik graph yang dibangun LLM: 300 jenis relasi yang artinya sama).
- Tidak memakai OWL/RDF penuh. Terlalu berat untuk kebutuhan ini; yang dibutuhkan hanya node bertipe + edge bertipe + beberapa constraint (mis. `supersedes` hanya antar node bertipe sama).

### 2.3 Dua kelas edge (penting)

| | **Semantic edge** | **Associative edge (synapse)** |
|---|---|---|
| Arti | Relasi bermakna: "rewire `uses` SQLite" | "Dua memory ini sering muncul bersama" |
| Dibuat oleh | Konsolidasi (LLM) atau user | **Otomatis** dari co-activation |
| Disimpan di | Frontmatter file node (dikurasi, di-git) | **Tidak disimpan** — dihitung dari event log |
| Bobot | Prior per tipe relasi + Hebbian | Murni Hebbian |

Ini meniru otak: ada pengetahuan terstruktur, dan ada asosiasi yang tumbuh sendiri dari pengalaman.

---

## 3. Model "synapse": matematika

Dasar: **ACT-R** (model kognitif Anderson) yang sudah divalidasi puluhan tahun terhadap data memory manusia, ditambah **Hebbian learning** untuk edge dan **spreading activation** untuk retrieval.

### 3.1 Kekuatan node — base-level activation

```
B(i) = ln( Σ_j  (now − t_j)^(−d) )        d ≈ 0.5
```

`t_j` = waktu setiap kali node *i* benar-benar dipakai. Efeknya:
- **Frekuensi** menaikkan nilai (makin banyak suku yang dijumlah).
- **Recency** menaikkan nilai (suku baru jauh lebih besar).
- **Decay** alami mengikuti power law — sesuai kurva lupa Ebbinghaus.
- **Spacing effect**: dipakai tersebar di banyak hari lebih awet daripada 10x dalam satu jam.

Contoh (waktu dalam jam):

| Pola pemakaian | Hitungan | B |
|---|---|---|
| Dipakai 1 jam, 1 hari, 7 hari lalu | ln(1 + 0.204 + 0.077) | **+0.25** |
| Dipakai sekali, 30 hari lalu | ln(720^−0.5) | **−3.29** |
| Dipakai 5x dalam 1 jam, 30 hari lalu | ln(5 · 0.037) | **−1.68** |

Untuk efisiensi, simpan *k* event terakhir secara persis + aproksimasi Petrov untuk sisanya: `ln(n/(1−d)) − d·ln(L)` (n = jumlah pemakaian, L = umur sejak pemakaian pertama).

### 3.2 Kekuatan synapse — Hebbian dengan decay

Setiap kali node *a* dan *b* dipakai dalam sesi yang sama → event `co_use(a,b)`. Kekuatan edge dihitung dengan rumus yang sama seperti 3.1 terhadap event co-use, lalu dinormalisasi ke 0..1:

```
H(a,b)   = σ( ln Σ_j (now − t_j)^(−d) )       # σ = sigmoid
W(a,b)   = prior(rel) + (1 − prior(rel)) · H(a,b)   # untuk semantic edge
W(a,b)   = H(a,b)                                  # untuk associative edge
```

Associative edge dengan `W < θ_edge` dianggap tidak ada (pruning alami).

### 3.3 Retrieval — spreading activation

Saat agent butuh memory (query/konteks sesi):

1. **Seed**: cari node yang cocok secara teks (FTS5) dan opsional embedding → skor relevansi `R(i)`.
2. **Spread** maksimal 2 hop dari seed:
   ```
   A(j) = B(j) + Σ_i  R(i) · W(i,j) · (S − ln fan(i))
   ```
   `fan(i)` = jumlah edge node *i*. **Fan effect** ACT-R: node "hub" yang terhubung ke segalanya menyebarkan aktivasi lebih lemah — mencegah node seperti "@me" atau "coding" mendominasi.
3. **Skor akhir**:
   ```
   score(j) = α·R(j) + β·norm(A(j)) + γ·importance(j)
   ```
   dengan filter: node `superseded`/`valid_to < now` dibuang kecuali query bersifat historis.
4. Ambil top-k sampai **budget token** habis; kembalikan ringkasan + ID (pola progresif claude-mem: `search → get`).

Pendekatan ini dekat dengan **HippoRAG** (Personalized PageRank di atas knowledge graph, terinspirasi hippocampus) dan skor *recency × importance × relevance* dari paper **Generative Agents** — keduanya terbukti di riset.

### 3.4 Apa yang dihitung sebagai "dipakai"? (bagian paling rawan)

Kalau definisi "dipakai" salah, sistem masuk **rich-get-richer loop**: memory di snapshot selalu terlihat → dianggap dipakai → makin kuat → selalu di snapshot.

| Sinyal | Bobot event | Catatan |
|---|---|---|
| Ada di snapshot awal sesi | **0** | Hanya terekspos, belum tentu dipakai |
| Muncul di hasil `search` | **0** | Hanya kandidat |
| Agent memanggil `get(id)` | 0.5 | Ada niat memakai |
| Konsolidasi menilai memory ini memengaruhi jawaban/aksi | **1.0** | Sinyal utama |
| User mengonfirmasi ("ya, betul") | 2.0 + `confidence: confirmed` | |
| User mengoreksi / bertentangan | event `contradict` | Tidak menaikkan B; memicu review validity |

Event berbobot → suku di §3.1 dikalikan bobotnya.

Penjaga tambahan: batas maksimal event per node per sesi (mis. 1), dan sedikit *exploration* (sesekali memasukkan node dormant yang relevan ke snapshot).

### 3.5 Tier (lupa yang manusiawi)

| Tier | Kondisi | Perilaku |
|---|---|---|
| **core** | top-N activation dalam budget token | Masuk snapshot awal sesi (beku) |
| **active** | B ≥ θ_active | Ikut spreading activation |
| **dormant** | B < θ_active | Tidak ikut spreading; hanya muncul lewat pencarian eksplisit |
| **archived** | dormant > X bulan atau superseded | Hanya untuk query historis |

Memory dormant yang dipakai lagi langsung naik tier — "teringat kembali".

Selain activation, setiap node punya `importance` (1–5) dari konsolidasi/user. Memory penting yang jarang dipakai (mis. alergi, password policy) tidak boleh hilang hanya karena jarang muncul: `importance ≥ 4` → tidak pernah turun di bawah `active`.

---

## 4. Penyimpanan (jawaban atas kekhawatiran utama)

### 4.1 Kenapa jangan simpan bobot di file/database utama

Kalau setiap pemakaian meng-update `weight: 0.73` di file:
- Setiap sesi mengubah puluhan file → **git diff berisik**, riwayat tidak berguna.
- Dua mesin/harness memakai brain bersamaan → **merge conflict** di angka yang tidak bermakna bagi manusia.
- Parameter (`d`, `θ`) tidak bisa di-tuning ulang tanpa kehilangan data.

Kalau yang disimpan adalah **event** (`use`, `co_use`, `confirm`, `contradict`) di log append-only:
- Merge antar device = **gabungkan log** (union). Tidak pernah konflik.
- Bobot bisa dihitung ulang kapan saja dengan parameter baru — bisa eksperimen dan *replay* sejarah.
- Ini event sourcing, pola yang sudah matang.

### 4.2 Layout

```
~/brain/                              # repo git = brain
├── brain.yaml                        # versi skema, ontology, parameter (d, θ, budget)
├── nodes/                            # SEMANTIC — 1 node = 1 file markdown
│   ├── preference/
│   │   └── design-before-build.md
│   ├── project/rewire.md
│   └── concept/hebbian-learning.md
├── events/                           # SYNAPSE — log append-only, 1 file per device per bulan
│   └── <device-id>/2026-10.jsonl     #   → tidak pernah konflik saat merge
├── sessions/                         # EPISODIC — ringkasan per sesi
│   └── 2026-10-10/<session-id>.md
├── inbox/                            # kandidat yang menunggu persetujuan
├── skills/<name>/SKILL.md            # PROCEDURAL
├── snapshot/CORE.md                  # hasil generate, yang diinject ke harness
└── .index/brain.db                   # SQLite, gitignored, bisa dibangun ulang
```

**File node:**

```markdown
---
id: 01JA8Z3K7Q...            # ULID, stabil walau file di-rename
type: preference
title: Desain matang dulu sebelum coding
aliases: [design-first]
importance: 4
confidence: confirmed        # confirmed | inferred
valid_from: 2026-10-10
valid_to: null
source: { harness: claude-code, session: 01JA8Z... }
edges:
  - { rel: about, to: 01JA8Z...rewire }
  - { rel: derived_from, to: 01JA8Y...session }
---
User ingin memvalidasi desain (terutama storage) sebelum implementasi dimulai.
```

**Event (JSONL):**

```json
{"t":"2026-10-10T15:02:11Z","ev":"use","node":"01JA8Z3K7Q","w":1.0,"session":"01JA8Z...","harness":"claude-code"}
{"t":"2026-10-10T15:02:11Z","ev":"co_use","a":"01JA8Z3K7Q","b":"01JA8Z9RWX","session":"01JA8Z..."}
{"t":"2026-10-10T15:40:00Z","ev":"contradict","node":"01JA7...","session":"01JA8Z..."}
```

### 4.3 Index (derived): SQLite

```
nodes(id, type, title, body, importance, valid_from, valid_to, tier, path)
edges(src, dst, rel, prior)              -- dari frontmatter
events(t, ev, node, a, b, w, session)    -- dari events/*.jsonl
activation(node, B, updated_at)          -- cache, dihitung ulang
assoc(a, b, H, updated_at)               -- cache associative edge
nodes_fts  (FTS5)
nodes_vec  (opsional, sqlite-vec)
```

Traversal 2 hop cukup dengan recursive CTE di SQLite — graph personal (ribuan sampai puluhan ribu node) jauh di bawah titik di mana graph DB khusus diperlukan.

### 4.4 Perbandingan opsi storage

| Opsi | Pro | Kontra | Verdict |
|---|---|---|---|
| **Markdown + JSONL + SQLite index** | Portabel, bisa di-git, bisa dibaca manusia, nol server, bisa rebuild | Query graph kompleks harus ditulis sendiri | ✅ **Rekomendasi** |
| Graph DB server (Neo4j, Memgraph) | Cypher, algoritma graph siap pakai | Wajib server, brain terkunci di DB, sulit di-git | ❌ bertentangan dengan "own the brain" |
| Embedded graph DB (Kuzu, dll) | Cypher tanpa server | Format biner, risiko maintenance proyek, tetap tidak bisa di-diff | ⚠️ bisa jadi backend index opsional |
| RDF triple store (Oxigraph) + OWL | Ontology formal, SPARQL | Kompleks, berlebihan untuk graph personal | ❌ untuk v1 |
| Hanya Markdown + wikilink (gaya Obsidian) | Sangat sederhana | Tidak ada tempat untuk bobot/event | ⚠️ kurang untuk konsep synapse |

Karena index adalah turunan, **backend bisa diganti nanti** (mis. ekspor ke Neo4j untuk visualisasi) tanpa memindahkan brain.

---

## 5. Siklus hidup sesi (pola Hermes, diperluas)

```
SessionStart ──► generate snapshot/CORE.md (tier core + spreading dari node project aktif)
                 → diinject & DIBEKUKAN (jaga prefix cache)
     │
During ────────► recall(query)   → search + spread → ringkasan + ID   (tidak menulis event)
                 get(id)          → detail node                        (event use w=0.5)
                 remember(text)   → tulis ke sessions/ + inbox/        (cepat, tanpa gate)
     │
PreCompact ────► flush: minta agent menyimpan hal penting sebelum konteks dipadatkan
     │
SessionEnd ────► KONSOLIDASI SESI (async, tidak memblok user)
                 1. Ringkas transcript → sessions/<tgl>/<id>.md
                 2. Ekstrak kandidat: entitas, fakta, preferensi, keputusan
                 3. Entity resolution: cocokkan ke node yang ada (FTS + embedding + LLM judge)
                 4. Tentukan aksi per kandidat: CREATE | UPDATE | SUPERSEDE | CONTRADICT | IGNORE
                 5. Nilai node mana yang benar-benar dipakai → event use/co_use
                 6. Security scan (injection, exfiltration, unicode tersembunyi)
                 7. Tulis file + event, git commit "session <id>"
     │
Periodik ──────► "TIDUR" (mis. harian)
                 - hitung ulang activation & tier
                 - gabung duplikat, pangkas associative edge lemah
                 - ringkas ulang node hub yang terlalu panjang
                 - usulan ontology baru → inbox/
                 - laporan perubahan → sessions/dreams/<tgl>.md (untuk direview manusia)
```

Aturan aman: aksi destruktif (SUPERSEDE, merge node, ubah `importance` ≥ 4) dari sumber `inferred` masuk `inbox/` dulu; dari sumber `confirmed` boleh langsung.

---

## 6. Risiko & mitigasi

| Risiko | Dampak | Mitigasi |
|---|---|---|
| **Rich-get-richer** | Snapshot membeku pada memory yang sama | Eksposur = 0 (§3.4), cap per sesi, exploration |
| **Duplikat node** (entity resolution gagal) | Activation terpecah, graph kotor | `aliases`, resolusi bertahap, merge saat "tidur" (event lama dipindah ke node pemenang) |
| **Ekstraksi LLM salah** | Fakta keliru jadi permanen | `confidence: inferred` vs `confirmed`, inbox, riwayat git untuk revert |
| **Fakta basi tapi sering dipakai** | Agent yakin pada info lama | P2: validity terpisah; `contradict` → review; `valid_to` |
| **Ontology sprawl** | Relasi tidak konsisten | Kosakata tetap + approval |
| **Hub node mendominasi** | Retrieval tidak relevan | Fan effect (§3.3) |
| **Prompt injection persisten** | Memory jahat masuk setiap sesi | Scan, sumber untrusted tak dipromosikan otomatis, `<private>` |
| **Event log membengkak** | Index lambat | Kompaksi bulanan: event > N bulan diringkas jadi (n, L) per node (aproksimasi Petrov) |
| **Biaya konsolidasi** | Mahal per sesi | Async, model kecil, skip sesi yang trivial |
| **Multi-device** | Konflik | Event per-device (union), node file jarang berubah, ULID |

---

## 7. Cara memvalidasi desain sebelum dibangun penuh

Urutan yang disarankan (murah → mahal):

1. **Fase 0 — Simulasi matematika** (tanpa LLM, tanpa harness). Script yang membuat event log sintetis (memory sering dipakai, jarang, berpola spaced, hub node, fakta yang di-supersede) dan memplot B, H, tier dari waktu ke waktu. Tujuan: memilih `d`, `θ`, bobot `α/β/γ`, dan membuktikan tidak ada rich-get-richer. **Ini cara tercepat membuktikan desain solid.**
2. **Fase 1 — Storage + index + CLI**: format file, event log, rebuild SQLite, `recall/get/remember`, generate `CORE.md`. Uji: hapus `.index/` → rebuild → hasil identik.
3. **Fase 2 — Konsolidasi**: pipeline SessionEnd dengan replay transcript nyata. Metrik: tingkat duplikat, presisi ekstraksi, berapa kandidat yang masuk inbox.
4. **Fase 3 — Adapter harness**: Claude Code (hooks + MCP), Codex, satu harness tanpa hook (AGENTS.md + CLI).

**Metrik keberhasilan** (dibuat sebagai eval set): recall@k fakta yang dibutuhkan suatu tugas, token snapshot, tingkat fakta basi yang diambil, tingkat duplikat, stabilitas snapshot antar sesi.

---

## 8. Keputusan terbuka (perlu jawaban user)

1. **Event "dipakai" ditentukan siapa?** Usulan: kombinasi tool call `get` + penilaian LLM saat konsolidasi. Alternatif lebih murah: hanya tool call (tapi bias ke agent yang rajin memanggil tool).
2. **Embedding di v1?** Usulan: tidak; FTS5 + spreading activation dulu, embedding sebagai plugin untuk entity resolution.
3. **Seberapa otomatis konsolidasi?** Full auto, auto + inbox untuk aksi destruktif (usulan), atau semua lewat review.
4. **Scope brain**: satu brain global dengan node `project`, atau brain per konteks (kerja/pribadi) yang bisa di-mount bersama?
5. **Bahasa implementasi** untuk CLI/MCP.

---

## Referensi konsep

- ACT-R base-level learning & fan effect — Anderson & Schooler (1991), *Reflections of the environment in memory*; Anderson (2007) *How Can the Human Mind Occur in the Physical Universe?*
- Aproksimasi base-level — Petrov (2006), *Computationally efficient approximation of the base-level learning equation in ACT-R*
- Hebbian learning — Hebb (1949), *The Organization of Behavior*
- Generative Agents (recency × importance × relevance, reflection) — Park et al. (2023)
- HippoRAG (graph + Personalized PageRank, terinspirasi hippocampus) — Gutiérrez et al. (2024)
- A-MEM (memory bergaya Zettelkasten yang saling menautkan & berevolusi) — Xu et al. (2025)
- Graphiti / Zep — temporal knowledge graph dengan invalidasi fakta
- Hermes (konsolidasi setelah sesi, snapshot beku), OpenClaw (dreaming dengan gate) — lihat dokumen riset
