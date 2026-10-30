---
type: Handoff
name: W-143
description: "Arpit, 2026-10-30 (voice): build Biome's 89 rules for Vue, Svelte, Astro and GraphQL — which means adding the four as codelaw languages, each linted and formatted."
item: W-143
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-143 — Vue, Svelte, Astro and GraphQL as codelaw languages

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Amend L3 (four languages) and L7 (each linted and formatted) | the law allows it | ⏳ |
| 2 | A parser for each — the decider ratifies each grammar by name (L11) | codelaw can read the files | 🔴 decider ratifies, when this starts |
| 3 | A formatter for each, Prettier's output as the target | L7 kept | ⏳ |
| 4 | Biome's 89 rules for these languages (`work/proposals/rule-parity/biome.csv`, class `out-of-scope`), plus each language's own ESLint plugin as round-2 research finds them | the rules Arpit asked for | ⏳ |

**Arpit, 2026-10-30 (voice):** *"The biome 89 rules, let's go ahead and do that as well."* — told this is four new languages, not 89 rules (L3, a parser each, a formatter each under L7), and asked whether to file it as its own item after W-139: *"Yes."*

**Model:** Opus (Claude Code).

## Goal

The four become L3 languages with the same standing as the nine: embedded `<script>`/`<style>` blocks linted in their own language (SR-LINTER's embedded-code path), a formatter each, rule packs as the round-2 research proposes.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · law | SR-LAW-3 and SR-LAW-7 amended on this ruling; CLAUDE.md's L3/L7 lines | ⏳ |
| 2 · parsers | four grammars, each ratified by name in L11 §3, vendored and pinned as tree-sitter-scss was (W-62) | 🔴 |
| 3 · formatters | each within 2% of Prettier (with its official plugin) on a pinned corpus | ⏳ |
| 4 · rules | Biome's 89 classed and ported; packs and registries as L6 is amended | ⏳ |
| 5 · budgets | L8 measured per language | ⏳ |

## Blockers

🟡 waits on **W-139**. 🔴 at start: the decider ratifies each grammar (L11).

## Record affected

SR-LAW-3 · SR-LAW-7 · SR-LAW-11 · SR-LAW-6 (packs) · SR-LINTER · SR-FORMATTER.

## Hazards

- Four formatters is the bulk of the work, not the 89 rules.
- Svelte and Astro mix JS/TS, HTML and CSS: reuse the existing languages through the embedded-code path rather than re-parsing them.
- Grammar licences must pass L2; generated parser sources committed as W-62 did.

## Reference

[Rule-parity proposal](../proposals/rule-parity.md) · W-62 (how SCSS was added) · W-139
