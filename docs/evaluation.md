# Evaluating rewire

This document defines **what we measure, how, and at what cost**, and how rewire's numbers compare with
other memory systems. The runner is [`experiments/eval/run.py`](../experiments/eval/run.py)
(OpenRouter, or any other OpenAI-compatible endpoint).

## 1. Two levels of evaluation

| Level | When | LLM | Cost | Question answered |
|---|---|---|---|---|
| **Retrieval regression** | every recall/index change | none | 0 tokens | Is the evidence for the answer retrieved, and how large is the context? |
| **End-to-end score (J)** | milestones, or before claiming a public number | answerer + judge (+ ingest with `--mode full`) | see §5 | Is the final answer correct, and can it be compared with other systems? |

Public scores (Mem0, Zep, etc.) are always **end-to-end**: retrieval → an LLM answers → an LLM judge grades. Retrieval
metrics have no equivalent on leaderboards, but they are free and stable, so they are used day to day.

## 2. Benchmarks used

| Benchmark | Contents | What is graded | Our protocol |
|---|---|---|---|
| **LoCoMo** | 10 long conversations (~27 sessions), 1,540 questions in categories 1–4 | single-hop, multi-hop, temporal, open-domain | Mem0's official answer prompt + judge (`experiments/eval/locomo_prompts.py`); category 5 (adversarial) is excluded, as Mem0 does |
| **LongMemEval_S** | 500 questions, each with a history of ~50 sessions (~115k tokens) | single-session (user / assistant / preference), multi-session, knowledge-update, temporal-reasoning, abstention (30 `_abs` questions) | Official answer template + official per-type judge prompt (`experiments/eval/lme_prompts.py`) |
| **BEAM** (ICLR 2026) | 100 chats (100K: 20, 500K: 35, 1M: 35, 10M: 10), 2,000 questions, 10 categories × 2 questions per chat | abstention, contradiction resolution, event ordering, information extraction, instruction following, knowledge update, multi-session reasoning, preference following, summarization, temporal reasoning | RAG answer prompt + per-rubric-item judge (1 / 0.5 / 0), copied verbatim (`experiments/eval/beam_prompts.py`, MIT); question score = mean of the rubric. The 10M size is not supported yet; Kendall tau for event ordering is not computed yet (rubric score only) |
| **PersonaMem** (COLM 2025) | 180+ persona histories, up to 60 sessions; 32k / 128k / 1M context versions | 7 question types, e.g. recalling user facts, following preference changes, new recommendations that fit the preferences | Multiple choice (a–d) with the official instructions and answer matching: **no LLM judge**. One brain per shared context; excerpts after the question's position are dropped so memory does not contain "the future" |

Ingest modes:
- `episodes`: transcripts are stored and indexed exactly as consolidation would produce them, **without** LLM extraction. Free.
- `full`: every session is consolidated by an LLM (`--ingest-model`), and the whole inbox is accepted. Most realistic, most expensive.
  BEAM and PersonaMem support only `episodes` (consolidating a 1M-token chat is too expensive; for PersonaMem,
  extracted facts could leak turns that come after the question).

Data sources:
- LoCoMo: `locomo10.json`, straight from GitHub:
  `https://raw.githubusercontent.com/snap-research/locomo/main/data/locomo10.json`.
- LongMemEval_S: `longmemeval_s_cleaned.json` (Hugging Face `xiaowu0162/longmemeval-cleaned`). When the proxy allows
  `huggingface.co` but blocks the file CDN (`us.aws.cdn.hf.co`, `*.xethub.hf.co`), fetch the rows through
  `datasets-server.huggingface.co/rows` from the mirror `lorenoamori/longmemeval` (10 rows per request, sort by `row_idx`);
  validated 2026-10-16: 500 questions, 30 `_abs`, 896 `has_answer` turns, per-type counts as in the paper.
- BEAM: the `chats/` folder of the repo `github.com/mohammadtavakoli78/BEAM` (cloned with git; validated at
  commit `b2da22e`). Validated:
  100K/500K/1M = 20/35/35 chats with 400/700/700 questions (20 per chat, 10 categories), chat sizes ≈146k / 538k /
  1.09M tokens (median), matching the paper; the official `download_dataset.py` script converts the Hugging Face data to the
  same format. A byte comparison with the Hugging Face copy has not been done yet (HF is blocked).
- PersonaMem: Hugging Face `bowen-upenn/PersonaMem-v1` (the old name `bowen-upenn/PersonaMem` redirects there; earlier copied to GCS, `bucket_for_claude`):
  `questions_32k.csv`, `shared_contexts_32k.jsonl`, `questions_1M.csv`, `shared_contexts_1M.jsonl`.
  Validated: 32k = 589 questions, 37 shared contexts, ≈27k tokens per context; 1M = 2,674 questions, 33 contexts,
  ≈925k tokens per context (median); 15 columns as described in the README, every context reference exists, and the answer keys
  are spread evenly over (a)–(d). Question type names differ between 32k and 1M (e.g. `track_full_preference_evolution`
  vs `track_full_preference_updates`; 1M has `acknowledge_latest_user_preferences`), so per-type scores for
  32k and 1M are not compared directly.

