---
type: Handoff
name: W-150
description: "Found 2026-10-30 (Cowork): the dev-server plugins and every Fux index package reach the registries only by hand — publish.yml has no tier for them. Arpit, the same day: once a name is published, CI publishes it. B-063 promoted into this item."
item: W-150
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-150 — `publish.yml` uploads the dev-server plugins and the Fux index packages

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Arpit publishes the three at 0.4.0 by hand from v0.4.0's `dist` (`ci` run 40932052080) | they catch up now | ✅ 2026-10-30 10:30 IST — Arpit, `npm publish` from v0.4.0's `dist` (`ci` 40932052080, checksums verified); all three at 0.4.0 on the registry |
| 2 | `publish.yml`'s npm job: a tier for `codelaw-vite-plugin-*.tgz`, `codelaw-webpack-plugin-*.tgz`, `codelaw-next-*.tgz` (after `codelaw`, since they spawn it) | every release ships them | ⏳ |
| 3 | `scripts/package.py check` (or the gate) fails when `dist/npm` holds a tarball no tier uploads | this cannot happen again | ⏳ |
| 4 | B-063: the Fux index packages from `dist/fux-index/` — npm (after their pack's tier) and PyPI (the pypi job; all eleven projects already have a trusted publisher, checked 2026-10-30) | every release ships them at its version | ⏳ |
| 5 | Arpit: `scripts/npm-trust.sh --fux-index` (one security-key tap per name) before the release that first uploads them — the trust check stops a release otherwise | CI may publish them | 🔴 Arpit, before that release |

**Asked 2026-10-30 (Arpit, text):** *"What are the packages that aren't published? Check on all of them."* → *"For npm, give me the command so that I can publish them right away from my local."*

**Model:** Sonnet (Claude Code): a workflow tier and one check.

**Arpit, 2026-10-30 (text):** *"if it is published once then we will publish it from ci"* — every name above already exists on its registry, so from the next release CI publishes them; nothing more by hand.

## Found

- npm: all three at **0.2.0** (2026-10-27, by hand, W-49); every release since built them into `dist/npm/` (`scripts/package.py`, `DEV_PLUGINS`) and listed them in `package.py names`, so `npm-trust.sh` and the trust check cover them — but the tiers upload only `codelaw-cli-*`, `codelaw-rules-*`, `codelaw-[0-9]*` and `codelaw-skills-*`.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 | npm shows the three at the current release | ⏳ |
| 2 | the next release uploads them from CI; the new check is tested | ⏳ |

## Blockers

None for step 2.

## Record affected

[SR-PACKAGES](../../records/0107_packages.md) decision 3a · RELEASING.md.

## Found 2026-10-30 — `@codelaw/rules-stencil-fux-index` 0.4.0 is *staged*

Arpit's `npm publish` printed `+ …@0.4.0`, but the registry shows only `0.0.0-stage` and `0.2.0`; a re-run answered *E409 "Cannot publish over previously staged version 0.4.0"*. A staged version is in the registry but not public until `npm stage approve` ([npm stage](https://docs.npmjs.com/cli/v11/commands/npm-stage)). Left as is by the ruling above: the next release's CI upload is a new version. The odd `0.0.0-stage` hints an earlier staging run — find what stages, and that CI's upload never does.

## Hazards

- They run `codelaw`; upload them after the `codelaw` tier, as `codelaw-skills` is.

## Reference

W-49 (`archive/open/`) · [W-126](W-126-fast-pipeline.md) (the tiers)
