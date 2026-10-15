# Analisis kesalahan dan rekomendasi: LoCoMo penuh, Haiku (mode full)

## Ringkasan

- Ada 111 soal yang salah (7,2%). Sekitar 33 di antaranya (30%) bukan kesalahan sistem. Penyebabnya: pembicara tertukar di pertanyaan (17), gold yang keliru (8), info yang hanya ada di gambar (4), dan juri yang tidak menerapkan aturannya sendiri (4). Kalau soal-soal ini dikoreksi, J kira-kira naik dari 92,8% ke ≈94,8% (inferensi).
- Kesalahan sistem yang sebenarnya terbagi rata. Ada 26 di sisi penjawab: menolak menyebut nama atau menyimpulkan (7), salah memilih detail (13), dan salah menghitung (6). Ada 26 di retrieval: kata kategori di pertanyaan tidak cocok dengan nama spesifik di memori (12), bukti nyaris lolos k (9), dan parafrase atau ejaan (5). Ada 11 di ingest: durasi relatif, keterangan foto, dan nama panggilan tidak diekstrak.
- Open-domain (J 66,7%) paling lemah karena dua hal. Pertama, 51% giliran buktinya tidak berbagi satu pun kata isi dengan pertanyaan. Run ini memakai BM25 saja (semantic = 0 di semua indeks). Kedua, 15 dari 32 soal OD yang salah punya gold spekulatif, sehingga jawaban yang masuk akal tetap dinilai salah.
- Juri longgar untuk daftar. Dari 130 `ok_list_part`, sampel manual 40 soal menunjukkan hanya sekitar 30% (≈39 soal) yang benar-benar kurang item. Sisanya adalah alarm palsu dari metrik leksikal: `gold_items` memecah kalimat pada kata "and" dan parafrase tidak dikenali. Inflasi J dari daftar kira-kira 2,5 poin (inferensi). Angka 68,8% item ditemukan terlalu rendah sebagai ukuran.
- Lima prioritas: (1) errata gold, juri independen, dan metrik daftar yang ketat; (2) anggaran retrieval k=50/episode=30, yang memulihkan bukti untuk 14 dari 51 soal yang buktinya hilang; (3) varian prompt jawab yang berani menyebut nama yang dikenali dan berani menyimpulkan; (4) kata kategori di keyword dan pertanyaan hipotetis saat ingest; (5) ekspansi kueri multi-query.

## 1. Cara analisis dan batasannya

- Saya membaca ke-111 baris salah (wrong_answer 60, wrong_retrieval 37, wrong_ingest 13, wrong_abstain 1) bersama giliran bukti asli di dataset: pembicara, tanggal sesi, dan keterangan foto (`blip_caption`).
- Konteks yang dikirim ke penjawab tidak disimpan di results JSON, jadi saya merekonstruksinya. Caranya: `locomo_prepare` dijalankan ulang pada salinan brain (bukan aslinya), dengan k=30 dan episodes=20. Untuk 233 dari 300 soal yang saya periksa, hash prompt-nya persis sama dengan kunci cache jawaban asli, jadi konteksnya terverifikasi identik. Untuk 66 dari 67 sisanya, status bukti (excerpt/fact/missing) tetap sama. Bedanya kemungkinan dari aktivasi yang bergantung waktu (`recall_beta`) (inferensi).
- Rekonstruksi cocok dengan report: semua bukti ada 84,5% (excerpt) dan 87,2% (excerpt atau fakta), sedangkan report menulis 87,3%.
- Peringkat bukti yang hilang dihitung dengan dua cara. Pertama, peringkat potongan mentah: tanpa penggabungan dan tanpa trim, dengan seed dari 30 fakta yang sama. Kedua, peringkat fakta dengan k=300.
- Tingkat kesalahan sebenarnya di `ok_list_part` diperkirakan dari sampel acak 40 baris (seed 7) yang saya nilai manual. Selang ketidakpastiannya lebar, kira-kira 17–45%.
- Pengelompokan ke klaster adalah penilaian saya per soal. Beberapa soal bisa masuk dua klaster, dan saya memilih penyebab yang paling menentukan.

## 2. Tabel klaster akar masalah (111 soal salah)

