---
type: Queue
description: "The single live work queue, three lanes, plus the Blocked-on-the-decider inbox. The list only — its rules live in the record below."
governed_by: SR-WORK-OPEN-QUEUE
governed_by_path: records/0051_WORK-open-queue.md
---

# OPEN-WORK — what is still open

**This file is the LIST, plus the legend below.** Everything else — what the
queue is, an item's lifecycle, the shape of a row, ball precedence, ordering,
how the inbox works, and what every session owes — is stated once in
**[SR-WORK-OPEN-QUEUE](../records/0051_WORK-open-queue.md)** and is not repeated
here. Read that record before changing anything below it.

**A ball on every row, always one:** 🔴 blocked on the decider, directly or through another item · 🟣 gated on a named date, directly or through another item · 🟡 waiting on another item · 🟢 no blockers.
**Optional, after the ball:** 🧨 broken or getting worse · 🔺 do first (the decider only). Full legend: [SR-WORK-OPEN-QUEUE](../records/0051_WORK-open-queue.md) rules 20–34.

---

## Blocked on the decider

| what the decider decides | filed | age |
|---|---|---|

*Empty since 2026-10-19 — every row ruled ("go with the recommendations"); the next gate is the first ruling W-02 raises.*

---

## Open items

### build

🟢 **W-02** `agent` Build the engine the rule packs run on — P0 crates, semantic model, native rules, pack loading. Filed by Arpit (Cowork). [detail](open/W-02-engine-p0.md)
🟡 **W-03** `agent` Build the five rule packs (js, html, css, python, data) from the researched ESLint/Ruff/stylelint/html-validate presets; waiting on W-02. Filed by Arpit (Cowork). [detail](open/W-03-rule-packs.md)

### testing

### record update

### research

---
