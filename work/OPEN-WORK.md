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

| what the decider decides | filed |
|---|---|
| 🔴 **The one-time registry setup, step 4** — the seven PyPI pending publishers ([`RELEASING.md`](../RELEASING.md) §One time); needs his PyPI login. Steps 1–3 done for npm (W-29) and step 5, the `npm`/`pypi` environments with tag rule `v*`, done 2026-10-24 | 2026-10-20 |
| ↳ **blocks:** W-04 step 5 (the first release on both registries) · W-03 step 6 · W-30 step 4 | |
| 🔴 Set L8's budgets: `perf` has run on `ubuntu-24.04` (CI runs 40124005285, 40124756694, 40124878075); rule each `budget` in `perf/budgets.toml` from the re-applied proposals ([W-19](open/W-19-performance-budgets.md)) | 2026-10-22 |
| ↳ **blocks:** W-19 DoD 3 · the perf gate binding anything | |

---

## Open items

### build

🔴 **W-04** `agent` The release pipeline, steps 1–4 done 2026-10-20. Step 5, the first release: blocked on the registry setup above; the engine is complete since 2026-10-21 (L7, L6 decision 3). Filed by Arpit (Claude Code). [detail](open/W-04-release-pipeline.md)

🔴 **W-03** `agent` The rule packs: steps 2–5 done (419 rules), packaging done; nine packs since W-31. Step 6, publishing: blocked on the registry setup above. Filed by Arpit (Cowork). [detail](open/W-03-rule-packs.md)

🟢 **W-29** `arpit` The first npm publish: 14 names live at 0.1.0, trusted publishing bound (2026-10-24), from the `dist` of a failed CI run; the Mac install check passed 2026-10-25; left: the 8-platform registry install and the records (DoD 2, 4). Filed by Arpit (Claude Code). [detail](open/W-29-first-npm-publish.md)

🟡 **W-30** `arpit` The first PyPI publish: `codelaw` (8 wheels) and the six `codelaw-rules-*` of W-31's layout, at 0.2.0 rather than 0.1.0; waiting on W-31 (0.2.0). Filed by Arpit (Claude Code). [detail](open/W-30-first-pypi-publish.md)

🟢 **W-31** `arpit` Eight packs on npm, six on PyPI, `rules-js` split in five: landed 2026-10-24, and `@codelaw/rules-python` is off npm. Left: commit and push, then 0.2.0. Filed by Arpit (Claude Code). [detail](open/W-31-registry-pack-split.md)

🟡 **W-32** `arpit` A Redux Toolkit pack and a TanStack Query pack, npm only: steps 2–4 done in the tree 2026-10-25 (7 + 12 rules, all green, wired); left: Arpit pushes the W-32 commit, then their first publish at 0.2.0 with W-31 step 7. Filed by Arpit (Claude Code). [detail](open/W-32-redux-and-query-packs.md)

🟢🧨 **W-24** `arpit` CI green on `main`: DoD 5 proven (run 40125039176 skipped `perf` on a docs-only push), but `main` is red since that run — `lint`'s docs-sync (CHANGELOG missed W-18, W-19); fixed 2026-10-25 in the W-32 commit; left: Arpit pushes it. Filed by Arpit (Claude Code). [detail](open/W-24-ci-green-on-main.md)

### testing

🔴 **W-19** `agent` `bench` times the three pinned repos and `perf` runs on `ubuntu-24.04` (2026-10-25); CI's numbers are in, three p95s past their Mac proposals; blocked on the L8 budgets ruling above. Filed by Arpit (Cowork). [detail](open/W-19-performance-budgets.md)

### record update

### research

🟢 **W-05** `cowork` Research guiding an agent **while** it writes: a `brief` before writing (rules filtered by path and construct) and a `hook --pre` check before a write reaches disk; Fux as an optional free-text layer. Filed by Arpit (Cowork). [detail](open/W-05-write-time-guidance.md)

---
