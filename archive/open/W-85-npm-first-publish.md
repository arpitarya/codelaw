---
type: Handoff
name: W-85
description: "Arpit, 2026-10-27 (voice docket): one item to first-publish every npm package not yet on the registry, by hand, then trust publish.yml for them and for the 15 Fux index packages."
item: W-85
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Claude Code (Opus 5.5)"
ball: closed
---

# W-85 — npm: first-publish the nine new packages, then trust `publish.yml`

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | The decider picks the version these nine are first published at | every package agrees with the engine it needs | ✅ **0.2.0** — Arpit published at 0.2.0, 2026-10-27 07:33 IST, from a local build of `main` |
| 2 | `npm publish` by hand: `@codelaw/rules-scss`, `rules-css-modules`, `rules-security-js`, `rules-security-ts`, `rules-security-html`, `@codelaw/vite-plugin`, `webpack-plugin`, `next`, `@codelaw/skills` | the nine names exist on npm (a trusted publisher binds only to a package that exists) | ✅ 2026-10-27 — all nine `latest: 0.2.0` (dist-tags endpoint; the package pages lagged) |
| 3 | `scripts/npm-trust.sh` | every release package trusts `publish.yml` · `npm`, the nine included | ✗ 2026-10-27: stopped at the first name, `codelaw` — **E409**, *"a trusted publisher configuration … already exists"* — under `set -e`, so none of the nine was reached. → ✅ the 24 untrusted names, 2026-10-27 (browser 11, Arpit's CLI loop the rest); `npm-trust.sh`'s E409 fix ✅ 2026-10-27 (an agent): E409 counts as trusted, the loop carries on, failures listed with exit 1 |
| 4 | The 15 `@codelaw/rules-<name>-fux-index` packages trust `publish.yml` too — `npm-trust.sh` reads `package.py names`, which does not list them | W-34's later `publish.yml` upload is accepted | ✅ 2026-10-27 — added in the browser and by Arpit's CLI loop; all 15 show `arpitarya/codelaw` · `publish.yml` · `npm` |
| 5 | Check each package's *Trusted Publisher* shows `arpitarya/codelaw` · `publish.yml` · `npm` | proof, not belief | ✅ 2026-10-27 07:52 IST — all 24 read back in the browser: *Pending validation*, `npm publish` allowed |

**Arpit, 2026-10-27 (voice docket):** *"create work items, one work item to publish all NPM packages and then multiple work items, four packages a day but make sure all the packages are accounted in this."* Earlier: *"create a work item just for publishing four packages a day for PyPI … add the command in the OpenWork file so that I can just copy it and paste it every day … don't count today's day"*, and *"Include these SCSS and security packs as well."*

**Why step 1 exists:** none of the nine is in the v0.2.0 tag (it has eleven packs, no plugins, no skills package), and `dist/npm/` on the Mac holds only the v0.2.0 set. Published as 0.2.0 from today's `main`, `@codelaw/rules-scss@0.2.0` would install beside `codelaw@0.2.0`, whose engine predates SCSS — a pair that does not work. The next release's version avoids that; its cost is waiting for a green CI `dist` (W-84) and a version bump.

## What happened, 2026-10-27

**2026-10-27 07:40–07:50 IST, in the browser (Cowork, Claude in Chrome):** trusted publisher added, shown as *Pending validation*, on 10 of the 24 — `@codelaw/rules-{js,ts,react,node,stencil,html,css,scss,css-modules,data}-fux-index` (checked 07:48 IST). After that, npm's Cloudflare check (*"Verifying you are human"*) stopped every save, and a save did not persist after it cleared; the browser work stopped there. Left: the 9 new packages and 5 Fux index packages (`tanstack-query`, `redux-toolkit`, `security-js`, `security-ts`, `security-html`) — by the CLI loop below (an E409 on one already done is harmless). **npm expires an unvalidated configuration:** each new one reads *"Publish once before Oct 29, 2026, 2:10 AM UTC to validate"*, so the Fux index packages (which `publish.yml` does not publish yet, W-34) will need it re-added after that date, or a publish before it.

Arpit published the nine from a local 0.2.0 build (`…/scratchpad/pub/dist/npm`), skipping any name already on npm; every publish succeeded. `scripts/npm-trust.sh` then failed on `codelaw` with E409 (its trust exists since 2026-10-24) and stopped. npm 12 also warned that Node 24.13.0 is below its `^24.15.0` — the call still reached the registry. **Consequence of 0.2.0:** `@codelaw/rules-scss@0.2.0` sits beside `codelaw@0.2.0`, whose engine predates SCSS (L3 amended 2026-10-26), so the SCSS pack works only from the next release; the PyPI days follow the same version (W-88, W-89).

**Trust now, only the 24 untrusted names** (one security-key tap each; a failure prints ✗ and the loop carries on):

```bash
for n in vite-plugin webpack-plugin next skills rules-scss rules-css-modules rules-security-js rules-security-ts rules-security-html $(python3 scripts/fux-index.py names > /tmp/fx; awk '$1=="npm"{sub("@codelaw/","",$2); print $2}' /tmp/fx);
do npx -y npm@12 trust github "@codelaw/$n" --file publish.yml --repo arpitarya/codelaw --env npm --allow-publish -y \
  || echo "✗ @codelaw/$n"; done
```

## Expiry — re-check before the next release

npm lists every new trusted publisher as *Pending validation* with a deadline: *"Publish once before … to validate this configuration."* Read 2026-10-27: `@codelaw/rules-tanstack-query` and `@codelaw/rules-redux-toolkit` — **Oct 27, 12:12 UTC**; the 24 added today — **Oct 29, ~02:10–02:20 UTC**. Nothing publishes them before then: every one is already on npm at 0.2.0, so the v0.2.0 Release (W-86) skips them. Assume they lapse. **Before the next release:** read each package's *Trusted Publisher* (or let `publish.yml`'s `trust` job report), and re-add any that lapsed — `npm-trust.sh` once it skips E409 (step left here).

