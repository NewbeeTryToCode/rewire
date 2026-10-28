# Hidupkan hub proyek, jangan pindahkan folder

Jawaban singkatnya: brain Anda tidak butuh pohon folder PARA. Yang dibutuhkan adalah **dua lapis catatan di dalam `nodes/` yang sudah ada**. Lapis bawahnya klaim atomik yang sudah ditulis rewire hari ini (`fact`, `decision`, `preference`, `constraint`). Lapis atasnya **halaman terkompilasi** yang benar-benar berisi: hub `project` yang hidup untuk setiap proyek atau area, dan halaman `concept` global untuk pengetahuan bersama. Lapis ini ditambah tiga gerbang promosi yang eksplisit: inbox → proyek → global → arsip. Ketiga keluhan Anda punya akar yang sama, dan akarnya ada di kode rewire, bukan di nama folder. Klaim dibatasi 600 karakter, hub proyek sengaja dibuat dengan body kosong, profil `coding` membuang keluaran asisten, dan "proyek" hanyalah string `scope` yang ditebak dari cwd. Tebakan itu sudah menghasilkan proyek hantu `project:user`. Memindahkan file ke `Projects/`, `Areas/`, `Resources/` di root brain akan merusak banyak hal tanpa peringatan: index hanya memindai satu level di bawah `nodes/`, pencarian id bergantung pada sufiks `-<id8>.md`, hub hanya dicari di `nodes/project/`, dan sync, review trust, serta `undo` semuanya berbasis path `nodes/`. Rekomendasinya mengambil bagian terbaik dari tiap pendekatan. Dari LYT/ACE: status "intensitas" proyek. Dari Karpathy LLM Wiki, Basic Memory, dan Cline Memory Bank: halaman terkompilasi di atas lapisan atomik. Dari Evergreen notes dan ADR: format catatan. Dari GTD dan agent-memory-vault: disiplin review dan gerbang manusia untuk promosi lintas proyek. Semuanya dijalankan lewat frontmatter dan halaman hasil generate, bukan lewat pemindahan file. Migrasinya bertahap. Fase 0 bisa dimulai hari ini tanpa mengubah kode. Fase 1–3 berupa perubahan kecil di Rust yang menghidupkan kembali desain yang dulu sudah direncanakan (hub page ≤300 kata, fase "sleep") tetapi ditunda.

## Tiga keluhan Anda adalah perilaku yang dirancang, bukan bug

**Keluhan 1: satu file md terlalu tipis.** Ini memang hasil desain. Validator menolak body klaim lebih dari **600 karakter** ("claims must be atomic") ([rust/src/schema.rs:92-94](/home/user/rewire/rust/src/schema.rs)). Prompt ekstraksi meminta "max 2 sentences" per klaim ([rust/src/consolidate.rs:70-106](/home/user/rewire/rust/src/consolidate.rs)). Profil `coding` yang aktif di brain Anda menyuruh LLM mengembalikan "few, high-quality claims; zero is fine" dan melewatkan narasi ([rust/src/consolidate.rs:45-57](/home/user/rewire/rust/src/consolidate.rs)). Pengaturan `transcripts = "auto"` hanya menyimpan giliran pengguna untuk profil coding ([rust/src/consolidate.rs:991-995](/home/user/rewire/rust/src/consolidate.rs), [/home/user/brain/brain.toml](/home/user/brain/brain.toml)). Akibatnya, roadmap belajar yang dibangun asisten hanya tersisa sebagai satu decision dua kalimat ([roadmap-belajar-…-r5t189wq.md](/home/user/brain/nodes/decision/roadmap-belajar-ai-engineering-dan-chip-design-3-r5t189wq.md)). Isi lengkapnya tidak tersimpan di nodes maupun di sessions. Ada satu sumber "tipis" lagi yang lebih mendasar. Dua hub proyek yang ada **hanya berisi frontmatter dengan body kosong** ([rewire-ntgx8mpe.md](/home/user/brain/nodes/project/rewire-ntgx8mpe.md)). Penyebabnya, `ensure_project_hub` sengaja menulis body `""` ([rust/src/ops.rs:265-285](/home/user/rewire/rust/src/ops.rs)). Keputusan ini diambil setelah LLM membuat hub ganda dan hub kosong "Project rewire." memakan budget snapshot ([docs/iterations.md:20-24](/home/user/rewire/docs/iterations.md)). Perbaikannya waktu itu membuang konten, padahal yang perlu dibuat deterministik adalah *pembuatan* hub. Spesifikasi awal sebenarnya menjanjikan `project` sebagai "hub page (summary ≤ 300 words, created by consolidation)" ([docs/design/mvp-spec.md:67-75](/home/user/rewire/docs/design/mvp-spec.md)).

**Keluhan 2: banyak proyek.** Di sistem file, proyek tidak ada sebagai wadah. Proyek hanya berupa `scope: "project:<slug>"` ditambah edge `about` ke hub kosong. Slug ditentukan dari pola `[projects]` di `brain.toml`, lalu nama git top-level, lalu, khusus saat konsolidasi, **basename cwd** ([rust/src/consolidate.rs:556-563](/home/user/rewire/rust/src/consolidate.rs), [rust/src/project.rs:41-63](/home/user/rewire/rust/src/project.rs)). Karena `brain.toml` Anda belum punya entri `[projects]`, sesi yang berjalan dari `/home/user` melahirkan hub `user` ([user-4tnc2n12.md](/home/user/brain/nodes/project/user-4tnc2n12.md)). Decision tentang konsolidasi idle rewire ikut tersasar ke sana ([konsolidasi-…-fg531z4v.md](/home/user/brain/nodes/decision/konsolidasi-rewire-saat-session-idle-10-menit-fg531z4v.md)). Ada yang lebih buruk: recall dan snapshot memakai `project::detect` tanpa fallback itu ([rust/src/ops.rs:86-90](/home/user/rewire/rust/src/ops.rs)). Dari cwd yang sama, catatan `project:user` jadi **tidak terlihat** oleh recall default. Struktur semacam ini tidak akan bertahan dengan puluhan proyek, termasuk proyek non-kode seperti belajar atau karier yang tidak punya repo git.

