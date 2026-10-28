# PARA and CODE (Tiago Forte / Building a Second Brain) as a basis for an LLM-read/written markdown + git second brain

Research date: 2026-10-28. **Method caveat:** this session's network blocked direct page fetches of fortelabs.com, every.to, hackmd.io, forum.zettelkasten.de, stefanimhoff.de and web.archive.org, and GitHub API access to third-party repos. All findings below come from web-search result extracts (the search tool reads the pages and quotes them). Primary Forte pages are cited where the search surfaced them, but I could not read them end-to-end. Exact wording of Forte's definitions should be checked against the book *Building a Second Brain* (Atria, June 2022) and *The PARA Method* (Atria, 2023) before anyone quotes them as authoritative. To fetch the primary sources directly, the session's network settings must allow fortelabs.com, every.to and forum.zettelkasten.de.

---

## PARA: definitions, actionability, movement between categories, concurrency, nesting, "organize by where it will be used"

### Takeaway
PARA sorts information into four top-level buckets, plus an Inbox outside the acronym, by **how actionable it is right now, not by topic**:
- **Projects:** a goal plus a deadline.
- **Areas:** an ongoing standard to maintain.
- **Resources:** topics of interest.
- **Archives:** anything inactive from the other three.

Items move between buckets as their actionability changes, typically Resource → Project → Archive. Practitioners recommend a shallow hierarchy (1–2 levels under each top folder) and links or tags for cross-cutting relations.

