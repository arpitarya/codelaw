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

---

## `ungated` — a stated rule nothing mechanically checks

| id | what is outstanding | named in | what closes it |
|---|---|---|---|

---

## `unmeasured` — a live default, constant or claim with no measurement behind it

| id | what is outstanding | named in | what closes it |
|---|---|---|---|

---

## `unruled` — a fork named and not decided

| id | what is outstanding | named in | what closes it |
|---|---|---|---|
| B-047 | move the 70 CSS rules off `css_syntax.rs` onto a real tree: (a) extend the tree-sitter-css fork for the forms that still give `ERROR` or misparse (`--x: { … }`), then port; or (b) run the native rules on Biome's CSS tree, already in the binary for the formatter. Not decided; W-21's DoD 3 was restated without it (Arpit, 2026-10-24) | [SR-ARCHITECTURE](../records/0108_architecture.md) decision 15b · `archive/open/W-21-pack-engine-gaps.md` §DoD 3, settled | the decider picks (a), (b) or neither, and an item is filed |

---

## `cost` — a consequence accepted deliberately

| id | what is outstanding | named in | what closes it |
|---|---|---|---|
| B-006 | The guard proves a record was touched, never that it is coherent; L0 obligation (iii) is judgment | [SR-LAW-0](../records/0002_LAW-0-authority.md) decision 1a | never graduates; it is the stated price of a mechanical gate |
| B-007 | The `Stop` hook relents after three refusals, so a blocker can be walked past | [SR-WORK-BLOCKERS](../records/0056_WORK-blockers.md) decision 8 | never graduates; a hook that never relents is a hang |
| B-045 | Biome's CSS formatter rewrites `@media not(` to `@media not (`, so `codelaw format` turns an invalid query valid — a meaning change; lint runs first, so `css/media-query-no-invalid` still reports it | [SR-FORMATTER](../records/0105_formatter.md) decision 3 (measured, 2026-10-22; re-judged by tokens, 2026-10-24, `W-28`: codelaw's CSS grammar parses `not(` and `not (` alike, so by codelaw's tokens the rewrite is layout — CSS Syntax says otherwise; the rule's `not(` stays a case, outside the layout check, and the advisory formatter check names B-045 wherever tokens differ but the formatted texts agree) | the CSS engine leaves an invalid prelude byte-identical, or the decider accepts the repair as the formatter's |
