---
type: Handoff
name: W-18
description: "Arpit, 2026-10-20: the pipeline defects an independent review found, plus the install matrix on every platform."
item: W-18
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-20
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-18 — Harden CI and the release pipeline

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Make the clippy step fail CI (`pipefail`), then fix the warnings it was hiding | a green CI means clippy passed | ⏳ |
| 2 | Pin every action by commit SHA and pin npm's version in the OIDC jobs | no moved tag can publish | ⏳ |
| 3 | Require tag-only deployment rules on the `npm`/`pypi` environments | only a release can publish | ⏳ |
| 4 | Verify `SHA256SUMS` before upload; attest the CI build | publish exactly what CI built | ⏳ |
| 5 | Fix the remaining findings below | the pipeline matches L6 decision 9 and SR-PACKAGES decision 11 | ⏳ |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Model:** Sonnet: each finding is a concrete, checkable change to a workflow or script.

## Goal

Every finding below is fixed, and the L6 install matrix and runtime fence run on all eight platforms.

## Definition of done

| # | finding | where | severity | state |
|---|---|---|---|---|
| 1 | The clippy step pipes into `tee` under `bash -e` with no `pipefail`, so clippy failures never fail CI; a green verdict is cached and the publish gate trusts it. Fix the warnings it hides first (W-17's last row). | `.github/workflows/ci.yml:118` | 🧨 high | ⏳ |
| 2 | None of the 28 `uses:` is pinned by SHA, including `Swatinem/rust-cache@v2` and the branch ref `pypa/gh-action-pypi-publish@release/v1` in an `id-token: write` job. | `.github/workflows/*.yml` | high | ⏳ |
| 3 | `npm i -g npm@latest` runs unpinned in both OIDC jobs. | `.github/workflows/publish.yml:78,119` | high | ⏳ |
| 4 | Environment protection is optional; trusted publishers check workflow and environment, not the ref — require tag-only (`v*`) deployment rules on `npm` and `pypi`. | `RELEASING.md:45` | major (plausible) | ⏳ |
| 5 | `SHA256SUMS` is generated but never verified before upload; no build attestation in `ci.yml` (`--provenance` attests `publish.yml`). | `scripts/package.py:249` | minor | ⏳ |
| 6 | `npm deprecate … && echo` hides a failed deprecate under `set -e`: a partial withdrawal reports success. | `scripts/withdraw.sh:25` | minor | ⏳ |
| 7 | The SR-guard replay prints "0 commits" and passes when `git rev-list` fails (bad `RULE-SINCE`). | `.github/workflows/ci.yml:94` | minor | ⏳ |
| 8 | No PEP 440 conversion: `0.2.0-beta.1` yields an invalid wheel; its metadata differs from `package.py`'s (W-22 fixes the record). | `scripts/build-packs.py:150` | minor | ⏳ |
| 9 | Any non-pre-release becomes npm `latest`, so a backport (0.1.5 after 0.2.0) demotes the newer line. | `.github/workflows/publish.yml:128` | minor | ⏳ |
| 10 | `actions/checkout` keeps `persist-credentials` (low impact: read-only token). | `.github/workflows/*.yml` | minor | ⏳ |
| 11 | The unquoted regex `^(Number\.)?NaN$` is YAML 1.2 only; quote it for other tools. | `packs/js/rules/js/use-isnan.md:23` | minor | ⏳ |
| 12 | L6's install matrix and L3's runtime fence run on linux-x64 only, from built files not a staging index (was B-009, B-012). | `scripts/smoke.sh · ci.yml` | gap | ⏳ |

## Blockers

None for the code. The environment rules are set by the decider in GitHub settings.

## Record affected

[SR-PACKAGES](../../records/0107_packages.md) decisions 9, 11, 12 · [SR-LAW-6](../../records/0008_LAW-6-python-and-node-packages.md) §How L6 is enforced · `RELEASING.md`.

## Hazards

- **Changing `ci.yml` resets every cache key** (W-04 §Measured): batch the workflow edits into one change.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
