# Riset: Pendekatan Memory di Agent Harness

> Tujuan rewire: **own the brain, transfer it anywhere** — satu "second brain" milik user yang bisa di-attach ke harness apa pun (Claude Code, Codex, Gemini CLI, OpenCode, Cursor, Hermes, OpenClaw, dst).
>
> Sumber utama: [best-of-agent-harnesses](https://github.com/ryanalberts/best-of-agent-harnesses) (section *Memory and state*, `comparisons/memory-layers.md`, `comparisons/progressive-disclosure.md`, dan rating `context_memory` di `attributes/*.json`), lalu diverifikasi ke dokumentasi masing-masing proyek. Diriset 2026-10-10.

---

## 1. Peta besar: 4 bentuk memory

List tersebut membagi memory jadi beberapa "shape". Pertanyaan kuncinya: **siapa yang memutuskan apa yang diingat?**

| Bentuk | Contoh | Siapa yang memutuskan | Portabel antar harness? |
|---|---|---|---|
| **File instruksi** (ditulis manusia) | AGENTS.md, CLAUDE.md, GEMINI.md | Manusia | ✅ AGENTS.md dibaca 20+ tool |
| **File memory yang ditulis agent** | Claude Code auto memory, Hermes `MEMORY.md`/`USER.md`, OpenClaw `MEMORY.md` + `memory/YYYY-MM-DD.md`, Qwen Code auto-memory | Agent (lewat tool tulis file) | ⚠️ formatnya mirip, tapi lokasi & aturan beda per harness |
| **Plugin harness** (auto-capture) | claude-mem, context-mode, beads | Otomatis lewat hooks | ✅ claude-mem & beads sengaja multi-harness |
| **Memory layer / engine** (dipanggil lewat API/MCP) | Mem0/OpenMemory, Graphiti (Zep), cognee | Kode aplikasi | ✅ tapi butuh server + DB |
| **Runtime berbasis memory** | Letta (MemGPT) | Agent itu sendiri, sebagai bagian cara berpikirnya | ❌ agent harus hidup di dalam Letta (ada export `.af`) |

---

## 2. Bedah pendekatan per harness

### Claude Code — file instruksi + auto memory
- **CLAUDE.md** berlapis (org/managed → user `~/.claude/CLAUDE.md` → project → subdirektori), mendukung `@path` import (maks 4 hop), `.claude/rules/` yang bisa di-scope per path. Sejak v2.1.277 membaca **AGENTS.md** langsung.
- **Auto memory** di `~/.claude/projects/<project>/memory/`:
  - `MEMORY.md` = **index**, satu baris per memory; hanya **200 baris / 25KB pertama** yang dimuat tiap sesi.
  - Detail disimpan di **topic file** (`feedback_testing.md`, `user_role.md`) yang dibaca **on demand**.
  - Frontmatter punya `type`: `user`, `feedback`, `project`, `reference`; harness menambahkan timestamp `modified`.
  - Aturan penting: **jangan simpan yang bisa diturunkan dari kode/git**, dan jangan duplikasi CLAUDE.md.
  - Kelemahan untuk visi kita: **machine-local, per-repo**, tidak tersinkron antar mesin/harness. Tapi lokasinya bisa dipindah via `autoMemoryDirectory` → ini titik attach yang bagus.

### Hermes (Nous Research) — memory kecil, dibekukan, ditulis lewat tool
- Dua file di `~/.hermes/memories/`: `MEMORY.md` (catatan agent, cap **2.200 karakter**) dan `USER.md` (profil user, cap **1.375 karakter**). Entry dipisah `§`.
- Dimuat sekali di awal sesi sebagai **frozen snapshot** → menjaga **prefix cache** LLM. Tulisan mid-session langsung ke disk tapi baru tampil di sesi berikutnya.
- Tool `memory`: `add`, `replace` (cari via substring `old_text`), `remove`. Tidak ada `read` (sudah di prompt).
- **Cap keras**: kalau penuh, tulisan gagal dan agent *dipaksa* konsolidasi. Tidak ada yang dibuang diam-diam.
- **Security scan**: entry dicek untuk prompt injection, exfiltration, unicode tak terlihat.
- `write_approval` opsional (review sebelum tersimpan).
- Episodic: semua sesi di SQLite FTS5 (`state.db`), dicari lewat `session_search` (raw, tanpa ringkasan LLM).
- **Provider plugin**: Honcho, Mem0, Hindsight, Supermemory, dll berjalan *berdampingan* dengan file bawaan.

### OpenClaw — Markdown sebagai sumber kebenaran + "dreaming"
- Semua memory = Markdown biasa di workspace. "Model tidak menyimpan apa pun selain yang ditulis ke disk."
  - `MEMORY.md`: fakta jangka panjang & keputusan, diinject awal sesi (dipotong kalau melebihi budget, file di disk tetap utuh).
  - `USER.md`: preferensi stabil, budget lebih kecil.
  - `memory/YYYY-MM-DD.md`: **daily notes** (episodic), hari ini + kemarin dimuat saat `/new`, sisanya hanya diindeks.
  - `DREAMS.md`: log konsolidasi untuk direview manusia.
- Retrieval: `memory_search` (hybrid vector + keyword, index SQLite) dan `memory_get` (baca file/rentang baris).
- **Pre-compaction flush**: sebelum context dikompres, harness menjalankan turn senyap yang menyuruh agent menyimpan hal penting.
- **Dreaming**: cron background yang mempromosikan kandidat dari short-term ke `MEMORY.md` dengan gate skor, frekuensi recall, dan keragaman query; kandidat dari sumber untrusted tidak pernah dipromosikan.

### claude-mem — auto-capture lewat hooks, retrieval progresif
- 5 lifecycle hook (SessionStart, UserPromptSubmit, PostToolUse, Stop, SessionEnd) — semua **observer**, fire-and-forget.
- Worker lokal (Bun) + SQLite (FTS5) + Chroma (vector) → hybrid search.
- **Progressive disclosure 3 langkah** via MCP: `search` (index ringkas ~50–100 token/hasil) → `timeline` (konteks sekitar) → `get_observations` (detail ~500–1000 token, hanya ID terpilih). Klaim hemat ~10x token.
- Multi-harness: installer untuk Claude Code, Codex, OpenCode, Gemini, Copilot, OpenClaw, Pi, dll; harness tanpa hook ditangani dengan **watch file log chat**.
- Privasi: konten dalam `<private>…</private>` tidak disimpan.

### beads — memory portabel yang hidup di luar context window
- Store: **Dolt** (SQL ber-versi) di `.beads/`; git dipakai sebagai transport (`refs/dolt/data`), JSONL hanya untuk export.
- `bd remember "insight"` menyimpan fakta; `bd prime` mencetak workflow + memories untuk diinject ke agent.
- Integrasi: `bd init` menulis section di **AGENTS.md**; `bd setup claude|codex|cursor` memasang hooks/skill; untuk agent lain cukup snippet AGENTS.md yang menunjuk ke `bd prime`. Ada `beads-mcp`.
- Pelajaran: **CLI + AGENTS.md snippet adalah adapter universal termurah** — semua harness yang bisa menjalankan shell bisa memakainya.

### Letta (MemGPT) — memory sebagai OS
- **Core memory blocks** (label, value, limit, description, read_only) selalu ada di context; agent mengeditnya sendiri lewat tool.
- **Archival memory** (tak terbatas, semantic search on demand) + **recall memory** (riwayat percakapan).
- Block bisa **dibagi antar agent**.
- **Agent File (`.af`)**: format open untuk mengekspor agent (system prompt, memory blocks, tools, message history, model config). Archival belum didukung, secrets diekspor `null`. Ini preseden terdekat untuk "transfer brain".

### Mem0 / OpenMemory — memory layer universal
- Ekstraksi fakta otomatis, di-scope per user/agent/session, hybrid retrieval (semantic + BM25 + entity).
- **OpenMemory** = server MCP lokal yang dipakai bersama oleh banyak client (Claude, Cursor, dll) — model "satu brain, banyak klien".

### Graphiti (Zep) — temporal knowledge graph
- Fakta disimpan beserta **kapan berlaku** (valid_at / invalid_at); fakta lama tidak ditimpa, tapi di-*invalidate*. Bisa menjawab "apa yang benar pada waktu X".
- Pelajaran: brain jangka panjang butuh **supersession, bukan overwrite**.

### Lainnya yang relevan
- **Qwen Code**: auto-memory per project + "dream" pass harian untuk dedupe & buang entry basi.
- **oh-my-pi (Hindsight)**: `retain` / `recall` / `reflect`, dan tiap sesi dikompres jadi "mental model" project yang auto-load di turn pertama sesi berikutnya.
- **gsd-core / planning-with-files**: state kerja disimpan di file `.planning/STATE.md`, `PLAN.md` dll — memory kerja jangka pendek yang crash-proof.
- **Codex / OpenInterpreter / Pi**: hook `PreCompact`/`PostCompact`/`SessionStart(resume)` — titik standar untuk flush & restore.
- **Anthropic memory tool (API)**: direktori `/memories` yang dioperasikan model lewat perintah file (`view`, `create`, `str_replace`, `insert`, `delete`, `rename`) — storage-nya disediakan aplikasi, jadi bisa diarahkan ke brain kita.

---

## 3. Pola yang konvergen (yang harus kita adopsi)

1. **Plain Markdown di disk = source of truth.** Claude Code, Hermes, OpenClaw, Qwen Code, Anthropic memory tool semuanya berakhir di sini. Index (FTS/vector) hanya turunan yang bisa dibangun ulang. Ini kunci *own the brain*: bisa dibaca manusia, di-diff, di-git.
2. **Dua tingkat: index kecil selalu dimuat + detail on demand.** `MEMORY.md` 200 baris (Claude Code), cap 2.200 char (Hermes), budget bootstrap (OpenClaw), `search → get` (claude-mem). Context itu mahal; brain yang bagus adalah **peta**, bukan ensiklopedia.
3. **Tipe memory eksplisit.** Minimal: `user` (siapa saya), `feedback` (koreksi/preferensi kerja), `project` (keputusan & state), `reference` (di mana info berada), plus `episodic` (log harian) dan `procedure` (skill).
4. **Snapshot beku di awal sesi** untuk menjaga prefix cache; tulisan mid-session ke disk, tampil sesi berikutnya.
5. **Capture ≠ promote.** Tulis cepat ke inbox/daily note → konsolidasi background ("dreaming") → promosi ke core dengan gate. Cap keras memaksa kurasi.
6. **Waktu & supersession.** Simpan `modified`, `valid_from`/`valid_to`, `supersedes`. Jangan overwrite fakta yang berubah.
7. **Hook lifecycle sebagai titik integrasi:** `SessionStart` (inject), `PreCompact` (flush), `Stop`/`SessionEnd` (ringkas). Untuk harness tanpa hook: AGENTS.md snippet + CLI (`bd prime`), atau watch log.
8. **Keamanan memory:** scan injection/exfiltration sebelum promosi (Hermes, OpenClaw), tag `<private>` (claude-mem), approval opsional, sumber untrusted tidak boleh dipromosikan otomatis. Memory yang persisten = permukaan serangan prompt injection yang persisten.
9. **Jangan simpan yang bisa diturunkan** dari kode/git/dokumen lain.

### Gap yang belum ada yang menyelesaikan (peluang rewire)
- Semua memory di atas **terikat ke satu harness** (lokasi, format, aturan load) atau **butuh server/DB** (Mem0, Graphiti, Letta). Belum ada **format brain netral** yang: file-first, git-syncable, punya skema, dan punya adapter tipis ke tiap harness.
- Auto memory Claude Code & Hermes bersifat **machine-local**; tidak ada sinkronisasi lintas mesin/harness.
- Belum ada standar seperti AGENTS.md **untuk memory personal** (bukan instruksi repo).

---

## 4. Usulan arsitektur rewire

```
~/brain/                      # repo git milik user — INI brain-nya
├── brain.yaml                # manifest: versi skema, identitas, budget, policy
├── core/
│   ├── USER.md               # siapa saya (selalu dimuat, cap ketat)
│   └── MEMORY.md             # index: 1 baris per memory → link ke topic file
├── memories/                 # topic file, 1 memory per file, frontmatter ber-skema
│   ├── feedback/…            # koreksi & preferensi kerja
│   ├── project/<slug>/…      # keputusan & state per proyek
│   └── reference/…
├── journal/YYYY-MM-DD.md     # episodic, append-only (capture cepat)
├── inbox/                    # kandidat dari harness, belum dipromosikan
├── skills/<name>/SKILL.md    # procedural memory (format Agent Skills)
└── .index/                   # SQLite FTS5 (+ embedding opsional) — gitignored, rebuildable
```

**Frontmatter memory (draf):**
```yaml
---
id: 01JABC...            # ULID, stabil lintas rename
type: feedback           # user | feedback | project | reference | procedure
scope: global            # global | project:<slug>
created: 2026-10-10T14:00:00Z
modified: 2026-10-10T14:00:00Z
valid_to: null           # diisi saat fakta tidak berlaku lagi
supersedes: []           # id memory yang digantikan
source: claude-code      # harness/sesi asal
confidence: confirmed    # confirmed | inferred
---
```

**Tiga lapis adapter (dari termurah ke terkaya):**

| Lapis | Mekanisme | Harness yang tercakup |
|---|---|---|
| 1. **Render** | `rewire render <harness>` menulis core ke format native: `@~/brain/core/MEMORY.md` import di `~/.claude/CLAUDE.md`, section di AGENTS.md/GEMINI.md, `~/.hermes/memories/*.md`, `~/.openclaw/workspace/MEMORY.md`; set `autoMemoryDirectory` Claude Code ke brain | Semua yang membaca file instruksi |
| 2. **CLI + MCP** | `rewire prime` (cetak konteks, gaya `bd prime`), `rewire remember`, `rewire recall`; server MCP dengan `search → get` progresif + `remember` | Semua harness yang punya shell atau MCP |
| 3. **Hooks** | SessionStart → inject snapshot; PreCompact → flush ke `inbox/`; SessionEnd → ringkasan ke `journal/` | Claude Code, Codex, Gemini CLI, OpenCode, Pi, dll |

Plus **importer** dua arah: tarik `~/.claude/projects/*/memory`, `~/.hermes/memories`, OpenClaw workspace ke `inbox/` → konsolidasi → brain. Dan **export** ke format seperti Letta `.af` bila perlu.

**Write pipeline:** harness menulis ke `inbox/`/`journal/` (cepat, tanpa gate) → `rewire consolidate` (manual atau cron "dream": dedupe, merge, supersede, scan injection, cek budget) → promosi ke `memories/` + update `MEMORY.md` → `git commit` (riwayat = audit trail, sync = `git push/pull`).

---

## 5. Keputusan yang perlu diambil berikutnya

1. **Bahasa implementasi CLI/MCP** (mis. TypeScript/Bun mengikuti claude-mem, atau Go/Rust untuk single binary).
2. **Embedding atau tidak di v1?** Rekomendasi: mulai FTS5 saja (zero-dependency, Hermes membuktikan cukup), embedding sebagai plugin.
3. **Harness target v1.** Rekomendasi: Claude Code + Codex + satu harness tanpa hook (via AGENTS.md + CLI) untuk membuktikan portabilitas.
4. **Konsolidasi pakai LLM siapa?** Pakai model dari harness yang sedang aktif, atau API key milik user.
5. **Kebijakan privasi**: apa yang tidak boleh keluar dari mesin (tag `<private>`, file terenkripsi untuk secrets, brain publik vs privat).

---

## Referensi
- best-of-agent-harnesses — https://github.com/ryanalberts/best-of-agent-harnesses
- Claude Code memory — https://code.claude.com/docs/en/memory
- Hermes memory — https://github.com/NousResearch/hermes-agent (docs `user-guide/features/memory`)
- OpenClaw memory — https://github.com/openclaw/openclaw (docs `concepts/memory`)
- claude-mem — https://github.com/thedotmack/claude-mem
- beads — https://github.com/gastownhall/beads
- Letta memory blocks & Agent File — https://github.com/letta-ai/letta, https://github.com/letta-ai/agent-file
- Mem0 / OpenMemory — https://github.com/mem0ai/mem0
- Graphiti — https://github.com/getzep/graphiti
- AGENTS.md — https://agents.md