**Keluhan 3: alur yang kabur.** Satu-satunya "promosi" di rewire hari ini adalah inbox → `nodes/` lewat `rewire review approve|reject` ([rust/src/ops.rs:419-464](/home/user/rewire/rust/src/ops.rs)). Tingkat kedua (proyek → global) tidak ada. Tidak ada perintah promote, rescope, atau merge ([rust/src/cli.rs:39-160](/home/user/rewire/rust/src/cli.rs)). Auto-UPDATE lintas scope dilarang ([rust/src/consolidate.rs:357-366](/home/user/rewire/rust/src/consolidate.rs)). Judge hanya membandingkan kandidat dari `global` dan proyek saat ini ([rust/src/consolidate.rs:911-932](/home/user/rewire/rust/src/consolidate.rs)), sehingga fakta yang sama dari dua proyek tidak pernah dipertemukan. Scope dipilih sekali saja oleh LLM saat ekstraksi, sebagai pilihan biner `global|project`. Pengguna hanya bisa mengubahnya dengan mengedit file inbox secara manual sebelum approve, dan cara itu tidak terdokumentasi.

Dari diagnosis ini turun satu set batasan keras untuk setiap desain baru:

| Batasan rewire | Dampak pada desain | Bukti |
|---|---|---|
| Index memindai hanya `nodes/<dir>/*.md` (satu level, tanpa rekursi) | Folder bersarang `nodes/projects/rewire/decision/` atau folder PARA di root **tidak terindeks** | [rust/src/index.rs:764-875](/home/user/rewire/rust/src/index.rs), [rust/src/brain.rs:561-582](/home/user/rewire/rust/src/brain.rs) |
| Nama file wajib berakhir `-<id8>.md` | Nama file tanpa sufiks memutus `get`, resolusi link, dan `resolve_id` | [rust/src/nodefmt.rs:34-62](/home/user/rewire/rust/src/nodefmt.rs), [rust/src/brain.rs:584-612](/home/user/rewire/rust/src/brain.rs) |
| Note baru selalu ditulis ke `nodes/<type>/`; hub dicari hanya di `nodes/project/` | Hub yang dipindah akan memicu hub duplikat | [rust/src/brain.rs:637-656](/home/user/rewire/rust/src/brain.rs), [rust/src/ops.rs:265-277](/home/user/rewire/rust/src/ops.rs) |
| `scope` hanya `global` atau `project:*` | `area:*` atau `resource:*` butuh perubahan schema | [rust/src/schema.rs:79-82](/home/user/rewire/rust/src/schema.rs) |
| `status`, `aliases`, `tags` diturunkan ulang pada setiap tulis | Status siklus hidup proyek **tidak boleh** memakai `status` atau `tags`; perlu field lain | [docs/design/format.md:43-67](/home/user/rewire/docs/design/format.md) |
| Key frontmatter tak dikenal dipertahankan saat rewrite, tetapi tidak dikembalikan `get` | Field baru aman disimpan, tetapi baru berguna bagi agen setelah masuk `schema::FIELDS` | [rust/src/nodefmt.rs:236-240](/home/user/rewire/rust/src/nodefmt.rs), [rust/src/ops.rs:208-214](/home/user/rewire/rust/src/ops.rs) |
| Recall membuang hub `project` kecuali dicapai lewat link; `concept` tidak dibuang | Halaman `concept` bisa jadi hit recall langsung; hub proyek tidak | [rust/src/index.rs:1300-1302](/home/user/rewire/rust/src/index.rs) |
| Snapshot memakai hub hanya jika `body != ''`, tiap baris dipotong 380 karakter; snippet recall = 160 karakter pertama body | Kalimat pertama body harus berupa ringkasan padat | [rust/src/snapshot.rs:228-238](/home/user/rewire/rust/src/snapshot.rs), [rust/src/index.rs:1313](/home/user/rewire/rust/src/index.rs) |

Kabar baiknya, batasan ini justru menunjuk ke solusi yang murah. Body hub dan `concept` tidak punya batas panjang. Snapshot sudah memakai body hub yang tidak kosong sebagai "Current context". Field tambahan sudah bertahan saat rewrite. Memindah file ke folder lain *di level pertama* `nodes/` tetap terindeks.

## Delapan belas pendekatan diuji terhadap tiga keluhan

Setiap pendekatan dinilai pada tiga kriteria: K1 (satu catatan kaya informasi), K2 (banyak proyek berjalan bersamaan), dan K3 (alur capture → proyek → bersama). Kolom "biaya di rewire" mengacu pada batasan di tabel sebelumnya.

### Metode PKM manusia: kuat di alur, lemah di kepadatan

