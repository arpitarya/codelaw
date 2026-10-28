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
| 🔴 **PyPI day 1 — the Release** — the four 0.2.0 uploads landed 2026-10-28 (`codelaw-rules-{html,js,python,react}`). Their trusted publishers added 2026-10-28 (Claude in Chrome). Left: the Release, which brings `codelaw` itself: `awk '/^## 0\.2\.0/{f=1;next} /^## /{f=0} f' CHANGELOG.md > /tmp/v0.2.0-notes.md && gh release create v0.2.0 -R arpitarya/codelaw --verify-tag --title "v0.2.0 — the rule packs, one per ecosystem" --notes-file /tmp/v0.2.0-notes.md` — [W-86](open/W-86-pypi-day-1.md) | 2026-10-27 |
| ↳ **blocks:** W-86 itself; W-38 waits on it (and W-64 through W-38) | |
| 🔴 **PyPI day 2 — Thu 2026-10-29** (four Fux index projects, 0.2.0, already in `dist/fux-index/pypi/`): `twine upload --skip-existing dist/fux-index/pypi/codelaw_rules_{react,scss,python,security_js}_fux_index-0.2.0-py3-none-any.whl` — then a trusted publisher on each — [W-87](open/W-87-pypi-day-2.md) | 2026-10-27 |
| ↳ **blocks:** W-87 itself; W-34 waits on it | |
| 🔴 **PyPI day 3 — Fri 2026-10-30** (four projects at 0.2.0 — the version npm got: two Fux index wheels in `dist/fux-index/pypi/`, and `codelaw-skills`, `codelaw-rules-scss` from the build you published npm from, copied into `dist/pypi/` first): `twine upload --skip-existing dist/fux-index/pypi/codelaw_rules_security_{html,python}_fux_index-0.2.0-py3-none-any.whl dist/pypi/codelaw_skills-0.2.0-py3-none-any.whl dist/pypi/codelaw_rules_scss-0.2.0-py3-none-any.whl` — then a trusted publisher on each — [W-88](open/W-88-pypi-day-3.md) | 2026-10-27 |
| ↳ **blocks:** W-88 itself; W-34 and W-77 wait on it | |
| 🔴 **PyPI day 4 — Sat 2026-10-31** (the three security rule packs at 0.2.0, from the same build, copied into `dist/pypi/` first): `twine upload --skip-existing dist/pypi/codelaw_rules_security_{js,html,python}-0.2.0-py3-none-any.whl` — then a trusted publisher on each — [W-89](open/W-89-pypi-day-4.md) | 2026-10-27 |
| ↳ **blocks:** W-89 itself | |
| 🔴 **VS Code extension — publisher, namespace and tokens** ([W-95](open/W-95-publish-vscode-extension.md) step 2; the upload is `curl` from `publish.yml`, no tool — ruled 2026-10-27). **1. Environment** `vscode`, tag rule `v*` like `npm`/`pypi`: `gh api -X PUT repos/arpitarya/codelaw/environments/vscode -F 'deployment_branch_policy[protected_branches]=false' -F 'deployment_branch_policy[custom_branch_policies]=true' && gh api -X POST repos/arpitarya/codelaw/environments/vscode/deployment-branch-policies -f name='v*' -f type=tag`. **2. Marketplace publisher:** <https://marketplace.visualstudio.com/manage> → sign in (Microsoft account) → *Create publisher* → ID **`codelaw`** (it must equal `publisher` in `packaging/vscode/package.json`; if taken, tell the agent the ID you got), name `codelaw`. **3. Marketplace token:** <https://dev.azure.com> (create an organization if asked; any name) → *User settings* (top right) → *Personal access tokens* → *New Token* → name `codelaw-vscode-publish`, Organization **All accessible organizations** (the Marketplace refuses a one-organization PAT; if that option is gone, stop and say so — the alternative is an Entra ID credential, another ruling), expiry 1 year, Scopes *Custom defined* → *Show all scopes* → **Marketplace: Manage** → *Create*, copy it → `gh secret set VSCE_PAT --env vscode -R arpitarya/codelaw` (paste at the prompt). **4. Open VSX:** <https://open-vsx.org> → *Log in* (GitHub) → avatar → *Settings* → *Log in with Eclipse* (an <https://accounts.eclipse.org> account with your GitHub username in its profile) → sign the *Publisher Agreement* → *Access Tokens* → *Generate New Token* `codelaw-publish`, copy it → `gh secret set OVSX_PAT --env vscode -R arpitarya/codelaw`; then the namespace: `read -rs T && curl -fsS -X POST -H 'Content-Type: application/json' -d '{"name":"codelaw"}' "https://open-vsx.org/api/-/namespace/create?token=$T"` (optional, the verified badge: *Claim namespace ownership* at <https://github.com/EclipseFdn/open-vsx.org/issues/new/choose>). | 2026-10-27 |
| ↳ **blocks:** W-95 itself (step 3, the `publish.yml` job, and step 4) | |
| 🔴 **GitHub Action — the Marketplace listing** ([W-97](open/W-97-github-action.md)): on the next release (v0.3.0), github.com/marketplace/actions → *Publish* on the release's edit page → accept the Marketplace Developer Agreement → tick *Publish this Action to the GitHub Marketplace*. Ruled 2026-10-27: SARIF stays on `gh`; a moving `v0` tag is kept; Claude runs the throwaway-repo proof after that tag. | 2026-10-27 |
| ↳ **blocks:** W-97 itself | |
| 🔴 **Two leftover worktrees — keep or drop** ([W-111](open/W-111-two-unmerged-worktrees.md); [SR-WORK-CLEANUP](../records/0062_WORK-cleanup.md) decision 3). `lm/W-90` and `lm/W-96` hold commits `main` lacks, though both items closed and their work is in `main` (`archive/DEPENDENCIES.md`, `agent_env.rs`). Say **"drop both"** (recommended), or name one to keep. | 2026-10-28 |
| ↳ **blocks:** W-111 itself | |

