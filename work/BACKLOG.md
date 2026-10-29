---
type: Queue
description: "The named-but-unclaimed: everything a record, proposal or compare doc says is outstanding and nobody is doing. The list only — its rules live in the record below."
governed_by: SR-WORK-BACKLOG
governed_by_path: records/0052_WORK-backlog.md
---

# BACKLOG — what has been named and is not being done

**This file is the LIST.** Everything else — what the backlog is, what feeds
it, the shape of a row, the five classes, how an item leaves, and the one
obligation it puts on every session — is stated once in
**[SR-WORK-BACKLOG](../records/0052_WORK-backlog.md)** and is not repeated here.

⚠ **Its length is not a signal.** `OPEN-WORK.md`'s length is how much is
pending; this file's length is how honest the records have been. Do not trim
it, and do not read it top-down as a priority order.

**Nothing here is in [`OPEN-WORK.md`](OPEN-WORK.md).**

---

## `unbuilt` — a ratified decision the code does not implement

| id | what is outstanding | named in | what closes it |
|---|---|---|---|
| B-063 | `publish.yml` does not upload the Fux index packages (`dist/fux-index/`): they reach npm and PyPI only by hand. Moved here when W-34 closed into the PyPI days (Arpit, 2026-10-29) | [SR-PACKAGES](../records/0107_packages.md) decision 14 · [L6](../records/0008_LAW-6-python-and-node-packages.md) decision 12 · `archive/open/W-34-fux-index-packages.md` | `publish.yml` publishes `dist/fux-index/` on every release, each name trusted, a missing PyPI name skipped as W-116's rule |
| B-043 | "Diagnostics: pull and push" and "Workspace diagnostics: for project rules" — the server pushes only, per open document | [SR-EDITOR](../records/0114_editor.md) decision 1 | `textDocument/diagnostic` and `workspace/diagnostic` served, with a server test; project rules wait on W-11 |
| B-044 | the "Explain rule" and "Ignore here with reason" code actions | [SR-EDITOR](../records/0114_editor.md) §1 table, Quick fixes row | both offered as code actions, tested, writing nothing |
| B-046 | the tree-sitter-css upstream PR: `grammar.patch`, a ready commit with `src/` regenerated (`0001-tree-sitter-css-modern.patch`) and `PR.md` in `crates/codelaw-lang/grammars/tree-sitter-css/upstream/`; not filed — **Arpit, 2026-10-24:** *"Then let's keep using the patch copy and put the PR in backlog."* | [SR-ARCHITECTURE](../records/0108_architecture.md) decision 3 · [W-21](../archive/open/W-21-pack-engine-gaps.md) | the PR is opened on tree-sitter/tree-sitter-css; the fork is dropped when a release carries it |
| B-048 | The *later* presets: `js/strict` (typescript-eslint `strict`, jsx-a11y, unicorn), `react/a11y`, `py/strict`, and an opt-in pydocstyle `D` preset | [SR-RULE-PACKS](../records/0101_rule-packs.md) §1; the research table of [W-03](../archive/open/W-03-rule-packs.md) | each preset ships in its pack, or the decider rules it out |
| B-050 | `codelaw migrate --from eslint\|ruff\|stylelint\|biome` (no oxlint): "Maps existing ESLint, typescript-eslint, Ruff, stylelint … configs rule by rule through `ported_from:`" and writes "a mapping report"; was W-48, moved here by the decider, 2026-10-25 | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) §P2 `codelaw-migrate` row · `archive/open/W-48-codelaw-migrate.md` | the decider reopens it as a `W-nn` |
| B-056 | The CSS and SCSS forms the grammar forks still read as `ERROR` (the file's sheet then comes from the text reader): `[type="a" i]`, SCSS `@container #{$name} (…)`, a string holding an interpolation with its own quotes, `:not(.a)#{$x}`, an escape after an interpolation, `@page wide:left` in the SCSS fork, a list after `:nth-child(… of`; CSS an unquoted `url(data:…;…)` holding `;`, `@layer a, b.c`, `@nest`, `@value`, a backslash-newline in a string | [SR-ARCHITECTURE](../records/0108_architecture.md) decisions 3 and 15b · [W-62](../archive/open/W-62-scss-support.md) *Left* 2 | each form parses without an error in its fork's corpus test (`src/` regenerated with D-36), the forks' READMEs updated |
| B-057 | The 15 `scss/*` rules are all `preview`: the promotion criterion is ratified, but the `scss` corpus has two repos (Bootstrap, djangoproject.com) where it asks three | [SR-RULE-AUTHORING](../records/0113_rule-authoring.md) decision 6a · [W-62](../archive/open/W-62-scss-support.md) *Left* 3 · [W-100](../archive/open/W-100-rule-promotion.md) | a third SCSS repo pinned in `scripts/corpus.json`, then `corpus.py promote --pack scss --write --run <CI run>` |
| B-058 | No pinned corpus for CSS Modules (a React repo that uses them) and no `css-modules` `corpus.toml` counts | [W-63](../archive/open/W-63-css-modules.md) *Left* · [W-62](../archive/open/W-62-scss-support.md) *Left* 3 | the repo in `scripts/corpus.json` and a count for every `css-modules/*` rule |
| B-059 | The project graph does not read `.scss`: `graph/no-unused-css-classes` and `css/project` stay CSS-only | [SR-PROJECT-GRAPH](../records/0103_project-graph.md) (measurement: "SCSS is not read") · [W-62](../archive/open/W-62-scss-support.md) *Left* 4 | SCSS stylesheets give graph facts, with a fixture and the corpus counts |
| B-061 | The live Bitbucket proof of `--format bitbucket` and USAGE's step (W-109's DoD; 0.2.0 has no `--format bitbucket`), the decider's: bitbucket.org → *Create repository* `codelaw-pipeline-proof` (private) → *Repository settings → Pipelines → Settings → Enable Pipelines*; then `mkdir /tmp/bb-proof && cd /tmp/bb-proof && git init -b main && printf 'export function run() {\n  debugger;\n}\n' > app.js && npm init -y >/dev/null && npx -y codelaw@0.3.0 init` (it writes `codelaw.toml` and installs the JS pack), paste the YAML block from USAGE.md §Bitbucket Pipelines into `bitbucket-pipelines.yml`, then `git add -A && git commit -m proof && git remote add origin git@bitbucket.org:<workspace>/codelaw-pipeline-proof.git && git push -u origin main && git switch -c pr && git commit --allow-empty -m pr && git push -u origin pr` and open a pull request from `pr`. Expected: the build fails, the PR's *Reports* shows `codelaw` with one annotation on `app.js` line 2; set the repository variable `CODELAW_SARIF=true` and re-run → `codelaw.sarif` under *Artifacts*; set `CODELAW_CODE_INSIGHTS=false` and re-run → still fails, no new report. Open question: **the two switches stay the step's variables** (`CODELAW_SARIF`, `CODELAW_CODE_INSIGHTS`, a repository variable overriding each) — the agent's reading of *"just have serif and code [insights] configurable"*; the decider says if it needs a codelaw-side switch instead. Closed by the decider, 2026-10-28: *"close where only you're left"* | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) decisions 4 and 12b · [W-109](../archive/open/W-109-bitbucket-pipelines.md) *Left* 1–2 | reopen **after the v0.3.0 release**: the pipeline run (its URL to the agent) passes every expected line above, and the switches' reading confirmed or changed |

