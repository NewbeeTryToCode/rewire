# Analisis: LoCoMo 80 soal, Nemotron 3 Ultra (free)

Hasil run: [2026-10-14-locomo-nemotron-s20.md](2026-10-14-locomo-nemotron-s20.md). J 88,8%, F1 0,232, BLEU-1 0,163,
semua bukti ada di konteks untuk 67,5% soal. Analisis ini tanpa LLM: retrieval 80 soal yang sama direproduksi
persis (seed sama, 67,5%), lalu diubah satu variabel per kali.

## 1. Mana yang sebenarnya rendah

| Angka | Nilai | Bacaan |
|---|---|---|
| J | 88,8% (±6,9 poin, n=80) | Tidak rendah; dekat 94% iterasi 6–7 (Claude, mode `full`, k=50). Penjawab = judge, jadi cenderung longgar (contoh "Banff" untuk gold "Rockies, Jasper" dinilai benar). |
| F1 / BLEU-1 | 0,23 / 0,16 | Rendah. Jawaban rata-rata 69 kata (temporal 150 kata) sedangkan gold 1–5 kata. Penyebab terbesar: parser `ANSWER:` peka huruf besar dan markdown, jadi `**Answer:**` atau `Final answer:` membuat seluruh penalaran ikut dihitung sebagai jawaban. Diperbaiki (lihat §3). |
| Semua bukti ada | 67,5% | Rendah untuk multi-hop (35%) dan open-domain (50%). Inilah batas atas yang sesungguhnya. |

## 2. Kenapa retrieval multi-hop / open-domain gagal

26 dari 80 soal buktinya tidak lengkap: multi-hop 13, open-domain 10, temporal 2, single-hop 1. Polanya:

- **Soal agregasi / daftar.** "What causes has John done events for?", "What people has Maria met...?",
  "What writing classes has Maria taken?" perlu 3–5 giliran dari sesi berbeda. Dengan 10 potongan, potongan
  teratas sering berasal dari satu sesi yang sama.
- **Kesenjangan kata (lexical gap) dan pengetahuan dunia.** "France" vs bukti "Paris"; "roadtrips" vs "a trip
  with my family"; "stress reliever" vs "yoga ... sunsets". BM25 tidak bisa menjembatani ini.
- **Bukti tanpa nama entitas.** Giliran bukti sering tidak menyebut nama orangnya ("I took a dog grooming
  course"); yang menyebut nama ada di giliran lain.
- **Noise.** Context precision 15,6%: satu potongan berisi 6 giliran, jadi sebagian besar konteks tidak relevan.

## 3. Ablasi (80 soal yang sama, tanpa LLM; persen soal dengan semua bukti di konteks)

| Konfigurasi | single | temporal | multi-hop | open | **overall** | konteks |
|---|---|---|---|---|---|---|
| Run ini: episodes, keyword, k=30, 10 potongan | 95 | 90 | 35 | 50 | **67,5** | 2,5k |
| + 20 potongan | 95 | 95 | 65 | 60 | **78,8** | 4,9k |
| + semantic (WordLlama), 10 potongan | 90 | 90 | 60 | 50 | **72,5** | 2,3k |
| + semantic, 20 potongan | 95 | 95 | 65 | 65 | **80,0** | 4,5k |
| mode `full` (fakta L2), k=30, 10 potongan | 95 | 90 | 50 | 50 | **71,2** | 3,3k |
| mode `full`, k=50, 20 potongan | 95 | 95 | 65 | 65 | **80,0** | 6,0k |

n=20 per kategori, jadi selisih per kategori di bawah ±15 poin belum bermakna. Untuk mode `full`, metrik ini
hanya menghitung potongan transcript; fakta yang sudah memuat informasinya (mis. "Jolene bought a pendant in
Paris") tidak ikut dihitung, jadi manfaat `full` di sini terlalu rendah.

Kesimpulan: **jumlah potongan adalah tuas termurah** (multi-hop 35 → 65%) dan konteksnya tetap ≈5× lebih kecil
dari riwayat penuh (≈26k token). Semantic membantu bila potongannya sedikit. Run ini juga tidak menguji fitur
inti rewire (konsolidasi menjadi fakta + kata kunci lintas bahasa), karena memakai mode `episodes`.

## 4. Yang dilakukan sistem lain (riset)

- **Hindsight** (91,4% LongMemEval, 89,6% LoCoMo menurut paper-nya): retrieval multi-strategi (vektor + BM25 +
  graph entitas + filter waktu) digabung RRF lalu di-rerank cross-encoder; saat menyimpan, entitas
  di-resolve dan dihubungkan. Keunggulannya paling besar di multi-session dan open-domain.
- **Mastra observational memory** (94,9% dengan GPT-5-mini, 84,2% dengan GPT-4o): agen Observer menulis
  observasi ringkas terus-menerus, Reflector menggabungkannya; riwayat mentah diganti log observasi.
- **T-Mem**: rute "bridge" khusus untuk multi-hop yang menarik bukti lintas adegan, terpisah dari rute entitas.
- **Second-hop retrieval** (usulan PRISM): untuk soal daftar dan multi-hop, query lanjutan dari hasil pertama,
  digabung RRF. Motivasinya sama dengan temuan kita: bukti multi-hop berperingkat rendah.

## 5. Rekomendasi, diurutkan dari biaya terendah

1. **Perbaiki parser jawaban** (selesai): F1/BLEU sekarang mengukur jawaban, bukan penalaran.
2. **Potongan adaptif:** 20 potongan untuk soal daftar/agregasi/multi-hop ("what … has X", "how many", bentuk
   jamak), 10 untuk sisanya. rewire sudah punya intent hitungan; perluas ke soal daftar.
3. **Judge berbeda dari penjawab** untuk J yang kredibel (model murah lain), dan kalibrasi pada ±100 soal.
4. **Second-hop berbasis entitas:** ambil nama dan kata benda dari hasil pertama, cari ulang, gabung RRF.
   Menjembatani "Evan" → giliran tanpa nama, dan sebagian kesenjangan kata.
5. **Potongan lebih kecil + perluasan tetangga:** giliran tunggal sebagai unit ranking, lalu ±1 giliran di
   sekitarnya: precision naik, lebih banyak bukti berbeda dalam anggaran yang sama.
6. **Evaluasi mode `full`** (fakta + kata kunci lintas bahasa dari konsolidasi), karena itu cara rewire dipakai
   sebenarnya. Dengan model gratis: ±2 panggilan × 272 sesi LoCoMo; batas harian jadi kendala.
7. Lebih jauh: indeks entitas (siapa → giliran mana) dan reranker; keduanya terbukti di Hindsight.

Sumber: [Hindsight (arXiv 2512.12818)](https://arxiv.org/html/2512.12818v1) ·
[Mastra observational memory](https://mastra.ai/research/observational-memory) ·
[T-Mem (arXiv 2606.15405)](https://arxiv.org/pdf/2606.15405) ·
[PRISM issue #94: second-hop retrieval](https://github.com/dwamianm/prism/issues/94) ·
[QueryLink (ACL Findings 2026)](https://aclanthology.org/2026.findings-acl.765.pdf) ·
[LoCoMo overview](https://www.emergentmind.com/topics/locomo).
