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
| 🔴 **PyPI, ruled 2026-10-25: upload the six `codelaw-rules-*` 0.2.0 wheels by hand, then add a trusted publisher to each** — `codelaw` comes from CI through its pending publisher (registered 2026-10-25); commands in [W-30](open/W-30-first-pypi-publish.md) | 2026-10-20 |
| ↳ **blocks:** W-04 step 5 (the first release on both registries) · W-03 step 6 · W-30 step 4 | |

---

## Open items

### build

🔴 **W-04** `agent` The release pipeline, steps 1–4 done 2026-10-20. Step 5, the first release: blocked on the registry setup above; the engine is complete since 2026-10-21 (L7, L6 decision 3). Filed by Arpit (Claude Code). [detail](open/W-04-release-pipeline.md)

🔴 **W-03** `agent` The rule packs: steps 2–5 done (419 rules), packaging done; nine packs since W-31. Step 6, publishing: blocked on the registry setup above. Filed by Arpit (Cowork). [detail](open/W-03-rule-packs.md)

🟡 **W-29** `arpit` The first npm publish: 14 names live at 0.1.0, trusted publishing bound, the Mac install check passed 2026-10-25, records done; left: the 8-platform registry install, moved to 0.2.0 (ruled 2026-10-25). Filed by Arpit (Claude Code). [detail](open/W-29-first-npm-publish.md)

🟡 **W-30** `arpit` The first PyPI publish: `codelaw` (8 wheels) and the six `codelaw-rules-*` of W-31's layout, at 0.2.0 rather than 0.1.0; waiting on W-31 (0.2.0). Filed by Arpit (Claude Code). [detail](open/W-30-first-pypi-publish.md)

🟢 **W-31** `arpit` Eight packs on npm, six on PyPI: landed and pushed; 0.2.0 committed 2026-10-25 (`f40b2e6`, unpushed); two PyPI packs live, four uploading; left: Arpit pushes, publishes npm's two new names, cuts the release. Filed by Arpit (Claude Code). [detail](open/W-31-registry-pack-split.md)

🟡 **W-32** `arpit` A Redux Toolkit pack and a TanStack Query pack, npm only: steps 2–4 done in the tree 2026-10-25 (7 + 12 rules, all green, wired); pushed, CI run 40185344678 green; left: their first publish at 0.2.0 with W-31 step 7. Filed by Arpit (Claude Code). [detail](open/W-32-redux-and-query-packs.md)

🟢 **W-33** `agent` The two Stencil rules missing from `stencil/recommended`: port `ban-side-effects` and `enforce-slot-jsdoc` from @stencil/eslint-plugin 1.4.0 (or record why not), and fix the pack README's count (23 rules on, not 21). Filed by Arpit (Claude). [detail](open/W-33-stencil-preset-gaps.md)

### testing

🟢 **W-19** `agent` `bench` times the three pinned repos on `ubuntu-24.04`; the method for the budgets ruled 2026-10-25; left: apply it to the three CI runs (`gh` on the Mac) and set every budget, a capped one back to the inbox. Filed by Arpit (Cowork). [detail](open/W-19-performance-budgets.md)

### record update

### research

🟢 **W-05** `cowork` Research guiding an agent **while** it writes: a `brief` before writing (rules filtered by path and construct) and a `hook --pre` check before a write reaches disk; Fux as an optional free-text layer. Filed by Arpit (Cowork). [detail](open/W-05-write-time-guidance.md)

---
