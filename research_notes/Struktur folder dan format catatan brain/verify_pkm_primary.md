# Primary-source verification of PKM-method claims (PARA/CODE, Zettelkasten, Evergreen notes, LYT/ACE, Johnny.Decimal, Diátaxis, Nygard ADR)

Scope: every claim in `reports/Struktur folder dan format catatan brain.md` and in `research_notes/Struktur folder dan format catatan brain/para_code.md` + `zettelkasten_moc_jd_adr.md` that is sourced to these methods, checked against the primary page. All pages were fetched in full with curl on **2026-10-28** (access date for every URL below) and converted to text, so the quotes are verbatim from the live pages.

Status key:
- **CONFIRMED**: the primary page says this.
- **CORRECTED**: the primary page says something different. The correct fact and a verbatim quote are given.
- **NOT FOUND**: not on the primary pages I could reach. The claim may still be true elsewhere, for example in a book or on an X post.

Still blocked on 2026-10-28: `blog.linkingyourthinking.com` and `notes.linkingyourthinking.com` (proxy 403 / DNS fail). every.to, help.fortelabs.com and x.com are not on the allow list and were not retried. The LYT "four intensities" and "ACE folder framework" blog posts therefore remain unverified.

---

## 1. PARA (Tiago Forte)

### Takeaway
The four categories, "organize by actionability", the Inbox, and the weekly filing heuristic are all confirmed on fortelabs.com. Five things need correcting:
- The often-quoted "series of tasks linked to a goal, with a deadline" wording is no longer on Forte's site. His current definitions are "desired outcome + deadline/timeframe" and "standard to be maintained + continuous over time".
- Forte gives no numeric depth rule ("1–2 levels" is third-party). He says only "a small number of subfolders", and in 2026 "You don't need a folder hierarchy 7 layers deep".
- The "PARA for Teams" quote is by Nat Eliason (guest post), not Forte.
- "Modular context bundles" is not on the cited blog post. Forte's own primary wording is "pre-assembled bundles of context".
- **New:** in 2026 Forte argues *for* physical PARA folders as the unit you "point" an AI at.

### Cited Findings

**Definitions and categories**

