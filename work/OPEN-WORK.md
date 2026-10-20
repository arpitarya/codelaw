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
| Place the five `.github/` issue and PR templates — the path is protected from remote writes ([W-04](open/W-04-consumer-readme.md)) | 2026-10-20 | 0 d |
| ↳ **blocks:** W-04 | | |
| Give the go to publish the packs with the engine — the licence is ruled, MIT ([W-03](open/W-03-rule-packs.md)) | 2026-10-20 | 0 d |
| ↳ **blocks:** W-03 | | |

---

## Open items

### build

🔴 **W-03** `agent` The five rule packs: built, tested, packaged and MIT-licensed 2026-10-20; blocked on the decider's go to publish, and needs the engine's packages (B-038). Filed by Arpit (Cowork). [detail](open/W-03-rule-packs.md)

🔴 **W-04** `agent` README, CHANGELOG, CONTRIBUTING, SECURITY and CODE_OF_CONDUCT landed; blocked on the decider placing the `.github/` templates. Filed by Arpit (Cowork). [detail](open/W-04-consumer-readme.md)

### testing

### record update

### research

🟢 **W-05** `cowork` Research guiding an agent **while** it writes: a `brief` before writing (rules filtered by path and construct) and a `hook --pre` check before a write reaches disk; Fux as an optional free-text layer. Filed by Arpit (Cowork). [detail](open/W-05-write-time-guidance.md)

---