## 3. Metrics

| Metric | Definition | LLM? | Reported by |
|---|---|---|---|
| **J (accuracy)** | Percentage of answers the judge grades as correct with the official prompt (BEAM: mean rubric score 0–1; PersonaMem: multiple-choice accuracy without a judge) | yes (PersonaMem: not for grading) | run.py |
| **F1** (LoCoMo) | Token F1 between the answer and the gold answer (the original LoCoMo paper metric) | answer only | run.py |
| **BLEU-1** (LoCoMo) | Unigram precision with a brevity penalty | answer only | run.py |
| **Abstention** (LongMemEval) | Percentage of unanswerable questions recognized as unanswerable | yes | run.py |
| **Evidence turn recall** | Percentage of evidence turns that appear in the recalled context | no | run.py, `--retrieval-only` |
| **All evidence** | Percentage of questions for which **all** evidence is in the context | no | run.py |
| **MRR** | 1 / rank of the first excerpt containing evidence (0 if none), averaged: whether evidence is near the top or buried | no | run.py |
| **Context precision** | Percentage of retrieved excerpts that contain evidence; the rest is *noise* that pollutes the context (noise ratio = 1 − precision) | no | run.py |
| **Context tokens** | Size of the memory context per question (mean, median, p95), counted with the GPT-4o tokenizer (o200k) when available, else characters/4 (reports before 2026-10-20 used characters/4; o200k counts are 3–28% lower on these benchmarks). Billed answer prompt tokens are reported separately per model | no | run.py |
| **Compression ratio** | Full-history tokens ÷ rewire context tokens: how many times cheaper than sending the whole history (*full-context*) | no | run.py |
| **Prompt / completion / reasoning tokens per response** | From the API `usage`, per stage (ingest / answer / judge) | yes | run.py |
| **Cost (USD)** | `usage.cost` from OpenRouter, per stage and per question | yes | run.py |
| **Latency** | Recall (ms, p50/p95/p99): CLI wall time minus the process start measured on an empty brain; a warm MCP server, one call at a time: `latency.py`. LLM calls (p50/p95), and end to end per question (extra queries + recall + answer) | — | run.py, latency.py |
| **Full-context baseline** | `--mode fullctx`: the whole history up to the question in place of recalled memory, same answer and judge models | yes | run.py |
| **Ingest cost** (`--mode full`) | LLM calls, prompt/completion tokens and USD per session (logged by `llm_cmd.py`), seconds per session p50/p95, active nodes by type | yes | run.py |
| **Event ordering** (BEAM) | (Kendall tau-b + 1) / 2 × F1 after the judge aligns the answer's items to the reference order, next to the rubric score | yes | run.py |
| **J without disputed gold** (LoCoMo) | J after dropping the questions in `experiments/eval/locomo_errata.json`: speaker swapped in the question, gold contradicts the conversation, or the answer exists only in a photo. The official J remains the number that is compared | yes | run.py |
| **Strict J / list items named** (LoCoMo) | For questions whose gold is a list, a list judge (same model as the judge) splits the gold and counts the items the answer mentions, paraphrases included. Strict J: a list answer is correct only if the judge accepts it **and** every item is mentioned | yes | run.py |
| **Answer length** | Mean words per answer (long answers tend to be graded correct by a lenient judge) | — | results JSON |

Retrieval results measured so far are in §8. Context precision is low (≈11–20%) because one excerpt contains
6 turns: this is a clear target for improvement (smaller excerpts or trimming) without lowering recall.

### Capability dimensions → benchmark categories

| Dimension | Measured by |
|---|---|
| Fact recall (*needle-in-a-haystack*) | LoCoMo single-hop; LongMemEval single-session-user / -assistant |
| Belief update (an old fact replaced by a new one) | LongMemEval knowledge-update; supersede unit tests in the repo |
| Temporal reasoning | LoCoMo temporal; LongMemEval temporal-reasoning |
| Cross-session synthesis | LoCoMo multi-hop; LongMemEval multi-session |
| Preference consistency | LongMemEval single-session-preference; PersonaMem; BEAM preference following |
| Contradictions, event order, long-term instructions, summaries | BEAM contradiction resolution, event ordering, instruction following, summarization |
| Very long scale (500K–10M tokens) | BEAM |
| Knowing when it does not know | LongMemEval abstention |

Product metrics (not benchmarks; measured from real use): fresh-first rate (the newest value is used
when a fact changes), captured-but-not-surfaced (a fact is stored but does not surface when needed),
inbox items per session, snapshot hit rate (`memory used` in `sessions/*.md`).

