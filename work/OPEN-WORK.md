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
| 🔴 Set L8's budgets: after the `perf` job's first run on `main` (on `ubuntu-24.04`, the reference machine named 2026-10-24), apply min(design target, 1.5 × worst p95) and rule each `budget` in `perf/budgets.toml` ([W-19](open/W-19-performance-budgets.md)) | 2026-10-22 |
| ↳ **blocks:** W-19 DoD 3 · the perf gate binding anything | |

---

## Open items

### build

🔴 **W-04** `agent` The release pipeline, steps 1–4 done 2026-10-20. Step 5, the first release: blocked on the registry setup above; the engine is complete since 2026-10-21 (L7, L6 decision 3). Filed by Arpit (Claude Code). [detail](open/W-04-release-pipeline.md)

🔴 **W-03** `agent` The rule packs: steps 2–5 done (419 rules), packaging done; nine packs since W-31. Step 6, publishing: blocked on the registry setup above. Filed by Arpit (Cowork). [detail](open/W-03-rule-packs.md)


🟢 **W-29** `arpit` The first npm publish: 14 names live at 0.1.0, trusted publishing bound (2026-10-24), from the `dist` of a failed CI run; left: the install check and the records (DoD 2, 4). Filed by Arpit (Claude Code). [detail](open/W-29-first-npm-publish.md)

🟡 **W-30** `arpit` The first PyPI publish: `codelaw` (8 wheels) and the six `codelaw-rules-*` of W-31's layout, at 0.2.0 rather than 0.1.0; waiting on W-31 (0.2.0). Filed by Arpit (Claude Code). [detail](open/W-30-first-pypi-publish.md)

🟢 **W-31** `arpit` Eight packs on npm, six on PyPI, `rules-js` split in five: landed 2026-10-24, and `@codelaw/rules-python` is off npm. Left: commit and push, then 0.2.0. Filed by Arpit (Claude Code). [detail](open/W-31-registry-pack-split.md)

🟢 **W-32** `agent` A Redux Toolkit pack and a React Query pack, npm only: named 2026-10-24 (`@codelaw/rules-redux-toolkit`, `@codelaw/rules-react-query`; Redux Style Guide rules); left: steps 2–4, then Arpit's first publish. Filed by Arpit (Claude Code). [detail](open/W-32-redux-and-query-packs.md)

🟢🧨 **W-24** `agent` CI cannot pass on `main`: all seven steps merged 2026-10-24 (`cf52b58`) — replay allow-list (ten merges), cache keys, law gate binds, perf path filter, inbox UTF-8, staging-index reverse-DNS stall; left: push `main` and prove green. Filed by Arpit (Claude Code). [detail](open/W-24-ci-green-on-main.md)

🟢 **W-27** `agent` Engine and packaging hygiene: all six steps merged 2026-10-24 (MSRV 1.97, one `kill_group`, Windows one-shot, `version.py` over the extension, one pack builder); left: CI proves DoD 1 and 3. Filed by Arpit (Claude Code). [detail](open/W-27-engine-and-packaging-hygiene.md)

### testing

🟢 **W-18** `agent` CI and release hardened 2026-10-20 (pipefail, SHA-pinned actions, pinned npm, verified `SHA256SUMS` + attestation, eight-platform smoke job); tag-only environments set 2026-10-24; left: CI's first run proves the staging-index install. Filed by Arpit (Cowork). [detail](open/W-18-release-hardening.md)
🟢 **W-20** `agent` Conformance tests: the corpus (11 repos, all nine packs) and every `packs/*/corpus.toml` done 2026-10-24, re-run clean; left: CI's first run on `main` proves the `corpus` job. Filed by Arpit (Cowork). [detail](open/W-20-conformance-tests.md)
🟢 **W-19** `agent` `codelaw bench`, the seeded 10k workload, proposed budgets and the CI perf gate landed 2026-10-22; machine and three repos named 2026-10-24; left: pin `perf` to `ubuntu-24.04`, bench the three repos, then the budgets (inbox). Filed by Arpit (Cowork). [detail](open/W-19-performance-budgets.md)

### record update

### research

🟢 **W-05** `cowork` Research guiding an agent **while** it writes: a `brief` before writing (rules filtered by path and construct) and a `hook --pre` check before a write reaches disk; Fux as an optional free-text layer. Filed by Arpit (Cowork). [detail](open/W-05-write-time-guidance.md)

---