| # | Klaster | n | single / multi / temporal / OD | Pemilik masalah | Contoh qid |
|---|---|---|---|---|---|
| A | Pertanyaan menukar pembicara: detail diucapkan orang lain, gold mengikuti pertanyaan yang keliru | 17 | 16 / 0 / 1 / 0 | dataset | conv-43-136, conv-50-151, conv-48-123 |
| B | Gold salah: tahun, tanggal, durasi, fakta, atau film dianggap buku | 8 | 1 / 3 / 4 / 0 | dataset | conv-49-80, conv-47-31, conv-43-6 |
| C | Jawaban hanya ada di gambar, sedangkan caption BLIP tidak memuatnya | 4 | 2 / 0 / 0 / 2 | dataset | conv-42-92, conv-42-91 |
| D | Juri (Haiku) melanggar aturan toleransinya sendiri (tanggal ±14 hari, durasi ±50%) | 4 | 2 / 0 / 2 / 0 | juri | conv-48-101, conv-42-24, conv-41-63 |
| E | Open-domain dengan gold spekulatif atau pengetahuan dunia; jawaban masuk akal tapi berbeda | 15 | 0 / 0 / 0 / 15 | dataset/juri (sebagian penjawab) | conv-43-51, conv-44-53, conv-26-69 |
| F | Penjawab menolak menyebut nama atau menarik kesimpulan padahal bukti ada (konflik di prompt) | 7 | 0 / 2 / 1 / 4 | prompt jawab | conv-47-16, conv-43-8, conv-44-18 |
| G | Penjawab salah memilih atau membaca detail; bukti ada di konteks (termasuk caption dan aritmetika waktu) | 13 | 11 / 0 / 2 / 0 | penjawab | conv-30-73, conv-43-76, conv-49-106 |
| H | Salah hitung atau agregasi (dobel hitung, rencana ikut dihitung) | 6 | 0 / 6 / 0 / 0 | penjawab | conv-42-79, conv-42-19, conv-50-47 |
| I | Kata kategori vs nama spesifik ("state" vs Tampa, "countries" vs Rio, "religious" vs church) | 12 | 0 / 5 / 1 / 6 | retrieval/ingest | conv-42-87, conv-47-6, conv-48-19 |
| J | Bukti nyaris lolos: peringkat tepat di bawah batas k atau episodes | 9 | 0 / 4 / 2 / 3 | retrieval (anggaran) | conv-42-53, conv-44-16, conv-49-43 |
| K | Parafrase atau ejaan tanpa kata bersama ("imposter" vs "impostors", "kid" vs "with my dad") | 5 | 0 / 3 / 1 / 1 | retrieval | conv-47-17, conv-50-40, conv-47-11 |
| L | Ekstraksi melewatkan detail: durasi relatif, rujukan ke foto, nama panggilan, sentimen kecil | 11 | 3 / 3 / 4 / 1 | ingest | conv-42-8, conv-49-95, conv-42-14 |

Jumlahnya adalah 111. Sebagai perbandingan dengan label diagnosis: semua 17 soal di A dan 7 dari 8 soal di B berlabel `wrong_answer`. Artinya, 24 dari 60 `wrong_answer` (40%) sebenarnya adalah gold yang bermasalah, bukan kegagalan penjawab.

Pengelompokan yang lebih kasar:

| Kelompok | Klaster | n | Bagian dari salah |
|---|---|---|---|
| Bukan kesalahan sistem (dataset atau juri) | A, B, C, D | 33 | 30% |
| Subjektif (OD spekulatif) | E | 15 | 14% |
| Penjawab | F, G, H | 26 | 23% |
| Retrieval | I, J, K | 26 | 23% |
| Ingest | L | 11 | 10% |

### 2.1 Di mana bukti yang hilang berada (51 soal: wrong_retrieval, wrong_ingest, wrong_abstain)

Peringkat potongan mentah terburuk per soal:

| Peringkat | Soal |
|---|---|
| ≤ 20 | 2 |
| 21–40 | 14 |
| 41–80 | 13 |
| > 80 | 22 |

Sebagian besar soal di kelompok > 80 punya overlap kata isi 0 dengan pertanyaan.

Simulasi anggaran retrieval (tanpa LLM, 1.535 soal yang punya bukti, brain yang sama):

| k fakta / episode | Semua bukti (excerpt) | Semua bukti (excerpt atau fakta) | Token konteks (perkiraan) | Soal hilang-bukti yang pulih (dari 51) |
|---|---|---|---|---|
| 30 / 20 (run ini) | 84,5% | 87,2% | 4,8k | 0 |
| 50 / 20 | 84,8% | 88,3% | 5,6k | 5 |
| 30 / 30 | 86,6% | 88,8% | 6,5k | 8 |
| 50 / 30 | 87,4% | 90,0% | 7,3k | 14 |
| 50 / 40 | 88,5% | 90,7% | 8,9k | 16 |
| 100 / 40 | 88,5% | 91,7% | 10,9k | 21 |
| 30 / 20 tanpa trim | 84,6% | 87,4% | 5,0k | 2 |

