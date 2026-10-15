# LoCoMo multi-hop A/B (282 questions)

Updated: 2026-10-15 15:55 UTC

## 1. Pattern: why multi-hop evidence is not retrieved

- The more sessions that contain evidence, the less often all of the evidence is retrieved:

| evidence sessions per question | n | all evidence retrieved |
|---|---|---|
| 1 | 13 | 69% |
| 2 | 164 | 66% |
| 3 | 56 | 52% |
| 4 | 28 | 39% |
| 5+ | 21 | 10% |

- Of the 220 missing evidence turns, the transcript excerpt was at rank 21–100 (171 turns) rather than missing entirely. Only 36 did not make the top 100 at all.
- The main cause: questions use **category words** ("desserts", "cities", "activities", "locations"), while the messages mention **specific examples** ("coconut milk ice cream", "San Francisco", "biking", "Bali"). Keyword search (BM25) cannot connect the two, and people's names appear in almost every message, so they do not help.

## 2. Retrieval tests without an LLM (all evidence, excerpts or facts)

Note: the two expansion rows in this table were measured while semantic search was still active; the clean numbers (without semantic search) are in section 3.

| variant | all evidence | mean context tokens |
|---|---|---|
| baseline (k=30 facts, 20 excerpts) | 56.4% | 5,133 |
| + semantic search (WordLlama) | 53.9% | 4,923 |
| k=50 facts | 56.4% | 5,965 |
| 30 excerpts | 61.3% | 6,795 |
| 40 excerpts | 63.8% | 8,339 |
| 50 excerpts | 67.0% | 9,865 |
| **query expansion** (Haiku writes 4 specific keyword queries, results merged) | **65.6%** | 6,338 |
| **query expansion + 30 excerpts** | **73.4%** | 8,956 |

- The built-in semantic search actually lowers the results, so it is not used.
- Adding facts (k) does not help; what helps is transcript excerpts.
- Query expansion is the most efficient: +9 points for +23% context, equivalent to 45 excerpts without expansion.

## 3. A/B with Haiku (answer + judge + list judge), 282 multi-hop questions

| metric | A: baseline | B: query expansion | difference |
|---|---|---|---|
| J (official judge) | 90.8% | 92.6% | +1.8 |
| strict J (list must be complete) | 77.3% | **81.6%** | **+4.3** |
| complete list answers | 66.1% | 73.4% | +7.3 |
| list items mentioned | 82.3% | 87.4% | +5.1 |
| all evidence (excerpts or facts) | 56.4% | 70.9% | +14.5 |
| mean context tokens | 5,133 | 7,010 | +37% |

- Paired test on the same questions: strict J went up on 20 questions and down on 8 (z = 2.27, significant at 5%). For the official J, 10 went up and 5 went down (z = 1.29, not yet significant).
- Examples that became correct: "What martial arts has John done?" (Kickboxing, Taekwondo), "What European countries has Maria been to?" (Spain, England), "What exercises has John done?" (4 items).
- Examples that became wrong: answers with too many items (3 letters when the gold has 2), or one missing item ("Running, pottery" where the answer only mentions running).

## 4. Proposed improvements

1. **Multi-query recall in rewire.** The `recall` tool accepts several queries at once, and the results are merged with RRF. The calling model (an agent using rewire over MCP) is already an LLM, so it writes the extra queries containing specific examples. rewire itself needs no extra LLM call. The MCP tool description gets a hint: for list or category questions, add queries that name the specific examples. This needs identical Python and Rust changes.
2. **Optional, afterwards:** increase excerpts to 30 only for list questions (the words "what ... has", "which", "how many"), because in this retrieval test that adds complete evidence without burdening other questions.
3. **Not recommended:** the built-in semantic search (WordLlama) and raising the number of facts (k=50). Neither adds evidence.

## 5. Already implemented (PR #2)

- `recall` accepts `also`: up to 4 extra queries from the calling model, merged by rank (MCP `also`, CLI `--also`). Without `also`, the results are exactly the same as before.
- Python and Rust are identical: 52 questions with extra queries and 152 without, 100% identical result order.
- Re-test through the product path (not the prototype): J 92.9%, strict J 81.6%, all evidence 71.3%, the same as arm B.
- Eval harness: `--expand-model` and `--categories`.

## 6. Check of the other categories: 80 questions (20 per category), Haiku

| category | J A | J B | strict J A | strict J B | all evidence A | all evidence B | context tokens A → B |
|---|---|---|---|---|---|---|---|
| multi-hop | 95.0% | 100.0% | 90.0% | 95.0% | 70.0% | 95.0% | 5,168 → 7,057 |
| open-domain | 75.0% | 85.0% | 75.0% | 85.0% | 55.0% | 65.0% | 5,122 → 6,992 |
| single-hop | 100.0% | 100.0% | 95.0% | 100.0% | 95.0% | 100.0% | 5,276 → 6,953 |
| temporal | 95.0% | 95.0% | 95.0% | 95.0% | 90.0% | 90.0% | 5,044 → 6,651 |
| overall | 91.2% | 95.0% | 88.8% | 93.8% | 77.5% | 87.5% | 5,152 → 6,913 |

Questions that changed: 4 became correct (conv-48-19 multi-hop, conv-43-34 and conv-26-50 open-domain, conv-42-24 temporal) and 1 became wrong (conv-44-34 temporal, counting months). No category went down. With 20 questions per category, per-category differences can still be noise (±20 points); the direction is consistent with the 282-question multi-hop test.
