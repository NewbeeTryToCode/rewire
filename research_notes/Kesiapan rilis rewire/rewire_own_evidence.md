# rewire's own measured results and release-readiness (local repo evidence)

All sources are files in `/home/user/rewire` (git HEAD `535ffaf`, 81 commits). Paths are given relative to the repo root, followed by the section or table. "J" = LLM-judge accuracy.

## 1. Every benchmark number rewire has measured, and under which settings

### Takeaway
rewire has **two full-dataset end-to-end results**. LoCoMo has all 1,540 questions (categories 1–4): 93.0% J in `--mode full` with Haiku 5.5 as answerer and judge, and 93.6% when Haiku 4.5 re-judges. LongMemEval_S has all 500 questions in `--mode episodes`: 78.4% → 83.6% → 85.6% with a Haiku 5.5 answerer, a Haiku 4.5 judge and the official prompts. PersonaMem 32k was also run in full (589 questions, 59.3%; no judge needed). BEAM was only sampled (20 questions of 100K). **No run used gpt-4o-mini or the official GPT-4o LongMemEval judge.** Every published J used an answerer and judge from the Claude family, so none is protocol-comparable to Mem0 or Zep. The repo itself says so (`docs/evaluation.md` §4 warning, §6–7).

### Cited Findings

**LoCoMo (1,540 questions = categories 1–4, category 5 adversarial excluded "as Mem0 does"; 1,986 total is never used)** — [docs/evaluation.md §2](docs/evaluation.md)
- **Full run, 2026-10-15**: all 1,540 questions, `--mode full`, k=30, 20 excerpts (`episodes=20`). Ingest, answer and judge all used `claude-cli/haiku` (Haiku 5.5), seed 0, run before the recall clock was pinned. Mem0 official answer prompt and judge.
  - J by category: multi-hop 92.2% (282), open-domain 66.7% (96), single-hop 95.8% (841), temporal 94.1% (321). **Overall 93.0%**; F1 0.243; BLEU-1 0.166.
  - Context ≈5,013 tokens mean / 6,067 p95; answer prompt ≈11.1k tokens; retrieval p50/p95 9/20 ms (p99 29 ms); compression 3.7×.
  - J excluding the 29 disputed-gold questions: 94.8%.
  - Second judge, Haiku 4.5, on the same answers: **93.6%**, 98.5% agreement. Strict list J: 89.4%. Strict J excluding errata: 95.3%.
  - Ingest: 272 sessions, 15.8 facts per session, p50 42 s per session. Cost $0, billed on a subscription through `claude -p`.
  - The report itself notes: "answers are judged by the same model … J may read higher than with an independent judge."
  - Source: [experiments/eval/results/2026-10-15-locomo-haiku-full.md](experiments/eval/results/2026-10-15-locomo-haiku-full.md)
- Error analysis of the same run: 111 wrong answers. About 33 are not system errors (17 questions with swapped speakers, 8 wrong gold answers, 4 answers only in images, 4 judge errors). The rest split into 26 answerer, 26 retrieval and 11 ingest errors. Lenient list judging inflates J by about 2.5 points (inferred). — [2026-10-15-locomo-haiku-full-recommendations.md, Summary](experiments/eval/results/2026-10-15-locomo-haiku-full-recommendations.md)
- **Earlier full LoCoMo runs (iterations 6–7, 2026-10-12/13)**, all 1,540 questions with the official Mem0 judge: 94.2% hybrid, 94.9% keywords only, 94.0% after "facts pull in chunks" (noise ±3). k=50 was used. Answerer and judge were Claude via `claude -p`, so the judge is from the same family as the answerer. — [docs/iterations.md, Iterations 6–7](docs/iterations.md); [docs/evaluation.md §4 LoCoMo table, "rewire 94.0–94.9%" row](docs/evaluation.md); [experiments/locomo/README.md line 74: "we use Claude for consolidation, answering, and judging"](experiments/locomo/README.md)
- Iteration 5: subset of 389 questions from 3 conversations, J 69.7% → 91.3% → 96.7%. Limitation stated in the same entry: "a lenient judge from the same family". — [docs/iterations.md, Iteration 5](docs/iterations.md)
- **Nemotron run (2026-10-14)**: 80 questions (20 per category), `episodes` mode, keyword-only search, k=30, 10 excerpts. Answerer and judge both `nvidia/nemotron-3-ultra-550b-a55b:free`.
  - J 88.8% (multi-hop 95, open-domain 80, single-hop 95, temporal 85); all evidence 67.5%; context 2,556 tokens.
  - Answer latency p50 ≈27 s / p95 ≈99 s; cost $0. The run's 95% CI is ±6.9 points.
  - Marked "for internal regression tracking, not a public claim".
  - Source: [2026-10-14-locomo-nemotron-s20.md](experiments/eval/results/2026-10-14-locomo-nemotron-s20.md)
