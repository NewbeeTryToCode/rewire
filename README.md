# rewire

**Own the brain, transfer it anywhere.** Memory portabel untuk agent harness: satu "brain" milik kamu (repo git berisi file markdown) yang bisa dipasang ke Claude Code, Codex, dan harness lain.

- **File adalah sumber kebenaran.** Node memory = markdown dengan frontmatter; bisa dibaca, diedit, di-diff, dibuka di Obsidian.
- **Synapse dihitung, bukan disimpan.** Pemakaian dicatat sebagai event append-only; kekuatan memory (ACT-R, `d=0.7`) dihitung ulang dari log. Aturan dan preferensi tidak pernah decay.
- **Belajar setelah sesi.** Transcript dikonsolidasi oleh LLM menjadi klaim atomik; yang eksplisit dari user langsung disimpan, sisanya masuk inbox untuk direview.
- **Aman secara default.** Provenance diambil dari transcript (bukan dari klaim model), output tool tidak pernah otomatis jadi aturan, secret ditolak, setiap konsolidasi bisa di-`undo`.

Desain: [`docs/design/mvp-spec.md`](docs/design/mvp-spec.md) · Konsep: [`docs/design/synaptic-memory.md`](docs/design/synaptic-memory.md) · Bukti simulasi: [`experiments/phase0`](experiments/phase0/README.md) · Log iterasi: [`docs/iterations.md`](docs/iterations.md)

## Instalasi

```bash
pipx install git+https://github.com/NewbeeTryToCode/rewire     # atau: uv tool install ...
rewire init                     # membuat ~/brain (atau set REWIRE_BRAIN)
```

Python ≥ 3.11, tanpa dependensi. Konsolidasi default memakai `claude -p` (login Claude Code yang sudah ada). Untuk memakai API langsung: `pip install 'rewire[anthropic]'` lalu set `provider = "anthropic"` di `brain.toml`.

## Pasang ke harness

```bash
rewire install claude-code --write   # hook SessionStart/SessionEnd/PreCompact (settings di-backup dulu)
claude mcp add --scope user rewire -- rewire mcp

rewire install codex                 # menulis Agent Skill + mencetak snippet AGENTS.md
codex mcp add rewire -- rewire mcp
```

Harness tanpa hook tetap bisa dipakai: agent menjalankan `rewire prime` lewat instruksi di AGENTS.md/skill, dan transcript-nya dikonsolidasi secara *lazy* pada hook berikutnya atau dengan `rewire consolidate --pending`.

## Pemakaian sehari-hari

```bash
rewire prime                      # snapshot yang disuntikkan di awal sesi
rewire recall "keputusan storage" # cari memory
rewire get <id>
rewire remember --type preference "Selalu tulis test sebelum fitur"   # masuk inbox
rewire review                     # lihat inbox
rewire review approve <id|all>
rewire consolidate --pending      # proses transcript yang tertunda
rewire undo <session-id>          # batalkan hasil konsolidasi satu sesi
rewire doctor                     # cek kesehatan brain
```

## Struktur brain

```
~/brain/
├── brain.toml                    # parameter, budget, provider, peta proyek
├── nodes/<type>/<slug>-<id>.md   # memory (constraint, preference, fact, decision, project, person, concept)
├── events/<device>/YYYY-MM.jsonl # log pemakaian (merge=union, tidak pernah konflik)
├── sessions/YYYY-MM-DD/*.md      # ringkasan tiap sesi yang dikonsolidasi
├── inbox/*.md                    # usulan yang menunggu review
├── .index/                       # SQLite FTS5, dibangun ulang otomatis (gitignored)
└── .state/                       # watermark & antrean per device (gitignored)
```

## Pengembangan

```bash
python -m unittest discover -s tests
python experiments/phase0/sim.py
```
