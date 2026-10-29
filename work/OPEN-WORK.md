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
| 🔴 **PyPI day 3 — Fri 2026-10-30** (four projects at 0.2.0 — the version npm got: two Fux index wheels in `dist/fux-index/pypi/`, and `codelaw-skills`, `codelaw-rules-scss` from the build you published npm from, copied into `dist/pypi/` first): `twine upload --skip-existing dist/fux-index/pypi/codelaw_rules_security_{html,python}_fux_index-0.2.0-py3-none-any.whl dist/pypi/codelaw_skills-0.2.0-py3-none-any.whl dist/pypi/codelaw_rules_scss-0.2.0-py3-none-any.whl` — then a trusted publisher on each — [W-88](open/W-88-pypi-day-3.md) | 2026-10-27 |
| ↳ **blocks:** W-88 itself | |
| 🔴 **PyPI day 4 — Sat 2026-10-31** (the three security rule packs at 0.2.0, from the same build, copied into `dist/pypi/` first, and `security-js`'s Fux index, moved from day 2): `twine upload --skip-existing dist/pypi/codelaw_rules_security_{js,html,python}-0.2.0-py3-none-any.whl dist/fux-index/pypi/codelaw_rules_security_js_fux_index-0.2.0-py3-none-any.whl` — then a trusted publisher on each — [W-89](open/W-89-pypi-day-4.md) | 2026-10-27 |
| ↳ **blocks:** W-89 itself | |
| 🔴 **GitHub Action — the Marketplace listing** ([W-97](open/W-97-github-action.md)): **v0.3.0 is out (2026-10-29)** — <https://github.com/arpitarya/codelaw/releases/edit/v0.3.0> → tick *Publish this Action to the GitHub Marketplace* → accept the Marketplace Developer Agreement → *Update release*. Ruled 2026-10-27: SARIF stays on `gh`; a moving `v0` tag; Claude then runs the throwaway-repo proof. | 2026-10-27 |
| ↳ **blocks:** W-97 itself | |
| 🔴 **VS Code extension — run its job once** ([W-95](open/W-95-publish-vscode-extension.md)): v0.3.0's Marketplace upload was refused (content type, now fixed on `main`). Publishes `codelaw.codelaw` 0.3.0 to the Marketplace: `gh workflow run publish.yml -R arpitarya/codelaw --ref main -f only=vscode`. Say **"run it"** and an agent runs it and checks the listing. | 2026-10-29 |
| ↳ **blocks:** W-95 itself | |
| 🔴 **Open VSX — the ownership claim and one token** ([W-117](open/W-117-open-vsx-setup.md)). Done 10-29: environment `vscode`, login, Publisher Agreement, namespace `codelaw`. **1. Claim:** <https://github.com/EclipseFdn/open-vsx.org/issues/new?template=claim-namespace-ownership.yml&namespace=codelaw> — tick *Ownership* and *Account Age*; Option 1 once the Marketplace has the extension (the repo is yours), or Option 4 now — or ask Claude to file it. **2. Token:** open-vsx.org → *Settings → Access Tokens* → *Generate* `codelaw-publish` → `gh secret set OVSX_PAT --env vscode -R arpitarya/codelaw` (paste at the prompt). | 2026-10-29 |
| ↳ **blocks:** W-117 itself | |
| 🔴 **Guidance-vs-check redesign — three rulings** ([W-118](open/W-118-guidance-experiment-redesign.md) → [proposal](proposals/guidance-experiment-redesign.md)): **R1** the design — six arms (a no-codelaw arm 0, guidance-only A, the pilot's A as A+, B, C, and `hook --pre` as D), tasks screened by running arm 0 (keep ≥ 40% at 5 runs), 12 tasks × 5 runs = 60 per arm; **R2** the README rule — the line stays unless arm C's upper 95% bound is below a bound you set (2% needs 0/150; 5% needs 0/60), or rate-only; **R3** the budget — ~$85 and ~3 h (screen ~$22 + run ~$60), screen gated separately or not. Open questions §7. | 2026-10-29 |
| ↳ **blocks:** W-118 itself; W-102 waits on it | |

---

## Open items (12)

### build (10)

🔴 **W-88** `arpit` PyPI day 3 (Fri 10-30): `security-html`, `security-python` Fux index, `codelaw-skills` and `codelaw-rules-scss`, all 0.2.0; then their trusted publishers. Command in the inbox. Filed by Arpit (Cowork). [detail](open/W-88-pypi-day-3.md)

🔴 **W-89** `arpit` PyPI day 4 (Sat 10-31): `codelaw-rules-security-{js,html,python}` and `security-js`'s Fux index (0.2.0), then their trusted publishers. Command in the inbox. Filed by Arpit (Cowork). [detail](open/W-89-pypi-day-4.md)

🔴 **W-95** `arpit` VS Code extension: v0.3.0 reached the Marketplace by trusted publishing, the upload refused (content type, fixed on `main`). Left: run the extension's job alone (inbox); Open VSX needs W-117's token. Filed by Arpit (Cowork). [detail](open/W-95-publish-vscode-extension.md)

🔴 **W-97** `arpit` GitHub Action built; ruled 10-27: SARIF via `gh`, a moving `v0`. Left: the Marketplace agreement on the v0.3.0 release (inbox), then Claude's throwaway-repo proof. Filed by Arpit (Cowork). [detail](open/W-97-github-action.md)

🟢 **W-105** `agent` Type-aware rules: five landed; ruled 10-29 — agents add the four type questions (SR-TYPES decision 4), the rules as previews; profile the slowest before budgets; corpus counts; the RC (11-10). Filed by Arpit (Cowork). [detail](open/W-105-type-aware-engine.md)

🟢 **W-103** `agent` Spell pack: seven rules, fixtures, corpus done; ruled 10-29 — measure it as its own `spell` operation (B-062: the shared benchmark later), then its L8 budget. Filed by Arpit (Cowork). [detail](open/W-103-spell-pack.md)

🟢 **W-64** `agent` `codelaw-pre-commit` mirror repo: the pre-commit framework installs codelaw from PyPI by version, as ruff-pre-commit does. Filed by Arpit (Cowork). [detail](open/W-64-codelaw-pre-commit-mirror.md)

🔴 **W-117** `arpit` Open VSX: environment, login, agreement and namespace done 10-29. Left: the ownership claim (no issue found yet) and one token as `OVSX_PAT` (inbox); then the trusted publisher. Filed by Arpit (Cowork). [detail](open/W-117-open-vsx-setup.md)

🟢 **W-115** `agent` The spell pack's first publish: npm `@codelaw/rules-spell` 0.2.0 live and trusted 10-29; left: PyPI `codelaw-rules-spell` and `@codelaw/rules-spell-fux-index`, by hand at the next release. Filed by Arpit (Cowork). [detail](open/W-115-spell-pack-first-publish.md)

🟡 **W-106** waits on **W-105** `agent` TypeScript version watch: tested range, CI (7.1 blocking, `typescript@next` nightly), a bump job that opens a PR or an issue. Filed by Arpit (Cowork). [detail](open/W-106-typescript-version-watch.md)

### testing (1)

🔴 **W-102** waits on **W-118** `agent` Pilot run 10-28: 60 sessions, $10.47; A = B = C at 2/20. Ruled 10-29: redesign first (W-118's research), then the run, the write-up and the README line. Filed by Arpit (Cowork). [detail](open/W-102-guidance-vs-check-experiment.md)

### record update (0)

### research (1)

🔴 **W-118** `arpit` Research done 10-29: the pilot's six causes, seven papers, a six-arm design with screened temptations and a pre-registered bound — [proposal](proposals/guidance-experiment-redesign.md); R1–R3 in the inbox. Filed by Arpit (Cowork). [detail](open/W-118-guidance-experiment-redesign.md)

---