## What is left

Nothing — closed 2026-10-27 (an agent, Claude Code):

- ✅ `scripts/npm-trust.sh`: an E409 (*"already exists"*) counts as trusted and the loop carries on; any other failure prints ✗, the loop carries on, and the script exits 1 listing the failed names. The 15 Fux index names come in behind `--fux-index` (a flag, because `publish.yml` does not publish them yet and each name costs a security-key tap); `--dry-run` kept; fd 3 kept, so npm keeps stdin. Verified offline: `bash -n`, and a stubbed `npx` on `PATH` returning E409 for `codelaw` and E403 for `@codelaw/rules-css` (E404 for `@codelaw/rules-css-fux-index` with `--fux-index`) — exit 1, both failures listed, every other name reached.
- ✅ The expiry re-check: RELEASING.md §Every release, step 0.

## The commands as filed (before the publish)

```bash
# 2 · from the green CI run of the commit that carries <v>
gh run download <run-id> -R arpitarya/codelaw -n dist -D /tmp/cl-dist
for p in rules-scss rules-css-modules rules-security-js rules-security-ts rules-security-html vite-plugin webpack-plugin next skills; do
  npm publish /tmp/cl-dist/npm/codelaw-$p-<v>.tgz --access public
done
# 3
scripts/npm-trust.sh
# 4 · fd 3, so npm keeps stdin for its login prompt (as npm-trust.sh does)
python3 scripts/fux-index.py names > /tmp/fx
while read -r r n <&3; do [ "$r" = npm ] && npx -y npm@12 trust github "$n" --file publish.yml --repo arpitarya/codelaw --env npm --allow-publish -y; done 3< /tmp/fx
```

`@codelaw/skills` depends on `codelaw` at exactly `<v>` (D-35), so `codelaw@<v>` should be on npm first — publish it with the rest of the release, or by hand before the skills package (W-77).

## The whole set (every name accounted for)

| day / item | PyPI projects created | version |
|---|---|---|
| live before 2026-10-27 | `codelaw-rules-css`, `-data`; `codelaw-rules-{css,data,html,js}-fux-index` | 0.2.0 |
| W-86 · Wed 10-28 | `codelaw-rules-{html,js,python,react}` | 0.2.0 (v0.2.0 tag) |
| W-87 · Thu 10-29 | `codelaw-rules-{react,scss,python,security-js}-fux-index` | 0.2.0 |
| W-88 · Fri 10-30 | `codelaw-rules-security-{html,python}-fux-index`; `codelaw-skills`, `codelaw-rules-scss` | 0.2.0; W-85's version |
| W-89 · Sat 10-31 | `codelaw-rules-security-{js,html,python}` | W-85's version |
| CI, through its pending publisher | `codelaw` | at the Release (W-86 step 3) |

npm: the nine still missing are W-85; the 15 Fux index packages are live and get their trust in W-85 too. Nothing else in `package.py names` or `fux-index.py names` is unpublished.

## Definition of done

✅ all met, 2026-10-27.

- The nine show `<v>` on npmjs.com; each package's *Trusted Publisher* lists `arpitarya/codelaw` · `publish.yml` · `npm` — and so do the 15 Fux index packages.
- The version picked in step 1 is written here with the decider's words.

## Record affected

No SR affected. `RELEASING.md` §"One time, by hand" already describes the first publish.

## Hazards

- npm asks for the security key on each publish and each trust; the package's *Publishing access* forbids tokens that bypass 2FA.
- A package npm does not trust stops the next release at `publish.yml`'s preflight, before either registry is written.
- `rules-tanstack-query` and `rules-redux-toolkit` show *Pending validation* (trust set, never yet used); expected to turn *Valid* on the first CI publish.
