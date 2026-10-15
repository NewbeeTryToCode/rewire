# A/B multi-hop LoCoMo (282 soal)

Diperbarui: 2026-10-15 15:55 UTC

## 1. Pola: kenapa bukti multi-hop tidak terambil

- Makin banyak sesi yang memuat bukti, makin jarang semua bukti terambil:

| sesi bukti per soal | n | semua bukti terambil |
|---|---|---|
| 1 | 13 | 69% |
| 2 | 164 | 66% |
| 3 | 56 | 52% |
| 4 | 28 | 39% |
| 5+ | 21 | 10% |

- Dari 220 giliran bukti yang hilang, potongan transkripnya ada di peringkat 21–100 (171 giliran), bukan hilang total. Hanya 36 yang tidak masuk 100 besar sama sekali.
- Penyebab utamanya: pertanyaan memakai **kata kategori** ("desserts", "cities", "activities", "locations"), sedangkan pesannya menyebut **contoh spesifik** ("coconut milk ice cream", "San Francisco", "biking", "Bali"). Pencarian kata (BM25) tidak bisa menghubungkan keduanya, dan nama orang muncul di hampir semua pesan sehingga tidak membantu.

## 2. Uji retrieval tanpa LLM (bukti lengkap, potongan atau fakta)

Catatan: dua baris ekspansi di tabel ini terukur saat pencarian semantik masih ikut aktif; angka bersih (tanpa semantik) ada di bagian 3.

| varian | bukti lengkap | token konteks rata-rata |
|---|---|---|
| baseline (k=30 fakta, 20 potongan) | 56,4% | 5.133 |
| + pencarian semantik (WordLlama) | 53,9% | 4.923 |
| k=50 fakta | 56,4% | 5.965 |
| 30 potongan | 61,3% | 6.795 |
| 40 potongan | 63,8% | 8.339 |
| 50 potongan | 67,0% | 9.865 |
| **ekspansi kueri** (Haiku menulis 4 kueri kata kunci spesifik, hasil digabung) | **65,6%** | 6.338 |
| **ekspansi kueri + 30 potongan** | **73,4%** | 8.956 |

- Pencarian semantik bawaan justru menurunkan hasil, jadi tidak dipakai.
- Menambah fakta (k) tidak membantu; yang membantu adalah potongan transkrip.
- Ekspansi kueri paling efisien: +9 poin dengan konteks +23%, setara 45 potongan tanpa ekspansi.

## 3. A/B dengan Haiku (jawab + juri + juri daftar), 282 soal multi-hop

| metrik | A: baseline | B: ekspansi kueri | selisih |
|---|---|---|---|
| J (juri resmi) | 90,8% | 92,6% | +1,8 |
| J ketat (daftar harus lengkap) | 77,3% | **81,6%** | **+4,3** |
| jawaban daftar lengkap | 66,1% | 73,4% | +7,3 |
| item daftar yang disebut | 82,3% | 87,4% | +5,1 |
| bukti lengkap (potongan atau fakta) | 56,4% | 70,9% | +14,5 |
| token konteks rata-rata | 5.133 | 7.010 | +37% |

- Uji berpasangan pada soal yang sama: J ketat naik di 20 soal dan turun di 8 (z = 2,27, signifikan pada 5%). Untuk J resmi, naik 10 dan turun 5 (z = 1,29, belum signifikan).
- Contoh yang jadi benar: "What martial arts has John done?" (Kickboxing, Taekwondo), "What European countries has Maria been to?" (Spain, England), "What exercises has John done?" (4 item).
- Contoh yang jadi salah: jawaban yang terlalu banyak (3 surat padahal gold 2), atau satu item hilang ("Running, pottery" hanya menyebut running).

## 4. Perbaikan yang diusulkan

1. **Recall multi-kueri di rewire.** Alat `recall` menerima beberapa kueri sekaligus, lalu hasilnya digabung dengan RRF. Model pemanggil (agen yang memakai rewire lewat MCP) sudah LLM, jadi dia yang menulis kueri tambahan berisi contoh spesifik. Rewire sendiri tidak perlu panggilan LLM tambahan. Deskripsi alat MCP diberi petunjuk: untuk pertanyaan daftar atau kategori, tambahkan kueri yang menyebut contoh spesifiknya. Perlu perubahan Python dan Rust yang identik.
2. **Opsional, setelah itu:** tambah potongan ke 30 hanya untuk pertanyaan daftar (kata "what ... has", "which", "how many"), karena di uji retrieval ini menambah bukti lengkap tanpa membebani soal lain.
3. **Tidak disarankan:** pencarian semantik bawaan (WordLlama) dan menaikkan jumlah fakta (k=50). Keduanya tidak menambah bukti.

## 5. Sudah diimplementasikan (PR #2)

- `recall` menerima `also`: hingga 4 kueri tambahan dari model pemanggil, digabung per peringkat (MCP `also`, CLI `--also`). Tanpa `also`, hasilnya sama persis seperti sebelumnya.
- Python dan Rust identik: 52 soal dengan kueri tambahan dan 152 tanpa, urutan hasil sama 100%.
- Uji ulang lewat jalur produk (bukan prototipe): J 92,9%, J ketat 81,6%, bukti lengkap 71,3%, sama dengan arm B.
- Harness eval: `--expand-model` dan `--categories`.

## 6. Cek kategori lain: 80 soal (20 per kategori), Haiku

| kategori | J A | J B | J ketat A | J ketat B | bukti lengkap A | bukti lengkap B | token konteks A → B |
|---|---|---|---|---|---|---|---|
| multi-hop | 95,0% | 100,0% | 90,0% | 95,0% | 70,0% | 95,0% | 5.168 → 7.057 |
| open-domain | 75,0% | 85,0% | 75,0% | 85,0% | 55,0% | 65,0% | 5.122 → 6.992 |
| single-hop | 100,0% | 100,0% | 95,0% | 100,0% | 95,0% | 100,0% | 5.276 → 6.953 |
| temporal | 95,0% | 95,0% | 95,0% | 95,0% | 90,0% | 90,0% | 5.044 → 6.651 |
| overall | 91,2% | 95,0% | 88,8% | 93,8% | 77,5% | 87,5% | 5.152 → 6.913 |

Soal yang berubah: 4 jadi benar (conv-48-19 multi-hop, conv-43-34 dan conv-26-50 open-domain, conv-42-24 temporal) dan 1 jadi salah (conv-44-34 temporal, menghitung bulan). Tidak ada kategori yang turun. Dengan 20 soal per kategori, selisih per kategori masih bisa noise (±20 poin); arahnya konsisten dengan uji 282 soal multi-hop.