---

## `ungated` — a stated rule nothing mechanically checks

| id | what is outstanding | named in | what closes it |
|---|---|---|---|

---

## `unmeasured` — a live default, constant or claim with no measurement behind it

| id | what is outstanding | named in | what closes it |
|---|---|---|---|
| B-062 | The shared `perf` workload does not load the spell pack: the pack is measured as its own `spell` operation (Arpit, 2026-10-29: *"We can, but add an item in the backlog so that we can have a shared benchmark."*) | [SR-SPELL](../records/0116_spell.md) decision 9 · W-103 | the spell pack in the shared workload, every set budget re-measured and re-ruled by L8's method |
| B-063 | W-102's guidance-vs-check experiment, redesigned: baseline arm 0, screened temptations, arm D in the run, a confidence bound as the README rule | [proposal](proposals/guidance-experiment-redesign.md) (W-118, 2026-10-29) | the decider rules R1–R3 in the proposal |

---
| B-053 | The scheduled auto-fix PR (`codelaw init --ci github --autofix daily` or `weekly`, GitLab twin) has never run for real: one PR, updated by a second run, and that PR's own CI green are unproven (W-51 DoD 1, 3) | [W-51](../archive/open/W-51-scheduled-autofix-pr.md) (parked by Arpit, 2026-10-27: *"Put this in the backlog for now."*) · [SR-CLI-MCP](../records/0115_cli-and-mcp.md) | one run in a throwaway GitHub repo with fixable findings and a `CODELAW_AUTOFIX_TOKEN` secret |

