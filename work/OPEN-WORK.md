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
| 🔴 **Side-by-side bench — ratify these 5 tools for CI-only bench use** ([W-99](open/W-99-side-by-side-bench.md); [L11](../records/0013_LAW-11-dependencies.md) §3). Installed only by `scripts/bench-rivals.py install` inside `.github/workflows/bench-rivals.yml` (weekly and by hand, a throwaway folder on `ubuntu-24.04`); never in the engine, a package, a test or `ci.yml`. **D-37 ESLint** `eslint@10.13.0` + `@eslint/js@10.0.1` — MIT, 78 packages (54 MIT, 12 Apache-2.0, 6 BSD-2-Clause, 4 ISC, 1 BSD-3-Clause, 1 **BlueOak-1.0.0** = `minimatch`, OSI-approved permissive but not one of L2's named examples) · **D-38 Prettier** `prettier@3.9.10` — MIT, no dependencies · **D-39 Biome** `@biomejs/biome@2.6.3` — MIT OR Apache-2.0, + one platform binary · **D-40 oxlint** `oxlint@1.89.0` — MIT, + one platform binary · **D-41 Ruff** `ruff==0.17.3` — MIT, one wheel, no dependencies. Say **"ratify D-37 to D-41"**, or name the ones to strike. | 2026-10-27 |
| ↳ **blocks:** W-99 itself (the first real run and step 3, the README's speed claim); W-100's source-tool agreement | |
| 🔴 **`hook_pre`'s budget — the guard held it back** ([W-93](open/W-93-hook-pre-and-brief.md); [SR-LAW-8](../records/0010_LAW-8-performance-budgets.md) decision 4). Three `perf` runs on `main` (`ubuntu-24.04`): p95 **62.4 / 78.3 / 86.2 ms**; worst 86.2 × 1.5 = **129.3 > the 100 ms target**, so the method leaves it unset and asks you. Say **"leave it unset"** (as you ruled for `fix_one_file`, 2026-10-25), or **"set 100 ms"** (1.16 × headroom over the worst run). | 2026-10-27 | 0 d |
| ↳ **blocks:** W-93 itself | |
| 🔴 **Guidance-vs-check experiment — approve running it** ([W-102](open/W-102-guidance-vs-check-experiment.md)). 180 Claude Code sessions (arms A, B, C × 20 tasks × 3 runs) of `claude-opus-5-5`; estimate per session ~250k–450k input tokens (mostly cache reads), ~30k–50k cache writes, ~6k–9k output → **~$86 in all (range ~$45–$170)**, 9–18 hours one at a time; disk a few MB. **1. Name the agent CLI** — proposed: Claude Code 2.1, `claude -p … --setting-sources project --strict-mcp-config` (`experiment.toml` `[agent.claude]`; `--bare` instead if you run on `ANTHROPIC_API_KEY`). **2. Allow `fux` (D-34) in arm C outside CI** — its ratified scope is the CI build of the index packages. **3. Run it** from the main checkout: `python3 experiments/guidance-vs-check/harness.py run --agent claude --out experiments/guidance-vs-check/runs/2026-10-w102` (the same command resumes; `harness.py plan` reprints the estimate; `--arms A,B,C,D` adds the optional check arm, 60 sessions, ~$22). Say **"approve running it"** (and which CLI), or name a cheaper model or fewer runs. | 2026-10-27 |
| ↳ **blocks:** W-102 itself (the runs, the write-up, step 4's README line); W-35's line on it | |
| 🔴 **oxlint type-aware — a live capture** ([W-107](../archive/open/W-107-oxlint-type-aware-gap.md), closed on a fixture built from oxlint's documented JSON). Nothing here installs oxlint (L11), so run it in your own shell, outside the repo: `d=$(mktemp -d) && cp -R crates/codelaw-adapters/tests/fixtures/oxlint-type-aware/. "$d" && cd "$d" && npm init -y >/dev/null && npm add -D oxlint oxlint-tsgolint typescript@7 && printf '{"compilerOptions":{"strict":true,"target":"es2022","module":"nodenext"}}' > tsconfig.json && npx oxlint --format=json src/index.ts > stdout.json; npx oxlint --version` — then hand `stdout.json` and the version to an agent, which swaps it in (`@ROOT@` for the folder) and checks the test still holds. Or say **"skip it"**: the fixture stands. | 2026-10-27 |
| ↳ **blocks:** nothing else in the queue | |

---

## Open items (19)

### build (17)

🔴 **W-86** `arpit` PyPI day 1 (Wed 10-28): upload `codelaw-rules-{html,js,python,react}` 0.2.0, add their trusted publishers, then `gh release create v0.2.0` brings `codelaw` itself. Command in the inbox. Filed by Arpit (Cowork). [detail](open/W-86-pypi-day-1.md)

🔴 **W-87** `arpit` PyPI day 2 (Thu 10-29): the `react`, `scss`, `python` and `security-js` Fux index projects (0.2.0), then their trusted publishers. Command in the inbox. Filed by Arpit (Cowork). [detail](open/W-87-pypi-day-2.md)

🔴 **W-88** `arpit` PyPI day 3 (Fri 10-30): `security-html`, `security-python` Fux index, `codelaw-skills` and `codelaw-rules-scss`, all 0.2.0; then their trusted publishers. Command in the inbox. Filed by Arpit (Cowork). [detail](open/W-88-pypi-day-3.md)

🔴 **W-89** `arpit` PyPI day 4 (Sat 10-31): `codelaw-rules-security-{js,html,python}` 0.2.0, then their trusted publishers. Command in the inbox. Filed by Arpit (Cowork). [detail](open/W-89-pypi-day-4.md)

🔴 **W-95** `arpit` VS Code extension: eight platform `.vsix` built and checked in CI (`scripts/vsix.py`, artifact `vscode`), installs in VS Code; left: Arpit's publisher, namespace, tokens and the upload-tool ruling (inbox), then the `publish.yml` job. Filed by Arpit (Cowork). [detail](open/W-95-publish-vscode-extension.md)

🔴 **W-97** `arpit` GitHub Action built (`action.yml`, CI's `action` job, `init --ci github`); left: your three calls in the inbox — SARIF route, the release-tag proof, Marketplace and `v0`. Filed by Arpit (Cowork). [detail](open/W-97-github-action.md)

🔴 **W-99** `arpit` Side-by-side bench built (`scripts/bench-rivals.py`, `perf/rivals.toml`, `bench-rivals.yml`), skipped until L11 ratifies the rivals; dry run with codelaw alone green. Left: ratify D-37…D-41 (inbox), the first runner run, L8 rows. Filed by Arpit (Cowork). [detail](open/W-99-side-by-side-bench.md)

🟢 **W-62** `agent` SCSS built: Biome formats it, 64 `css/*` rules run on `.scss`, `scss` pack; both grammars patched (D-36), Bootstrap's SCSS clean and in the pinned corpus; left: CSS rules onto trees, `&` in no-descending-specificity. Filed by Arpit (Cowork). [detail](open/W-62-scss-support.md)

🔴 **W-93** `arpit` `codelaw hook --pre` and `codelaw brief` built and landed (records, code, tests, the Claude/Cowork/Codex pre-write hooks, README and USAGE); left: the decider rules `hook_pre`'s budget (inbox), then close. Filed by Arpit (Cowork). [detail](open/W-93-hook-pre-and-brief.md)

🟢 **W-103** `agent` Spell pack (`@codelaw/rules-spell` / `codelaw-rules-spell`): known-typo check on (auto-fix), dictionary opt-in; own word data from SCOWL, VarCon, software terms (D-42–D-44). SR-SPELL written; build, then measure. Filed by Arpit (Cowork). [detail](open/W-103-spell-pack.md)

🔴 **W-100** `arpit` Rule promotion: `scripts/corpus.py promote` reports which preview rules pass, in CI's corpus summary; `--write` refuses until ratified. Left: Arpit's ruling on the criterion (SR-RULE-AUTHORING 6a), then the first batch. Filed by Arpit (Cowork). [detail](open/W-100-rule-promotion.md)

🟡 **W-38** waits on **W-86** `agent` PyPI 0.2.0: `css`, `data` live and trusted; day 1 uploads the other four and publishes the Release; then an agent checks `pip install codelaw` and `uvx codelaw` and writes it down. Filed by Arpit (Cowork). [detail](open/W-38-pypi-0-2-0-by-hand.md)

🟡 **W-34** waits on **W-87** and **W-88** `agent` Fux index packages built; D-34 ratified 2026-10-27 (W-91); npm 15/15 and PyPI 4/10 live; after the publishing items, `publish.yml` uploads `dist/fux-index/`. Filed by Arpit (Cowork). [detail](open/W-34-fux-index-packages.md)

🟡 **W-77** waits on **W-88** `agent` Skills package built (`@codelaw/skills` / `codelaw-skills` at codelaw's exact version; mismatch → exit 2); left: its first publish and trusted publishers (DoD 6), then close. Filed by Arpit (Cowork). [detail](open/W-77-skills-package.md)

🟡 **W-64** waits on **W-38** `agent` `codelaw-pre-commit` mirror repo: the pre-commit framework installs codelaw from PyPI by version, as ruff-pre-commit does. Filed by Arpit (Cowork). [detail](open/W-64-codelaw-pre-commit-mirror.md)


🟡 **W-105** waits on **W-104** `agent` Type-aware rules: `codelaw-types` Node helper on the project's TypeScript 7.1 API, `type:` clause, warm programs, `ts/type-error`, first rules in `ts` — preview on the 7.1 RC (11-10), stable after GA (11-24). Filed by Arpit (Cowork). [detail](open/W-105-type-aware-engine.md)

🟡 **W-106** waits on **W-105** `agent` TypeScript version watch: tested range, CI (7.1 blocking, `typescript@next` nightly), a bump job that opens a PR or an issue. Filed by Arpit (Cowork). [detail](open/W-106-typescript-version-watch.md)

### testing (1)

🔴 **W-102** `arpit` Experiment built in `experiments/guidance-vs-check/` (20 tasks, arms A–C, scorer, table; fake-agent dry run 180/180; arm D on `hook --pre`); left: Arpit approves ~$86 of runs and the agent CLI (inbox), then the runs and the README line. Filed by Arpit (Cowork). [detail](open/W-102-guidance-vs-check-experiment.md)

### record update (1)

🟢 **W-104** `agent` Records for type-aware rules (W-101): amend L3, L5, L6, L8; write SR-TYPES; the `type:` clause; the decider confirms L11 sees no dependency. Filed by Arpit (Cowork). [detail](open/W-104-type-aware-records.md)

### research (0)

---
