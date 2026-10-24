---
type: Standing Record
kind: process
name: SR-WORK-REGISTRY
title: "SR-WORK-REGISTRY (0060) — the doc registry"
description: "Every maintained document has a row in §3 of this record naming its update trigger and last-verified date; a task that fires a trigger bumps the row in the same change."
status: accepted
date: 2026-10-18
feature: the doc registry
owns: []
laws: [L0]
timestamp: 2026-10-18T12:52:51Z
content_sha: ""
---

# SR-WORK-REGISTRY — the doc registry

## §1 — For humans

**Every maintained document has a row in §3 below**: what fires an update to
it, and when it was last verified. A task that fires a trigger updates the
document and bumps its row in the same change.

**The registry holds live documents only.** An archived document loses its row,
and a row older than about thirty days is a prompt to review, not a failure.

```mermaid
flowchart LR
    T["a task fires a trigger"] --> D["the document is updated"]
    D --> R["its §3 row's Last verified is bumped — same change"]
```

<details>
<summary><b>ASCII twin</b> — the same diagram, for terminals and diffs</summary>

```text
   a task fires a trigger --> the document is updated --> its §3 row's Last verified bumped (same change)
```

</details>

---

## §2 — For agents

### Context

Documents rot silently. A trigger column turns *remember to update the docs*
into a scan anyone can do at task end.

### Decision — the registry discipline (normative)

1. The registry is §3 of this record — a table, no other file.
2. Every maintained document has an **update trigger** and a **last verified**
   date; a task that fires a trigger updates the document *and* bumps its row
   in the same change.
3. A row is `| Document | Update trigger | Last verified | Notes |`.
4. Live documents only; an archived document loses its row in the same change.
5. No row points into `archive/`. 6. Every row's target exists. 7. One row
   per document. 8. A new maintained document gets its row in the same change.
9. A row older than about thirty days is a review prompt, not a failure.
10. At task end, scan the trigger column.

### Consequences

- Decisions 5–8 are checked in CI (job `lint`) by
  `scripts/check-work-system.py`. Decision 8 reads "maintained" as: every
  Markdown file at the repo root, at the top of `docs/` and of `work/`, each
  `work/*/README.md`, and `records/README.md` and `records/TEMPLATE.md`; a
  record, an item file and an archived file are not rows.
- Every other decision is judgment; decision 10's scan is the session's.

### Alternatives considered

- A separate registry file — rejected: decision 1 keeps the table inside this
  record, so the rules and the table move together.

### Reference (required)

- §3 below — the table itself.
- [SR-WORK-DOCS](0055_WORK-docs.md) decision 10 — step 5 is the bump.

### Veto condition

**Reopen this decision if:** a registry row names a document that does not exist.

**How to check it:** `python3 scripts/check-work-system.py | grep registry:` must print nothing.

---

## §3 — The registry

| Document | Update trigger | Last verified | Notes |
|---|---|---|---|
| `CLAUDE.md` | any rule or pointer changes | 2026-10-24 | 2026-10-24 — the L6 line and the `packs/` layout line: nine packs, eight on npm and six on PyPI (W-31, the decider's ruling) |
| `work/README.md` | a file is added to or removed from `work/` | 2026-10-18 | 2026-10-18 — created at setup |
| `work/OPEN-WORK.md` | the same change as the work it tracks | 2026-10-24 | 2026-10-24 — W-31 and W-32 filed, W-32's ruling in the inbox; W-29 re-balled (0.1.0 on npm); W-30 moved to 0.2.0 |
| `work/BACKLOG.md` | a record names something nobody is doing | 2026-10-20 | 2026-10-20 — every row but B-006, B-007 promoted to W-07–W-22 |
| `work/open/README.md` | the item-file contract changes | 2026-10-18 | 2026-10-18 — created at setup |
| `work/INTERVIEW.md` | direction, a decision or the live state changes — during the session | 2026-10-24 | 2026-10-24 — §1: nine packs, 0.1.0 on npm, W-31 uncommitted |
| `work/IMPLEMENTATION.md` | something lands | 2026-10-24 | 2026-10-24 — W-29 (0.1.0 on npm) and W-31 (the registry pack split) |
| `work/WORKLOG.md` | every session, before it ends — chat-only ones too | 2026-10-24 | 2026-10-24 — the first npm publish and W-31 |
| `work/NOW.md` | every transition; `idle` when stopping | 2026-10-24 | 2026-10-24 — after W-31: unpublish `@codelaw/rules-python`, push, 0.2.0 |
| `work/LESSONS.md` | a failure teaches something durable | 2026-10-24 | 2026-10-24 — the cfg-split lesson and the two-sessions-one-checkout lesson |
| `work/MACHINE.md` | a surface breaks in a way the code did not cause | 2026-10-20 | 2026-10-20 — Windows runner cp1252; the cloud session's blocked Actions logs |
| `work/compare/README.md` | a fork opens, closes, or its trigger fires | 2026-10-18 | 2026-10-18 — created at setup |
| `work/proposals/README.md` | an idea is filed, graduates, or is rejected | 2026-10-18 | 2026-10-18 — created at setup |
| `docs/GLOSSARY.md` | a term is coined or redefined | 2026-10-24 | 2026-10-24 — pack, first-party pack and `dist` for the nine packs (W-31) |
| `docs/DESIGN.md` | the design changes — after the owning record | 2026-10-24 | 2026-10-24 — the L6 and SR-RULE-PACKS rows: nine packs (W-31) |
| `records/README.md` | a record's state, ownership or number changes | 2026-10-24 | 2026-10-24 — the L6 and SR-PACKAGES rows and the `packs` row: nine packs (W-31) |
| `RELEASING.md` | the release process, the one-time setup or a workflow's name changes | 2026-10-24 | 2026-10-24 — the names per registry; npm's one-time steps done (W-29) and owed again for the four new pack names (W-31) |
| `records/TEMPLATE.md` | the convention changes | 2026-10-18 | 2026-10-18 — created at setup |
| `README.md` | a command, flag, preset or pack's rule count changes; something on the roadmap lands; the licence is ruled | 2026-10-24 | 2026-10-24 — the rule-pack table per registry, the config example (W-31) |
| `CHANGELOG.md` | a user-visible change lands; a release is cut | 2026-10-24 | 2026-10-24 — Unreleased: the nine packs and the preset changes (W-31) |
| `CONTRIBUTING.md` | the build, test or contribution steps change | 2026-10-20 | 2026-10-20 — contributions are MIT |
| `SECURITY.md` | the reporting channel or the supported versions change | 2026-10-20 | 2026-10-20 — created (W-06, filed as W-04) |
| `CODE_OF_CONDUCT.md` | the code or its contact changes | 2026-10-20 | 2026-10-20 — created (W-06, filed as W-04) |
| `LICENSE` | the licence or the copyright holder changes (SR-PACKAGES decision 10); every `packs/*/LICENSE` copy changes with it | 2026-10-20 | 2026-10-20 — created: MIT |

---

## References

*Every source this record cites, gathered in one place. An archived document is never listed here.*

**Records** — [SR-WORK-DOCS](0055_WORK-docs.md) · [SR-WORK-GOVERNANCE](0059_WORK-governance.md)

**Project docs** — every document §3 lists.
