# LoCoMo, Nemotron 3 Ultra (free), 80 soal, 2026-10-14

Run dari runbook `docs/evaluation.md` §9, langkah 4.

- Perintah: `run.py --bench locomo --sample 20 --answer-model M --judge-model M --reasoning-effort low --answer-max-tokens 4000 --parallel 2 --label nemotron-s20`,
  M = `nvidia/nemotron-3-ultra-550b-a55b:free`. Seed default runner; 20 soal per kategori (stratified).
- Mode `episodes`, keyword (tanpa semantic), k=30, 10 potongan: sama dengan baseline retrieval §8.
- `--answer-max-tokens 4000` (bukan default 1500): pada smoke test 2 dari 8 jawaban terpotong karena
  reasoning menghabiskan batas, walaupun `--reasoning-effort low`. Di run ini masih 1 dari 80 terpotong.
- Biaya $0; ±5,1k prompt + 1,1k completion token per soal; jawaban p50 ≈27 s, p95 ≈99 s. Durasi total ≈25 menit.

## Catatan pembacaan

- **Dinilai ulang setelah perbaikan judge (commit 0987192).** Versi runner sebelumnya memberi judge 200 token
  dan menganggap balasan tanpa label sebagai CORRECT. Setelah `--judge-max-tokens 2000`, judge yang dulu
  terpotong dipanggil ulang: 0 panggilan judge terpotong, J tetap 88,8%.
- **J 88,8% lebih tinggi dari "semua bukti ada" (67,5%).** Dari 80 soal, 19 dinilai benar walaupun tidak
  semua bukti terambil (multi-hop 12, open-domain 6). Sebagian memang benar dari bukti parsial atau
  pengetahuan umum (open-domain), sebagian karena judge longgar: mis. "Where has Evan been on roadtrips with
  his family?" (gold: Rockies, Jasper) dijawab "Banff" dan dinilai benar. Penjawab dan judge model yang sama
  (§7): angka ini untuk regresi internal, bukan klaim publik.
- Interval kepercayaan 95% untuk n=80, p≈0,89: ±6,9 poin; per kategori (n=20) ±13–18 poin.
- Model kedua (`thinkingmachines/inkling:free`) tidak bisa dipakai: OpenRouter membalas 403, model itu hanya
  dibuka untuk "agentic harness" yang terdaftar, bukan panggilan API langsung.

## Report runner
answer model `nvidia/nemotron-3-ultra-550b-a55b:free` · judge model `nvidia/nemotron-3-ultra-550b-a55b:free`

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| multi-hop | 20 | 90.0% | 0.208 | 0.135 | 65.6% | 35.0% | 2,640 / 3,355 | 4,304 / 1,006 | 1 / 6 |
| open-domain | 20 | 80.0% | 0.204 | 0.152 | 47.4% | 50.0% | 2,514 / 3,048 | 4,188 / 1,170 | 1 / 2 |
| single-hop | 20 | 95.0% | 0.301 | 0.214 | 95.5% | 95.0% | 2,626 / 3,134 | 4,295 / 689 | 1 / 2 |
| temporal | 20 | 90.0% | 0.216 | 0.151 | 90.9% | 90.0% | 2,443 / 2,916 | 4,139 / 1,094 | 1 / 2 |
| overall | 80 | 88.8% | 0.232 | 0.163 | 66.7% | 67.5% | 2,556 / 3,120 | 4,232 / 990 | 1 / 3 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| multi-hop | 0.796 | 25.5% | - | 7.4× | 6 | 32 |
| open-domain | 0.560 | 14.0% | - | 7.3× | 2 | 66 |
| single-hop | 0.771 | 12.5% | - | 6.8× | 2 | 29 |
| temporal | 0.717 | 10.5% | - | 7.3× | 2 | 150 |
| overall | 0.711 | 15.6% | - | 7.2× | 6 | 69 |

WARNING: 0 empty and 1 truncated answers (max_tokens reached): raise --answer-max-tokens or pass --reasoning-effort low.

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|
| judge | 4 | 7,691 | 2,803 | 2,723 | 1,923 / 701 | 0.0000 | 44075 / 46192 |

Per question, answer + judge, including cached replies: 5,078 prompt tokens, 1,134 completion tokens, $0.00000.

Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
