# Error analysis and recommendations: full LoCoMo, Haiku (full mode)

## Summary

- There are 111 wrong questions (7.2%). About 33 of them (30%) are not system errors. The causes: speakers swapped in the question (17), wrong gold answers (8), information that exists only in an image (4), and a judge that does not apply its own rules (4). If these questions were corrected, J would rise from about 92.8% to ≈94.8% (inferred).
- The real system errors are spread evenly. There are 26 on the answerer side: refusing to name something or to draw a conclusion (7), picking the wrong detail (13), and miscounting (6). There are 26 in retrieval: category words in the question do not match the specific names in memory (12), evidence just missed the k cutoff (9), and paraphrase or spelling (5). There are 11 in ingest: relative durations, photo captions, and nicknames were not extracted.
- Open-domain (J 66.7%) is the weakest, for two reasons. First, 51% of its evidence turns do not share a single content word with the question. This run used BM25 only (semantic = 0 in every index). Second, 15 of the 32 wrong OD questions have speculative gold answers, so reasonable answers are still judged wrong.
- The judge is lenient on lists. Of 130 `ok_list_part`, a manual sample of 40 questions shows that only about 30% (≈39 questions) are genuinely missing items. The rest are false alarms from the lexical metric: `gold_items` splits sentences on the word "and", and paraphrases are not recognized. J inflation from lists is about 2.5 points (inferred). The 68.8% items-found figure is too low as a measure.
- Five priorities: (1) gold errata, an independent judge, and a strict list metric; (2) a retrieval budget of k=50/episodes=30, which recovers evidence for 14 of the 51 questions with missing evidence; (3) an answer-prompt variant that dares to name recognized entities and dares to draw conclusions; (4) category words in keywords and hypothetical questions at ingest; (5) multi-query query expansion.

## 1. Analysis method and its limitations

- I read all 111 wrong rows (wrong_answer 60, wrong_retrieval 37, wrong_ingest 13, wrong_abstain 1) together with the original evidence turns in the dataset: speaker, session date, and photo caption (`blip_caption`).
- The context sent to the answerer is not stored in the results JSON, so I reconstructed it. Method: `locomo_prepare` was rerun on a copy of the brain (not the original), with k=30 and episodes=20. For 233 of the 300 questions I checked, the prompt hash exactly matches the original answer cache key, so the context is verified to be identical. For 66 of the remaining 67, the evidence status (excerpt/fact/missing) stays the same. The difference likely comes from time-dependent activation (`recall_beta`) (inferred).
- The reconstruction matches the report: all evidence present is 84.5% (excerpt) and 87.2% (excerpt or fact), while the report says 87.3%.
- The rank of missing evidence was computed in two ways. First, the raw excerpt rank: no merging and no trim, seeded from the same 30 facts. Second, the fact rank with k=300.
- The true error rate within `ok_list_part` was estimated from a random sample of 40 rows (seed 7) that I graded manually. The uncertainty interval is wide, roughly 17–45%.
- Assigning questions to clusters is my per-question judgment. Some questions could fit two clusters, and I chose the most decisive cause.

## 2. Root-cause cluster table (111 wrong questions)