Trim excerpt hampir tidak merugikan (hanya 2 soal). Batas k dan kosakata yang jadi masalah utama.

### 2.2 Inflasi juri pada daftar (130 `ok_list_part`)

- Report menunjukkan list J 95,4% tetapi item ditemukan hanya 68,8%. Jika setiap jawaban daftar diberi nilai sebesar porsi item yang ditemukan secara leksikal, J total turun ke 87,5%. Ini batas bawah yang terlalu pesimis.
- 36 baris dinilai benar oleh juri tetapi punya 0% item leksikal, dan kebanyakan sebenarnya jawaban lengkap dengan kata lain. Contoh: conv-26-78 (gold "Figurines, shoes", jawaban "pink running sneakers … wooden figurine dolls"). Contoh lain adalah conv-26-87 dan conv-50-15. Pada keduanya, `gold_items` memecah kalimat alasan pada kata "and" menjadi dua item.
- Sampel manual 40 baris `ok_list_part`: 12 benar-benar kurang item (30%). Dari 12 itu, 9 karena bukti item yang hilang tidak terambil (conv-43-35 surfing, conv-41-25 Spain, conv-43-7 Smoky Mountains). Tiga sisanya karena penjawab melewatkan item yang ada di konteks (conv-42-61, conv-48-84, conv-41-74).
- Dari sampel ini, perkiraan saya (inferensi): sekitar 39 dari 130 soal (rentang kasar 22–59) diterima hanya karena aturan "satu item cukup". Nilai J yang ketat untuk daftar kira-kira 2,5 poin di bawah J resmi, yaitu ≈90,3% secara keseluruhan.
- Aturan "satu item cukup" adalah bagian dari juri resmi Mem0, jadi J resmi tetap perlu dilaporkan supaya bisa dibandingkan. Yang perlu ditambahkan adalah metrik daftar yang ketat dan berbasis LLM, bukan menggantikan J resmi.

### 2.3 Open-domain (96 soal, J 66,7%)

- Dari 32 soal OD yang salah: 15 punya gold spekulatif (E), 6 masalah kategori vs nama spesifik (I), 4 penjawab menolak menyebut nama (F), 3 nyaris lolos k (J), 2 hanya di gambar (C), 1 parafrase (K), dan 1 ingest (L).
- Masalah kosakata: 51% giliran bukti OD tidak punya kata isi yang sama dengan pertanyaan. Angkanya 38% untuk multi-hop, 19% untuk temporal, dan 15% untuk single-hop. Dengan BM25 saja, kesenjangan ini hanya bisa ditutup oleh keyword dan pertanyaan hipotetis dari ingest, dan keduanya belum memuat kata kategori.
  - Contoh: fakta "Nate's turtles at Tampa beach" punya keyword `florida`, tetapi tidak punya `state`. Pertanyaan "What state did Nate visit?" menempatkannya di peringkat fakta 95.
  - Contoh lain: fakta "Deborah's earlier Rio trip" tidak punya `brazil`, `country`, atau `travel`.
- Penjawab terlalu patuh pada satu aturan di prompt resmi: "NEVER generate specific names … that do not appear in any memory". Aturan ini mengalahkan Misc 2 ("you may name it"). Bukti dari teks jawaban di cache:
  - conv-47-16: "No memory gives a name for this game, so I won't supply one" (gold: UNO).
  - conv-48-75: gold Exploding Kittens.
  - conv-43-66: menolak menyebut buku Star Wars.
  - conv-43-8: "No candidate name appears", padahal excerpt D3:15 tentang Under Armour ada di konteks.
- Gold OD dipotong pada ";" (`run.py`), sesuai harness Mem0, dan ini menyangkut 11 soal. Pemotongan membuang alasannya, yang kadang memuat kata kunci yang membuat jawaban bisa diterima juri. Dampaknya belum diukur.
- Bahkan saat semua bukti ada, J OD hanya 86,0%, karena banyak soal yang memang menuntut tebakan pengetahuan dunia: Hatha yoga (conv-43-51), park ranger (conv-44-53), Voyageurs (conv-44-44).

## 3. Rekomendasi berperingkat

Perkiraan soal pulih adalah inferensi. Dasarnya: J naik dari 75,6% (bukti hilang) ke 95,9% (bukti lengkap), dan klaster di atas. Angka ini belum diverifikasi lewat run ulang.

