---
type: Handoff
name: W-145
description: "Arpit, 2026-10-30 (voice): a Tailwind pack — its bug-catching rules and the class-order table as data — and Tailwind class sorting in codelaw's formatter, on when the project uses Tailwind."
item: W-145
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-145 — The Tailwind pack and class sorting in the formatter

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Amend L6: a `tailwind` pack | one package to install | ⏳ |
| 2 | The class-order table as pack data, matching `prettier-plugin-tailwindcss`'s order | no churn for teams switching | ⏳ |
| 3 | The formatter sorts `class`/`className` (and `clsx`/`cn`/`tw` calls, configurable) when the pack is installed and the project uses Tailwind | sorting with no extra tool | ⏳ |
| 4 | The lint rules W-144 picked: the 8 `tailwind` rows in the two Tailwind CSVs (conflicting, unknown, concatenated classes …); `tailwind/recommended` on when the pack is installed | bugs caught | ⏳ ruled 2026-10-30 |

**Arpit, 2026-10-30 (voice):** *"I want the class sorting in the formatter."* · on a linter package and a formatter package: told packs hold data, never code, and the formatter lives in the engine — *"But will the package size increase significantly?"* → no, tens to a few hundred kilobytes (estimate) → *"Okay. Then let's go for it."*

**Model:** Opus (Claude Code).

## Goal

- **One pack, both registries** (Tailwind classes live in JSX, HTML and Python templates): `@codelaw/rules-tailwind`, `codelaw-rules-tailwind` (PyPI name free, checked 2026-10-30).
- **Sorting is on** when the pack is installed and the project uses Tailwind (a `tailwindcss` dependency, a `tailwind.config.*`, or `@import "tailwindcss"` in a stylesheet); `[format.tailwind] sort = true | false` overrides; plain Prettier does not sort, so it is never on without Tailwind.
- **Where it sorts:** `class` in HTML, `className` in JSX/TSX, and calls named in `[format.tailwind] functions` (default `clsx`, `cn`, `cva`, `tw`).
- **The order:** Tailwind's own (layer, then property order, then variants), from a table generated once from Tailwind and committed as pack data; custom utilities from the project's CSS `@theme`/`@utility` read as data; unknown classes first, as the Prettier plugin does.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · law | L6 amended (the pack, both registries); CLAUDE.md's L6 line | ⏳ |
| 2 · table | generated from Tailwind v4 by a script in `scripts/`, its source and version recorded; size measured | ⏳ |
| 3 · sorting | byte-identical to `prettier-plugin-tailwindcss` on a corpus of Tailwind projects (≥ 98% of files) | ⏳ |
| 4 · names | npm first publish and PyPI pending publisher (as W-134/W-135) | 🔴 Arpit, before release |
| 5 · records, docs | SR-FORMATTER (the `[format.tailwind]` keys), SR-RULE-PACKS, SR-PACKAGES; CHANGELOG | ⏳ |

## Blockers

None — W-144 ruled 2026-10-30.

## Record affected

SR-LAW-6 · [SR-FORMATTER](../../records/0105_formatter.md) · SR-RULE-PACKS · SR-PACKAGES.

## Hazards

- **No code in a pack, no JavaScript at runtime:** `prettier-plugin-tailwindcss` asks the project's installed Tailwind for the order; codelaw cannot (L5; Node is allowed only for type rules). A project's JS plugins that add utilities cannot be ordered exactly — they sort as unknown; say so in the docs.
- Sorting is a formatting change: the layout check must still see the same tokens (the class string is one token; only its contents reorder — record why that is layout).

## Reference

[W-144](../../archive/open/W-144-rule-parity-round-2.md) · [W-141](W-141-formatter-parity.md) · prettier-plugin-tailwindcss (MIT)
