# rewire di sesi cloud Claude Code (claude.ai/code, aplikasi Claude, `claude --cloud`)

Sesi cloud selalu mulai dari VM baru. Yang terpasang di komputer sendiri (`~/.claude/settings.json`,
MCP `--scope user`, `~/brain`) tidak ikut. Panduan ini membuat rewire — MCP server, hooks, dan memorinya —
aktif otomatis di **setiap** sesi cloud, cukup disiapkan sekali.

## Cara kerjanya

| Bagian | Di mana disimpan | Kapan berjalan |
| --- | --- | --- |
| Binary `rewire` | **Setup script** environment cloud | Sekali; hasilnya di-cache (±7 hari) untuk sesi berikutnya |
| Hooks + aturan izin | `.claude/settings.json` di repo kerja | Setiap sesi (repo ikut di-clone) |
| MCP server `rewire` | `.mcp.json` di repo kerja | Setiap sesi |
| Brain (memori) | Branch `rewire-brain` di **repo kerja yang sama** | Di-clone saat sesi mulai, di-pull, dan di-push setelah konsolidasi |

Kenapa brain di branch repo kerja, bukan di repo brain terpisah? Sesi cloud hanya bisa menjangkau repo
yang terpasang (attached) di sesi itu. Kalau memasang dua repo, Claude Code tidak memuat hooks dan
`.mcp.json` dari repo mana pun. Branch di repo yang sama selalu terjangkau dan bisa di-push.

Semua file ada di `integrations/claude-code-cloud/`:

- `environment-setup.sh` — isi untuk kolom Setup script.
- `hook.sh` — wrapper hooks dan MCP. Di komputer sendiri ia **tidak melakukan apa-apa** (kecuali menjalankan
  MCP seperti biasa), jadi hooks user-level dari `rewire install claude-code --write` tidak dobel.
- `install-into-repo.sh` — menyalin `hook.sh` dan menggabungkan konfigurasi ke repo kerja.

## Langkah 1 — Pastikan source rewire bisa diambil dari cloud

Repo `NewbeeTryToCode/rewire` saat ini **private**, dan setup script mengambil source-nya dengan git. Pilih salah satu:

- **(a) Jadikan repo rewire publik.** Paling sederhana; tidak perlu token.
- **(b) Tetap private**, lalu buat *fine-grained personal access token* di GitHub: hanya repo `rewire`,
  izin *Contents: Read-only*. Simpan sebagai variabel environment `REWIRE_GITHUB_TOKEN` (langkah 2).
  Catatan: variabel environment bisa dibaca siapa pun yang memakai environment itu, jadi pakai environment pribadi.
  Jalur (b) belum teruji dari setup script cloud; kalau gagal, pakai (a).

## Langkah 2 — Siapkan environment cloud (sekali)