## `unruled` — a fork named and not decided

| id | what is outstanding | named in | what closes it |
|---|---|---|---|
| B-051 | `deps refresh` and `data/outdated-dependency` do not read `requirements*.txt`: it is not one of L3's nine languages, so no rule can report on its lines | [L3](../records/0005_LAW-3-languages.md) · `archive/open/W-52-dependency-latest-versions.md` §As built | the decider amends L3, or rules that refresh reads it with findings elsewhere |
| B-054 | Reading a project's `cspell.json` (`words`, `ignoreWords`, `flagWords`) into `[spell] words`, through `spell/suspicious-word`, so a CSpell user switches in one step | [SR-SPELL](../records/0116_spell.md) Consequences | the decider rules it into W-94's `codelaw migrate` (a `cspell` source) or out |
| B-055 | The spell pack checks English only; another language needs a word list under a permissive licence and its own L11 row (many CSpell dictionaries are LGPL/GPL) | [SR-SPELL](../records/0116_spell.md) decision 10 | the decider names a language and ratifies a source for it, or rules English-only permanent |

---

## `cost` — a consequence accepted deliberately

| id | what is outstanding | named in | what closes it |
|---|---|---|---|
| B-006 | The guard proves a record was touched, never that it is coherent; L0 obligation (iii) is judgment | [SR-LAW-0](../records/0002_LAW-0-authority.md) decision 1a | never graduates; it is the stated price of a mechanical gate |
| B-007 | The `Stop` hook relents after three refusals, so a blocker can be walked past | [SR-WORK-BLOCKERS](../records/0056_WORK-blockers.md) decision 8 | never graduates; a hook that never relents is a hang |
| B-045 | Biome's CSS formatter rewrites `@media not(` to `@media not (`, so `codelaw format` turns an invalid query valid — a meaning change; lint runs first, so `css/media-query-no-invalid` still reports it | [SR-FORMATTER](../records/0105_formatter.md) decision 3 (measured, 2026-10-22; re-judged by tokens, 2026-10-24, `W-28`: codelaw's CSS grammar parses `not(` and `not (` alike, so by codelaw's tokens the rewrite is layout — CSS Syntax says otherwise; the rule's `not(` stays a case, outside the layout check, and the advisory formatter check names B-045 wherever tokens differ but the formatted texts agree) | the CSS engine leaves an invalid prelude byte-identical, or the decider accepts the repair as the formatter's |
| B-060 | `scss/operator-no-unspaced` (stylelint-scss) is not shipped: it is a layout rule, which the formatter owns | [L7](../records/0009_LAW-7-linter-and-formatter.md) decision 1 · [SR-FORMATTER](../records/0105_formatter.md) decision 3 · [W-62](../archive/open/W-62-scss-support.md) *Left* 5 | never graduates while layout belongs to the formatter |