- **gpt-4o-mini**: this was chosen as the "main model" on 2026-10-14 because it matches the Mem0 paper and Zep (full LoCoMo estimated at ≈$1.60–2.30). **No gpt-4o-mini result file exists.** It appears only as planned or recommended in [docs/evaluation.md §7](docs/evaluation.md), [nemotron-s20-analysis.md lines 129, 187, 197](experiments/eval/results/2026-10-14-locomo-nemotron-s20-analysis.md) and [haiku-full-recommendations.md line 121](experiments/eval/results/2026-10-15-locomo-haiku-full-recommendations.md).
- **LoCoMo multi-hop A/B, all 282 questions** (full mode, Haiku, clock frozen, `REWIRE_SEMANTIC=0`): base J 90.8% → 92.9% with extra queries (`also`); all evidence 48.6% → 66.7%; context 5.1k → 7.0k tokens. — [2026-10-15-locomo-ab-A-base.md](experiments/eval/results/2026-10-15-locomo-ab-A-base.md), [2026-10-15-locomo-ab-B-also.md](experiments/eval/results/2026-10-15-locomo-ab-B-also.md). An 80-question sample gave 91.2% → 95.0% — [ab80-A/B](experiments/eval/results/2026-10-15-locomo-ab80-B.md).
- **Retrieval-only LoCoMo, all 1,540 questions (no LLM, 2026-10-14)**, episodes mode, keyword-only, k=30, 10 excerpts:
  - Evidence turns 73.2%; all evidence 77.5% (single-hop 94.5, temporal 83.4, multi-hop 32.3, open-domain 40.2); MRR 0.71.
  - Context ≈2.5k tokens (≈2.2k after merging); retrieval p50 4 ms / p99 16 ms.
  - Source: [docs/evaluation.md §8](docs/evaluation.md)

**LongMemEval_S (all 500 questions, ~115k-token histories)**
- **Baseline 78.4%, 2026-10-16**: `episodes` mode, BM25 only, k=30, 10 excerpts. Answerer `claude-haiku-5-5`, judge `claude-haiku-4-5`, both via `claude -p`, with the **official answer template and per-type judge prompt** (the judge model is not the official GPT-4o).
  - By type: knowledge-update 93.6, multi-session 65.4, single-session-assistant 85.7, single-session-preference 43.3, single-session-user 94.3, temporal 78.9. Abstention 76.7%.
  - Context 8,842 / 11,052 tokens (p95); compression 13.8×; retrieval 21/56 ms.
  - J is 88.1% when all evidence is in context and 20.9% when evidence is missing.
  - Source: [2026-10-16-longmemeval-haiku55-ep10.md](experiments/eval/results/2026-10-16-longmemeval-haiku55-ep10.md)
- **Arm A 83.6% / arm B 85.6%**: same models and questions as the baseline.
  - Arm A changes: semantic search at weight 0.25, whole turns with query-aware condensing, the session opener, and 2× excerpts for aggregation questions. Arm B adds Haiku-written query expansion.
  - Retrieval, all evidence: 85.7 → 91.1 → 98.3%.
  - Answer prompt tokens: 13.6k / 18.4k / 27.4k. Retrieval p50: 21 ms / 235 ms / 1,119 ms.
  - API-price equivalent: $3.37 / $3.89 / $4.96 for 500 questions.
  - Abstention: 76.7 / 80.0 / 73.3%. Question-level change vs baseline: A +41/−15, B +60/−24.
  - The repo's own read: A (+5.2) and B (+7.2) exceed the ±1–3 point judge variance; A→B (+2.0) is "at its edge".
  - Source: [2026-10-16-longmemeval-improvements.md, Results](experiments/eval/results/2026-10-16-longmemeval-improvements.md)
- Earlier iteration 8: 85.4% with 10 excerpts and ≈89.8% with 20 (Claude via `claude -p`, official prompts). — [docs/evaluation.md §4 LongMemEval table](docs/evaluation.md); [docs/iterations.md Iteration 8](docs/iterations.md)
- Retrieval-only, all 500: evidence turns 86.2%, all evidence 85.7%, session found 97.0%, context ≈8.8k tokens. — [2026-10-16-longmemeval-retrieval.md](experiments/eval/results/2026-10-16-longmemeval-retrieval.md)

