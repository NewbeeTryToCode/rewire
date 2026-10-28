# Landing design direction

Source of truth for `docs/design/preview/mockup-v2.html`. Facts below are read from the shipped design. Lines marked **owner to confirm** need a reason from the product owner; they were not invented here (antislop R-37, R-31).

## Dials
ENERGY 2 / RHYTHM 1 / MOTION 1 (after audit 001: endless marquees pause on hover and stop under `prefers-reduced-motion`; the hero canvas and the two flow diagrams are the only moving parts).
**Owner to confirm** whether RHYTHM 1 (every section the same shape) is deliberate.

## Palette
- Base `#07060f`, card `#0b0920`.
- Text `#e6e6f5`, muted `#9a94d8`, quiet `#8a84c8` (5.96:1 on base, 5.79:1 on card).
- Accent violet `#b09dff`, blue `#7dd3fc`, cream `#fdf9c4`.
- Gradient violet to blue to cream on the H1 and the two buttons.
- **Owner to confirm:** why a dark violet scheme with a gradient (R-01, R-21). A developer tool is a valid reason for dark; the gradient needs its own.

## Type
- Geist (sans) for headings and body, Geist Mono for labels, commands and code.
- Section labels are mono, uppercase, wide-tracked (`// 01 · HOW IT WORKS`).
- **Owner to confirm:** the reason for the mono label voice (R-06). A command-line product is the obvious one, but it should be written down.

## Voice
Plain statements about what the tool does. No claims without a source in `README.md`, `docs/evaluation.md` or the code. Harness support is stated as it is: hooks for Claude Code, a skill for Codex, MCP or the AGENTS.md snippet for everything else.

## Rules for changes
- No em dashes in UI text. Contrast 4.5:1 or better for text, 3:1 for components.
- Placeholder or example output is labelled as such.
- Do not add a harness, number or customer to the page unless it is real.
