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
| 🔴 **Reserve and publish the 16 Fux index package names** (npm `@codelaw/rules-<name>-fux-index` ×10 by hand, then `npm-trust.sh`; PyPI `codelaw-rules-<name>-fux-index` ×6 by `twine`, then a trusted publisher each) — built by CI as the `fux-index` artifact; RELEASING.md §"The optional Fux index packages"; also ratify `fux-engine` as D-33 in `DEPENDENCIES.md` — [W-34](open/W-34-fux-index-packages.md) | 2026-10-26 |
| ↳ **blocks:** W-34 itself | |

---

## Open items

### build (17)

🔴 **W-38** `arpit` PyPI 0.2.0 by hand over 2–3 days: `css`, `data` live; left the other four packs (four a day), six trusted publishers, then the Release on W-37's tag brings `codelaw`. Filed by Arpit (Cowork). [detail](open/W-38-pypi-0-2-0-by-hand.md)

🔴 **W-34** `arpit` Fux index packages built (L6 decision 12; `scripts/fux-index.py`, CI, `init` prints not installs); left: reserve the 16 names and publish them by hand (npm ten, PyPI six), then teach `publish.yml` to upload `dist/fux-index/`. Filed by Arpit (Cowork). [detail](open/W-34-fux-index-packages.md)

🟢 **W-44** `agent` `check --watch` / `fix --watch` built (polling, ruled 2026-10-26); left: DoD 5 on CI's Windows legs and the CI-runner measurements. Filed by Arpit (Cowork). [detail](open/W-44-watch-mode.md)

🟢 **W-45** `agent` CI formats built; PR [#3](https://github.com/arpitarya/codelaw/pull/3) is open; left: Arpit looks at its annotations and says so (DoD 2). Filed by Arpit (Cowork). [detail](open/W-45-ci-report-formats.md)

🟢 **W-51** `agent` A scheduled auto-fix PR built (`init --ci github|gitlab --autofix daily|weekly [--unsafe]`, one PR, burn-down in the body); left: a real GitHub run (DoD 1, 3) and the exceptions section (W-46 landed). Filed by Arpit (Cowork). [detail](open/W-51-scheduled-autofix-pr.md)

🟢 **W-58** `agent` Size limits for JS, TS and Python: `max-lines`, `max-lines-per-function`, `max-params` with configurable options; opt-in `size` presets. Filed by Arpit (Cowork). [detail](open/W-58-size-limits.md)

🟢 **W-59** `agent` `[scope.<name>]` replaces `[[overrides]]` and `[profile.<name>]` replaces `[profiles.*]` (aliases for one release); presets ship `tests`, `stories`, `config` scopes. Filed by Arpit (Cowork). [detail](open/W-59-scope-and-profile-tables.md)


🟡 **W-61** waits on **W-59** `agent` `html/max-lines` and `css/max-lines` built with the `size` presets; left: DoD 3, a `[scope]` giving `_*.scss` partials their own limit, once W-59 (scope tables) and W-62 (SCSS) land, and `data` in JSON/SARIF once W-58's `data_of` lands. Filed by Arpit (Cowork). [detail](open/W-61-max-lines-html-css.md)

🟢 **W-62** `agent` SCSS: L3 amended, Biome formats SCSS, CSS rules moved onto the tree-sitter-css fork + tree-sitter-scss (B-047 ruled), 15 rules in a new `scss` pack (npm + PyPI). Filed by Arpit (Cowork). [detail](open/W-62-scss-support.md)

🟢 **W-63** `agent` CSS Modules: `:global`/`composes`/`:export` accepted; unused/undefined-class and `composes` rules in a new `css-modules` pack (npm). Filed by Arpit (Cowork). [detail](open/W-63-css-modules.md)

🟢 **W-49** `agent` Dev-server plugins (`@codelaw/vite-plugin`, `webpack-plugin` also for Rspack, `next`) built and tested against the binary; left: an example app per real host (needs Vite, webpack, Rspack, Next.js installed, L11) and the first npm publish. Filed by Arpit (Cowork). [detail](open/W-49-dev-server-plugins.md)

🟢 **W-54** `agent` `security-js`: 66 rules — injection (SQL, shell, SSTI …), XSS and DOM sinks, traversal, SSRF, crypto, secrets, cookies, TLS, Electron, mass assignment, prototype pollution, prompt injection. Filed by Arpit (Cowork). [detail](open/W-54-security-js-pack.md)

🟢 **W-55** `agent` `security-ts`: 16 rules on top of security-js — unchecked casts of input, permissive schemas, Angular sinks, NestJS guards/CORS/whitelist, Next.js server actions, tRPC, raw ORM queries. Filed by Arpit (Cowork). [detail](open/W-55-security-ts-pack.md)

🟢 **W-56** `agent` `security-html`: 19 rules plus five existing `html/*` — inline handlers, `javascript:` URLs, mixed content, iframes, CSP, SRI, CSRF fields, secrets in markup, template sinks. Filed by Arpit (Cowork). [detail](open/W-56-security-html-pack.md)

🟢 **W-57** `agent` `security-python`: 61 rules — Bandit's set ported, injection (SQL, NoSQL, LDAP, XPath, SSTI), Django/Flask/FastAPI settings, mass assignment, crypto, uploads, ML loaders, `trust_remote_code`, prompt injection. Filed by Arpit (Cowork). [detail](open/W-57-security-python-pack.md)

🟡 **W-64** waits on **W-38** `agent` `codelaw-pre-commit` mirror repo: the pre-commit framework installs codelaw from PyPI by version, as ruff-pre-commit does. Filed by Arpit (Cowork). [detail](open/W-64-codelaw-pre-commit-mirror.md)

🟡 **W-35** waits on **W-05** `agent` README leads with "your rules, before, while and after the AI writes"; then it drops While's "in design" and restores Before's "handful". Filed by Arpit (Cowork). [detail](open/W-35-before-while-after-positioning.md)

### testing (0)

### record update (0)

### research (2)

🟢 **W-36** `cowork` Research codelaw as the linter and formatter for AI — before, while, after and with no AI — beside ESLint + Prettier, Biome, Ruff, oxlint: gaps, provable claims, who adopts it. Filed by Arpit (Cowork). [detail](open/W-36-position-against-eslint-and-biome.md)

🟢 **W-05** `cowork` Research guiding an agent **while** it writes: a `brief` of the rules for a path and construct, and a `hook --pre` check before a write reaches disk; Fux as an optional layer. Filed by Arpit (Cowork). [detail](open/W-05-write-time-guidance.md)

---