| # | Cluster | n | single / multi / temporal / OD | Problem owner | Example qids |
|---|---|---|---|---|---|
| A | The question swaps speakers: the detail was said by someone else, and the gold follows the mistaken question | 17 | 16 / 0 / 1 / 0 | dataset | conv-43-136, conv-50-151, conv-48-123 |
| B | Wrong gold: year, date, duration, fact, or a film treated as a book | 8 | 1 / 3 / 4 / 0 | dataset | conv-49-80, conv-47-31, conv-43-6 |
| C | The answer exists only in the image, and the BLIP caption does not contain it | 4 | 2 / 0 / 0 / 2 | dataset | conv-42-92, conv-42-91 |
| D | The judge (Haiku) violates its own tolerance rules (dates ±14 days, durations ±50%) | 4 | 2 / 0 / 2 / 0 | judge | conv-48-101, conv-42-24, conv-41-63 |
| E | Open-domain with speculative or world-knowledge gold; the answer is reasonable but different | 15 | 0 / 0 / 0 / 15 | dataset/judge (partly answerer) | conv-43-51, conv-44-53, conv-26-69 |
| F | The answerer refuses to name something or draw a conclusion even though the evidence is present (conflict in the prompt) | 7 | 0 / 2 / 1 / 4 | answer prompt | conv-47-16, conv-43-8, conv-44-18 |
| G | The answerer picks or reads the wrong detail; the evidence is in context (including captions and date arithmetic) | 13 | 11 / 0 / 2 / 0 | answerer | conv-30-73, conv-43-76, conv-49-106 |
| H | Counting or aggregation errors (double counting, plans counted as done) | 6 | 0 / 6 / 0 / 0 | answerer | conv-42-79, conv-42-19, conv-50-47 |
| I | Category word vs specific name ("state" vs Tampa, "countries" vs Rio, "religious" vs church) | 12 | 0 / 5 / 1 / 6 | retrieval/ingest | conv-42-87, conv-47-6, conv-48-19 |
| J | Evidence just missed: ranked right below the k or episodes cutoff | 9 | 0 / 4 / 2 / 3 | retrieval (budget) | conv-42-53, conv-44-16, conv-49-43 |
| K | Paraphrase or spelling with no shared word ("imposter" vs "impostors", "kid" vs "with my dad") | 5 | 0 / 3 / 1 / 1 | retrieval | conv-47-17, conv-50-40, conv-47-11 |
| L | Extraction misses details: relative durations, references to photos, nicknames, small sentiments | 11 | 3 / 3 / 4 / 1 | ingest | conv-42-8, conv-49-95, conv-42-14 |

The total is 111. Compared with the diagnosis labels: all 17 questions in A and 7 of the 8 questions in B are labeled `wrong_answer`. This means 24 of the 60 `wrong_answer` (40%) are actually problematic gold answers, not answerer failures.

A coarser grouping:

| Group | Clusters | n | Share of wrong |
|---|---|---|---|
| Not a system error (dataset or judge) | A, B, C, D | 33 | 30% |
| Subjective (speculative OD) | E | 15 | 14% |
| Answerer | F, G, H | 26 | 23% |
| Retrieval | I, J, K | 26 | 23% |
| Ingest | L | 11 | 10% |

### 2.1 Where the missing evidence is (51 questions: wrong_retrieval, wrong_ingest, wrong_abstain)

Worst raw excerpt rank per question:

| Rank | Questions |
|---|---|
| ≤ 20 | 2 |
| 21–40 | 14 |
| 41–80 | 13 |
| > 80 | 22 |

Most questions in the > 80 group have a content-word overlap of 0 with the question.

Retrieval budget simulation (no LLM, 1,535 questions that have evidence, same brain):

| k facts / episodes | All evidence (excerpt) | All evidence (excerpt or fact) | Context tokens (estimate) | Missing-evidence questions recovered (of 51) |
|---|---|---|---|---|
| 30 / 20 (this run) | 84.5% | 87.2% | 4.8k | 0 |
| 50 / 20 | 84.8% | 88.3% | 5.6k | 5 |
| 30 / 30 | 86.6% | 88.8% | 6.5k | 8 |
| 50 / 30 | 87.4% | 90.0% | 7.3k | 14 |
| 50 / 40 | 88.5% | 90.7% | 8.9k | 16 |
| 100 / 40 | 88.5% | 91.7% | 10.9k | 21 |
| 30 / 20 without trim | 84.6% | 87.4% | 5.0k | 2 |

Excerpt trimming does almost no harm (only 2 questions). The k cutoff and vocabulary are the main problems.

### 2.2 Judge inflation on lists (130 `ok_list_part`)

- The report shows list J at 95.4%, but items found at only 68.8%. If every list answer were scored by the share of items found lexically, total J would drop to 87.5%. That is an overly pessimistic lower bound.
- 36 rows were judged correct by the judge but have 0% lexical items, and most of them are actually complete answers in different words. Example: conv-26-78 (gold "Figurines, shoes", answer "pink running sneakers … wooden figurine dolls"). Other examples are conv-26-87 and conv-50-15. In both, `gold_items` splits a reason sentence on the word "and" into two items.
- Manual sample of 40 `ok_list_part` rows: 12 are genuinely missing items (30%). Of those 12, 9 are because the evidence for the missing item was not retrieved (conv-43-35 surfing, conv-41-25 Spain, conv-43-7 Smoky Mountains). The other three are because the answerer skipped an item that was in context (conv-42-61, conv-48-84, conv-41-74).
- From this sample, my estimate (inferred): about 39 of the 130 questions (rough range 22–59) were accepted only because of the "one item is enough" rule. A strict J for lists is about 2.5 points below the official J, i.e. ≈90.3% overall.
- The "one item is enough" rule is part of the official Mem0 judge, so the official J still needs to be reported for comparability. What needs to be added is a strict, LLM-based list metric, not a replacement for the official J.

