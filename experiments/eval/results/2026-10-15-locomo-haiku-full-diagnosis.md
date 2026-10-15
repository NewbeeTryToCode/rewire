# Diagnosis per question: results-locomo-haiku-full.json

| category | n | ok_full | ok_partial | ok_list_part | wrong_answer | wrong_abstain | wrong_retrieval | wrong_ingest | no_evidence |
|---|---|---|---|---|---|---|---|---|---|
| multi-hop | 282 | 120 (43%) | 70 (25%) | 66 (23%) | 9 (3%) | 0 (0%) | 13 (5%) | 4 (1%) | 0 (0%) |
| open-domain | 96 | 38 (40%) | 17 (18%) | 5 (5%) | 9 (9%) | 1 (1%) | 20 (21%) | 2 (2%) | 4 (4%) |
| single-hop | 841 | 743 (88%) | 4 (0%) | 59 (7%) | 32 (4%) | 0 (0%) | 0 (0%) | 3 (0%) | 0 (0%) |
| temporal | 321 | 288 (90%) | 14 (4%) | 0 (0%) | 10 (3%) | 0 (0%) | 4 (1%) | 4 (1%) | 1 (0%) |
| overall | 1540 | 1189 (77%) | 105 (7%) | 130 (8%) | 60 (4%) | 1 (0%) | 37 (2%) | 13 (1%) | 5 (0%) |


  ok_full        right, every evidence turn in the context
  ok_partial     right with some evidence missing (other memories, inference, or a lenient judge)
  ok_list_part   right by the judge, but the answer names only part of the gold list
  wrong_answer   wrong although every evidence turn was in the context (answering, gold or judge)
  wrong_abstain  wrong, the answer says the memories do not tell (and evidence was missing)
  wrong_retrieval wrong, evidence missing although a fact from the turn exists (ranked out)
  wrong_ingest   wrong, evidence missing and no fact was extracted from that turn (only its excerpt could help)
  no_evidence    the dataset lists no evidence turn for the question

## ok_partial (examples)

- conv-26-7 (multi-hop): What is Caroline's relationship status? | gold: Single | answer: Caroline appears to be single. No partner or romantic relationship comes up in any memory, and she is pursuing adoption to build her own family, which points to
- conv-26-15 (multi-hop): What activities does Melanie partake in? | gold: pottery, camping, painting, swimming | answer: Melanie partakes in these activities: - **Painting and art:** She painted a lake sunrise in 2022, and she and her kids paint together, especially nature-inspire
- conv-26-24 (multi-hop): What does Melanie do to destress? | gold: Running, pottery | answer: Melanie destresses mainly by running, which she says she has been doing more of to de-stress and clear her mind (July 2023). She also takes daily "me-time" for 
- conv-26-37 (multi-hop): What did Melanie paint recently? | gold: sunset | answer: Melanie most recently painted a sunset with a pink sky (about a week before 2023-10-13), and also an abstract painting with peaceful blue streaks and vibrant co
- conv-26-39 (multi-hop): In what ways is Caroline participating in the LGBTQ community? | gold: Joining activist group, going to pride parades, participating in an art show, mentoring program | answer: Caroline participates in the LGBTQ community in many ways. She attends support groups, workshops, pride parades, and conferences. She mentors a transgender teen
- conv-26-42 (open-domain): Would Melanie be more interested in going to a national park or a theme park? | gold: National park | answer: A national park. Melanie's family regularly goes camping, hiking in the mountains and forests, and exploring nature, and she has said that nature inspires her p

## ok_list_part (examples)

- conv-26-2 (open-domain): What fields would Caroline be likely to pursue in her educaton? | gold: Psychology, counseling certification | answer: Caroline would most likely pursue counseling and mental health, specifically therapy for transgender and LGBTQ people, helping them with self-acceptance and men
- conv-26-23 (multi-hop): What books has Melanie read? | gold: "Nothing is Impossible", "Charlotte's Web" | answer: Melanie has read *Charlotte's Web* (her favorite childhood book), *Becoming Nicole* by Amy Ellis Nutt (which she was reading in October 2023 after Caroline reco
- conv-26-32 (multi-hop): What LGBTQ+ events has Caroline participated in? | gold: Pride parade, school speech, support group | answer: Caroline has participated in these LGBTQ+ events: - Pride fest with friends (2022) - LGBTQ support group (2023-05-07) - School talk about her transgender journe
- conv-26-34 (multi-hop): What events has Caroline participated in to help children? | gold: Mentoring program, school speech | answer: Caroline has taken part in these events to help children and young people: 1. **School talk (around 2023-06-02):** She spoke to students about her transgender j
- conv-26-38 (multi-hop): What activities has Melanie done with her family? | gold: Pottery, painting, camping, museum, swimming, hiking | answer: Melanie has done these activities with her family:  - **Camping trips:** She went camping in the mountains around June 20, 2023, and at the beach in early July 
- conv-26-56 (multi-hop): What symbols are important to Caroline? | gold: Rainbow flag, transgender symbol | answer: Caroline's most important symbols are:  - **Rainbow flag mural:** Represents the courage and strength of the trans community (2023-08-25). - **Eagle mural:** Sy

## wrong_answer (examples)