### Cited Findings
- **Forte's own primary article** is now titled "The PARA Method: The Simple System for Organizing Your Digital Life in Seconds" (fortelabs.com/blog/para). It was surfaced but could not be fetched. — [Forte Labs](https://fortelabs.com/blog/para/)
- **Classic definitions,** repeated verbatim across many secondary sources:
  - **Project:** "a series of tasks linked to a goal, with a deadline".
  - **Area of responsibility:** "a sphere of activity with a standard to be maintained over time".

  — [RescueTime Blog](https://blog.rescuetime.com/the-p-a-r-a-system-and-the-importance-of-folders/); [byarbrough/obsidian-para](https://github.com/byarbrough/obsidian-para); [TIL MagmaLabs](https://til.magmalabs.io/posts/b1df4b12f4-para-method)
- **Resource:** "a topic or theme of ongoing interest". **Archives:** "completed projects, closed Areas, outdated Resources": everything no longer active that shouldn't be deleted. — [web-highlights summary](https://web-highlights.com/blog/para-method-the-secret-to-digital-organization-i-wish-i-knew-sooner/); [Every/Forte Labs (republished PARA article)](https://every.to/forte-labs/the-para-method-a-universal-system-540221)
- **Projects vs. Areas:** "Projects have a finish line; Areas do not". Confusing the two is "the most common PARA implementation error". — [Alfred blog](https://get-alfred.ai/blog/para-method)
- **Inbox:** PARA is really "a system of five folders – Projects, Areas, Resources, Archives, and an Inbox (which is not represented in the PARA acronym)". — [Thomas Frank, PARA book notes](https://thomasjfrank.com/productivity/books/the-para-method-by-tiago-forte-summary-and-book-notes/); [Taskade](https://www.taskade.com/blog/the-para-method)
- **Sorting rule:** organize by actionability ("how active is this right now?"), not by topic. A note can move from Resources to a Project when it becomes useful, then to Archives when the work ends. — [Atlas Workspace](https://www.atlasworkspace.ai/blog/para-method-explained); [Storyflow](https://storyflow.so/blog/what-is-the-para-method-complete-guide)
- **Category ambiguity:** "Learn Spanish" could be a multi-year Area or a bounded Project with a fluency target, and PARA "does not resolve this for you". — [search extract of practitioner guides, e.g. A Pragmatic Mind](https://www.apragmaticmind.com/blog/para-method)
- **Teams:** Forte's "PARA for Teams" says archiving is the simplest layer: whenever something stops being relevant, it goes into the Archive folder in Drive or gets archived in Asana. — [Forte Labs, PARA for Teams](https://fortelabs.com/blog/para-for-teams/)
- **Hot or Cold:** Forte (Second Brain Office Hours, and an X post from March 2025) has promoted a simplified two-bucket variant. "Hot" holds active projects you're working on now. "Cold" holds everything else: reference, templates, dormant projects and archives. The PARA book reportedly mentions that some people just keep "Hot" and "Cold" folders. — [Forte on X](https://x.com/fortelabs/status/1896923704036041037); [Medium summary](https://medium.com/never-stop-writing/the-only-two-folders-you-need-to-stay-organised-8e9144fda92d); [Sterling Long, 8 learnings from The PARA Method](https://medium.com/@sterlinglong/8-beautifully-distilled-learnings-from-tiago-fortes-the-para-method-6d2ac635dfd2)
- **Depth:** one guide reports PARA "works best with 1-2 levels of depth" and recommends tags or links over nested folders for related items. Common setups are 4 top-level folders, each with one sub-folder per project, area or resource. — [smartremotegigs PARA SOP (2026)](https://smartremotegigs.com/para-method-complete-guide/); [Kyle Nitchen](https://kylenitchen.substack.com/p/the-para-method)
- **Flexibility:** Forte's book says you needn't be "super-rigid" about PARA. — [search extract referencing the Profile Books preview of The PARA Method](https://profilebooks.com/wp-content/uploads/wpallimport/files/PDFs/9781800819542_preview.pdf)

### Inferences
- **Folder layout this implies:**
  - `Projects/<project-slug>/`
  - `Areas/<area-slug>/`
  - `Resources/<topic-slug>/`
  - `Archive/<original-bucket>/<slug>/`
  - plus `Inbox/`

  Only one level of subfolder per bucket. Further structure goes in frontmatter or links.
- **Lifecycle:** the *Project* is the unit of lifecycle. When the project ends, its whole folder moves to Archive. Notes are not re-sorted one by one. This makes archiving cheap: one `git mv`.
- **"Where will I use this?":** the "organize by where it'll be used" heuristic is widely attributed to Forte: file a note under the *most actionable* container where it would help, so Project before Area before Resource. In the search extracts I could only confirm the actionability ordering, not the exact phrasing (see Gaps).
- **Concurrent projects:** PARA handles many concurrent projects simply as many sibling folders under `Projects/`. There is no limit and no prioritization mechanism inside PARA itself; prioritization is left to the task manager and the weekly review.

### Gaps
- I couldn't read Forte's exact current wording of the four definitions or his explicit nesting/depth advice. The "1–2 levels" figure is from a third-party SOP, not Forte.
- I found no Forte-sourced figure for a typical number of concurrent active projects. Forte has been quoted elsewhere with figures like "10–15 projects", but I could not verify it in this session.
- The exact quote "organize by where you will use it, not where you found it / not by topic" was not confirmed in a primary source here.

---

## CODE (Capture, Organize, Distill, Express): Progressive Summarization, Intermediate Packets, Archipelago of Ideas, Divergence/Convergence

### Takeaway
CODE is the workflow and PARA is the "Organize" step.
- **Capture** what resonates.
- **Organize** for actionability (PARA).
- **Distill** using Progressive Summarization: successive layers of bolding and highlighting, then an executive summary, done opportunistically when a note is touched again.
- **Express** by assembling reusable Intermediate Packets.

Capture + Organize are divergence; Distill + Express are convergence.

### Cited Findings
- **Progressive Summarization (2017, Forte)** distills a note's essence in several "layers". The layers, as summarized from Forte's original article *Progressive Summarization: A Practical Technique for Designing Discoverable Notes* (Forte, 2017):
  - **Layer 1:** captured excerpts.
  - **Layer 2:** **bolded** passages.
  - **Layer 3:** highlighted passages: only the most interesting or surprising of the bolded ones.
  - **Layer 4:** an executive summary written at the top of the note "in my own words". Not always needed.
  - **Layer 5:** a remix into something new. Used for "a tiny minority of sources".

  — [Readwise copy of Forte 2017](https://garden.synesthesia.co.uk/references/readwise/articles/progressive-summarization-a-practical-technique-for-designing-discoverable-notes-forte-2017/); [HackMD copy](https://hackmd.io/@_6snqgesSRWI1ygKSv1TtA/prog-sum-doc); [Josh Knox summary (2023)](https://iamjoshknox.com/2023/08/25/what-tiago-forte-taught-me-about-summarization-a-summary/); [Sloww deep summary](https://www.sloww.co/building-a-second-brain-book/)
- **Timing:** the bolding pass happens "at a later time, when I'm already reviewing this note anyway". Distillation is incremental and just-in-time, not done at capture. — [Readwise copy of Forte 2017](https://garden.synesthesia.co.uk/references/readwise/articles/progressive-summarization-a-practical-technique-for-designing-discoverable-notes-forte-2017/); [Taskade](https://www.taskade.com/blog/the-para-method)
- **Variant warning:** a 2026 "three-pass" variant with a 10–20% bolding target is a reinterpretation, not Forte's original. — [smartremotegigs CODE guide (2026)](https://smartremotegigs.com/tiago-forte-code-method/)
- **Intermediate Packets:** small, reusable units of work-in-process. Examples are distilled notes, outtakes, work in progress, or final deliverables from older projects. They are reusable in future projects. — [Sloww](https://www.sloww.co/building-a-second-brain-book/); [Eric Sandroni summary](https://ericsandroni.com/building-a-second-brain-summary/)
- **Divergence/Convergence:** "the first two steps of CODE, Capture and Organize, make up divergence and the final two steps, Distill and Express, are about convergence". Convergence is where you "eliminate options, make trade-offs, and decide what is truly essential". — [Walter Teng BASB notes](https://walterteng.com/building-a-second-brain); [Sloww](https://www.sloww.co/building-a-second-brain-book/)
- **Archipelago of Ideas:** start a project from a set of pre-gathered "islands" of ideas, the IPs and notes, rather than a blank page. Forte credits the idea to Steven Johnson. — [David Tran](https://www.davidtran.me/archipelago-of-ideas/); [Sloww](https://www.sloww.co/building-a-second-brain-book/)
- **Express survives the AI shift:** a 2026 commentary argues CODE survives because it describes a knowledge workflow, not a tool. — [ctaio.dev](https://ctaio.dev/en/second-brain/second-brain-method-ai/)

### Inferences
- **What Distill means for a single note:** structure it so a reader, human or LLM, gets the gist at the top. This is a Layer-4-style summary or TL;DR block. Key claims are marked inline; the source material stays underneath for drill-down.
- **Markdown mapping:**
  - `**bold**` = L2
  - `==highlight==` (Obsidian syntax) = L3
  - a `## Summary` or frontmatter `summary:` = L4
  - links to derivative outputs = L5
- **LLM-agent caveat:** an LLM reading a note pays for every token. A dense top-of-note summary (L4) plus a stable slug/title is what makes "quickly readable" true for agents. L2/L3 emphasis matters less to a model than an explicit summary line, but it can steer which spans get re-summarized.
- **"Discoverability over understanding":** Forte's framing of Progressive Summarization, as recalled from the 2017 article title, is "designing discoverable notes". This argues for summaries optimized for *later retrieval by a future reader*, which fits LLM retrieval well.
- **Intermediate Packets in a git vault:** they map to small, independently linkable files. For example, one decision, one snippet, one checklist per file. This is in tension with PARA's folder-per-project unit (see Criticisms).

### Gaps
- I could not read the full 2017 article, so I have no confirmed quotes for its guidance on how much to bold or highlight (e.g. percentages) or for "opportunistic compression". The "10–20%" figure is from a non-Forte 2026 source.
- No primary-source quotes from the BASB book itself; all CODE details here are from book summaries.

---

## Where the inbox fits; weekly and monthly review

### Takeaway
The Inbox is the fifth, un-acronymed folder. Everything lands there first and is filed into PARA during the **Weekly Review**: clear email, review calendar, file notes, choose tasks. The **Monthly Review** is a system check for archiving finished projects and realigning Areas. PARA's correctness depends on these human review rituals.

### Cited Findings
- **Five folders:** Projects, Areas, Resources, Archives, and an Inbox not represented in the acronym. — [Thomas Frank](https://thomasjfrank.com/productivity/books/the-para-method-by-tiago-forte-summary-and-book-notes/)
- **Forte's Weekly Review** has 4 steps, "in that order": "1. Clear Email 2. Review Calendar 3. File Notes 4. Choose Tasks". He keeps the checklist on a desktop sticky note. — [Forte on X (Dec 2023)](https://twitter.com/fortelabs/status/1736401134384758879); [Forte on X (Jun 2024)](https://x.com/fortelabs/status/1802295115886584175)
- **Review layers:** "The Weekly Review is an Operating System". The monthly review is a system check and the annual review a rearchitecture. The monthly review "alternates back and forth between top-down and bottom-up methods, seeking to connect my long-term vision with the ground truth". — [Forte, The Design of a Weekly Review (Every)](https://every.to/forte-labs/the-design-of-a-weekly-review-538446)
- **Secondary account:** a 15–30 min weekly review plus a monthly review to archive finished projects. — [Taskade (secondary, verify)](https://www.taskade.com/blog/the-para-method)
- **Just-in-time:** Forte favors systems adopted bit by bit and distillation done when notes are actually used. — [search extract, Taskade / Storyflow](https://storyflow.so/blog/how-to-build-a-second-brain)

### Inferences
- **For an automated system:** "Inbox → file during review" maps naturally onto "session transcripts / raw captures land in `Inbox/` and a consolidation job files them". The weekly-review step "File Notes" is exactly the job an LLM consolidator would take over.
- **Archiving trigger:** the Monthly Review is where Forte archives completed projects. An automated equivalent needs an explicit "project status" check on a schedule.

### Gaps
- I found no Forte guidance on maximum inbox age or on what to do with items that can't be classified.

---

## Known constraints and criticisms

### Takeaway
Credible critics (Sascha Fast of zettelkasten.de, Nick Milo, Andy Matuschak's principles, Obsidian forum users) agree that PARA is a **storage/retrieval scheme for action**, not a knowledge-building system. Its single-folder-per-file model handles notes relevant to many projects badly. Resources tends to become a junk drawer, and archiving hides material from topic-based retrieval. It also depends on manual review discipline.

### Cited Findings
- **Sascha Fast (zettelkasten.de):**
  - BASB/PARA is essentially project management, ordered by action and urgency. The Zettelkasten deals only in ideas and their links.
  - He keeps the two separate.
  - He uses PARA as a place to collect and roughly pre-process material; only processed ideas go into the Zettelkasten.
  - Trying to house Zettelkasten notes in PARA added friction because the category had to be decided upfront.

  — [Zettelkasten.de: Combine Zettelkasten and BASB](https://zettelkasten.de/posts/building-a-second-brain-and-zettelkasten/); [Forum thread 2615 p2](https://forum.zettelkasten.de/discussion/2615/building-a-second-brain-and-the-zettelkasten-method/p2)
- **Zettelkasten forum:** PARA "is not a second brain in the way that the Zettelkasten is or can be. It is a logical form of storage and retrieval". — [Zettelkasten Forum, Processing BASB (thread 2429)](https://forum.zettelkasten.de/discussion/2429/processing-building-a-second-brain-by-tiago-forte)
- **Curtis McHale:** PARA is a productivity system for tasks and their documents, not a knowledge development system. He notes PARA explicitly says not to collect merely "interesting" material. — [Curtis McHale, Sept 2023](https://curtismchale.ca/2023/09/16/member-reads-sept-16-2023)
- **Andy Matuschak:** evergreen notes "should be factored by concept (rather than by author, book, event, project, topic, etc)". If notes are bundled by source or project, you must remember which project covered an idea to find it. This is a direct counter-principle to filing by project. — [Matuschak, Evergreen notes should be concept-oriented](https://notes.andymatuschak.org/Evergreen_notes_should_be_concept-oriented)
- **Nick Milo (LYT):** folders work "for receipts and tax forms" but "do not work well for ideas". LYT uses links and Maps of Content (MOCs). His current Ideaverse still has folders: ACE = Atlas, Calendar, Efforts. MOCs combine tag-like grouping and folder-like proximity "without moving every note into one exclusive folder". — [Ideaverse Lite](https://obsidianmate.com/vaults/ideaverse); [LYT/Ideaverse chapter](https://www.linkingyourthinking.com/ideaverse/a-powerful-system-for-your-thoughts); [yu-wenhao LYT vs PARA (2026)](https://yu-wenhao.com/en/blog/lyt-framework-guide/)
- **Cross-project knowledge and MECE:** PARA doesn't explain how Projects and Resources relate. A resource can serve several Projects or Areas, but a file "cannot reside within two places at once". The structure isn't MECE, which leaves users an "overlap burden". — [Medium @paralloid](https://medium.com/@paralloid/para-method-for-organizing-your-content-26e29435b92d)
- **Linking across folders:** "the major problem with PARA is it doesn't allow you to link notes between folders" (in a folder sense). The proposed fix is MOCs. — [Aidan Helfant, PARA not working? Create MOCs](https://medium.com/@aidan.helfant/para-not-working-create-mocs-in-obsidian-3e16c176bf46)
- **Resources bloat:** Resources is "the easiest bucket to turn into a junk drawer" because "might be interesting someday" has no filter. — [search extract of 2026 PARA guides, e.g. iwoszapar](https://www.iwoszapar.com/p/para-method)
- **Archive graveyard (contested):**
  - Archives should hold completed work "in reasonable shape, not a graveyard of unsorted notes". — [DEV: PARA for Engineers](https://dev.to/rosgluk/para-method-for-engineers-organize-knowledge-by-action-1npg)
  - Counter-view: the archive is searchable, reactivatable, and "a search index you rarely open but are glad exists". — [Vaultorial](https://vaultorial.com/para-method/); [PKM Wiki](https://pkm-wiki.knowii.net/para-method)
  - Another view: once things go into the archive, "the structure disappears". — [Medium @paralloid](https://medium.com/@paralloid/para-method-for-organizing-your-content-26e29435b92d)
- **Archive vs. AI retrieval:** when a project ends and notes move to Archive, an AI tool may struggle to find them; this was one author's reason for choosing LYT. — [yu-wenhao (2026)](https://yu-wenhao.com/en/blog/lyt-framework-guide/)
- **Action vs. connection:** PARA optimizes for action, Zettelkasten for connection. Choose by output: many deliverables favor PARA, connected ideas favor Zettelkasten. — [memx PKM glossary](https://memx.app/glossary/personal-knowledge-management/)
- **Obsidian forum: not mutually exclusive.** Calling the two mutually exclusive is "a kind of fallacy". The useful part of PARA is the project-oriented "PA" layer. — [Obsidian Forum: PARA and Zettelkasten](https://forum.obsidian.md/t/how-can-para-and-zettelkasten-workflow-live-together/3570)
- **Folders vs. tags:** "a document can only be in one folder, but it can have multiple tags". MOCs need ongoing upkeep: Milo says a MOC is "fluid", so you keep adding and deleting links. — [Obsidian Forum, folders vs linking vs tags](https://forum.obsidian.md/t/folders-vs-linking-vs-tags-the-definitive-guide-extremely-short-read-this/78468); [Obsidian Forum, MOC vs tags](https://forum.obsidian.md/t/whats-the-advantage-of-using-moc-vs-just-tagging/34721)

### Inferences
- **Key design constraint:** PARA has **one location per file**. Any note relevant to several projects needs one of three fixes:
  - a canonical home, usually Resources or an Area, with links from the projects;
  - duplication, which drifts out of sync;
  - transclusion.

  Practitioners overwhelmingly choose "canonical home + links/MOC".
- **Archiving costs findability:** archiving a project folder buries its reusable insights. A system should extract reusable knowledge (Intermediate Packets, decisions, lessons) into Resources/Areas *before* or *at* archive time. Otherwise, archive search must stay first-class.
- **Topic retrieval** is PARA's weakest axis. It must come from another mechanism: tags, frontmatter `topics:`, MOCs/index files, full-text or semantic search.

### Gaps
- I found no direct, dated PARA critique by Nick Milo himself. His position is inferred from his anti-folder stance and LYT/ACE design.
- I found no Sönke Ahrens statement specifically on PARA. Zettelkasten-side critiques here come from Sascha Fast and forum users.
- I could not read Reddit r/PKMS or r/ObsidianMD threads directly (network restriction).

---

## Adaptations: Obsidian templates, PARA + Zettelkasten, PARA + Johnny.Decimal, PARA with AI agents / LLM-readable vaults, PARA for code

### Takeaway
The dominant 2025–2026 pattern for agent-readable vaults has these parts:
- numbered PARA top folders (`00_Inbox`, `01_Projects`…`04_Archive`, or 10/20/30/40);
- a root `CLAUDE.md`, sometimes with per-project ones, describing conventions;
- an agent that files Inbox items and reads project context on demand.

Forte himself (2026) now teaches PARA as "modular context bundles" for AI. Hybrids put Zettelkasten or evergreen notes inside Resources, and use Johnny.Decimal numbering under PARA's top level.

### Cited Findings
- **Forte's AI course:** "The AI Second Brain" is a 3-week live course. It was announced March 13, 2026; cohort 1 ran April 15–May 1, 2026, and cohort 2 ran Sep 17–Oct 8, 2026. Deliverables include a Master Prompt, "a PARA structure optimized as modular context bundles for AI", a capture system, an "AI Board of Advisors" and a 90-day roadmap. — [Forte Labs help center](https://help.fortelabs.com/hc/en-us/articles/44591754695821-What-is-the-AI-Second-Brain); [Forte Labs, Introducing the AI Second Brain](https://fortelabs.com/blog/introducing-the-ai-second-brain/); [Forte on X, cohort 2](https://x.com/fortelabs/status/2097672194159513926)
- **Forte's rationale:** the bottleneck is now "collecting and curating the right context for models with limited effective context windows". The course progresses from a minimum viable prompt, to smaller context bundles, to longer-term memory and agents. — [brainfeed.ai summary of Forte walkthrough](https://brainfeed.ai/pages/the-ai-second-brain-complete-system-walkthrough-live-with-tiago-forte--TAow.html)
- **Forte's Master Prompt experiment (2026, on X):** he pointed Claude Code at his entire PARA folder structure and asked it to write a "Master Prompt". The result condensed hundreds of folders into one markdown document with ~10 sections: identity, business, frameworks, values, strengths, life context, work habits, favorite problems, aspirations, communication preferences. It also pulled in his Obsidian vault, which sits under Resources. — [Forte on X](https://x.com/fortelabs/status/2022679961845313968)
- **Claudesidian (heyitsnoah):** a Claude Code + Obsidian starter kit that uses `00_Inbox` (temporary capture), `01_Projects` (active, time-bound), `02_Areas`, `03_Resources` and `04_Archive`. A setup wizard generates a personalized `CLAUDE.md`. — [GitHub heyitsnoah/claudesidian](https://github.com/heyitsnoah/claudesidian); [sourcepulse listing](https://www.sourcepulse.org/projects/16162920)
- **Other numberings and `CLAUDE.md` placement:** `00_Inbox, 10_Projects, 20_Areas, 30_Resources, 40_Archives`, with 99 last. Claude reads `CLAUDE.md` hierarchically: root always, then any project one when working in that folder. Another layout uses `00-Meta/` as an operating layer holding `CLAUDE.md`. — [codemeetai substack](https://codemeetai.substack.com/p/how-to-build-a-second-brain-with); [Janette Roush, Personal OS for Claude](https://janetteroush.com/personal-os); [DEV: Claude Code folder structure](https://dev.to/lamingsrb/the-claude-code-folder-structure-that-survives-real-clients-5497)
- **jamesmcroft/obsidian-ai-second-brain:** a starter template combining Obsidian, CODE/PARA and AI skills, including templates, queries and AI instructions. — [GitHub](https://github.com/jamesmcroft/obsidian-ai-second-brain)
- **Stefan Imhoff (2026):** after subscribing to Claude Code in January 2026, he combined Zettelkasten and PARA in one vault ("two different concepts in one vault"). — [Stefan Imhoff, Agentic note taking](https://www.stefanimhoff.de/agentic-note-taking-obsidian-claude-code/)
- **Write-permission guardrail:** one setup separates human-written raw notes, which the agent reads but doesn't edit, from folders the agent writes to. — [search extract of Claude Code + Obsidian guides, e.g. MindStudio](https://www.mindstudio.ai/blog/build-ai-second-brain-claude-code-obsidian)
- **PARA as agent memory (OpenClaw / Clawd Tools):** PARA gives an agent "structured memory that compounds daily". They add Inbox and Daily Notes, and argue markdown files beat vector DBs for the small set of files an agent needs per session. — [Clawd Tools blog](https://clawdtools.ai/blog/para-method-for-ai/)
- **Graham Mann:** keeps agent memory as a `life/` folder in PARA structure, adapted from Nat Eliason. The rule: anything the agent should remember goes into a file it reads after restarts. — [Graham Mann](https://grahammann.net/blog/memory-and-task-systems-giving-your-ai-agent-a-brain)
- **Claw Mart:** PARA "doesn't work for agents out of the box" because it was designed for humans navigating folders. It needs semantic retrieval and **automated lifecycle management**. — [Claw Mart](https://www.shopclawmart.com/blog/para-method-openclaw-memory)
- **Semantic search:** with semantic search, PARA shifts "from a strict filing requirement to a helpful organizing default". — [ctaio.dev](https://ctaio.dev/en/second-brain/second-brain-method-ai/)
- **PARA + Zettelkasten:** Sascha Fast keeps PARA for collection and pre-processing and a separate Zettelkasten for processed ideas. A 2024 blogger reports that adding Zettelkasten to PARA streamlined his setup. There is also an Obsidian community plugin, "PARA ZK". — [zettelkasten.de](https://zettelkasten.de/posts/building-a-second-brain-and-zettelkasten/); [sudoscience (Dec 2024)](https://sudoscience.blog/2024/12/27/combine-your-second-brain-with-zettelkasten/); [Obsidian community: para-zk](https://community.obsidian.md/plugins/para-zk)
- **PARA + Johnny.Decimal:**
  - Typical hybrid: PARA for the top-level split, JD numbers for areas and categories beneath. JD IDs follow the form `AC.ID` (e.g. `10.01`), with a "no more than ten" rule per level and a central index.
  - One template puts Archives in 90–99 and uses year-based project IDs.
  - Official JD guidance says never create folders below categories; some hybrid users nest deeper anyway.

  — [NotePlan help](https://help.noteplan.co/article/155-how-to-organize-your-notes-and-folders-using-johnny-decimal-and-para); [Johnny.Decimal forum: Hybridized JD](https://forum.johnnydecimal.com/t/hybridized-johnny-decimal-system/2742); [johnnydecimal.com](https://johnnydecimal.com); [dsebastien (2022)](https://dsebastien.net/2022-04-29-johnny-decimal/)
- **PARA for engineers and technical knowledge:**
  - Archives should be completed work "in reasonable shape". — [DEV: PARA for Engineers](https://dev.to/rosgluk/para-method-for-engineers-organize-knowledge-by-action-1npg)
  - A January 2026 post applies PARA to technical knowledge, including code repositories and API references as assets. — [dasroot.net (Jan 2026)](https://dasroot.net/posts/2026/01/para-method-technical-knowledge-projects-areas-resources-archives/)

### Inferences
- **Numbering:** numeric prefixes (`00_`…`04_` or `10_`…`40_`) are the de facto convention for stable sort order and for agents to reference folders unambiguously.
- **Instructions:** a root `CLAUDE.md` holds the vault's rules: what goes where, who may write where, note format. It is the standard way practitioners make PARA legible to Claude Code.
- **Forte's own direction:** his 2026 "context bundles" framing suggests each Project or Area folder should be loadable as a self-contained context unit. That argues for a per-folder index or README summary file.
- **"PARA for code repos":** no well-defined practice turned up. In practice, a repo maps to a Project (time-bound) or an Area (a maintained product). Repo-specific memory lives next to the code (e.g. a repo `CLAUDE.md`), while cross-repo knowledge lives in Resources.

### Gaps
- I could not read the Claudesidian README, Stefan Imhoff's post or Clawd Tools posts in full, so details of their agent rules (frontmatter schemas, inbox processing prompts) are unverified.
- I found no rigorous comparison of PARA-structured agent memory against other agent-memory designs.
- No well-documented "PARA for code repos" methodology was found.

---

## Constraints PARA would impose on an automated LLM consolidator (session transcripts → memories)

### Takeaway
PARA's categories depend on **user intent and project state**: Is this time-bound? Is it still active? Where will it be used? Neither is visible in a single transcript. An automated consolidator therefore needs:
- an explicit project registry with status;
- a rule for multi-project knowledge (canonical home + links);
- an inbox / low-confidence fallback;
- scheduled lifecycle checks that replace Forte's weekly and monthly reviews.

### Cited Findings
- **Vendors say PARA needs added machinery for agents:** PARA is human-oriented and needs automated lifecycle management and semantic retrieval to work for agents. — [Claw Mart](https://www.shopclawmart.com/blog/para-method-openclaw-memory)
- **Project vs. Area is the most common error,** and PARA doesn't resolve ambiguous cases like "Learn Spanish". — [Alfred](https://get-alfred.ai/blog/para-method); [A Pragmatic Mind](https://www.apragmaticmind.com/blog/para-method)
- **Archiving can hide notes from AI retrieval.** — [yu-wenhao (2026)](https://yu-wenhao.com/en/blog/lyt-framework-guide/)
- **Forte's filing runs inside a human weekly review** ("File Notes" step), and archiving happens in the monthly review. — [Forte on X](https://twitter.com/fortelabs/status/1736401134384758879); [Forte, Design of a Weekly Review](https://every.to/forte-labs/the-design-of-a-weekly-review-538446)
- **Write guardrail:** practitioners separate human-authored notes (read-only to the agent) from agent-written folders. — [MindStudio guide](https://www.mindstudio.ai/blog/build-ai-second-brain-claude-code-obsidian)
- **Semantic search** makes strict filing less critical. — [ctaio.dev](https://ctaio.dev/en/second-brain/second-brain-method-ai/)

### Inferences
These are design implications, not sourced facts.

1. **Who decides the category:** the LLM can propose, but PARA's Project/Area distinction needs ground truth.
   - **(a)** Keep a machine-readable project registry, e.g. `01_Projects/<slug>/README.md` with frontmatter `status: active|paused|done`, `deadline:`, `outcome:`.
   - **(b)** Map each session to a project by working directory or repo, which is a strong signal in Claude Code. Otherwise use explicit user tags.
   - **(c)** Route low-confidence items to `00_Inbox/` for human or next-pass triage rather than guessing.
2. **Detecting project lifecycle:** "done" is rarely stated in transcripts. Proxies:
   - no sessions or commits touching the project for N weeks;
   - the deadline has passed;
   - an explicit "done/ship" marker.

   A scheduled job should *propose* archiving, not silently move folders. Before archiving, distill reusable lessons and decisions into Resources/Areas (Intermediate Packets) so they stay retrievable.
3. **Cross-project knowledge:**
   - Give each fact or insight exactly one canonical file, usually in Resources or an Area, and link to it from projects. Never duplicate.
   - Record every project a note serves in frontmatter (`related_projects: [...]`).
   - This directly addresses the "one folder per file" weakness.
4. **Moves in git:**
   - Use `git mv` to preserve history.
   - Prefer stable IDs or slugs in links rather than paths, so archive moves don't break references.
   - The consolidator must update or validate links after any move. Obsidian users treat broken-link checking as a known need.
5. **Distill as the consolidation format:** each memory note should carry an L4-style summary at the top, then L2/L3-style key claims, then sources (transcript/session refs). Re-distillation happens opportunistically when the note is retrieved or updated again, matching Forte's just-in-time principle.
6. **Topic retrieval** must be supplied orthogonally: tags or `topics:` frontmatter, index/MOC files per Area or Resource, plus search. PARA alone won't answer "what do I know about X?".
7. **Granularity:** session transcripts naturally produce small, atomic insights that are Zettelkasten- or IP-sized. PARA folders should hold those as many small files plus a per-folder index, not as one growing document per project.

### Gaps
- I found no published system that uses PARA specifically for LLM consolidation of *session transcripts*, or that documents automated project-lifecycle detection with measured accuracy.
- Search extracts mentioned a critic arguing that "a few typed categories (decisions, facts, open loops)" beat "a deep folder tree nobody maintains". I could not pin this to a verifiable source, so it is excluded from Cited Findings.
