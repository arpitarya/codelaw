---
type: Handoff
name: W-04
description: "Arpit, 2026-10-20: a CI/CD pipeline that publishes codelaw's packages to npm and PyPI, modelled on fux and sheetpane, in about two minutes or less."
item: W-04
filed: 2026-10-20
filed_by: "Arpit (Claude Code, Opus 5.5)"
updated: 2026-10-20
updated_by: "Arpit (Claude Code, Opus 5.5)"
ball: agent
---

# W-04 — The release pipeline: build once in CI, publish what CI built

**Arpit, 2026-10-20:** *"checkout fux and sheetpane for how the cicd pipeline
are built how pypi and npm packages are published — build a cicd pipeline for
codelaw to publish packages it should take around 2 mins even faster."*

**His rulings the same day:** the licence is **MIT** (first recorded here as `MIT OR Apache-2.0`; ruled MIT only at the merge, SR-PACKAGES decision 10); CI builds
the binaries and **publish reuses them**; the pipeline is timed on a scratch
branch of the real repo, publishing nothing; and, after the first timings,
**pin the Rust toolchain** (1.99.1).

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Packaging (B-038): npm `codelaw` launcher + API + types, 8 `@codelaw/cli-*`, PyPI wheels via maturin, 5 packs on both | every package SR-PACKAGES §1 names | ✅ 2026-10-20 |
| 2 | `ci.yml`: tests, clippy, the SR-guard replay, 8 parallel platform builds, assemble + check + install smoke | a checked `dist` on every push | ✅ 2026-10-20 |
| 3 | `publish.yml`: gate on CI, upload that run's `dist` to npm and PyPI by OIDC | a release with no token and no rebuild | ✅ written; ⏳ first live run |
| 4 | Time it on real runners and cut it to ~2 min | the target Arpit set | ✅ 63 s for a non-code push, 2 m 45 s for a code push (§Measured) |
| 5 | The first release | codelaw on both registries | 🔴 the one-time registry setup, then the ruling to publish |

## What was taken from fux and sheetpane

| from | kept | changed for codelaw |
|---|---|---|
| both | publish only on a published GitHub Release; refuse unless `ci.yml` succeeded on that sha on `main`; OIDC trusted publishing, no token; `--provenance`; a `-` version goes to `next`; skip a version already published | — |
| sheetpane | ask npm, per package, whether it trusts the workflow **before** publishing anything | — |
| fux | build once, upload one artifact, every publish job downloads it — never rebuild per registry | fux builds in `publish.yml`; codelaw builds in `ci.yml` and `publish.yml` only downloads, so a release compiles nothing |
| fux | dependents last (`fux-engine` after its role packages) | `codelaw` after the eight binaries and five packs |
| fux | FAST stage ~2 min | codelaw is one stage: every job is parallel except `package` |

## Measured

GitHub-hosted runners, public repo, 2026-10-20, branch `ci/w-04-release-pipeline`.
Wall time is first job start to last job end.

| push | what changed | `ci.yml` wall time | slowest job |
|---|---|---|---|
| **docs / records / packs / `work/` only** | no engine input | **63 s** | all eight wheels reused; `package` 24 s |
| **engine code**, warm Rust cache | one crate | **2 m 45 s** | Windows arm64 and x64 builds, ~2 m 10 s each |
| cold: a new toolchain or `ci.yml` itself | every cache key | 4 m 14 s | Windows, plus installing the toolchain |

- **What makes the fast row fast:** each build keys its wheel by every input that goes into it (sources, `Cargo.lock`, the pinned rustc, maturin, the build steps). Test and clippy key a green verdict the same way. The binary's `--version`, every pack's tests and the whole `package` job still run on every push.
- **What bounds the code row:** one LTO release build on a Windows runner. Measured and rejected as levers: `codegen-units = 16` (slower here, 1 m 44 s vs 1 m 26 s) and cross-compiling Windows from Linux (the Windows binary would then never run before release).
- **`publish.yml` compiles nothing:** a gate (~15 s), then npm (14 uploads) and PyPI (13 files) in parallel. Not yet timed live — nothing may be published before the decider's setup and ruling.
- **The nightly run** (02:00 IST) and any manual dispatch ignore every saved verdict and rebuild all eight from scratch.

## Progress

- **2026-10-20:** packaging, `ci.yml`, `publish.yml`, `RELEASING.md`, `LICENSE-*`. Local proof on linux-x64: maturin wheel `manylinux_2_17` with the binary, the Python module and a CycloneDX SBOM; `package.py check` passes the 14 tarballs + 5 pack wheels; `smoke.sh` — npm with no Python on `PATH` and PyPI with no Node on `PATH` give byte-equal JSON over the L3 fixtures + `rules-data` (16 diagnostics), and both APIs agree.
- **Independent review** (a separate agent, read-only) found five defects, all fixed: the `next` tag matched the hyphen in package names; PyPI could publish while npm's trust check failed (the check is now a job both wait on); `version.py` accepted pre-releases PyPI can't take; a `dist` older than 30 days (documented); the Python API could pick up another `codelaw` on the same bin folder (it now reads its own file list).
- **The SR guard now runs in CI** (SR-WORK-OWNERSHIP decision 7): `SR_GUARD_COMMIT=<sha>` replays a commit against the register as it stood then; the 21 commits since `RULE-SINCE` all pass.

## Blockers

- **Step 5 waits on W-07, W-08 and W-09** (2026-10-20): L7 and L6 decision 3 forbid releasing an engine without the formatter, MCP server and LSP. `scripts/law-gate.py` now refuses such a release in `publish.yml`; `ci.yml` reports it on every push.
- **Step 5 is the decider's:** the `@codelaw` npm org, the first hand publish of the 14 npm packages, `scripts/npm-trust.sh`, six PyPI pending publishers and the `npm` / `pypi` GitHub environments ([`RELEASING.md`](../../RELEASING.md) §One time) — then his ruling to publish, which is W-03 step 6.

## Not done here (backlog)

- **The JS/Python API has `check` and `fix` only** — SR-PACKAGES decision 3's `rulesFor` and `format`, and `codelaw.schema.json`, need a `rules --format json`, the `format` command (W-07) and the schema (W-16): `W-16`.
- **The install matrix runs on linux-x64 only**; the other seven platforms run `--version` and every pack's tests on their own binary: `W-18`.

## Record affected

SR-PACKAGES decisions 10–12 (licence, the pipeline, withdrawal) and its owns · SR-LAW-6 Consequences and veto check (facts) · SR-WORK-OWNERSHIP decision 7 (the CI replay) · the register's ownership table.
