# Analysis: LoCoMo 80 questions, Nemotron 3 Ultra (free)

Run results: [2026-10-14-locomo-nemotron-s20.md](2026-10-14-locomo-nemotron-s20.md). J 88.8%, F1 0.232, BLEU-1 0.163,
all evidence in context for 67.5% of questions. This analysis uses no LLM: retrieval for the same 80 questions was reproduced
exactly (same seed, 67.5%), and then one variable was changed at a time.

## 1. What is actually low

| Number | Value | Reading |
|---|---|---|
| J | 88.8% (±6.9 points, n=80) | Not low; close to the 94% of iterations 6–7 (Claude, `full` mode, k=50). The answerer is also the judge, so it tends to be lenient (e.g. "Banff" for gold "Rockies, Jasper" was judged correct). |
| F1 / BLEU-1 | 0.23 / 0.16 | Low. Answers average 69 words (temporal 150 words) while gold answers are 1–5 words. The biggest cause: the `ANSWER:` parser is sensitive to capitalization and Markdown, so `**Answer:**` or `Final answer:` caused the whole reasoning to be counted as the answer. Fixed (see §3). |
| All evidence present | 67.5% | Low for multi-hop (35%) and open-domain (50%). This is the real upper bound. |

## 2. Why multi-hop / open-domain retrieval fails

26 of the 80 questions have incomplete evidence: multi-hop 13, open-domain 10, temporal 2, single-hop 1. The patterns:

- **Aggregation / list questions.** "What causes has John done events for?", "What people has Maria met...?",
  "What writing classes has Maria taken?" need 3–5 turns from different sessions. With 10 excerpts, the top
  excerpts often come from the same single session.
- **Lexical gap and world knowledge.** "France" vs the evidence "Paris"; "roadtrips" vs "a trip
  with my family"; "stress reliever" vs "yoga ... sunsets". BM25 cannot bridge these.
- **Evidence without entity names.** Evidence turns often do not mention the person's name ("I took a dog grooming
  course"); the name appears in a different turn.
- **Noise.** Context precision is 15.6%: one excerpt contains 6 turns, so most of the context is irrelevant.

## 3. Ablation (the same 80 questions, no LLM; percent of questions with all evidence in context)

| Configuration | single | temporal | multi-hop | open | **overall** | context |
|---|---|---|---|---|---|---|
| This run: episodes, keyword, k=30, 10 excerpts | 95 | 90 | 35 | 50 | **67.5** | 2.5k |
| + 20 excerpts | 95 | 95 | 65 | 60 | **78.8** | 4.9k |
| + semantic (WordLlama), 10 excerpts | 90 | 90 | 60 | 50 | **72.5** | 2.3k |
| + semantic, 20 excerpts | 95 | 95 | 65 | 65 | **80.0** | 4.5k |
| `full` mode (L2 facts), k=30, 10 excerpts | 95 | 90 | 50 | 50 | **71.2** | 3.3k |
| `full` mode, k=50, 20 excerpts | 95 | 95 | 65 | 65 | **80.0** | 6.0k |