| Pendekatan | Apa yang disyaratkan | Di mana gagal (K1/K2/K3) | Biaya atau kerusakan di rewire | Yang diambil |
|---|---|---|---|---|
| **PARA** (Forte) | Empat folder menurut *actionability*, plus Inbox; review mingguan/bulanan oleh manusia | K1: tidak mengatur isi catatan. K2: baik (folder sejajar), tetapi Project vs Area sering tertukar dan satu file hanya bisa punya satu rumah. K3: Resources jadi gudang; arsip menyembunyikan catatan dari AI | **Tinggi** bila jadi folder fisik: tidak terindeks, keluar dari sync/trust/undo | Registry proyek dengan status; "panen pelajaran sebelum arsip" |
| **CODE** / Progressive Summarization | Distill bertahap: bold → highlight → ringkasan eksekutif di atas | K1: **menjawab langsung** lewat ringkasan L4 di atas catatan. K2/K3: bergantung pada PARA | Rendah: cukup konvensi body | Kalimat pertama = ringkasan (cocok dengan snippet 160 karakter) |
| **Zettelkasten** | Fleeting → literature → permanent; satu ide per kartu; structure notes | K1: **lemah** bila atomik secara harfiah. K2: tidak mengurus proyek. K3: **paling eksplisit** | Rendah: klaim rewire sudah zettel | Pipeline tiga tingkat sebagai model K3 |
| **Evergreen notes** (Matuschak) | "Satu hal, tetapi seluruhnya"; judul berupa klaim; diatur per konsep, bukan per proyek | K1: **terkuat**. K2: sengaja tidak per proyek. K3: implisit | Rendah: batas 600 karakter cukup untuk satu klaim lengkap | Judul proposisional; "proyek merujuk, tidak memiliki" |
| **LYT/ACE + MOC** (Milo) | Atlas/Calendar/Efforts; intensitas On/Ongoing/Simmering/Sleeping; MOC sebagai folder non-eksklusif | K1: MOC membantu navigasi. K2: **terbaik** (ada status parkir). K3: sedang | Rendah bila intensitas jadi field frontmatter, tinggi bila jadi folder | Field `stage`; hub dan concept berfungsi sebagai MOC |
| **Johnny.Decimal** | Batas 10×10×100; ID AC.ID; JDex | K1: netral. K2: lemah (PRO.AC.ID dibatasi 99). K3: tidak ada | Sedang: angka tidak bermakna bagi LLM tanpa JDex | Tidak diambil, kecuali prinsip "dangkal" |
| **ADR** (Nygard/MADR) | Title/Status/Context/Decision/Consequences; supersede, jangan diedit | K1: **padat** untuk keputusan. K2/K3: lokal proyek | Rendah: `supersedes` dan `valid_to` sudah ada; 600 karakter cukup untuk ADR ringkas | Template decision |
| **Diátaxis** | Empat tipe: tutorial/how-to/reference/explanation; jangan dicampur | K1: mencegah catatan campur aduk. K2/K3: tidak relevan | Rendah: cukup field tambahan | `form:` pada halaman concept |
| **GTD** | Capture → clarify → organize → reflect → engage; review mingguan sebagai faktor sukses | K3: **disiplin inbox** yang hilang di rewire | Rendah: inbox sudah ada | Inbox-zero mingguan dan aksi rute |