| # | Rekomendasi | Jenis | Perkiraan soal pulih | Biaya | Parity Py+Rust | Aman sekarang? | Bukti qid |
|---|---|---|---|---|---|---|---|
| 1 | Errata gold, juri independen, dan metrik daftar ketat | eval | 0 soal sistem; J bersih ≈ +2 poin, J ketat ≈ −2,5 poin | rendah (juri ~3M token prompt) | tidak (eval Python saja) | ya | conv-43-136, conv-49-80, conv-48-101 |
| 2 | Anggaran retrieval k=50, episodes=30 | query-time | 6–10 | +50% token konteks, ≈ +20% token jawab | tidak, selama hanya flag eval; ya jika default `ops.recall` atau MCP berubah | ya (sebagai flag eval) | conv-42-53, conv-47-6, conv-49-43 |
| 3 | Varian prompt jawab: boleh menyebut nama yang dikenali, wajib berkomitmen pada kesimpulan | prompt jawab | 5–9 | rendah (jawab ulang ~1.176 panggilan) | tidak | ya, sebagai varian terpisah dari prompt resmi | conv-47-16, conv-43-8, conv-44-18 |
| 4 | Kata kategori dan hipernim di `KEYWORDS_RULE` dan `QUESTIONS_RULE` | ingest | 5–9 | sedang (ingest ulang 272 sesi) | ya (`consolidate.py` + `consolidate.rs`, tes kesamaan prompt) | belum (perlu anggaran ingest ulang) | conv-42-87, conv-48-19, conv-44-20 |
| 5 | Ekspansi kueri multi-query oleh model pemanggil | query-time | 6–12 | sedang (1 panggilan LLM kecil per soal) | ya jika masuk `ops.recall` atau MCP; tidak jika prototipe di eval | prototipe eval: ya | conv-50-56, conv-26-59, conv-50-40 |
| 6 | Aturan ekstraksi untuk durasi relatif, caption, dan nama panggilan | ingest | 4–7 | sedang (ingest ulang) | ya | belum | conv-42-8, conv-49-95, conv-42-14 |
| 7 | Aturan hitung dan agregasi di prompt jawab | prompt jawab | 2–3 | rendah | tidak | ya, sebagai varian | conv-42-79, conv-42-19, conv-50-47 |
| 8 | Varian ejaan dan kata majemuk di stemming | query-time | 1–2 | rendah | ya (`text.py` + `text.rs`) | ya, dengan tes | conv-47-17, conv-49-95 |
| 9 | Kebersihan eval: hapus nama dan pertanyaan LoCoMo dari contoh prompt ingest, catat konfigurasi retrieval di report, perbaiki 11 approve yang gagal | eval/ingest | 0–1 | rendah | ya untuk prompt; tidak untuk report | ya | conv-48-87, log `approve failed` |
| ✗ | Jangan: aturan "jawab saja untuk pembicara lain" agar klaster A lolos | — | (≈17 di LoCoMo) | — | — | tidak | conv-43-136, conv-50-151 |

### Rincian per rekomendasi

**1. Errata gold, juri independen, dan metrik daftar ketat (eval, aman sekarang, tanpa parity).**

Langkahnya:
- Simpan daftar qid yang gold-nya bermasalah (klaster A, B, C; 29 soal) di `experiments/eval/` lengkap dengan alasan. Laporkan J resmi dan J tanpa errata secara berdampingan.
- Jalankan ulang juri dengan model dari keluarga lain, misalnya gpt-4o-mini seperti Mem0, atau Sonnet. Ini cukup dengan cache jawaban, tanpa menjawab ulang. Ukur tingkat kesepakatan dengan juri Haiku.
- Tambahkan metrik daftar berbasis LLM: berapa item gold yang tercakup, dengan parafrase dihitung.
- Perbaiki `gold_items` supaya gold berbentuk kalimat tidak dipecah pada kata "and" (contoh conv-26-87, conv-50-15).

Alasannya: juri sama dengan penjawab (report sudah memberi peringatan), dan 4 soal di D menunjukkan juri tidak konsisten. Contohnya conv-48-101: "a year ago", dihitung dari Januari 2023, menjadi "Januari 2022", tetapi dinilai salah. Contoh lain conv-42-24: 18 Juni vs "weekend after 3 June" masih dalam 14 hari. Tanpa langkah ini, perbaikan 2–8 tidak bisa diukur dengan andal, karena noise label (≈33 soal) lebih besar daripada gain yang diharapkan dari tiap perbaikan.

**2. Anggaran retrieval k=50 fakta, episodes=30 (query-time, aman sebagai flag).**