n=20 per category, so per-category differences below ±15 points are not yet meaningful. For `full` mode, this metric
counts only transcript excerpts; facts that already contain the information (e.g. "Jolene bought a pendant in
Paris") are not counted, so the benefit of `full` is understated here.

Conclusion: **the number of excerpts is the cheapest lever** (multi-hop 35 → 65%), and the context stays ≈5× smaller
than the full history (≈26k tokens). Semantic helps when there are few excerpts. This run also does not test rewire's
core features (consolidation into facts + cross-language keywords), because it uses `episodes` mode.

## 4. What other systems do (research)

- **Hindsight** (91.4% LongMemEval, 89.6% LoCoMo according to its paper): multi-strategy retrieval (vector + BM25 +
  entity graph + time filter) fused with RRF and then reranked by a cross-encoder; at write time, entities
  are resolved and linked. Its advantage is largest on multi-session and open-domain.
- **Mastra observational memory** (94.9% with GPT-5-mini, 84.2% with GPT-4o): an Observer agent continuously writes
  concise observations, and a Reflector merges them; the raw history is replaced by the observation log.
- **T-Mem**: a dedicated "bridge" route for multi-hop that pulls evidence across scenes, separate from the entity route.
- **Second-hop retrieval** (PRISM proposal): for list and multi-hop questions, a follow-up query from the first results,
  fused with RRF. The motivation matches our finding: multi-hop evidence is ranked low.

## 5. Recommendations, ordered from lowest cost

1. **Fix the answer parser** (done): F1/BLEU now measure the answer, not the reasoning.
2. **Adaptive excerpts:** 20 excerpts for list/aggregation/multi-hop questions ("what … has X", "how many", plural
   forms), 10 for the rest. rewire already has a count intent; extend it to list questions.
3. **A judge different from the answerer** for a credible J (another cheap model), and calibration on ±100 questions.
4. **Entity-based second hop:** take names and nouns from the first results, search again, fuse with RRF.
   Bridges "Evan" → turns without the name, and part of the lexical gap.
5. **Smaller excerpts + neighbor expansion:** single turns as the ranking unit, then ±1 turn
   around them: precision rises, and more distinct evidence fits in the same budget.
6. **Evaluate `full` mode** (facts + cross-language keywords from consolidation), because that is how rewire is actually
   used. With free models: ±2 calls × 272 LoCoMo sessions; the daily limit becomes the constraint.
7. Further out: an entity index (who → which turns) and a reranker; both are proven in Hindsight.

Sources: [Hindsight (arXiv 2512.12818)](https://arxiv.org/html/2512.12818v1) ·
[Mastra observational memory](https://mastra.ai/research/observational-memory) ·
[T-Mem (arXiv 2606.15405)](https://arxiv.org/pdf/2606.15405) ·
[PRISM issue #94: second-hop retrieval](https://github.com/dwamianm/prism/issues/94) ·
[QueryLink (ACL Findings 2026)](https://aclanthology.org/2026.findings-acl.765.pdf) ·
[LoCoMo overview](https://www.emergentmind.com/topics/locomo).

## 6. Ablation on all 1,540 questions (no LLM, 2026-10-14)

Percent of questions with all evidence in context (`episodes` mode, k=30):

| Configuration | single | temporal | multi-hop | open | **overall** | context |
|---|---|---|---|---|---|---|
| keyword, 10 excerpts | 94.5 | 83.4 | 32.3 | 40.2 | **77.5** | 2.5k |
| keyword, 20 excerpts | 97.5 | 88.1 | 44.3 | 50.0 | **82.9** | 4.9k |
| semantic, 10 excerpts | 92.6 | 80.9 | 28.4 | 40.2 | **75.2** | 2.4k |
| semantic, 20 excerpts | 97.1 | 90.0 | 42.6 | 47.8 | **82.7** | 4.6k |

On the full sample, semantic (WordLlama) does **not** help LoCoMo (the same as the iteration 6 finding); the gain on the
80-question sample above was noise. 20 excerpts give +5.4 points overall and +12 points multi-hop, but multi-hop
stays at 44%: adding more excerpts alone is not enough; the search method needs to change (entities, smaller units).

## 7. Retrieval improvement: excerpt shape (no LLM, 2026-10-14)

Question: with the same token budget, can the context hold more evidence? All numbers: all
1,540 LoCoMo questions, `episodes` mode, keyword, k=30 facts; "evidence" = percent of questions with all evidence in context.

| Variant | k=10 | k=15 | k=20 | k=25 | Result |
|---|---|---|---|---|---|
| Baseline (6-turn window, step 4) | 77.5% / 2,458 tok | – | 82.9% / 4,795 tok | – | reference |
| Unit of 1 turn ± 1 neighbor (then merged) | – | – | – | 74.4% / 2,655 tok | **worse** at the same tokens |
| **Merge overlapping excerpts** | 77.5% / 2,196 tok | 80.4% / 3,223 tok | 82.9% / 4,219 tok | 84.8% / 5,174 tok | **same evidence, tokens −11–12%** |
| Trim lines that do not contain question words (±1/±2 lines) | – | – | – | – | evidence −0.2 to −2 points |
| Pseudo-relevance feedback (top 3/5/10 excerpts) + merge | 77.1–77.7% | 80.5–80.6% | 82.9–83.2% | – | neutral (within noise) |

Why merging works: the 6-turn window slides by 4, so two adjacent excerpts share 2 turns; when
both are retrieved, the same turns are sent twice. Overlapping or contiguous excerpts from one session are now
combined into a single excerpt (each line once), at the rank of its best excerpt. Session summaries are never
merged.

Why small units and trimming fail: LoCoMo evidence is often in turns that do not contain the question words
(the answer is in the turn before/after the matching one); cutting the context around keywords throws that evidence away.
PRF adds words from the top excerpts, but the words that appear frequently there are mostly general chat topics, not
bridges to the second piece of evidence.

**LongMemEval_S (120 questions, 20 per type, keyword):** identical evidence (turns 87.6%, all evidence 85.5%, sessions 94.9%);
context 11,090 → 9,780 tokens (−11.8%), MRR 0.709 → 0.794, context precision 16.9% → 21.3%; retrieval latency
unchanged (p50 ≈170 ms).

**Applied:** merging is now the default behavior in Python (`Index._merge`) and Rust (`merge_excerpts`); Rust–Python
recall parity remains 304/304 (keyword) and 298/304 (semantic, the same as before). The other variants were not
kept.

**Still unsolved:** multi-hop (32–44%) and open-domain. Raw transcripts alone are not enough; the most promising next
step is a second hop through entities/facts (`full` mode, which previously reached J 94%
at k=50), measured at a fixed token budget (≈2.5k), rather than adding more excerpts.

**Re-evaluation with an LLM** (gpt-4o-mini as answerer and judge) is run in the evaluation session, following
the runbook `docs/evaluation.md` §9: this session does not have `OPENROUTER_API_KEY`. Because replies are cached per prompt,
only questions whose context changed are paid for again.

## 8. Ideas from other systems: entities, second hop, session diversity (no LLM, 2026-10-14)

Sources of ideas: Mem0 2026 (entity boost via spaCy, damped for frequently occurring entities), Hindsight and
CortexDB/OpenHuman (an entity route in RRF, results interleaved across sessions/scopes), SmartSearch and the
Entity–Memory Graph paper (rule-based entity expansion). OpenHuman v2 itself no longer has local retrieval:
ranking and multi-hop run on the closed CortexDB server; only the client layer is open (a budget of
1,200 tokens per section, dates as a boost rather than a filter; rewire already does this).

**Diagnosis first:** the multi-hop evidence exists, but it is ranked deep. `episodes` mode, keyword:

| Excerpts | Context | Multi-hop | Open-domain | Overall |
|---|---|---|---|---|
| 10 | 2.2k | 32.3% | 40.2% | 77.5% |
| 20 | 4.2k | 44.3% | 50.0% | 82.9% |
| 50 | 9.7k | 61.3% | 57.6% | 89.1% |
| 100 | 17.9k | 79.8% | 78.3% | 94.5% |
| 200 | 26k (≈ the whole conversation) | 99.6% | 100% | 99.9% |

**Results (percent of questions with all evidence, k=10 / k=20):**

| Variant | Overall | Multi-hop | Result |
|---|---|---|---|
| Baseline | 77.5 / 82.9 | 32.3 / 44.3 | reference |
| Max. 1 excerpt per session (the rest follow afterwards) | 70.2 / 73.2 | 27.3 / 33.7 | worse |
| Max. 2 excerpts per session | 75.8 / 79.9 | 31.6 / 42.2 | worse |
| Entity route (capitalized words in the question, df ≤ 5%) in RRF | 77.5 / 82.8 | 32.3 / 44.3 | neutral |
| Second hop through entities from the top 5 excerpts, weight 0.5 | 71.3 / 80.4 | 25.5 / 40.8 | worse |
| Second hop, weight 1.0 | 70.2 / 76.5 | 23.4 / 35.8 | worse |

Why it does not work on LoCoMo: almost all of the failing multi-hop questions are aggregation questions whose vocabulary
differs from their evidence ("Which countries has Deborah traveled to?" ↔ "I was in Rio de Janeiro"; "favorite
desserts" ↔ "coconut milk icecream"; "as a child" ↔ "as a kid"). The entities in the question are usually only the
speaker's name, which appears in almost every excerpt; new entities from the top results bring in other topics, not the second
piece of evidence. Evidence is usually spread over many sessions, but limiting excerpts per session actually throws away evidence
whose session is genuinely relevant.

**What helps: facts (`full` mode) at the same token budget.** Evidence is also counted when a retrieved
fact was extracted from an evidence turn (`experiments/locomo/evidence.py --facts`; 1,535 questions, full-mode brain
from an earlier iteration):

| Context contents | Context | Multi-hop | Open-domain | Overall |
|---|---|---|---|---|
| episodes, 15 excerpts | 3.2k | 36.9% | 45.7% | 80.4% |
| full: 30 facts + 10 excerpts | 3.0k | 41.5% | 44.6% | 82.5% |
| full: 50 facts + 10 excerpts | 3.7k | 45.4% | 50.0% | 83.8% |
| episodes, 20 excerpts | 4.2k | 44.3% | 50.0% | 82.9% |
| **full: 30 facts + 20 excerpts** | **4.7k** | **51.4%** | **52.2%** | **86.4%** |
| full: 100 facts only | 3.4k | 35.1% | 47.8% | 66.4% |

Facts alone lose detail (single-hop only 74%), but facts + their source excerpts give +3.5 points overall
and +7 points multi-hop compared with excerpts alone at a context of ≈4.5k. The cost is at ingest (one LLM extraction per
session), not at query time.

**Conclusion:** none of the entity-based variants was adopted. The remaining next levers:
1. `full` mode for LoCoMo/LongMemEval in the evaluation session (gpt-4o-mini ingest cost, once per brain).
2. A small contextual encoder (bge-small / e5-small, ONNX) replacing WordLlama, and/or a small cross-encoder
   to rerank the top 50 candidates. This is the most likely way to close the vocabulary gap, but the
   models cannot be downloaded in this session (Hugging Face and the fastembed GCS mirror are rejected by the proxy, 403).

## 9. Toward 80% multi-hop (2026-10-14)

The 79.8% multi-hop figure from CortexDB/OpenHuman is **J** (LLM answers graded by a judge, a vendor claim),
not "all evidence in context". J is more lenient: an answer can be correct from partial evidence. In the Nemotron run
(80 questions), multi-hop J was 95% while complete evidence was only 35%; the old `full` mode run at k=50 (885 questions, Mem0 judge):
multi-hop J 96.5%. So the 80% target has to be proven with gpt-4o-mini J in the evaluation session.

For the evidence measure (no LLM, 1,535 questions, `full` mode, keyword unless stated):

| Facts + excerpts | Context | Multi-hop | Open-domain | Overall |
|---|---|---|---|---|
| 30 + 20 | 4.7k | 51.4% | 52.2% | 86.4% |
| 30 + 20, semantic | 4.2k | 52.8% | 54.3% | 86.4% |
| 50 + 20 | 5.3k | 56.0% | 57.6% | 87.8% |
| 30 + 40 | 7.6k | 61.7% | 59.8% | 89.2% |
| 100 + 20 | 7.1k | 64.2% | 68.5% | 90.3% |
| 100 + 20, semantic | 6.8k | 66.0% | 66.3% | 90.0% |
| 200 + 20 | 10.0k | 71.6% | 81.5% | 92.7% |
| 100 + 40 | 10.2k | 73.8% | 71.7% | 92.7% |
| episodes only, 100 excerpts | 17.9k | 79.8% | 78.3% | 94.5% |

Per token, facts are more efficient than excerpts (100+20 at 7k tokens beats 30+40 at 7.6k). Complete multi-hop evidence
at 80% needs ±12k tokens; that is not the goal, because a large context lowers answer precision. The configurations
to test with J: 30+20 (≈4.7k) and 100+20 (≈7k, comparable to Mem0 2026 with 200 facts ≈7k tokens, J 91.6% using
GPT-5). The commands are in the runbook `docs/evaluation.md` §9.

## 10. Where rewire retrieval stands relative to RAG techniques (NirDiamant/rag_techniques), 2026-10-14

**Strong:** single-hop 94–99% and temporal 83–95% complete evidence at 2–5k tokens; queries without an LLM; retrieval
in milliseconds (LoCoMo p50 4 ms); overlapping excerpts are merged.
**Weak:** multi-hop and open-domain. Evidence whose vocabulary differs from the question is ranked deep (multi-hop MRR
0.55); static embeddings (WordLlama) do not close that gap, and there is no reranker. High-scoring systems without an LLM
at query time (SmartSearch, Hindsight) use cross-encoders and contextual encoders.

| Technique (number in the repo) | Status in rewire |
|---|---|
| Proposition chunking (5) | present: `full` mode facts |
| Choose chunk size (4), semantic chunking (12) | tested: 6 turns / step 4 is better than a 1-turn unit |
| Relevant segment extraction (10), context window enrichment (11) | present: excerpt merging (§7) |
| Contextual compression (13) | tested (line trimming without an LLM): worse |
| Fusion retrieval (15) | present: BM25 + WordLlama, RRF |
| Intelligent reranking (16) | not yet: a cross-encoder needs a model file (blocked by the proxy in this session) |
| Multi-faceted filtering (17) | present: time window as a boost |
| Hierarchical indices (18), RAPTOR (32) | partial: session summaries are indexed; the LoCoMo brain does not yet have per-session summaries |
| Contextual chunk headers (9) | partial: dates and summaries are included in results; not yet tested in the index |
| Dartboard / diversity (20) | tested (per-session cap): worse |
| Feedback loops (22) | present for facts: ACT-R activation from `use` events |
| Adaptive retrieval (23) | tried before (count intent): removed, answers over-counted |
| Graph RAG, LightRAG, GraphRAG (29–31, 36) | partial: relations + spreading activation; entity hop tested: worse (§8) |
| Query transformation, HyDE, HyPE-query, Self/Corrective RAG, agentic (6–8, 28, 33–34) | not a fit: LLM at query time (the calling agent can already rewrite the query itself) |
| **Document augmentation / question generation (14), HyPE (8)** | **most promising**: LLM at ingest, queries stay LLM-free |

**Test of document augmentation with the existing facts** (the excerpt index is augmented with the text of facts extracted from
its turns; the text sent is unchanged; 1,535 questions, keyword):

| Configuration | Context | Multi-hop | Overall |
|---|---|---|---|
| 10 excerpts, plain index | 1.7k | 25.9% | 75.8% |
| 10 excerpts, index + facts | 2.0k | 31.9% | 80.2% |
| 20 excerpts, plain index | 3.3k | 38.3% | 81.2% |
| 20 excerpts, index + facts | 3.9k | 43.6% | 84.6% |
| 30 facts + 20 excerpts, plain index | 4.7k | 51.4% | 86.4% |
| 30 facts + 20 excerpts, index + facts | 5.0k | 50.0% | 86.4% |

The vocabulary from extraction makes excerpts easier to find (+3–4 points at equal tokens when the context is excerpts
only), but when facts are already sent, seeding facts → source excerpts already gives the same effect.
Not adopted. This suggests that hypothetical questions per excerpt (technique 14), which use the form of a
question ("Which countries has Deborah visited?"), could give more; testing it needs an LLM at ingest.

## 11. Contextual encoder, hypothetical questions, query decomposition (2026-10-14)

All: 1,535 LoCoMo questions, `episodes` mode, percent of questions with all evidence in context.

**Contextual encoder (all-MiniLM-L6-v2, ONNX, from Chroma's S3 mirror).** Hugging Face is blocked by the proxy; this model is
the only contextual encoder that can be downloaded in this session. A cross-encoder could not be tested yet (no reachable
mirror).

| Variant | k=10 | k=20 |
|---|---|---|
| BM25 (current) | 77.5% / 2.2k tok | 82.9% / 4.2k tok |
| BM25 + MiniLM (RRF) | 76.5% / 2.1k | 83.5% / 4.0k |
| MiniLM only | 65.8% / 2.0k | 76.0% / 3.8k |

Neutral, just like WordLlama: BM25 is strong on LoCoMo because questions and evidence often share names and nouns.

**Hypothetical questions per excerpt (document augmentation / HyPE, techniques 14 and 8).** At ingest, one
LLM call per session (Claude Haiku via `claude -p`, 272 calls) writes 4 questions per excerpt whose
answers are in that excerpt, using category words ("countries visited", "desserts", "as a child"). The LLM
does not see the benchmark questions. The questions are only added to the excerpt FTS index; the text sent is unchanged.

| | k=10 | k=15 | k=20 |
|---|---|---|---|
| Plain index | 77.5% (multi 32.3, temporal 83.4) | 80.4% (36.9 / 86.2) | 82.9% (44.3 / 88.1) |
| + hypothetical questions | **80.6%** (34.8 / 89.1) | **83.1%** (41.8 / 91.6) | **85.0%** (47.2 / 92.5) |

Gains in every category at the same tokens (single-hop +1–2, temporal +4–6, multi-hop +2.5–5, open-domain
+1–4). A 3-conversation pilot (383 questions) briefly showed multi-hop +11 points; on the full set it is +3–5. This is the first
technique since excerpt merging that raises evidence without adding context. Queries stay LLM-free; the cost is
one small-model call per session at ingest.

**Rule-based query decomposition.** 40–52 of the 282 multi-hop questions mention two people ("What have X and Y
both …"). The query is split per person and the results are interleaved: k=10 77.7% (multi-hop 32.3%), k=20 82.9% (44.3%) →
neutral; with hypothetical questions also neutral. LLM-based decomposition needs an LLM at query time (the calling agent
can do it itself through several `recall` calls).

**Summary of techniques tested:**

| Technique | Result |
|---|---|
| Hybrid BM25 + vector (WordLlama, MiniLM) | neutral |
| Parent context: small unit → parent window (1 turn ± neighbors) | worse |
| Parent context: facts → source excerpts (`full` mode) | helps (§8) |
| Parent context: hypothetical questions → excerpts | **helps** |
| Lightweight entity graph: entity route / entity hop | neutral / worse (§8) |
| Relation graph + spreading activation (`full` mode) | already present |
| Query decomposition (rules) | neutral |
| Per-session diversity, PRF, line trimming | worse / neutral |
| Not yet tested: cross-encoder (model unreachable), Personalized PageRank, GraphRAG community summaries | – |

## 12. Best combination per context budget (2026-10-14)

1,535 LoCoMo questions, keyword, percent of questions with all evidence in context. "Q" = hypothetical questions per excerpt
in the index (§11); "full" = facts + source excerpts (§8).

| Budget | Combination | Context | Multi-hop | Temporal | Overall |
|---|---|---|---|---|---|
| ≈2.2k | episodes, 10 excerpts | 2.2k | 32.3% | 83.4% | 77.5% |
| ≈2.2k | episodes + Q, 10 excerpts | 2.2k | 34.8% | 89.1% | 80.6% |
| ≈3k | full: 30 facts + 10 excerpts | 3.0k | 41.5% | 92.5% | 82.5% |
| ≈3k | full + Q: 30 + 10 | 3.1k | 43.6% | 93.4% | 83.3% |
| ≈4.5k | episodes + Q, 20 excerpts | 4.2k | 47.2% | 92.5% | 85.0% |
| ≈4.5k | full: 30 + 20 | 4.7k | 51.4% | 93.8% | 86.4% |
| ≈4.5k | **full + Q: 30 + 20** | 4.8k | **53.9%** | 93.8% | **87.0%** |
| ≈5.5k | full + Q: 50 + 20 | 5.6k | 59.2% | 95.3% | 88.5% |
| ≈7k | full: 100 + 20 | 7.1k | 64.2% | 95.6% | 90.3% |
| ≈7k | **full + Q: 100 + 20** | 7.3k | **67.4%** | 95.9% | **90.9%** |

Q adds +0.6–0.8 points overall and +2–3 points multi-hop on top of `full` mode; without facts, Q adds +2–3
points overall. Both techniques use an LLM only at ingest.

## 13. Latency and processing time (2026-10-14)

**Query (recall), all 1,540 LoCoMo questions, 4-core CPU, keyword:**

| Combination | Python (in-process) p50 / p95 / p99 | Rust CLI (new process per query) p50 / p95 |
|---|---|---|
| episodes, 10 excerpts (current) | 0.6 / 0.9 / 1.2 ms | – |
| episodes + Q, 20 excerpts | 1.4 / 2.3 / 2.8 ms | – |
| full 30 + 20 | 6.0 / 9.4 / 11.7 ms | 14 / 20 ms (conv-26) |
| full + Q 30 + 20 | 6.8 / 11.7 / 13.8 ms | – |
| full + Q 100 + 20 | 13.5 / 21.4 / 24.6 ms | 19 / 26 ms (100 + 20, without Q) |
| full + Q 30 + 20, semantic WordLlama | 7.2 / 10.9 / 13.2 ms | – |

Q adds almost no latency (only longer FTS text). The Rust numbers already include process startup.

**Rust vs Python measured the same way** (new process per query, like a per-prompt hook; conv-26, 152 questions):

| | Process start only | recall full 30 + 20 | recall full 100 + 20 | default recall (k=8, 3 excerpts) |
|---|---|---|---|---|
| Python CLI | 18 ms | 89 ms | – | – |
| Rust CLI, before optimization | 2 ms | 15.5 ms | 19 ms | – |
| Rust CLI, after optimization | 2 ms | 11.1 ms | 12.6 ms | 7.6 ms |

Python in an already-warm process (5.6 ms) looks faster only because the regexes and connection are already prepared.
Profile (callgrind) of Rust before optimization: 47% of instructions went to compiling regexes in every process (the
secret/injection scanner, the time parser, `history_intent`), 24% to `LIKE` queries per seed fact. Fixes: regexes are only
compiled when their required keyword is present in the text (same results, tested), the time parser is skipped when the query does not
contain 4 digits, and the excerpt lines per session file are read once. Instructions per recall dropped from 70 million to ±40
million; Rust–Python recall parity is unchanged (items 304/304; order 302/304, the same as the binary before the
change, because activation depends on time).

**Ingest (once per session, in the background after the session ends, not on the answer path):**

| Step | Time per session | LoCoMo (272 sessions) |
|---|---|---|
| Fact extraction (`claude -p`, from the commit timestamps of the old brain) | 23–26 s | ≈1.8 hours sequential, ≈30 minutes with 4 in parallel |
| Hypothetical questions (`claude -p --model haiku`, 4–6 excerpts per session) | ≈11 s | ≈50 minutes sequential, ≈12 minutes with 4 in parallel |
| Index (FTS, no LLM) | milliseconds | seconds |

The `claude -p` numbers include CLI startup (a few seconds); a direct API (OpenRouter) is faster.
If the questions are generated in the same fact-extraction call, there is no extra call, only more output
tokens.