Untuk PARA, temuan yang paling berpengaruh di sini adalah kritik soal satu lokasi per file. Satu resource bisa melayani beberapa proyek, tetapi "cannot reside within two places at once" ([Medium @paralloid](https://medium.com/@paralloid/para-method-for-organizing-your-content-26e29435b92d)). Selain itu, arsip proyek dapat membuat catatan sulit ditemukan oleh alat AI ([yu-wenhao](https://yu-wenhao.com/en/blog/lyt-framework-guide/)). Vendor agen juga menyimpulkan bahwa PARA "doesn't work for agents out of the box" tanpa retrieval semantik dan **manajemen siklus hidup otomatis** ([Claw Mart](https://www.shopclawmart.com/blog/para-method-openclaw-memory)). Forte sendiri, dalam kursus "AI Second Brain" 2026, kini mengajarkan PARA sebagai "modular context bundles for AI" ([Forte Labs](https://fortelabs.com/blog/introducing-the-ai-second-brain/)). Bagi rewire, intinya begini: setiap proyek perlu satu unit konteks yang bisa dimuat utuh. Unit itu adalah hub page. Folder tidak diperlukan untuk itu. Dari sisi Zettelkasten, zettelkasten.de sendiri mengakui bahwa menuntut atomisitas sejak awal memindahkan pekerjaan berpikir yang sulit ke luar sistem ([zettelkasten.de](https://zettelkasten.de/posts/principle-of-atomicity-difference-between-principle-and-implementation/)). Rumusan Matuschak, "only about one thing—but … capture the entirety of that thing" ([Matuschak](https://notes.andymatuschak.org/Evergreen_notes_should_be_atomic)), adalah perbaikan paling tepat untuk K1. Klaim tetap sempit, tetapi lengkap di dalam lingkupnya. Konsensus praktisi untuk hibrida ini ringkas: "Zettelkasten is for ideas. PARA is for action", dan "the project uses the concept without owning it" ([glukhov.org](https://www.glukhov.org/de/knowledge-management/methods/para-method-for-engineers/)). ACE menambahkan status Simmering, sesuatu yang tidak dimiliki PARA. Dengan status itu, banyak proyek bisa diparkir tanpa diarsipkan ([LYT blog](https://blog.linkingyourthinking.com/notes/the-four-intensities-of-efforts)). Johnny.Decimal kalah untuk brain yang terus tumbuh. Dokumentasinya sendiri menyebut bahwa pola "Expand an area" telah "abandoned the constraints of Johnny.Decimal" ([johnnydecimal.com](https://johnnydecimal.com/documentation/expand-an-area-overview.md)).

### Memori agen berbasis file: konsensus menuju "atomik di bawah, terkompilasi di atas"

| Pendekatan | Apa yang disyaratkan | Di mana gagal (K1/K2/K3) | Biaya atau kerusakan di rewire | Yang diambil |
|---|---|---|---|---|
| **Karpathy LLM Wiki** | `raw/` immutable + wiki milik LLM + file schema; ingest/query/lint; `index.md` + `log.md` | K1: **kuat** (halaman terkompilasi). K2: tidak dirinci. K3: ada ingest, tidak ada gerbang. Index-first hanya teruji sampai ±100 sumber | Sedang: perlu fase sleep/lint, dan rewrite berisiko halusinasi | Hub terkompilasi; lint; README sebagai `index.md` |
| **Cline Memory Bank** | Enam file tetap per proyek, semua dibaca di setiap tugas | K1: kuat. K2: satu bank per repo. K3: tidak ada global | Sedang: biaya token di setiap tugas | Pemisahan bagian stabil (brief) dan volatil (activeContext) |
| **Claude Code auto memory** | `MEMORY.md` satu baris per memori (200 baris/25KB) + file topik | K1: file topik bisa panjang. K2: per repo, lokal mesin. K3: tidak ada promosi | Rendah | Batas keras untuk index; "jangan simpan yang bisa diturunkan dari kode" |
| **Basic Memory** | Satu entitas per file; baris `- [kategori] isi #tag`; `relasi [[Target]]` | K1: **hibrida terbaik** (halaman padat, baris bisa di-grep). K2: multi-project. K3: manual | Rendah: format body cocok untuk hub/concept | Format "Observasi" di halaman concept |
| **Letta / MemFS** | Memory blocks yang ditulis ulang agen; dilacak git; "dreaming" | K1: blok terkompilasi. K2/K3: global/proyek/agen | Tinggi bila diadopsi penuh | Validasi bahwa "brain git ditulis agen" itu layak |
| **Graphiti/Zep** | Graf temporal; validity window; episode sebagai provenance | K1: lemah per fakta. K3: invalidasi, bukan penghapusan | Sudah ada di rewire (bitemporal) | Pertahankan `valid_to`/`supersedes`/`source` |
| **Mem0, A-MEM** | Ekstraksi atomik ADD-only; catatan Zettelkasten yang berevolusi | K1: lemah | Rendah | Konfirmasi bahwa lapis atomik tetap perlu |
| **Template Obsidian + Claude** (claudesidian, obsidian-mind, Rome-1, agent-memory-vault) | AGENTS.md/CLAUDE.md pendek; folder PARA atau entitas; skill inbox; frontmatter `type`/`status`/`updated` | K1: kuat (Brief/Current-State/Decisions). K2: per folder proyek. K3: **graduation queue** dengan gerbang manusia | Tinggi bila folder ditiru; rendah bila hanya konvensi | AGENTS.md, aturan kesegaran, gerbang promosi |

Karpathy merumuskan argumen intinya sebagai "the wiki is a persistent, compounding artifact". Index-first navigation "works surprisingly well at moderate scale (~100 sources, ~hundreds of pages)" ([Karpathy gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)). Brain yang menelan setiap transkrip sesi akan melewati 100 sumber dengan cepat. Rewire sudah punya FTS dan recall, jadi batas ini tidak menjadi masalah. Pelajaran yang relevan justru dari dokumen riset rewire sendiri: lint LLM berskala kuadratik, dan kesalahan ekstraksi awal dapat "permanently poison the store" ([docs/research/karpathy-llm-wiki.md:24-36](/home/user/rewire/docs/research/karpathy-llm-wiki.md)). Karena itu, halaman terkompilasi harus selalu bisa dibangun ulang dari lapis atomik dan tidak boleh menjadi sumber kebenaran tunggal. Basic Memory memberi bentuk konkretnya: halaman entitas yang body-nya terdiri dari baris observasi berkategori dan baris relasi bertipe ([basic-memory README](https://github.com/basicmachines-co/basic-memory)). Cline Memory Bank membagi konteks proyek ke `projectbrief.md` yang stabil dan `activeContext.md`/`progress.md` yang volatil ([cline memory-bank.md](https://github.com/cline/prompts/blob/main/.clinerules/memory-bank.md)). Pembagian yang sama muncul di Rome-1 sebagai `Brief.md` dan `Current-State.md` ([Rome-1 vault-schema](https://github.com/Rome-1/obsidian-claude-starter/blob/main/System/vault-schema.md)).

Untuk K3, satu-satunya mekanisme promosi lintas proyek yang terdokumentasi adalah milik agent-memory-vault. Aturan lokal proyek ditulis langsung ke memori proyek. Kandidat lintas proyek, yaitu yang punya "evidence in 2+ distinct projects", masuk `graduation-queue.md`. Hanya manusia yang bisa meluluskannya ke `00-Rules/_principles/` ([agent-memory-vault README](https://github.com/cindyxu1030/agent-memory-vault/blob/main/README.md)). Tidak ada sumber primer yang menunjukkan promosi otomatis "jika berulang di banyak proyek" pada Claude Code, Letta, Basic Memory, maupun Mem0. Aturan promosi harus dirancang sendiri. Obsidian-mind menyumbang aturan tulis yang lahir dari kegagalan nyata. Dari "single-source status", satu pernyataan status yang salah "hardened into ~8 notes downstream". Dari aturan "date-stamp volatile facts", fakta yang bisa berubah ditulis dengan "as of YYYY-MM-DD" ([obsidian-mind AGENTS.md](https://github.com/breferrari/obsidian-mind/blob/main/AGENTS.md)). Obsidian-second-brain menegakkan aturan yang sejalan: setiap fakta harus "timeless, dated, or a pointer" ([obsidian-second-brain README](https://github.com/eugeniughelbur/obsidian-second-brain/blob/main/README.md)). Fakta basi di brain Anda, yang menyebut remote brain "had not yet been initialized" ([vt34xd89.md](/home/user/brain/nodes/fact/remote-brain-rewire-newbeetrytocode-brain-vt34xd89.md)), adalah contoh persis kegagalan yang dicegah aturan ini.

### Bagian yang bersumber dari ekstrak pencarian

Proxy jaringan memblokir banyak situs primer, jadi beberapa temuan hanya berasal dari ekstrak hasil pencarian dan belum dibaca langsung di halaman aslinya. Situs yang terkena adalah fortelabs.com dan every.to (Forte), zettelkasten.de dan forumnya, notes.andymatuschak.org, johnnydecimal.com, linkingyourthinking.com, diataxis.fr, cognitect.com (posting asli Nygard), forum.obsidian.md, reddit, dev.to, blog Kenneth Reitz dan Stefan Imhoff, docs.cline.bot, docs.basicmemory.com, docs.letta.com, arxiv.org, dan mpt.solutions. Akibatnya, definisi PARA, lapisan Progressive Summarization, intensitas ACE, kutipan Matuschak, aturan Johnny.Decimal, dan kompas Diátaxis di laporan ini **berasal dari ekstrak pencarian**. Sumber bahkan tidak sepakat soal jumlah intensitas ACE (empat atau tiga). Gist Karpathy dibaca lewat alat fetch yang merangkum, sehingga kutipannya hanyalah potongan pendek. Skema memory block Letta tidak terverifikasi. Klaim bahwa sebuah posting Agustus 2026 di mpt.solutions menemukan tiga mode kegagalan LLM wiki dalam produksi hanya terlihat sebagai snippet. Jumlah bintang repo berasal dari agregator. Sumber yang dibaca langsung dan utuh: README, AGENTS.md, dan schema dari template GitHub (claudesidian, obsidian-mind, Rome-1, agent-memory-vault, productory, work-vault, obsidian-second-brain); dokumen memori Claude Code; dokumen memory tool Anthropic; template ADR (Nygard via joelparkerhenderson, MADR, adr-tools); `memory-bank.md` Cline; README basic-memory, graphiti, letta-code, dan mem0; serta seluruh kode dan brain lokal. Untungnya, rekomendasi di bawah bersandar terutama pada sumber yang terbaca langsung dan pada kode rewire. Bagian yang hanya didukung ekstrak, seperti intensitas ACE dan kutipan Matuschak, dipakai sebagai inspirasi konvensi, bukan sebagai dasar perubahan kode.

## Rekomendasi: hub terkompilasi di atas klaim atomik, tetap di dalam `nodes/`

Pilihlah **hibrida "hub-dan-klaim"**. Klaim atomik tetap menjadi lapis kebenaran yang bisa di-supersede, di-recall, dan dilacak sumbernya. Di atasnya ada dua jenis halaman terkompilasi yang panjang dan padat. Pertama, hub `project`, satu per proyek *atau* area, dibedakan dengan field `kind`. Kedua, halaman `concept` ber-scope `global` sebagai pengetahuan bersama sekaligus MOC. Semantik PARA dan ACE disimpan di frontmatter (`kind`, `stage`), bukan di folder. Arsip tetap berarti status, bukan pemindahan file. Dengan begitu, ketiga keluhan terjawab tanpa melanggar satu pun batasan index.

### Struktur folder target

```text
/home/user/brain/
├── brain.toml                 # + registry [projects.<slug>] match = [...]  (urutan: pola spesifik dulu)
├── AGENTS.md                  # BARU, ditulis manusia: lapisan "schema" ala Karpathy (≤200 baris)
├── CLAUDE.md                  # BARU, satu baris: @AGENTS.md
├── README.md                  # generated: Home/index (proyek aktif per stage, area, pengetahuan bersama, inbox)
├── inbox/<ulid>.md            # GTD inbox: usulan memori + scope usulan + alasan rute
├── sessions/<YYYY-MM-DD>/<harness>-<slug>.md   # lapis "Calendar"/raw: episode, immutable
├── nodes/
│   ├── project/<slug>-<id8>.md    # hub proyek DAN area (kind: project|area), body terkompilasi
│   ├── concept/<slug>-<id8>.md    # pengetahuan bersama (global) + MOC topik, body terkompilasi
│   ├── person/<slug>-<id8>.md     # orang (tetap)
│   ├── decision/  fact/  preference/  constraint/   # klaim atomik, ≤600 karakter (tetap)
├── skills/<name>/SKILL.md
├── events/<device>/<YYYY-MM>.jsonl
└── .index/  .state/           # derived, device-local
```

Pohon ini sengaja hampir sama dengan pohon sekarang. Ada tiga perubahan nyata. Pertama, `AGENTS.md`/`CLAUDE.md` di root. File ini menampung konvensi bagi agen lain yang membaca brain dan berada di luar `nodes/`, sehingga tidak tersentuh index. Templat Rome-1 dan agent-memory-vault memakai pola "AGENTS.md sebagai sumber tunggal, CLAUDE.md mengimpor", sesuai aturan presedensi Claude Code ([Claude Code memory docs](https://code.claude.com/docs/en/memory)). Kedua, folder `nodes/concept/` mulai dipakai. Ketiga, registry `[projects]` diisi. Proyek non-kode seperti belajar AI engineering atau karier dipetakan lewat direktori kerja khusus, misalnya `~/projects/belajar-ai`. Fungsi `detect` mencocokkan pola sebagai substring terhadap remote git, git top-level, *dan* path absolut cwd, bahkan di luar repo git. Pola pertama yang cocok menang ([rust/src/project.rs:41-63](/home/user/rewire/rust/src/project.rs)). Jadi jangan pernah memakai pola `/home/user` saja, karena pola itu akan menelan semua sesi. Opsi yang ditolak beserta alasannya: `Projects/` di root (tidak terindeks), `nodes/projects/<slug>/…` (terlalu dalam), angka Johnny.Decimal (tidak bermakna bagi LLM), dan `archive/` fisik (memutus link `## Related` serta membuat file tampak baru di mata review trust pada perangkat lain).

### Template catatan

Semua frontmatter ditulis dengan gaya rewire: satu nilai JSON per baris ([rust/src/fm.rs:1-46](/home/user/rewire/rust/src/fm.rs)). Field bertanda *baru* bertahan saat rewrite tetapi belum dibaca `get` sampai Fase 1. Field `status`, `aliases`, dan `tags` dihitung ulang oleh rewire, jadi jangan diisi manual.

**Hub proyek/area** (`nodes/project/rewire-ntgx8mpe.md`). Kalimat pertama body menjadi snippet recall (160 karakter) dan baris pertama snapshot. Itulah ringkasan L4 ala Progressive Summarization, ditaruh di posisi yang benar-benar dibaca mesin.

```markdown
---
id: "01M60R2HJ4PRKQYG6NNTGX8MPE"
type: "project"
title: "rewire"
scope: "project:rewire"
importance: 4
confidence: "explicit"
provenance: "user"
valid_from: "2026-10-28"
valid_to: null
recorded_at: "2026-10-28T07:54:55Z"
keywords: ["rewire", "memori agen", "rust", "brain", "konsolidasi"]
related_to: ["[[konsep-memori-agen-berbasis-file-xxxxxxxx]]"]
kind: "project"
stage: "on"
outcome: "rewire siap rilis dengan brain yang bisa memuat banyak proyek"
reviewed: "2026-10-28"
---
rewire adalah tool Rust yang mengubah sesi agen menjadi brain markdown+git; fokus saat ini: struktur multi-proyek.

## Ringkasan
(≤300 kata, ditulis manusia atau LLM lewat inbox; per 2026-10-28)

## Tujuan dan definisi selesai
## Status per 2026-10-28
## Pertanyaan terbuka
## Pelajaran yang layak dibawa keluar
(kandidat promosi ke global; dipanen sebelum stage → done)

<!-- rewire:members:start -->
## Keputusan
- [[konsolidasi-rewire-saat-session-idle-10-menit-fg531z4v]] — konsolidasi saat idle 10 menit
## Fakta dan batasan
- [[remote-brain-rewire-newbeetrytocode-brain-vt34xd89]] — remote brain di GitHub
## Sesi terkait
- sessions/2026-10-28/claude-code-7314e139-….md
<!-- rewire:members:end -->
```

Nilai `stage` meminjam intensitas ACE: `on`, `ongoing` (dipakai untuk `kind: "area"`), `simmering`, `sleeping`, dan `done`. Hub area seperti "karier" atau "belajar-ai-engineering" memakai scope `project:<slug>` yang sama. Ini kompromi yang disengaja supaya schema tidak perlu diubah. Field `kind` yang membedakannya, dan perbedaan itu menjawab kesalahan PARA yang paling sering (Project vs Area) tanpa harus memindahkan file.

**Klaim decision** (ADR ringkas, ≤600 karakter). Strukturnya mengikuti Nygard (Context/Decision/Consequences), sedangkan supersede ditangani mekanisme rewire yang sudah ada. Isi di bawah hanya contoh ilustratif bentuk, bukan fakta tentang alasan keputusan Anda.

```markdown
---
type: "decision"
title: "Konsolidasi rewire berjalan saat sesi idle 10 menit"
scope: "project:rewire"
about: ["[[rewire-ntgx8mpe]]"]
source: "claude-code · 7314e139-… · turn 41"
valid_from: "2026-10-28"
reviewed: "2026-10-28"
---
Keputusan: konsolidasi juga dipicu bila sesi idle ≥600 detik, selain SessionEnd/PreCompact.
Konteks: <masalah yang mendorong keputusan, 1 kalimat>.
Konsekuensi: <apa yang jadi lebih mudah atau sulit, 1 kalimat>.
Tinjau ulang bila: <syarat revisi>.
```

**Klaim fact/preference/constraint.** Judulnya berupa proposisi lengkap ("judul sebagai API" ala Matuschak). Body berisi satu klaim yang lengkap di dalam lingkupnya, satu baris konteks atau alasan, dan penanda waktu bila faktanya bisa berubah ("per 2026-10-28"). Provenance tetap lewat `source`. Lapis episode di `sessions/` berperan sebagai sumber kebenaran mentah, sama seperti episode pada Graphiti ([graphiti README](https://github.com/getzep/graphiti)).

**Halaman pengetahuan bersama** (`nodes/concept/`, scope `global`). Polanya adalah body Basic Memory ditambah daftar proyek yang memakainya. Karena recall hanya membuang hub bertipe `project` ([rust/src/index.rs:1300-1302](/home/user/rewire/rust/src/index.rs)), halaman ini **langsung bisa ditemukan lewat recall dari proyek mana pun**. Itulah yang membuatnya cocok menjadi rumah pengetahuan lintas proyek.

```markdown
---
type: "concept"
title: "Jalur belajar AI engineering"
scope: "global"
importance: 4
keywords: ["ai engineering", "roadmap", "belajar", "karier", "llm"]
related_to: ["[[belajar-ai-engineering-xxxxxxxx]]", "[[tujuan-karier-bekerja-di-perusahaan-ai-gmwxfsqt]]"]
form: "explanation"
reviewed: "2026-10-28"
---
Roadmap belajar AI engineering dan chip design: tiga jalur, ±15 jam/minggu, target 12–24 bulan (per 2026-10-28).

## Ringkasan
## Observasi
- [prinsip] Utamakan sumber gratis yang sudah jadi standar industri ([[preferensi-sumber-belajar-gratis-dan-standar-ind-yafrehew]])
- [fase] … (sumber: sessions/2026-10-28/…)
## Dipakai di
- [[belajar-ai-engineering-xxxxxxxx]]
## Klaim pendukung
- [[roadmap-belajar-ai-engineering-dan-chip-design-3-r5t189wq]]
```

Field `form` memakai kosakata Diátaxis (`how-to`, `reference`, `explanation`) untuk mencegah satu halaman mencampur panduan dan penjelasan. Relasi hanya memakai enam relasi rewire yang sudah ada (`about`, `part_of`, `depends_on`, `related_to`, `supersedes`, `contradicts`). Dengan begitu, graph Obsidian dan `## Related` tetap berfungsi tanpa perubahan schema. Bagian "Dipakai di" dan daftar anggota hub ditulis sebagai link eksplisit di teks. Backlink dan hasil Dataview/Bases tidak terlihat oleh agen yang memakai `cat`/grep maupun oleh GitHub ([obsidian-mind AGENTS.md](https://github.com/breferrari/obsidian-mind/blob/main/AGENTS.md), [Obsidian help: Bases](https://github.com/obsidianmd/obsidian-help/blob/master/en/Bases/Introduction%20to%20Bases.md)).

## Alur hidup catatan: empat gerbang dengan aturan promosi yang bisa diperiksa

Alur yang direkomendasikan memetakan pipeline Zettelkasten (fleeting → permanent) dan GTD (capture → clarify → organize → reflect) ke struktur rewire. Setiap perpindahan tingkat berarti mengubah `scope`, edge, atau `stage`. Tidak ada perpindahan folder.

| Gerbang | Dari → ke | Aturan (siapa, kapan, kriteria) | Mekanisme di rewire |
|---|---|---|---|
| **G0 Capture** | Sesi → `sessions/` + `inbox/` | Otomatis setelah SessionEnd/idle. Semua yang tidak lolos gerbang auto masuk inbox | Sudah ada ([rust/src/consolidate.rs:368-375](/home/user/rewire/rust/src/consolidate.rs)) |
| **G1 Clarify/rute** | `inbox/` → `nodes/` dengan scope yang benar | Review mingguan sampai inbox kosong. Per item, tentukan: klaim proyek, klaim global, bahan halaman (hub/concept), atau buang. **Bila proyek tidak terdeteksi, item ditahan tanpa scope dan tidak boleh ditebak dari cwd** | Fase 0: edit `scope` di file inbox lalu `approve`. Fase 1: `review approve --scope` |
| **G2 Promosi** | `project:x` → `global` | Dipromosikan bila **salah satu** benar: (a) klaim tetap benar tanpa menyebut proyeknya (preferensi, aturan, fakta tentang Anda); (b) klaim serupa muncul di **≥2 proyek** (aturan graduation agent-memory-vault); (c) dipanen dari "Pelajaran yang layak dibawa keluar" sebelum proyek `done`. Selalu lewat persetujuan manusia | Fase 0: ubah `scope` di file (id tetap, git mencatat). Fase 1: `rewire promote <id>`. Fase 2: lint mengusulkan lewat inbox |
| **G3 Arsip** | `stage: on/simmering` → `sleeping`/`done` | Diusulkan bila tidak ada sesi selama N minggu atau ada penanda "selesai" yang eksplisit. Panen pelajaran (G2c) dulu, baru arsip. Klaim yang basi di-supersede, tidak dihapus | Fase 0: ubah `stage` manual. Fase 3: usulan otomatis. Snapshot tidak memuat hub `sleeping`/`done` |

Ada tiga detail desain yang penting. Pertama, promosi **mengubah scope di tempat**. Promosi tidak memakai `SUPERSEDE`, karena di rewire SUPERSEDE berarti "kebenaran berubah" dan akan mengisi `valid_to` ([rust/src/ops.rs:291-375](/home/user/rewire/rust/src/ops.rs)). Edge `about` ke hub asal dipertahankan sebagai jejak "dipelajari di proyek x". Konsekuensinya, klaim yang sudah dipromosikan keluar dari hitungan "Projects" di README, karena README hanya menghitung anggota dengan `scope == hub.scope` ([rust/src/ops.rs:945-1066](/home/user/rewire/rust/src/ops.rs)), tetapi tetap tampil di daftar anggota hub. Kedua, preferensi dan constraint tentang Anda sendiri, misalnya "menulis dalam bahasa Indonesia", sebaiknya **langsung global** di G1. Recall default hanya melihat `global` ditambah proyek cwd ([rust/src/project.rs:65-71](/home/user/rewire/rust/src/project.rs)), jadi preferensi yang ber-scope proyek tidak akan ikut ke proyek lain. Ketiga, gerbang manusia di G2 dan G3 bukan formalitas. Karpathy sendiri memilih ingest satu sumber "with close supervision" karena satu sumber bisa menyentuh 10–15 halaman ([Karpathy gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)). Dua preferensi bahasa Indonesia yang terduplikasi di inbox Anda juga membuktikan bahwa de-dup LLM di AWAITING REVIEW hanya bersifat saran ([/home/user/brain/README.md:41-49](/home/user/brain/README.md)).

## Migrasi empat fase yang tidak memutus index dan recall

| Fase | Isi | Perubahan kode | Risiko terhadap batasan |
|---|---|---|---|
| **0: hari ini, tanpa kode** | (1) Isi `[projects.rewire] match = ["NewbeeTryToCode/rewire", "/home/user/rewire"]` dan satu entri per proyek non-kode dengan direktori kerjanya sendiri. (2) Perbaiki proyek hantu: ubah `scope` decision konsolidasi menjadi `project:rewire`, ganti `about` ke `[[rewire-ntgx8mpe]]`, lalu `rewire forget` hub `user-4tnc2n12`. (3) Tulis body hub rewire secara manual mengikuti template. (4) Buat hub `belajar-ai-engineering` (`kind: "area"`) dan satu halaman `concept` roadmap di `nodes/concept/` dengan nama `<slug>-<id8>.md` dan id ULID valid. (5) Perbarui fakta remote brain yang basi. (6) Tambahkan `AGENTS.md`/`CLAUDE.md`. (7) Ubah `transcripts` ke `"all"` (nilai sah: `auto|user|all|none`, [rust/src/config.rs:243-244](/home/user/rewire/rust/src/config.rs)) supaya keluaran asisten seperti roadmap tersimpan di `sessions/` | Tidak ada | Rendah. Semua file tetap di `nodes/<dir>/` dengan sufiks id8. Jalankan `rewire rebuild` dan `rewire doctor` sesudahnya. `transcripts = "all"` memperbesar sesi dan menyimpan lebih banyak teks (sudah diredaksi secret) |
| **1: perbaikan kecil** | (a) Hapus fallback basename cwd di `project_of` ([rust/src/consolidate.rs:556-563](/home/user/rewire/rust/src/consolidate.rs)); item tanpa proyek ditahan di inbox. (b) `rewire review approve --scope <s>` dan `rewire promote <id> [--scope global]`. (c) Masukkan `kind`, `stage`, `reviewed`, `form` ke `schema::FIELDS` agar dikembalikan `get`. (d) Generate blok `rewire:members` di hub secara deterministik, seperti `## Related`. (e) Snapshot hanya mengambil paragraf pertama/Ringkasan hub dan melewati hub `sleeping`/`done` | Kecil, terlokalisasi di `consolidate.rs`, `ops.rs`, `cli.rs`, `schema.rs`, `nodefmt.rs`, `snapshot.rs` | Rendah. Format on-disk tetap. Validator scope tidak berubah |
| **2: kepadatan** | (a) Profil `mixed` untuk brain berisi kode dan pribadi: boleh mengusulkan halaman `concept` panjang (selalu lewat inbox) untuk keluaran substantif asisten. (b) Batas body per tipe, misalnya `decision` sampai ±1.200 karakter untuk ADR penuh, sementara klaim lain tetap 600. (c) Aksi inbox `RESCOPE`/`PROMOTE` | Sedang. Daftar aksi inbox dan schema ekstraksi berubah | Sedang. Ubah `docs/design/format.md` bersamaan, karena kontraknya "frozen" |
| **3: sleep/lint** | Fase "sleep" yang dulu direncanakan ([docs/design/synaptic-memory.md:294-305](/home/user/rewire/docs/design/synaptic-memory.md)): ringkas ulang hub (sebagai usulan di inbox), cari duplikat lintas scope dengan FTS semua scope (kandidat G2b), usulkan arsip (G3), cari fakta tanpa tanggal, hub tanpa anggota, dan link mati. README dibatasi keras, misalnya 200 baris seperti `MEMORY.md` | Sedang. Perintah baru, lint inkremental | Rendah bila semua keluaran berupa usulan di inbox |

Urutan ini penting. Fase 0 sudah menyelesaikan sebagian besar keluhan 1 dan 2 untuk brain Anda yang masih kecil (7 node, 5 item inbox), karena body hub yang ditulis manusia langsung dipakai snapshot sebagai "Current context" ([rust/src/snapshot.rs:228-238](/home/user/rewire/rust/src/snapshot.rs)). Fase 1 menutup sumber kesalahan struktural, yaitu tebakan cwd dan tidak adanya perintah promosi, sebelum brain membesar. Ada satu peringatan untuk Fase 0. Saya tidak menguji pemindahan atau pengeditan file secara end-to-end, karena riset ini bersifat read-only. Perilaku trust di perangkat lain setelah file diedit juga belum ditelusuri, dan `.state/untrusted.json` yang dikunci per path mengisyaratkan bahwa file baru akan dianggap perlu review. Jadi lakukan perubahan Fase 0 di satu perangkat, commit, lalu sync.

## Kesimpulan

Pertanyaan "folder apa yang tepat" ternyata salah sasaran. Di rewire, folder adalah detail implementasi yang dikunci oleh index. Unit organisasi yang sebenarnya adalah tiga hal: **scope** (siapa yang bisa melihat), **hub** (di mana konteks dikompilasi), dan **gerbang** (kapan sesuatu naik tingkat). Desain asli rewire sebenarnya sudah benar soal atomisitas klaim, yang didukung riset Dense X ([docs/design/mvp-spec.md:79-81](/home/user/rewire/docs/design/mvp-spec.md)) dan model bitemporal ala Graphiti. Yang hilang adalah lapis terkompilasi di atasnya, dan lapis itu hilang karena sebuah perbaikan bug yang membuang konten hub, bukan karena keputusan prinsipil. Menghidupkan kembali hub page ≤300 kata, kali ini dengan *pembuatan* yang deterministik dan *isi* yang di-generate atau dikurasi, mempertemukan rewire dengan arah yang sama yang ditempuh Karpathy, Basic Memory, Cline, dan template Obsidian 2026.

Implikasi berikutnya: hambatan terbesar untuk brain multi-proyek bukan skala, melainkan **penetapan proyek pada sesi non-kode**. Selama sesi pribadi Anda berjalan dari `/home/user`, setiap struktur akan gagal di titik yang sama. Konvensi "satu direktori kerja per proyek atau area" ditambah registry `[projects]` adalah keputusan paling berpengaruh dalam laporan ini, dan biaya penerapannya nol baris kode. Setelah itu, aturan promosi "berlaku tanpa nama proyek, atau terbukti di ≥2 proyek, dengan persetujuan manusia" memberi jawaban yang bisa diperiksa untuk pertanyaan yang tidak dijawab oleh PARA, Zettelkasten, maupun Karpathy: kapan sesuatu yang dipelajari di satu proyek layak menjadi milik semua proyek.
