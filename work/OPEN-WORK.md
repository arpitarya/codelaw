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

---

## Open items (21)

### build (19)

🔴 **W-86** `arpit` PyPI day 1 (Wed 10-28): upload `codelaw-rules-{html,js,python,react}` 0.2.0, add their trusted publishers, then `gh release create v0.2.0` brings `codelaw` itself. Command in the inbox. Filed by Arpit (Cowork). [detail](open/W-86-pypi-day-1.md)

🔴 **W-87** `arpit` PyPI day 2 (Thu 10-29): the `react`, `scss`, `python` and `security-js` Fux index projects (0.2.0), then their trusted publishers. Command in the inbox. Filed by Arpit (Cowork). [detail](open/W-87-pypi-day-2.md)

🔴 **W-88** `arpit` PyPI day 3 (Fri 10-30): `security-html`, `security-python` Fux index, `codelaw-skills` and `codelaw-rules-scss`, all 0.2.0; then their trusted publishers. Command in the inbox. Filed by Arpit (Cowork). [detail](open/W-88-pypi-day-3.md)

🔴 **W-89** `arpit` PyPI day 4 (Sat 10-31): `codelaw-rules-security-{js,html,python}` 0.2.0, then their trusted publishers. Command in the inbox. Filed by Arpit (Cowork). [detail](open/W-89-pypi-day-4.md)

🟢 **W-62** `agent` SCSS built: L3/L6 amended, Biome formats it, 64 `css/*` rules run on `.scss`, new `scss` pack; `tree-sitter-cli` 0.25.10 ratified (D-36, 2026-10-27); left: grammar patches, CSS rules onto trees, corpus. Filed by Arpit (Cowork). [detail](open/W-62-scss-support.md)

🟢 **W-91** `agent` fux-engine at its latest: `fux-bump.yml` (daily) bumps `FUX_VERSION`, builds, checks and opens or updates one PR, dispatching `ci.yml` on it; the Actions PR setting is on (2026-10-27); left: a live PR and its re-run. Filed by Arpit (Cowork). [detail](open/W-91-fux-auto-bump.md)

🟢 **W-93** `agent` `codelaw hook --pre` (refuse a write that adds an error, for every agent with a pre-write hook, fail-open) and `codelaw brief` (only what is specific to the repo); records first. Filed by Arpit (Cowork). [detail](open/W-93-hook-pre-and-brief.md)

🟢 **W-94** `agent` `codelaw migrate eslint|prettier|biome|ruff|stylelint`: translate a config into `codelaw.toml` with a report; accept `eslint-disable`/`noqa` as ignore aliases. Filed by Arpit (Cowork). [detail](open/W-94-codelaw-migrate.md)

🟢 **W-95** `agent` Publish the VS Code extension on the Marketplace and Open VSX; Arpit creates the publisher and tokens at step 2. Filed by Arpit (Cowork). [detail](open/W-95-publish-vscode-extension.md)

🟢 **W-96** `agent` Pick `--format agent` automatically when an agent's environment variable is set; an explicit `--format` wins. Filed by Arpit (Cowork). [detail](open/W-96-agent-format-autodetect.md)

🟢 **W-97** `agent` A first-party GitHub Action (`uses: arpitarya/codelaw@v…`): install, `check --format github`, optional SARIF upload. Filed by Arpit (Cowork). [detail](open/W-97-github-action.md)

🟢 **W-98** `agent` `llms.txt`, `llms-full.txt` and a `rules.json` catalogue generated from `docs/rules/`, checked in CI. Filed by Arpit (Cowork). [detail](open/W-98-llms-txt.md)

🟢 **W-99** `agent` Side-by-side bench against ESLint, Prettier, Biome, oxlint, Ruff on the pinned repos and runner; no speed claim before it. Filed by Arpit (Cowork). [detail](open/W-99-side-by-side-bench.md)

🟢 **W-100** `agent` Promote rules from `preview` to `stable` by pinned corpus runs, with a recorded criterion. Filed by Arpit (Cowork). [detail](open/W-100-rule-promotion.md)

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
