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
| 🔴 **GitHub Action — the Marketplace listing** ([W-97](open/W-97-github-action.md)): ruled 2026-10-30 — the listing refused `name: codelaw` (taken by a user or org) and the 138-character description; `action.yml` is now **`codelaw check`** with a 115-character description. **On the next release** (after 0.4.0): edit it → tick *Publish this Action to the GitHub Marketplace* → accept the agreement → category *Code quality* → *Update release*. | 2026-10-27 |
| ↳ **blocks:** W-97 itself | |
| 🔴 **Open VSX — waiting on Open VSX** ([W-117](open/W-117-open-vsx-setup.md)): the ownership claim is [EclipseFdn/open-vsx.org#14928](https://github.com/EclipseFdn/open-vsx.org/issues/14928), open. Ruled 2026-10-30: **no token — trusted publishing.** Nothing to do until the claim is approved; then you add the trusted publisher (`arpitarya` · `codelaw` · `publish.yml` · `vscode`). | 2026-10-29 |
| ↳ **blocks:** W-117 itself | |

---

## Open items (20)

### build (18)

🟢 **W-126** `agent` A five-minute pipeline: built and running — cached CI 4m25, Rust+packs 12m30–13m38 (a macOS runner queue), release 3m50 (was 9m). Left: more runs of each kind measured. Filed by Arpit (Cowork). [detail](open/W-126-fast-pipeline.md)

🟢 **W-95** `agent` VS Code extension live as `codelaw.codelaw-vscode`; 0.4.0 uploaded for all eight platforms by `publish.yml` (10-30). Left: Open VSX (W-117), the install docs, `vsce`/`ovsx` in `publish.yml` (D-46, D-47). Filed by Arpit (Cowork). [detail](open/W-95-publish-vscode-extension.md)

🔴 **W-97** `arpit` GitHub Action built; SARIF via `gh`, a moving `v0`. Ruled 10-30: listed as **codelaw check** (the name `codelaw` is taken), description shortened. Left: the Marketplace tick on the next release (inbox), then Claude's throwaway-repo proof. Filed by Arpit (Cowork). [detail](open/W-97-github-action.md)

🟢 **W-105** `agent` Type-aware rules: five landed; ruled 10-29 — agents add the four type questions (SR-TYPES decision 4), the rules as previews; profile the slowest before budgets; corpus counts; the RC (11-10). Filed by Arpit (Cowork). [detail](open/W-105-type-aware-engine.md)

🟢 **W-64** `agent` `codelaw-pre-commit` mirror repo: the pre-commit framework installs codelaw from PyPI by version, as ruff-pre-commit does. Filed by Arpit (Cowork). [detail](open/W-64-codelaw-pre-commit-mirror.md)

🔴 **W-117** `arpit` Open VSX: environment, login, agreement, namespace done 10-29; claim #14928 filed, waiting on Open VSX. Ruled 10-30: no token, trusted publishing (inbox). Filed by Arpit (Cowork). [detail](open/W-117-open-vsx-setup.md)

🟡 **W-106** waits on **W-105** `agent` TypeScript version watch: tested range, CI (7.1 blocking, `typescript@next` nightly), a bump job that opens a PR or an issue. Filed by Arpit (Cowork). [detail](open/W-106-typescript-version-watch.md)

🟢 **W-129** `agent` `deps/licence`, `deps/lockfile-sync`, `deps/version-drift`, `deps/runtime-agreement`. Filed by Arpit (Cowork). [detail](open/W-129-deps-hygiene-rules.md)

🟢 **W-132** `agent` Complexity, nesting and React component limits; CODEOWNERS coverage. Filed by Arpit (Cowork). [detail](open/W-132-complexity-and-codeowners.md)

🟢 **W-134** `arpit` PyPI pending trusted publishers for `codelaw-rules-deps` and its Fux index — Claude fills the form in Chrome, Arpit submits; just before the deps release. Filed by Arpit (Cowork). [detail](open/W-134-deps-pypi-pending-publishers.md)

🟢 **W-135** `arpit` npm first hand publish of `@codelaw/rules-deps` and its Fux index, then their trusted publisher; just before the deps release. Filed by Arpit (Cowork). [detail](open/W-135-deps-npm-first-publish.md)

🟢 **W-139** `agent` Ruled 10-30: build all — both rounds, 2,572 rules: five PyPI framework packs, two npm packs (`testing`, `nextjs`); tranche A on by default, the rest preview/opt-in; SonarJS, Pylint, jsdoc opt-in; regexp, promise on. Filed by Arpit (Cowork). [detail](open/W-139-rule-parity-build.md)

🟢 **W-141** `agent` Formatter vs Prettier, measured on 1,279 files: CSS 42% and HTML 17% identical — a CSS comment/selector defect and void elements first, then each language to ≥ 98%. Filed by Arpit (Cowork). [detail](open/W-141-formatter-parity.md)

🟡 **W-142** waits on **W-139** `arpit` PyPI pending publishers for the five framework packs and their Fux indexes (names free, checked 10-30) — Claude fills, Arpit submits. Filed by Arpit (Cowork). [detail](open/W-142-framework-pack-names.md)

🟡 **W-143** waits on **W-139** `agent` Vue, Svelte, Astro, GraphQL as codelaw languages (L3, L7; a parser and a formatter each, grammars ratified under L11) and Biome's 89 rules for them. Filed by Arpit (Cowork). [detail](open/W-143-vue-svelte-astro-graphql.md)

🟢 **W-145** `agent` A `tailwind` pack (both registries) with the class-order table as data; the formatter sorts classes when the project uses Tailwind, matching prettier-plugin-tailwindcss. Filed by Arpit (Cowork). [detail](open/W-145-tailwind-pack-and-class-sorting.md)

🟢 **W-146** `agent` PR hygiene: `git/commit-message` (Conventional Commits, configurable), `git/branch-name`, `git/pr-size` (400 lines), `git/pr-description`; commit-msg hook, CI, agent hook; git use widened (SR-ARCHITECTURE 14). Filed by Arpit (Cowork). [detail](open/W-146-pr-hygiene.md)

🟡 **W-147** waits on **W-139** `arpit` npm first hand publish of `@codelaw/rules-testing`, `@codelaw/rules-nextjs` and their Fux indexes, then their trusted publisher; just before the release that carries them. Filed by Arpit (Cowork). [detail](open/W-147-testing-nextjs-npm-first-publish.md)

### testing (0)

### reporting and benchmarks (2)

🟢 **W-103** `agent` Spell pack: rules, fixtures, corpus done; the `spell` bench operation built 2026-10-30 (`unset`; local p95 584 ms). Next: three CI `perf` runs on `main`, then the budget to the decider. Filed by Arpit (Cowork). [detail](open/W-103-spell-pack.md)

🟢 **W-102** `agent` Pilot 10-28: 60 sessions, $10.47; A = B = C at 2/20. Ruled 10-30: W-118's redesign as proposed — six arms, screened tasks, 60 runs per arm; the README line stays unless arm C is 0/60 (bound 5%); ~$85, no separate gate on the screen. Filed by Arpit (Cowork). [detail](open/W-102-guidance-vs-check-experiment.md)

### record update (0)

### research (0)

---
