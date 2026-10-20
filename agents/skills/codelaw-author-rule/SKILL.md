---
name: codelaw-author-rule
description: Turn a team convention into a tested codelaw rule file in .codelaw/rules/. Use when the user says "make a rule that…", "enforce X", "we always do Y", "forbid Z in our code", "add a lint rule for…", or describes a convention they want checked automatically. Writes a failing example first, inspects the syntax tree, writes the matcher, picks the fix class, writes Do/Don't examples for every language, and runs codelaw test until green. For long iterative authoring, the codelaw-rule-author subagent runs the same loop in its own context.
---

# codelaw-author-rule — a convention becomes a rule, test first

A codelaw rule is one Markdown file, `.codelaw/rules/<namespace>/<name>.md`:
frontmatter, one summary paragraph, `**Why:**`, a ` ```codelaw ` block,
`## How to fix` (manual rules), and `## Examples`. You write it; **`codelaw
test` decides whether it works**, and a human reviews and merges it. You
never mark your own rule as passing.

## 0. Pin down the convention

Before any file, restate the convention in one sentence and confirm it:

- what exactly is forbidden or required;
- which languages (codelaw names: `javascript`, `jsx`, `typescript`, `tsx`,
  `python`, `json`, `yaml`, `toml`, `markdown`, `css`, `html`);
- which files (`files:` / `exclude:` globs, relative to the repo root), if
  not all;
- one real example of a violation from this repo, if there is one.

Check it is not already a rule: `list_rules` (CLI: `codelaw rules`) and
`explain_rule` on likely ids. If a pack rule is close, tell the user — enabling
or configuring it beats a new rule.

Pick an id `<namespace>/<name>`: the namespace is the team's (for example
`acme`), never a reserved one (`js`, `ts`, `react`, `node`, `stencil`, `py`,
`json`, `yaml`, `toml`, `md`, `data`, `css`, `html`, `graph`, `format`,
`codelaw`).

## 1. Scaffold

`codelaw new-rule <namespace>/<name> --lang <l1,l2> [--fix auto|suggest|manual]`
writes the template with every required field. Open it.

## 2. Write a failing Don't first

Under `## Examples` → `### <Language>` → `#### Don't`, put the smallest real
violation. Run `codelaw test <id>`: it must **fail** now (the Don't does not
fire yet). A Don't that already passes means the matcher is not testing
anything.

## 3. Inspect the tree

`dump_tree` `{"code": "<the Don't>", "lang": "<language>"}` (CLI:
`codelaw tree --lang <l> -e "<code>"`, or `codelaw tree <file> --line N`).
Note the node `kind`s and `field` names you need. Write matchers against what
the tree shows, not against what you expect it to show.

## 4. Write the matcher

Use the simplest form that expresses the convention:

- `match:` — an ast-grep rule (`pattern:`, `kind:`, `has`/`inside`/`not`,
  `constraints:`), plus `where:` semantic predicates (`unused`, `undeclared`,
  `imported-from`, `reassigned`, `binding`, …) for scope-aware checks;
- `target:` + `assert:` — for keys and values in JSON, YAML or TOML;
- `graph:` — only for cross-file facts (`scope: project`).

Then `message:` (use `$CAPTURE` names so the message shows the code) and, for
`auto`/`suggest`, `fix:` (a template, or edit primitives such as `replace`,
`remove`, `insert-before`, `move-to`).

## 5. Choose the fix class (L4)

- `auto` — only when the edit is mechanical **and** cannot change behaviour.
- `suggest` — mechanical but could change behaviour; needs `fix_reason:`.
- `manual` — no mechanical edit; needs `fix_reason:` and a `## How to fix`
  with numbered steps precise enough for another agent to follow. Name a
  specialised skill in `fix_skill:` only if codelaw ships it.

"No fix" is not an option; the loader refuses it.

## 6. Do/Don't for every declared language

Each language in `languages:` needs its own `### <Language>` with at least one
`#### Don't` and one `#### Do`, and `#### Fixed` when the fixed Don't differs
from the Do. Use `path=<file>` on a fence when `files:` must apply.

## 7. Edge cases

In `<name>.cases.md`, add `### case: <title>` sections — an input fence, an
`expect` fence (`line:col rule-id` lines, or `(none)`) and an optional
`fixed` fence — for the near-misses that must **not** fire and the awkward
shapes that must.

## 8. Run until green

`codelaw rules lint` (template check) and `codelaw test <id>` (examples,
cases, fix(Don't) = Do, fixpoint, invariants). Fix the rule, not the test.
Repeat until both pass. Then run `codelaw check` on the repo to see what the
new rule finds in real code, and show the user the count and two or three
samples — a surprising number is usually a matcher that is too broad.

## Hand back

The rule file path, the cases file path, the `codelaw test` output, and what
`codelaw check` found. Leave `stability: preview` unless the user says
otherwise.

## Never

- **Never weaken an example or delete a case to get green**; change the
  matcher.
- **Never write a `native:` rule** — those are engine code, first-party only.
- **Never claim the rule works** without a passing `codelaw test`.
