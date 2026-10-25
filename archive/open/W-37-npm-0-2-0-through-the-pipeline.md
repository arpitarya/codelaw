---
type: Handoff
name: W-37
description: "Arpit, 2026-10-25: 0.2.0 on npm from CI now, without waiting for PyPI — publish.yml gains an npm-only run by hand on a v* tag (L6 decision 9 amended); Claude Code commits it, tags the release, runs it, and checks all 19 npm names from the registry on the eight platforms."
item: W-37
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-25
updated_by: "Claude Code (Opus 5.5), 2026-10-25: closed — npm 0.2.0 live from CI"
ball: done
---

# W-37 — 0.2.0 on npm, through the pipeline, now

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Let `publish.yml` publish npm on its own: a run by hand on a `v*` tag publishes npm only; the GitHub Release later does both and skips what is there | npm need not wait for PyPI | ✅ 2026-10-25 (Cowork), committed `08b7158`: `publish.yml`, L6 decision 9, SR-PACKAGES decision 11, `RELEASING.md` |
| 2 | Claude Code checks the workflow (`actionlint`), commits it with explicit paths, and Arpit pushes `main` | the change is on `main` | ✅ 2026-10-25: `actionlint` 1.7.12 clean; committed `08b7158`, pushed |
| 3 | Wait for `ci.yml` green on that commit; tag it `v0.2.0` and push the tag | the release commit — the one PyPI's release (W-38) reuses | ✅ 2026-10-25: CI run 40194930874 green on `08b7158`; annotated tag `v0.2.0` on it, pushed |
| 4 | Run `gh workflow run publish.yml --ref v0.2.0` and watch it | the engine, eight binaries and eight packs at 0.2.0 on npm, from CI, with provenance; the two new packs skipped as already there | ✅ 2026-10-25: run 40207195373 — gate, npm trust preflight, npm green; `pypi`, `attach` skipped; 5 min 21 s |
| 5 | Check from the registry: all 19 npm names at `latest` 0.2.0, provenance on each; `npm i codelaw && npx codelaw --version` on the Mac; the install smoke from the registry on the eight platforms | npm works for real | 🟡 2026-10-25: 19/19 at `latest` 0.2.0; provenance on 17 (the two hand-published W-32 packs have none, and a published version cannot gain it); `npm i codelaw@0.2.0 && npx codelaw --version` → `codelaw 0.2.0` on the Mac (darwin-arm64); the eight-platform install from the registries moved to W-38 (below) |
| 6 | Write it down: `IMPLEMENTATION.md`, the publish run's time in `RELEASING.md`, CHANGELOG, the README's version badge and preview note | the docs say npm is on 0.2.0 | ✅ 2026-10-25: IMPLEMENTATION, RELEASING's time, README badge, preview note and roadmap; the 0.2.0 release notes are CHANGELOG's section (`88b9c6f`) |

**Arpit, 2026-10-25:** *"Everything else related to npm, let's go ahead and
build it out, even have the pipelines start working and start publishing those
packages. Do whatever you can over here. Whatever Claude Code needs to do,
create separate work items for it."* That is also the go for steps 3–4: Claude
Code tags and runs the publish with Arpit's `gh` on his Mac.

**Model:** Claude Code, Sonnet — it needs `gh`, `git push` of a tag and a
workflow run, which the Cowork bridge does not have.

## What changed in step 1

- **`publish.yml`** gains `workflow_dispatch`. The gate's first step refuses a
  run by hand that is not on a `v*` tag; the `pypi` and `attach` jobs run only
  for a published Release. Everything else — CI green on that commit on `main`,
  `SHA256SUMS`, the attestation check, `law-gate.py`, the npm trust preflight,
  the skip-what-exists loop, `half-published` — is unchanged.
- **L6 decision 9** says npm may go first, on the same tag, by the decider's
  ruling of 2026-10-25; "together" means the same release commit, not the same
  hour.
- **The npm and pypi environments' tag rule `v*`** still holds: a run by hand
  from a branch is refused at the environment as well as by the gate.

## Absorbed (archived the same day)

| from | what came here |
|---|---|
| [W-29](../../archive/open/W-29-first-npm-publish.md) | its DoD 2: the install from the registry on the eight platforms, moved to 0.2.0 |
| [W-31](../../archive/open/W-31-registry-pack-split.md) | step 7's npm half: the 17 tarballs at 0.2.0 from CI (its two hand-published names and `npm-trust.sh` were done) |
| [W-04](../../archive/open/W-04-release-pipeline.md) | step 3's "first live run" and its timing |

## Definition of done

- `npm view <name> dist-tags.latest` is `0.2.0` for all 19 names; each 0.2.0
  shows provenance from `publish.yml`.
- ~~The install check passes on the eight platforms.~~ **Moved to [W-38](W-38-pypi-0-2-0-by-hand.md), 2026-10-25 (Claude Code):** `scripts/smoke.sh` installs only from a staging index, so a registry check needs a new CI job; run once after the Release, it covers npm and PyPI together. The Mac install from npm passed.
- The publish run's id and time are in `RELEASING.md` and `IMPLEMENTATION.md`.

## Hazards

- **The tag must be on a commit that has this `publish.yml`** — a run by hand
  reads the workflow from the tag, so the older `f33b772` cannot be used.
- **One tag, kept:** W-38's GitHub Release reuses it, so PyPI gets the same
  binary as npm (L6 decision 2). Never move or re-create `v0.2.0`.
- **`dist` lives 30 days**; W-38 must finish inside that, or re-run CI on the tag's commit.

**Records affected:** L6 decision 9 and SR-PACKAGES decision 11 (amended in step 1).