### 2.3 Open-domain (96 questions, J 66.7%)

- Of the 32 wrong OD questions: 15 have speculative gold (E), 6 are category vs specific name problems (I), 4 are the answerer refusing to name something (F), 3 just missed k (J), 2 are image-only (C), 1 is paraphrase (K), and 1 is ingest (L).
- The vocabulary problem: 51% of OD evidence turns share no content word with the question. The figure is 38% for multi-hop, 19% for temporal, and 15% for single-hop. With BM25 only, this gap can only be closed by keywords and hypothetical questions from ingest, and neither contains category words yet.
  - Example: the fact "Nate's turtles at Tampa beach" has the keyword `florida`, but not `state`. The question "What state did Nate visit?" puts it at fact rank 95.
  - Another example: the fact "Deborah's earlier Rio trip" has no `brazil`, `country`, or `travel`.
- The answerer is too obedient to one rule in the official prompt: "NEVER generate specific names … that do not appear in any memory". This rule overrides Misc 2 ("you may name it"). Evidence from the cached answer text:
  - conv-47-16: "No memory gives a name for this game, so I won't supply one" (gold: UNO).
  - conv-48-75: gold Exploding Kittens.
  - conv-43-66: refuses to name the Star Wars book.
  - conv-43-8: "No candidate name appears", even though excerpt D3:15 about Under Armour is in context.
- OD gold is cut at ";" (`run.py`), following the Mem0 harness, and this affects 11 questions. The cut drops the reason, which sometimes contains the keyword that would make an answer acceptable to the judge. The impact has not been measured.
- Even when all evidence is present, OD J is only 86.0%, because many questions really demand a world-knowledge guess: Hatha yoga (conv-43-51), park ranger (conv-44-53), Voyageurs (conv-44-44).

## 3. Ranked recommendations

The estimated questions recovered are inferred. The basis: J rises from 75.6% (evidence missing) to 95.9% (evidence complete), plus the clusters above. These numbers have not been verified by a rerun.

| # | Recommendation | Type | Estimated questions recovered | Cost | Py+Rust parity | Safe now? | Evidence qids |
|---|---|---|---|---|---|---|---|
| 1 | Gold errata, independent judge, and strict list metric | eval | 0 system questions; clean J ≈ +2 points, strict J ≈ −2.5 points | low (judge ~3M prompt tokens) | no (Python eval only) | yes | conv-43-136, conv-49-80, conv-48-101 |
| 2 | Retrieval budget k=50, episodes=30 | query-time | 6–10 | +50% context tokens, ≈ +20% answer tokens | no, as long as it is only an eval flag; yes if the `ops.recall` or MCP default changes | yes (as an eval flag) | conv-42-53, conv-47-6, conv-49-43 |
| 3 | Answer-prompt variant: may name recognized entities, must commit to conclusions | answer prompt | 5–9 | low (re-answer ~1,176 calls) | no | yes, as a variant separate from the official prompt | conv-47-16, conv-43-8, conv-44-18 |
| 4 | Category words and hypernyms in `KEYWORDS_RULE` and `QUESTIONS_RULE` | ingest | 5–9 | medium (re-ingest 272 sessions) | yes (`consolidate.py` + `consolidate.rs`, prompt equality test) | not yet (needs a re-ingest budget) | conv-42-87, conv-48-19, conv-44-20 |
| 5 | Multi-query query expansion by the calling model | query-time | 6–12 | medium (1 small LLM call per question) | yes if it goes into `ops.recall` or MCP; no if prototyped in eval | eval prototype: yes | conv-50-56, conv-26-59, conv-50-40 |
| 6 | Extraction rules for relative durations, captions, and nicknames | ingest | 4–7 | medium (re-ingest) | yes | not yet | conv-42-8, conv-49-95, conv-42-14 |
| 7 | Counting and aggregation rules in the answer prompt | answer prompt | 2–3 | low | no | yes, as a variant | conv-42-79, conv-42-19, conv-50-47 |
| 8 | Spelling variants and compound words in stemming | query-time | 1–2 | low | yes (`text.py` + `text.rs`) | yes, with tests | conv-47-17, conv-49-95 |
| 9 | Eval hygiene: remove LoCoMo names and questions from ingest prompt examples, record the retrieval configuration in the report, fix the 11 failed approves | eval/ingest | 0–1 | low | yes for the prompt; no for the report | yes | conv-48-87, `approve failed` log |
| ✗ | Don't: a rule to "just answer for the other speaker" so that cluster A passes | — | (≈17 on LoCoMo) | — | — | no | conv-43-136, conv-50-151 |

