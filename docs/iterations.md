# Log iterasi MVP

Setiap iterasi: apa yang diuji, dengan data apa, apa yang ditemukan, apa yang diubah.

## Iterasi 1 — 2026-10-10: build MVP v0.1 + uji nyata pertama

**Dibangun:** M1–M4 dari [mvp-spec](design/mvp-spec.md): format brain, index FTS5, activation ACT-R, `prime/recall/get/remember`, MCP server (3 tool), parser Claude Code & Codex, konsolidasi (extract → judge → gate → scan → write → commit), inbox, `undo`, hook Claude Code, Agent Skill untuk Codex. 29 unit test.

**Uji nyata (bukan fixture):**

| Uji | Data | Hasil |
|---|---|---|
| Konsolidasi transcript nyata | Transcript sesi desain rewire ini (Claude Code, 6 giliran user, 131 giliran total, ~33 ribu karakter setelah filter) via `claude -p` | 1 panggilan LLM, ~10 detik. 5 klaim auto-apply, 1 ke inbox |
| Hook Claude Code sungguhan | `claude -p --settings hooks.json` di folder proyek lain | SessionStart menyuntikkan snapshot; agent menyebut preferensi dari memory. Scope proyek bekerja (preferensi proyek `rewire` tidak muncul di proyek lain) |
| Siklus penuh otomatis | Di sesi itu user berkata "untuk proyek ini saya selalu ingin test ditulis sebelum fitur" | SessionEnd → konsolidasi background → tersimpan sebagai `preference` dengan scope `project:proj`, tanpa intervensi |
| Proteksi rekursi | Sesi `claude -p` milik konsolidasi sendiri | Terdeteksi `internal`, tidak dikonsolidasi |

**Temuan → perubahan:**

| Temuan | Perubahan |
|---|---|
| Klaim ditulis dalam bahasa Inggris padahal user menulis dalam Bahasa Indonesia | Prompt ekstraksi: klaim ditulis dalam bahasa user. Diuji ulang: semua klaim berbahasa Indonesia |
| LLM membuat node `project` sendiri → duplikat hub proyek | Tipe `project` dikeluarkan dari skema ekstraksi; deskripsi proyek menjadi `fact` ber-scope proyek |
| Hub proyek kosong ("Project rewire.") memakan budget snapshot | Hub dibuat tanpa body dan tidak ditampilkan selama kosong |
| Hook bisa terlewat jika konsolidasi lain sedang berjalan (lock) | Proses yang memegang lock memeriksa ulang antrean sebelum selesai (+ test) |
| Self-review: baris snapshot yang dipotong kehilangan ID; `remember` > 600 karakter gagal saat approve; `review approve` tanpa argumen meng-approve semua | Diperbaiki (+ test) |

**Kualitas ekstraksi (penilaian manual atas 1 sesi):** 6 dari 6 klaim relevan dan benar. 1 klaim tidak atomik (menggabungkan dua preferensi), tapi tertahan di inbox karena `inferred`. Satu sesi belum cukup untuk metrik E1 (target: 20 transcript, presisi ≥ 0.85).

## Iterasi 2 — 2026-10-11: dipasang di Codespace, sinkronisasi, performa

**Pemasangan nyata:** Codespace user, lewat sesi `claude remote-control` yang diinstruksikan dari sesi cloud. 29 test lulus di Python 3.14, hook + MCP terdaftar, `rewire doctor` bersih. Sesi Codespace menemukan bug: menjalankan ulang `install claude-code --write` menimpa backup settings asli → diperbaiki (backup pertama tidak pernah ditimpa) + test.

**Sinkronisasi antar mesin:** `rewire remote set`, `rewire init --from`, `rewire sync`, pull otomatis di awal sesi (budget 8 detik), push otomatis setelah konsolidasi background, clone otomatis dari `REWIRE_REMOTE` di mesin yang ephemeral. Event per device tidak pernah konflik; node yang konflik → versi terbaru menang, versi lain ke inbox. Nama device sekarang lokal per mesin (sebelumnya ikut tersinkron di `brain.toml` → dua mesin bisa menulis ke folder event yang sama). Diuji dengan dua brain + remote bare (5 test) dan E2E nyata (konsolidasi `claude -p` → push → clone di device kedua → snapshot muncul).

**Performa:** hook awal sesi 1.240 → 168 ms dan recall MCP 908 → 34 ms pada 10.000 memory, lewat index inkremental. Detail di [experiments/perf](../experiments/perf/README.md).

## Iterasi 3 — 2026-10-11: brain yang bisa dibaca manusia + graph

Pemicu: perbandingan dengan Obsidian dan graph DB. Fondasinya (file sebagai sumber, index turunan) dipertahankan, tapi tiga kekurangan diperbaiki.

