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
| 🔴 **The one-time registry setup, steps 4 and 5** — the seven PyPI pending publishers (`codelaw` and six packs, `codelaw-rules-react` among them since W-31), the `npm`/`pypi` environments ([`RELEASING.md`](../RELEASING.md) §One time). Steps 1–3 are done for npm (2026-10-24, W-29), and the decider gave the go by publishing 0.1.0 there | 2026-10-20 |
| ↳ **blocks:** W-04 step 5 (the first release on both registries) · W-03 step 6 · W-30 step 4 | |
| 🔴 Create the `npm` and `pypi` environments and give each one tag rule `v*` and no branch rule (GitHub Settings → Environments; [`RELEASING.md`](../RELEASING.md) step 5). Checked 2026-10-22: neither exists yet; an agent's attempt through the browser was refused by the permission check, so it stays the decider's | 2026-10-20 |
| ↳ **blocks:** W-18 | |
| 🔴 **W-32: name the Redux Toolkit and React Query packs and pick the Redux source** — proposed `@codelaw/rules-redux` and `@codelaw/rules-query`, rules from the official Redux Style Guide (no official RTK ESLint plugin exists) ([W-32](open/W-32-redux-and-query-packs.md) §The decider decides) | 2026-10-24 |
| ↳ **blocks:** W-32 | |
| 🔴 Set L8's budgets: after the `perf` job's first run on `main`, apply min(design target, 1.5 × worst p95) to the CI runner's numbers and rule each `budget` in `perf/budgets.toml`; name the reference machine ([W-19](open/W-19-performance-budgets.md)) | 2026-10-22 |
| ↳ **blocks:** W-19 DoD 3 · the perf gate binding anything | |
| 🔴 **W-21: submit the tree-sitter-css upstream PR and rule DoD 3** — the patch and paste-ready PR text are in `crates/codelaw-lang/grammars/tree-sitter-css/upstream/` (checked against upstream v0.25.0: 40/40 → 46/46). DoD 3 does not hold: all 70 CSS rules read text, and the fork still misparses forms they need. (a) restate DoD 3 as engine-only (recommended) or (b) file the fork extension and a 70-rule port ([W-21](open/W-21-pack-engine-gaps.md)) | 2026-10-24 |
| ↳ **blocks:** W-21 | |
| 🔴 **W-28: confirm two consequences of the token rule** — SR-FORMATTER decision 3 was rewritten on your 2026-10-22 *"make the decision"*; under it a rule that only changes quotes, semicolons or trailing commas passes with a warning instead of being refused, and CSS `not(` vs `not (` is layout ([W-28](open/W-28-layout-check-by-tokens.md)) | 2026-10-24 |
| ↳ **blocks:** W-28 closing | |
| 🔴 **W-20: may CI install each corpus repo's own dependencies?** — needed for DoD "the repo still builds and passes its tests after fixes"; an L2-scope question ([W-20](open/W-20-conformance-tests.md)) | 2026-10-24 |
| ↳ **blocks:** W-20's build-and-test leg | |

---

## Open items

### build

🔴 **W-04** `agent` The release pipeline, steps 1–4 done 2026-10-20. Step 5, the first release: blocked on the registry setup above; the engine is complete since 2026-10-21 (L7, L6 decision 3). Filed by Arpit (Claude Code). [detail](open/W-04-release-pipeline.md)

🔴 **W-03** `agent` The rule packs: steps 2–5 done (419 rules), packaging done; nine packs since W-31. Step 6, publishing: blocked on the registry setup above. Filed by Arpit (Cowork). [detail](open/W-03-rule-packs.md)

🔴 **W-21** `agent` Engine gaps: all three steps landed 2026-10-22; the upstream patch checked against upstream's v0.25.0 and its PR text written (2026-10-24, merged `1f48463`); waiting on the decider to submit it and rule DoD 3 (inbox). Filed by Arpit (Cowork). [detail](open/W-21-pack-engine-gaps.md)

🟢 **W-29** `arpit` The first npm publish: 14 names live at 0.1.0, trusted publishing bound (2026-10-24), from the `dist` of a failed CI run; left: the install check and the records (DoD 2, 4). Filed by Arpit (Claude Code). [detail](open/W-29-first-npm-publish.md)