### Details per recommendation

**1. Gold errata, independent judge, and strict list metric (eval, safe now, no parity).**

Steps:
- Store the list of qids with problematic gold (clusters A, B, C; 29 questions) in `experiments/eval/` together with the reasons. Report the official J and J without errata side by side.
- Rerun the judge with a model from another family, for example gpt-4o-mini as Mem0 does, or Sonnet. This works from the answer cache, without re-answering. Measure the agreement rate with the Haiku judge.
- Add an LLM-based list metric: how many gold items are covered, with paraphrases counted.
- Fix `gold_items` so that gold answers in sentence form are not split on the word "and" (examples conv-26-87, conv-50-15).

Why: the judge is the same as the answerer (the report already warns about this), and the 4 questions in D show the judge is inconsistent. For example conv-48-101: "a year ago", counted from January 2023, becomes "January 2022", but it was judged wrong. Another example, conv-42-24: 18 June vs "weekend after 3 June" is still within 14 days. Without this step, improvements 2–8 cannot be measured reliably, because the label noise (≈33 questions) is larger than the gain expected from each improvement.

**2. Retrieval budget k=50 facts, episodes=30 (query-time, safe as a flag).**

In the simulation, this setting raises "all evidence (excerpt or fact)" from 87.2% to 90.0%. Evidence is also recovered for 14 of the 51 questions with missing evidence. Examples are conv-42-53 (excerpt D5:4 at rank 23), conv-49-43 (fact D7:2 at rank 45), and conv-47-6 (Stamford fact at rank 34).

The cost: context tokens rise from ≈4.8k to ≈7.3k. The risk: a longer context may slightly reduce the answerer's precision (inferred). So run an A/B test on a stratified sample before changing the default.

There is a cheaper alternative: k=50 with episodes=20 adds only ≈0.8k tokens and recovers 5 questions. If the product default (`ops.recall` or MCP) also changes, Python and Rust must be kept in sync, because recall is currently identical 304/304.

**3. Answer-prompt variant (answer prompt, Python only, safe as a variant).**

The answerer interprets the "NEVER generate specific names" rule as a ban on naming things recognized from a description. Examples are conv-47-16 (UNO from a description of the game rules) and conv-48-75 (Exploding Kittens). The answerer also refuses to connect two memories, as in conv-43-8 and conv-44-18 (breeder). In addition, the answerer stops at "the memories don't give an exact date" even when the date can be computed (conv-50-62, conv-43-48).

Proposal:
- Add an explicit exception: well-known names that match the description (games, books, brands) may be named.
- For "likely/might", give the best guess with the word "likely".
- Always convert date ranges into durations.

Important: `ANSWER_GENERATION_PROMPT` is a verbatim copy of the Mem0 prompt. Changing it makes scores no longer comparable with Mem0. So report it as a separate column ("rewire prompt"), and do not replace the official prompt.

**4. Category words in keywords and hypothetical questions (ingest, needs parity, not safe without a re-ingest budget).**

Add the following instructions to `KEYWORDS_RULE` and `QUESTIONS_RULE`:
- For places, also write the city, state, and country, along with their category words ("city", "state", "country", "travel").
- For food, sports, games, and religion, write the category words ("meat", "sport", "console", "religion", "church").

Evidence:
- conv-42-87: the Tampa fact has `florida`, but not `state`, so it is at rank 95.
- conv-48-87 and conv-48-19: Rio and Paris without `brazil`, `france`, or `country`.
- conv-44-20: "Chicken Pot Pie" without `meat`.
- conv-26-59: "made for a local church" without `religious`.

These prompt constants live in `src/rewire/consolidate.py` and `rust/src/consolidate.rs`, and there is a test that checks the consolidation prompts are identical. Both must be changed together, and then the brain must be re-ingested (272 sessions, p50 42 seconds per session).

