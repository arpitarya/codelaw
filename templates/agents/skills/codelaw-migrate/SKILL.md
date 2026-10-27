---
name: codelaw-migrate
description: Move a repo from ESLint, typescript-eslint, Ruff, stylelint, html-validate or markdownlint to codelaw, or turn a written style guide into codelaw rules. Use when the user says "move from ESLint", "replace Ruff with codelaw", "migrate our lint config", "turn our style guide into rules", or "port our CONTRIBUTING conventions". Maps each existing rule to a codelaw rule through ported_from, picks presets, drafts rules for prose conventions, and writes a mapping report of what was covered, dropped or needs a human.
---

# codelaw-migrate — rule by rule, with a report

A migration is only trustworthy if every source rule ends up in exactly one
column: **covered**, **covered differently**, **dropped (with a reason)** or
**needs a human**. You produce that table; codelaw's own rule metadata is the
source of the mapping, not your memory of the old tool.

## 0. Run `codelaw migrate` first

For ESLint, Prettier, Biome, Ruff and stylelint the translation is a
command, not a judgement: `codelaw migrate <tool> --format json` reads the
config (never running one that is a program), maps every rule through
`ported_from:`, and returns `mapped`, `changed`, `unmapped` and `ignored`
lines and the `config` it would write. Show the user the report, then
`codelaw migrate <tool> --write` (run it once per tool; a second tool adds to
the same `codelaw.toml`). Your work starts from its `changed` and `unmapped`
lines: steps 2–5 below, for those only. A config it refuses as a program:
ask the user to convert it to JSON, or list its rules with them by hand.
For html-validate, markdownlint and prose conventions, start at step 1.

## 1. Inventory the source

Collect every config in scope: `eslint.config.*` / `.eslintrc*`,
`pyproject.toml` `[tool.ruff]` / `ruff.toml`, `.stylelintrc*`,
`.htmlvalidate.*`, `.markdownlint*`, and for prose conventions
`CONTRIBUTING.md`, ADRs, style guides. List each enabled source rule with its
severity and options, as `<tool>/<rule>` (for example `eslint/no-var`,
`ruff/F401`). Resolve `extends` presets of the old tool to their rules.

## 2. Map through `ported_from`

codelaw rules that reimplement a source rule say so in `ported_from:`.

1. Make sure codelaw is set up with the packs for these languages
   (`codelaw-setup` if not).
2. `codelaw rules --format json` lists every rule; for each id,
   `explain_rule` (CLI: `codelaw explain <id> --format json`) returns
   `ported_from`. Build the map `<tool>/<rule>` → codelaw id from that data.
3. For each source rule:
   - one codelaw rule ports it → **covered**; carry over severity and options
     in `[rules]` of `codelaw.toml` only if they differ from the preset;
   - a preset already enables it → **covered**, no config line needed;
   - no codelaw rule ports it → check whether another codelaw rule enforces the
     same thing (read its Do/Don't); if so, **covered differently**, else
     **needs a human** or **dropped**.
4. Layout-only rules (indentation, quotes, semicolons, line length) are the
   formatter's job in codelaw, not lint rules: mark them **covered by the
   formatter**.

Prefer presets in `extends` over long `[rules]` lists.

## 3. Prose conventions → draft rules

For each convention in prose that a machine can check, draft a rule with the
`codelaw-author-rule` loop (failing Don't first, `codelaw test` green). Mark
drafts `stability: preview`. A convention no matcher can express goes in the
report as **needs a human**.

## 4. Run both, compare

Before removing the old tool, run it and `codelaw check --format json` on the
same files. For each source finding, note whether codelaw reports the same
location. Differences go in the report with the reason. Do not tune codelaw
rules to hide differences.

## 5. Write the report

A Markdown table at a path the user picks (default
`codelaw-migration.md`): source rule · status · codelaw id · note. Then
counts per status. Removing the old tool, its config and its dependencies is a
separate step the user approves.

## Never

- **Never claim a rule is covered** without a `ported_from` entry or a read of
  the codelaw rule's examples that shows the same check.
- **Never drop a rule silently**; every dropped rule has a reason in the
  report.
- **Never remove the old linter** before the user has seen the report.
