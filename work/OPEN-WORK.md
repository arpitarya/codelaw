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
| 🔴 **Release 0.4.0 — push `main`** ([W-127](open/W-127-release-0-4-0.md)): bumped and CHANGELOG written 10-29; `main` is ahead of `origin` (W-95, W-119, W-126, the bump). Say *push*, or `git push origin main`; when `ci.yml` is green, `gh release create v0.4.0` from CHANGELOG's section (W-127 step 4). | 2026-10-29 |
| ↳ **blocks:** W-127; W-126 step 4 | |
| 🔴 **Guidance-vs-check redesign — three rulings** ([W-118](open/W-118-guidance-experiment-redesign.md) → [proposal](proposals/guidance-experiment-redesign.md)): **R1** the design — six arms (a no-codelaw arm 0, guidance-only A, the pilot's A as A+, B, C, and `hook --pre` as D), tasks screened by running arm 0 (keep ≥ 40% at 5 runs), 12 tasks × 5 runs = 60 per arm; **R2** the README rule — the line stays unless arm C's upper 95% bound is below a bound you set (2% needs 0/150; 5% needs 0/60), or rate-only; **R3** the budget — ~$85 and ~3 h (screen ~$22 + run ~$60), screen gated separately or not. Open questions §7. | 2026-10-29 |
| ↳ **blocks:** W-118 itself; W-102 waits on it | |

---

## Open items (11)

### build (8)

🔴 **W-127** `arpit` Release 0.4.0 through CI: bumped and CHANGELOG written 10-29; next push `main` (your go), `ci.yml` green, then `gh release create v0.4.0` from CHANGELOG's section. Filed by Arpit (Claude Code). [detail](open/W-127-release-0-4-0.md)

🟢 **W-126** `agent` A five-minute pipeline: built 10-29 — corpus one job per pack, Fux job apart, smoke and `publish.yml` in parallel (npm tiers kept). Left: step 4, three runs of each kind measured, once `main` is pushed. Filed by Arpit (Cowork). [detail](open/W-126-fast-pipeline.md)

🟢 **W-95** `agent` VS Code extension: the Marketplace blocks `codelaw` (removed 0.2.0; Support emailed 10-29). Renamed `codelaw.codelaw-vscode` 10-29; its 0.3.0 `.vsix` files in `dist/vscode/` for the upload by hand; then `vsce`/`ovsx` in `publish.yml` (D-46, D-47). Filed by Arpit (Cowork). [detail](open/W-95-publish-vscode-extension.md)

🔴 **W-97** `arpit` GitHub Action built; ruled 10-27: SARIF via `gh`, a moving `v0`. Left: the Marketplace agreement on the v0.3.0 release (inbox), then Claude's throwaway-repo proof. Filed by Arpit (Cowork). [detail](open/W-97-github-action.md)

🟢 **W-105** `agent` Type-aware rules: five landed; ruled 10-29 — agents add the four type questions (SR-TYPES decision 4), the rules as previews; profile the slowest before budgets; corpus counts; the RC (11-10). Filed by Arpit (Cowork). [detail](open/W-105-type-aware-engine.md)

🟢 **W-64** `agent` `codelaw-pre-commit` mirror repo: the pre-commit framework installs codelaw from PyPI by version, as ruff-pre-commit does. Filed by Arpit (Cowork). [detail](open/W-64-codelaw-pre-commit-mirror.md)

🔴 **W-117** `arpit` Open VSX: environment, login, agreement and namespace done 10-29. Left: the ownership claim (no issue found yet) and one token as `OVSX_PAT` (inbox); then the trusted publisher. Filed by Arpit (Cowork). [detail](open/W-117-open-vsx-setup.md)

🟡 **W-106** waits on **W-105** `agent` TypeScript version watch: tested range, CI (7.1 blocking, `typescript@next` nightly), a bump job that opens a PR or an issue. Filed by Arpit (Cowork). [detail](open/W-106-typescript-version-watch.md)

### testing (0)

### reporting and benchmarks (2)

🟢 **W-103** `agent` Spell pack: seven rules, fixtures, corpus done; ruled 10-29 — measure it as its own `spell` operation (B-062: the shared benchmark later), then its L8 budget. Filed by Arpit (Cowork). [detail](open/W-103-spell-pack.md)

🔴 **W-102** waits on **W-118** `agent` Pilot run 10-28: 60 sessions, $10.47; A = B = C at 2/20. Ruled 10-29: redesign first (W-118's research), then the run, the write-up and the README line. Filed by Arpit (Cowork). [detail](open/W-102-guidance-vs-check-experiment.md)

### record update (0)

### research (1)

🔴 **W-118** `arpit` Research done 10-29: the pilot's six causes, seven papers, a six-arm design with screened temptations and a pre-registered bound — [proposal](proposals/guidance-experiment-redesign.md); R1–R3 in the inbox. Filed by Arpit (Cowork). [detail](open/W-118-guidance-experiment-redesign.md)

---