**5. Multi-query query expansion (query-time).**

The calling model generates 2–4 additional queries containing concrete examples. For example, "Which cities did Dave travel to?" is expanded into "trip San Francisco Detroit workshop". The results are fused with RRF.

This targets I, K, and part of E: conv-50-56 (excerpt ranks 129 and 98), conv-50-40 (Tupac in "used to rock a song", rank 75), and conv-26-69. Its shape is consistent with the existing `refers_to` parameter, i.e. context resolved by the calling model.

Prototype it first in `experiments/eval` (no parity). Only if it proves helpful, make it an `ops.recall` or MCP parameter in both implementations.

**6. Extraction rules for missed details (ingest, parity).**

Missed cases:
- conv-42-8: "I've had them for 3 years" did not become a fact, although it should have become a dated fact "since ±2019".
- conv-49-25: "working on it for two years".
- conv-49-95: the words "smart watch" appear only in the caption, and the text only says "it tracks my progress".
- conv-42-14: the fact from "Hey Jo, …" does not keep the nickname.

Add rules:
- Convert relative durations into a start date.
- Merge the caption into the claim when the text refers to "this" or "it".
- Keep nicknames.

Methodology note: the diagnosis counts evidence as "present via a fact" even when that fact lost the detail being asked about. For example conv-42-14 (labeled `wrong_answer`, even though "Jo" is not in the context). So the `wrong_answer` count is slightly too high, and ingest problems are undercounted.

**7. Counting rules (answer prompt, variant).**

Cases:
- conv-42-79: 4 vs 3 screenplays; "started writing another one" and then "wrapped up my second" were counted twice.
- conv-42-19: a trail that was only planned was counted.
- conv-50-47: childhood visits were counted.

Proposal: add the rule "merge reports of the same event; count only what has already happened". The gain is small, and this rule also changes the official prompt.

**8. Spelling and compound words (query-time, parity `text.py` and `text.rs`).**

Cases: "imposter" vs "impostors" (conv-47-17, fact at rank 59, and then the answerer abstained) and "smartwatch" vs "smart watch" (conv-49-95). This is a small, cheap fix, but prone to overfitting. Test it on LongMemEval too.

**9. Eval hygiene.**

- `QUESTIONS_RULE` uses names from LoCoMo (Deborah, Jon, Caroline) and the example "Which countries has Deborah visited?". That example is almost the same as test question conv-48-87 ("Which countries has Deborah traveled to?"). This is a small leak from the test set into the product prompt. Replace them with neutral names and examples (parity in `consolidate.rs`).
- The report does not mention that semantic search was off. The index meta shows `semantic = 0`, because `wordllama` is not installed. Record this together with `SEED_WEIGHT`, `EP_WHOLE`, and the commit SHA in the summary.
- `run.log` records 11 "approve failed" (proposals whose note changed before being approved), so those changes did not make it into the brain. The impact has not been measured.

**Not recommended.** Haiku often already recognizes that the question swaps speakers (conv-50-151: "photography was Dave's, not Calvin's"). Adding a rule to "just answer with the other speaker's detail" would raise the LoCoMo score by up to ≈17 questions, but that amounts to tuning the system on wrong labels. In real use, such a rule would actually cause wrong attributions. It is better to put these questions in the errata (recommendation 1).

## 4. Evidence examples per cluster

Each entry contains the qid, question, gold, answer (summarized), and reason.

### A. Speakers swapped in the question (17)

- **conv-43-136**: "What did Tim say about his injury on 16 November, 2023?" Gold: "The doctor said it's not too serious". Answer: Tim was not injured; he was responding to John's injury. Reason: D18:10 was said by **John**, so the answer is substantively correct.
- **conv-50-151**: "What hobby did Calvin take up recently?" Gold: Photography. Answer: fixing cars, with the note "photography was Dave's, not Calvin's". Reason: D30:1 was said by **Dave**.
- **conv-48-123**: "What did Jolene and Anna discuss while watching the sunset?" Answer: it was Deborah and Anna, and they realized they inspire each other. Reason: D10:17 was said by **Deborah**. Its twin questions, conv-48-75 and conv-48-179, attribute the same turn (D27:12) to Deborah and to Jolene respectively.