**PersonaMem v1 (multiple choice, no LLM judge)**
- Run 2026-10-17: Haiku 5.5 answerer, episodes mode, k=30, 10 excerpts.
  - **32k, all 589 questions: 59.3%.** 128k, 140 sampled (20 per type): 55.7%. 1M, 140 sampled: 51.4%.
  - Context ≈11.0–11.7k tokens; compression 3.1× / 14.5× / 99.4×; retrieval p50 42 / 57 / 77 ms.
  - API-price equivalent $2.89 for 869 answers.
  - Weak question types: `suggest_new_ideas` 20–26% and `generalizing_to_new_scenarios` 30–46%.
  - Stated sampling error: ±4 points for 32k overall, ±8 points for 128k and 1M.
  - Source: [2026-10-17-personamem-haiku55.md](experiments/eval/results/2026-10-17-personamem-haiku55.md)

**BEAM (ICLR 2026)**
- Only the **100K size, 20-question samples** (2 per category) were run. The 500K, 1M and 10M sizes have no end-to-end runs (10M is not supported), and Kendall tau is not computed. — [docs/evaluation.md §2, §10](docs/evaluation.md)
- Rubric-score J across runs: base 47%, A 39%, B 54% — [2026-10-17-ab-four-benchmarks.md](experiments/eval/results/2026-10-17-ab-four-benchmarks.md). W 51.1% — [ab-embedding-rerank.md](experiments/eval/results/2026-10-17-ab-embedding-rerank.md). Replicated W mean 45.8% — [2026-10-20-replicated-ab.md](experiments/eval/results/2026-10-20-replicated-ab.md).
- Context ≈11.6–18k tokens per question. The dry run showed 52.9% all evidence and ≈13.4k tokens. — [docs/evaluation.md §8](docs/evaluation.md)

**Sampled 190-question A/B suite (2026-10-17 → 2026-10-20)**
- Composition: LoCoMo 40, LongMemEval_S 60, BEAM 100K 20, PersonaMem 32k 70, plus an extra 40 LoCoMo multi-hop questions (seed 1). Haiku 5.5 answers, Haiku 4.5 judges.
- Single-run scores over the 190 questions: base 70.2%, A 70.9%, B 67.3% ([ab-four-benchmarks.md](experiments/eval/results/2026-10-17-ab-four-benchmarks.md)); C2 72.2% ([ab-selective-expansion.md](experiments/eval/results/2026-10-17-ab-selective-expansion.md)).
- Entity graph: 67.4%, −4.8 points ([ab-episode-entity-graph.md](experiments/eval/results/2026-10-19-ab-episode-entity-graph.md)).
- Embedding and rerank arms show no J gain: W 72.2%, potion 71.7%, R2 71.1% ([ab-embedding-rerank.md](experiments/eval/results/2026-10-17-ab-embedding-rerank.md)).

### Inferences
- The headline "LoCoMo 93–95%" is a same-family-judge number: Haiku judges Haiku, or Claude judges Claude. The only cross-model check is Haiku 4.5 re-judging Haiku 5.5, which is still the same vendor. The repo's §4 table compares it to Mem0's 92.5%, which used GPT-5 for both answering and judging, and the doc itself flags that comparison as not comparable.
- LongMemEval 85.6% uses the official judge *prompt* but a Haiku 4.5 judge instead of GPT-4o. It also needs per-question LLM query expansion; the no-LLM default config scores 83.6%.

### Gaps
- No gpt-4o-mini or GPT-4o run exists, so there is no protocol-matched number against Mem0, Zep or LongMemEval leaderboards.
- No full-scale BEAM result exists, and BEAM 500K/1M were never answered end to end.
- The head-to-head with Mem0 and Graphiti in the same harness is "not implemented yet" ([docs/evaluation.md §4 and §10](docs/evaluation.md)).

## 2. Replicated results, run-to-run noise and sample sizes

### Takeaway
On the 190-question sampled suite, **three replicates show that no tested change beats the baseline**. The baseline W averages **69.6%**, lower than the single cached run's 72.2%. A single PersonaMem run moves **±4–6 points** from answer sampling alone. The fact graph and spreading activation add **0.0 points** on LoCoMo (95% CI [−4.2, +4.6]).

