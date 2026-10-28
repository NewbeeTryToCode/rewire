# Knowledge-organization methods beyond PARA/CODE for a markdown + git "brain" (human + LLM readers)

> **How this was researched (October 28, 2026):** The session's network proxy blocked direct fetches of every primary site: zettelkasten.de, notes.andymatuschak.org, johnnydecimal.com, cognitect.com, diataxis.fr and linkingyourthinking.com all returned 403 or failed DNS. Two kinds of material were reachable:
> - **Downloaded and read in full** from GitHub raw: the Nygard ADR template, the joelparkerhenderson ADR guide, the MADR template and the adr-tools README.
> - **Everything else** came from web-search result extracts that point at the primary pages. The URLs below are the pages those extracts came from. Exact wording should be re-checked on the live page before anyone quotes it.
>
> Judging criteria used throughout:
> - **C1:** a single note is self-contained and information-rich.
> - **C2:** the method supports many concurrent projects.
> - **C3:** there is a clear path from raw capture to project-specific to shared/general knowledge.

## 1. Zettelkasten (Luhmann; Ahrens, *How to Take Smart Notes*)

### Takeaway
Zettelkasten gives the best-defined pipeline from capture to durable knowledge (fleeting → literature → permanent), built from one-idea notes that are linked and indexed by structure notes. Its weaknesses are notes too thin to read on their own, the up-front work of deciding what counts as "atomic", and maintenance friction. It says nothing about managing projects.