## 4. Scores of other systems (published)

**Warning:** the numbers below are not comparable with one another. The answering model, judge model, prompt, and
subset all differ, and many are self-reported by the vendor. Compare only when the protocol
is the same (§6).

### LoCoMo (J, categories 1–4)

| System | J | Tokens / latency | Source, notes |
|---|---|---|---|
| Full-context (whole conversation in the prompt) | ≈72.9% | ≈26k tokens per conversation, p95 ≈17 s | Mem0 paper (2025) |
| Mem0 | ≈66.9% (Mem0ᵍ ≈68.4%) | ≈7k tokens, p95 ≈1.44 s | Mem0 paper (2025) |
| Zep | ≈66.0% | | Mem0 paper (2025); Zep self-reports 94.7%, a third party 75.1% |
| LangMem | ≈58.1% | latency ≈60 s | Mem0 paper (2025) |
| OpenAI Memory | ≈52.9% | | Mem0 paper (2025) |
| A-Mem | ≈48.4% | | Mem0 paper (2025) |
| Mem0 (2026 platform) | 92.5% | median 6.7–7k tokens per retrieval, p50 0.88–1.09 s | Vendor report |
| Memori evaluation (third party) | Zep 79.1% · LangMem 78.1% · Mem0 62.5% | Mem0 ≈1,764 tokens per question | Memori Labs |
| rewire (historical) | 94.0–94.9% | context ≈2.7k tokens per question (dry run conv-26, k=30, 10 excerpts) | Iterations 6–7: removed Python engine, k=50, same-family Claude answerer and judge via `claude -p`, Mem0 prompt; **not comparable** with the rows above. The replicated 80-question runs in [the results](../experiments/eval/results/) also used the Python engine; the shipped Rust engine was last measured end to end in a parity run (Rust recall over sessions written by Python, 190 questions, 72.2% = Python's 72.2%, the same verdict on every question) before Rust-only ingest and review rounds 1–5; nothing since |

### LongMemEval_S (accuracy, 500 questions)

| System | Accuracy | Source, notes |
|---|---|---|
| Zep | 71.2% | GPT-4o judge; the most consistent number across several sources |
| Supermemory | 85.2% | Summarized in the Agent Zero Memory paper (2026) |
| EmergenceMem | 86.0% | same |
| Hindsight | 91.4% | same |
| ByteRover | 92.8% | Vendor report |
| Mem0 | 94.4% | Vendor report |
| Mastra (observational memory) | 94.9% (GPT-5-mini); 84.2% with GPT-4o | Vendor report; shows how much the answering model matters |
| Agent Zero Memory | 95.6% | Agent Zero Memory paper (2026) |
| rewire (`episodes`, no extraction), not comparable: Claude judge | 85.4% (10 excerpts); ≈89.8% (20 excerpts) | Iteration 8, removed Python engine; answerer + judge Claude via `claude -p`, official prompts (not the official GPT-4o judge) |
| rewire (`episodes`, no extraction), not comparable: Claude judge | 78.4% (10 excerpts; abstention 76.7%) | 2026-10-16, all 500: answerer Haiku 5.5, judge Haiku 4.5 via `claude -p`, official prompts; per-question diagnosis in `experiments/eval/results/2026-10-16-longmemeval-haiku55-ep10-analysis.md` |
| rewire (`episodes`, recall changes of 2026-10-16), not comparable: Claude judge | 83.6%; 85.6% with query expansion (`also`) | Same models and questions as the 78.4% row; whole turns + query-aware condensing, session opener, semantic at 0.25, 2× excerpts for counting questions. `experiments/eval/results/2026-10-16-longmemeval-improvements.md` |

rewire retrieval (no LLM, LongMemEval_S, 470 non-abstention questions): evidence session found 97.0%, all
evidence sessions 87.9%, mean context ≈11.2k tokens out of ≈115k.

Sources: [Mem0 paper](https://arxiv.org/pdf/2504.19413) ·
[Mem0: benchmarks 2026](https://mem0.ai/library/agent-memory/ai-memory-benchmarks-in-2026) ·
[Mem0: state of agent memory 2026](https://mem0.ai/blog/state-of-ai-agent-memory-2026) ·
[Memori LoCoMo results](https://memorilabs.ai/docs/memori-cloud/benchmark/results/) ·
[Agent Zero Memory](https://arxiv.org/pdf/2608.29606) ·
[Mastra observational memory](https://mastra.ai/research/observational-memory) ·
[ZenBrain (notes on vendor numbers)](https://arxiv.org/pdf/2604.23878) ·
[User as Code (Mem0 reimplementation 29.3%)](https://arxiv.org/pdf/2606.16707).

### Answering and judge models used by the comparison systems (checked in their evaluation code, 2026-10-14)

| Number | Answerer | Judge | Source |
|---|---|---|---|
| Mem0 paper 2025 (LoCoMo: Mem0 ≈66.9%, Mem0ᵍ ≈68.4%, Zep ≈66.0%, full-context ≈72.9%) | `gpt-4o-mini` (env `MODEL`, README default) | `gpt-4o-mini` (hard-coded in `metrics/llm_judge.py`) | repo `mem0ai/mem0`, folder `evaluation/` (commit `d85fcda`) |
| Mem0 platform 2026 (LoCoMo 92.5%, LongMemEval 94.4%) | `gpt-5` | `gpt-5` | repo `mem0ai/memory-benchmarks`, `results/platform/*.json` (runner default: `gpt-4o` / `gpt-4o`) |
| Zep LoCoMo 75.14% (Zep's rebuttal) | `gpt-4o-mini` | `gpt-4o-mini`, Zep's own grader prompt | repo `getzep/zep-papers`, `locomo_eval/` |
| Zep LongMemEval (2025 paper) | `gpt-4o-mini`, and `gpt-4o` for the 71.2% number | code: `gpt-4o-mini`, Zep's own grader prompt, not the official LongMemEval prompt | `zep-papers`, `zep_longmem_eval.ipynb` ("models default to gpt-4o-mini") |

Consequence: to compare with the Mem0 paper and Zep's LoCoMo number, use `gpt-4o-mini` as both
answerer **and** judge. The Mem0 2026 numbers use GPT-5 for both, so they are not comparable with runs that
use smaller models.

### Size of the context sent to the answerer (per question, LoCoMo)

| System | Context / question | Context contents | Score | Source |
|---|---|---|---|---|
| Memori | ≈1,294 tokens | concise facts | 81.95% | Memori report (vendor) |
| Zep | ≈1,602 tokens | facts (edges) + entity summaries | 79.1% (Memori), 75.1% (Zep) | MemCoT paper; Memori; Zep |
| Mem0 (2025 paper / third-party measurement) | ≈1,764 tokens | 10–30 concise facts | 62.5–68.4% | MemCoT paper; Memori; Mem0 paper |
| Mem0 platform 2026, top-50 | 50 facts (≈1.7–2k tokens, estimated) | concise facts | 82.7% (GPT-5) | `memory-benchmarks/results/platform/locomo_top50_results.json` |
| Mem0 platform 2026, top-200 | 200 facts, mean 6,956 tokens | concise facts | 91.6% (GPT-5) | `locomo_results.json` + Mem0 report |
| Full-context | ≈26k tokens | the whole conversation | 72.9% (4o-mini) – 87.5% | Mem0 paper; Memori |
| **rewire, 10 excerpts** | **≈2.2k tokens** (≈2.5k before merging) | 10 excerpts × 6 turns of raw transcript | all evidence 77.5% | §8 |
| **rewire, 20 excerpts** | **≈4.2k tokens** (≈4.8k before merging) | 20 excerpts of raw transcript | all evidence 82.9% | §8 |

Lesson: systems with small contexts (≈1.3–1.8k tokens) send **concise facts**, not raw transcripts;
Mem0 2026's high score actually uses a large context (≈7k tokens, 200 facts). rewire in `episodes` mode sends
raw transcripts with a context precision of only 11–20%: its tokens are spent on irrelevant turns. The right target
is not "a larger context" but **more evidence per token**: the retrieval KPI = percentage of questions with
all evidence at a fixed token budget (e.g. 2.5k).

### Head-to-head comparison (apples-to-apples)

The only fair way to compare is to run Mem0, Zep (Graphiti, open source) and rewire
**in the same harness**: the same dataset and sample, the same answering and judge models, and the same local embedding
for the systems that need embeddings (e.g. `bge-small-en-v1.5` or `all-MiniLM-L6-v2`). Things to watch:
- **Ingest costs differ greatly.** Mem0 and Graphiti call an LLM when writing memory (extraction per message /
  per episode); rewire in `episodes` mode makes no LLM calls at all, and `full` mode makes about 2 calls per
  session. Ingest tokens must be reported separately, not only the tokens spent answering.
- **Infrastructure.** Graphiti needs a graph database (Neo4j/FalkorDB); open-source Mem0 needs a vector store.
  Run them locally, without their cloud services, so the versions are pinned and the runs are repeatable.
- **Start small:** 20–50 questions per benchmark for all three systems, and go to full scale only if the difference is interesting.
Not implemented yet; the runner already separates tokens per stage (ingest / answer / judge), so the Mem0
and Graphiti adapters only need to fill in the ingest and retrieval stages.

## 5. Tokens: where they come from, and how many

Each question has three possible token consumers:

| Stage | Typical size per question | Notes |
|---|---|---|
| **ingest** (`--mode full` only) | ~2 calls per transcript **session**, not per question | LoCoMo: ~272 sessions; LongMemEval: ~50 sessions × 500 questions (each question has its own history), so `full` for LongMemEval is only feasible for small samples |
| **answer** | LoCoMo ≈4.4k prompt (≈1.8k Mem0 instructions + ≈2.7k context); LongMemEval ≈11.4k prompt | **The largest stage**; output 100–500 tokens (step-by-step reasoning) |
| **judge** | ≈400–600 prompt, ≈10–60 output | ±5% of the total; a candidate for a cheap model or Jev |
| **BEAM judge** | ≈3 calls per question (one per rubric item), each ≈1.3k tokens | the official BEAM judge is not cheap: ≈20% of the total in the dry run |
| **PersonaMem** | 0 | multiple choice, graded by matching |

BEAM 100K dry run (20 questions): ≈13.4k context tokens per question (excerpts contain long assistant answers),
≈$0.06 per 20 questions at the example prices above. Lower `--episodes` if the context is too large.

Dry-run estimates (prompts counted from the actual text, example prices $0.15 / $0.60 per 1M tokens):
LoCoMo 8 questions ≈ $0.008; LongMemEval 6 questions ≈ $0.012, which means 500 LongMemEval questions ≈ $1 with a
GPT-4o-mini-class model for answering and judging. Actual prices are taken from the OpenRouter catalog during the dry run.

Why earlier runs used up the quota quickly: every call went through `claude -p` on the same subscription
as the working session, used the largest default model, re-answered every question on each run, and for
LongMemEval carried ≈11–22k context tokens per question.

## 6. Rules for comparable numbers

1. **Same model as the comparison system.** Record the answering and judge models in every result (`report-*.md` writes them).
   For LongMemEval, the official judge is GPT-4o; for LoCoMo use the model used by the comparison system.
2. **Official prompts**, unmodified (already copied verbatim into the repo).
3. **Same subset**: LoCoMo categories 1–4; LongMemEval_S (not M, not the full version).
4. **Sample size**: the 95% confidence interval ≈ ±1.96·√(p(1−p)/n). For p≈0.85: n=120 → ±6.4 points;
   n=500 → ±3.1 points. 20 per category is enough for rough regression checks, not for public claims.
5. **Recall clock pinned.** Activation decays with memory age, so run.py recalls each question at the moment the brain
   finished ingesting (the time in `.state/eval_done`). Re-runs get exactly the same context and use the answer
   cache, whenever they are run. Runs before 2026-10-15 did not pin this clock yet.
6. **Judge variance**: J can shift ±1–3 points between runs with the same model; do not draw conclusions from
   small differences.

## 7. Model choice

| Role | Recommendation | Reason |
|---|---|---|
| Judge for public numbers | the same model as the comparison system (LongMemEval: GPT-4o) | numbers are comparable only when the judge is the same |
| Judge for regressions | a cheap, stable model; first check its agreement with the official judge on ~100 questions | the judge is only ~5% of tokens, but it must be consistent between runs |
| Answerer | one fixed model for every system being compared | the answering model shifts scores by tens of points (Mastra: 94.9% vs 84.2% from the model alone) |
| Ingest (`--mode full`) | a model that is reliable with JSON schemas | failed extraction = lost memory, not just a lower score |

**Main model (decision of 2026-10-14, after the Nemotron run):** `openai/gpt-4o-mini` as both answerer **and**
judge: the same as the Mem0 paper and Zep's LoCoMo number (table in §4), its judging is stable, it is not a reasoning model
(no `--reasoning-effort` needed), and it is cheap (full LoCoMo ≈$1.60–2.30; `openai/gpt-4o-mini:batch` costs half).
For LongMemEval, the official judge is GPT-4o: add a run with `--judge-model openai/gpt-4o` when the number
will be compared under the official protocol (answers come from the cache; only the judge is paid for).

**Model for the first run (2026-10-14):** `nvidia/nemotron-3-ultra-550b-a55b:free` as answerer
and judge. If its score is far below the retrieval upper bound (§8), or there are many empty / truncated answers, try
`thinkingmachines/inkling:free` on the same questions (same seed; the cache stores replies per model).
**Note 2026-10-14:** `inkling:free` returns 403 for direct API calls (it is only for registered "agentic harnesses");
use `thinkingmachines/inkling` (paid) or another free model. Both
are in the OpenRouter catalog (free, context ±1M tokens) and both are *reasoning* models: use
`--reasoning-effort low`, because the thinking process also consumes `max_tokens` and the answer can come back empty
(the report warns when that happens). When the answerer and the judge are the same model, it tends to grade itself
more leniently: acceptable for internal regressions, not for public numbers.

Free (`:free`) models on OpenRouter are good for trying out the harness, but less suitable for official numbers:
- **Rate limits:** their request limits are usually tight, so long runs are easily interrupted.
- **Availability:** the list of free models can change, which makes results hard to reproduce.
- **Data privacy:** check their data policy before sending real chat histories.
The public benchmarks themselves contain no personal data.
The model keys (`--answer-model`, `--judge-model`, `--ingest-model`) are always recorded in the report.

Other benchmarks considered:
- **PersonaMem**: relevant for user preferences.
- **BEAM**: very long-term memory.
- **GAIA**: not a fit, because it tests general assistants (tools, web), not memory.
- **SWE-bench**: relevant only if rewire is measured as memory for long-running coding agents.

## 8. Baseline without an LLM (already measured)

The upper bound for runs with an LLM: if the evidence is not in the context, the answering model cannot answer correctly
(unless it guesses). All numbers: `episodes` mode, keyword (no semantic search), k=30, 10 excerpts. Since
2026-10-14, overlapping excerpts from the same session are merged (same evidence, context −11–12%; details in
`experiments/eval/results/2026-10-14-locomo-nemotron-s20-analysis.md` §7).

**LoCoMo, 1,540 questions (2026-10-14):**

| Category | n | Evidence turns retrieved | All evidence present | MRR | Context / question |
|---|---|---|---|---|---|
| single-hop | 841 | 94.5% | 94.5% | 0.81 | ≈2.5k tokens |
| temporal | 321 | 84.0% | 83.4% | 0.68 | ≈2.5k |
| multi-hop | 282 | 54.9% | 32.3% | 0.55 | ≈2.6k |
| open-domain | 96 | 39.9% | 40.2% | 0.40 | ≈2.4k |
| **overall** | 1,540 | **73.2%** | **77.5%** | 0.71 | ≈2.5k (7.4× smaller than the full history); ≈2.2k after merging |

Retrieval p50 4 ms, p99 16 ms. Open-domain often needs general knowledge (not in the conversation); multi-hop
is rewire's improvement target. Note: the 94% J score from iterations 6–7 used k=50.

**LongMemEval_S after the 2026-10-16 recall changes:** all evidence 91.1% (98.3% with query expansion), evidence turns
94.4% (98.1%), context ≈12.5k tokens (19.1k), retrieval p50 235 ms (1.1 s): condensing embeds sentences at recall
time, to be cached in the index.

**LongMemEval_S, all 500 (2026-10-16, frozen clock):** evidence turns 86.2%, all evidence 85.7%, session found 97.0%,
MRR 0.857, context precision 25.0%, context ≈8.8k tokens (13.8× smaller), retrieval p50 21 ms / p99 195 ms
(`experiments/eval/results/2026-10-16-longmemeval-retrieval.md`).

**LongMemEval_S, 470 non-abstention questions (iteration 8):** evidence session found 97.0%, all evidence sessions 87.9%,
context ≈11.2k tokens out of ≈115k (≈10× smaller). A 120-question sample after merging: same evidence, context
11,090 → 9,780 tokens, MRR 0.71 → 0.79.

**PersonaMem, end to end (2026-10-17, Haiku 5.5, multiple choice):** 32k all 589: 59.3%; 128k and 1M, 20 per type:
55.7% and 51.4%, at ≈11k context tokens (compression 3× / 15× / 99×). Weak on `suggest_new_ideas` (20–26%) and
`generalizing_to_new_scenarios` (30–46%). `experiments/eval/results/2026-10-17-personamem-haiku55.md`.

**PersonaMem (dry run):** 32k, all 589 questions: context ≈12.6k tokens, compression 2.8× (the histories are short),
estimated $1.30 with GPT-4o-mini. 1M, 140 questions (20 per type): context ≈12.7k tokens out of ≈925k
(compression ≈92×), retrieval p50 42 ms / p99 277 ms, estimate for all 2,674 questions ≈$5.90.

**BEAM 100K (dry run, 20 questions):** all evidence present 52.9%, MRR 0.43, context ≈13.4k tokens (compression ≈11×),
52 judge calls for 20 questions; information extraction and preference following had 0% evidence in this small sample.

## 9. Evaluation session runbook

### Prerequisites (once, in the environment settings)
- `openrouter.ai` in **Allowed domains** (Network access). Checked as open on 2026-10-14.
- Secret / environment variable: `OPENROUTER_API_KEY` (required). Optional: `OPENROUTER_MODEL` (default model
  for answerer, judge and ingest), `OPENROUTER_BASE_URL` (default `https://openrouter.ai/api/v1`).
  New secrets are only picked up in a **new session**. Never paste the key into the chat or commit it.
- Models are written as ids (`nvidia/nemotron-3-ultra-550b-a55b:free`); model page URLs
  (`openrouter.ai/<id>`) are also accepted and normalized.
- The rewire binary: `cargo build --release --manifest-path rust/Cargo.toml` (the runner calls
  `rust/target/release/rewire`, or `$REWIRE_BIN`). Semantic search is optional: `rewire semantic install`
  (without it recall uses keywords, the same as the §8 baseline).
- Python ≥ 3.11 for the runner. Optional: the `tokenizers` package, for GPT-4o-tokenizer context counts (the
  tokenizer file is fetched from Hugging Face once; without it counts are characters / 4).

### Step 1: data (new session = new container, the old scratchpad is gone)

```bash
D=/tmp/eval-data OUT=/tmp/eval-out && mkdir -p $D/personamem $OUT && cd $D
curl -sSfLO https://raw.githubusercontent.com/snap-research/locomo/main/data/locomo10.json
# LongMemEval and PersonaMem: fresh signed URLs from GCS (valid for 2 hours), one URL per file
curl -sSf -o longmemeval_s_cleaned.json             '<signed url longmemeval_s_cleaned.json>'
curl -sSf -o personamem/questions_32k.csv           '<signed url questions_32k.csv>'
curl -sSf -o personamem/shared_contexts_32k.jsonl   '<signed url shared_contexts_32k.jsonl>'
curl -sSf -o personamem/questions_1M.csv            '<signed url questions_1M.csv>'
curl -sSf -o personamem/shared_contexts_1M.jsonl    '<signed url shared_contexts_1M.jsonl>'
# BEAM: only chat.json + probing_questions.json for 100K/500K/1M (no pickles, ±263 MB)
git clone -q --filter=blob:none --no-checkout https://github.com/mohammadtavakoli78/BEAM && cd BEAM
git sparse-checkout init --no-cone
for s in 100K 500K 1M; do printf '/chats/%s/*/chat.json\n/chats/%s/*/probing_questions/probing_questions.json\n' $s $s; done \
  > .git/info/sparse-checkout
git checkout -q main && cd /home/user/rewire
```

Do not use `git ls-tree --long` on a `--filter=blob:none` clone: it downloads every blob one by one.

### Step 2: check the key without printing it

```bash
test -n "$OPENROUTER_API_KEY" && echo "key loaded (${#OPENROUTER_API_KEY} characters)"
```

### Step 3: the smallest test run (confirms that model replies are read and tokens are recorded)

```bash
M=nvidia/nemotron-3-ultra-550b-a55b:free
python experiments/eval/run.py --bench locomo --data $D/locomo10.json --out $OUT --sample 2 \
  --answer-model $M --judge-model $M --reasoning-effort low --parallel 2 --max-cost 0.10 --label smoke
```

Check `report-locomo-smoke.md`: there are no warnings about empty / truncated answers, tokens per stage are filled in,
and J is plausible compared with §8. Open a few answers in `results-locomo-smoke.json`.

### Step 4: sample runs, one benchmark at a time

```bash
# LoCoMo 20 questions per category (80 questions), compare with §8
python experiments/eval/run.py --bench locomo --data $D/locomo10.json --out $OUT --sample 20 \
  --answer-model $M --judge-model $M --reasoning-effort low --parallel 2 --label nemotron-s20
# if far below the retrieval upper bound: a second model on the same questions
python experiments/eval/run.py ... --answer-model thinkingmachines/inkling:free --judge-model thinkingmachines/inkling:free --label inkling-s20
# then the other benchmarks with the same flags:
#   --bench personamem  --data $D/personamem --personamem-size 32k   (then 1M; no judge)
#   --bench longmemeval --data $D/longmemeval_s_cleaned.json
#   --bench beam        --data $D/BEAM/chats --beam-sizes 100K
```

**Multi-hop target (J ≥ 80%, on par with CortexDB's claimed 79.8% = 225/282 category 1 questions).** Run `full` mode
(facts + source excerpts) with gpt-4o-mini, at two context budgets, on the same questions:

```bash
M=openai/gpt-4o-mini
# ≈4.7k context tokens: multi-hop all evidence 51% (without an LLM)
python experiments/eval/run.py --bench locomo --data $D/locomo10.json --out $OUT --mode full --ingest-model $M \
  --k 30 --episodes 20 --sample 40 --answer-model $M --judge-model $M --max-cost 3 --label full-30-20
# ≈7k tokens, on par with Mem0 2026 (200 facts ≈7k): multi-hop all evidence 64–66%
python experiments/eval/run.py ... --k 100 --episodes 20 --label full-100-20
```

Ingest (fact extraction) is paid once per brain and reused by later runs. J is more lenient than
"all evidence present": in the Nemotron run, multi-hop J was 95% with all evidence at only 35%, so 80% J is likely
reached before all evidence reaches 80%. Also watch for overcounting: a large context once made
"how many" answers count too many (commit 084fae2).

Free models have per-minute and per-day limits: use `--parallel 2`, and if a run stops (429 / daily limit),
run the same command again later: the cache resumes from the last point.

**Full `full`-mode run on a free model (1,000 requests/day).** The 1,540 LoCoMo questions need ±3,600 requests:
±550 for ingest (extraction + judge per session, 272 sessions), then 1,540 answers and 1,540 judgments, so ±4 days. When
a 429 reply mentions the daily limit (`free-models-per-day`), the runner stops immediately without retrying, writes a
partial report (`PARTIAL RUN`), and answers questions only for conversations whose ingest has finished. Ingest
resumes per session (`<brain>/.state/eval_ingest.jsonl`), and answers and judgments come from the cache. An economical order:

```bash
# day 1: ingest every conversation (questions run with the remaining quota)
python experiments/eval/run.py --bench locomo --data $D/locomo10.json --out $OUT --mode full --ingest-model $M \
  --k 30 --episodes 20 --sample 0 --answer-model $M --judge-model $M --reasoning-effort low --parallel 2 --label full-q
# after ingest is complete, 0 tokens: evidence and context for every question on the same brain
python experiments/eval/run.py --bench locomo --data $D/locomo10.json --out $OUT --mode full \
  --k 30 --episodes 20 --sample 0 --retrieval-only --label full-q-retrieval
# following days: the same day-1 command, until there is no PARTIAL RUN
```

A `full`-mode report includes an **Ingest** section: sessions consolidated, failed attempts, replies that were not JSON
(retried up to 3 times), facts per session, the number of hypothetical questions, and active nodes per type. When the answering model
is the same as the judge, the report notes that J tends to be more lenient.

### Step 5: save the results to the repo

`$OUT` lives in the scratchpad and disappears with the session. Copy the report (not the cache, not the brain) to
`experiments/eval/results/<date>-<bench>-<label>.md`, record the model, sample, and seed, then commit.

### Commonly used runner options

```bash
# 0 tokens: retrieval only
python experiments/eval/run.py --bench longmemeval --data longmemeval_s_cleaned.json --out OUT --sample 0 --retrieval-only

# 0 tokens: estimate a run's tokens and cost before running it
python experiments/eval/run.py --bench locomo --data locomo10.json --out OUT --sample 20 --dry-run \
    --answer-model openai/gpt-4o-mini --judge-model openai/gpt-4o-mini

# BEAM (data: the chats/ folder from the BEAM repo) and PersonaMem (data: a folder with csv + jsonl), dry run
python experiments/eval/run.py --bench beam --data BEAM/chats --beam-sizes 100K,500K,1M --out OUT --sample 5 --dry-run
python experiments/eval/run.py --bench personamem --data personamem/ --personamem-size 32k --out OUT --sample 20 --dry-run

# A/B on one LoCoMo category; --expand-model: the model writes up to 4 extra queries (`also` in recall), like
# an agent calling rewire over MCP. Multi-hop, Haiku: strict J 77.3% -> 81.6%, all evidence 56% -> 71%
python experiments/eval/run.py --bench locomo --data locomo10.json --out OUT --mode full --sample 0 \
    --categories multi-hop --expand-model claude-cli/haiku --answer-model claude-cli/haiku --judge-model claude-cli/haiku

# numbers comparable with public ones (LongMemEval, official GPT-4o judge)
python experiments/eval/run.py --bench longmemeval --data longmemeval_s_cleaned.json --out OUT --sample 0 \
    --answer-model openai/gpt-4o --judge-model openai/gpt-4o --max-cost 20
```

Token-saving features in the runner:
- **Reply cache** per (stage, model, prompt) in `OUT/cache/llm.jsonl`: a re-run only pays for questions whose
  retrieval context changed, or for a new model.
- `--sample N` per category (stratified, fixed seed) · `--max-cost` (stops, then resumes from the cache) ·
  `--dry-run` and `--retrieval-only` without an LLM.

Output: `OUT/report-<bench>-<label>.md` (per-category tables, tokens per stage and per response, cost, latency)
and `OUT/results-<bench>-<label>.json` (per question: context, answer, verdict, usage).

## 10. Next

- Multi-hop retrieval (32–44% all evidence in `episodes` mode). Already tried, and did not help: 1-turn units,
  line trimming, pseudo-relevance feedback, an entity path, a second hop via entities, a per-session excerpt
  limit. What helped: `full` mode (30 facts + 20 excerpts: multi-hop 51%, overall 86% at ≈4.7k tokens).
  Next: a small contextual encoder or a cross-encoder (needs a model file; details in
  `experiments/eval/results/2026-10-14-locomo-nemotron-s20-analysis.md` §8).

- A cheaper alternative judge: Jev (yes/no decisions with probabilities) cascading to an LLM judge
  for uncertain cases; it must first be calibrated against the official judge on ~100 questions.
- Head-to-head with Mem0 and Graphiti in the same harness (§4).
- BEAM 10M and Kendall tau for event ordering.
- Faithfulness (whether the answer uses only the context): an extra judge, only if hallucination becomes a problem.
- Product metrics from dogfooding (§3).
