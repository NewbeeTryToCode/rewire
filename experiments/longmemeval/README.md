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

Not run yet: `huggingface.co` is blocked in this environment.
