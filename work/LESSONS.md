---
type: Log
description: "Dated lessons — what this project learned the hard way, newest first. A log, not a record: the date is part of the lesson."
---

# LESSONS — what this build learned the hard way

Newest first. A lesson whose remedy is a *rule* belongs in a record and is
named here only as the failure that produced it. A lesson whose remedy is
*judgment* lives here. ⚠ Nothing here is normative.

---

## 2026-10-20 — a law with no gate gets walked past by the next release

L7 says the linter and formatter never ship apart, and L6 decision 3 says every
package carries the formatter, MCP server and LSP. The work was split sensibly —
lint first (W-02), packs (W-03), the pipeline (W-04) — and each item was green
on its own terms, while the formatter sat in the backlog as "L7 requires it".
Nothing mechanical stood between that and a publish, so the queue ended up
asking the decider to hand-publish 0.1.0. **Before a step that cannot be
undone (a publish), check every law against what will ship, and make the
check a gate** — `scripts/law-gate.py` is that gate for L6 and L7.

## 2026-10-20 — the slowest platform sets the pipeline's time; skip it, don't rush it

Eight platform builds run in parallel, so a push takes as long as Windows arm64
(~2½ min for one LTO build) whatever the other seven do. Faster runners were not
the lever: **not rebuilding was** — the wheel is cached under the hash of every
input that goes into it, so a push that changes no engine source (records,
docs, packs, `work/`) restores eight wheels instead of compiling them. The
binary's tests still run on every push; only the compile is skipped.

## 2026-10-19 — a rule's own examples don't find its false positives; a corpus does

All 58 first ports passed their examples, then a run over npm's source found
eight false-positive patterns (`this` inside an arrow passed to `super()`,
`[]` inside `[{[]`, ambient `declare` code, a tree-sitter misparse, `await`
read as an identifier). **Run a new rule over real code before trusting it**;
the examples only prove the cases the author thought of.

## 2026-10-19 — YAML keeps the last of two equal keys, silently

A rule written with two `has:` keys compiled with one of them gone, and its
broken half was never noticed until the duplicate was removed. Rule YAML now
refuses duplicate keys (SR-RULE-FORMAT decision 3d).

## 2026-10-19 — a benchmark that writes into the repo it checks measures itself

Timing `codelaw check > cold.json` inside the generated repo made the second
run check `cold.json` too, so "cold" and "warm" output differed and looked
like a cache bug. **Write a run's output outside the folder it checks** before
reading a difference as a finding.

## 2026-10-19 — rule YAML must be read as YAML 1.2

`serde-saphyr`'s default options read `message: n` as the boolean `false`
(YAML 1.1's "Norway problem"), so a one-letter message refused a rule. ast-grep
reads its rules as YAML 1.2; codelaw now does the same, and the rule is in
SR-RULE-FORMAT decision 3d. A test in `codelaw-rules` pins it.