1. Buka [claude.ai/code](https://claude.ai/code), lalu buka pemilih environment dan edit environment yang kamu pakai
   (atau buat yang baru).
2. **Network access**: biarkan **Trusted** (sudah mengizinkan crates.io dan GitHub).
3. **Setup script**: tempel isi `integrations/claude-code-cloud/environment-setup.sh`.
4. **Environment variables** (opsional, format `.env`):
   ```
   REWIRE_GITHUB_TOKEN=github_pat_...      # hanya untuk pilihan (b)
   REWIRE_CLOUD_TRUST_BRAIN=1              # lihat "Aturan dan preferensi" di bawah
   ```
5. Simpan. Sesi pertama berikutnya menjalankan build (±3 menit), lalu di-cache.

Build dibuat tanpa LTO agar selesai di bawah batas cache ±5 menit (diuji: ±2m50d di VM cloud).
Kalau lebih dari 5 menit, environment tidak di-cache dan build akan berulang di tiap sesi.

## Langkah 3 — Pasang konfigurasi ke repo kerja (sekali per repo)

Di komputer sendiri, dengan clone repo rewire dan repo kerjamu:

```bash
bash ~/path/ke/rewire/integrations/claude-code-cloud/install-into-repo.sh ~/path/ke/repo-kerja
cd ~/path/ke/repo-kerja
git add .claude/rewire/hook.sh .claude/settings.json .mcp.json
git commit -m "Use rewire memory in Claude Code cloud sessions"
git push
```

Script ini menambahkan (tanpa menghapus isi yang sudah ada):

- hooks `SessionStart`, `UserPromptSubmit`, `Stop`, `PreCompact`, `SessionEnd` → `.claude/rewire/hook.sh`;
- aturan `permissions.deny` agar agen tidak bisa menyetujui memorinya sendiri, `forget`, `undo`,
  `remote set`, atau mengedit `/root/brain` langsung;
- MCP server `rewire` di `.mcp.json`.

Di komputer sendiri Claude Code akan menanyakan sekali apakah server MCP dari `.mcp.json` boleh dipakai; setujui saja.
Ia tetap menjalankan `rewire mcp` biasa.

## Langkah 4 — Hubungkan brain ke branch `rewire-brain` (sekali, dari komputer sendiri)

Jalankan di **terminal kamu sendiri**, bukan lewat agen (rewire menolak `remote set` di dalam sesi agen):

```bash
# belum punya brain: rewire init dulu (atau scripts/setup-local.sh)
git -C ~/brain branch -m rewire-brain                 # brain dipush ke branch ini
rewire remote set https://github.com/<kamu>/<repo-kerja>.git
rewire sync                                           # push pertama
```

`remote set` juga menyalakan auto-sync di `brain.toml`, yang ikut tersinkron ke cloud: pull saat sesi mulai,
push setelah konsolidasi.

Brain berisi memori pribadi: pastikan repo kerja itu **private**. Siapa pun yang punya akses tulis ke repo
juga bisa mengubah branch `rewire-brain`.

## Langkah 5 — Mulai sesi dan periksa

Buka sesi baru di claude.ai/code dengan repo kerja itu sebagai **satu-satunya** repo, lalu minta Claude:

```
jalankan: rewire doctor && rewire sessions | head && cat ~/.rewire-cloud/hook.log
```

Yang diharapkan:

- `hook.log` berisi `brain cloned from https://github.com/... (rewire-brain)`;
- di awal sesi ada blok `# rewire memory` (kalau brain sudah berisi catatan);
- tool MCP `recall`, `get`, `remember` tersedia (cek dengan `/mcp`).

## Aturan dan preferensi: mode aman vs. trust

rewire menahan *rules*, *preferences*, dan *skills* yang datang dari remote sampai disetujui **di perangkat itu**
(`rewire review approve`), dan persetujuan itu ditolak di dalam sesi agen. VM cloud selalu baru, jadi:

- **Default (aman)**: di cloud, fakta, keputusan, dan transkrip lama tetap bisa di-*recall*, tapi aturan dan
  preferensi **tidak** dimuat.
- **`REWIRE_CLOUD_TRUST_BRAIN=1`**: brain di-clone apa adanya, sehingga aturan dan preferensi langsung aktif.
  Pakai ini kalau hanya kamu yang bisa menulis ke repo kerja. Item yang masuk *lewat pull di tengah sesi* tetap menunggu review.

Usulan memori baru dari sesi cloud (`rewire remember`) masuk ke inbox dan tersinkron. Setujui dari komputer sendiri
dengan `rewire review`.

## Konsolidasi di cloud

VM cloud sering dibuang tanpa `SessionEnd`, padahal transkripnya ikut hilang. Karena itu hook `Stop` mengantrikan
transkrip untuk dikonsolidasi tiap `REWIRE_CLOUD_CONSOLIDATE_EVERY` detik (default 1800 = 30 menit), selain saat
`PreCompact` dan `SessionEnd`. Konsolidasi berjalan di latar dengan `claude -p` (login sesi sudah tersedia di VM;
diuji), lalu brain otomatis di-push ke `rewire-brain`.

Bagian sesi setelah konsolidasi terakhir bisa hilang kalau VM dibuang sebelum waktunya. Untuk menutup sesi
penting, minta Claude: `printf '{"transcript_path":"%s"}' <path> | rewire hook session-end`, atau cukup tunggu
`Stop` berikutnya.

## Variabel yang bisa diatur (Environment variables)

| Variabel | Default | Fungsi |
| --- | --- | --- |
| `REWIRE_BRAIN_BRANCH` | `rewire-brain` | Branch repo kerja yang berisi brain |
| `REWIRE_REMOTE` | – | Repo brain terpisah; harus terjangkau dari sesi |
| `REWIRE_CLOUD_TRUST_BRAIN` | `0` | `1` = aturan dan preferensi aktif tanpa review |
| `REWIRE_CLOUD_CONSOLIDATE_EVERY` | `1800` | Jeda konsolidasi dari hook `Stop` (detik) |
| `REWIRE_DEVICE` | `cloud` | Nama perangkat di log event |
| `REWIRE_GITHUB_TOKEN` | – | Token baca untuk repo rewire yang private (setup script) |
| `REWIRE_REF` | `main` | Branch/tag rewire yang di-build |

## Batasan

- Sesi dengan **beberapa repo** dan *thread* di dalam Project tidak memuat hooks dari `.claude/settings.json` repo,
  jadi rewire tidak aktif di sana. MCP-nya juga hanya dimuat di sesi satu repo.
- Satu brain per repo kerja. Kalau kamu memakai beberapa repo kerja di cloud, masing-masing butuh branch
  `rewire-brain` sendiri. Komputer lokal hanya bisa sinkron ke salah satunya.
- Push brain melewati GitHub proxy sesi: branch biasa diizinkan, tag dan penghapusan branch ditolak.

## Masalah umum

| Gejala | Penyebab / solusi |
| --- | --- |
| `rewire is not installed in this cloud environment` | Setup script belum dipasang, gagal (repo private tanpa token), atau melewati batas waktu. Cek log setup di awal sesi. |
| `no brain yet: push one to the 'rewire-brain' branch` | Langkah 4 belum dijalankan, atau nama branch berbeda (`REWIRE_BRAIN_BRANCH`). |
| `the brain could not be fetched` | Lihat `~/.rewire-cloud/hook.log`. Biasanya repo brain di luar jangkauan sesi. |
| Memori ada, tapi aturan/preferensi tidak muncul | Mode aman (default); set `REWIRE_CLOUD_TRUST_BRAIN=1`. |
| Tool MCP `rewire` tidak muncul | Sesi punya lebih dari satu repo, atau binary belum terpasang. |
