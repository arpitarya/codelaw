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

## Open items (18)

### build (16)

🔴 **W-86** `arpit` PyPI day 1 (Wed 10-28): upload `codelaw-rules-{html,js,python,react}` 0.2.0, add their trusted publishers, then `gh release create v0.2.0` brings `codelaw` itself. Command in the inbox. Filed by Arpit (Cowork). [detail](open/W-86-pypi-day-1.md)

🔴 **W-87** `arpit` PyPI day 2 (Thu 10-29): the `react`, `scss`, `python` and `security-js` Fux index projects (0.2.0), then their trusted publishers. Command in the inbox. Filed by Arpit (Cowork). [detail](open/W-87-pypi-day-2.md)

🔴 **W-88** `arpit` PyPI day 3 (Fri 10-30): `security-html`, `security-python` Fux index, `codelaw-skills` and `codelaw-rules-scss`, all 0.2.0; then their trusted publishers. Command in the inbox. Filed by Arpit (Cowork). [detail](open/W-88-pypi-day-3.md)

🔴 **W-89** `arpit` PyPI day 4 (Sat 10-31): `codelaw-rules-security-{js,html,python}` 0.2.0, then their trusted publishers. Command in the inbox. Filed by Arpit (Cowork). [detail](open/W-89-pypi-day-4.md)

🟢 **W-85** `agent` npm: the nine new packages live at 0.2.0 and all 24 untrusted names now trust `publish.yml` (2026-10-27); left: `npm-trust.sh` treats E409 as trusted, and re-check the pending ones before the next release. Filed by Arpit (Cowork). [detail](open/W-85-npm-first-publish.md)

🟢 **W-62** `agent` SCSS built: L3/L6 amended, Biome formats it, 64 `css/*` rules run on `.scss`, new `scss` pack; `tree-sitter-cli` 0.25.10 ratified (D-36, 2026-10-27); left: grammar patches, CSS rules onto trees, corpus. Filed by Arpit (Cowork). [detail](open/W-62-scss-support.md)

🟢 **W-90** `agent` L11 renamed "dependencies" and holds the list: `DEPENDENCIES.md`'s tables move into it, the file goes to `archive/` with a banner, every reference (scripts, tests, records) repointed. Filed by Arpit (Cowork). [detail](open/W-90-l11-holds-the-dependency-list.md)

🟢 **W-91** `agent` fux-engine always at its latest: a scheduled CI job finds a new release, bumps `FUX_VERSION` and opens a PR; each commit stays pinned and reproducible (D-34, ratified 2026-10-27). Filed by Arpit (Cowork). [detail](open/W-91-fux-auto-bump.md)

🟢 **W-71** `agent` Tests for new or edited functions (JS, TS, Python), diff-scoped; steps 1–6 built (`graph/untested-function`, lcov `graph/function-coverage`, `.codelawignore`, `codelaw:tests`); left: CI's `perf` run with a change set (DoD 10), after it lands. Filed by Arpit (Cowork). [detail](open/W-71-tests-for-new-code.md)

🟢 **W-44** `agent` `check --watch` / `fix --watch` built (polling, ruled 2026-10-26); the watch step passed on both `win32` smoke legs (run 40388001095); left: DoD 5 confirmed on a green `smoke` run (W-84) and the CI-runner measurements. Filed by Arpit (Cowork). [detail](open/W-44-watch-mode.md)

🟢 **W-70** `agent` Dev-server example apps on real Vite, webpack, Rspack, Next.js. CI run 40388001095: vite, next-turbopack pass; the other three fixed, and vite's `Origin` refusal (`f419f32`); left: a green `dev-servers` run (W-84), close. Filed by Arpit (chat). [detail](open/W-70-dev-server-example-apps.md)

🟡 **W-38** waits on **W-86** `agent` PyPI 0.2.0: `css`, `data` live and trusted; day 1 uploads the other four and publishes the Release; then an agent checks `pip install codelaw` and `uvx codelaw` and writes it down. Filed by Arpit (Cowork). [detail](open/W-38-pypi-0-2-0-by-hand.md)

🟡 **W-34** waits on **W-87** and **W-88** `agent` Fux index packages built; D-34 ratified 2026-10-27 (W-91); npm 15/15 and PyPI 4/10 live; after the publishing items, `publish.yml` uploads `dist/fux-index/`. Filed by Arpit (Cowork). [detail](open/W-34-fux-index-packages.md)

🟡 **W-77** waits on **W-88** `agent` Skills package built (`@codelaw/skills` / `codelaw-skills` at codelaw's exact version; mismatch → exit 2); left: its first publish and trusted publishers (DoD 6), then close. Filed by Arpit (Cowork). [detail](open/W-77-skills-package.md)

🟡 **W-64** waits on **W-38** `agent` `codelaw-pre-commit` mirror repo: the pre-commit framework installs codelaw from PyPI by version, as ruff-pre-commit does. Filed by Arpit (Cowork). [detail](open/W-64-codelaw-pre-commit-mirror.md)

🟡 **W-35** waits on **W-05** `agent` README leads with "your rules, before, while and after the AI writes"; then it drops While's "in design" and restores Before's "handful". Filed by Arpit (Cowork). [detail](open/W-35-before-while-after-positioning.md)

### testing (0)

### record update (0)

### research (2)

🟢 **W-36** `cowork` Research codelaw as the linter and formatter for AI — before, while, after and with no AI — beside ESLint + Prettier, Biome, Ruff, oxlint: gaps, provable claims, who adopts it. Filed by Arpit (Cowork). [detail](open/W-36-position-against-eslint-and-biome.md)

🟢 **W-05** `cowork` Research guiding an agent **while** it writes: a `brief` of the rules for a path and construct, and a `hook --pre` check before a write reaches disk; Fux as an optional layer. Filed by Arpit (Cowork). [detail](open/W-05-write-time-guidance.md)

---
