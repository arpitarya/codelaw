---
type: Handoff
name: W-150
description: "Found 2026-10-30 (Cowork): @codelaw/vite-plugin, @codelaw/webpack-plugin and @codelaw/next are stuck at 0.2.0 on npm — publish.yml trust-checks them but has no tier that uploads them."
item: W-150
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-150 — `publish.yml` uploads the dev-server plugins

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Arpit publishes the three at 0.4.0 by hand from v0.4.0's `dist` (`ci` run 40932052080) | they catch up now | ✅ 2026-10-30 10:30 IST — Arpit, `npm publish` from v0.4.0's `dist` (`ci` 40932052080, checksums verified); all three at 0.4.0 on the registry |
| 2 | `publish.yml`'s npm job: a tier for `codelaw-vite-plugin-*.tgz`, `codelaw-webpack-plugin-*.tgz`, `codelaw-next-*.tgz` (after `codelaw`, since they spawn it) | every release ships them | ⏳ |
| 3 | `scripts/package.py check` (or the gate) fails when `dist/npm` holds a tarball no tier uploads | this cannot happen again | ⏳ |

**Asked 2026-10-30 (Arpit, text):** *"What are the packages that aren't published? Check on all of them."* → *"For npm, give me the command so that I can publish them right away from my local."*

**Model:** Sonnet (Claude Code): a workflow tier and one check.

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

## Hazards

- They run `codelaw`; upload them after the `codelaw` tier, as `codelaw-skills` is.

## Reference

W-49 (`archive/open/`) · [W-126](W-126-fast-pipeline.md) (the tiers)
