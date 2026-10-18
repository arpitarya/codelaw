---
type: Log
description: "Append-only per-session trail, date-grouped, newest first."
---

# WORKLOG — the append-only session log

Every session appends **one entry before it ends** — a chat-only session
counts. **Never edit a past entry; append only, newest on top.** A wrong old
entry is corrected by a new one. Distinct from `INTERVIEW.md`, which is the
*current* state of play. Rules: [SR-WORK-SESSION](../records/0054_WORK-session.md).

**Entry format:**

```
## YYYY-MM-DD — <one-line title>  ·  <surface> (<model>)

- **Asked:** *<the ask>*
- **Done:** <what landed, with commit shas where they exist>
- **Decided / left open:** <rulings taken in-session; forks put to the decider>
- **Mistakes:** <what went wrong and what was done about it, or "none">
- **Next:** <the single next step>
```

---

## 2026-10-18 — law L2 filed  ·  Claude Code (Opus 5.5)

- **Asked:** *"add a new law to not use paid and open source dependencies."*
- **Done:** SR-LAW-2 (`records/0004_LAW-2-foss-dependencies.md`); routed in SR-LAWS, `CLAUDE.md` §The laws and the register; backlog row B-008 for the licence check nobody runs yet. `CLAUDE.md` changed: one pointer line for L2.
- **Decided / left open:** the ask read two ways, so it went to the decider, who ruled: **FOSS only, nothing paid**, and **no copyleft anywhere**, dev tooling included. Weak copyleft (MPL, EPL) and paid hosted services are refused too — that reading is this session's, stated in SR-LAW-2 decisions 2 and 5 for the decider to correct.
- **Mistakes:** none.
- **Next:** the decider installs the commit-msg guard; B-008 becomes a `W-nn` when the first manifest lands.

---

## 2026-10-18 — the work system set up from WORK-SYSTEM.md  ·  Claude Code

- **Asked:** *set up the work system from WORK-SYSTEM.md.*
- **Done:** records 0001–0003 and 0051–0060, `work/`, `archive/`, `scripts/`, `.claude/` hooks, `CLAUDE.md`; WORK-SYSTEM.md moved to `archive/bootstrap/`.
- **Decided / left open:** tests deferred — filed as backlog rows B-001–B-005.
- **Mistakes:** none.
- **Next:** the decider installs the commit-msg guard; the first `W-nn` is filed.
