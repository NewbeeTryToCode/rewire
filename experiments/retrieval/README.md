# Retrieval quality

Measures whether `recall` returns the right memory, not how fast it is (that is in [`../perf`](../perf/README.md)).

## Data

- [`dataset.json`](dataset.json): created by a separate LLM ([`make_dataset.py`](make_dataset.py)) — the persona of an Indonesian developer who writes in a mix of Indonesian/English, 3 projects (Go, Flutter, Python), **70 memories**, 22 relations, 6 facts that have been superseded, and **60 questions** with known answers, 10 per category:

| Category | What is tested |
|---|---|
| lexical | the question uses the same words as the memory |
| paraphrase | same meaning, **no important words in common** |
| cross_lingual | Indonesian question, English memory (or vice versa) |
| relational | the answer is a memory **connected** to the memory the question mentions |
| stale | the fact has changed: must get the new one, not the old one |
| scope | a similar memory exists in another project: must get the one in the active project |

- [`keywords.json`](keywords.json): keywords per memory, created with **exactly the same** instructions as consolidation (`KEYWORDS_RULE`); the generator did not see the questions.
- [`agent_queries.json`](agent_queries.json): queries written by an LLM agent for each question, following the guidance in the MCP tool description (`QUERY_GUIDE`); the generator did not see the memories.

Run: `python experiments/retrieval/eval.py`

## Results (R@5 / MRR)

| Configuration | lexical | paraphrase | cross_lingual | relational | stale | scope | **all** |
|---|---|---|---|---|---|---|---|
| Start of iteration (text + graph, v0.3) | 100% / 1.00 | 70%* / 0.52 | 80% / 0.57 | 60% / 0.28 | 80% / 0.70 | 70% / 0.70 | **77% / 0.63** |
| + stopwords, EN/ID stemming, stale facts → their replacements | 100% / 1.00 | 30% / 0.25 | 70% / 0.71 | 90% / 0.39 | 100% / 1.00 | 80% / 0.68 | **78% / 0.67** |
| + keywords from consolidation | 100% / 1.00 | 50% / 0.45 | 100% / 0.86 | 90% / 0.47 | 100% / 1.00 | 100% / 1.00 | **90% / 0.80** |
| + queries written by the agent following the tool guidance | 100% / 1.00 | 100% / 1.00 | 100% / 1.00 | 100% / 0.63 | 100% / 1.00 | 100% / 0.85 | **100% / 0.91** |

\* The initial paraphrase number is **spurious**: the answers were retrieved because they happened to match on common words ("should", "use", "the"). After stopwords were removed, the real number became visible (30%).

R@5 = the correct answer is in the top 5. MRR = the mean of 1/rank of the correct answer (1.0 = always in first position). In no configuration did a stale fact or another project's memory appear in the top 5.

## What was changed because of this eval

1. **Stopwords + light stemming** for English & Indonesian on the query side (`deployed` → `deploy*`, `menyimpan` → `simpan*`).
2. **Stale memories redirect to their replacements**: they are still searched and never displayed, but their energy is passed on via `supersedes` (stale 80% → 100%, MRR 0.70 → 1.00).
3. **Keywords** (synonyms + ID/EN translations) are written by the LLM during consolidation, stored in the frontmatter (`keywords:`, also visible in Obsidian) and indexed.
4. **Query guidance** in the MCP tool description and the skill: the agent is asked to write keywords + synonyms in both languages.

## Limitations — read before trusting the 100% number

- **One model family** created the dataset, the keywords, and the agent queries. Their vocabulary tends to be the same, so these results are **optimistic**. Real data (the user's own memories and questions) will almost certainly score lower.
- **Small**: 10 questions per category, so one question = 10 percentage points.
- Activation (usage weighting) is **not tested** here: the dataset has no usage history.
- The more honest next step: E2 in the [mvp-spec](../../docs/design/mvp-spec.md) — 50 questions from the user's own history, and/or a subset of a public benchmark (LongMemEval/LoCoMo, with the audit caveat about the LoCoMo judge).
