---
name: codelaw-port-rule
description: Bring a rule from another tool into codelaw — ESLint and its plugins, typescript-eslint, Ruff, Pylint, flake8 plugins, stylelint, Biome, markdownlint, html-validate, Semgrep or any other. Use when the user says "I want ESLint's no-nested-ternary in codelaw", "port pylint's global-statement", "codelaw is missing stylelint's declaration-no-important", "add <tool>/<rule> to codelaw", or pastes a link to another linter's rule page. First looks for a codelaw rule whose ported_from names it and says how to turn it on; otherwise reads the source rule's docs and tests, writes the rule fresh into .codelaw/rules/ as a preview rule, and stops only when codelaw test is green. Then offers, never assumes, to share the port with codelaw.
---

# codelaw-port-rule — another tool's rule, in codelaw, tested

The user names a rule another tool has (`<tool>/<rule>`, or a link to its
page). You either point at the codelaw rule that already ports it, or write
it into **their** project as a project rule — fresh, from what the source
rule does — and **`codelaw test` decides** when it works. A human reviews
and merges it.

## 1. Name the source rule

Normalise what the user gave to the id a rule file's `ported_from:` takes,
`<tool>/<rule>`:

| tool | id |
|---|---|
| ESLint core | `eslint/no-nested-ternary` |
| an ESLint plugin | its package: `eslint-plugin-react/jsx-key`, `@eslint-react/…`, `eslint-plugin-n/…` |
| typescript-eslint | `typescript-eslint/no-floating-promises` |
| Ruff | `ruff/<code>` (`ruff/PLW0603`) |
| Pylint | `pylint/<symbol>` (`pylint/global-statement`, not `W0603`) |
| flake8 plugin | its package and code: `flake8-bugbear/B006` |
| stylelint | `stylelint/<rule>`; a plugin by package: `stylelint-scss/…` |
| Biome, markdownlint, html-validate, Semgrep, … | `biome/<rule>`, `markdownlint/MD033`, `html-validate/<rule>`, `semgrep/<rule-id>` |

Say the id back to the user in one line before going on.

## 2. Check codelaw first — never port a rule twice

