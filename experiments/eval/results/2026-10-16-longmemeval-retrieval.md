# longmemeval · mode episodes · k=30 · episodes=10
retrieval only: no LLM (answer and judge models unused)

| category | n | J | F1 | BLEU-1 | evidence turns | all evidence | ctx tokens (mean / p95) | answer tokens in / out | retrieval ms p50 / p95 |
|---|---|---|---|---|---|---|---|---|---|
| knowledge-update | 78 | - | - | - | 97.2% | 95.8% | 8,851 / 10,986 | - / - | 115 / 170 |
| multi-session | 133 | - | - | - | 79.6% | 78.5% | 8,955 / 11,203 | - / - | 104 / 154 |
| single-session-assistant | 56 | - | - | - | 98.2% | 100.0% | 8,555 / 10,254 | - / - | 128 / 182 |
| single-session-preference | 30 | - | - | - | 75.0% | 76.7% | 9,520 / 11,218 | - / - | 105 / 153 |
| single-session-user | 70 | - | - | - | 100.0% | 100.0% | 7,539 / 9,999 | - / - | 81 / 128 |
| temporal-reasoning | 133 | - | - | - | 84.2% | 75.6% | 9,379 / 11,219 | - / - | 113 / 171 |
| overall | 500 | - | - | - | 86.2% | 85.7% | 8,842 / 11,052 | - / - | 108 / 167 |

| category | MRR (first evidence excerpt) | context precision | evidence session found | compression vs full history | retrieval ms p99 | answer words |
|---|---|---|---|---|---|---|
| knowledge-update | 0.951 | 32.2% | 98.6% | 13.8× | 188 | - |
| multi-session | 0.844 | 32.1% | 99.2% | 13.7× | 177 | - |
| single-session-assistant | 0.982 | 13.6% | 100.0% | 14.3× | 212 | - |
| single-session-preference | 0.517 | 11.8% | 76.7% | 12.9× | 166 | - |
| single-session-user | 0.943 | 18.5% | 100.0% | 16.2× | 146 | - |
| temporal-reasoning | 0.798 | 25.6% | 96.1% | 13.0× | 183 | - |
| overall | 0.857 | 25.0% | 97.0% | 13.8× | 183 | - |

## LLM usage this run (cached replies cost nothing)

| stage | calls | prompt tokens | completion tokens | reasoning tokens | tokens/call in / out | cost USD | latency p50 / p95 ms |
|---|---|---|---|---|---|---|---|


Published scores of other memory systems, and why they are not directly comparable: docs/evaluation.md.
