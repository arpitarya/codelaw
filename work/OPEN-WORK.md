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

---

## Open items

### build (21)

🟢 **W-67** `agent` Replace `strsim`, `anyhow` (unused) and `postcard` with codelaw's own code, then drop them (L11; ruled in W-66). Filed by Arpit (Cowork). [detail](open/W-67-replace-three-dependencies.md)

🔴 **W-38** `arpit` PyPI 0.2.0 by hand over 2–3 days: `css`, `data` live; left the other four packs (four a day), six trusted publishers, then the Release on W-37's tag brings `codelaw`. Filed by Arpit (Cowork). [detail](open/W-38-pypi-0-2-0-by-hand.md)

🟢 **W-44** `agent` `check --watch` / `fix --watch` built (polling, ruled 2026-10-26); left: DoD 5 on CI's Windows legs and the CI-runner measurements. Filed by Arpit (Cowork). [detail](open/W-44-watch-mode.md)

🟢 **W-45** `agent` CI formats built; left: Claude Code opens a PR running `--format github`, Arpit looks at the annotations (DoD 2). Filed by Arpit (Cowork). [detail](open/W-45-ci-report-formats.md)

🟢 **W-65** `agent` L11's check: CI fails on any dependency missing from `DEPENDENCIES.md`, or a ratified unit whose tree grew. Filed by Arpit (Cowork). [detail](open/W-65-dependency-check.md)

🟡 **W-50** waits on **W-46** `agent` Git hooks built; rulings in (no profile in the hook; `language: system`); left: post-commit's two reminders. Filed by Arpit (Cowork). [detail](open/W-50-git-hooks.md)

🟢 **W-51** `agent` A scheduled auto-fix PR: `init --ci github --autofix weekly`, one open PR with every safe fix, expiries and burn-down in its body. Filed by Arpit (Cowork). [detail](open/W-51-scheduled-autofix-pr.md)

🟢 **W-46** `agent` `codelaw exceptions --expiring 30d --group-by added_by`: every exception, days left, owner; MCP twin. Filed by Arpit (Cowork). [detail](open/W-46-exceptions-report.md)

🟢 **W-58** `agent` Size limits for JS, TS and Python: `max-lines`, `max-lines-per-function`, `max-params` with configurable options; opt-in `size` presets. Filed by Arpit (Cowork). [detail](open/W-58-size-limits.md)

🟢 **W-59** `agent` `[scope.<name>]` replaces `[[overrides]]` and `[profile.<name>]` replaces `[profiles.*]` (aliases for one release); presets ship `tests`, `stories`, `config` scopes. Filed by Arpit (Cowork). [detail](open/W-59-scope-and-profile-tables.md)

🟢 **W-60** `agent` L10: move every consumer-file text into `templates/` with a registry, a slot-only renderer and a test; output byte-identical. Filed by Arpit (Cowork). [detail](open/W-60-consumer-templates.md)

🟢 **W-61** `agent` `html/max-lines` and `css/max-lines` (CSS, SCSS, CSS Modules) with skip options; `size` presets. Filed by Arpit (Cowork). [detail](open/W-61-max-lines-html-css.md)

🟢 **W-62** `agent` SCSS: L3 amended, Biome formats SCSS, CSS rules moved onto the tree-sitter-css fork + tree-sitter-scss (B-047 ruled), 15 rules in a new `scss` pack (npm + PyPI). Filed by Arpit (Cowork). [detail](open/W-62-scss-support.md)

🟢 **W-63** `agent` CSS Modules: `:global`/`composes`/`:export` accepted; unused/undefined-class and `composes` rules in a new `css-modules` pack (npm). Filed by Arpit (Cowork). [detail](open/W-63-css-modules.md)

🟡 **W-49** waits on **W-44** `agent` Dev-server plugins for Vite, Webpack, Rspack, Next.js: overlay and terminal on `local`, builds fail on `ci`. Filed by Arpit (Cowork). [detail](open/W-49-dev-server-plugins.md)


🟢 **W-55** `agent` `security-ts`: 16 rules on top of security-js — unchecked casts of input, permissive schemas, Angular sinks, NestJS guards/CORS/whitelist, Next.js server actions, tRPC, raw ORM queries. Filed by Arpit (Cowork). [detail](open/W-55-security-ts-pack.md)

🟢 **W-56** `agent` `security-html`: 19 rules plus five existing `html/*` — inline handlers, `javascript:` URLs, mixed content, iframes, CSP, SRI, CSRF fields, secrets in markup, template sinks. Filed by Arpit (Cowork). [detail](open/W-56-security-html-pack.md)

🟢 **W-57** `agent` `security-python`: 61 rules — Bandit's set ported, injection (SQL, NoSQL, LDAP, XPath, SSTI), Django/Flask/FastAPI settings, mass assignment, crypto, uploads, ML loaders, `trust_remote_code`, prompt injection. Filed by Arpit (Cowork). [detail](open/W-57-security-python-pack.md)

🟡 **W-64** waits on **W-38** `agent` `codelaw-pre-commit` mirror repo: the pre-commit framework installs codelaw from PyPI by version, as ruff-pre-commit does. Filed by Arpit (Cowork). [detail](open/W-64-codelaw-pre-commit-mirror.md)

🟡 **W-34** waits on **fux W-299** `agent` An optional Fux index package beside each pack (`@codelaw/rules-<name>-fux-index`, `codelaw-rules-<name>-fux-index`), never a default; then L6's amendment. Filed by Arpit (Cowork). [detail](open/W-34-fux-index-packages.md)

🟡 **W-35** waits on **W-05** `agent` README leads with "your rules, before, while and after the AI writes"; then it drops While's "in design" and restores Before's "handful". Filed by Arpit (Cowork). [detail](open/W-35-before-while-after-positioning.md)

### testing (0)

### record update (0)

### research (2)

🟢 **W-36** `cowork` Research codelaw as the linter and formatter for AI — before, while, after and with no AI — beside ESLint + Prettier, Biome, Ruff, oxlint: gaps, provable claims, who adopts it. Filed by Arpit (Cowork). [detail](open/W-36-position-against-eslint-and-biome.md)

🟢 **W-05** `cowork` Research guiding an agent **while** it writes: a `brief` of the rules for a path and construct, and a `hook --pre` check before a write reaches disk; Fux as an optional layer. Filed by Arpit (Cowork). [detail](open/W-05-write-time-guidance.md)

---
