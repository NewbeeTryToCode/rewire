# Evaluation methodology for agent memory benchmarks and release-readiness evidence (as of 2026-10-20)

Research caveat: several primary hosts (arxiv.org, blog.getzep.com, dev.to, vectorize.io, mastra.ai) failed DNS resolution from this environment, so many findings below come from search-engine extracts of those primary pages rather than full-page reads. Where only a search extract was seen, that is noted ("via search extract"). Numbers should be spot-checked against the primary page before being quoted in a public release.

## 1. LLM-as-judge reliability for memory QA

### Takeaway
LLM judges on LoCoMo/LongMemEval-style QA agree with humans at roughly 92–99% on clean samples, but they are systematically lenient: they credit wrong answers far more often than they reject right ones. Swapping the judge (e.g. GPT-4o-mini vs GPT-4o) moves absolute scores by several points while mostly preserving rankings. So absolute numbers are only comparable when the judge model and prompt are the same, and the judge must be reported.

### Cited Findings
- Self-preference bias: Wataoka, Takahashi and Ri (SB Intuitions), "Self-Preference Bias in LLM-as-a-Judge", arXiv:2410.21819 (Oct 2024, revised Jun 2025; NeurIPS 2024 workshop listing). It proposes a metric and finds GPT-4 shows significant self-preference. The mechanism is perplexity: judges rate lower-perplexity (more familiar) text higher than humans do, whether or not they generated it. This implies a same-family judge/answerer pair (e.g. GPT-4o-mini judging GPT-4o-mini answers) can be favoured. — [arXiv 2410.21819](https://arxiv.org/html/2410.21819v1); [NeurIPS listing](https://neurips.cc/virtual/2024/106181)
- Follow-up measurement work: "Beyond the Surface: Measuring Self-Preference in LLM Judgments" (arXiv:2506.02592, Jun 2025). — [arXiv 2506.02592](https://arxiv.org/abs/2506.02592v1)
- Judge–human agreement on memory QA (via search extracts):
  - EverMemOS (arXiv:2601.02163, Jan 2026), judge GPT-4o-mini, 25+25 items: LongMemEval κ=0.978, accuracy 99.2%; LoCoMo κ=0.891 (CI 0.742–1.000), accuracy 98.4%. The sample is very small and the CIs are wide. — [EverMemOS](https://arxiv.org/pdf/2601.02163)
  - MemSIF (arXiv:2608.01742, 2026), judge GPT-4o against adjudicated human labels on 220 LoCoMo and 220 LongMemEval-S answers: judge–human agreement 93.6% / 92.3% (κ 0.82 / 0.79). Human–human agreement was 96.4% / 94.5%. So the judge sits 2–3 points below the human ceiling. — [MemSIF](https://arxiv.org/pdf/2608.01742)
- Judge swaps:
  - MemFlow (arXiv:2605.03312): GPT-4o-mini and GPT-4o judges agree on 85.7% of 300 LongMemEval predictions. In the 14.3% of disagreements GPT-4o-mini was the stricter one. The authors argue a swap would shift all systems roughly uniformly. — [MemFlow](https://arxiv.org/pdf/2605.03312)
  - EdgeMem (arXiv:2609.05553, an LLM-free memory construction paper, close to rewire's design): a "generous" gpt-4o-mini judge upgrades 17.7% of a stricter judge's negatives and downgrades only 0.1%, yet row orderings were identical. — [EdgeMem](https://arxiv.org/pdf/2609.05553)
  - Memory-R2 (arXiv:2605.21768): the LLM judge credits deterministically wrong answers on a weighted 5.4% of items and rejects deterministically correct ones on only 1.0%. That inflates accuracy by about 5 points, with rank correlation versus rule-based scoring ρ=0.78. Leniency concentrates on partial-match and date-format items (8.6% judge false positives on date extraction). — [Memory-R2](https://arxiv.org/pdf/2605.21768)
- LoCoMo answer-key and judge audit by Penfield Labs (blog, not peer reviewed): it reports that 6.4% of the LoCoMo answer key is wrong and that the standard LLM judge accepts up to 63% of intentionally wrong answers. — [Penfield Labs Substack](https://penfieldlabs.substack.com/p/we-audited-locomo-64-of-the-answer); [dev.to mirror](https://dev.to/penfieldlabs/we-audited-locomo-64-of-the-answer-key-is-wrong-and-the-judge-accepts-up-to-63-of-intentionally-33lg)
- Claude Haiku as judge: PRBench (arXiv:2511.11562, an expert-rubric domain, not memory) found Claude Haiku 4.5 κ=0.535 against experts, versus 0.603 for o4-mini and 0.605 for GPT-4.1. Haiku was the only judge below the expert–expert baseline. — [PRBench](https://arxiv.org/pdf/2511.11562). context-bench's MemoryJudge uses Claude Haiku as its default judge but publishes no comparison. — [context-bench docs](https://www.mintlify.com/npow/context-bench/api/evaluators/memory-judge)

### Inferences
- Comparable memory scores need the same judge model, the same judge prompt, and the same answer model. A gap of 2–5 points between systems scored by different judges is within the plausible judge-swap effect (~5 points, Memory-R2), so it cannot be interpreted.
- If rewire publishes absolute J scores, it should:
  - name the judge (model and version) and publish the judge prompt;
  - avoid a same-family answerer/judge pair where possible, or report a second judge;
  - report judge–human agreement on a hand-labelled sample (≥100–200 items is more credible than 25–50);
  - publish a sanity check with intentionally wrong answers, which directly answers the Penfield-style criticism.
- A deterministic or rule-based secondary metric (e.g. exact or normalised match on short answers, or retrieval recall@k against the gold evidence turns) is cheap and immune to judge bias. It also suits an LLM-free-ingest system.

### Gaps
- I found no single controlled study that swaps only the judge among GPT-4o, GPT-4o-mini and Claude Haiku on one memory benchmark and reports per-judge accuracy. The evidence comes from different papers.
- I could not read the original LongMemEval paper's judge-validation figure (arxiv unreachable). It is commonly cited as above 97% agreement for GPT-4o, but that is unverified here.

## 2. Statistical practice for n = 40–500 questions

### Takeaway
Standard errors at these sizes are large: about ±4 points at n=500 and about ±14 points at n=40 (95% CI, accuracy around 70%). Claiming a 2–5 point difference between two systems needs paired analysis (McNemar or paired bootstrap on the same questions), clustering by conversation, and usually several hundred to over a thousand questions.

### Cited Findings
- Evan Miller (Anthropic), "Adding Error Bars to Evals" (arXiv:2411.00640, Nov 2024). The blog post of 19 Nov 2024 gives five recommendations:
  1. Report the SEM and a 95% CI = mean ± 1.96·SEM.
  2. Cluster standard errors on the unit of randomisation, e.g. the passage. Clustered SEs on popular evals were more than 3× the naive ones.
  3. Reduce within-question variance by resampling answers several times per question (Inspect "epochs").
  4. Use paired differences when models answer the same questions. Frontier models' per-question scores correlate at about 0.3–0.7, which substantially shrinks the SE of the difference. Report the mean difference, SE, CI and correlation.
  5. Do a power analysis up front, e.g. hypothesise a 3-point difference and compute the number of questions needed.
  — [Anthropic blog](https://www.anthropic.com/research/statistical-approach-to-model-evals); [arXiv 2411.00640](https://arxiv.org/pdf/2411.00640); `errorbars` package: [PyPI](https://pypi.org/project/errorbars/)
- Position paper "Don't Use the CLT in LLM Evals With Fewer Than a Few Hundred Datapoints" (arXiv:2503.01747, 2025). It argues that CLT intervals are unreliable for small n and recommends alternatives such as Bayesian or Wilson-type intervals. I could not open the full text to confirm the specific recommendations. — [arXiv 2503.01747](https://arxiv.org/pdf/2503.01747)
- Memory benchmarks are inherently clustered. LoCoMo has about 10 conversations, each with many questions, and LongMemEval has 500 questions over per-question haystacks. Miller's clustering advice applies to LoCoMo especially. — [Anthropic blog](https://www.anthropic.com/research/statistical-approach-to-model-evals)
- Zep reports its corrected LoCoMo score as 75.14% ± 0.17. The ± reflects run-to-run variation over repeated runs, not question-sampling error. — [Zep blog via search extract](https://blog.getzep.com/lies-damn-lies-statistics-is-mem0-really-sota-in-agent-memory/)
- MemDelta (arXiv:2606.29914, 2026) uses significance tests when comparing memory arms. For example, it finds verbatim RAG 47.2% vs full context 49.8% on LongMemEval-S with GPT-4o-mini: a 2.6-point gap, p=0.34, not significant. — [MemDelta](https://arxiv.org/pdf/2606.29914)

### Inferences (my own arithmetic, standard formulas; not from a source)
- Unpaired 95% half-width at p≈0.7 is 1.96·√(p(1−p)/n):
  - n=40: about ±14.2 points
  - n=100: about ±9.0 points
  - n=200: about ±6.4 points
  - n=500: about ±4.0 points

  At p≈0.9, n=500 gives about ±2.6 points. For n<100, use Wilson or Clopper–Pearson intervals.
- Paired McNemar power: the required n is approximately (z_{α/2}+z_β)²·d/Δ², where d is the discordant-pair rate and Δ the true difference. With α=0.05, 80% power and d=0.15:
  - Δ=5 points needs about 470 questions
  - Δ=3 points needs about 1,300 questions
  - Δ=2 points needs about 2,900 questions

  So on LongMemEval (n=500), only differences of roughly 5 points or more are reliably detectable. On a 40-question subset, only differences of about 20 points or more are.
- Recommended practice for rewire:
  - report a 95% CI per score;
  - use a paired bootstrap (resampling conversations for LoCoMo, questions for LongMemEval) or McNemar for every A/B claim;
  - run 3+ replicates where any LLM step is stochastic, and report the mean ± SD across runs separately from sampling error;
  - correct for multiple comparisons (Holm or Bonferroni) when many ablation arms or per-category breakdowns are reported;
  - label subset runs (n=40–100) as "directional" only.

### Gaps
- I found no memory-benchmark-specific paper that reports observed discordant rates between memory systems. The d=0.15 above is an assumption; rewire can measure d from its own A/B runs.

## 3. Answer-model strength, token budget and full-context baselines

### Takeaway
The answer model and the retrieved-token budget move scores as much as, or more than, the memory system itself. The same memory gains about 7 points going from GPT-4o-mini to GPT-4o, and RAG-vs-full-context conclusions can flip sign across answer models. Fair comparisons must fix the answer model and report the token budget, alongside a full-context baseline run with the same model and judge.

### Cited Findings
- Zep on LongMemEval-S (Zep "State of the art agent memory" post, early 2025, via search extract):

  | Setup | Answer model | Accuracy | Median latency | Context tokens |
  |---|---|---|---|---|
  | Zep | GPT-4o | 71.2% | 2.58 s | ~1.6k |
  | Full context | GPT-4o | 60.2% | 28.9 s | ~115k |
  | Zep | GPT-4o-mini | 63.8% | 3.20 s | ~1.6k |
  | Full context | GPT-4o-mini | 55.4% | 31.3 s | ~115k |

  The model swap is worth about 7 points for the same memory. — [Zep blog](https://blog.getzep.com/state-of-the-art-agent-memory/)
- Emergence AI found full-context GPT-4o on LongMemEval at 60–64% depending on implementation, attributing a 3.6-point difference from Zep's figure to prompting alone. This shows the baseline itself is prompt-sensitive. — [Emergence blog](https://www.emergence.ai/blog/sota-on-longmemeval-with-rag); [Emergence SOTA post](https://www.emergence.ai/blog/og2q4h3p2zcmkxjdvj099ogqm21a4a)
- MemDelta (arXiv:2606.29914, "Controlled Baselines and Hidden Confounds in Agent Memory Evaluation", 2026):
  - with GPT-4o-mini fixed, verbatim RAG (47.2%) is about equal to full context (49.8%), p=0.34;
  - with cloud embeddings, RAG reaches 53.4%, significantly above an S4 system (p=0.004);
  - model choice flips conclusions: full context wins by 14 points on Gemini Flash, while retrieval wins by 31 points on Sonnet.

  — [MemDelta](https://arxiv.org/pdf/2606.29914); [catalyzex summary](https://www.catalyzex.com/paper/memdelta-controlled-baselines-and-hidden)
- In the LoCoMo debate (2025), a simple full-context baseline scored about 73%, above Mem0's best reported 68.5% (graph variant). Letta reported 74.0% on LoCoMo by storing conversations as plain files with grep-style tools; Letta is itself a memory vendor. — [Zep rebuttal via search extract](https://blog.getzep.com/lies-damn-lies-statistics-is-mem0-really-sota-in-agent-memory/); [Towards AI article](https://pub.towardsai.net/mem0-vs-zep-vs-letta-a-folder-of-text-files-shouldnt-beat-the-61k-star-memory-layer-9d7e65c5799c)
- "Fidelity Before Structure: Verbatim Chunks Beat Lossy Artifact Extraction in Long-Conversation LLM Memory" (arXiv:2601.00821, 2026) is supportive evidence for verbatim storage such as rewire's markdown. I saw only the title. — [arXiv 2601.00821](https://arxiv.org/pdf/2601.00821)
- Zep's follow-up post tested gpt-4.1 and o4-mini with long context and argued that processing 115k tokens adds substantial latency and cost. — [Zep blog](https://blog.getzep.com/gpt-4-1-and-o4-mini-is-openai-overselling-long-context/)
- Later vendor scores are not comparable. Mem0's 2026 self-reported scores are about 92.5 on LoCoMo and 94.4 on LongMemEval, at a 200-memory retrieval budget. Zep's 2026 claim is 94.7%. These used different models and setups. Mem0's own guide notes that "run LoCoMo" isn't one fixed procedure. — [Mem0 benchmarks blog](https://mem0.ai/blog/ai-memory-benchmarks); [rywalker summary](https://rywalker.com/research/mem0)

### Inferences
- A credible rewire results table should have:
  - one fixed answer model, ideally one cheap and one strong, both reported;
  - one fixed judge;
  - a reported retrieved-context budget (mean tokens per query);
  - a full-context baseline with the same answer model and judge;
  - a naive baseline (BM25/FTS5-only, or a plain-files/grep baseline in the spirit of Letta's) to show the added value of embeddings and ranking.
- Cross-vendor comparisons to published numbers should be labelled "vendor-reported, different answer model, judge and budget". Better still, re-run competitors through a shared harness (see section 4).
- An accuracy-vs-tokens curve (e.g. budgets of 1k, 2k, 4k and 8k tokens) is more informative than a single point and pre-empts "you just stuffed more context" critiques.

### Gaps
- I found no study isolating Claude (Sonnet or Haiku) as the answer model on the same memory retrieval, apart from MemDelta's sign-flip finding.

## 4. What open-source memory projects ship at release, and community criticism

### Takeaway
The credible norm in 2025–2026 includes:
- an open, runnable eval harness, with checkpoints and the judge prompts included;
- results on LoCoMo and/or LongMemEval with the answer model, judge and token budget stated;
- latency and token tables;
- a permissive licence (MIT or Apache-2.0).

Overclaiming has been punished publicly, as with the Mem0-vs-Zep dispute and MemPalace's "100%". Vendor numbers are widely treated as non-comparable unless re-run in a shared harness.

### Cited Findings
- Harnesses and evidence projects ship:
  - **Supermemory MemoryBench** (MIT): a unified CLI that runs the same questions, pipeline and judges across providers (supermemory, mem0, zep) and benchmarks (LoCoMo, LongMemEval, ConvoMem), with checkpointed and resumable runs. — [GitHub](https://github.com/supermemoryai/memorybench); [docs](https://supermemory.ai/docs/memorybench/overview)
  - **Mem0** open-sourced its evaluation harness at github.com/mem0ai/memory-benchmarks. A paper footnote notes the scores are vendor-reported. I did not verify the repo. — [search extract via arXiv 2605.28969](https://arxiv.org/pdf/2605.28969)
  - **Hindsight** (Vectorize) paper "Hindsight is 20/20" (arXiv:2512.12818, Dec 2025): states its code can be downloaded and its benchmarks re-run, but the baseline numbers are "reference points rather than independently reproduced baselines". — [arXiv 2512.12818](https://arxiv.org/pdf/2512.12818)
  - **Community neutral harnesses:**
    - MemoryStackBench: local, with adapters for Hindsight, Letta, Supermemory and others. — [GitHub](https://github.com/aetna000/MemoryStackBench)
    - agent-memory-bench: grades what systems retrieve rather than what they generate. Retrieval recall: Letta 0.11.7 52.2%, Mem0 2.1.0 50.0%, Zep/Graphiti 0.30.2 37.0%. — [GitHub](https://github.com/hamza-dev-tech/agent-memory-bench)
    - Bkankim/memory-bench: mem0 vs Hindsight under the same conditions, covering recall and latency plus offline, isolation, audit and telemetry axes, summarised as a decision matrix. — [GitHub](https://github.com/Bkankim/memory-bench)
  - **Independent vs self-reported gap:** one comparison lists Mem0 at about 66% self-reported vs about 58% independent on LoCoMo. — [dev.to comparison](https://dev.to/varun_pratapbhardwaj_b13/5-ai-agent-memory-systems-compared-mem0-zep-letta-supermemory-superlocalmemory-2026-benchmark-59p3)
  - **Licences:** Letta and Graphiti (Zep's engine) are Apache-2.0; Zep's full service is cloud plus self-host. Mem0 is described both as Apache-2.0 and as open-core, so sources conflict. Hindsight's open-source status is contested between sources. — same dev.to comparison; [Mem0 vs Zep (Mem0 library)](https://mem0.ai/library/agent-memory/zep-vs-mem0-which-ai-memory-layer-should-you-choose)
- Mem0 vs Zep dispute (2025):
  - Zep's rebuttal, "Lies, Damn Lies, & Statistics: Is Mem0 Really SOTA in Agent Memory?", argues that Mem0 ran Zep incorrectly: both speakers were assigned the user role, timestamps were appended to text instead of the created_at field, and searches ran sequentially, which inflated latency.
  - Zep claims about 10% higher accuracy, and itself corrected a miscalculated LoCoMo score to 75.14% ± 0.17.
  - Lesson: when you benchmark a competitor, its integration must follow its documented best practice, or the comparison will be attacked.

  — [Zep blog (search extract)](https://blog.getzep.com/lies-damn-lies-statistics-is-mem0-really-sota-in-agent-memory/)
- MemPalace (April 2026):
  - It launched with "100% on LongMemEval" (hybrid with a Claude Haiku rerank) and 96.6% raw, reaching the top of Hacker News.
  - Criticisms:
    1. The developers fixed the 3 failing questions and retested on the same set (overfitting to the test set).
    2. "100%" was retrieval recall@5, not LongMemEval's end-to-end QA accuracy.
    3. The 96.6% ran on raw text with ChromaDB default embeddings, so the advertised "palace" architecture was not exercised.
    4. "Lossless" AAAK compression dropped the score to 84.2%.
  - The headline was revised to 96.6% after pushback. One reviewer still called the architecture clever (34% better than flat vector search in their tests) but the marketing misleading.

  — [Hackernoon](https://hackernoon.com/resident-evil-star-milla-jovovich-shipped-an-ai-memory-system-devs-shredded-its-benchmarks); [Vectorize review](https://vectorize.io/articles/mempalace-review); [danilchenko.dev review, 2026-04-10](https://www.danilchenko.dev/posts/2026-04-10-mempalace-review-ai-memory-system-milla-jovovich/); [SDxCentral](https://www.sdxcentral.com/news/milla-jovovichs-ai-memory-project-has-a-few-things-it-would-rather-forget/); [arXiv 2604.21284 critical analysis](https://arxiv.org/pdf/2604.21284)
- Mastra "Observational Memory: 95% on LongMemEval" is another vendor-reported high score; the page could not be fetched to check its answer model and judge. — [Mastra](https://mastra.ai/research/observational-memory)

### Inferences
- The most common overclaim patterns, which a release should explicitly avoid:
  1. Reporting retrieval recall as if it were QA accuracy.
  2. Tuning on the test set (fixing specific failing questions).
  3. Comparing against competitor numbers from different answer models or judges.
  4. Running competitors with a misconfigured integration.
  5. Benchmarking a configuration different from the advertised one (e.g. an LLM rerank in the headline but "LLM-free" in marketing). This is directly relevant to rewire's "LLM-free ingest by default" claim: the headline score must come from the default configuration, or the difference must be labelled.
  6. Omitting a full-context or naive baseline.
- A release-ready evidence pack for rewire would be:
  - an eval script runnable with one command, with pinned dataset versions and commit hash;
  - the judge and answer prompts;
  - raw per-question outputs (JSONL), so others can re-judge;
  - CIs and paired tests;
  - a dev/test split, with any tuning done on a dev split or on other benchmarks;
  - latency (p50/p95) and token/cost tables;
  - ideally, results in a neutral harness such as Supermemory MemoryBench or MemoryStackBench by writing an adapter.

### Gaps
- MCP support per project was not confirmed by sources in this pass. Known from general knowledge but not verified here: basic-memory is MCP-native, and Mem0 has an OpenMemory MCP server. Treat both as unverified.
- I did not retrieve primary READMEs for Cognee, MemOS, Memobase or basic-memory to list their exact release evidence.
- I did not retrieve the original Hacker News or Reddit threads; only secondary articles.

## 5. Non-accuracy release criteria (latency, cost, privacy, portability, robustness, security)

### Takeaway
Credible memory releases report per-query latency (median and tail) and context tokens alongside accuracy; Zep's tables are the reference pattern. Security against memory poisoning is an established research threat (MINJA, NeurIPS 2025), so stored-memory prompt injection should be addressed in docs and tests. Offline operation, isolation, audit and telemetry are emerging comparison axes for self-hosted memory.

### Cited Findings
- Latency and tokens: Zep reports median latency and average context tokens per configuration (e.g. 2.58 s and 1.6k tokens vs 28.9 s and 115k for full context), arguing that it uses under 2% of the baseline's tokens. — [Zep blog](https://blog.getzep.com/state-of-the-art-agent-memory/)
- Measurement artefacts can distort latency: Zep alleged that Mem0 ran Zep's searches sequentially, inflating Zep's latency. — [Zep rebuttal (search extract)](https://blog.getzep.com/lies-damn-lies-statistics-is-mem0-really-sota-in-agent-memory/)
- Self-hosted comparison axes in Bkankim/memory-bench: recall and latency plus offline operation, isolation, audit and telemetry, presented as a decision matrix. — [GitHub](https://github.com/Bkankim/memory-bench)
- Memory-injection security:
  - MINJA (Dong et al., arXiv:2503.03704, Mar 2025; NeurIPS 2025) injects malicious records into an agent's memory through query-only interaction, with no direct write access. It uses bridging steps, an indication prompt and progressive shortening, so that later unrelated victim queries retrieve the poisoned record.
  - Recommended defences: memory isolation, user authentication, secure memory management and prompt filtering.

  — [arXiv 2503.03704](https://arxiv.org/html/2503.03704v2); [NeurIPS 2025](https://neurips.cc/virtual/2025/poster/118152)
- A January 2026 paper (EHR agents) reports MINJA reaching above 95% injection success and 70% attack success under idealized conditions. It notes that robustness in realistic deployments and effective defences remain understudied. — [arXiv 2601.05504](https://arxiv.org/pdf/2601.05504v1)

### Inferences
- For rewire, the non-accuracy release table should include:
  - ingest throughput and cost (rewire's LLM-free ingest by default is a real differentiator: $0 LLM cost per ingested message, which should be stated alongside time per 1k messages);
  - query latency p50/p95 for the Python and Rust paths;
  - index size on disk;
  - retrieved tokens per query (which drive downstream LLM cost);
  - cold-start time;
  - fully offline operation (static embeddings, no network);
  - portability (plain markdown plus git: human-readable, diffable, versioned).
- Security for a git-backed, MCP-exposed store:
  - document that retrieved memory content is untrusted data, i.e. a stored prompt-injection risk;
  - consider provenance metadata per memory (source, author, timestamp; git history helps with this);
  - offer quarantine or review for externally sourced memories;
  - add a small poisoning test (inject an instruction-bearing note, then check whether retrieval surfaces it for unrelated queries and how the MCP tool labels it).
- Robustness checks to include: paraphrased queries, temporal and knowledge-update questions (LongMemEval categories), and abstention (questions with no answer in memory).

### Gaps
- I found no standard, community-agreed release checklist for memory systems. The criteria above are synthesised from vendor practice and critiques.
- I found no published cost-per-query tables comparing multiple open-source memory systems under the same conditions.