---

## Open items (18)

### build (16)

🔴 **W-86** `arpit` PyPI day 1: the four 0.2.0 uploads and their trusted publishers landed 10-28; left: `gh release create v0.2.0` (inbox), which brings `codelaw` itself. Filed by Arpit (Cowork). [detail](open/W-86-pypi-day-1.md)

🔴 **W-87** `arpit` PyPI day 2 (Thu 10-29): the `react`, `scss`, `python` and `security-js` Fux index projects (0.2.0), then their trusted publishers. Command in the inbox. Filed by Arpit (Cowork). [detail](open/W-87-pypi-day-2.md)

🔴 **W-88** `arpit` PyPI day 3 (Fri 10-30): `security-html`, `security-python` Fux index, `codelaw-skills` and `codelaw-rules-scss`, all 0.2.0; then their trusted publishers. Command in the inbox. Filed by Arpit (Cowork). [detail](open/W-88-pypi-day-3.md)

🔴 **W-89** `arpit` PyPI day 4 (Sat 10-31): `codelaw-rules-security-{js,html,python}` 0.2.0, then their trusted publishers. Command in the inbox. Filed by Arpit (Cowork). [detail](open/W-89-pypi-day-4.md)

🔴 **W-95** `arpit` VS Code extension: eight `.vsix` built and checked in CI; upload ruled 10-27: `curl` from `publish.yml`, no tool. Left: Arpit's publisher, namespace and tokens (inbox), then the `publish.yml` job. Filed by Arpit (Cowork). [detail](open/W-95-publish-vscode-extension.md)

🔴 **W-97** `arpit` GitHub Action built; ruled 10-27: SARIF via `gh`, a moving `v0`. Left: the Marketplace agreement on the v0.3.0 release (inbox), then Claude's throwaway-repo proof. Filed by Arpit (Cowork). [detail](open/W-97-github-action.md)

