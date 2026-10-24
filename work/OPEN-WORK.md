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
| 🔴 **The one-time registry setup, steps 4 and 5** — the seven PyPI pending publishers (`codelaw` and six packs, `codelaw-rules-react` among them since W-31), the `npm`/`pypi` environments ([`RELEASING.md`](../RELEASING.md) §One time). Steps 1–3 are done for npm (2026-10-24, W-29), and the decider gave the go by publishing 0.1.0 there | 2026-10-20 | 4 d |
| ↳ **blocks:** W-04 step 5 (the first release on both registries) · W-03 step 6 · W-30 step 4 | | |
| 🔴 Create the `npm` and `pypi` environments and give each one tag rule `v*` and no branch rule (GitHub Settings → Environments; [`RELEASING.md`](../RELEASING.md) step 5). Checked 2026-10-22: neither exists yet; an agent's attempt through the browser was refused by the permission check, so it stays the decider's | 2026-10-20 | 4 d |
| ↳ **blocks:** W-18 | | |
| 🔴 **W-32: name the Redux Toolkit and React Query packs and pick the Redux source** — proposed `@codelaw/rules-redux` and `@codelaw/rules-query`, rules from the official Redux Style Guide (no official RTK ESLint plugin exists) ([W-32](open/W-32-redux-and-query-packs.md) §The decider decides) | 2026-10-24 | 0 d |
| ↳ **blocks:** W-32 | | |
| 🔴 Set L8's budgets: after the `perf` job's first run on `main`, apply min(design target, 1.5 × worst p95) to the CI runner's numbers and rule each `budget` in `perf/budgets.toml`; name the reference machine ([W-19](open/W-19-performance-budgets.md)) | 2026-10-22 | 2 d |
| ↳ **blocks:** W-19 DoD 3 · the perf gate binding anything | | |

---

## Open items

### build

🔴 **W-04** `agent` The release pipeline, steps 1–4 done 2026-10-20. Step 5, the first release: blocked on the registry setup above; the engine is complete since 2026-10-21 (L7, L6 decision 3). Filed by Arpit (Claude Code). [detail](open/W-04-release-pipeline.md)

🔴 **W-03** `agent` The rule packs: steps 2–5 done (419 rules), packaging done; nine packs since W-31. Step 6, publishing: blocked on the registry setup above. Filed by Arpit (Cowork). [detail](open/W-03-rule-packs.md)

🟢 **W-21** `agent` Engine gaps: all three steps landed 2026-10-22, the CSS grammar as an in-tree fork (the decider's ruling); left: file the upstream patch, and whether DoD 3 holds while CSS rules read text. Filed by Arpit (Cowork). [detail](open/W-21-pack-engine-gaps.md)

🟢 **W-29** `arpit` The first npm publish: 14 names live at 0.1.0, trusted publishing bound (2026-10-24), from the `dist` of a failed CI run; left: the install check and the records (DoD 2, 4). Filed by Arpit (Claude Code). [detail](open/W-29-first-npm-publish.md)

🟡 **W-30** `arpit` The first PyPI publish: `codelaw` (8 wheels) and the six `codelaw-rules-*` of W-31's layout, at 0.2.0 rather than 0.1.0; waiting on W-31 (0.2.0). Filed by Arpit (Claude Code). [detail](open/W-30-first-pypi-publish.md)

🟢 **W-31** `arpit` Eight packs on npm, six on PyPI, `rules-js` split in five: landed 2026-10-24, and `@codelaw/rules-python` is off npm. Left: commit and push, then 0.2.0. Filed by Arpit (Claude Code). [detail](open/W-31-registry-pack-split.md)

🔴 **W-32** `agent` A Redux Toolkit pack and a React Query pack, npm only: waiting on the decider's naming and Redux-source ruling (inbox). Filed by Arpit (Claude Code). [detail](open/W-32-redux-and-query-packs.md)

🟢🧨 **W-24** `agent` CI cannot pass on `main`: the Windows `bench` build fixed 2026-10-24 (unproven until the push); left: the guard replay on nine as-is merges, cache keys that miss test inputs, the law gate on `--report`. Filed by Arpit (Claude Code). [detail](open/W-24-ci-green-on-main.md)

🟢 **W-26** `agent` Generate the README's counts, command table and roadmap, the pack READMEs and the CHANGELOG from the binary and the packs, with a CI check; make the three `graph/*` rules reachable. Filed by Arpit (Claude Code). [detail](open/W-26-public-docs-from-the-tree.md)

🟢 **W-27** `agent` Engine and packaging hygiene: a true `rust-version`, one process-group kill, Windows one-shot behaviour stated and smoke-tested, `version.py` over the extension manifest, one pack builder. Filed by Arpit (Claude Code). [detail](open/W-27-engine-and-packaging-hygiene.md)

🟢 **W-28** `agent` The layout-rule check decided by codelaw's own tokens (same token stream = layout), the formatter comparison kept as an advisory cross-check; SR-FORMATTER decision 3 rewritten. Filed by Arpit (Claude Code). [detail](open/W-28-layout-check-by-tokens.md)

### testing

🔴 **W-18** `agent` CI and release hardened 2026-10-20 (pipefail, SHA-pinned actions, pinned npm, verified `SHA256SUMS` + attestation, eight-platform smoke job); left: the tag-only environment rule (decider, inbox) and the staging-index install. Filed by Arpit (Cowork). [detail](open/W-18-release-hardening.md)
🟢 **W-20** `agent` Conformance tests landed and verified 2026-10-22 (L5 fence, L2 over npm/PyPI tools, `init` paths, promotion gate, work-system checks); npm's Artistic-2.0 ruled permissive 2026-10-22; left: the pinned corpus and its CI job. Filed by Arpit (Cowork). [detail](open/W-20-conformance-tests.md)
🔴 **W-19** `agent` `codelaw bench`, the seeded 10k workload, proposed budgets and the CI perf gate landed 2026-10-22 (measured on a loaded Mac); left: re-measure on CI's runner, then the decider sets the budgets (inbox). Filed by Arpit (Cowork). [detail](open/W-19-performance-budgets.md)

### record update

🟢 **W-25** `agent` Records and work files back in line with the tree: owners for `codelaw-graph` and `perf/`, the register's `built` column, dead `B-nnn` ids, stale blockers and `ball:` values in item files, the copied inbox `age`. Filed by Arpit (Claude Code). [detail](open/W-25-governance-drift.md)

### research

🟢 **W-05** `cowork` Research guiding an agent **while** it writes: a `brief` before writing (rules filtered by path and construct) and a `hook --pre` check before a write reaches disk; Fux as an optional free-text layer. Filed by Arpit (Cowork). [detail](open/W-05-write-time-guidance.md)

---
