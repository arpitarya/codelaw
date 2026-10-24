---
type: Handoff
name: W-18
description: "Arpit, 2026-10-20: the pipeline defects an independent review found, plus the install matrix on every platform."
item: W-18
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-24
updated_by: "Claude (Opus 5.5), docket: step 3 done in the decider's browser"
ball: agent
---

# W-18 — Harden CI and the release pipeline

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Make the clippy step fail CI (`pipefail`), then fix the warnings it was hiding | a green CI means clippy passed | ✅ pipefail · the warnings are W-17's last row |
| 2 | Pin every action by commit SHA and pin npm's version in the OIDC jobs | no moved tag can publish | ✅ |
| 3 | Require tag-only deployment rules on the `npm`/`pypi` environments | only a release can publish | ✅ 2026-10-24: `npm` and `pypi`, one tag rule `v*` each, no branch rule |
| 4 | Verify `SHA256SUMS` before upload; attest the CI build | publish exactly what CI built | ✅ unproven until the first release |
| 5 | Fix the remaining findings below | the pipeline matches L6 decision 9 and SR-PACKAGES decision 11 | ✅ row 12's staging index 2026-10-22: proven on darwin-arm64, the other seven by the first CI run |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Model:** Sonnet: each finding is a concrete, checkable change to a workflow or script.

## Goal

Every finding below is fixed, and the L6 install matrix and runtime fence run on all eight platforms.

## Definition of done

| # | finding | where | severity | state |
|---|---|---|---|---|
| 1 | The clippy step pipes into `tee` under `bash -e` with no `pipefail`, so clippy failures never fail CI; a green verdict is cached and the publish gate trusts it. Fix the warnings it hides first (W-17's last row). | `.github/workflows/ci.yml:118` | 🧨 high | ✅ 2026-10-20: `defaults.run.shell: bash` in both workflows (`-eo pipefail` on every step). CI's lint job fails until W-17 row 21's clippy fixes land |
| 2 | None of the 28 `uses:` is pinned by SHA, including `Swatinem/rust-cache@v2` and the branch ref `pypa/gh-action-pypi-publish@release/v1` in an `id-token: write` job. | `.github/workflows/*.yml` | high | ✅ 2026-10-20: all 28 by full SHA with a `# vX.Y.Z` comment, resolved with `git ls-remote` (same majors as before; `release/v1` → v1.14.3, the same commit) |
| 3 | `npm i -g npm@latest` runs unpinned in both OIDC jobs. | `.github/workflows/publish.yml:78,119` | high | ✅ 2026-10-20: `NPM_VERSION: '11.21.0'`, exact, in both jobs |
| 4 | Environment protection is optional; trusted publishers check workflow and environment, not the ref — require tag-only (`v*`) deployment rules on `npm` and `pypi`. | `RELEASING.md:45` | major (plausible) | ✅ 2026-10-24: both environments exist with one tag rule `v*` and no branch rule (`RELEASING.md` step 5) |
| 5 | `SHA256SUMS` is generated but never verified before upload; no build attestation in `ci.yml` (`--provenance` attests `publish.yml`). | `scripts/package.py:249` | minor | ✅ 2026-10-20: `package.py check` verifies `SHA256SUMS` (exact file set and digests); `ci.yml` attests the set on `main` (`attest-build-provenance`); `publish.yml`'s gate runs `gh attestation verify` on every file (signer `ci.yml`, ref `main`, the release sha), and each upload job re-checks its download against the gate's `SHA256SUMS`. Unproven until the first release |
| 6 | `npm deprecate … && echo` hides a failed deprecate under `set -e`: a partial withdrawal reports success. | `scripts/withdraw.sh:25` | minor | ✅ 2026-10-20: a failed `npm deprecate` is reported and makes the script exit 1 |
| 7 | The SR-guard replay prints "0 commits" and passes when `git rev-list` fails (bad `RULE-SINCE`). | `.github/workflows/ci.yml:94` | minor | ✅ 2026-10-20: `RULE-SINCE` must name a commit, and a failing `git rev-list` fails the step |
| 8 | No PEP 440 conversion: `0.2.0-beta.1` yields an invalid wheel; its metadata differs from `package.py`'s (W-22 fixes the record). | `scripts/build-packs.py:150` | minor | ✅ 2026-10-20: `build-packs.py` uses `package.py`'s `pack_manifest`, `wheel_for_pack` and `pep440`; SR-PACKAGES decision 9 says the one difference (it writes the tarball itself) |
| 9 | Any non-pre-release becomes npm `latest`, so a backport (0.1.5 after 0.2.0) demotes the newer line. | `.github/workflows/publish.yml:128` | minor | ✅ 2026-10-20: a version older than npm's `latest` gets `release-<major>.<minor>` |
| 10 | `actions/checkout` keeps `persist-credentials` (low impact: read-only token). | `.github/workflows/*.yml` | minor | ✅ 2026-10-20: `persist-credentials: false` on all six checkouts |
| 11 | The unquoted regex `^(Number\.)?NaN$` is YAML 1.2 only; quote it for other tools. | `packs/js/rules/js/use-isnan.md:23` | minor | ✅ 2026-10-20: quoted |
| 12 | L6's install matrix and L3's runtime fence run on linux-x64 only, from built files not a staging index (was B-009, B-012). | `scripts/smoke.sh · scripts/staging-index.py · ci.yml` | gap | ✅ the matrix: a `smoke` job on all eight platforms (musl in Alpine containers, darwin-x64 on `macos-15-intel`), `smoke.sh DIST [KEY]`. ✅ 2026-10-22, the staging index: `scripts/staging-index.py` (standard library only) serves an npm registry and a PyPI simple index on 127.0.0.1; `smoke.sh` publishes all of `dist` to it (`npm publish`, `uv publish`) and installs by name from it — `npm i`, `npx`, `pip install`, `uvx` — with caches and config of its own, and fails unless the index served every download. No remote registry, TestPyPI included. Proven by hand on darwin-arm64 (a `dist` whose other seven binaries were stand-ins; a missing wheel fails it); the eight platforms are proven by the first CI run that carries it. Build tool `uv` 0.11.3 (`MIT OR Apache-2.0`) declared in `licences.rs` |

## Blockers

None. The environment rules are set (row 4, 2026-10-24). **Arpit, 2026-10-24:** *"Drive my Chrome"* — the agent created both environments in his browser; checked on the Environments page: `npm` and `pypi`, 1 protection rule each. Left: the first CI run on `main` proves row 12 on the eight platforms.

## Record affected

[SR-PACKAGES](../../records/0107_packages.md) decisions 9, 11, 12 · [SR-LAW-2](../../records/0004_LAW-2-foss-dependencies.md) §Consequences (the staging index's tool) · [SR-LAW-6](../../records/0008_LAW-6-python-and-node-packages.md) §How L6 is enforced · `RELEASING.md`.

## Hazards

- **Changing `ci.yml` resets every cache key** (W-04 §Measured): batch the workflow edits into one change.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