- conv-26-85 (single-hop): What are Caroline's plans for the summer? | gold: researching adoption agencies | answer: Caroline's summer 2023 plan was a special trip with Melanie. She suggested either a family outing or a trip just the two of them, to catch up and explore nature
- conv-26-94 (single-hop): What is Melanie's hand-painted bowl a reminder of? | gold: art and self-expression | answer: The hand-painted bowl that serves as a reminder is Caroline's, not Melanie's. Caroline's bowl, painted by a friend for her 18th birthday about ten years ago, re
- conv-30-43 (single-hop): What do the dancers in the photo represent? | gold: They are performing at the festival | answer: The dancers on stage with their arms in the air represent determination and perseverance. Gina shared the photo alongside her "Just do it!" encouragement to Jon
- conv-30-47 (single-hop): What did Gina find for her clothing store on 1 February, 2023? | gold: The perfect spot for her store | answer: On February 1, 2023, Gina found a wholesaler willing to work with her. She had emailed several wholesalers, and one replied yes, which let her expand her clothi
- conv-30-73 (single-hop): How does Gina describe the feeling that dance brings? | gold: magical | answer: Gina describes dance as her stress relief: as soon as she starts, all her worries vanish. She says she can't picture life without it, comparing it to air. She a
- conv-41-48 (temporal): When did John have his first firefighter call-out? | gold: The sunday before 3` July 2023 | answer: John's first firefighter call-out was on Sunday, July 30, 2023.

## wrong_abstain (examples)

- conv-47-17 (open-domain): What is the board game where you have to find the imposter that John mentions to James? | gold: Mafia | answer: The memories contain no mention of an imposter. The closest match is "Dungeons of the Dragon," the board game John told James about on 2022-09-18, but nothing i

## wrong_retrieval (examples)

- conv-26-59 (open-domain): Would Caroline be considered religious? | gold: Somewhat, but not extremely religious | answer: Probably not. Her memories show no religious beliefs or practices. Her identity and values center on the LGBTQ community, acceptance, advocacy, and family. The 
- conv-26-66 (multi-hop): What does Melanie do with her family on hikes? | gold: Roast marshmallows, tell stories | answer: Melanie's family hikes in the mountains and explores forests together. She says it's a cool way to connect with nature and with each other. Examples include a h
- conv-26-69 (open-domain): What personality traits might Melanie say Caroline has? | gold: Thoughtful, authentic, driven | answer: Melanie would likely describe Caroline as:  - **Empathetic and understanding**: "You'd be a great counselor! Your empathy and understanding will really help the
- conv-42-53 (multi-hop): How many times has Nate taken his turtles on a walk? | gold: Twice. | answer: Once. Nate took his turtles on a walk on 2022-10-25, when he was bored.
- conv-42-60 (open-domain): What Console does Nate own? | gold: A Nintendo Switch | answer: Nate doesn't appear to own a specific console. His gaming setup looks like a PC rig: a desk with two monitors and a laptop, plus headphones and a video game con
- conv-42-66 (open-domain): What alternative career might Nate consider after gaming? | gold: an animalkeeper at a localzoo and workingwith turtles | answer: Nate might consider screenwriting, specifically writing a drama and publishing his own screenplay, which he mentioned in May 2022. He had also started creating 

## wrong_ingest (examples)

- conv-30-44 (single-hop): What does Gina say about the dancers in the photo? | gold: They look graceful | answer: Gina shared a photo of a group of dancers on a stage with their arms in the air on 2023-07-23, along with a push to keep going: "Just do it! You should get to t
- conv-42-4 (open-domain): What pets wouldn't cause any discomfort to Joanna? | gold: Hairless cats or pigs,since they don't have fur, which is one of the main causes of Joanna's allergy. | answer: Only non-living pets wouldn't bother Joanna. The stuffed dog Nate gave her, which she named Tilly, is always with her while she writes. She can also enjoy Nate'
- conv-42-8 (temporal): When did Nate get his first two turtles? | gold: 2019 | answer: Nate already had his first two turtles by January 23, 2022, the earliest date they appear in the memories. The exact purchase date isn't recorded.
- conv-42-51 (multi-hop): When did Nate get Tilly for Joanna? | gold: 25 May, 2022 | answer: Nate gave Joanna Tilly before 2022-10-21, when she mentioned she still had the stuffed dog he had given her. The memories don't give the exact date of the gift.
- conv-42-52 (multi-hop): How many of Joanna's writing have made it to the big screen? | gold: two | answer: Three of Joanna's writings have made it to the big screen. The third was shown on 2022-10-23. Her new love-story script is still in progress and has not been re
- conv-43-49 (multi-hop): How many times has John injured his ankle? | gold: two times | answer: John has injured his ankle once. He hurt it last season, before 2023-11-21, and needed time off and physical therapy. A later injury in late 2023 involved a cas

## no_evidence (examples)

- conv-26-30 (open-domain): Would Melanie be considered a member of the LGBTQ community? | gold: Likely no, she does not refer to herself as part of it | answer: Probably not. Melanie comes across as a supportive friend and ally of Caroline and the LGBTQ community, not as a member. She speaks about the community from the
- conv-26-46 (open-domain): Would Melanie be considered an ally to the transgender community? | gold: Yes, she is supportive | answer: Yes. Melanie would be considered an ally to the transgender community. She consistently affirms Caroline's identity, praises her courage as a trans person, supp
- conv-50-39 (open-domain): Would Dave prefer working on a Dodge Charger or a Subaru Forester? | gold: Dodge Charger | answer: Dodge Charger
- conv-50-42 (open-domain): Based on the conversation, did Calvin and Dave have a meeting in Boston between August and November 2023? Answer in yes or no. | gold: No | answer: No. Calvin and Dave planned to meet in Boston, but the conversations only show the plan. Calvin's last message on 2023-11-17 still described the meeting as upco
- conv-50-69 (temporal): When did Dave buy a vintage camera? | gold: November 2023 | answer: November 2023
