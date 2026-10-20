---
type: Glossary
description: "Recurring terms, defined once."
---

# Glossary

| term | meaning |
|---|---|
| Standing Record (SR) | a maintained specification in `records/`, cited by name; see the register |
| Law | a rule that outranks every other record; `SR-LAW-n` |
| W-nn | an open work item — a queue row plus a detail file |
| B-nnn | a backlog row — named, unclaimed, no file |
| ball | the one marker every queue row opens with: 🔴 🟣 🟡 🟢 |
| blocker | `work/BLOCKED.json` with a decision other than `PROCEED` |
| veto condition | the checkable condition under which a decision reopens |
| fix class | what a rule's fix is: `auto` (applied), `suggest` (a diff to accept) or `manual` (written steps); SR-LAW-4 |
| preset | a named group of rules inside a rule pack, e.g. `codelaw:js/recommended`; SR-RULE-PACKS |
| pack | a rule pack: a folder of rule files, presets and data tables with a `codelaw-pack.toml` manifest, on npm, PyPI or a path; codelaw's own five are `@codelaw/rules-*` / `codelaw-rules-*`; SR-RULE-PACKS |
| project graph | the whole-repo map of modules, exports, imports and references that cross-file rules query; SR-PROJECT-GRAPH |
| baseline | the committed per-file, per-rule counts of violations allowed to remain, with a deadline and a touch policy; SR-BASELINE |
| as-of date | the date deadline rules evaluate against — the `HEAD` commit date unless `--as-of` says otherwise; SR-BASELINE |
| layout rule | a lint rule whose fix only changes formatting; refused — layout belongs to the formatter (SR-FORMATTER) |
| embedded region | code of one language inside a file of another — HTML `<script>`/`<style>`, Markdown fences — linted and formatted as its own language (SR-LINTER) |
| budget | an L8 time limit for one operation, set from measurement, never exceeded or loosened |
| reference workload | the seeded 10,000-file repo plus pinned real repos that L8 budgets are measured on |
| warm process | a long-lived `codelaw lsp`, `mcp` or `daemon` that keeps trees, rules and the graph in memory (SR-ARCHITECTURE) |
| project root | the topmost config, or a nested config with `root = true`; one graph and one baseline per project root (SR-ARCHITECTURE) |
| warm socket | the `.codelaw/cache/run/<pid>.sock` a running lsp, mcp or daemon registers so one-shot runs can reuse it (SR-ARCHITECTURE) |
| semantic model | the per-file map of scopes, bindings and references that rules like `no-unused-vars` query (SR-ARCHITECTURE decision 15a) |
| native rule | a rule whose matcher is implemented in the engine in Rust; its Markdown file still carries docs, fix class and examples (SR-ARCHITECTURE decision 15b) |
| change set | the files counted as touched: from `--changed-since`, `--staged` or the agent hook, never inferred; SR-BASELINE |
| predicate | a semantic-model condition on a capture, written under `where:` in a `match:` rule — `unused`, `undeclared`, `shadowed`, `reassigned`, `imported-from`; SR-RULE-FORMAT decision 3a |
| data table | a JSON file in a pack's `data/` (CSS properties, HTML elements, globals), read once and shared by the rules that name it; SR-RULE-PACKS decision 3 |
| fixpoint | a file the fixes no longer change: the apply loop runs until it reaches one, at most ten passes; SR-ARCHITECTURE decision 5 |
| group fix | a fix computed once over every match in a file (`collect-into-block`); each diagnostic carries the whole edit set; SR-RULE-FORMAT decision 3c |
| first-party pack | one of codelaw's five packs, recognised by its manifest name `@codelaw/rules-*`; only these may use `native:`; SR-RULE-PACKS decision 4 |
| trusted publishing | a registry (npm, PyPI) accepting an upload because GitHub's OIDC token proves it came from one named workflow and environment — no stored token; SR-PACKAGES decision 11 |
| `dist` | the checked set of 14 npm tarballs and 13 wheels one `ci.yml` run assembles; `publish.yml` uploads exactly that set and never builds its own; SR-PACKAGES decision 11 |
| half-published | a release some of whose packages reached a registry before a publish failed; withdrawn by the decider with `scripts/withdraw.sh`; SR-PACKAGES decision 12 |
