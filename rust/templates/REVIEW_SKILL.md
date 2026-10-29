---
name: rewire-review
description: Show the user's rewire memory inbox (proposed memories waiting for review) so they can approve or reject them from the chat. Use when the user runs /rewire-review or asks to review, approve or reject proposed memories.
---

# Review the rewire inbox

The user approves or rejects proposed memories by sending a message of review commands. A hook carries those
out from the user's own message. **You never approve, reject or edit memories yourself**, and you never run
`rewire review approve|reject` or `rewire hook`: rewire refuses both from an agent session, and trying is wrong.

## 1. Read the inbox

```bash
rewire review
```

## 2. Show it to the user

Write in the user's language. Group the items and keep each item's text as the inbox says it:

1. **Constraints and contradictions first.** These change future sessions the most.
2. **Duplicates.** Items that say the same thing as another item or an existing memory: suggest rejecting all but one.
3. **Everything else**, by type (preference, decision, fact, skill).

For each item give its id (its first 12 characters: ids proposed in the same minute share the first 10), type, scope, its full text, and your recommendation:
approve, reject, or "check" with one line of reason. Mark items labelled `FROM ANOTHER DEVICE` or flagged by the
scan, and do not recommend approving those without saying why.

## 3. Hand the decision to the user

End with ready-to-copy lines that follow your recommendations, in one code block, for example:

```
rewire approve 01M61NFFZ5J8 01M61NFG063W
rewire reject 01M60TX5CXH8
```

Tell the user to send those lines **as their own message, with nothing else in it** (they may edit them first).
After they send it, the hook adds a `# rewire review` block to your context with the outcome: report it to the
user exactly as listed. If it says the review was not carried out, tell them why and do not retry it yourself.
