---
name: codelaw-write
description: Write or edit code that passes this repo's codelaw rules the first time. Use whenever you are about to create or change a JavaScript, TypeScript, JSX/TSX, Python, JSON, YAML, TOML, Markdown, CSS or HTML file in a repository that has a codelaw.toml (or a [tool.codelaw] table in pyproject.toml, or a "codelaw" key in package.json). Asks codelaw which rules apply to the path before writing, writes compliant code, then runs codelaw on the changed files and handles every finding by its fix class.
---

# codelaw-write — ask before you write, check after

codelaw is this repo's linter and formatter. Its rules are Markdown files, and
**codelaw decides every verdict** — you never decide on your own whether code
complies. This skill is the loop you follow for every file you create or edit.

## 1. Before you write: ask which rules apply

For each file you are about to create or change:

- **MCP (preferred):** call `rules_for_path` with `{"path": "<the file>"}`. The
  file need not exist yet.
- **CLI:** `codelaw rules --for <the file> --format json`.

You get one short card per active rule: `id`, `severity`, `fix` class and
`title`. Read every card. For any rule whose title does not tell you exactly
what to write, call `explain_rule` with `{"id": "<rule id>"}` (CLI:
`codelaw explain <id>`) and read its **Do** and **Don't** examples — they are
the most precise statement of what the rule wants.

Do not inline the whole rule set into your plan; ask per path, and only open
the detail you need.

### Adding or upgrading a dependency

Before you add a package to `package.json` or `pyproject.toml`, or change its
version, ask codelaw about it — offline, from the committed snapshot:

- **MCP:** `check_dependency` with `{"ecosystem": "npm" | "pypi", "name": "<package>", "version": "<the version you would use>"}`.
- **CLI:** `codelaw deps check <npm|pypi> <package> [version] --format json`.

When it says `vulnerable`, use its `target` — the lowest version every advisory
leaves clean — instead; when `target` is null, tell the user and do not add it.
Never run an install to find out: codelaw never runs installers, and neither do
you until the version is settled. `rules_for_path` on a manifest, or on a file
that imports a package, also lists the packages it brings in that are vulnerable
or a major version behind, under `dependencies`.

## 2. Write code that complies

Write the change so that it already follows every rule on the cards. When a
rule and the user's request conflict, say so to the user before writing; do
not silently break the rule and do not silently ignore the request.

If you are writing code you have not saved yet and want a verdict first, call
`check_snippet` with `{"path": "<file>", "code": "<full file text>"}`.

## 3. After you write: check the files you changed

Run codelaw on exactly the files you created or changed:

- **MCP:** `check_paths` with `{"paths": ["<file>", ...]}`.
- **CLI:** `codelaw check <file> ... --format json`.

A post-edit hook may already have run `codelaw hook` and applied the `auto`
fixes; if it reported findings back to you, start from that report.

## 4. Handle each finding by its fix class

Every diagnostic carries `fix.class`. Handle them in this order:

1. **`auto`** — mechanical and safe. Apply them with `fix_paths`
   `{"paths": [...]}` (CLI: `codelaw fix <files>`). Do not hand-edit what an
   `auto` fix will do for you.
2. **`suggest`** — mechanical but might change behaviour. The diagnostic
   carries `fix.edits` (byte ranges and replacement text). **Show the user the
   change as a diff and apply it only after they accept.** To apply accepted
   suggestions, call `fix_paths` with `{"paths": [...], "unsafe": true}` (CLI:
   `codelaw fix --unsafe <files>`) — that applies every suggestion in those
   files, so only do it when every suggestion there was accepted; otherwise
   apply the accepted edits by hand.
3. **`manual`** — no mechanical fix. The diagnostic carries `fix.steps`.
   Follow the steps **exactly and in order**. If a step is unclear, call
   `explain_rule` for the rule. For a longer manual fix, switch to the
   `codelaw-fix` skill (or the skill the rule names in `explain_rule`'s
   `fix.skill`).

Then check the same files again. Repeat until the check is clean or only
findings the user has explicitly decided to keep remain.

## 5. Done when

- `check_paths` (or `codelaw check`) on every file you touched reports no
  `error` and no `warn` you introduced, **and**
- you have told the user about any `suggest` fix you did not apply and any
  finding you could not resolve, with the rule id.

## Never

- **Never disable, loosen or eject a rule to make a check pass**, and never
  edit `codelaw.toml` to switch a rule off unless the user asked for exactly
  that.
- **Never suppress a finding to make it go away.** If the user says a finding
  is intended, use the `codelaw-suppress` skill; a suppression always carries
  a reason.
- **Never edit a generated file** — skills under `.claude/skills/`,
  `.github/skills/`, `.agents/skills/`, `.kiro/`, or the text between
  `<!-- codelaw:begin -->` and `<!-- codelaw:end -->` in `CLAUDE.md` or
  `AGENTS.md`. They are rewritten by `codelaw sync-agents`.
- **Never claim code is compliant from your own reading.** Only a clean
  codelaw result says so.