Pada simulasi, setelan ini menaikkan "semua bukti (excerpt atau fakta)" dari 87,2% ke 90,0%. Bukti juga pulih untuk 14 dari 51 soal yang buktinya hilang. Contohnya conv-42-53 (potongan D5:4 di peringkat 23), conv-49-43 (fakta D7:2 di peringkat 45), dan conv-47-6 (fakta Stamford di peringkat 34).

Biayanya: token konteks naik dari ≈4,8k ke ≈7,3k. Risikonya: konteks yang lebih panjang bisa sedikit menurunkan ketelitian penjawab (inferensi). Karena itu, jalankan A/B pada sampel stratified sebelum mengganti default.

Ada alternatif yang lebih murah: k=50 dengan episodes=20 hanya menambah ≈0,8k token dan memulihkan 5 soal. Jika default produk (`ops.recall` atau MCP) ikut diubah, Python dan Rust harus disamakan, karena recall saat ini identik 304/304.

**3. Varian prompt jawab (prompt jawab, Python saja, aman sebagai varian).**

Penjawab menafsirkan aturan "NEVER generate specific names" sebagai larangan menyebut nama yang dikenali dari deskripsi. Contohnya conv-47-16 (UNO dari deskripsi aturan main) dan conv-48-75 (Exploding Kittens). Penjawab juga menolak menghubungkan dua memori, seperti pada conv-43-8 dan conv-44-18 (breeder). Selain itu, penjawab berhenti di "the memories don't give an exact date" padahal tanggal bisa dihitung (conv-50-62, conv-43-48).

Usulan:
- Tambahkan pengecualian eksplisit: nama terkenal yang cocok dengan deskripsi (permainan, buku, merek) boleh disebut.
- Untuk "likely/might", berikan tebakan terbaik dengan kata "likely".
- Selalu ubah rentang tanggal menjadi durasi.

Penting: `ANSWER_GENERATION_PROMPT` adalah salinan verbatim prompt Mem0. Mengubahnya membuat skor tidak bisa dibandingkan lagi dengan Mem0. Jadi laporkan sebagai kolom terpisah ("prompt rewire"), jangan menggantikan prompt resmi.

**4. Kata kategori di keyword dan pertanyaan hipotetis (ingest, perlu parity, belum aman tanpa anggaran ingest ulang).**

Tambahkan instruksi berikut ke `KEYWORDS_RULE` dan `QUESTIONS_RULE`:
- Untuk tempat, tulis juga kota, negara bagian, dan negara, beserta kata kategorinya ("city", "state", "country", "travel").
- Untuk makanan, olahraga, permainan, dan agama, tulis kata kategorinya ("meat", "sport", "console", "religion", "church").

Bukti:
- conv-42-87: fakta Tampa punya `florida`, tetapi tidak punya `state`, sehingga ada di peringkat 95.
- conv-48-87 dan conv-48-19: Rio dan Paris tanpa `brazil`, `france`, atau `country`.
- conv-44-20: "Chicken Pot Pie" tanpa `meat`.
- conv-26-59: "made for a local church" tanpa `religious`.

Konstanta prompt ini ada di `src/rewire/consolidate.py` dan `rust/src/consolidate.rs`, dan ada tes yang memeriksa prompt konsolidasi identik. Keduanya harus diubah bersamaan, lalu brain di-ingest ulang (272 sesi, p50 42 detik per sesi).

**5. Ekspansi kueri multi-query (query-time).**

Model pemanggil membuat 2–4 kueri tambahan berisi contoh konkret. Contohnya, "Which cities did Dave travel to?" diperluas menjadi "trip San Francisco Detroit workshop". Hasilnya digabung dengan RRF.

Ini menyasar I, K, dan sebagian E: conv-50-56 (peringkat potongan 129 dan 98), conv-50-40 (Tupac di "used to rock a song", peringkat 75), dan conv-26-69. Bentuknya sejalan dengan parameter `refers_to` yang sudah ada, yaitu konteks yang diselesaikan oleh model pemanggil.

Buat prototipe dulu di `experiments/eval` (tanpa parity). Kalau terbukti membantu, baru jadikan parameter `ops.recall` atau MCP di kedua implementasi.

**6. Aturan ekstraksi untuk detail yang terlewat (ingest, parity).**

Kasus yang terlewat:
- conv-42-8: "I've had them for 3 years" tidak menjadi fakta, padahal seharusnya menjadi fakta bertanggal "sejak ±2019".
- conv-49-25: "working on it for two years".
- conv-49-95: kata "smart watch" hanya ada di caption, dan teksnya hanya berbunyi "it tracks my progress".
- conv-42-14: fakta dari "Hey Jo, …" tidak menyimpan nama panggilan.

