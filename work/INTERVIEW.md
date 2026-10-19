---
type: Handoff
description: "Cold-start state of play for a successor session: in-flight work, constraints, lessons."
---

# Model handoff — the agent-succession record

*The departing model answers for the record, so whatever succeeds it inherits
the judgment and not just the files.*

**Standing instruction to every future model reading this: you are now the
maintainer of this document. When the project's direction, decisions or state
change, update it in the same change. Leave it better.**

## How to use this file

A state-of-play doc, not an end-of-session report. Keep it current **during**
the session. Write it for a different model arriving with zero other context,
mid-task. The four sections below are maintained; anything after them is
history. **A stale INTERVIEW at handoff is as serious as a missing worklog
entry.**

---

## 1 · State of play

*Updated 2026-10-19.* The work system is set up; laws L0–L8 are accepted (L8's
budgets unset; L6 host floors Python ≥ 3.11, Node ≥ 22). The design is `docs/DESIGN.md`;
every record 0101–0115 is accepted except SR-RULE-PACKS, which W-01 rewrites for
separately published packs and then accepts (ruled 2026-10-19).
SR-ARCHITECTURE is the design of record (rewritten 2026-10-19 to close the
cross-cutting gaps: I/O, git, contracts, warm processes, failures). No code exists yet.
**Ground it before you edit it** — `git log`, the queue, the code.

## 2 · In flight, and the immediate next step

**W-01** (🟢, records for five separately published rule packs) is in flight;
**W-02** (engine, 🟡) waits on W-01 and **W-03** (the packs, with the preset research,
🟡) waits on W-02. The inbox is empty since 2026-10-19 (every row ruled). When the first manifest lands, promote
B-008 and B-012 to `W-nn`s in the same change. Next: the decider (`git config user.name`) installs `scripts/sr-guard.sh` as the
`commit-msg` hook; the first item is filed from the first real ask.

## 3 · Standing constraints

- The laws: L0 (records are the only source; links, never restatements), L1
  (retired records are archived), L2 (every dependency is free, open source and
  permissive — no paid, no copyleft, dev tooling included), L3 (nine
  languages — CSS and HTML added — alone or combined), L4 (every rule has a fix), L5 (no model in the
  engine), L6 (`codelaw` on PyPI + npm, identical; npm sub-packages only `@codelaw/*`), L7 (linter and formatter,
  always both), L8 (performance budgets: never exceeded, never loosened). See `records/0001_LAWS.md`.
- Adapters must pass L2 too: yamllint (GPL-3.0) is out.
- The queue is the list; a blocker is a file; nothing closed is deleted.
- Tests are not yet in place (`BACKLOG.md` B-001–B-005, B-008); the hooks and the
  commit guard are the only mechanical enforcement.

## 4 · Lessons learned

- None yet. The first lesson goes here and in `LESSONS.md`.