Look for a rule whose `ported_from:` lists that id (also the same rule under
its other names: a Ruff code for a Pylint symbol, `ruff/PLW0603` for
`pylint/global-statement`; Biome's source for a Biome rule):

1. **The rules this project loads:** MCP `list_rules` (CLI: `codelaw rules
   --format json`). Each card carries `ported_from` when the rule has one.
   That is every rule its presets and `[rules]` turn on, and the project's
   own `.codelaw/rules/` — not a pack rule that is installed but off.
2. **Every first-party rule, installed or not:** the catalogue
   `https://raw.githubusercontent.com/arpitarya/codelaw/main/docs/rules/rules.json`
   (each rule's `ported_from`, `packs`, `npm`, `pypi`, `presets`). Fetch it
   yourself; no `codelaw` command fetches.

**Found** → stop porting. Tell the user the codelaw id, `explain_rule <id>`'s
summary, and how to turn it on: the preset that has it (`extends =
["codelaw:<pack>/<preset>"]`), or one line under `[rules]` (`"<id>" =
"error"`), plus `codelaw init` / the package to install when its pack is not
installed. A rule in `.codelaw/rules/` with that `ported_from:` is the
user's own earlier port: say so and stop. If a first-party rule exists *and*
the project has its own port, say the project file can be deleted.

## 3. Read the source rule — read, never run

Read its documentation page and, where the source is public, its test
file: what it reports, where it puts the report, what it leaves alone, its
options and their defaults, and whether it has an autofix.

- **Never run** the source tool, its rule, its config or its tests; never
  install it. You read them.
- **Licences (L2):** write fresh. Pylint is GPL-2.0, other sources may be
  copyleft: never paste their tests, examples or doc text into the rule —
  write new cases from what the rule does. From a permissive source (MIT,
  Apache-2.0, BSD) still prefer new cases; one reused verbatim keeps its
  copyright notice in the cases file.
- Ask the user only what the docs leave open and the port needs (which
  options to carry; whether to keep a quirk).

## 4. Can codelaw express it?

A project rule is declarative: a `match:` ast-grep rule with `where:`
predicates, a `target:`/`assert:` rule for JSON, YAML or TOML keys, or a
`graph:` rule for cross-file facts ([`codelaw-author-rule`](../codelaw-author-rule/SKILL.md)).
It cannot be a `native:` rule (engine code), and it cannot use type
information the project has not asked for.

If the source rule needs either, **say which, and stop**: offer step 7's
share prompt as a request for a first-party rule instead. A partial port is
allowed only when the user agrees, and its summary names what it does not
cover.

## 5. Write it — the authoring loop

Follow `codelaw-author-rule` (or hand the whole port to the
`codelaw-rule-porter` subagent, which runs steps 3–6 in its own context and
returns only the result). The source fills the loop's inputs:

- **Scaffold from the template:** `codelaw new-rule <namespace>/<name>
  --lang <languages> --fix <class>`. The namespace is the project's (ask, or
  use the one its other rules use), **never** a first-party one (`js`, `ts`,
  `py`, `css`, …); the name is the source rule's where it fits.
- **Frontmatter:** keep `stability: preview`; add `ported_from:
  [<tool>/<rule>]` (and the same rule's other ids from step 2).
- **The source's invalid cases are the Don'ts, its valid cases the Dos** —
  rewritten in your own words and code, one `### <Language>` per declared
  language, and its edge cases as `<name>.cases.md` (`### case:` sections
  with an `expect` fence), including the near-misses the source does not
  report.
- **Fix class (L4):** the source's autofix suggests `auto` (an edit that
  cannot change behaviour) or `suggest` (it can; give `fix_reason:`). No
  autofix → `manual` with `fix_reason:` and a `## How to fix` precise
  enough for another agent.
- **`**Why:**`** says where it was ported from, and: *if a first-party pack
  ships a rule with the same `ported_from:`, turn that one on and delete this
  file.*
- A known difference from the source (an option not carried, a form the
  grammar does not parse) goes under a last section, `## Differences from
  the source`, as the first-party ports do.

## 6. Done means green

`codelaw rules lint`, then `codelaw test <id>` until every example and case
passes. Change one `expect` line and run it again: it must fail — a case that
cannot fail tests nothing — then put it back. Then `codelaw check --rule
<id>` on the project and show the user the count and two or three samples.

Hand back: the rule file and cases file paths, the last `codelaw test <id>`
output, and the check count. The user reviews and merges; you never mark the
rule as passing yourself.

## 7. Offer to share — once, opt-in

After a green port (or when step 4 stopped), ask exactly once:

> Share this rule with codelaw? It opens a pre-filled issue on
> arpitarya/codelaw that you review and submit; nothing is sent otherwise.

**No, or no answer → stop.** Nothing is sent, and you do not ask again.

**Yes →** open the `rule_request.yml` form pre-filled, for the user to
review and submit in the browser:

```
https://github.com/arpitarya/codelaw/issues/new?template=rule_request.yml
  &title=port%3A%20<tool>%2F<rule>
  &labels=ported-rule
  &ported_from=<tool>%2F<rule>
  &source=<the source rule's page>
  &convention=<the rule's summary paragraph>
  &dont=<the first Don't>
  &do=<the first Do>
  &rule_file=<the rule file's text>
```

each value URL-encoded, one line, no spaces. Show the URL and open it
(`open` on macOS, `xdg-open` on Linux, `start` on Windows). When the URL
would pass about 8,000 characters, leave `rule_file` out and tell the user to
paste the file into that field. The title `port: <tool>/<rule>` is how
maintainers count requests. With `gh` and the user's explicit yes to the
exact text you show, `gh issue create --repo arpitarya/codelaw --title
"port: <tool>/<rule>" --label ported-rule --body-file <file>` is the
alternative (a body using the form's `### <label>` headings); drop
`--label` if `gh` refuses it.

## Never

- **Never run** a source rule, its tool, its config or its tests.
- **Never paste** a copyleft source's tests, examples or documentation; write
  new ones.
- **Never port a rule codelaw already has** — step 2 first, every time.
- **Never use a first-party namespace** or write a `native:` rule.
- **Never weaken an example or delete a case to get green**; fix the matcher.
- **Never claim the port works** without a green `codelaw test`, and never
  mark it `stable`.
- **Never send anything without the user's yes** to step 7, and never open
  an issue or a pull request on their behalf otherwise.
