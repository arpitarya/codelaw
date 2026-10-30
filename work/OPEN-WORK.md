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
| 🔴 **Guidance experiment — the screen came back null** ([W-102](open/W-102-guidance-vs-check-experiment.md)): arm 0 (no codelaw at all) broke a rule on only **7 of 38** candidate tasks (6 at 5/5, 31 at 0/5) — it uses `apiFetch`, `http_get` and the clock because it reads them in the code; what tempts is inline style, SQL column names and copied `== None` / `tags=[]`. Fewer than 12, so by the pre-registration no run. **Pick one:** (1) *recommended* — run the 7 at 9 runs each (63 per arm, the 5% bound holds; ~$65, ~$85 with the screen); (2) write ~10 more in the families that tempted, re-screen, then 12 × 5 (~$95 in all); (3) stop and close: the README line stays. | 2026-10-30 |
| ↳ **blocks:** W-102 itself | |
| 🔴 **A public repo, twice** ([W-97](open/W-97-github-action.md)): the Action's proof needs a throwaway repo, and the pre-commit mirror its own. Claude Code's permission check refused `gh repo create --public`. **Allow it** when it asks, or create `arpitarya/codelaw-action-test` and `arpitarya/codelaw-pre-commit` yourself and say so. | 2026-10-30 |
| ↳ **blocks:** W-97 itself | |
| 🔴 **Open VSX — waiting on Open VSX** ([W-95](open/W-95-publish-vscode-extension.md), was W-117): the ownership claim is [EclipseFdn/open-vsx.org#14928](https://github.com/EclipseFdn/open-vsx.org/issues/14928), open. Ruled 2026-10-30: **no token — trusted publishing.** Nothing to do until the claim is approved; then you add the trusted publisher (`arpitarya` · `codelaw` · `publish.yml` · `vscode`). | 2026-10-29 |
| ↳ **blocks:** W-95 (its Open VSX half) | |
| 🔴 **After v0.4.2** ([W-151](open/W-151-release-0-4-1.md)): (1) `git tag -f v0 v0.4.2 && git push -f origin v0` — Claude Code's permission check refused it; `@v0` is v0.4.0 until then. (2) pypi.org → *Publishing → Add a pending publisher*: `codelaw-rules-deps-fux-index` · `arpitarya` · `codelaw` · `publish.yml` · `pypi` (Cowork can fill it in Chrome); the next release creates it. | 2026-10-30 |
| ↳ **blocks:** W-151's close | |

---

## Open items (15)

### build (8)

🟢 **W-105** `agent` Type-aware TypeScript: five rules landed; the four type questions, previews, profiling, corpus counts, the RC (11-10); then the version watch (was W-106: tested range, CI, nightly, bump job). Merged 10-30. Filed by Arpit (Cowork). [detail](open/W-105-type-aware-engine.md)

🟢 **W-129** `agent` `deps/licence`, `deps/lockfile-sync`, `deps/version-drift`, `deps/runtime-agreement` — unblocked: W-128 and W-136 built 10-30; nothing built yet. Filed by Arpit (Cowork). [detail](open/W-129-deps-hygiene-rules.md)

🟢 **W-139** `agent` Ruled 10-30: build all — both rounds, 2,572 rules: five PyPI framework packs, two npm packs (`testing`, `nextjs`); tranche A on by default, the rest preview/opt-in; SonarJS, Pylint, jsdoc opt-in; regexp, promise on. Filed by Arpit (Cowork). [detail](open/W-139-rule-parity-build.md)

🟢 **W-141** `agent` Formatter vs Prettier (CSS 42%, HTML 17%): CSS comment defect and void elements first; Oxc's formatters (was W-148: CSS 88.6%, Markdown 97.3%) measured in codelaw, then L11 to Arpit. Merged 10-30. Filed by Arpit (Cowork). [detail](open/W-141-formatter-parity.md)

🟡 **W-143** waits on **W-139** `agent` Vue, Svelte, Astro, GraphQL as codelaw languages (L3, L7; a parser and a formatter each, grammars ratified under L11) and Biome's 89 rules for them. Filed by Arpit (Cowork). [detail](open/W-143-vue-svelte-astro-graphql.md)

🟢 **W-145** `agent` A `tailwind` pack (both registries) with the class-order table as data; the formatter sorts classes when the project uses Tailwind, matching prettier-plugin-tailwindcss. Filed by Arpit (Cowork). [detail](open/W-145-tailwind-pack-and-class-sorting.md)

🟢 **W-146** `agent` PR hygiene: `git/commit-message` (Conventional Commits, configurable), `git/branch-name`, `git/pr-size` (400 lines), `git/pr-description`; commit-msg hook, CI, agent hook; git use widened (SR-ARCHITECTURE 14). Filed by Arpit (Cowork). [detail](open/W-146-pr-hygiene.md)

🟢 **W-152** `agent` The queue checks refuse a ball that disagrees with its row — `agent` while the next step is Arpit's, or an unruled choice left green (W-97 and W-126 were). Filed by Arpit (Cowork). [detail](open/W-152-ball-matches-the-text.md)

### ci/cd (5)

🟢 **W-126** `agent` Ruled 10-30: `perf` split across parallel jobs, nothing waits on its verdict (until a budget is set, L8); release 3m50 accepted. Cached CI 4m11–4m25 ✅. Next: the `perf` matrix, then the measurement runs. Filed by Arpit (Cowork). [detail](open/W-126-fast-pipeline.md)

🔴 **W-151** `arpit` **v0.4.2 released 10-30**: npm 47/47, PyPI 25/26, Marketplace 8/8, CI and `publish` green. Left (inbox): move `v0` to v0.4.2; the PyPI pending publisher for `codelaw-rules-deps-fux-index`. Filed by Arpit (Claude Code). [detail](open/W-151-release-0-4-1.md)

🔴 **W-95** `arpit` VS Code extension: Marketplace live (0.4.0); Open VSX (was W-117) uploaded by hand, inactive until claim #14928 is approved — then the trusted publisher and a VSCodium check. Merged 10-30. Filed by Arpit (Cowork). [detail](open/W-95-publish-vscode-extension.md)

🔴 **W-97** `arpit` The Action (listed as **codelaw check**, `v0` at v0.4.0) and the pre-commit mirror (was W-64) both need a public repo: allow `gh repo create` in Claude Code, or create them (inbox). Filed by Arpit (Cowork). [detail](open/W-97-github-action.md)

🟡 **W-142** waits on **W-139** `arpit` New packs' registry names: five PyPI framework packs (pending publishers), npm `testing`/`nextjs` (was W-147) and Tailwind's, with Fux indexes; before their release. Merged 10-30. Filed by Arpit (Cowork). [detail](open/W-142-framework-pack-names.md)

### cleanup (0)

### testing (0)

### reporting and benchmarks (2)

🟢 **W-103** `agent` Spell pack done; the `spell` bench operation has its three CI `perf` runs on `main` (10-30: 41005346138, 41006814384, 41007205048). Next: read their p95s, then the budget to Arpit. Filed by Arpit (Cowork). [detail](open/W-103-spell-pack.md)

🔴 **W-102** `arpit` Redesign built (six arms, 42 tasks, screen, PREREG); screened 10-30: only 7 of 38 tasks tempt an agent with no codelaw ($19.45) — the null outcome, no run. Left: run the 7, widen the screen, or stop (inbox). Filed by Arpit (Cowork). [detail](open/W-102-guidance-vs-check-experiment.md)

### record update (0)

### research (0)

---
