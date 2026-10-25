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
| ↳ **blocks:** W-38 itself; nothing else in the queue | |
| 🔴 **Watch mode's two questions:** `notify` is CC0 (L2 refuses it), so the watch polls — keep polling, amend L2, or write OS backends later; and does DoD 1 time the re-check (~3–6 ms) or save-to-print (~250 ms)? — [W-44](open/W-44-watch-mode.md) | 2026-10-25 |
| ↳ **blocks:** W-44; W-49 waits on it | |
| 🔴 **Open a PR that runs `codelaw check --format github`** so the annotations are seen on GitHub (DoD 2) — [W-45](open/W-45-ci-report-formats.md) | 2026-10-25 |
| ↳ **blocks:** W-45 itself; nothing else in the queue | |
| 🔴 **Git hooks:** confirm the pre-commit hook runs with no profile (W-39 detection; `CODELAW_HOOK_ARGS` overrides), and rule DoD 5 — a root manifest for the pre-commit framework, or codelaw on `PATH` — [W-50](open/W-50-git-hooks.md) | 2026-10-25 |
| ↳ **blocks:** W-50 itself; nothing else in the queue | |

---

## Open items

### build

🔴 **W-38** `arpit` PyPI 0.2.0 by hand over 2–3 days: `css`, `data` live; left the other four packs (four a day), six trusted publishers, then the Release on W-37's tag brings `codelaw`. Filed by Arpit (Cowork). [detail](open/W-38-pypi-0-2-0-by-hand.md)

🔴 **W-44** `arpit` `check --watch` and `fix --watch` built (polling, no new dependency); two questions for the decider, DoD 5 on CI's Windows legs. Filed by Arpit (Cowork). [detail](open/W-44-watch-mode.md)

🔴 **W-45** `arpit` CI formats built (`github`, `gitlab`, `junit`, `checkstyle`, stable fingerprints); left: seeing the annotations on a real GitHub PR (DoD 2). Filed by Arpit (Cowork). [detail](open/W-45-ci-report-formats.md)

🔴 **W-50** `arpit` Git hooks built (`init --git-hooks`, `.pre-commit-hooks.yaml`, recipes); left: two rulings (profile, DoD 5) and post-commit's reminders once W-46 lands. Filed by Arpit (Cowork). [detail](open/W-50-git-hooks.md)

🟢 **W-51** `agent` A scheduled auto-fix PR: `init --ci github --autofix weekly`, one open PR with every safe fix, expiries and burn-down in its body. Filed by Arpit (Cowork). [detail](open/W-51-scheduled-autofix-pr.md)

🟢 **W-46** `agent` `codelaw exceptions --expiring 30d --group-by added_by`: every exception, days left, owner; MCP twin. Filed by Arpit (Cowork). [detail](open/W-46-exceptions-report.md)

🟡 **W-49** waits on **W-44** `agent` Dev-server plugins for Vite, Webpack, Rspack, Next.js: overlay and terminal on `local`, builds fail on `ci`. Filed by Arpit (Cowork). [detail](open/W-49-dev-server-plugins.md)

🟢 **W-54** `agent` `security-js`: 66 rules — injection (SQL, shell, SSTI …), XSS and DOM sinks, traversal, SSRF, crypto, secrets, cookies, TLS, Electron, mass assignment, prototype pollution, prompt injection. Filed by Arpit (Cowork). [detail](open/W-54-security-js-pack.md)

🟢 **W-55** `agent` `security-ts`: 16 rules on top of security-js — unchecked casts of input, permissive schemas, Angular sinks, NestJS guards/CORS/whitelist, Next.js server actions, tRPC, raw ORM queries. Filed by Arpit (Cowork). [detail](open/W-55-security-ts-pack.md)

🟢 **W-56** `agent` `security-html`: 19 rules plus five existing `html/*` — inline handlers, `javascript:` URLs, mixed content, iframes, CSP, SRI, CSRF fields, secrets in markup, template sinks. Filed by Arpit (Cowork). [detail](open/W-56-security-html-pack.md)

🟢 **W-57** `agent` `security-python`: 61 rules — Bandit's set ported, injection (SQL, NoSQL, LDAP, XPath, SSTI), Django/Flask/FastAPI settings, mass assignment, crypto, uploads, ML loaders, `trust_remote_code`, prompt injection. Filed by Arpit (Cowork). [detail](open/W-57-security-python-pack.md)

🟡 **W-34** waits on **fux W-299** `agent` An optional Fux index package beside each pack (`@codelaw/rules-<name>-fux-index`, `codelaw-rules-<name>-fux-index`), never a default; then L6's amendment. Filed by Arpit (Cowork). [detail](open/W-34-fux-index-packages.md)

🟡 **W-35** waits on **W-05** `agent` README leads with "your rules, before, while and after the AI writes"; then it drops While's "in design" and restores Before's "handful". Filed by Arpit (Cowork). [detail](open/W-35-before-while-after-positioning.md)

### testing

### record update

### research

🟢 **W-36** `cowork` Research codelaw as the linter and formatter for AI — before, while, after and with no AI — beside ESLint + Prettier, Biome, Ruff, oxlint: gaps, provable claims, who adopts it. Filed by Arpit (Cowork). [detail](open/W-36-position-against-eslint-and-biome.md)

🟢 **W-05** `cowork` Research guiding an agent **while** it writes: a `brief` of the rules for a path and construct, and a `hook --pre` check before a write reaches disk; Fux as an optional layer. Filed by Arpit (Cowork). [detail](open/W-05-write-time-guidance.md)

---