### Cited Findings
- **Noise check.** W was re-answered twice on identical contexts: 54.3% (cached), then 50.0% and 48.6%, for a mean of 51.0%. Between 3 and 9 of 70 answers flip between runs. "A single PersonaMem run moves by ±4–6 points from answer sampling alone." A single run "cannot separate changes smaller than about 5 points per set." — [2026-10-19-ab-excerpt-budget-by-type.md, "How noisy is one run?"](experiments/eval/results/2026-10-19-ab-excerpt-budget-by-type.md)
- **Replicated A/B (3 replicates, paired bootstrap 95% CI via `compare.py`)**. W per set: LoCoMo 40 at 86.7%, multi-hop 40 at 92.5%, LongMemEval 60 at 87.2%, BEAM 20 at 45.8%, PersonaMem 70 at 51.4%; overall 190 at **69.6%**.
  - Other arms, overall: k12 70.3, add2 70.4, ex25 67.6, t6 67.4, t6f12 69.2. Every overall CI includes 0.
  - add2's "significant" +3.9 on LongMemEval is shown to be a false positive: the gain comes entirely from questions whose context did not change.
  - The losses from ex25 and t6 on PersonaMem are real.
  - The report concludes the graph adds no J, and W stays the default ("best among those tested, not a proven optimum").
  - Source: [2026-10-20-replicated-ab.md](experiments/eval/results/2026-10-20-replicated-ab.md)
- **Full-mode graph ablation (3 replicates)**. All 10 LoCoMo conversations were ingested by Haiku 5.5 (272 sessions, ≈3,600 nodes). Same 40 + 40 sample.
  - J over 80 questions: W 89.6%, F0 (spreading off) 91.2%, F2 (2-hop spreading) 91.2%. F2−F0 = 0.0 [−4.2, +4.6]; F2−W = +1.7 [−2.5, +6.2].
  - Full mode costs about 2 LLM calls per session at ingest and about 40% more context (≈3.9–4.1k vs ≈2.8–2.9k tokens).
  - Warm recall p50/p95 was 15–21 / 54–72 ms.
  - The step-1 kill criterion applies: "no further graph retrieval work is planned."
  - Source: [2026-10-20-locomo-full-graph-ablation.md](experiments/eval/results/2026-10-20-locomo-full-graph-ablation.md)
- **Other stated noise levels**:
  - Judge variance ±1–3 points between runs with the same model — [docs/evaluation.md §6 rule 6](docs/evaluation.md)
  - Full LoCoMo J ±3 points (46 questions flip with 87% identical context) — [experiments/locomo/README.md line 30](experiments/locomo/README.md); [docs/iterations.md Iteration 7](docs/iterations.md)
  - CI rule of thumb: n=120 gives ±6.4 points and n=500 gives ±3.1 points — [docs/evaluation.md §6 rule 4](docs/evaluation.md)
  - On the 190-question suite: ±10 points per 60–70 question benchmark and ±6.5 points over all 190 — [ab-four-benchmarks.md](experiments/eval/results/2026-10-17-ab-four-benchmarks.md)
- **Full-dataset vs sampled**:
  - Full: LoCoMo 1,540 (Haiku full mode 2026-10-15; iterations 6–7; retrieval-only), LoCoMo multi-hop 282, LongMemEval_S 500 (78.4 / 83.6 / 85.6 and iteration 8), PersonaMem 32k 589.
  - Sampled: LoCoMo 80 (Nemotron), PersonaMem 128k and 1M (140 each), BEAM 100K (20), and every 190-question A/B and replicated run. All sources are as cited above.

### Inferences
- Before 2026-10-19 every A/B was single-run. Differences of 2–5 points in those reports (for example LongMemEval A 90% vs base 82% at n=60) are within the noise the repo later measured. The full-500 LongMemEval gains of +5.2 and +7.2 are more robust because n is larger, but they come from a single run.
- Runs before 2026-10-15 did not pin the recall clock, so their exact contexts are not reproducible ([docs/evaluation.md §6 rule 5](docs/evaluation.md)). This includes the LoCoMo 93.0% run.

### Gaps
- The full-dataset numbers (LoCoMo 1,540; LongMemEval 500) were never replicated, so there is no measured run-to-run spread at full scale.

## 3. Release-readiness beyond accuracy