| Kekurangan | Perubahan |
|---|---|
| Dibuka di Obsidian, graph view kosong; relasi berupa ID panjang | Relasi ditulis sebagai properti `[[wikilink]]` (Obsidian) **dan** link markdown relatif di bagian `## Related` (bisa diklik di GitHub). Ditambah `status`, `tags`, `aliases`; `source` jadi teks biasa. Format lama tetap terbaca; `rewire upgrade` menulis ulang |
| Tidak ada tempat untuk "melihat isi kepala" | `README.md` di root brain, dibuat ulang otomatis: aturan, preferensi, proyek beserta memory-nya, yang baru dipelajari, sesi terakhir, dan inbox. Deterministik: tidak berubah kalau memory tidak berubah |
| Tabel relasi ada tapi tidak dipakai recall | Spreading activation 2 langkah dari hasil pencarian teks, dengan bobot per relasi dan fan effect; hasil menyertakan `via` (alasan memory itu ikut muncul). Hub proyek tidak membanjiri hasil |
| Konsolidasi hanya membuat relasi `about` | LLM boleh menautkan klaim ke memory yang ada atau ke klaim lain di sesi yang sama (`part_of`, `depends_on`, `related_to`) |

Validasi: frontmatter diparse dengan PyYAML (parser YAML yang standar), semua link di README menunjuk file yang ada, 43 test lulus, konsolidasi nyata menghasilkan klaim berbahasa Indonesia + relasi + beranda, dan recall MCP tetap 16 ms pada 10.000 memory dengan 20.000 relasi.

## Iterasi 4 — 2026-10-11: mengukur kualitas retrieval

Eval baru ([experiments/retrieval](../experiments/retrieval/README.md)): 70 memory + 60 pertanyaan dibuat LLM terpisah, 6 kategori. Keseluruhan R@5 77% → **100%**, MRR 0.63 → **0.91**, lewat stopword + stemming EN/ID, fakta basi yang mengarahkan ke penggantinya, kata kunci dwibahasa dari konsolidasi, dan panduan query untuk agent. Temuan penting: angka parafrase awal (70%) semu. Hasil akhir optimis karena satu keluarga model membuat data dan query; validasi berikutnya harus pakai data user.

## Iterasi 5 — 2026-10-11: LoCoMo, dibandingkan dengan Mem0 dan Zep

Benchmark publik end-to-end ([experiments/locomo](../experiments/locomo/README.md)) dengan **prompt jawaban dan judge resmi Mem0**, 389 pertanyaan dari 3 percakapan. J: **69.7% → 91.3% → 96.7%** (Mem0 platform mempublikasikan 92.5% dengan judge yang sama). Tiga perubahan, masing-masing diambil dari cara kerja pesaing: klaim diberi tanggal kejadian (temporal 38.5 → 93.8%), profil ekstraksi `general` untuk second brain personal, dan episode (transcript yang bisa dicari, secret disamarkan). Batasan: subset, judge longgar dan sekeluarga dengan model penjawab, Mem0/Zep tidak dijalankan ulang.

## Iterasi 6 — 2026-10-12: insight pesaing + LoCoMo penuh

Diimplementasikan dari analisis Mem0/Zep: semantic search opsional (WordLlama, offline, fusi aditif dengan BM25), bonus jendela waktu ("in July 2023"), dan pertanyaan sejarah mengembalikan fakta `superseded` berlabel. LoCoMo **penuh** (10 percakapan, 1.540 pertanyaan, judge resmi Mem0): **94.2%** hybrid, **94.9%** kata kunci saja, vs 92.5% Mem0 platform. Ablasi: semantik tidak membantu di LoCoMo (17 vs 29 pertanyaan berbeda, tidak signifikan) karena episode + kata kunci sudah menutupinya, jadi tetap opsional. Sisa kelemahan: open-domain 75% — kebanyakan pertanyaan spekulatif yang butuh pengetahuan dunia, bukan memory. Bug ditemukan run penuh: `time_window` crash pada "2022-23".

## Berikutnya (iterasi 7)

1. **E1 penuh:** konsolidasi 20 transcript nyata (Claude Code + Codex) milik user, label manual, ukur presisi & duplikat.
2. **Verifikasi parser Codex** dengan file `~/.codex/sessions/**/rollout-*.jsonl` asli (fixture saat ini dibuat dari dokumentasi pihak ketiga).
3. **Dogfooding E4:** pasang di mesin user selama 2 minggu; pantau hit rate snapshot (dari `used_memory_ids` di `sessions/*.md`) dan jumlah item inbox per sesi.
4. Pertimbangkan pemisahan klaim gabungan secara otomatis saat judge.