🟢 **W-109** `agent` Bitbucket ruled 10-28: keep only the Code Insights output (`--format bitbucket`), it and SARIF switchable; remove `init --ci bitbucket`, its template and the pipe fork. Then the live proof after v0.3.0. Filed by Arpit (Cowork). [detail](open/W-109-bitbucket-pipelines.md)

🟢 **W-62** `agent` SCSS built (Biome formats it, `scss` pack, both grammars patched, Bootstrap in corpus); CSS/SCSS rules read statements, selectors, values, preludes, media queries from the tree; left: pseudo-class args, keyframe selectors, font lists. Filed by Arpit (Cowork). [detail](open/W-62-scss-support.md)

🟢 **W-103** `agent` Spell pack: typo list, dictionary, locale pairs, `[spell]`, six rules, fixtures and the corpus run (reviewed, all `preview`) landed; left: `spell/inconsistent-term` (project scope), the L8 budget. Filed by Arpit (Cowork). [detail](open/W-103-spell-pack.md)

🟢 **W-110** `agent` Clean session 2df06f6d's leftovers on the Mac: five merged worktrees and three `.fux`-only folders, by `worktree-cleanup.py --apply`. Filed by Arpit (Cowork). [detail](open/W-110-session-2df06f6d-cleanup.md)

🟢 **W-105** `agent` Type-aware rules: `codelaw-types` Node helper on the project's TypeScript 7.1 API, `type:` clause, warm programs, `ts/type-error`, first rules in `ts` — preview on the 7.1 RC (11-10), stable after GA (11-24). Filed by Arpit (Cowork). [detail](open/W-105-type-aware-engine.md)

🟡 **W-38** waits on **W-86** `agent` PyPI 0.2.0: `css`, `data` live and trusted; day 1 uploads the other four and publishes the Release; then an agent checks `pip install codelaw` and `uvx codelaw` and writes it down. Filed by Arpit (Cowork). [detail](open/W-38-pypi-0-2-0-by-hand.md)

🟡 **W-34** waits on **W-87** and **W-88** `agent` Fux index packages built; D-34 ratified 2026-10-27 (W-91); npm 15/15 and PyPI 4/10 live; after the publishing items, `publish.yml` uploads `dist/fux-index/`. Filed by Arpit (Cowork). [detail](open/W-34-fux-index-packages.md)

🟡 **W-77** waits on **W-88** `agent` Skills package built (`@codelaw/skills` / `codelaw-skills` at codelaw's exact version; mismatch → exit 2); left: its first publish and trusted publishers (DoD 6), then close. Filed by Arpit (Cowork). [detail](open/W-77-skills-package.md)

🟡 **W-64** waits on **W-38** `agent` `codelaw-pre-commit` mirror repo: the pre-commit framework installs codelaw from PyPI by version, as ruff-pre-commit does. Filed by Arpit (Cowork). [detail](open/W-64-codelaw-pre-commit-mirror.md)


🟡 **W-106** waits on **W-105** `agent` TypeScript version watch: tested range, CI (7.1 blocking, `typescript@next` nightly), a bump job that opens a PR or an issue. Filed by Arpit (Cowork). [detail](open/W-106-typescript-version-watch.md)

### testing (1)

🟢 **W-102** `agent` Experiment approved 10-27: Claude Code CLI, Fux in arm C. Left: the pilot (`--runs 1`, ~$29) on the Mac, then the full 180, the write-up and the README line. Filed by Arpit (Cowork). [detail](open/W-102-guidance-vs-check-experiment.md)

### record update (1)

🔴 **W-111** `arpit` Two unmerged worktrees of session 2df06f6d — `lm/W-90`, `lm/W-96` — keep or drop (inbox); both items closed, their work in `main`. Filed by Arpit (Cowork). [detail](open/W-111-two-unmerged-worktrees.md)

### research (0)

---