### Takeaway
The engineering basics are in place. There is a Rust main binary, a Python reference implementation, CI that runs both plus clippy, an MCP server, hooks for Claude Code and Codex, and security scanning. Tests pass locally. The gaps are packaging and validation:
- no PyPI or crates.io release;
- mismatched versions (Python 0.1.0, Rust 0.2.0);
- the go/no-go evaluations on real user data (E1–E4) remain undone;
- multi-device node conflicts are "format ready, not yet enabled".

### Cited Findings
- **Install path**:
  - Rust: `cargo install --git https://github.com/NewbeeTryToCode/rewire rewire`, giving a single ~9 MB binary with bundled SQLite (`prime` ~20 ms, recall ~20 ms).
  - Python: `pipx install git+…`. Neither the Rust nor the Python install uses a registry.
  - MCP: `claude mcp add --scope user rewire -- rewire mcp`, `codex mcp add rewire -- rewire mcp`; `rewire install claude-code --write` installs the hooks.
  - Source: [README.md, Installation / Connect to a harness](README.md)
- **Versions**: `pyproject.toml` declares version 0.1.0, requires Python ≥3.11, has no required dependencies, and has optional extras `[anthropic]` and `[semantic]` (wordllama, numpy) — [pyproject.toml](pyproject.toml). `rust/Cargo.toml` declares version 0.2.0 with the `semantic` feature on by default — [rust/Cargo.toml](rust/Cargo.toml).
- **Tests run on 2026-10-20 (this session)**:
  - `pytest` is not installed, so `pytest -q tests` failed with "No module named pytest". The documented command `python -m unittest discover -s tests` **ran 71 tests, OK**. `tests/test_rewire.py` contains 71 `def test`.
  - `cargo test --offline` in `rust/` gave **66 passed, 0 failed**: 15 unit, 1 semantic, 50 suite.
  - Iteration 10 reported 61 Python and 54 Rust tests — [docs/iterations.md Iteration 10](docs/iterations.md)
- **CI**: `.github/workflows/ci.yml` runs Python unittest on 3.11 and 3.13, plus Rust `cargo test --locked` and `cargo clippy --all-targets -D warnings` — [.github/workflows/ci.yml](.github/workflows/ci.yml)
- **Python/Rust parity**:
  - Node files are byte-identical, and recall is identical on 304/304 LoCoMo questions (keyword and hybrid), via [experiments/conformance/recall_parity.py](experiments/conformance/recall_parity.py) — [README.md](README.md); [docs/iterations.md Iterations 9–10](docs/iterations.md)
  - End-to-end, Rust C2 matched Python C2 on all 190 questions: 72.2% for both, with 189/190 identical context sizes — [2026-10-17-ab-rust-gated-expansion.md](experiments/eval/results/2026-10-17-ab-rust-gated-expansion.md)
  - Note: most later A/Bs (2026-10-19/20) ran on the Python engine only.
- **Security**:
  - Secret scan and injection scan. `security::blocked()` rescans stored notes whenever they are displayed and returns "secret" or "injection" (high-entropy tokens included), shown as `[BLOCKED: …; check this note with rewire doctor]` — [rust/src/security.rs lines 120–147](rust/src/security.rs)
  - Injected memory is fenced in `<rewire-memory>`. Provenance comes from the transcript. Tool output never auto-becomes a rule, and `undo` reverts a session's consolidation — [README.md bullets](README.md)
  - The threat model covers indirect injection, malicious repos, secret leaks, public remotes and poisoned memory — [docs/design/mvp-spec.md §9](docs/design/mvp-spec.md)
  - Design review scores security 8.5/10 but notes "mitigations not yet tested against real attacks" — [docs/design/design-review.md §6](docs/design/design-review.md)
- **Data model**:
  - The brain is a git repo of markdown nodes with frontmatter, plus append-only `events/<device>/YYYY-MM.jsonl` merged with `merge=union`. A SQLite FTS5 index is derived from these and gitignored. It is Obsidian-compatible — [README.md Brain structure](README.md)
  - The frozen format spec is in [docs/design/format.md](docs/design/format.md).
  - Multi-device: "format ready, not yet enabled". Two devices editing the same node file is not yet handled — [docs/design/mvp-spec.md §8](docs/design/mvp-spec.md); [design-review.md §1 row 2](docs/design/design-review.md)
