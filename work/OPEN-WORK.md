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

**A ball on every row, always one:** 🔴 blocked on the decider, directly or through another item · 🟣 gated on a named date, directly or through another item · 🟡 waiting on another item, named right after the id — `waits on **W-mm**`, or `**<project> W-mm**` for another project's · 🟢 no blockers.
**Optional, after the ball:** 🧨 broken or getting worse · 🔺 do first (the decider only). Full legend: [SR-WORK-OPEN-QUEUE](../records/0051_WORK-open-queue.md) rules 20–34.

---

## Blocked on the decider

| what the decider decides | filed |
|---|---|
| 🔴 **GitHub Action — the Marketplace listing** ([W-97](open/W-97-github-action.md)): ruled 2026-10-30 — the listing refused `name: codelaw` (taken by a user or org) and the 138-character description; `action.yml` is now **`codelaw check`** with a 115-character description. **On the next release** (after 0.4.0): edit it → tick *Publish this Action to the GitHub Marketplace* → accept the agreement → category *Code quality* → *Update release*. | 2026-10-27 |
| ↳ **blocks:** W-97 itself | |
| 🔴 **Open VSX — waiting on Open VSX** ([W-117](open/W-117-open-vsx-setup.md)): the ownership claim is [EclipseFdn/open-vsx.org#14928](https://github.com/EclipseFdn/open-vsx.org/issues/14928), open. Ruled 2026-10-30: **no token — trusted publishing.** Nothing to do until the claim is approved; then you add the trusted publisher (`arpitarya` · `codelaw` · `publish.yml` · `vscode`). | 2026-10-29 |
| ↳ **blocks:** W-117 itself | |

---

## Open items (18)

### build (15)

🟢 **W-126** `agent` A five-minute pipeline: built and running — cached CI 4m25, Rust+packs 12m30–13m38 (a macOS runner queue), release 3m50 (was 9m). Left: more runs of each kind measured. Filed by Arpit (Cowork). [detail](open/W-126-fast-pipeline.md)

🟢 **W-95** `agent` VS Code extension live as `codelaw.codelaw-vscode`; 0.4.0 uploaded for all eight platforms by `publish.yml` (10-30). Left: Open VSX (W-117), the install docs, `vsce`/`ovsx` in `publish.yml` (D-46, D-47). Filed by Arpit (Cowork). [detail](open/W-95-publish-vscode-extension.md)

🔴 **W-97** `arpit` GitHub Action built; ruled 10-27: SARIF via `gh`, a moving `v0`. Ruled 10-30: listed as **codelaw check** (the name `codelaw` is taken), description shortened. Left: the Marketplace tick on the next release (inbox), then Claude's throwaway-repo proof. Filed by Arpit (Cowork). [detail](open/W-97-github-action.md)

🟢 **W-105** `agent` Type-aware rules: five landed; ruled 10-29 — agents add the four type questions (SR-TYPES decision 4), the rules as previews; profile the slowest before budgets; corpus counts; the RC (11-10). Filed by Arpit (Cowork). [detail](open/W-105-type-aware-engine.md)

🟢 **W-64** `agent` `codelaw-pre-commit` mirror repo: the pre-commit framework installs codelaw from PyPI by version, as ruff-pre-commit does. Filed by Arpit (Cowork). [detail](open/W-64-codelaw-pre-commit-mirror.md)

🔴 **W-117** `arpit` Open VSX: environment, login, agreement, namespace done 10-29; claim #14928 filed, waiting on Open VSX. Ruled 10-30: no token, trusted publishing (inbox). Filed by Arpit (Cowork). [detail](open/W-117-open-vsx-setup.md)

🟡 **W-106** waits on **W-105** `agent` TypeScript version watch: tested range, CI (7.1 blocking, `typescript@next` nightly), a bump job that opens a PR or an issue. Filed by Arpit (Cowork). [detail](open/W-106-typescript-version-watch.md)

🟢 **W-128** `agent` The `deps` pack (nineteenth): `deps refresh` merges advisories from on/off sources (registry first, OSV), `deps/vulnerable` names the safe version, direct and transitive; agents told via MCP, hook (inform/block), `rules_for_path`, skills. Filed by Arpit (Cowork). [detail](open/W-128-deps-pack-and-vulnerabilities.md)

🟡 **W-129** waits on **W-128** `agent` `deps/licence`, `deps/lockfile-sync`, `deps/version-drift`, `deps/runtime-agreement`. Filed by Arpit (Cowork). [detail](open/W-129-deps-hygiene-rules.md)

🟢 **W-130** `agent` Feature-flag expiry: accessor patterns with six SDK presets, wrappers found through the graph, `.codelaw/flags.toml` and inline markers (registry wins), `keep` collapse fix. Filed by Arpit (Cowork). [detail](open/W-130-feature-flag-expiry.md)

🟢 **W-131** `agent` TODOs with configurable ticket and date; no focused tests; skipped tests need a reason and expiry. Filed by Arpit (Cowork). [detail](open/W-131-todo-and-skipped-test-markers.md)

🟢 **W-132** `agent` Complexity, nesting and React component limits; CODEOWNERS coverage. Filed by Arpit (Cowork). [detail](open/W-132-complexity-and-codeowners.md)

🟢 **W-133** `agent` Skill: repeated review comments → proposed rules (GitHub, GitLab, Bitbucket, Azure DevOps; ≥ 3 repeats, ≥ 1 reviewer, 6 months — all configurable), approved step by step, landed as previews. Filed by Arpit (Cowork). [detail](open/W-133-rules-from-review-comments.md)

🟡 **W-134** waits on **W-128** `arpit` PyPI pending trusted publishers for `codelaw-rules-deps` and its Fux index — Claude fills the form in Chrome, Arpit submits; just before the deps release. Filed by Arpit (Cowork). [detail](open/W-134-deps-pypi-pending-publishers.md)

🟡 **W-135** waits on **W-128** `arpit` npm first hand publish of `@codelaw/rules-deps` and its Fux index, then their trusted publisher; just before the deps release. Filed by Arpit (Cowork). [detail](open/W-135-deps-npm-first-publish.md)

### testing (0)

### reporting and benchmarks (2)

🟢 **W-103** `agent` Spell pack: seven rules, fixtures, corpus done; ruled 10-29 — measure it as its own `spell` operation (B-062: the shared benchmark later), then its L8 budget. Filed by Arpit (Cowork). [detail](open/W-103-spell-pack.md)

🟢 **W-102** `agent` Pilot run 10-28: 60 sessions, $10.47; A = B = C at 2/20. Ruled 10-30: W-118's redesign as proposed — six arms, screened tasks, 60 runs per arm; the README line stays unless arm C is 0/60 (bound 5%); ~$85, no separate gate on the screen. Filed by Arpit (Cowork). [detail](open/W-102-guidance-vs-check-experiment.md)

### record update (1)

🟢 **W-136** `agent` L3 amended: `.nvmrc`, `.python-version`, `.tool-versions`, Dockerfile, CODEOWNERS and yarn classic `yarn.lock` may carry findings (never linted as languages); a finding shows on the manifest **and** on the text file's line. Filed by Arpit (Cowork). [detail](open/W-136-l3-findings-on-plain-text-config.md)

### research (0)


---
