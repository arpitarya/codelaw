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
| 🔴 **PyPI day 2 — Thu 2026-10-29** (four Fux index projects, 0.2.0, already in `dist/fux-index/pypi/`): `twine upload --skip-existing dist/fux-index/pypi/codelaw_rules_{react,scss,python,security_js}_fux_index-0.2.0-py3-none-any.whl` — then a trusted publisher on each — [W-87](open/W-87-pypi-day-2.md) | 2026-10-27 |
| ↳ **blocks:** W-87 itself; W-34 waits on it | |
| 🔴 **PyPI day 3 — Fri 2026-10-30** (four projects at 0.2.0 — the version npm got: two Fux index wheels in `dist/fux-index/pypi/`, and `codelaw-skills`, `codelaw-rules-scss` from the build you published npm from, copied into `dist/pypi/` first): `twine upload --skip-existing dist/fux-index/pypi/codelaw_rules_security_{html,python}_fux_index-0.2.0-py3-none-any.whl dist/pypi/codelaw_skills-0.2.0-py3-none-any.whl dist/pypi/codelaw_rules_scss-0.2.0-py3-none-any.whl` — then a trusted publisher on each — [W-88](open/W-88-pypi-day-3.md) | 2026-10-27 |
| ↳ **blocks:** W-88 itself; W-34 and W-77 wait on it | |
| 🔴 **PyPI day 4 — Sat 2026-10-31** (the three security rule packs at 0.2.0, from the same build, copied into `dist/pypi/` first): `twine upload --skip-existing dist/pypi/codelaw_rules_security_{js,html,python}-0.2.0-py3-none-any.whl` — then a trusted publisher on each — [W-89](open/W-89-pypi-day-4.md) | 2026-10-27 |
| ↳ **blocks:** W-89 itself | |
| 🔴 **GitHub Action — the Marketplace listing** ([W-97](open/W-97-github-action.md)): on the v0.3.0 release ([W-116](open/W-116-release-0-3-0.md)), github.com/marketplace/actions → *Publish* on the release's edit page → accept the Marketplace Developer Agreement → tick *Publish this Action to the GitHub Marketplace*. Ruled 2026-10-27: SARIF stays on `gh`; a moving `v0` tag is kept; Claude runs the throwaway-repo proof after that tag. | 2026-10-27 |
| ↳ **blocks:** W-97 itself | |
| 🔴 **Open VSX — environment, account, token, namespace** ([W-117](open/W-117-open-vsx-setup.md); ruled 2026-10-29: Open VSX now, the Visual Studio Marketplace by hand on release day). **1. Environment** `vscode`, tag rule `v*` like `npm`/`pypi`: `gh api -X PUT repos/arpitarya/codelaw/environments/vscode -F 'deployment_branch_policy[protected_branches]=false' -F 'deployment_branch_policy[custom_branch_policies]=true' && gh api -X POST repos/arpitarya/codelaw/environments/vscode/deployment-branch-policies -f name='v*' -f type=tag`. **4. Open VSX:** <https://open-vsx.org> → *Log in* (GitHub) → avatar → *Settings* → *Log in with Eclipse* (an <https://accounts.eclipse.org> account with your GitHub username in its profile) → sign the *Publisher Agreement* → *Access Tokens* → *Generate New Token* `codelaw-publish`, copy it → `gh secret set OVSX_PAT --env vscode -R arpitarya/codelaw`; then the namespace: `read -rs T && curl -fsS -X POST -H 'Content-Type: application/json' -d '{"name":"codelaw"}' "https://open-vsx.org/api/-/namespace/create?token=$T"` (optional, the verified badge: *Claim namespace ownership* at <https://github.com/EclipseFdn/open-vsx.org/issues/new/choose>). Or do it with Claude in Chrome in a text session. | 2026-10-29 |
| ↳ **blocks:** W-117 itself | |

---

## Open items (19)

### build (17)

🟢 **W-114** `agent` Five worktrees of session 2df06f6d: ruled 10-29 "drop all five" — a Claude Code session on the Mac runs the item's commands (`--force`, `-D`). Filed by now-ledger.py (Claude Code). [detail](open/W-114-session-cleanup.md)

🟡 **W-86** waits on **W-116** `arpit` PyPI day 1: the four uploads and trusted publishers landed 10-28; the Release is now v0.3.0's (ruled 10-29), which creates `codelaw` on PyPI. Filed by Arpit (Cowork). [detail](open/W-86-pypi-day-1.md)

🔴 **W-87** `arpit` PyPI day 2 (Thu 10-29): the `react`, `scss`, `python` and `security-js` Fux index projects (0.2.0), then their trusted publishers. Command in the inbox. Filed by Arpit (Cowork). [detail](open/W-87-pypi-day-2.md)

