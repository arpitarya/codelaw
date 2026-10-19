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
| preset | a named group of built-in rules, e.g. `codelaw:recommended`; SR-RULE-PACKS |
| pack | a folder of rule files published outside codelaw (npm, PyPI or a path); SR-RULE-PACKS |
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
| semantic model | the per-file map of scopes, bindings and references that rules like `no-unused-vars` query (W-02) |
| native rule | a rule whose matcher is implemented in the engine in Rust; its Markdown file still carries docs, fix class and examples (W-01) |
| change set | the files counted as touched: from `--changed-since`, `--staged` or the agent hook, never inferred; SR-BASELINE |