- **Four categories, by actionability, not subject.** CONFIRMED.
  > "Instead of organizing information according to broad subjects like in school, I advise you to organize it according to the projects and goals you are committed to right now. This is what it means to "organize by actionability,""

  — [Forte, The PARA Method (pub. 2023-02-24, upd. 2026-04-15)](https://fortelabs.com/blog/para/) (acc. 2026-10-28)
- **Archives definition.** CONFIRMED.
  > "archives, which include anything from the previous three categories that is no longer active, but you might want to save for future reference: Projects you've completed or put on hold; Areas that are no longer active or relevant; Resources that you're no longer interested in"

  Note: "put on hold" projects go to Archives, so PARA has no separate "parked" state. — [fortelabs.com/blog/para](https://fortelabs.com/blog/para/) (acc. 2026-10-28)
- **Classic definitions.** The notes quote Project = "a series of tasks linked to a goal, with a deadline" and Area = "a sphere of activity with a standard to be maintained over time".
  - **NOT FOUND** on current fortelabs.com. A WordPress API full-text search for "series of tasks" matched no PARA post. These are likely 2017-era wordings since replaced.
  - **CORRECTED** to Forte's current definitions:
    > "A project is any endeavor that has 1) a desired outcome that will enable you to mark it "complete," and 2) a deadline or timeframe by which you'd like it done." / "An area of responsibility has 1) a standard to be maintained that 2) is continuous over time."

    — [Forte, Project People vs. Area People (2022-07-11)](https://fortelabs.com/blog/project-people-vs-area-people-are-you-running-a-sprint-or-a-marathon/) (acc. 2026-10-28)
  - Short forms from Forte's BASB guide:
    > "Projects: short-term efforts (in your work or personal life) that you take on with a certain goal in mind / Areas: Long-term responsibilities you want to manage over time / Resources: Topics or interests that may be useful in the future / Archive: Inactive items from the other 3 categories"

    — [Forte, BASB: The Definitive Introductory Guide (pub. 2023-05-01, upd. 2023-11-23)](https://fortelabs.com/blog/basboverview/) (acc. 2026-10-28)
- **Projects have a finish line; Areas do not.** CONFIRMED in substance (the notes had it from Alfred blog):
  > "Every area of responsibility has a standard to be maintained. And there is no end date or final outcome."

  — [Project People vs. Area People](https://fortelabs.com/blog/project-people-vs-area-people-are-you-running-a-sprint-or-a-marathon/) (acc. 2026-10-28)
- **Project/Area confusion is the common error.** CONFIRMED in substance.
  - Forte's client's "project list" turned out to contain only areas: "Not a single item on this list is a project, according to our earlier definition... Every item on this list is, in fact, an area of responsibility."
  - The "most common PARA implementation error" wording itself is third-party (Alfred), not Forte.
  - Source: [fortelabs.com/blog/para](https://fortelabs.com/blog/para/) (acc. 2026-10-28)

**Inbox, filing and "where will it be used"**

- **Inbox outside the acronym; weekly filing.** CONFIRMED.
  > "I prefer to do this "filing" in one batch about once a week. I pull up my "Inbox" – a separate folder as a temporary holding area where new items accumulate until I have time to put them in their proper place – and process 10-20 items in just a few minutes."

  — [basboverview](https://fortelabs.com/blog/basboverview/) (acc. 2026-10-28)
- **"Organize by where it will be used" heuristic.** The notes listed this as a gap. Now **CONFIRMED** as a three-question cascade:
  > "Ask yourself the following three questions to consider in which project, area, or resource a given piece of information will be most relevant and useful. What project would this be useful? If none: Which area will this be useful for? If none: Which resource does this belong to? If none, it's probably best to archive this information or not save it at all."

  Also: "Don't overthink where you're going to put a certain piece of information within PARA. Remember that you can almost certainly find anything again using a simple search." — [basboverview](https://fortelabs.com/blog/basboverview/) (acc. 2026-10-28)
  - The exact phrase "organize by where you will use it, not where you found it" remains NOT FOUND.

**Folder depth and nesting**

- **Depth "1–2 levels".** **NOT FOUND** as a Forte rule; it comes from a third-party SOP. What Forte does say:
  - "Four top-level folders – Projects, Areas, Resources, and Archives – each containing a small number of subfolders dedicated to each active project, area of responsibility, resource, and archive in your life." — [fortelabs.com/blog/para](https://fortelabs.com/blog/para/) (acc. 2026-10-28)
  - "It can be tempting to try to create a perfect hierarchy of folders from the get-go... However, this approach is not only time-consuming, but it also diverts your attention from what interests you now, requiring too much effort." — [basboverview](https://fortelabs.com/blog/basboverview/) (acc. 2026-10-28)
  - "You don't need a folder hierarchy 7 layers deep or a perfect taxonomy of tags or an elegant knowledge graph – only some loosely organized collections of messy notes that you can point an LLM at (such as with my PARA method)." — [Forte, Finding Alpha (2026-07-22)](https://fortelabs.com/blog/finding-alpha-whats-worth-consuming-in-the-age-of-ai/) (acc. 2026-10-28, read via the fortelabs WP JSON API)
- **Nesting in teams.** CORRECTED on attribution.
  - The "PARA for Teams" post on fortelabs.com is a **guest post by Nat Eliason**, not Forte.
  - The quote is accurate: "The archive layer is the simplest: whenever something stops being relevant, it either goes into the Archive folder in Drive, or gets archived in Asana."
  - The same post allows a deviation: "The one difference here from normal PARA is that I will do nested Areas... a two-tiered Area hierarchy makes life much easier."
  - Source: [PARA for Teams (2018-11-12, guest post by Nat Eliason)](https://fortelabs.com/blog/para-for-teams/) (acc. 2026-10-28)

**Other PARA items**

- **"Hot or Cold" two-folder variant.** NOT FOUND on fortelabs.com (WP search for "Hot Cold folders" returned no matching post). The X post was not checkable.
- **Forte is not rigid about PARA.** CONFIRMED in spirit: "PARA isn't a filing system; it's a production system. It's no use trying to find the "perfect place" where a note or file belongs. There isn't one."
  - This is Forte 2022 (the BASB book), quoted verbatim by Sascha Fast.
  - Source: [zettelkasten.de, Combine BASB and Zettelkasten](https://zettelkasten.de/posts/building-a-second-brain-and-zettelkasten/) (acc. 2026-10-28)

**PARA and AI (2026)**

- **"Modular context bundles for AI".** The report cites this to [Introducing The AI Second Brain](https://fortelabs.com/blog/introducing-the-ai-second-brain/).
  - **NOT FOUND** on that page (pub. 2026-03-13, upd. 2026-04-14). The phrase may be on help.fortelabs.com, which was not checked.
  - **CORRECTED** to Forte's own primary wording:
    > "PARA gives you something more valuable: pre-assembled bundles of context that reflect how you actually work and think. When you ask an AI to help you with a project, you can point it at a Project folder – not scattered fragments that don't belong together."

    — [Forte, Why PARA Is the Key to the AI Era (2026-04-13)](https://fortelabs.com/blog/why-para-is-the-key-to-the-ai-era/) (acc. 2026-10-28)
- **AI Second Brain course.**
  - CONFIRMED: announced 2026-03-13; "3 weeks ... from April 15 to May 1"; "Personal Context Management is replacing Personal Knowledge Management". — [Introducing The AI Second Brain](https://fortelabs.com/blog/introducing-the-ai-second-brain/) (acc. 2026-10-28)
  - Cohort 2: the post is dated 2026-09-16. The Sep 17–Oct 8 dates were NOT FOUND on the page. — [The AI Second Brain Is Back](https://fortelabs.com/blog/the-ai-second-brain-is-back-whats-new-for-cohort-2/) (acc. 2026-10-28)
- **New primary facts relevant to rewire** (all from [Why PARA Is the Key to the AI Era](https://fortelabs.com/blog/why-para-is-the-key-to-the-ai-era/), acc. 2026-10-28):
  - **Archive as designed forgetting:** "It knows how to forget... The Archives category of PARA is designed to facilitate that forgetting without actually deleting anything in case it becomes useful again later."
  - **Minimum viable context:** "LLMs have a limited effective context window... You need to be able to find and provide access to only the "minimum viable context" needed for the task at hand. That means your context has to be well-organized and sorted into pre-defined chunks you can easily point the LLM at."
  - **Search is not enough:** "Even if you could somehow run a search of your entire digital world, search is only good for finding exact (or close) matches of individual files."
  - **No detailed hierarchy needed:** "AIs don't really need a highly precise, detailed hierarchy."
- **Local files as agent memory.** "Local file systems have increasingly become the de facto memory system for most LLMs this year" (Cohort 2 post) — [fortelabs](https://fortelabs.com/blog/the-ai-second-brain-is-back-whats-new-for-cohort-2/) (acc. 2026-10-28)

### Inferences
- Forte's 2026 argument ("point it at a Project folder") is the strongest primary-source counterpoint to the report's "no PARA folder tree" line. But its substance is a **pre-assembled, per-project context bundle**, not the folder itself. The report's live `project` hub page plus its member links deliver the same "minimum viable context" unit inside `nodes/`. The report should say this explicitly: the compiled hub *is* the PARA "bundle".
- Forte files paused projects into Archives. ACE's "simmering" state is therefore a real extension over PARA, not a restatement of it. This supports the report's `stage` field.
- "It knows how to forget" directly supports excluding `sleeping`/`done` hubs from the snapshot (report Phase 1e) while keeping them searchable.

### Gaps
- The book wording (*Building a Second Brain*, *The PARA Method*) was not checked. Only fortelabs.com was.
- every.to (weekly-review article), help.fortelabs.com and x.com (Hot/Cold, weekly-review tweets, Master Prompt tweet) were not fetched. Those claims stay at their original status.

---

## 2. CODE / Progressive Summarization / Intermediate Packets (Forte)

### Takeaway
The layers are confirmed, but the numbering needs one fix. Forte defines **Layer 0** (the original source). Layers 1–4 are captured, bold, highlight, and executive summary. The remix stage is described without a "Layer 5" label. The article also frames notes as a balance between **discoverability (compression) and understanding (context)**, explicitly warning that over-compressed notes "lose all meaning". This is directly relevant to the 600-character claim limit.

### Cited Findings
All Progressive Summarization quotes are from [Forte, Progressive Summarization: A Practical Technique for Designing Discoverable Notes (pub. 2017-12-27, upd. 2023-05-16)](https://fortelabs.com/blog/progressive-summarization-a-practical-technique-for-designing-discoverable-notes/) (acc. 2026-10-28).

- **Layers.** CORRECTED.
  - Layer 0: "Layer 0 is the original, full-length source text."
  - Layer 1: "the content that I initially bring into my note-taking program... I just capture anything that feels insightful, interesting, or useful."
  - Layer 2: "I bold only the best parts of the passages I've imported."
  - Layer 3: "I switch to highlighting... looking for the "best of the best," only highlighting something if it is truly unique or valuable."
  - Layer 4: "For a small number of notes that are the most insightful, I summarize layers 2 and 3 in an informal executive summary at the top of the note, restating the key points in my own words."
  - Remix (not numbered): "And finally, for a tiny minority of sources... I remix them."
- **Timing.** CONFIRMED: "I do this bolding layer at a later time, when I'm already reviewing this note anyway."
- **Opportunistic compression.** The notes listed this as a gap. Now CONFIRMED: "It is a method for opportunistic compression — summarizing and condensing a piece of information in small spurts, spread across time, in the course of other work, and only doing as much or as little as the information deserves."
- **No bolding percentage.** The article gives no "how much to bold" figure ("I have no explicit criteria"). The "10–20%" figure is not Forte's. CONFIRMED.
- **Discoverability vs understanding.** NEW and relevant:
  > "the two priorities we are trying to balance are discoverability and understanding. Making a note discoverable involves making it small, simple, and easy to digest... But we also want to make our notes understandable. This involves including all the context"

  > "If we compress a note too much... we lose the context and it loses all meaning."
- **Note-first.** NEW: "I propose we make the design of individual notes the primary factor, instead of tags or notebooks." Forte also says tags are "too fragile and requires too much maintenance".
- **Intermediate Packets.** The notes had only secondary sources. Now CONFIRMED.
  - Definition:
    > "breaking down their large projects or goals into smaller chunks, which I call "Intermediate Packets."... any piece of work – a slide, a paragraph, a diagram, a quote – can always become a component in a larger work. And not just one larger work, but multiple ones."

    — [Forte, Intermediate Packets in the Wild (2021-09-26)](https://fortelabs.com/blog/intermediate-packets-in-the-wild/) (acc. 2026-10-28)
  - Work-session rule: "end each work session... by completing just one Intermediate Packet... These Intermediate Packets can then be saved to your Second Brain, and re-used the next time you have a similar need." — [basboverview](https://fortelabs.com/blog/basboverview/) (acc. 2026-10-28)
- **CODE steps.** CONFIRMED: "a simple four-step method called CODE, which stands for Capture, Organize, Distill, and Express"; "Keep only what resonates"; "Distill opportunistically, a little bit at a time". — [basboverview](https://fortelabs.com/blog/basboverview/) (acc. 2026-10-28)
- **Divergence/Convergence.** Partly CONFIRMED.
  - Forte defines the terms: ""Divergence" refers to opening up your senses and taking in new sources of information... "Convergence" refers to shutting off sources of distraction and narrowing your focus to arrive at an end result." — [Forte, Divergence and Convergence (2022-05-16)](https://fortelabs.com/blog/divergence-and-convergence-the-two-fundamental-stages-of-the-creative-process/) (acc. 2026-10-28)
  - The explicit mapping "Capture+Organize = divergence, Distill+Express = convergence" was **NOT FOUND** on that post. It is likely from the book.
- **Archipelago of Ideas / Steven Johnson credit.** NOT FOUND on fortelabs.com (WP search did not surface a dedicated post).

### Inferences
- The report maps "first body sentence = L4 summary" (report line 99). This matches Layer 4's position ("at the top of the note"). But Forte reserves L4 for "a small number of notes that are the most insightful". That fits compiled hubs and concept pages better than every atomic claim.
- The compression-vs-context warning is primary support for the report's diagnosis that 600-character claims plus empty hub bodies are "too compressed", and for adding long compiled pages.

### Gaps
- BASB book text (divergence/convergence mapping, Archipelago) was not checked.

---

## 3. Zettelkasten (zettelkasten.de; Luhmann; Ahrens)

### Takeaway
The zettelkasten.de claims are confirmed: the atomicity critique, the Friction Fallacy, the Collector's Fallacy, structure notes and hub notes, and Sascha's position on PARA. Three corrections:
- zettelkasten.de's own Introduction does **not** use the fleeting/literature/permanent triad. That triad is Ahrens.
- Atomicity is defined as "a guiding compass", "not a rigid law".
- Sascha's "deciding upfront added friction" quote is from the forum, not the blog post.

### Cited Findings
- **Atomicity definition.** CONFIRMED and sharpened:
  > "Atomicity refers to the idea that knowledge is made up of discrete building blocks. The Principle of Atomicity is a processing direction in note-taking, aiming for one knowledge building block per note. It is not a rigid law, but a guiding compass. It needs to be contextualised for each application."

  — [zettelkasten.de Introduction](https://zettelkasten.de/introduction/); [zettelkasten.de/atomicity](https://zettelkasten.de/atomicity/) (acc. 2026-10-28)
- **Atomicity-on-entry critiques.** CONFIRMED.
  > "It creates anxiety about being right about atomicity right away." / "It moves a significant proportion of the thinking outside of the Zettelkasten... this is a significant proportion of the whole thinking process."

  Also: "you can treat atomicity as the desired outcome that might be at the end of a long process that happens within the Zettelkasten... notes are not atomic for a long time, but continuously approach atomicity". — [Sascha Fast, The Principle of Atomicity – principle vs implementation (Aug 2025)](https://zettelkasten.de/posts/principle-of-atomicity-difference-between-principle-and-implementation/) (acc. 2026-10-28)
- **Friction Fallacy.** CONFIRMED.
  > "Any note-taking or knowledge system whose marginal cost per note rises as the system grows will eventually become unusable" / "A Zettelkasten must be scale-indifferent. It should remain usable even if flooded with massive amounts of low-quality notes. The "one-million bad notes" thought experiment is a stress test"

  Additional: "Folgezettel and structure-note-first workflows trade simplicity for long-term cost... Structure notes are more flexible, refactorable, and resilient over time." — [zettelkasten.de, The Friction Fallacy](https://zettelkasten.de/posts/the-friction-fallacy/) (acc. 2026-10-28)
- **Collector's Fallacy.** CONFIRMED on the primary blog (the notes cited only the forum): "Let's call this "The Collector's Fallacy". Why fallacy? Because 'to know about something' isn't the same as 'knowing something'." — [zettelkasten.de, The Collector's Fallacy](https://zettelkasten.de/posts/collectors-fallacy/) (acc. 2026-10-28)

All remaining Zettelkasten quotes in this list are from the [zettelkasten.de Introduction](https://zettelkasten.de/introduction/) (acc. 2026-10-28) unless another source is named.

- **Structure and hub notes.** CONFIRMED.
  - "A Structure Note is a Meta-Note: it is a Zettel about other Zettels and their relationships. Luhmann's hub notes served as fast tracks to navigate through the web of notes."
  - Structure notes create a "semilattice structure": one note can sit in several structure notes.
- **Luhmann's register.** NEW (the notes had this as a gap): "His register is purely a list of entry points, not a tag list... the term system had just one entry."
- **Note-type triad.** CORRECTED attribution. zettelkasten.de's Introduction contains **no** "fleeting/literature/permanent" terms (0 matches). The triad is Ahrens's (book not checked here). zettelkasten.de's model is unique ID + atomic content + links with explicit link context + structure notes.
- **IDs.** NEW: time-based IDs such as "202006110955" are recommended. Using the title as the ID is "not recommend[ed]" because "you cannot change the title unless you change any reference to it".
- **Link context.** NEW: "always state explicitly why you made it. This is the link context... If you just add links without any explanation you will not create knowledge."
- **Sascha on BASB/PARA (blog).** CONFIRMED in substance.
  - "BASB speaks the language of action. ZKM speaks the language of knowledge."
  - "the purpose of my PARA is to provide resource collections, sorted into projects"
  - Critique: "This dynamic brings a certain restlessness into the system. Nothing has its fixed place... anything that is complex and exists over a long period of time needs stability and constancy."
  - Source: [Sascha Fast, Combine BASB and Zettelkasten](https://zettelkasten.de/posts/building-a-second-brain-and-zettelkasten/) (acc. 2026-10-28)
- **Sascha on PARA (forum).** CORRECTED source: the friction claim is from the forum, not the blog post.
  > "found it was really difficult to decide upfront whether something should in in Resources or a Zettel (even when I put the Zettelkasten in Resources). It added a ton of cognitive dissonance and friction when writing." / "Only processed ideas and thoughts go in my Zettelkasten... PARA is my system to collect and roughly pre-process if needed."

  — [forum.zettelkasten.de, thread 2615 p2, Sascha, Comment_19558, 2024-02-09](https://forum.zettelkasten.de/discussion/2615/building-a-second-brain-and-the-zettelkasten-method/p2) (acc. 2026-10-28)
- **"PARA is not a second brain... logical form of storage and retrieval".** CONFIRMED, and now attributable to Sascha:
  > "My current hypothesis is that PARA is not a second brain in a way that the Zettelkasten is or can be. It is logical form of storage and retrieval."

  — [forum.zettelkasten.de, thread 2429, Sascha, Comment_16891, 2022-12-02](https://forum.zettelkasten.de/discussion/2429/processing-building-a-second-brain-by-tiago-forte) (acc. 2026-10-28)

### Inferences
- "Atomicity is a direction reached inside the system" supports rewire keeping capture permissive (inbox, long concept pages) and refining toward atomic claims later, rather than forcing 600-character atoms at extraction time.
- Sascha's "restlessness / nothing has its fixed place" critique of PARA is primary support for the report's "status, not file moves" design (stable `nodes/<type>/<slug>-<id8>.md` paths).
- The Friction Fallacy is a test the report's promotion gates must pass: human review cost per item must not grow with brain size.

### Gaps
- The Ahrens book was not checked (triad wording, "one idea per card"). The ~90,000-card Luhmann figure was not checked.
- The Folgezettel ID examples (21/3d7a) were not checked.

---

## 4. Evergreen notes (Andy Matuschak)

### Takeaway
All core claims are confirmed verbatim. Two need correcting:
- The "links should not be generated automatically" claim is **not** on the densely-linked page.
- "Titles as complete claims" comes from a separate note: "Prefer note titles with complete phrases to sharpen claims".

New primary material: a **writing inbox** that must be reliably drained, a **ladder of note types** (log → inbox → evergreen → higher-level APIs/outline notes), and an explicit warning that tags are ineffective.

### Cited Findings
All pages are on notes.andymatuschak.org (acc. 2026-10-28). The site carries no dates.

**Core principles**

- **Atomic, but complete.** CONFIRMED.
  > "It's best to create notes which are only about one thing—but which, as much as possible, capture the entirety of that thing." / "If your notes are too fragmented, you'll also fragment your link network... There's no clear litmus test or correct answer here—just a bunch of tradeoffs." / "if you fragment modules too much, you'll have a cohesion problem."

  — [Evergreen notes should be atomic](https://notes.andymatuschak.org/Evergreen_notes_should_be_atomic)
- **Concept-oriented.** CONFIRMED.
  > "It's best to factor Evergreen notes by concept (rather than by author, book, event, project, topic, etc)." / "start a new note for each book, each project... it can be hard to find what you've written when a concept comes up again later: you have to remember the name of each book or project which dealt with the topic"

  — [Evergreen notes should be concept-oriented](https://notes.andymatuschak.org/Evergreen_notes_should_be_concept-oriented)
- **Titles are like APIs.** The notes had this from secondary sources. Now CONFIRMED.
  > "When Evergreen notes are factored and titled well, those titles become an abstraction for the note itself. The entire note's ideas can then be referenced using that handle... you'll be able to write individual notes which abstract over increasingly large subtrees"

  — [Evergreen note titles are like APIs](https://notes.andymatuschak.org/Evergreen_note_titles_are_like_APIs)
- **Titles as complete claims.** CORRECTED source: this is a separate note.
  > "using complete phrases as note titles helps maintain concept-orientation... These are often declarative or imperative phrases making a strong claim. This puts pressure on me to adequately support the claim in the body."

  Exceptions: "I use nouns and noun phrases in note titles only to define core terms". — [Prefer note titles with complete phrases to sharpen claims](https://notes.andymatuschak.org/Prefer_note_titles_with_complete_phrases_to_sharpen_claims)
- **Associative ontologies.** The notes had this as a gap. Now CONFIRMED.
  > "Let structure emerge organically. When it's imposed from the start, you prematurely constrain what may emerge... often items belong in many places... because everything's already been sorted, further sorting requires undoing existing structure."

  — [Prefer associative ontologies to hierarchical taxonomies](https://notes.andymatuschak.org/Prefer_associative_ontologies_to_hierarchical_taxonomies)
- **Densely linked.** Partly CORRECTED.
  - The page says: "If we push ourselves to add lots of links between our notes, that makes us think expansively... It creates pressure to think carefully about how ideas relate to each other."
  - The page also says: "Finding the right links requires reading old notes, so it's also an organic mechanism for intermittently reviewing the notes."
  - The notes' "Links should be added deliberately, **not generated automatically**" is **NOT FOUND** on this page.
  - Source: [Evergreen notes should be densely linked](https://notes.andymatuschak.org/Evergreen_notes_should_be_densely_linked)
- **Tags.** NEW: "Tags are an easy way to relate heterogenous items, but they're quite a low-signal way of describing relationships... Better to link to associate ideas more precisely." — [Tags are an ineffective association structure](https://notes.andymatuschak.org/Tags_are_an_ineffective_association_structure)
- **"Across projects".** CONFIRMED: "Evergreen notes are written and organized to evolve, contribute, and accumulate over time, across projects." The five listed principles are atomic, concept-oriented, densely linked, associative ontologies, and "Write notes for yourself by default". — [Evergreen notes](https://notes.andymatuschak.org/Evergreen_notes)

**Capture-to-evergreen path (C3; the notes listed this as a gap)**

- **Writing inbox.** NEW:
  > "most notes begin as transient notes. You should be able to capture thoughts without friction... then reliably develop them into evergreen notes over time... a quick way to capture transient notes which clearly isolates them from evergreen notes; and a place to put notes you want to develop further and a practice which reliably drains it (Inboxes only work if you trust how they're drained)" / "If a note doesn't seem sufficiently interesting after a few looks, it's best to archive or delete it."

  — [A writing inbox for transient and incomplete notes](https://notes.andymatuschak.org/A_writing_inbox_for_transient_and_incomplete_notes)
- **Ladder of note types.** NEW. The ladder runs: "Ephemeral scratchings in Daily working log" → "Prompts and incomplete notes in writing inbox" → evergreen notes "in increasingly complex stages" (stubs → definitions → precise declarative notes → "higher-level APIs... notes abstracting over many other notes" → "Outline notes").
  - "Log" notes "accumulate ephemeral observations about a specific practice, system, or project over time".
  - Literature notes and proper-noun notes are "weakly evergreen".
  - "I usually denote a note's "type" with a tag."
  - Source: [Taxonomy of note types](https://notes.andymatuschak.org/Taxonomy_of_note_types)
- **Evergreen vs Zettelkasten.** NEW: "Titles are very important indexes in my system. Zettels normally have numeric identifiers"; Zettels "are mostly meant to represent a few "middle" levels of this hierarchy." — [Similarities and differences between evergreen note-writing and Zettelkasten](https://notes.andymatuschak.org/Similarities_and_differences_between_evergreen_note-writing_and_Zettelkasten)

### Inferences
- Matuschak's ladder maps almost one-to-one onto the report's layers:
  - Daily working log = `sessions/`
  - writing inbox = rewire inbox
  - precise declarative notes = atomic claims
  - "notes abstracting over many other notes" / outline notes = compiled `concept`/hub pages
  - "Log" notes about a project = a project hub's changelog section

  This is primary support for the report's two-layer "hub-and-claim" design and its inbox → promote path.
- "Inboxes only work if you trust how they're drained" plus "archive or delete after a few looks" gives a concrete rule for the report's G1 gate: an item that is not promoted after N reviews gets discarded.
- The report says "judul sebagai API ala Matuschak" ("title as API, Matuschak-style") for claim titles. That is confirmed, but the claim-phrased title is technically the "complete phrases" note. Core-term (noun) titles stay legitimate for `concept` pages.

### Gaps
- None material. The pages are undated.

---

## 5. LYT / ACE / MOCs (Nick Milo)

### Takeaway
Primary www.linkingyourthinking.com pages confirm the following:
- ACE = Atlas (Knowledge) / Calendar (Time) / Efforts (Action).
- Efforts sit in intensity folders ("On" → "Simmering") with a rank.
- "Folders work for receipts and tax forms" but "not well for ideas".
- MOC phases are "gather, develop, and navigate".
- Make a MOC when you feel the "squeeze".

The **current Ideaverse Pro uses three intensities**, not four ("By 3 Intensities"). The page does not name them. The four-intensity post (On/Ongoing/Simmering/Sleeping) sits on the blocked blog subdomain. "Dots"/"Sources" as Atlas subfolders: **NOT FOUND**. The homepage only has the marketing phrase "system of small dots". Older LYT Kit used an "ACCESS" folder set.

### Cited Findings
- **ACE headspaces.** CONFIRMED in substance: "Can you define in which headspace you currently spend the most energy: Knowledge, Time, Action?" — [Ideaverse Lesson 2: The headspace of ACE](https://www.linkingyourthinking.com/ideaverse-for-obsidian/ideaverse-lesson-2-the-headspace-of-ace) (acc. 2026-10-28). Testimonial on the product page: "Atlas for Knowledge, Calendar for Time and Efforts for Action". — [Ideaverse Pro](https://www.linkingyourthinking.com/ideaverse-pro) (acc. 2026-10-28)
  - The guiding questions ("where would you like to go?" etc.): NOT FOUND (blog subdomain blocked).
- **Atlas and Calendar.** CONFIRMED: "The Atlas is your central hub for knowledge. It's where topics, concepts, and questions go—organized in a way that mirrors how you think, not how folders demand." Calendar: "brings temporal context into your thinking system". — [Ideaverse Pro](https://www.linkingyourthinking.com/ideaverse-pro) (acc. 2026-10-28)
- **Effort intensities: count.** CORRECTED / conflict resolved in favour of three for the current product.
  > "Organize Your Projects By 3 Intensities... Efforts gives you a flexible way to budget your bandwidth, and see what matters most across areas, projects, and written works... Use the three intensities to sort your various efforts based on energy."

  — [Ideaverse Pro](https://www.linkingyourthinking.com/ideaverse-pro) (acc. 2026-10-28)
  - The page does not name the three.
  - The 2023 Ideaverse lesson confirms "On" and "Simmering" as folders, plus a rank: "Drag and drop an effort from the "On" folder to the "Simmering" folder... Change their rank." — [Ideaverse Lesson 4: How to Manage Your Efforts](https://www.linkingyourthinking.com/ideaverse-for-obsidian/ideaverse-lesson-4-how-to-manage-your-efforts) (acc. 2026-10-28)
  - "Ongoing" and "Sleeping": **NOT FOUND** on accessible primary pages (the "four intensities" post is on the blocked blog.linkingyourthinking.com).
- **Efforts vs projects.** CONFIRMED.
  > "Efforts which I believe can be a lot more liberating than projects... If you add the tag #effort to your effort note, it'll automatically be added to the data view in the "Efforts" note." / "Go to the "Efforts" note (listed as number 08 in the "Home" note)" / "It doesn't need a completion date right now"

  — [LYT Kit Lesson 6](https://www.linkingyourthinking.com/lyt-kit-lessons/lesson-6) (acc. 2026-10-28)
- **Folders vs links.** CONFIRMED: "All of us grew up putting files into folders. That works well for receipts and tax forms. It does not work well for ideas." — [Milo, A powerful system for your thoughts (Ch. 4)](https://www.linkingyourthinking.com/ideaverse/a-powerful-system-for-your-thoughts) (acc. 2026-10-28)
- **When to make a MOC.** NEW:
  > "Then create a note and connect it to other notes... Do this 100 times. When you feel the "squeeze", gather notes into a Map of Content. Do this 10 times... Create a home note"

  — [same page](https://www.linkingyourthinking.com/ideaverse/a-powerful-system-for-your-thoughts) (acc. 2026-10-28)
- **MOC phases.** CONFIRMED: "the 3 phases of MOCs work: gather, develop, and navigate." — [Ideaverse Lesson 6](https://www.linkingyourthinking.com/ideaverse-for-obsidian/ideaverse-lesson-6-how-to-do-note-making-and-map-making) (acc. 2026-10-28). "MOCs are built to evolve over time". "Continue exploring the MOCs in the Atlas folder" (top-level MOCs live in Atlas): CONFIRMED. — [LYT Kit Lesson 3](https://www.linkingyourthinking.com/lyt-kit-lessons/lesson-3) (acc. 2026-10-28)
- **MOC as "non-exclusive folder".** NOT FOUND. The definition page is on the blocked notes.linkingyourthinking.com.
- **One home per note, found by links.** NEW (older ACCESS framework):
  > "Sometimes a note could potentially be placed in two different locations - try to trust your intuition, pick one folder, and know that it can always be found later using links and MOCs - that's the beauty of this hybrid approach."

  — [LYT Kit Lesson 5: Why use ACCESS folders](https://www.linkingyourthinking.com/lyt-kit-lessons/lesson-5) (acc. 2026-10-28)
- **Atlas subfolders "Dots"/"Maps"/"Sources".** NOT FOUND. Homepage only: "Learn the "system of small dots"". — [linkingyourthinking.com](https://www.linkingyourthinking.com/) (acc. 2026-10-28)
  - "Sources" is plausibly an ACCESS-era folder (ACCESS is named in LYT Kit Lesson 5, but the folder list is not shown in page text).

### Inferences
- The report's `stage: on|ongoing|simmering|sleeping|done` borrows from a four-intensity scheme that is not verifiable on accessible primary pages. The current product uses three, unnamed on the page.
  - This does not break the recommendation, since rewire's vocabulary is its own design.
  - But the report should stop presenting the five values as "ACE intensities". Better: "inspired by LYT Efforts (On/Simmering confirmed; LYT now uses three intensities)". Alternatively, collapse to `on | simmering | sleeping` plus `done`, and use `kind: area` (not a stage value) to express "ongoing".
- Milo's "100 notes → squeeze → MOC" heuristic is a primary-sourced trigger for creating a `concept`/MOC page. It supports creating concept pages on demand (a cluster threshold) rather than upfront.
- LYT's "pick one folder, find by links and MOCs" is the same "canonical home + links" rule the report adopts (one file in `nodes/<type>/`, membership through links and hubs).

### Gaps
- blog.linkingyourthinking.com ("The four intensities of efforts", "ACE folder framework", "The 3 phases of MOCs") and notes.linkingyourthinking.com ("MOCs (defn)") must be allowed in the network settings to verify the names and count of intensities, the guiding questions and the "non-exclusive folder" wording.

---

## 6. Johnny.Decimal

### Takeaway
The limits and the "abandoned the constraints" quote are confirmed verbatim. Corrections:
- Multi-project notation is now **SYS.AC.ID** with identifiers `A00 … Z99`. The "PRO.AC.ID limited to 99" claim is outdated.
- Overflow beyond 100 IDs uses "Extend the end" (`11.63+ Wendy`), not "append an identifier" loosely.
- Folders *inside an ID* are allowed if patterned. The rule is against ad-hoc subfolders, not against all depth.

New (2026-09-07): a standard-zero slot **`.05 AI`**, "A place for your AI agent", plus "Your JDex is an amazing communication tool: both you and your agent can read and write anything in it."

### Cited Findings
- **Hard limits and three levels.** CONFIRMED.
  > "A design principle of Johnny.Decimal is that there are no more than 10 categories in each of 10 areas. And each category has no more than 100 IDs. This enforces a shallow structure: areas contain categories contain IDs. That's it: three levels."

  — [JD docs, Expand an area: Overview](https://johnnydecimal.com/documentation/expand-an-area-overview) (acc. 2026-10-28)
- **"Expanding abandons constraints".** CONFIRMED verbatim.
  > "One of your standard areas is being expanded. It can grow to an essentially infinite size: we have abandoned the constraints of Johnny.Decimal. So _be careful_. Remember the chaos of the past. Don't go back there."

  Also: "The items in this area might not have numerical IDs. They might not have IDs at all." — [Expand an area: Overview](https://johnnydecimal.com/documentation/expand-an-area-overview) (acc. 2026-10-28)
- **Three expansion options.** CORRECTED / clarified.
  - "Multiple systems", "Expand an area" (for "more than 10 of something, or... more depth than the system allows"), and "Extend the end" (for "an ID that needs to be repeated, or you require more than 100 IDs in a category", e.g. "`11.63+ Wendy`").
  - Source: [System expansion: Introduction](https://johnnydecimal.com/documentation/system-expansion-introduction) (acc. 2026-10-28)
- **Multi-project / multi-system notation.** CORRECTED.
  > "Use three characters of the form `[A-Z][0-9][0-9]`, i.e. `A00 … Z99`. We refer to this as `SYS.AC.ID`."

  That gives 2,600 identifiers, not 99. The site also has a blog post titled "Project is now system" (0028; not fetched). — [Multiple systems: Overview](https://johnnydecimal.com/documentation/multiple-systems-overview); [AC.ID notation](https://johnnydecimal.com/documentation/acid-notation) (acc. 2026-10-28)
  - The report's "K2: lemah (PRO.AC.ID dibatasi 99)" ("K2: weak, PRO.AC.ID capped at 99") should read "SYS.AC.ID".
  - JD multi-system support is meant for *separate systems* (home/work), not for many small projects.
- **Areas and categories.** CONFIRMED.
  > "Areas are the filing cabinets... If they're too narrow, you'll use them up. Leave room for expansion." / "Categories... are the most important Johnny.Decimal concept... A category is where you work" / "Prefer fewer, broader categories"

  — [Areas and categories](https://johnnydecimal.com/documentation/areas-and-categories) (acc. 2026-10-28)
  - "Areas and categories hold no content; only IDs store things": NOT FOUND verbatim on this page. It is implied by the structure.
- **Subfolders inside IDs.** CORRECTED.
  - The notes said "Official JD guidance says never create folders below categories". In fact folders below IDs are allowed if patterned: "The secret to keeping them neat is well-defined subfolders... you should never have: Ad-hoc, Randomly named, Un-numbered subfolders in an ID."
  - Dates are recommended as `yyyy-mm-dd`.
  - Source: [Subfolder patterns](https://johnnydecimal.com/documentation/subfolder-patterns) (acc. 2026-10-28)
- **JDex.** CONFIRMED and strengthened: "A Johnny.Decimal index (or JDex) is the master record of every ID in a system... I consider this more important than your filesystem... The index _is_ your Johnny.Decimal system." — [The JDex](https://johnnydecimal.com/documentation/the-jdex) (acc. 2026-10-28)
- **Standard zeros.** The notes listed this as unverified. Now CONFIRMED, with new 2026 content.
  - Category-level IDs: `.00 JDex`, `.01 Inbox`, `.02 Task & project management`, `.03 Templates`, `.04 Links`, `.05 AI`, `.06–.08` reserved, `.09 Archive`. Area-level and system-level equivalents also exist.
  - Inbox: "You _must_ tidy this up regularly. Don't let anything sit here".
  - AI slot:
    > "`.05 AI` — Added: 2026-09-07. A place for your AI agent. Agents work best when you write things down. Your JDex is an amazing communication tool: both you and your agent can read and write anything in it. Best memory ever. Sometimes it's useful to have a _dedicated_ space that the agent controls. This is a place that it can store persistent information for recall across sessions."
  - Archive: "`.09 Archive` — A place to store something that you do not intend to organise, but do not want to throw away... If there's stuff in here that _should_ be organised, move it to `.01`."
  - Source: [The standard zeros](https://johnnydecimal.com/documentation/the-standard-zeros) (acc. 2026-10-28)

### Inferences
- The report's "not adopted except the 'shallow' principle" verdict stands. The `.05 AI` slot and "JDex as shared human/agent memory" are an independent 2026 primary-source endorsement of a single index that both human and agent read and write, analogous to rewire's index/snapshot. They are also an endorsement of keeping a dedicated agent-controlled area separate from human-curated material.
- JD's `.01 Inbox` → `.09 Archive` → "move back to `.01` if it should be organised" is a minimal lifecycle that mirrors the report's G1/G3 gates without file moves for active items.

### Gaps
- When-not-to-use-JD guidance and the "Project is now system" blog post were not fetched.

---

## 7. Diátaxis (Daniele Procida)

### Takeaway
All claims are confirmed, including the compass's two questions. One correction: the lineage is "Divio between 2014-2021", not "2017". A "2020" creation date was NOT FOUND.

### Cited Findings
- **Four kinds.** CONFIRMED: "there are fundamentally four identifiable kinds of documentation, that respond to four different needs. The four kinds are: tutorials, how-to guides, reference and explanation. Each has a different purpose, and needs to be written in a different way." — [Diátaxis: Start here](https://diataxis.fr/start-here/) (acc. 2026-10-28)
- **Compass.** The notes listed this as a gap. Now CONFIRMED.
  > "To use the compass, just two questions need to be asked: action or cognition? acquisition or application?"

  - The table: "informs action / acquisition of skill → a tutorial; informs action / application of skill → a how-to guide; informs cognition / application of skill → reference; informs cognition / acquisition of skill → explanation".
  - Terms: "action: practical steps, doing; cognition: theoretical or propositional knowledge, thinking; acquisition: study; application: work".
  - The compass can be applied "at the level of sentences and words, or from a wider perspective, considering an entire document."
  - Source: [The compass](https://diataxis.fr/compass/) (acc. 2026-10-28)
- **Mixing types is the main failure.** CONFIRMED.
  > "Crossing or blurring the boundaries described in the map is at the heart of a vast number of problems in documentation." / "When these distinctions are allowed to blur, the different kinds of documentation bleed into each other... In the worst case there is a complete or partial collapse of tutorials and how-to guides into each other"

  — [Start here](https://diataxis.fr/start-here/); [The map](https://diataxis.fr/map/) (acc. 2026-10-28)
- **Origin.** CORRECTED.
  > "Diátaxis is the work of Daniele Procida." / "an earlier presentation of some of these ideas, that I created while working at Divio between 2014-2021... The original context for the Diátaxis approach was limited to software product documentation. In 2021 I was awarded a Fellowship of the Software Sustainability Institute"

  The "2020" date (pasqualepillitteri.it) is NOT FOUND on the primary site. — [Colophon](https://diataxis.fr/colophon/) (acc. 2026-10-28)
- **Applies to documentation itself.** CONFIRMED: "The compass can be applied equally to user situations that need documentation, or to documentation itself that perhaps needs to be moved or improved." — [The compass](https://diataxis.fr/compass/) (acc. 2026-10-28)

### Inferences
- The report's `form: how-to | reference | explanation` on concept pages is consistent with the primary vocabulary. The compass gives a deterministic two-question classifier that an LLM consolidator can apply per page.
- Procida says the original context "was limited to software product documentation". Applying it to personal-knowledge pages is an extension. It is reasonable, but it is not Procida's claim.

### Gaps
- None material.

---

## 8. ADR (Michael Nygard, "Documenting Architecture Decisions", 2011-11-15)

### Takeaway
The original post is now read in full. Context, Decision, Status and Consequences are confirmed verbatim. The notes listed several items as gaps; all are now confirmed:
- one or two pages
- "conversation with a future developer"
- monotonic numbering, never reused
- supersede rather than delete

Three corrections:
- Nygard's titles are **"short noun phrases"**, not imperative verb phrases (that convention is joelparkerhenderson's).
- His status list has **no "rejected"**.
- His section order is Title, Context, Decision, **Status**, Consequences.

### Cited Findings
All quotes in this section are from [Nygard, Documenting Architecture Decisions (2011-11-15)](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions) (acc. 2026-10-28).

- **Small, modular documents.** CONFIRMED: "Large documents are never kept up to date. Small, modular documents have at least a chance at being updated."
- **Scope.** CONFIRMED: "Each record describes a set of forces and a single decision in response to those forces"; "One ADR describes one significant decision for a specific project."
- **Numbering and supersede.** CONFIRMED (the notes listed this as a gap):
  > "ADRs will be numbered sequentially and monotonically. Numbers will not be reused. If a decision is reversed, we will keep the old one around, but mark it as superseded. (It's still relevant to know that it was the decision, but is no longer the decision.)"
- **Title.** CORRECTED: "These documents have names that are short noun phrases. For example, "ADR 1: Deployment on Ruby on Rails 3.0.10" or "ADR 9: LDAP for Multitenant Integration"". The notes' "present-tense imperative verb phrase" is the joelparkerhenderson guide's convention, not Nygard's.
- **Context.** CONFIRMED: "This section describes the forces at play, including technological, political, social, and project local. These forces are probably in tension, and should be called out as such. The language in this section is value-neutral. It is simply describing facts."
- **Decision.** CONFIRMED: "It is stated in full sentences, with active voice. "We will …""
- **Status.** CORRECTED: the original has no "rejected":
  > "A decision may be "proposed" if the project stakeholders haven't agreed with it yet, or "accepted" once it is agreed. If a later ADR changes or reverses a decision, it may be marked as "deprecated" or "superseded" with a reference to its replacement."
- **Consequences.** CONFIRMED: "This section describes the resulting context, after applying the decision. All consequences should be listed here, not just the "positive" ones."
- **Length and style.** CONFIRMED as Nygard's own (the notes had this as a gap):
  > "The whole document should be one or two pages long. We will write each ADR as if it is a conversation with a future developer. This requires good writing style, with full sentences organized into paragraphs. Bullets are acceptable only for visual style, not as an excuse for writing sentence fragments."
- **Chaining.** NEW: "The consequences of one ADR are very likely to become the context for subsequent ADRs."
- **Location.** NEW: "We will keep ADRs in the project repository under doc/arch/adr-NNN.md"

### Inferences
- The report's "ADR ringkas ≤600 karakter" ("compact ADR, ≤600 characters") is a deliberate compression *below* Nygard's own guidance ("one or two pages", "full sentences organized into paragraphs", value-neutral context listing forces in tension). The report's Phase 2(b) proposal of ~1,200 characters for `decision` moves toward Nygard. Even that is well under one page.
  - Say explicitly that ≤600 characters is "Nygard-shaped" (Context/Decision/Consequences), not Nygard-sized.
  - Alternatively, store the full ADR as a compiled page linked from the claim.
- Nygard's per-project scope ("for a specific project") and supersede-not-delete match rewire's existing `supersedes`/`valid_to` mechanism. CONFIRMED fit.

### Gaps
- None for the original post.

---

## 9. Primary-source facts that bear on the report's recommendation

### Takeaway
No primary source contradicts the core recommendation: keep `nodes/` flat, put lifecycle in frontmatter, compile hub and concept pages over atomic claims, and add explicit promotion gates. Several primary sources strengthen it. One requires reframing: Forte's 2026 pro-folder argument. One requires a wording fix: ACE intensities, where three are current and four are unverified.

### Cited Findings
- **For "hub-and-claim inside `nodes/`"** (each source is cited with a verbatim quote in its section above):
  - Matuschak's ladder runs from log → inbox → precise claims → "notes abstracting over many other notes" / outline notes ([Taxonomy of note types](https://notes.andymatuschak.org/Taxonomy_of_note_types)).
  - zettelkasten.de structure notes are "a Zettel about other Zettels" ([Introduction](https://zettelkasten.de/introduction/)).
  - Forte says over-compressed notes "lose all meaning" ([Progressive Summarization](https://fortelabs.com/blog/progressive-summarization-a-practical-technique-for-designing-discoverable-notes/)).
  - Forte's per-project "pre-assembled bundles of context" for AI ([Why PARA Is the Key to the AI Era](https://fortelabs.com/blog/why-para-is-the-key-to-the-ai-era/)) is satisfied by a compiled hub rather than a folder.
- **Against physical folder moves:**
  - Sascha Fast: PARA's "restlessness... Nothing has its fixed place" ([zettelkasten.de](https://zettelkasten.de/posts/building-a-second-brain-and-zettelkasten/)).
  - Matuschak: "further sorting requires undoing existing structure" ([associative ontologies](https://notes.andymatuschak.org/Prefer_associative_ontologies_to_hierarchical_taxonomies)).
  - zettelkasten.de: titles-as-IDs are discouraged because renames break links ([Introduction](https://zettelkasten.de/introduction/)). This matches rewire's `-<id8>` suffix.
- **For physical folders:** Forte (2026), "point it at a Project folder" ([Why PARA Is the Key to the AI Era](https://fortelabs.com/blog/why-para-is-the-key-to-the-ai-era/)). He also says AIs "don't really need a highly precise, detailed hierarchy" and that "You don't need a folder hierarchy 7 layers deep" ([Finding Alpha](https://fortelabs.com/blog/finding-alpha-whats-worth-consuming-in-the-age-of-ai/)).
- **For lifecycle as `kind`/`stage`:**
  - Forte's Project vs Area definitions (outcome + deadline vs standard + continuous) give a primary test for `kind: project|area` ([Project People vs. Area People](https://fortelabs.com/blog/project-people-vs-area-people-are-you-running-a-sprint-or-a-marathon/)).
  - Forte puts "on hold" projects into Archives ([PARA](https://fortelabs.com/blog/para/)), so a "simmering" state goes beyond PARA.
  - LYT confirms "On" → "Simmering" plus rank ([Ideaverse Lesson 4](https://www.linkingyourthinking.com/ideaverse-for-obsidian/ideaverse-lesson-4-how-to-manage-your-efforts)), and now "3 Intensities" ([Ideaverse Pro](https://www.linkingyourthinking.com/ideaverse-pro)).
  - Forte, Archives "facilitate that forgetting without actually deleting anything" ([Why PARA Is the Key to the AI Era](https://fortelabs.com/blog/why-para-is-the-key-to-the-ai-era/)).
- **For promotion gates:**
  - Matuschak: an inbox needs "a practice which reliably drains it"; after "a few looks... archive or delete it" ([writing inbox](https://notes.andymatuschak.org/A_writing_inbox_for_transient_and_incomplete_notes)).
  - Forte's filing cascade: project → area → resource → archive or don't save ([basboverview](https://fortelabs.com/blog/basboverview/)).
  - Milo: build a MOC when you feel the "squeeze" after ~100 linked notes ([LYT Ch. 4](https://www.linkingyourthinking.com/ideaverse/a-powerful-system-for-your-thoughts)).
  - zettelkasten.de: atomicity is reached "in your Zettelkasten, not outside of it" ([principle vs implementation](https://zettelkasten.de/posts/principle-of-atomicity-difference-between-principle-and-implementation/)).
  - Friction Fallacy: review cost per note must not rise with size ([Friction Fallacy](https://zettelkasten.de/posts/the-friction-fallacy/)).
- **For agent-controlled space plus a shared index:** JD `.05 AI` (added 2026-09-07): "a _dedicated_ space that the agent controls... persistent information for recall across sessions" and "both you and your agent can read and write anything in it" ([standard zeros](https://johnnydecimal.com/documentation/the-standard-zeros)).

### Inferences
Suggested edits to the report (for the report writer; not applied here):
1. **Section on PARA (line 46).** Replace "modular context bundles for AI" (not on the cited page) with Forte's verbatim "pre-assembled bundles of context". Add one sentence: the live `project` hub is rewire's equivalent of Forte's "point it at a Project folder" bundle. This answers Forte's strongest pro-folder argument instead of leaving it unaddressed.
2. **Line 141 (`stage` values).** Present `on/ongoing/simmering/sleeping/done` as rewire's own vocabulary "inspired by LYT Efforts". Primary pages confirm only On and Simmering, and current Ideaverse Pro uses "3 Intensities". Optionally drop `ongoing` as a stage, since `kind: area` already encodes "continuous over time" per Forte's definition.
3. **Johnny.Decimal row (line 41).** Change "PRO.AC.ID dibatasi 99" ("PRO.AC.ID capped at 99") to "SYS.AC.ID (A00–Z99), meant for separate systems".
4. **ADR row (line 42) and line 143.** Note that Nygard's own guidance is "one or two pages... full sentences organized into paragraphs" with "short noun phrase" titles. ≤600 characters keeps the shape, not the size. This strengthens the case for Phase 2(b).
5. **Progressive Summarization.** Forte's L4 is for "a small number of notes that are the most insightful". Apply the first-sentence-summary rule to hubs and concept pages as true L4, and to claims as a one-line gist.
6. **Method-caveat paragraph (line 67).** It can now say that PARA, Progressive Summarization, Matuschak, Johnny.Decimal, Diátaxis and Nygard were verified on primary pages on 2026-10-28. Only the LYT blog/notes subdomains remain unverified.

### Gaps
- LYT intensity names and the four-vs-three history need blog.linkingyourthinking.com allowed.
- The Forte book text (BASB 2022, The PARA Method 2023) and the Ahrens book were not checked. Some secondary-sourced claims (divergence/convergence mapping to CODE steps, Archipelago of Ideas, the Hot/Cold variant, the fleeting/literature/permanent wording) may originate there.