🔴 **W-88** `arpit` PyPI day 3 (Fri 10-30): `security-html`, `security-python` Fux index, `codelaw-skills` and `codelaw-rules-scss`, all 0.2.0; then their trusted publishers. Command in the inbox. Filed by Arpit (Cowork). [detail](open/W-88-pypi-day-3.md)

🔴 **W-89** `arpit` PyPI day 4 (Sat 10-31): `codelaw-rules-security-{js,html,python}` 0.2.0, then their trusted publishers. Command in the inbox. Filed by Arpit (Cowork). [detail](open/W-89-pypi-day-4.md)

🟢 **W-95** `agent` VS Code extension: Marketplace **trusted publishing** policy active 10-29 (`arpitarya` · `codelaw` · `publish.yml` · `vscode`), no token. Left: the `vscode` environment, the `publish.yml` OIDC + `curl` job; Open VSX is W-117. Filed by Arpit (Cowork). [detail](open/W-95-publish-vscode-extension.md)

🔴 **W-97** `arpit` GitHub Action built; ruled 10-27: SARIF via `gh`, a moving `v0`. Left: the Marketplace agreement on the v0.3.0 release (inbox), then Claude's throwaway-repo proof. Filed by Arpit (Cowork). [detail](open/W-97-github-action.md)

🟢 **W-105** `agent` Type-aware rules: five landed; ruled 10-29 — agents add the four type questions (SR-TYPES decision 4), the rules as previews; profile the slowest before budgets; corpus counts; the RC (11-10). Filed by Arpit (Cowork). [detail](open/W-105-type-aware-engine.md)

🟢 **W-103** `agent` Spell pack: seven rules, fixtures, corpus done; ruled 10-29 — measure it as its own `spell` operation (B-062: the shared benchmark later), then its L8 budget. Filed by Arpit (Cowork). [detail](open/W-103-spell-pack.md)

🟡 **W-38** waits on **W-86** `agent` PyPI 0.2.0: `css`, `data` live and trusted; day 1 uploads the other four and publishes the Release; then an agent checks `pip install codelaw` and `uvx codelaw` and writes it down. Filed by Arpit (Cowork). [detail](open/W-38-pypi-0-2-0-by-hand.md)

🟡 **W-34** waits on **W-87** and **W-88** `agent` Fux index packages built; D-34 ratified 2026-10-27 (W-91); npm 15/15 and PyPI 4/10 live; after the publishing items, `publish.yml` uploads `dist/fux-index/`. Filed by Arpit (Cowork). [detail](open/W-34-fux-index-packages.md)

🟡 **W-77** waits on **W-88** `agent` Skills package built (`@codelaw/skills` / `codelaw-skills` at codelaw's exact version; mismatch → exit 2); left: its first publish and trusted publishers (DoD 6), then close. Filed by Arpit (Cowork). [detail](open/W-77-skills-package.md)

🟡 **W-64** waits on **W-38** `agent` `codelaw-pre-commit` mirror repo: the pre-commit framework installs codelaw from PyPI by version, as ruff-pre-commit does. Filed by Arpit (Cowork). [detail](open/W-64-codelaw-pre-commit-mirror.md)

🟢 **W-116** `agent` Release 0.3.0 through CI (ruled 10-29): `publish.yml` skips PyPI projects that do not exist yet; bump, green CI, then Arpit's `gh release create v0.3.0`; npm everything, PyPI `codelaw` and six packs. Filed by Arpit (Cowork). [detail](open/W-116-release-0-3-0.md)

🔴 **W-117** `arpit` Open VSX: the `vscode` environment, an Eclipse-linked login, the token and the `codelaw` namespace (inbox); then the `publish.yml` upload. Filed by Arpit (Cowork). [detail](open/W-117-open-vsx-setup.md)

🟢 **W-115** `agent` The spell pack's first publish: `@codelaw/rules-spell` and `codelaw-rules-spell` exist on neither registry; at the next release, take them from `dist` and put the by-hand commands in the inbox. Filed by Arpit (Cowork). [detail](open/W-115-spell-pack-first-publish.md)

🟡 **W-106** waits on **W-105** `agent` TypeScript version watch: tested range, CI (7.1 blocking, `typescript@next` nightly), a bump job that opens a PR or an issue. Filed by Arpit (Cowork). [detail](open/W-106-typescript-version-watch.md)

### testing (1)

🟡 **W-102** waits on **W-118** `agent` Pilot run 10-28: 60 sessions, $10.47; A = B = C at 2/20. Ruled 10-29: redesign first (W-118's research), then the run, the write-up and the README line. Filed by Arpit (Cowork). [detail](open/W-102-guidance-vs-check-experiment.md)

### record update (0)

### research (1)

🟢 **W-118** `cowork` Research: why W-102's pilot could not separate the arms, a redesign (unrequested temptations, a no-codelaw baseline), its downsides and open questions — a proposal for the decider. Filed by Arpit (Cowork). [detail](open/W-118-guidance-experiment-redesign.md)

---
