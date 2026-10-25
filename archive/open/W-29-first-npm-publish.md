---
type: Handoff
name: W-29
description: "Arpit, 2026-10-24: the first publish of the 14 npm packages, by hand from his machine, from CI's dist artifact."
item: W-29
filed: 2026-10-24
filed_by: "Arpit (Claude Code, Opus 5.5)"
updated: 2026-10-25
updated_by: "Claude (Opus 5.5), cloud docket 2026-10-25"
ball: arpit
---

# W-29 — Publish the 14 npm packages for the first time, from Arpit's machine

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | The `@codelaw` npm org exists, Arpit (`aarya11`) its owner, 2FA on | the scope the 13 sub-packages publish under | ✅ 2026-10-24, checked in the browser: org `codelaw`, 1 member, 0 packages; `codelaw` (unscoped) is not taken |
| 2 | W-24 step 1 lands (the Windows build compiles) and `main` is pushed | CI builds all 8 binaries and uploads the `dist` artifact | ✅ 2026-10-24: run 40026216188 on `5e489f4` built all 8 and uploaded `dist`, but the run **failed** elsewhere (W-24), so it was not green |
| 3 | Download `dist` from that green run; `package.py check` and `law-gate.py` pass on it | exactly what CI built and checked, nothing rebuilt locally | ✅ 2026-10-24, from run 40026216188: both passed, and all 14 dry runs were clean |
| 4 | Publish in order — 8 `@codelaw/cli-*`, then 5 `@codelaw/rules-*`, then `codelaw` last; dry run first | no moment where `codelaw` exists and its platform binary does not | ✅ 2026-10-24 03:18 IST: all 14 live at 0.1.0 (`latest`), checked on the registry |
| 5 | Bind trusted publishing: `scripts/npm-trust.sh` (every package trusts `arpitarya/codelaw` · `publish.yml` · environment `npm`) | every later release comes from CI, no token | ✅ 2026-10-24: "Trust configuration created successfully" for all 14 (permissions: publish, stage publish) |

> **2026-10-24, the same day:** W-31 splits the packs by registry. `@codelaw/rules-python@0.1.0` is to be unpublished from npm, and `@codelaw/rules-ts`, `-react`, `-node`, `-stencil` get their first hand publish at 0.2.0 ([W-31](W-31-registry-pack-split.md) step 7).

**Arpit, 2026-10-24:** *"Create a work item to publish npm packages … Give me the command … so I can do the first publish from local. I have already configured token for my whole system."*

**Model:** NONE — Arpit runs the commands; an agent prepares the `dist` (W-24) and checks the result.

## The commands (from the repo root on the Mac)

```bash
# 3 · the dist CI built on main (RUN_ID: the green ci.yml run on the pushed main)
gh run list -R arpitarya/codelaw -w ci.yml -b main -L 3
rm -rf dist && gh run download <RUN_ID> -R arpitarya/codelaw -n dist -D dist
python3 scripts/package.py check dist && python3 scripts/law-gate.py dist

# 4 · publish — dry run first, then the same loop without --dry-run
npm whoami                                   # aarya11
cd dist/npm
for t in codelaw-cli-*.tgz codelaw-rules-*.tgz codelaw-0.1.0.tgz; do
  npm publish "$t" --access public --dry-run || break
done
for t in codelaw-cli-*.tgz codelaw-rules-*.tgz codelaw-0.1.0.tgz; do
  npm publish "$t" --access public || { echo "STOPPED at $t"; break; }
done
cd ../..

# check: every name is live at 0.1.0
python3 scripts/package.py names | awk '$1=="npm"{print $2}' | xargs -n1 -I{} npm view {} version

# 5 · trusted publishing for the next release (npm 12, logged in)
bash scripts/npm-trust.sh
```

## Goal

All 14 npm packages exist at the release version, published from CI's checked `dist`, and trusted publishing is bound so no later release needs a token.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · live | `npm view <name> version` prints the version for all 14 names in `package.py names` | ✅ 2026-10-24 (the 0.1.0 names) |
| 2 · works | in an empty folder: `npm i codelaw && npx codelaw --version` on the Mac; the `smoke` job's install from the registry passes on the 8 platforms | 🟡 the Mac half ✅ 2026-10-25: in an empty folder `npm i codelaw@0.1.0` added 2 packages, `npx codelaw --version` printed `codelaw 0.1.0`, `npx codelaw check a.js` ran (exit 0); `smoke` still installs from the staging index, not the registry. **The 8-platform half moves to 0.2.0** — **Arpit, 2026-10-25 (docket):** *"Yes, both"* — an agent prepares 0.2.0 (the bump and the command list); W-29's 8-platform registry install is checked on 0.2.0, not 0.1.0. |
| 3 · trusted | `npm trust list <name>` shows `arpitarya/codelaw` · `publish.yml` · `npm` for each package | ✅ 2026-10-24 (`npm-trust.sh`'s output) |
| 4 · recorded | `work/IMPLEMENTATION.md` row; `RELEASING.md` §One time step 2 marked done with the date | ✅ found done 2026-10-25: the 2026-10-24 W-29 row in `IMPLEMENTATION.md`, and `RELEASING.md` §One time "Done for npm on 2026-10-24 (steps 1–3, 0.1.0)" |

## Blockers

W-24 step 1 (the Windows build) and the push of `main` — without all eight binaries there is no `dist`, and publishing `codelaw` without its eight `@codelaw/cli-*` breaks every platform but the one it was built on.

## Record affected

[SR-PACKAGES](../../records/0107_packages.md) §Consequences already says the first npm publish is by hand — **no SR change**; `RELEASING.md` §One time steps 1–3 get their dates.

## Hazards

- **A published version is permanent.** npm never lets `0.1.0` be reused, even after an unpublish; a mistake means `0.1.1`. Publish from the checked `dist` only, never a local rebuild.
- **Order matters:** `codelaw` declares the eight `@codelaw/cli-*` as optional dependencies; published first, it installs with no binary. The loop above puts it last.
- **One release, both registries (L6 decision 9):** publish W-30's wheels from the **same** `dist`, the same day.
- **npm's token policy:** npm shows "tokens that bypass 2FA are being restricted — account changes (Aug 2026) and direct publishing (Jan 2027)". A by-hand publish after January 2027 may need an interactive 2FA approval; step 5 removes the need for later releases.
- **A partial run:** the loop stops at the first failure; re-run it — `npm publish` refuses a version that already exists, so skip the names `npm view` already shows.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
- [`RELEASING.md`](../../RELEASING.md) §One time; `scripts/package.py` (`names`, `check`); `scripts/law-gate.py`; `scripts/npm-trust.sh`.
