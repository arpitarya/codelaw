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
| 🔴 **PyPI day 1 — Wed 2026-10-28** (four new projects; the 0.2.0 wheels already in `dist/pypi/`, from the v0.2.0 tag): `twine upload --skip-existing dist/pypi/codelaw_rules_{html,js,python,react}-0.2.0-py3-none-any.whl` — then a trusted publisher on each (`arpitarya` · `codelaw` · `publish.yml` · `pypi`; or ask Claude to add them) — then the Release, which brings `codelaw` itself: `awk '/^## 0\.2\.0/{f=1;next} /^## /{f=0} f' CHANGELOG.md > /tmp/v0.2.0-notes.md && gh release create v0.2.0 -R arpitarya/codelaw --verify-tag --title "v0.2.0 — the rule packs, one per ecosystem" --notes-file /tmp/v0.2.0-notes.md` — [W-86](open/W-86-pypi-day-1.md) | 2026-10-27 |
| ↳ **blocks:** W-86 itself; W-38 waits on it (and W-64 through W-38) | |
| 🔴 **PyPI day 2 — Thu 2026-10-29** (four Fux index projects, 0.2.0, already in `dist/fux-index/pypi/`): `twine upload --skip-existing dist/fux-index/pypi/codelaw_rules_{react,scss,python,security_js}_fux_index-0.2.0-py3-none-any.whl` — then a trusted publisher on each — [W-87](open/W-87-pypi-day-2.md) | 2026-10-27 |
| ↳ **blocks:** W-87 itself; W-34 waits on it | |
| 🔴 **PyPI day 3 — Fri 2026-10-30** (four projects at 0.2.0 — the version npm got: two Fux index wheels in `dist/fux-index/pypi/`, and `codelaw-skills`, `codelaw-rules-scss` from the build you published npm from, copied into `dist/pypi/` first): `twine upload --skip-existing dist/fux-index/pypi/codelaw_rules_security_{html,python}_fux_index-0.2.0-py3-none-any.whl dist/pypi/codelaw_skills-0.2.0-py3-none-any.whl dist/pypi/codelaw_rules_scss-0.2.0-py3-none-any.whl` — then a trusted publisher on each — [W-88](open/W-88-pypi-day-3.md) | 2026-10-27 |
| ↳ **blocks:** W-88 itself; W-34 and W-77 wait on it | |
| 🔴 **PyPI day 4 — Sat 2026-10-31** (the three security rule packs at 0.2.0, from the same build, copied into `dist/pypi/` first): `twine upload --skip-existing dist/pypi/codelaw_rules_security_{js,html,python}-0.2.0-py3-none-any.whl` — then a trusted publisher on each — [W-89](open/W-89-pypi-day-4.md) | 2026-10-27 |
| ↳ **blocks:** W-89 itself | |
| 🔴 **VS Code extension — publisher, namespace, tokens, and the upload tool** ([W-95](open/W-95-publish-vscode-extension.md) step 2). **1. Environment** `vscode`, tag rule `v*` like `npm`/`pypi`: `gh api -X PUT repos/arpitarya/codelaw/environments/vscode -F 'deployment_branch_policy[protected_branches]=false' -F 'deployment_branch_policy[custom_branch_policies]=true' && gh api -X POST repos/arpitarya/codelaw/environments/vscode/deployment-branch-policies -f name='v*' -f type=tag`. **2. Marketplace publisher:** <https://marketplace.visualstudio.com/manage> → sign in (Microsoft account) → *Create publisher* → ID **`codelaw`** (it must equal `publisher` in `packaging/vscode/package.json`; if taken, tell the agent the ID you got), name `codelaw`. **3. Marketplace token:** <https://dev.azure.com> (create an organization if asked; any name) → *User settings* (top right) → *Personal access tokens* → *New Token* → name `codelaw-vscode-publish`, Organization **All accessible organizations** (the Marketplace refuses a one-organization PAT; if that option is gone, stop and say so — the alternative is an Entra ID credential, another ruling), expiry 1 year, Scopes *Custom defined* → *Show all scopes* → **Marketplace: Manage** → *Create*, copy it → `gh secret set VSCE_PAT --env vscode -R arpitarya/codelaw` (paste at the prompt). **4. Open VSX:** <https://open-vsx.org> → *Log in* (GitHub) → avatar → *Settings* → *Log in with Eclipse* (an <https://accounts.eclipse.org> account with your GitHub username in its profile) → sign the *Publisher Agreement* → *Access Tokens* → *Generate New Token* `codelaw-publish`, copy it → `gh secret set OVSX_PAT --env vscode -R arpitarya/codelaw`; then the namespace: `read -rs T && curl -fsS -X POST -H 'Content-Type: application/json' -d '{"name":"codelaw"}' "https://open-vsx.org/api/-/namespace/create?token=$T"` (optional, the verified badge: *Claim namespace ownership* at <https://github.com/EclipseFdn/open-vsx.org/issues/new/choose>). **5. Rule on the upload tool:** ratify `@vscode/vsce` 4.0.0 and `ovsx` 1.3.0 (licences, CI-only scope)? **L2 refuses both as they stand** — `ovsx` is EPL-2.0 (copyleft) and `vsce` brings `@vscode/vsce-sign` under Microsoft's proprietary licence — so (b) means amending L2 too. **Recommended (a):** no dependency — `publish.yml` uploads the built `.vsix` with `curl` to each registry's API. Say *"a"* or *"b"*. | 2026-10-27 |
| ↳ **blocks:** W-95 itself (step 3, the `publish.yml` job, and step 4) | |
| 🔴 **Rule promotion — ratify the criterion's numbers** ([W-100](open/W-100-rule-promotion.md); [SR-RULE-AUTHORING](../records/0113_rule-authoring.md) decision 6a). A preview rule becomes `stable` when **(1)** ≥ **3** of its pack's pinned corpus repos hold its files, ≥ **10** files in all; **(2)** no false positive — every finding reviewed by a person, and **0 findings counts as none** (no proof it fires); **(3)** its fixes held level 4 (already enforced); **(4)** ≥ **1** published release as preview. At these values 326 of 620 rules pass today, all through 0 findings; 190 with findings need a review; with "0 findings is not evidence", none passes yet. Source-tool agreement replaces the review once W-99's L11 ruling puts the tools in CI. Say **"ratify 6a as proposed"**, or give the numbers you want. | 2026-10-27 |
| ↳ **blocks:** W-100 itself (the first batch, its DoD) | |
| 🔴 **GitHub Action — three calls** ([W-97](open/W-97-github-action.md)): **(1) SARIF route** — keep the code-scanning REST API via `gh` (built; say *"keep gh"*), or ratify `github/codeql-action/upload-sarif` (MIT) as a new row in [L11 §3](../records/0013_LAW-11-dependencies.md#3--the-list) · **(2) the DoD proof** — after the next release tag (`gh release create v0.3.0 …`, RELEASING.md), create a throwaway public repo (`gh repo create arpitarya/codelaw-action-test --public --clone`), add `.github/workflows/codelaw.yml` from `codelaw init --ci github` with `sarif: true` and `permissions: security-events: write`, push one `debugger`, and check the annotation and Security → Code scanning; or ask Claude to do it · **(3) Marketplace + `v0`** — accept the Marketplace Developer Agreement (github.com/marketplace/actions → *Publish* on the v0.3.0 release's edit page → tick *Publish this Action to the GitHub Marketplace*); say whether a moving `v0` tag is kept (`git tag -f v0 v0.3.0 && git push -f origin v0` each release) or not | 2026-10-27 |
| ↳ **blocks:** W-97 itself | |

---

## Open items (18)

### build (16)

🔴 **W-86** `arpit` PyPI day 1 (Wed 10-28): upload `codelaw-rules-{html,js,python,react}` 0.2.0, add their trusted publishers, then `gh release create v0.2.0` brings `codelaw` itself. Command in the inbox. Filed by Arpit (Cowork). [detail](open/W-86-pypi-day-1.md)

🔴 **W-87** `arpit` PyPI day 2 (Thu 10-29): the `react`, `scss`, `python` and `security-js` Fux index projects (0.2.0), then their trusted publishers. Command in the inbox. Filed by Arpit (Cowork). [detail](open/W-87-pypi-day-2.md)

🔴 **W-88** `arpit` PyPI day 3 (Fri 10-30): `security-html`, `security-python` Fux index, `codelaw-skills` and `codelaw-rules-scss`, all 0.2.0; then their trusted publishers. Command in the inbox. Filed by Arpit (Cowork). [detail](open/W-88-pypi-day-3.md)

🔴 **W-89** `arpit` PyPI day 4 (Sat 10-31): `codelaw-rules-security-{js,html,python}` 0.2.0, then their trusted publishers. Command in the inbox. Filed by Arpit (Cowork). [detail](open/W-89-pypi-day-4.md)

🔴 **W-95** `arpit` VS Code extension: eight platform `.vsix` built and checked in CI (`scripts/vsix.py`, artifact `vscode`), installs in VS Code; left: Arpit's publisher, namespace, tokens and the upload-tool ruling (inbox), then the `publish.yml` job. Filed by Arpit (Cowork). [detail](open/W-95-publish-vscode-extension.md)

🔴 **W-97** `arpit` GitHub Action built (`action.yml`, CI's `action` job, `init --ci github`); left: your three calls in the inbox — SARIF route, the release-tag proof, Marketplace and `v0`. Filed by Arpit (Cowork). [detail](open/W-97-github-action.md)

🟢 **W-62** `agent` SCSS built: Biome formats it, 64 `css/*` rules run on `.scss`, `scss` pack; both grammars patched (D-36), Bootstrap's SCSS clean and in the pinned corpus; left: CSS rules onto trees, `&` in no-descending-specificity. Filed by Arpit (Cowork). [detail](open/W-62-scss-support.md)

🟢 **W-93** `agent` `codelaw hook --pre` (refuse a write that adds an error, for every agent with a pre-write hook, fail-open) and `codelaw brief` (only what is specific to the repo); records first. Filed by Arpit (Cowork). [detail](open/W-93-hook-pre-and-brief.md)

🟢 **W-94** `agent` `codelaw migrate eslint|prettier|biome|ruff|stylelint`: translate a config into `codelaw.toml` with a report; accept `eslint-disable`/`noqa` as ignore aliases. Filed by Arpit (Cowork). [detail](open/W-94-codelaw-migrate.md)


🟢 **W-99** `agent` Side-by-side bench against ESLint, Prettier, Biome, oxlint, Ruff on the pinned repos and runner; no speed claim before it. Filed by Arpit (Cowork). [detail](open/W-99-side-by-side-bench.md)

🔴 **W-100** `arpit` Rule promotion: `scripts/corpus.py promote` reports which preview rules pass, in CI's corpus summary; `--write` refuses until ratified. Left: Arpit's ruling on the criterion (SR-RULE-AUTHORING 6a), then the first batch. Filed by Arpit (Cowork). [detail](open/W-100-rule-promotion.md)

🟡 **W-38** waits on **W-86** `agent` PyPI 0.2.0: `css`, `data` live and trusted; day 1 uploads the other four and publishes the Release; then an agent checks `pip install codelaw` and `uvx codelaw` and writes it down. Filed by Arpit (Cowork). [detail](open/W-38-pypi-0-2-0-by-hand.md)

🟡 **W-34** waits on **W-87** and **W-88** `agent` Fux index packages built; D-34 ratified 2026-10-27 (W-91); npm 15/15 and PyPI 4/10 live; after the publishing items, `publish.yml` uploads `dist/fux-index/`. Filed by Arpit (Cowork). [detail](open/W-34-fux-index-packages.md)

🟡 **W-77** waits on **W-88** `agent` Skills package built (`@codelaw/skills` / `codelaw-skills` at codelaw's exact version; mismatch → exit 2); left: its first publish and trusted publishers (DoD 6), then close. Filed by Arpit (Cowork). [detail](open/W-77-skills-package.md)

🟡 **W-64** waits on **W-38** `agent` `codelaw-pre-commit` mirror repo: the pre-commit framework installs codelaw from PyPI by version, as ruff-pre-commit does. Filed by Arpit (Cowork). [detail](open/W-64-codelaw-pre-commit-mirror.md)

🟡 **W-35** waits on **W-93** `agent` README leads with "your rules, before, while and after the AI writes"; then it drops While's "in design" and restores Before's "handful". Filed by Arpit (Cowork). [detail](open/W-35-before-while-after-positioning.md)

### testing (1)

🟢 **W-102** `agent` Experiment: does an agent with Fux and the rule docs still break rules? 3 arms × 20 tasks × 3 runs; decides the README's "rules it can't skip" line. Filed by Arpit (Cowork). [detail](open/W-102-guidance-vs-check-experiment.md)

### record update (0)

### research (1)

🟢 **W-101** `cowork` Research: type-aware TypeScript rules reopened (B-049) — oxlint's type-aware linting is stable since 2026-07-22; route, cost, a pick. Filed by Arpit (Cowork). [detail](open/W-101-type-aware-ts-research.md)


---
