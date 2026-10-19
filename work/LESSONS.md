---
type: Log
description: "Dated lessons — what this project learned the hard way, newest first. A log, not a record: the date is part of the lesson."
---

# LESSONS — what this build learned the hard way

Newest first. A lesson whose remedy is a *rule* belongs in a record and is
named here only as the failure that produced it. A lesson whose remedy is
*judgment* lives here. ⚠ Nothing here is normative.

---

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