🟡 **W-30** `arpit` The first PyPI publish: `codelaw` (8 wheels) and the six `codelaw-rules-*` of W-31's layout, at 0.2.0 rather than 0.1.0; waiting on W-31 (0.2.0). Filed by Arpit (Claude Code). [detail](open/W-30-first-pypi-publish.md)

🟢 **W-31** `arpit` Eight packs on npm, six on PyPI, `rules-js` split in five: landed 2026-10-24, and `@codelaw/rules-python` is off npm. Left: commit and push, then 0.2.0. Filed by Arpit (Claude Code). [detail](open/W-31-registry-pack-split.md)

🔴 **W-32** `agent` A Redux Toolkit pack and a React Query pack, npm only: waiting on the decider's naming and Redux-source ruling (inbox). Filed by Arpit (Claude Code). [detail](open/W-32-redux-and-query-packs.md)

🟢🧨 **W-24** `agent` CI cannot pass on `main`: all seven steps merged 2026-10-24 (`cf52b58`) — replay allow-list (ten merges), cache keys, law gate binds, perf path filter, inbox UTF-8, staging-index reverse-DNS stall; left: push `main` and prove green. Filed by Arpit (Claude Code). [detail](open/W-24-ci-green-on-main.md)

🟢 **W-26** `agent` Public docs from the tree: `docs-sync.py`, the `*/project` presets, README/CHANGELOG regenerated, merged 2026-10-24; left: the `ci.yml` step, the register row for the script, the item file. Filed by Arpit (Claude Code). [detail](open/W-26-public-docs-from-the-tree.md)

🟢 **W-27** `agent` Engine and packaging hygiene: all six steps merged 2026-10-24 (MSRV 1.97, one `kill_group`, Windows one-shot, `version.py` over the extension, one pack builder); left: CI proves DoD 1 and 3. Filed by Arpit (Claude Code). [detail](open/W-27-engine-and-packaging-hygiene.md)

🔴 **W-28** `agent` Layout check by codelaw's tokens: `token_stream`, the advisory formatter cross-check, SR-FORMATTER decision 3 and SR-RULE-AUTHORING decision 4 rewritten, merged 2026-10-24 (`666ae38`); waiting on the decider's confirmation (inbox). Filed by Arpit (Claude Code). [detail](open/W-28-layout-check-by-tokens.md)

### testing

🔴 **W-18** `agent` CI and release hardened 2026-10-20 (pipefail, SHA-pinned actions, pinned npm, verified `SHA256SUMS` + attestation, eight-platform smoke job); left: the tag-only environment rule (decider, inbox) and the staging-index install. Filed by Arpit (Cowork). [detail](open/W-18-release-hardening.md)
🟢 **W-20** `agent` Conformance tests: the pinned corpus (10 permissive repos) and its `corpus` CI job merged 2026-10-24; left: generate `packs/*/corpus.toml` (the job fails until then); the build-and-test leg waits on an L2 ruling (inbox). Filed by Arpit (Cowork). [detail](open/W-20-conformance-tests.md)
🔴 **W-19** `agent` `codelaw bench`, the seeded 10k workload, proposed budgets and the CI perf gate landed 2026-10-22 (measured on a loaded Mac); left: re-measure on CI's runner, then the decider sets the budgets (inbox). Filed by Arpit (Cowork). [detail](open/W-19-performance-budgets.md)

### record update

🟢 **W-25** `agent` Records back in line with the tree: owner rows, `built` column, dead B-ids mapped, inbox `age` computed (rule 40), merged 2026-10-24; left: the checker, item-file `ball:` fixes, registry §3 dates. Filed by Arpit (Claude Code). [detail](open/W-25-governance-drift.md)

### research

🟢 **W-05** `cowork` Research guiding an agent **while** it writes: a `brief` before writing (rules filtered by path and construct) and a `hook --pre` check before a write reaches disk; Fux as an optional free-text layer. Filed by Arpit (Cowork). [detail](open/W-05-write-time-guidance.md)

---