Other questions in this cluster: conv-26-94, conv-42-119, conv-42-195, conv-43-86, conv-43-135, conv-43-164, conv-43-166, conv-48-116, conv-48-188, conv-49-24, conv-49-144, conv-50-137, conv-50-147.

### B. Wrong gold (8)

- **conv-49-80**: "When did Evan have a drunken night?" Gold: 9 January 2023. Answer: 9 January 2024. Reason: session D24 is dated 10 January 2024 and the text says "Yesterday", so the answer is correct.
- **conv-47-31**: "How many days did James plan to spend in Canada?" Gold: 19 days. Answer: 9 days. Reason: he left the "day after tomorrow" from 9 July (meaning 11 July) and returned on 20 July, so 9 days is correct.
- **conv-43-6**: "In which month's game did John achieve a career-high?" Gold: June 2023. Answer: July 2023. Reason: the session is 16 July 2023 and the text says "Last week I scored 40", so July is correct.
- Also: conv-41-48 (gold typo "3` July", while the session is 31 July), conv-30-47 (the evidence turn mentions a wholesaler, not a "spot"), conv-42-34 ("Little Women" is a film), conv-48-77 (Phuket was a retreat, not a diving spot), and conv-48-87 (gold "Thailand", while the evidence is Bali in Indonesia).

### C. Image only (4)

- **conv-42-92**: "What kind of lighting does Nate's gaming room have?" Gold: red and purple lighting. Answer: not recorded. Reason: the BLIP caption is only "a gaming room with a computer and a gaming chair".
- **conv-42-91**: "favorite book series about?" Gold: dragons. Reason: the text only says "adventures, magic", and the caption is only "bunch of books".

Also: conv-44-43 (trail map) and conv-47-33 (SD photo).

### D. Inconsistent judge (4)

- **conv-48-101**: "When did Jolene buy her pet snake?" Gold: "A year ago". Answer: "around January 2022". Reason: the session is January 2023, and judge rule no. 4 accepts "last year" converted into a year.
- **conv-42-24**: Gold: "weekend after 3 June". Answer: "weekend of June 18". Reason: the text says "two weekends later"; the difference is ≤14 days and the answer is more precise than the gold.
- **conv-41-63**: Gold: two weeks. Answer: ≈9 days. Reason: still within the ±50% duration tolerance.

Also: conv-47-80.

### E. Speculative open-domain (15)

- **conv-43-51**: "What kind of yoga for building core strength might John benefit from?" Gold: Hatha Yoga. Reason: not in the conversation; pure world knowledge.
- **conv-26-69**: "What personality traits might Melanie say Caroline has?" Gold: thoughtful, authentic, driven. Answer: empathetic, caring, courageous, supported by quotes. Reason: the gold picks arbitrary words.
- **conv-44-53**: Gold: park ranger. Answer: wildlife biologist. Reason: both are reasonable.

### F. Refusing to name something or draw a conclusion (7)

- **conv-47-16**: "the game with different colored cards". Gold: UNO. Answer: does not give a name. Reasoning text: "No memory gives a name for this game, so I won't supply one".
- **conv-43-8**: "Which outdoor gear company likely signed up John?" Gold: Under Armour. Answer: "The memories don't name the company". Reason: excerpt D3:15 "I've always liked Under Armour" is in context.
- **conv-44-18**: "Where did Audrey get Pixie from?" Gold: breeder. Answer: refuses to connect "found a breeder nearby" with Pixie.

Also: conv-48-75, conv-43-66, conv-43-48, conv-50-62.

### G. Picking or reading the wrong detail (13)

- **conv-30-73**: "How does Gina describe the feeling that dance brings?" Gold: magical. Answer: stress relief and joy. Reason: D15:8 "…it's magical!" is in context.
- **conv-43-76**: Gold: MinaLima. Answer: "a photo of a collection of Harry Potter books". Reason: the answerer chose the caption over the direct statement.
- **conv-49-106**: "What did Evan start painting…?" Gold: forest scene. Answer: watercolors. Reason: "forest scene" is in the D8:14 caption in context.

Also: conv-26-85, conv-30-43, conv-41-85, conv-43-87, conv-43-152, conv-44-55, conv-47-53 (arithmetic: April→October answered as "a month"), conv-47-147, conv-48-115, conv-49-103.

### H. Counting or aggregation (6)