- **Cost and latency**:
  - `prime`, recall and index refresh use **0 LLM calls**. Consolidation makes about 2 LLM calls per session; `remember` makes 0 — [docs/design/architecture.md "Token cost profile"](docs/design/architecture.md)
  - In `episodes` mode, ingest makes no LLM calls at all — [docs/evaluation.md §4 head-to-head](docs/evaluation.md)
  - Recall latency ranges from 4 ms (LoCoMo keyword) to 21 ms (LongMemEval baseline) to 235 ms (arm A, sentence embedding at recall time) to 1.1 s (arm B). The LongMemEval improvements report states "Latency regressed and needs work before shipping" — [2026-10-16-longmemeval-improvements.md Reading](experiments/eval/results/2026-10-16-longmemeval-improvements.md)
  - Later, the PersonaMem report says sentence vectors in condensing were "since dropped for latency with the same evidence on LongMemEval" — [2026-10-17-personamem-haiku55.md](experiments/eval/results/2026-10-17-personamem-haiku55.md)
  - The Rust engine measures recall p50 at 10–156 ms excluding 96 ms process start plus model load — [ab-rust-gated-expansion.md](experiments/eval/results/2026-10-17-ab-rust-gated-expansion.md)
- **Open TODOs and limitations listed in the docs**:
  - E1: consolidation precision on 20 real transcripts, target ≥0.85. Only 1 session has been assessed (6/6 claims correct).
  - Verify the Codex parser on real rollout files.
  - E4: 2-week dogfooding with snapshot hit rate ≥40%, ≤3 inbox items per session, 0 secrets.
  - Sources: [docs/iterations.md "Next (iteration 11)" and line 28](docs/iterations.md); [docs/design/mvp-spec.md §10–11](docs/design/mvp-spec.md). Milestone M5 (dogfooding, "E4 passes → v0.1") is not done.
  - Evaluation TODOs: head-to-head vs Mem0/Graphiti, BEAM 10M and Kendall tau, a calibrated cheap judge (Jev), and a faithfulness judge — [docs/evaluation.md §10](docs/evaluation.md)
  - Weak spots: multi-hop retrieval (32–44% all evidence in episodes mode); generative preference questions on PersonaMem; open-domain LoCoMo (66.7%).
- **Reproducibility assets**:
  - Runner [experiments/eval/run.py](experiments/eval/run.py) (OpenRouter or `claude -p`), with flags `--dry-run`, `--retrieval-only`, `--max-cost`, `--replicate`, a stratified `--sample` with fixed seed, and the recall clock pinned to `.state/eval_done`.
  - Reply cache `OUT/cache/llm.jsonl`, which lives in the scratchpad and is not committed.
  - Paired comparison [experiments/eval/compare.py](experiments/eval/compare.py); diagnosis scripts `diagnose.py` and `diagnose_lme.py`; verbatim official prompts (`experiments/locomo/official_prompts.py`, `experiments/longmemeval/lme_prompts.py`, `experiments/eval/beam_prompts.py`).
  - [experiments/eval/locomo_errata.json](experiments/eval/locomo_errata.json) holds 29 disputed-gold questions plus a `_doc` key. The official J is still the reported number.
  - Data sources and validation are documented, along with the runbook — [docs/evaluation.md §2 and §9](docs/evaluation.md)
  - Only the report .md files are committed (Step 5); caches, brains and per-question results JSON are not. The one exception is a committed CSV diagnosis (`2026-10-16-longmemeval-haiku55-ep10-diagnosis.csv`).
- **Docs quality**: the README is concise (83 lines) and links to architecture, format, MVP spec, concept, iteration log and evaluation. `docs/evaluation.md` (460 lines) carries explicit non-comparability warnings and the model-protocol table of competitors.
  - One language inconsistency: the README example uses Indonesian queries ("keputusan storage", "Selalu tulis test sebelum fitur").
  - No CHANGELOG file exists at the repo root.
  - Source: [README.md](README.md); repo listing.

### Inferences
- In engineering terms the project is ready for an early or alpha release. CI is green, tests pass, parity is verified, and the security design is explicit. Two things keep it from meeting its own definition of v0.1 (M5/E4): the planned validations on real user data are undone, and no registry packages exist.
- For public claims, the main readiness gap is evaluative, not functional. Producing comparable numbers needs a gpt-4o-mini (LoCoMo) or GPT-4o-judge (LongMemEval) run. The runner supports this from cache: "answers come from the cache; only the judge is paid for."

### Gaps
- `pytest` itself could not be run because it is not installed; the unittest results are reported instead.
- `cargo clippy` was not run in this session, and there is no CI run history locally.
- No measured memory or disk footprint of a brain at scale was found, nor any security red-team results.
