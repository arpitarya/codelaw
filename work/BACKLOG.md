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
| B-043 | "Diagnostics: pull and push" and "Workspace diagnostics: for project rules" — the server pushes only, per open document | [SR-EDITOR](../records/0114_editor.md) decision 1 | `textDocument/diagnostic` and `workspace/diagnostic` served, with a server test; project rules wait on W-11 |
| B-044 | the "Explain rule" and "Ignore here with reason" code actions | [SR-EDITOR](../records/0114_editor.md) §1 table, Quick fixes row | both offered as code actions, tested, writing nothing |
| B-046 | the tree-sitter-css upstream PR: `grammar.patch`, a ready commit with `src/` regenerated (`0001-tree-sitter-css-modern.patch`) and `PR.md` in `crates/codelaw-lang/grammars/tree-sitter-css/upstream/`; not filed — **Arpit, 2026-10-24:** *"Then let's keep using the patch copy and put the PR in backlog."* | [SR-ARCHITECTURE](../records/0108_architecture.md) decision 3 · [W-21](../archive/open/W-21-pack-engine-gaps.md) | the PR is opened on tree-sitter/tree-sitter-css; the fork is dropped when a release carries it |
| B-048 | The *later* presets: `js/strict` (typescript-eslint `strict`, jsx-a11y, unicorn), `react/a11y`, `py/strict`, and an opt-in pydocstyle `D` preset | [SR-RULE-PACKS](../records/0101_rule-packs.md) §1; the research table of [W-03](../archive/open/W-03-rule-packs.md) | each preset ships in its pack, or the decider rules it out |
| B-050 | `codelaw migrate --from eslint\|ruff\|stylelint\|biome` (no oxlint): "Maps existing ESLint, typescript-eslint, Ruff, stylelint … configs rule by rule through `ported_from:`" and writes "a mapping report"; was W-48, moved here by the decider, 2026-10-25 | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) §P2 `codelaw-migrate` row · `archive/open/W-48-codelaw-migrate.md` | the decider reopens it as a `W-nn` |
| B-052 | `rules_for_path` on a source file names the major-version gaps of the packages it imports, from the deps snapshot; in W-52's Goal, not its DoD, and not built | `archive/open/W-52-dependency-latest-versions.md` §Goal, Agents · [SR-RULE-PACKS](../records/0101_rule-packs.md) decision 15 | built and tested in the MCP server |

---

## `ungated` — a stated rule nothing mechanically checks

| id | what is outstanding | named in | what closes it |
|---|---|---|---|

---

## `unmeasured` — a live default, constant or claim with no measurement behind it

| id | what is outstanding | named in | what closes it |
|---|---|---|---|

---
| B-053 | The scheduled auto-fix PR (`codelaw init --ci github --autofix daily` or `weekly`, GitLab twin) has never run for real: one PR, updated by a second run, and that PR's own CI green are unproven (W-51 DoD 1, 3) | [W-51](../archive/open/W-51-scheduled-autofix-pr.md) (parked by Arpit, 2026-10-27: *"Put this in the backlog for now."*) · [SR-CLI-MCP](../records/0115_cli-and-mcp.md) | one run in a throwaway GitHub repo with fixable findings and a `CODELAW_AUTOFIX_TOKEN` secret |

## `unruled` — a fork named and not decided

| id | what is outstanding | named in | what closes it |
|---|---|---|---|
| B-049 | type-aware rules (typescript-eslint's `*-type-checked` set) and TypeScript's own errors with its fixes, answered by a typescript-go helper; needs L3 and L6 amended. Parked by the decider, 2026-10-25: *"For now, put type aware rules and details in backlog"* | [proposal](proposals/type-aware-rules.md) | the decider reopens it, or TypeScript 7.1 ships its programmatic API |
| B-051 | `deps refresh` and `data/outdated-dependency` do not read `requirements*.txt`: it is not one of L3's nine languages, so no rule can report on its lines | [L3](../records/0005_LAW-3-languages.md) · `archive/open/W-52-dependency-latest-versions.md` §As built | the decider amends L3, or rules that refresh reads it with findings elsewhere |

---

## `cost` — a consequence accepted deliberately

| id | what is outstanding | named in | what closes it |
|---|---|---|---|
| B-006 | The guard proves a record was touched, never that it is coherent; L0 obligation (iii) is judgment | [SR-LAW-0](../records/0002_LAW-0-authority.md) decision 1a | never graduates; it is the stated price of a mechanical gate |
| B-007 | The `Stop` hook relents after three refusals, so a blocker can be walked past | [SR-WORK-BLOCKERS](../records/0056_WORK-blockers.md) decision 8 | never graduates; a hook that never relents is a hang |
| B-045 | Biome's CSS formatter rewrites `@media not(` to `@media not (`, so `codelaw format` turns an invalid query valid — a meaning change; lint runs first, so `css/media-query-no-invalid` still reports it | [SR-FORMATTER](../records/0105_formatter.md) decision 3 (measured, 2026-10-22; re-judged by tokens, 2026-10-24, `W-28`: codelaw's CSS grammar parses `not(` and `not (` alike, so by codelaw's tokens the rewrite is layout — CSS Syntax says otherwise; the rule's `not(` stays a case, outside the layout check, and the advisory formatter check names B-045 wherever tokens differ but the formatted texts agree) | the CSS engine leaves an invalid prelude byte-identical, or the decider accepts the repair as the formatter's |