Tambahkan aturan:
- Ubah durasi relatif menjadi tanggal mulai.
- Gabungkan caption ke klaim jika teks merujuk "this" atau "it".
- Simpan nama panggilan.

Catatan metodologi: diagnosis menghitung bukti "ada lewat fakta" walaupun fakta itu kehilangan detail yang ditanyakan. Contohnya conv-42-14 (berlabel `wrong_answer`, padahal "Jo" tidak ada di konteks). Jadi angka `wrong_answer` sedikit terlalu tinggi, dan masalah ingest terlalu rendah.

**7. Aturan hitung (prompt jawab, varian).**

Kasusnya:
- conv-42-79: 4 vs 3 skenario; "started writing another one" lalu "wrapped up my second" dihitung dua kali.
- conv-42-19: trail yang baru direncanakan ikut dihitung.
- conv-50-47: kunjungan masa kecil ikut dihitung.

Usulan: tambahkan aturan "gabungkan laporan dari peristiwa yang sama; hitung yang sudah terjadi saja". Gain-nya kecil, dan aturan ini juga mengubah prompt resmi.

**8. Ejaan dan kata majemuk (query-time, parity `text.py` dan `text.rs`).**

Kasusnya: "imposter" vs "impostors" (conv-47-17, fakta di peringkat 59, lalu penjawab abstain) dan "smartwatch" vs "smart watch" (conv-49-95). Ini perbaikan kecil yang murah, tetapi rawan overfitting. Uji di LongMemEval juga.

**9. Kebersihan eval.**

- `QUESTIONS_RULE` memakai nama dari LoCoMo (Deborah, Jon, Caroline) dan contoh "Which countries has Deborah visited?". Contoh itu hampir sama dengan soal uji conv-48-87 ("Which countries has Deborah traveled to?"). Ini kebocoran kecil dari set uji ke prompt produk. Ganti dengan nama dan contoh netral (parity di `consolidate.rs`).
- Report tidak menyebut bahwa semantic search mati. Meta indeks menunjukkan `semantic = 0`, karena `wordllama` tidak terpasang. Catat ini bersama `SEED_WEIGHT`, `EP_WHOLE`, dan SHA commit di summary.
- `run.log` mencatat 11 "approve failed" (proposal yang note-nya berubah sebelum di-approve), jadi perubahan itu tidak masuk brain. Dampaknya belum diukur.

**Yang tidak disarankan.** Haiku sering sudah mengenali bahwa pertanyaan menukar pembicara (conv-50-151: "photography was Dave's, not Calvin's"). Menambah aturan "jawab saja dengan detail pembicara lain" akan menaikkan skor LoCoMo sampai ≈17 soal, tetapi itu sama dengan menyetel sistem pada label yang keliru. Di pemakaian nyata, aturan itu justru menimbulkan atribusi yang salah. Lebih baik masukkan soal-soal ini ke errata (rekomendasi 1).

## 4. Contoh bukti per klaster

Setiap entri memuat qid, pertanyaan, gold, jawaban (diringkas), dan alasan.

### A. Pembicara tertukar di pertanyaan (17)

- **conv-43-136**: "What did Tim say about his injury on 16 November, 2023?" Gold: "The doctor said it's not too serious". Jawaban: Tim tidak cedera; ia menanggapi cedera John. Alasan: D18:10 diucapkan oleh **John**, jadi jawabannya benar secara substansi.
- **conv-50-151**: "What hobby did Calvin take up recently?" Gold: Photography. Jawaban: memperbaiki mobil, dengan catatan "photography was Dave's, not Calvin's". Alasan: D30:1 diucapkan **Dave**.
- **conv-48-123**: "What did Jolene and Anna discuss while watching the sunset?" Jawaban: itu Deborah dan Anna, dan mereka sadar saling menginspirasi. Alasan: D10:17 diucapkan **Deborah**. Soal kembarannya, conv-48-75 dan conv-48-179, menyebut giliran yang sama (D27:12) sebagai milik Deborah sekaligus milik Jolene.

Soal lain di klaster ini: conv-26-94, conv-42-119, conv-42-195, conv-43-86, conv-43-135, conv-43-164, conv-43-166, conv-48-116, conv-48-188, conv-49-24, conv-49-144, conv-50-137, conv-50-147.

### B. Gold salah (8)

