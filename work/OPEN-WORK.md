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
| 🔴 **PyPI 0.2.0 by hand, over 2–3 days** (four new projects a day): upload `codelaw-rules-{html,js,python,react}`, add six trusted publishers, then `gh release create v0.2.0` on W-37's tag — [W-38](open/W-38-pypi-0-2-0-by-hand.md) | 2026-10-20 |
| ↳ **blocks:** W-38 itself; W-64 waits on it | |
| 🔴 **Reserve and publish the 25 Fux index package names** (npm `@codelaw/rules-<name>-fux-index` ×15 by hand, then `npm-trust.sh`; PyPI `codelaw-rules-<name>-fux-index` ×10 by `twine`, then a trusted publisher each; built with fux 5.1.1) — built by CI as the `fux-index` artifact; RELEASING.md §"The optional Fux index packages"; also ratify `fux-engine` as D-34 in `DEPENDENCIES.md` — [W-34](open/W-34-fux-index-packages.md) | 2026-10-26 |
| ↳ **blocks:** W-34 itself | |

---

## Open items

### build (14)

🔴 **W-38** `arpit` PyPI 0.2.0 by hand over 2–3 days: `css`, `data` live; left the other four packs (four a day), six trusted publishers, then the Release on W-37's tag brings `codelaw`. Filed by Arpit (Cowork). [detail](open/W-38-pypi-0-2-0-by-hand.md)

🔴 **W-34** `arpit` Fux index packages built (L6 decision 12; `scripts/fux-index.py`, fux 5.1.1, CI, `init` prints not installs); left: reserve the 25 names and publish them by hand (npm fifteen, PyPI ten), then teach `publish.yml` to upload `dist/fux-index/`. Filed by Arpit (Cowork). [detail](open/W-34-fux-index-packages.md)

🟢 **W-71** `agent` Tests for new or edited functions (JS, TS, Python), diff-scoped; step 1 built (`debug changed-functions`); left: the hook's file, `graph/untested-function`, lcov `graph/function-coverage`, `.codelawignore`. Filed by Arpit (Cowork). [detail](open/W-71-tests-for-new-code.md)

🟢 **W-44** `agent` `check --watch` / `fix --watch` built (polling, ruled 2026-10-26); left: DoD 5 on CI's Windows legs and the CI-runner measurements. Filed by Arpit (Cowork). [detail](open/W-44-watch-mode.md)

🟢 **W-45** `agent` CI formats built; PR [#3](https://github.com/arpitarya/codelaw/pull/3) is open; left: Arpit looks at its annotations and says so (DoD 2). Filed by Arpit (Cowork). [detail](open/W-45-ci-report-formats.md)

🟢 **W-51** `agent` A scheduled auto-fix PR built (`init --ci github|gitlab --autofix daily|weekly [--unsafe]`, one PR, burn-down in the body); exceptions section done (W-46); left: a real GitHub run in a throwaway repo (DoD 1, 3). Filed by Arpit (Cowork). [detail](open/W-51-scheduled-autofix-pr.md)

🟢 **W-58** `agent` Size limits for JS, TS and Python built (six rules, `js/size` and `py/size`, measured value in JSON and SARIF); left: CI's `perf` run (DoD 6); the six corpus counts are in. Filed by Arpit (Cowork). [detail](open/W-58-size-limits.md)

🔴 **W-62** `arpit` SCSS built: L3/L6 amended, Biome formats it, 64 `css/*` rules run on `.scss`, new `scss` pack (15 rules, npm + PyPI), `init`; left: Arpit ratifies `tree-sitter-cli` 0.25.10 (L11), then grammar patches, CSS rules onto trees, corpus. Filed by Arpit (Cowork). [detail](open/W-62-scss-support.md)

🟢 **W-70** `agent` Example apps for the dev-server plugins on real Vite, webpack, Rspack, Next.js, via a CI job that installs the hosts (L11). Built; left: the first CI run on the real hosts, fix, close. Filed by Arpit (chat). [detail](open/W-70-dev-server-example-apps.md)

🟢 **W-73** `agent` Configurable approved calls: 5 HTML-sink rules take a `sanitizers` list (extend or replace the built-in; name + import source); `js/use-instead`, `py/use-instead` for wrappers. Filed by Arpit (Cowork). [detail](open/W-73-configurable-approved-calls.md)

🟢 **W-74** `agent` Editor logs you can debug from: the VS Code extension and `codelaw lsp` say what runs, which config each file got, why a file shows nothing, and every failure; `codelaw.logLevel`; *Copy debug info*. Filed by Arpit (Cowork). [detail](open/W-74-editor-logs.md)

🟢 **W-77** `agent` A separate package for everything agent-facing: `@codelaw/skills` (npm) and `codelaw-skills` (PyPI), pinned to the same codelaw version; codelaw itself keeps no agent files (ruled 2026-10-27). Filed by Arpit (Cowork). [detail](open/W-77-skills-package.md)

🟡 **W-64** waits on **W-38** `agent` `codelaw-pre-commit` mirror repo: the pre-commit framework installs codelaw from PyPI by version, as ruff-pre-commit does. Filed by Arpit (Cowork). [detail](open/W-64-codelaw-pre-commit-mirror.md)

🟡 **W-35** waits on **W-05** `agent` README leads with "your rules, before, while and after the AI writes"; then it drops While's "in design" and restores Before's "handful". Filed by Arpit (Cowork). [detail](open/W-35-before-while-after-positioning.md)

### testing (0)

### record update (0)

### research (2)

🟢 **W-36** `cowork` Research codelaw as the linter and formatter for AI — before, while, after and with no AI — beside ESLint + Prettier, Biome, Ruff, oxlint: gaps, provable claims, who adopts it. Filed by Arpit (Cowork). [detail](open/W-36-position-against-eslint-and-biome.md)

🟢 **W-05** `cowork` Research guiding an agent **while** it writes: a `brief` of the rules for a path and construct, and a `hook --pre` check before a write reaches disk; Fux as an optional layer. Filed by Arpit (Cowork). [detail](open/W-05-write-time-guidance.md)

---