### Cited Findings
- Ahrens' book dates from 2017 ("How to Take Smart Notes (Ahrens, 2017)") — [LessWrong](https://www.lesswrong.com/posts/T382CLwAjsy3fmecf/how-to-take-smart-notes-ahrens-2017)
- **Three note types:**
  - *Fleeting notes* are quick, disposable reminders, thrown away once they have been turned into permanent notes.
  - *Literature notes* are short, selective, written in your own words, and always carry the source reference.
  - *Permanent notes* hold "one idea and one idea only" per card. They are written while reviewing the fleeting and literature notes.
  - Sources: [Ness Labs](https://nesslabs.com/?p=9347); [Rufus Pollock excerpts](https://rufuspollock.com/post/ahrens-how-to-take-smart-notes); [Ernest Chiang (2025)](https://www.ernestchiang.com/en/posts/2025/sonke-ahrens-how-to-take-smart-notes/)
- **Two boxes.** Luhmann kept a bibliographic box (references plus brief content notes) and a main box (his own ideas). A new permanent note is filed behind one or more related notes, and that placement is itself a link. On paper a note can sit in only one place; software can file it behind several at once. — [Rufus Pollock excerpts](https://rufuspollock.com/post/ahrens-how-to-take-smart-notes); [Ernest Chiang](https://www.ernestchiang.com/en/posts/2025/sonke-ahrens-how-to-take-smart-notes/)
- **IDs (Folgezettel).**
  - Luhmann's IDs work as coordinates in a tree that can branch at any node. Examples: 1/1a branches off 1/1; 2.1/1 and 2/2 are unrelated siblings.
  - He also used a comma notation (1/1,1 … 1/1,4) that worked more like footnotes.
  - Practitioners still argue about how far the scheme scales for branching. One forum claim is that each note can carry at most two branches; Bob Doto's variant handles branching better but long sequences worse.
  - Sources: [Zettelkasten Forum](https://forum.zettelkasten.de/discussion/comment/13601); [Zettelkasten Forum](https://forum.zettelkasten.de/discussion/comment/24772); [Zettelkasten Forum: numbering](https://forum.zettelkasten.de/discussion/comment/1738)
- **Structure notes.** These are auxiliary notes that list note IDs in order to define a sequence of thought. Practitioners use them as hubs instead of deep ID chains. — [Zettelkasten Forum](https://forum.zettelkasten.de/discussion/comment/13299)
- **Atomicity critiques (from zettelkasten.de itself).**
  - Requiring notes to be atomic *on entry* creates anxiety about getting it right immediately.
  - It also pushes the hard thinking (finding the atomic unit) outside the system, even though "this is a significant proportion of the whole thinking process."
  - The site separates atomicity as a *principle* from how it is *implemented*.
  - Source: [zettelkasten.de: principle of atomicity](https://zettelkasten.de/posts/principle-of-atomicity-difference-between-principle-and-implementation/)
- **Friction Fallacy.** Any system whose marginal cost per note rises as it grows will eventually become unusable. The essay tests this with a "one million bad notes" thought experiment. — [zettelkasten.de: The Friction Fallacy](https://zettelkasten.de/posts/the-friction-fallacy/)
- **Time cost.** A practitioner complaint: atomic notes take a lot of time, because each note has to be split and its links worked out. — [Hacker News via hn.svelte.dev](https://hn.svelte.dev/item/32227826)
- **Collector's Fallacy.** This is the trap of feeling productive just because you collected material. It comes up repeatedly in Zettelkasten discussion. — [Zettelkasten Forum](https://forum.zettelkasten.de/discussion/comment/23736)
- **Origin dispute.** Forum members argue that the rigid literature/permanent split common in online guides is "a byproduct of the internet's interpretation of the translation" of Ahrens, not Luhmann's own practice. — [Zettelkasten Forum](https://forum.zettelkasten.de/discussion/comment/19539)
- **Project notes don't fit the slip-box.** Projects are temporary, while slip-box notes are meant to be permanent. — [Zettelkasten Forum](https://forum.zettelkasten.de/discussion/comment/18272/)

### Inferences
- **C3 (capture → general): strong.** The fleeting → literature → permanent pipeline maps directly onto inbox → processed → shared knowledge. It is the most explicit promotion path of all the methods here.
- **C1 (self-contained note): weak if atomicity is taken literally.** One-idea cards depend on their neighbours for context. An LLM agent retrieving one chunk gets a thin claim without the surrounding argument. The fix is to require each permanent note to restate its context and source, which is closer to Matuschak's "capture the entirety of that thing".
- **C2 (many projects): not addressed.** Zettelkasten organizes ideas, not work, so it needs a separate effort/project layer.
- **IDs in git:** Luhmann-style positional IDs (21/3d7a) encode placement. In a git repo with wikilinks and backlinks, timestamp or slug IDs plus structure notes give the same function with fewer merge conflicts and no need to renumber.

### Gaps
- Luhmann's original register/keyword index (Schlagwortregister) and the number of notes in his boxes could not be verified this session (zettelkasten.de was blocked, and searches returned only forum posts). The figure of roughly 90,000 cards is background knowledge, not verified here.
- The "21/3d7a" style ID example could not be matched to a fetched source.

## 2. Evergreen notes (Andy Matuschak)

### Takeaway
Evergreen notes keep Zettelkasten's one-concept focus but add "capture the entirety of that thing", claim-style titles that act like APIs, and organization by concept rather than by source or project. Of all the methods here, this best balances atomicity against self-containment.

### Cited Findings
- **Atomic, but complete.** "It's best to create notes which are only about one thing—but which, as much as possible, capture the entirety of that thing." Matuschak compares this to separation of concerns in software. — [Evergreen notes should be atomic](https://notes.andymatuschak.org/Evergreen_notes_should_be_atomic)
- **Fragmentation tradeoff.** On the same page: if notes are too fragmented, you also fragment your link network ("cohesion problem"), and "there's no clear litmus test… just a bunch of tradeoffs". — [Evergreen notes should be atomic](https://notes.andymatuschak.org/zNUaiGAXp21eorsER1Jm9yU)
- **Concept-oriented.** Notes should be factored by concept rather than by author, book, event or *project*. — [Evergreen notes should be concept-oriented](https://notes.andymatuschak.org/Evergreen_notes_should_be_concept-oriented)
- **Densely linked.** Links should be added deliberately, not generated automatically; adding many of them forces you to think about how ideas relate. — [Evergreen notes should be densely linked](https://notes.andymatuschak.org/z2HUE4ABbQjUNjrNemvkTCsLa1LPDRuwh1tXC)
- **Titles as claims.** Titles are complete propositions, for example "Evergreen notes should be densely linked". Good titles act like APIs: other notes can cite a title without restating the content, and notes behave like reusable, loosely coupled modules. — [ithelp.ithome.com.tw](https://ithelp.ithome.com.tw/articles/10273883); [rashitamemo (scrapbox)](https://scrapbox.io/rashitamemo/Evergreen%20notes%E3%81%AE%E3%82%B3%E3%83%B3%E3%82%BB%E3%83%97%E3%83%88); [sspai](https://sspai.com/post/69048)
- **Date.** The canonical "Evergreen notes" note is catalogued as Matuschak 2020. — [stafforini.com](https://stafforini.com/works/matuschak-2020-evergreen-notes/)
- **Scope ambiguity.** Practitioners ask what counts as "one thing": "the 'one thing' is going on 3 years now. Wouldn't want to put that in one note!" — [Obsidian forum](https://forum.obsidian.md/t/evergreen-notes-and-learning-scientific-fields/3394)

### Inferences
- **C1: strongest of all the methods.** "One thing, but all of it", plus a claim title, gives notes that an LLM can retrieve and use alone: the title states the conclusion and the body carries the full argument.
- **"Atomic" vs "self-contained" is less a contradiction than a scoping rule.** Narrow the scope, then be complete within it.
- **C2: deliberately excludes projects.** "Not by project" matches the "projects reference shared knowledge, don't own it" principle (see section 7).
- **C3:** Matuschak's broader corpus implies a "write notes for yourself, then evolve them" flow, but that flow was not captured from primary pages this session.
- **Agent-friendly convention:** use claim titles as filenames or H1s. They double as an index line in a MOC, and agents can match them semantically.

### Gaps
- The exact text of "Evergreen note titles are like APIs", "Prefer associative ontologies to hierarchical taxonomies" and Matuschak's notes on writing for yourself could not be fetched (domain blocked). The claims about them come from secondary sources.

## 3. LYT / ACE (Nick Milo: Atlas, Calendar, Efforts; Maps of Content)

### Takeaway
ACE uses three top-level folders split by "headspace":
- **Atlas:** timeless knowledge and maps.
- **Calendar:** time-based notes such as dailies and logs.
- **Efforts:** work, sorted by *intensity* (On / Ongoing / Simmering / Sleeping) rather than PARA's binary of active project vs archive.

MOCs act as non-exclusive hub "folders" made of links. Of all the methods here, this layout comes closest to the needs of a multi-project brain.

### Cited Findings
- **The three folders.** ACE = Atlas, Calendar, Efforts. Atlas holds timeless information, Calendar holds time-related material (links to daily notes, etc.), and Efforts holds work done on a topic. — [dsebastien notes: ACE Framework](https://notes.dsebastien.net/30+Areas/32+Literature+notes/32.02+Content/ACE+Framework)
- **Guiding questions.** Milo frames ACE as headspaces, not just folders. Atlas asks "where would you like to go?", Calendar asks "what's on your mind?", Efforts asks "what can you work on?". He advises using the pre-provided ACE layout before customizing it. — [LYT blog: ACE folder framework](https://blog.linkingyourthinking.com/notes/ace-folder-framework)
- **Atlas holds maps.** Atlas is described as a "map of maps": map notes, MOCs and index-style notes that help you find your own knowledge. — [vocus.cc](https://vocus.cc/article/64fcdc42fd89780001b4a9d6)
- **Four effort intensities.**
  - *On:* the most active efforts.
  - *Ongoing:* broader, continuing efforts.
  - *Simmering:* efforts kept in the back of the mind.
  - *Sleeping:* everything else, including finished, dormant and cold-storage efforts.
  - When there's too much going on, move an effort from On to Simmering. Intensities can map onto four folders.
  - Sources: [LYT blog: The four intensities of efforts](https://blog.linkingyourthinking.com/notes/the-four-intensities-of-efforts); [Ideaverse Lesson 4](https://www.linkingyourthinking.com/ideaverse-for-obsidian/ideaverse-lesson-4-how-to-manage-your-efforts)
  - The *Ideaverse Pro 2.0* course listing mentions three intensities, so sources conflict on the count. — [brainlearns.org](https://brainlearns.org/courses/nick-milo-ideaverse-pro-2-0/)
- **Efforts as an alternative to projects.** The LYT Kit presents Efforts as a replacement for "projects". Tagging a note `#effort` adds it automatically to a Dataview list in the "Efforts" note, which sits at position 08 in the Home note. — [LYT Kit lesson 6](https://www.linkingyourthinking.com/lyt-kit-lessons/lesson-6)
- **Ranked effort table (secondary source).** A Bases table ranks active, simmering and sleeping projects by a numeric rank, and an effort changes state by moving its file. — [Taskade: History of Obsidian (2026)](https://www.taskade.com/blog/obsidian-history)
- **MOC definition and placement.** A MOC is a "non-exclusive folder with a completely customized organizational structure". Top-level MOCs live in Atlas. Milo describes phases of MOC development. — [notes.linkingyourthinking.com: MOCs (defn)](https://notes.linkingyourthinking.com/Cards/MOCs+(defn)); [The 3 phases of MOCs](https://blog.linkingyourthinking.com/notes/the-3-phases-of-mocs); [LYT Kit lesson 3](https://linkingyourthinking.com/lyt-kit-lessons/lesson-3)
- **Home note.** A Home note is the entry point that helps the system scale. — [LYT blog: ACE folder framework](https://blog.linkingyourthinking.com/notes/ace-folder-framework)

### Inferences
- **C2: strong.** Intensity levels let many efforts exist at once without pretending they are all "active". Simmering is a legitimate parked state, which PARA lacks (PARA only has project or archive). For an agent, an `intensity:` frontmatter field is cheaper than moving folders and is git-diff friendly.
- **Efforts vs PARA projects.** PARA projects need a goal and a deadline. Efforts are looser and can be ongoing (close to PARA "Areas"), so one Efforts tree replaces both Projects and Areas.
- **C3: mid.** Calendar (daily notes) works as the capture/inbox layer. Knowledge is promoted to Atlas as "Dots" (idea notes) linked from MOCs. The promotion step is less formal than in Zettelkasten.
- **MOCs as the agent's entry point.** A MOC is a curated list of links with one-line glosses, which makes it a natural routing file for an LLM (read the MOC, then open the target notes). It is also the bridge between an Effort and the Atlas notes it uses.

### Gaps
- The exact Atlas subfolders (Dots / Maps / Sources) and current Ideaverse folder names could not be confirmed. Searches found no primary source for "Dots" or "Sources", and the material may sit behind the paid Ideaverse or course. They are background knowledge only.
- The date of "The four intensities of efforts" post is not known. Ideaverse for Obsidian launched around August/September 2023, per [Nick Milo on X](https://x.com/NickMilo/status/1696784687069372459); the exact date is not verified.

## 4. Johnny.Decimal

### Takeaway
Johnny.Decimal is a strict, shallow, numeric address system: 10 areas × 10 categories × up to 100 IDs, written as AC.ID, for example 11.01. It is excellent for stable, finite domains such as admin, finance and reference. It is a poor fit as the main structure of a growing, link-heavy, multi-project knowledge brain. Its PRO.AC.ID multi-project extension is advised only for large projects.

### Cited Findings
- **Hard limits.** At most 10 areas (00-09, 10-19, …), at most 10 categories per area, and at most 100 IDs per category. An ID starts with its area's number (83.66 lives in area 80-89). Areas and categories hold no content; only IDs store things. — [johnnydecimal.com: areas & categories](https://johnnydecimal.com/concepts/areas-categories/); [documentation: areas and categories](https://johnnydecimal.com/documentation/areas-and-categories); [documentation: the numbers](https://johnnydecimal.com/documentation/the-numbers.md)
- **Overflow handling.** For more than 100 IDs in a category, or for an ID that "repeats", append an identifier to the end. The "Expand an area" pattern lets an area grow to "essentially infinite size", which the docs themselves describe as having "abandoned the constraints of Johnny.Decimal". — [documentation: expand an area overview](https://johnnydecimal.com/documentation/expand-an-area-overview.md)
- **The JDex.** The index is the master record of a system's IDs. Keeping one is recommended, especially when the system spans several places. — [johnnydecimal.com](https://www.johnnydecimal.com); [dsebastien (2022-04-29)](https://dsebastien.net/2022-04-29-johnny-decimal/)
- **Multi-project use (community advice).**
  - "Exhaust AC.ID first."
  - PRO.AC.ID makes sense only when each project is large enough to need its own AC.ID tree; otherwise it over-complicates the system.
  - With 10-15+ projects a year, PRO gives only 99 project slots. Users have proposed YYYY.ID instead.
  - Others want a lighter scheme for "many small projects".
  - Sources: [JD forum: alternate project naming](https://forum.johnnydecimal.com/t/alternate-project-naming-idea/542); [JD forum: YYYY.ID for projects](https://forum.johnnydecimal.com/t/yyyy-id-for-projects/1915); [JD forum: simpler version for many small projects](https://forum.johnnydecimal.com/t/trying-a-simpler-version-for-many-small-projects/785)
- **PARA hybrids.** Community members combine JD with PARA. — [JD forum: J.D + PARA](https://forum.johnnydecimal.com/t/curating-project-management-to-your-needs-and-my-j-d-para-system/612)

### Inferences
- **Fits:** stable reference and admin domains, short spoken or typed addresses ("see 14.03"), and keeping folders shallow (two levels max), which suits both humans and agents.
- **Fails for a growing brain:**
  - The 10×10 ceiling forces renumbering or "expansion" once the taxonomy changes.
  - A note can have only one hierarchical home, which conflicts with Matuschak's associative, concept-oriented linking.
  - Numbers carry no meaning for an LLM unless the JDex is loaded. Semantic slugs plus MOCs are self-describing.
- **C1:** neutral (it doesn't govern note content). **C2:** weak to mid (PRO.AC.ID caps and adds complexity). **C3:** absent (no capture or processing flow).
- **Best role:** optionally number a few top-level folders (for example `00-inbox`, `10-efforts`, `20-atlas`, `90-archive`) for stable sort order, without adopting AC.ID for individual notes.

### Gaps
- The official johnnydecimal.com page on when *not* to use J.D, and any official "PRO" documentation, could not be fetched. Multi-project guidance above is forum opinion.
- The current (2026) state of JD documentation and its "standard zeros" conventions (for example x0 categories for system or meta notes) is not verified.

## 5. ADR (Nygard) and Diátaxis as note templates

### Takeaway
- **ADR** gives decision notes a proven, tiny, append-only format: Title / Status / Context / Decision / Consequences, numbered and superseded rather than edited.
- **Diátaxis** gives knowledge notes a type system. There are four types: tutorial, how-to, reference and explanation. Don't mix them in one note.

Together they make strong per-note templates for C1.

### Cited Findings
- **Nygard template sections (read in full).**
  - *Title.*
  - *Status:* proposed, accepted, rejected, deprecated, superseded, etc.
  - *Context:* "What is the issue that we're seeing that is motivating this decision or change?"
  - *Decision:* "What is the change that we're proposing and/or doing?"
  - *Consequences:* "What becomes easier or more difficult to do because of this change?"
  - Source: [joelparkerhenderson ADR repo: Nygard template](https://github.com/joelparkerhenderson/architecture-decision-record/tree/main/locales/en-001/templates/decision-record-template-by-michael-nygard)
- **Origin.** Nygard proposed the format in "Documenting Architecture Decisions", dated 2011-11-15 and originally on thinkrelevance.com, now on cognitect.com. — [arc42 FAQ C-9-3](https://faq.arc42.org/questions/C-9-3/)
- **What the sections mean (secondary sources).**
  - *Context* describes the forces at play (technological, political, social, project-local), which "are probably in tension, and should be called out as such".
  - *Consequences* describes the resulting context after the decision, with "all consequences… not just the 'positive' ones".
  - *Status* becomes "deprecated" or "superseded", with a reference to the replacement.
  - Sources: [arc42 docs tip 9-5](https://docs.arc42.org/tips/9-5/); [GOV.UK ADR 0001](https://docs.publishing.service.gov.uk/repos/content-publisher/adr/0001-record-architecture-decisions.html)
- **Length.** Fowler: ADRs should be short, "just a couple of pages". The "one or two pages" wording could not be confirmed as Nygard's own. — [martinfowler.com: ADR](https://www.martinfowler.com/bliki/ArchitectureDecisionRecord.html)
- **Supersede mechanics (read in full).**
  - `adr new -s 9 <title>` creates a new numbered ADR flagged as superseding ADR 9 and changes ADR 9's status to "superseded by…".
  - `adr init doc/architecture/decisions` sets up the folder, and ADR 0001 records the decision to use ADRs.
  - Source: [npryce/adr-tools README](https://github.com/npryce/adr-tools)
- **Good-ADR guidance (read in full).**
  - One decision per ADR.
  - Timestamp each item.
  - "Immutable: Don't alter existing information… amend… or supersede". The same guide admits "in practice, mutability has worked better for our teams" and recommends dated inline updates.
  - Filenames use a present-tense imperative verb phrase in lowercase with dashes, for example `choose-database.md`.
  - Consequences should list follow-on ADRs and plan an after-action review about a month later.
  - The repo now lists AI-agent tooling, such as an "append-only decision log for AI coding agents" (kgai) and "ADR enforcement for AI coding agents" (Mneme).
  - Source: [joelparkerhenderson/architecture-decision-record README](https://github.com/joelparkerhenderson/architecture-decision-record)
- **MADR template (read in full).**
  - YAML frontmatter: `status` (proposed | rejected | accepted | deprecated | superseded by ADR-0123), `date`, `decision-makers`, `consulted`, `informed`.
  - Sections: Context and Problem Statement, Decision Drivers, Considered Options, Decision Outcome ("Chosen option… because…"), Consequences (Good/Bad because…), Confirmation, Pros and Cons of the Options, More Information.
  - Source: [adr/madr template](https://github.com/adr/madr/blob/main/template/adr-template.md)
- **Diátaxis origin.** Daniele Procida created Diátaxis (a secondary source dates it to 2020). He is a Django core developer and Canonical's Director of Engineering. — [pasqualepillitteri.it](https://pasqualepillitteri.it/news/5527/diataxis-framework-documentazione-ai); [Canonical blog](https://canonical.com/blog/diataxis-a-new-foundation-for-canonical-documentation)
- **Diátaxis's four types and two axes.**
  - Each type answers a different user need.
  - The axes are action ↔ cognition (doing vs knowing) and acquisition ↔ application (study vs work).
  - Tutorial = action + acquisition; how-to = action + application; reference = cognition + application; explanation = cognition + acquisition.
  - The "compass" is the tool for deciding which type a piece of text should be. Mixing types is the main failure, for example a tutorial overloaded with explanation.
  - Sources: [Diátaxis: Start here](https://www.diataxis.fr/start-here/); [diataxis.fr PDF](https://diataxis.fr/_/downloads/en/latest/pdf/)
- **Diátaxis for AI agents.** Diátaxis is now packaged as agent skills (for example Oxide's Claude plugin) for writing documentation with LLMs. — [tessl.io registry: oxidecomputer diataxis](https://tessl.io/registry/skills/github/oxidecomputer/claude-plugins/diataxis)

### Inferences
- **ADR for decision notes in a personal brain.**
  - Use the Nygard four sections plus MADR frontmatter (`status`, `date`, `supersedes`/`superseded-by`) and number the files per effort or globally.
  - "Supersede, don't edit" fits git history well, and it lets an agent see which decision is current by following `superseded-by` links.
  - It is also high-density for C1: Context + Decision + Consequences makes a decision understandable without the rest of the project.
- **Diátaxis for knowledge notes.** Give each knowledge note a `type:` drawn from {how-to, reference, explanation} (tutorials are rare in a personal brain). Mixing types is the main source of bloated, unreadable notes, so "one note = one Diátaxis type" pairs naturally with "one note = one concept".
- **C3:** ADR naturally sits at the project-specific level. A decision's *generalizable lesson* can be promoted to an Atlas or explanation note that links back to the ADR as evidence.

### Gaps
- The exact text of Nygard's original post (length guidance, "conversation with a future developer", sequential numbering, never reusing numbers) could not be fetched. cognitect.com was blocked.
- diataxis.fr pages could not be fetched, so the compass's exact questions are not quoted. Diátaxis's lineage from Procida's 2017 "What nobody tells you about documentation" (Divio) is background knowledge, unverified here.

## 6. GTD inbox and review discipline → a review queue for AI-proposed memories

### Takeaway
GTD has five steps: capture, clarify, organize, reflect, engage. The weekly review ("get clear, get current, get creative") is its critical success factor. That maps directly onto a queue where AI-proposed memories land in an inbox and are regularly clarified, filed, promoted or deleted by a human.

### Cited Findings
- **The five steps.** Capture everything that has your attention into a trusted system, then clarify (what is it, and what should be done with it), organize, reflect and engage. — [Wikipedia: Getting Things Done](https://en.wikipedia.org/wiki/Getting_Things_Done); [tiny.write.as: GTD notes ch. 2](https://tiny.write.as/noisydeadlines/gtd-notes-chapter-02-five-steps); [Paperlike](https://paperlike.com/de/blogs/paperlikers-insights/getting-things-done)
- **Clarify empties the inboxes.** Allen's methodology guide describes clarifying as the process for emptying your inboxes. The weekly review is meant to "get clear, get current, and get creative". — [Allen's GTD methodology guide (Scribd copy)](https://www.scribd.com/document/516946948/tuxdoc-com-gtdmethodologyguidesa4pdf)
- **The weekly review.** It is the critical success factor: go through every list, clear finished items, and make sure each active project has a next action. Most reviews only reach the lower "horizons of focus". — [Fabric: Getting Things Done](https://developers.fabric.so/learn/getting-things-done)
- **Dates and naming.** GTD dates from Allen's 2001 book, revised in 2015. Some summaries call step 4 "Review" instead of "Reflect". — [Fabric](https://developers.fabric.so/learn/getting-things-done); [Thomas Frank](https://thomasjfrank.com/productivity/the-5-minute-guide-to-gtd-getting-things-done/)
- **A PKM review cadence in practice.** One practitioner runs a daily 10-minute filing pass, a weekly review that turns fleeting notes into permanent ones and checks project flow, and a monthly scan for notes with no backlinks. — [Obsidian forum: PARA + Zettelkasten](https://forum.obsidian.md/t/how-can-para-and-zettelkasten-workflow-live-together/3570)

### Inferences
- **Mapping to a review queue of AI-proposed memories.**
  - *Capture:* the agent writes a proposed memory into `inbox/` with frontmatter (source session, proposed target, confidence).
  - *Clarify:* a human or agent asks whether this is actionable (→ effort task), a decision (→ ADR), knowledge (→ Atlas or evergreen note), or noise (→ delete).
  - *Organize:* move or merge it, then link it from the relevant MOC.
  - *Reflect:* the weekly review empties the inbox, checks each "On" effort for a next action and stale "Simmering" items, and lists orphans.
  - *Engage:* agents read only processed notes by default.
- **Inbox-zero discipline** is what stops the Collector's Fallacy (section 1) in an AI-fed system, where capture costs nothing and volume can explode. The Friction Fallacy argument implies that review cost per item must stay flat.

### Gaps
- No primary gettingthingsdone.com page was fetched (search only). The official weekly-review checklist items are not quoted verbatim.

## 7. Hybrid recommendations from practitioners

### Takeaway
The community consensus is "PARA (or ACE Efforts) for action, Zettelkasten/evergreen + MOCs for ideas". Projects *link to* concept notes rather than owning them. Before archiving a project, harvest whatever is reusable into the shared knowledge layer.

### Cited Findings
- **Division of labour.** "Zettelkasten is for ideas. PARA is for action." Project folders hold the plan and open questions, each linking to concept notes. "The project uses the concept without owning it." Before archiving a finished project, check whether its notes contain anything worth keeping as a resource or permanent note. — [glukhov.org: PARA method for engineers](https://www.glukhov.org/de/knowledge-management/methods/para-method-for-engineers/)
- **Compositional vs atomic.** "Projects and areas draw from the slip box, but they are compositional rather than atomic." — [Obsidian forum: How can PARA and Zettelkasten live together?](https://forum.obsidian.md/t/how-can-para-and-zettelkasten-workflow-live-together/3570/11)
- **Link integrity.** Running two systems works as long as links update when notes move between folders, and archived notes stay easy to reach. — [Obsidian forum thread (p.2)](https://forum.obsidian.md/t/how-can-para-and-zettelkasten-workflow-live-together/3570/40?page=2)
- **Library vs workshop.** One framing: the second brain is the library or archive, and the Zettelkasten is the studio or workshop. — [Zettelkasten Forum: BASB and Zettelkasten](https://forum.zettelkasten.de/discussion/comment/19539)
- **Applying PARA inside a Zettelkasten** is a recurring pattern. — [Obsidian forum: Applying PARA in Zettelkasten](https://forum.obsidian.md/t/share-on-applying-para-in-zettelkasten-system/98203)

### Inferences
- **Synthesized layout for a markdown/git brain read by humans and agents** (an inference, not any single source's prescription):
  - `inbox/`: GTD capture, including AI-proposed memories. It has a review queue and must be emptied weekly.
  - `calendar/` or `log/`: ACE Calendar, holding dated session and daily logs (raw, append-only).
  - `efforts/<effort>/`: ACE Efforts with frontmatter `intensity: on|ongoing|simmering|sleeping`. Each effort holds a README/MOC (goal, status, next action, links), project-local notes, and `decisions/NNNN-verb-phrase.md` ADRs.
  - `atlas/`: evergreen or permanent notes with claim titles and a Diátaxis `type`, plus `atlas/maps/` MOCs as hubs, plus `atlas/sources/` literature notes.
  - Optional Johnny.Decimal-style numeric prefixes on the top-level folders only.
- **Promotion path (C3).** inbox → (clarify) → effort-local note or ADR → (when it is reused by a second effort, or during the weekly review) → atlas evergreen note. The effort note is then replaced by a link: "projects reference shared knowledge, don't own it".
- **Note density (C1).** Combine Matuschak's "one thing, but the entirety of it" with required context fields (source, date, scope, status) and a claim title. This counters the "atomic notes are too thin" failure for both humans and RAG/agent retrieval.
- **Concurrency (C2).** Intensity levels plus a per-effort MOC scale to many concurrent efforts. Shared knowledge stays deduplicated in Atlas.
- **Main failure modes to guard against:**
  - Moving files breaks links. Prefer stable IDs or slugs, and use a status field instead of folder moves.
  - Unreviewed AI capture (Collector's Fallacy).
  - Over-taxonomy (Johnny.Decimal ceilings).
  - Mixed-type notes (the Diátaxis anti-pattern).

### Gaps
- No practitioner source was found that specifically designs a PKM for *LLM agents as co-readers* combining these methods. The agent-oriented recommendations above are inference.
- Tiago Forte's own statements on combining PARA with Zettelkasten were not retrieved (out of scope here; covered by the PARA/CODE researcher).