- **conv-49-80**: "When did Evan have a drunken night?" Gold: 9 Januari 2023. Jawaban: 9 Januari 2024. Alasan: sesi D24 bertanggal 10 Januari 2024 dan teksnya "Yesterday", jadi jawabannya benar.
- **conv-47-31**: "How many days did James plan to spend in Canada?" Gold: 19 hari. Jawaban: 9 hari. Alasan: berangkat "day after tomorrow" dari 9 Juli (berarti 11 Juli), pulang 20 Juli, jadi 9 hari benar.
- **conv-43-6**: "In which month's game did John achieve a career-high?" Gold: Juni 2023. Jawaban: Juli 2023. Alasan: sesi 16 Juli 2023 dan teksnya "Last week I scored 40", jadi Juli benar.
- Juga: conv-41-48 (gold salah ketik "3` July", padahal sesinya 31 Juli), conv-30-47 (giliran bukti menyebut wholesaler, bukan "spot"), conv-42-34 ("Little Women" adalah film), conv-48-77 (Phuket adalah retret, bukan tempat menyelam), dan conv-48-87 (gold "Thailand" padahal buktinya Bali di Indonesia).

### C. Hanya di gambar (4)

- **conv-42-92**: "What kind of lighting does Nate's gaming room have?" Gold: red and purple lighting. Jawaban: tidak tercatat. Alasan: caption BLIP hanya "a gaming room with a computer and a gaming chair".
- **conv-42-91**: "favorite book series about?" Gold: dragons. Alasan: teks hanya "adventures, magic", dan caption hanya "bunch of books".

Juga: conv-44-43 (peta trail) dan conv-47-33 (foto SD).

### D. Juri tidak konsisten (4)

- **conv-48-101**: "When did Jolene buy her pet snake?" Gold: "A year ago". Jawaban: "around January 2022". Alasan: sesi Januari 2023, dan aturan juri no. 4 menerima "last year" yang diubah menjadi tahun.
- **conv-42-24**: Gold: "weekend after 3 June". Jawaban: "weekend of June 18". Alasan: teksnya "two weekends later"; selisihnya ≤14 hari dan jawaban lebih tepat dari gold.
- **conv-41-63**: Gold: dua minggu. Jawaban: ≈9 hari. Alasan: masih dalam toleransi durasi ±50%.

Juga: conv-47-80.

### E. Open-domain spekulatif (15)

- **conv-43-51**: "What kind of yoga for building core strength might John benefit from?" Gold: Hatha Yoga. Alasan: tidak ada di percakapan; murni pengetahuan dunia.
- **conv-26-69**: "What personality traits might Melanie say Caroline has?" Gold: thoughtful, authentic, driven. Jawaban: empathetic, caring, courageous, yang didukung kutipan. Alasan: gold memilih kata yang sewenang-wenang.
- **conv-44-53**: Gold: park ranger. Jawaban: wildlife biologist. Alasan: keduanya masuk akal.

### F. Menolak menyebut nama atau menyimpulkan (7)

- **conv-47-16**: "the game with different colored cards". Gold: UNO. Jawaban: tidak menyebut nama. Teks penalaran: "No memory gives a name for this game, so I won't supply one".
- **conv-43-8**: "Which outdoor gear company likely signed up John?" Gold: Under Armour. Jawaban: "The memories don't name the company". Alasan: excerpt D3:15 "I've always liked Under Armour" ada di konteks.
- **conv-44-18**: "Where did Audrey get Pixie from?" Gold: breeder. Jawaban: menolak mengaitkan "found a breeder nearby" dengan Pixie.

Juga: conv-48-75, conv-43-66, conv-43-48, conv-50-62.

### G. Salah memilih atau membaca detail (13)

- **conv-30-73**: "How does Gina describe the feeling that dance brings?" Gold: magical. Jawaban: stress relief dan joy. Alasan: D15:8 "…it's magical!" ada di konteks.
- **conv-43-76**: Gold: MinaLima. Jawaban: "a photo of a collection of Harry Potter books". Alasan: penjawab memilih caption daripada pernyataan langsung.
- **conv-49-106**: "What did Evan start painting…?" Gold: forest scene. Jawaban: watercolors. Alasan: "forest scene" ada di caption D8:14 di konteks.

Juga: conv-26-85, conv-30-43, conv-41-85, conv-43-87, conv-43-152, conv-44-55, conv-47-53 (aritmetika: April→Oktober dijawab "sebulan"), conv-47-147, conv-48-115, conv-49-103.

### H. Hitung atau agregasi (6)

- **conv-42-79**: "How many screenplays has Joanna written?" Gold: 3. Jawaban: 4. Alasan: "started writing another" dan "wrapped up my second" adalah naskah yang sama.
- **conv-42-19**: Gold: 2 trail. Jawaban: 3. Alasan: trail air terjun di D14:19 baru rencana.
- **conv-50-47**: Gold: 2 car show. Jawaban: 4. Alasan: memasukkan kunjungan masa kecil; gold juga bisa diperdebatkan.