- **conv-42-79**: "How many screenplays has Joanna written?" Gold: 3. Answer: 4. Reason: "started writing another" and "wrapped up my second" are the same script.
- **conv-42-19**: Gold: 2 trails. Answer: 3. Reason: the waterfall trail in D14:19 was only a plan.
- **conv-50-47**: Gold: 2 car shows. Answer: 4. Reason: it included childhood visits; the gold is also debatable.

Also: conv-41-62, conv-42-78, conv-42-52.

### I. Category word vs specific name (12)

- **conv-42-87**: "What state did Nate visit?" Gold: Florida. Answer: no state. Reason: the Tampa fact has the keyword `florida` but not `state`; fact rank 95, excerpt rank 77.
- **conv-47-6**: "Does James live in Connecticut?" Gold: likely yes. Answer: no. Reason: "shelter in Stamford" is at fact rank 34 and is recovered with k=50.
- **conv-48-19**: "How many times has Jolene been to France?" Gold: 2. Answer: 1. Reason: "a year ago in Paris" is not linked to "France", rank 104.

Also: conv-26-59 (church vs religious), conv-42-60 (Xenoblade vs console), conv-43-11 (surfing vs sports), conv-43-25, conv-43-40 (Smoky Mountains vs NC/TN), conv-44-20 (chicken vs meat), conv-49-64, conv-50-56, conv-47-0.

### J. Just missed k (9)

- **conv-42-53**: "How many times has Nate taken his turtles on a walk?" Gold: 2. Answer: 1. Reason: D5:4 is at excerpt rank 23, while the cutoff is 20.
- **conv-44-16**: "When is Andrew going to go hiking with Audrey?" Gold: August. Reason: D11:7 is at rank 38, so the answerer picked the other, November plan.
- **conv-49-43**: "How often does Sam get health checkups?" Reason: fact D7:2 at rank 45.

Also: conv-26-66, conv-43-5, conv-43-17, conv-44-48, conv-44-59, conv-50-4.

### K. Paraphrase or spelling (5)

- **conv-47-17** (abstain): "the board game where you have to find the imposter". Gold: Mafia. Reason: the text says "impostors", so the stems do not match; fact at rank 59.
- **conv-50-40**: "artists Calvin used to listen to when he was a kid". Gold: Tupac and Dr. Dre. Reason: there is no "kid" or "artist" in D20:8; rank 75.

Also: conv-47-11, conv-47-24, conv-49-11.

### L. Extraction misses details (11)

- **conv-42-8**: "When did Nate get his first two turtles?" Gold: 2019. Reason: "I've had them for 3 years" did not become a fact, and its excerpt is at rank 25.
- **conv-49-95**: "What does the smartwatch help Evan with?" Reason: "smart watch" appears only in the caption, there is no fact, and the excerpt is at rank 139.
- **conv-42-14**: "What nickname does Nate use for Joanna?" Gold: Jo. Reason: the fact from D7:1 is only "Nate dyed his hair purple", so the nickname is lost.

Also: conv-30-44, conv-42-51, conv-43-49, conv-44-39, conv-49-25, conv-50-31, conv-50-61, conv-50-140.

## 5. Notes from the reviewer-rubric perspective

- **RAG (rag-pipeline-reviewer):**
  - Retrieval is only BM25 and graph spreading, with excerpts seeded from facts. There is no reranker.
  - Context precision is low (8.5%). That is expected for k=30 + 20, but it is a sign that the context is full of noise.
  - A "not enough context" fallback is actually forbidden by the official prompt (Step 7), and that pushes cluster F in the opposite direction: the model abstains in other ways.
  - Baselines and thresholds already exist in `docs/evaluation.md` §8. What is still missing: an allowed regression delta and a "problematic gold" slice.
- **MLE (mle-reviewer):**
  - The judge is the same as the answerer: risk of self-preference.
  - There is a test-question example in the product prompt (a small leak).
  - The official answer prompt must not be tuned on the same test set without a held-out set. Suggestion: tune on 3–4 conversations, then report on the rest.
  - The retrieval configuration (semantic off) is not recorded in the report.
  - Label noise (≈30% of the errors) is larger than the gain from each improvement, so errata and an independent judge need to come first.

The analysis working files (context reconstruction, ranks, clusters) were created in the session scratchpad and were not saved; the per-question table is in `diagnosis-haiku-full.csv` in the project folder.
