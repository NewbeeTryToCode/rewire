# BEAM 100K, Nemotron 3 Ultra (free), 200 soal, 2026-10-14/15

Run dari runbook `docs/evaluation.md` §9, langkah 4.

- Perintah: `run.py --bench beam --beam-sizes 100K --sample 20 --answer-model M --judge-model M --reasoning-effort low --answer-max-tokens 4000 --parallel 2 --label nemotron-100k-s20`,
  M = `nvidia/nemotron-3-ultra-550b-a55b:free`. Seed default runner; 20 soal per kategori, 10 kategori.
- Data: repo BEAM commit `b2da22e`, `chats/100K` (20 chat). Mode `episodes` (tanpa ringkasan L1 dan tanpa node L2),
  keyword, k=30, 10 potongan.
- Biaya $0; ±17k prompt + 2,3k completion token per soal (judge ≈3 panggilan per soal, satu per butir rubrik).
- Dijalankan dalam tiga putaran karena batas harian model gratis (1.000 request/hari) dan 4 soal gagal karena
  `Connection reset`; cache melanjutkan. Putaran terakhir memakai `--judge-max-tokens 4000` untuk 6 judge yang
  masih terpotong di 2000: report akhir tanpa judge terpotong.

## Catatan pembacaan

- **Angka pertama (9,9%) tidak valid**: judge diberi 200 token dan habis untuk reasoning sebelum memberi skor,
  sehingga hampir semua butir rubrik bernilai 0. Diperbaiki di commit 0987192; angka di bawah sudah dinilai ulang.
- **Knowledge update (27,5%) dan temporal reasoning (36,2%) rendah walaupun semua bukti ada di 80% soal.** Dari 16
  soal knowledge update dengan bukti lengkap, 12 tidak mendapat skor penuh; polanya: model memilih nilai lama
  (mis. "overtime terbaru" gold 4 jam, dijawab 5; "kerja remote" gold 3 hari, dijawab 2). Potongan diurutkan
  menurut relevansi, bukan waktu, jadi nilai lama dan baru tampil tanpa urutan yang jelas. Sasaran perbaikan:
  urutan kronologis dan pengelompokan per sesi saat recall.
- **Instruction following (65%) dan preference following (68,8%) tinggi walaupun bukti jarang lengkap**
  (20% / 60%): rubriknya sering bisa dipenuhi oleh jawaban yang wajar tanpa bukti spesifik.
- **Summarization (36,7%) diukur tanpa ringkasan sesi**: mode `episodes` tidak menulis ringkasan L1.
- **Event ordering (47,1%)** dinilai dengan rubrik saja; Kendall tau belum dihitung.
- Penjawab dan judge model yang sama: untuk regresi internal, bukan klaim publik. Interval kepercayaan 95% untuk
  n=200, p≈0,49: ±6,9 poin; per kategori (n=20) ±22 poin.

## Report runner

answer model `nvidia/nemotron-3-ultra-550b-a55b:free` · judge model `nvidia/nemotron-3-ultra-550b-a55b:free`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| abstention | 20 | 70.0% | - | - | - | - | 14,546 / 21,749 | 14,154 / 424 | 2 / 10 |
| contradiction_resolution | 20 | 22.5% | - | - | 79.2% | 60.0% | 13,282 / 17,438 | 12,580 / 673 | 1 / 2 |
| event_ordering | 20 | 47.1% | - | - | 39.5% | 5.0% | 15,478 / 25,999 | 14,712 / 2,241 | 2 / 4 |
| information_extraction | 20 | 64.6% | - | - | 65.9% | 55.0% | 14,020 / 22,789 | 13,250 / 456 | 2 / 2 |
| instruction_following | 20 | 65.0% | - | - | 17.4% | 20.0% | 14,343 / 23,283 | 13,555 / 659 | 2 / 3 |
| knowledge_update | 20 | 27.5% | - | - | 86.7% | 80.0% | 13,816 / 19,608 | 13,114 / 502 | 1 / 3 |
| multi_session_reasoning | 20 | 51.6% | - | - | 60.2% | 25.0% | 14,280 / 21,016 | 13,677 / 1,090 | 2 / 3 |
| preference_following | 20 | 68.8% | - | - | 63.0% | 60.0% | 15,253 / 26,844 | 14,735 / 937 | 2 / 4 |
| summarization | 20 | 36.7% | - | - | 24.1% | 0.0% | 14,871 / 21,319 | 14,356 / 1,035 | 2 / 4 |
| temporal_reasoning | 20 | 36.2% | - | - | 90.0% | 80.0% | 13,660 / 18,552 | 13,306 / 634 | 2 / 2 |
| overall | 200 | 49.0% | - | - | 51.5% | 43.3% | 14,355 / 21,319 | 13,744 / 865 | 2 / 3 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| abstention | - | - | - | 9.6× | 10 | 36 |
| contradiction_resolution | 0.760 | 23.5% | - | 12.0× | 2 | 22 |
| event_ordering | 0.359 | 21.0% | - | 10.2× | 4 | 287 |
| information_extraction | 0.422 | 7.0% | - | 11.3× | 2 | 43 |
| instruction_following | 0.087 | 2.5% | - | 10.8× | 3 | 47 |
| knowledge_update | 0.681 | 21.0% | - | 11.5× | 3 | 3.650 |
| multi_session_reasoning | 0.585 | 26.0% | - | 10.9× | 3 | 172 |
| preference_following | 0.325 | 9.0% | - | 9.9× | 4 | 235 |
| summarization | 0.406 | 18.3% | - | 10.0× | 4 | 179 |
| temporal_reasoning | 0.739 | 22.5% | - | 11.1× | 2 | 3.000 |
| overall | 0.486 | 16.7% | - | 10.7× | 10 | 103 |

Abstention accuracy: 70.0%

WARNING: 1 empty and 4 truncated answers (max_tokens reached): raise --answer-max-tokens or pass --reasoning-effort low.

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| judge | 17 | 23,416 | 18,802 | 20,449 | 1,377 / 1,106 | 0.0000 | 20439 / 87640 |

Per question, answer + judge, including cached replies: 17,016 prompt tokens, 2,359 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