Juga: conv-41-62, conv-42-78, conv-42-52.

### I. Kata kategori vs nama spesifik (12)

- **conv-42-87**: "What state did Nate visit?" Gold: Florida. Jawaban: tidak ada negara bagian. Alasan: fakta Tampa punya keyword `florida` tetapi tidak punya `state`; peringkat fakta 95, potongan 77.
- **conv-47-6**: "Does James live in Connecticut?" Gold: likely yes. Jawaban: tidak. Alasan: "shelter in Stamford" ada di peringkat fakta 34 dan pulih dengan k=50.
- **conv-48-19**: "How many times has Jolene been to France?" Gold: 2. Jawaban: 1. Alasan: "a year ago in Paris" tidak berkaitan dengan "France", peringkat 104.

Juga: conv-26-59 (church vs religious), conv-42-60 (Xenoblade vs console), conv-43-11 (surfing vs sports), conv-43-25, conv-43-40 (Smoky Mountains vs NC/TN), conv-44-20 (chicken vs meat), conv-49-64, conv-50-56, conv-47-0.

### J. Nyaris lolos k (9)

- **conv-42-53**: "How many times has Nate taken his turtles on a walk?" Gold: 2. Jawaban: 1. Alasan: D5:4 ada di peringkat potongan 23, sedangkan batasnya 20.
- **conv-44-16**: "When is Andrew going to go hiking with Audrey?" Gold: August. Alasan: D11:7 ada di peringkat 38, sehingga penjawab memilih rencana November yang lain.
- **conv-49-43**: "How often does Sam get health checkups?" Alasan: fakta D7:2 di peringkat 45.

Juga: conv-26-66, conv-43-5, conv-43-17, conv-44-48, conv-44-59, conv-50-4.

### K. Parafrase atau ejaan (5)

- **conv-47-17** (abstain): "the board game where you have to find the imposter". Gold: Mafia. Alasan: teksnya "impostors", jadi stem tidak cocok; fakta di peringkat 59.
- **conv-50-40**: "artists Calvin used to listen to when he was a kid". Gold: Tupac dan Dr. Dre. Alasan: tidak ada kata "kid" atau "artist" di D20:8; peringkat 75.

Juga: conv-47-11, conv-47-24, conv-49-11.

### L. Ekstraksi melewatkan detail (11)

- **conv-42-8**: "When did Nate get his first two turtles?" Gold: 2019. Alasan: "I've had them for 3 years" tidak menjadi fakta, dan potongannya di peringkat 25.
- **conv-49-95**: "What does the smartwatch help Evan with?" Alasan: "smart watch" hanya ada di caption, tidak ada fakta, dan potongan di peringkat 139.
- **conv-42-14**: "What nickname does Nate use for Joanna?" Gold: Jo. Alasan: fakta dari D7:1 hanya "Nate dyed his hair purple", jadi nama panggilan hilang.

Juga: conv-30-44, conv-42-51, conv-43-49, conv-44-39, conv-49-25, conv-50-31, conv-50-61, conv-50-140.

## 5. Catatan dari sudut rubrik reviewer

- **RAG (rag-pipeline-reviewer):**
  - Retrieval hanya BM25 dan graph spreading, dengan seed excerpt dari fakta. Tidak ada reranker.
  - Context precision rendah (8,5%). Wajar untuk k=30 + 20, tetapi itu tanda konteks penuh noise.
  - Fallback "not enough context" justru dilarang oleh prompt resmi (Step 7), dan itu ikut mendorong klaster F ke arah sebaliknya: model abstain dengan cara lain.
  - Baseline dan threshold sudah ada di `docs/evaluation.md` §8. Yang belum ada: delta regresi yang diizinkan dan slice "gold bermasalah".
- **MLE (mle-reviewer):**
  - Juri sama dengan penjawab: risiko self-preference.
  - Ada contoh soal uji di prompt produk (kebocoran kecil).
  - Prompt jawab resmi tidak boleh disetel pada set uji yang sama tanpa held-out. Sarannya: setel di 3–4 percakapan, lalu laporkan di sisanya.
  - Konfigurasi retrieval (semantic off) tidak tercatat di report.
  - Noise label (≈30% dari kesalahan) lebih besar daripada gain tiap perbaikan, jadi errata dan juri independen perlu didahulukan.

Berkas kerja analisis (rekonstruksi konteks, peringkat, klaster) dibuat di scratchpad sesi dan tidak disimpan; tabel per soal ada di `diagnosis-haiku-full.csv` di folder proyek.
