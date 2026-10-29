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
| 🔴 **GitHub Action — the Marketplace listing** ([W-97](open/W-97-github-action.md)): **v0.3.0 is out (2026-10-29)** — <https://github.com/arpitarya/codelaw/releases/edit/v0.3.0> → tick *Publish this Action to the GitHub Marketplace* → accept the Marketplace Developer Agreement → *Update release*. Ruled 2026-10-27: SARIF stays on `gh`; a moving `v0` tag; Claude then runs the throwaway-repo proof. | 2026-10-27 |
| ↳ **blocks:** W-97 itself | |
| 🔴 **Open VSX — the ownership claim and one token** ([W-117](open/W-117-open-vsx-setup.md)). Done 10-29: environment `vscode`, login, Publisher Agreement, namespace `codelaw`. **1. Claim:** <https://github.com/EclipseFdn/open-vsx.org/issues/new?template=claim-namespace-ownership.yml&namespace=codelaw> — tick *Ownership* and *Account Age*; Option 1 once the Marketplace has the extension (the repo is yours), or Option 4 now — or ask Claude to file it. **2. Token:** open-vsx.org → *Settings → Access Tokens* → *Generate* `codelaw-publish` → `gh secret set OVSX_PAT --env vscode -R arpitarya/codelaw` (paste at the prompt). | 2026-10-29 |
| ↳ **blocks:** W-117 itself | |
| 🔴 **Guidance-vs-check redesign — three rulings** ([W-118](open/W-118-guidance-experiment-redesign.md) → [proposal](proposals/guidance-experiment-redesign.md)): **R1** the design — six arms (a no-codelaw arm 0, guidance-only A, the pilot's A as A+, B, C, and `hook --pre` as D), tasks screened by running arm 0 (keep ≥ 40% at 5 runs), 12 tasks × 5 runs = 60 per arm; **R2** the README rule — the line stays unless arm C's upper 95% bound is below a bound you set (2% needs 0/150; 5% needs 0/60), or rate-only; **R3** the budget — ~$85 and ~3 h (screen ~$22 + run ~$60), screen gated separately or not. Open questions §7. | 2026-10-29 |
| ↳ **blocks:** W-118 itself; W-102 waits on it | |
| 🔴 **`deps` pack — the four registry names** ([W-128](open/W-128-deps-pack-and-vulnerabilities.md)): PyPI pending trusted publishers for `codelaw-rules-deps` and `codelaw-rules-deps-fux-index` (same repo, `publish.yml`, the packs' environment) — you asked Claude to do these in the browser, confirming before submit; npm's first hand publish of `@codelaw/rules-deps` and `@codelaw/rules-deps-fux-index` (RELEASING.md). | 2026-10-30 |
| ↳ **blocks:** W-128's release only (DoD 8) | |
| 🔴 **Q-A — findings for files outside L3** ([proposal §8](proposals/deps-flags-markers-reviews.md)): `.nvmrc`, `.python-version`, `.tool-versions`, Dockerfile, CODEOWNERS, yarn classic `yarn.lock`. **A** (being built): report on the nearest L3 file, naming the text file and line — no law change. **B:** amend L3 to admit them for findings only. | 2026-10-30 |
| ↳ **blocks:** nothing — W-129 and W-132 build A; B would move those findings | |

---

## Open items (16)

### build (13)

🟢 **W-126** `agent` A five-minute pipeline: built and running — cached CI 4m25, Rust+packs 12m30–13m38 (a macOS runner queue), release 3m50 (was 9m). Left: more runs of each kind measured. Filed by Arpit (Cowork). [detail](open/W-126-fast-pipeline.md)

🟢 **W-95** `agent` VS Code extension live as `codelaw.codelaw-vscode`; 0.4.0 uploaded for all eight platforms by `publish.yml` (10-30). Left: Open VSX (W-117), the install docs, `vsce`/`ovsx` in `publish.yml` (D-46, D-47). Filed by Arpit (Cowork). [detail](open/W-95-publish-vscode-extension.md)

🔴 **W-97** `arpit` GitHub Action built; ruled 10-27: SARIF via `gh`, a moving `v0`. Left: the Marketplace agreement on the v0.3.0 release (inbox), then Claude's throwaway-repo proof. Filed by Arpit (Cowork). [detail](open/W-97-github-action.md)

🟢 **W-105** `agent` Type-aware rules: five landed; ruled 10-29 — agents add the four type questions (SR-TYPES decision 4), the rules as previews; profile the slowest before budgets; corpus counts; the RC (11-10). Filed by Arpit (Cowork). [detail](open/W-105-type-aware-engine.md)

🟢 **W-64** `agent` `codelaw-pre-commit` mirror repo: the pre-commit framework installs codelaw from PyPI by version, as ruff-pre-commit does. Filed by Arpit (Cowork). [detail](open/W-64-codelaw-pre-commit-mirror.md)

🔴 **W-117** `arpit` Open VSX: environment, login, agreement and namespace done 10-29. Left: the ownership claim (no issue found yet) and one token as `OVSX_PAT` (inbox); then the trusted publisher. Filed by Arpit (Cowork). [detail](open/W-117-open-vsx-setup.md)

🟡 **W-106** waits on **W-105** `agent` TypeScript version watch: tested range, CI (7.1 blocking, `typescript@next` nightly), a bump job that opens a PR or an issue. Filed by Arpit (Cowork). [detail](open/W-106-typescript-version-watch.md)

🟢 **W-128** `agent` The `deps` pack (nineteenth): `deps refresh` merges advisories from on/off sources (registry first, OSV), `deps/vulnerable` names the safe version, direct and transitive; agents told via MCP, hook (inform/block), `rules_for_path`, skills. Filed by Arpit (Cowork). [detail](open/W-128-deps-pack-and-vulnerabilities.md)

🟡 **W-129** waits on **W-128** `agent` `deps/licence`, `deps/lockfile-sync`, `deps/version-drift`, `deps/runtime-agreement`. Filed by Arpit (Cowork). [detail](open/W-129-deps-hygiene-rules.md)

🟢 **W-130** `agent` Feature-flag expiry: accessor patterns with six SDK presets, wrappers found through the graph, `.codelaw/flags.toml` and inline markers (registry wins), `keep` collapse fix. Filed by Arpit (Cowork). [detail](open/W-130-feature-flag-expiry.md)

🟢 **W-131** `agent` TODOs with configurable ticket and date; no focused tests; skipped tests need a reason and expiry. Filed by Arpit (Cowork). [detail](open/W-131-todo-and-skipped-test-markers.md)

🟢 **W-132** `agent` Complexity, nesting and React component limits; CODEOWNERS coverage. Filed by Arpit (Cowork). [detail](open/W-132-complexity-and-codeowners.md)

🟢 **W-133** `agent` Skill: repeated review comments → proposed rules (GitHub, GitLab, Bitbucket, Azure DevOps; ≥ 3 repeats, ≥ 1 reviewer, 6 months — all configurable), approved step by step, landed as previews. Filed by Arpit (Cowork). [detail](open/W-133-rules-from-review-comments.md)

### testing (0)

### reporting and benchmarks (2)

🟢 **W-103** `agent` Spell pack: seven rules, fixtures, corpus done; ruled 10-29 — measure it as its own `spell` operation (B-062: the shared benchmark later), then its L8 budget. Filed by Arpit (Cowork). [detail](open/W-103-spell-pack.md)

🔴 **W-102** waits on **W-118** `agent` Pilot run 10-28: 60 sessions, $10.47; A = B = C at 2/20. Ruled 10-29: redesign first (W-118's research), then the run, the write-up and the README line. Filed by Arpit (Cowork). [detail](open/W-102-guidance-vs-check-experiment.md)

### record update (0)

### research (1)

🔴 **W-118** `arpit` Research done 10-29: the pilot's six causes, seven papers, a six-arm design with screened temptations and a pre-registered bound — [proposal](proposals/guidance-experiment-redesign.md); R1–R3 in the inbox. Filed by Arpit (Cowork). [detail](open/W-118-guidance-experiment-redesign.md)

---
