# LongMemEval: rewire

Harness: [`run.py`](run.py) · Official prompts: [`lme_prompts.py`](lme_prompts.py) (copied verbatim from [xiaowu0162/LongMemEval](https://github.com/xiaowu0162/LongMemEval))

## Why LongMemEval after LoCoMo

LoCoMo saturated at ~94% for rewire, and most remaining errors are answerer errors ([../locomo/README.md](../locomo/README.md)). LongMemEval ([ICLR 2025](https://arxiv.org/abs/2410.10813)) tests what LoCoMo does not:

| Type | What it tests | Relevant rewire mechanism |
|---|---|---|
| knowledge-update | a fact changed; the answer is the newest one | `supersede`, history intent |
| temporal-reasoning | dates and intervals relative to the question date | `valid_from`, time window |
| multi-session | aggregating across sessions | fact → source excerpt, graph |
| single-session-assistant | what the **assistant** said | transcripts keep assistant turns (truncated to 1,500 chars) |
| single-session-preference | personalising an answer | preference type, `general` profile |
| abstention (`_abs`) | knowing what was never said | answer prompt |

## Data

`longmemeval_s_cleaned.json` (500 questions, ~50 sessions ≈ 115k tokens per question). Download from
[HuggingFace](https://huggingface.co/datasets/xiaowu0162/longmemeval-cleaned). The cloud environment must allow
`huggingface.co`.

## Modes and costs

Each question has its own history, so each question gets its own brain.

| Mode | Ingest | LLM calls | Scope |
|---|---|---|---|
| `--retrieval-only` | transcripts only | 0 | all 500: evidence session / turn recall |
| `--mode episodes` | transcripts only | ~125 answer + ~25 judge | all 500 |
| `--mode full --per-type N` | full consolidation (extract + judge) | ~2 per session (~100 per question) | subset |

```bash
python experiments/longmemeval/run.py --data longmemeval_s_cleaned.json --out OUT --retrieval-only
python experiments/longmemeval/run.py --data longmemeval_s_cleaned.json --out OUT --mode episodes --label ep
python experiments/longmemeval/run.py --data longmemeval_s_cleaned.json --out OUT --mode full --per-type 5 --label full5
```

Deviation from the official setup: several questions per answer/judge call (as in the LoCoMo harness), and Claude as the judge instead of GPT-4o.

## Results

### Episodes mode: all 500 questions, no LLM extraction

Transcripts only (what consolidation stores in `sessions/`), hybrid recall (BM25 + WordLlama), 10 excerpts ≈ 11k tokens of
context per question instead of the full ~115k-token history. Answers and judge: Claude, official prompts.

| Type | n | Evidence session found | All evidence sessions | **Accuracy** |
|---|---|---|---|---|
| single-session-user | 64 | 96.9% | 96.9% | **96.9%** |
| single-session-assistant | 56 | 100% | 100% | **92.9%** |
| single-session-preference | 30 | 86.7% | 86.7% | **83.3%** |
| multi-session | 121 | 99.2% | 82.6% | **73.6%** |
| knowledge-update | 72 | 100% | 94.4% | **90.3%** |
| temporal-reasoning | 127 | 94.5% | 79.5% | **82.7%** |
| abstention | 30 | – | – | **96.7%** |
| **overall** | 500 | 97.0% | 87.9% | **85.4%** (task-averaged 86.6%) |

Of the 73 wrong answers: 29 had all evidence sessions in context, 31 only some, 12 none (abstention excluded).
The largest group is multi-session counting ("how many X have I…"): with only some of the evidence sessions
retrieved the answer **undercounts**. (The opposite of LoCoMo, where extra excerpts made answers overcount the
same event mentioned in several sessions.)

### Published numbers (not re-run here; setups differ)

| System | LongMemEval_S | Notes |
|---|---|---|
| Zep | 71.2% | GPT-4o; the most repeated third-party figure |
| Emergence (RAG) | 86% | self-reported |
| Mem0 | 93.4% | self-reported |
| Mastra Observational Memory | ~95% | self-reported |
| Supermemory | 97% | Recall@20 retrieval metric, not QA accuracy |

Judge prompts alone move scores by ~10% (Mastra); our answer and judge model is Claude, not GPT-4o.
