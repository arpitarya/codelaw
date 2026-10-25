---
type: Handoff
name: W-37
description: "Arpit, 2026-10-25: 0.2.0 on npm from CI now, without waiting for PyPI — publish.yml gains an npm-only run by hand on a v* tag (L6 decision 9 amended); Claude Code commits it, tags the release, runs it, and checks all 19 npm names from the registry on the eight platforms."
item: W-37
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-25
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-37 — 0.2.0 on npm, through the pipeline, now

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Let `publish.yml` publish npm on its own: a run by hand on a `v*` tag publishes npm only; the GitHub Release later does both and skips what is there | npm need not wait for PyPI | ✅ 2026-10-25 (Cowork), in the tree, uncommitted: `publish.yml`, L6 decision 9, SR-PACKAGES decision 11, `RELEASING.md` |
| 2 | Claude Code checks the workflow (`actionlint`), commits it with explicit paths, and Arpit pushes `main` | the change is on `main` | 🟡 2026-10-25: `actionlint` 1.7.12 clean on `publish.yml` and `ci.yml`; committed; left: the push |
| 3 | Wait for `ci.yml` green on that commit; tag it `v0.2.0` and push the tag | the release commit — the one PyPI's release (W-38) reuses | ⏳ Claude Code |
| 4 | Run `gh workflow run publish.yml --ref v0.2.0` and watch it | the engine, eight binaries and eight packs at 0.2.0 on npm, from CI, with provenance; the two new packs skipped as already there | ⏳ Claude Code |
| 5 | Check from the registry: all 19 npm names at `latest` 0.2.0, provenance on each; `npm i codelaw && npx codelaw --version` on the Mac; the install smoke from the registry on the eight platforms | npm works for real | ⏳ Claude Code |
| 6 | Write it down: `IMPLEMENTATION.md`, the publish run's time in `RELEASING.md`, CHANGELOG, the README's version badge and preview note | the docs say npm is on 0.2.0 | ⏳ Claude Code |

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
- The install check passes on the eight platforms.
- The publish run's id and time are in `RELEASING.md` and `IMPLEMENTATION.md`.

## Hazards

- **The tag must be on a commit that has this `publish.yml`** — a run by hand
  reads the workflow from the tag, so the older `f33b772` cannot be used.
- **One tag, kept:** W-38's GitHub Release reuses it, so PyPI gets the same
  binary as npm (L6 decision 2). Never move or re-create `v0.2.0`.
- **`dist` lives 30 days**; W-38 must finish inside that, or re-run CI on the tag's commit.

**Records affected:** L6 decision 9 and SR-PACKAGES decision 11 (amended in step 1).
