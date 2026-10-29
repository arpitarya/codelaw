---
type: Handoff
name: W-119
description: "Arpit, 2026-10-29 (voice): create the ten PyPI projects still missing through CI, one pending publisher at a time — a project CI creates that way does not count against PyPI's four-new-projects-a-day limit. Replaces PyPI days 3 and 4 and the spell first publish."
item: W-119
filed: 2026-10-29
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-29
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-119 — The missing PyPI projects through CI, one pending publisher at a time

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Records first, then `publish.yml`: a run by hand that publishes **one named PyPI project** from a release's build, through its pending publisher | CI can create a new project without a release | ⏳ agent |
| 2 | The loop, per project: Claude in Chrome registers the pending publisher (`arpitarya/codelaw` · `publish.yml` · `pypi`) → Claude gives the command → Arpit runs it → CI creates the project → the pending publisher becomes its trusted publisher → the next | all ten created today, no limit | ⏳ Arpit + Claude |
| 3 | npm, separately: `@codelaw/rules-spell-fux-index`'s first version by hand (`npm publish`), then `scripts/npm-trust.sh --fux-index` | the one npm name missing | ⏳ Arpit |
| 4 | Check: each of the ten shows a version and a trusted publisher | done | ⏳ |

**Arpit, 2026-10-29 (voice):** asked whether pending publishers plus his own uploads could beat the limit; told a `twine` upload counts however the project is registered, and only a project **CI** creates through a personal pending publisher is not rate-limited (warehouse `oidc/views.py`: `ratelimited=pending_publisher.organization_id is not None`), one pending publisher per repository · workflow · environment at a time. Then: *"Okay, here is what you do. Let's do one by one. Um, one package in pending state, then you give me the command. Then another package in pending state, then you give me the command. I'll execute the command manually, and once I give it to you, you enter the next to pending state. Let's do that."* — the command each time being a workflow run, not `twine` — and *"That's all right. Let's go by that approach. Uh, let's create a work item remove all the existing publishing work items and we'll go with this approach."* Agreed: W-88, W-89 and W-115 close into this; the npm spell index stays a separate step: *"Yes, it does."*

**Model:** Opus — a `publish.yml` change with OIDC in it.

## The ten PyPI projects

| project | in the v0.3.0 `dist` | was |
|---|---|---|
| `codelaw-rules-scss` | `dist/pypi/` (0.3.0) | day 3 (W-88) |
| `codelaw-skills` | `dist/pypi/` (0.3.0; needs `codelaw==0.3.0`, on PyPI) | day 3 (W-88) |
| `codelaw-rules-security-js` | `dist/pypi/` (0.3.0) | day 4 (W-89) |
| `codelaw-rules-security-html` | `dist/pypi/` (0.3.0) | day 4 (W-89) |
| `codelaw-rules-security-python` | `dist/pypi/` (0.3.0) | day 4 (W-89) |
| `codelaw-rules-spell` | `dist/pypi/` (0.3.0) | W-115 |
| `codelaw-rules-security-html-fux-index` | the `fux-index` artifact, not `dist` | day 3 (W-88) |
| `codelaw-rules-security-python-fux-index` | the `fux-index` artifact | day 3 (W-88) |
| `codelaw-rules-security-js-fux-index` | the `fux-index` artifact | day 4 (W-89) |
| `codelaw-rules-spell-fux-index` | the `fux-index` artifact | W-115 |

## What step 1 must solve (found 2026-10-29)

- **Which ref runs it.** `workflow_dispatch` runs the workflow file *at the ref*: the v0.3.0 tag's `publish.yml` has no such input, and the `pypi` environment admits only `v*` tags. Either the run goes from `main` (the `pypi` environment then admits `main` — a settings change for Arpit's OK), or a v0.3.1 tag carries the change. Name the trade-off before building.
- **Which files.** The six packs and skills are in the v0.3.0 run's `dist` (SHA256SUMS, the gate's attestation check); the four Fux index wheels are in its `fux-index` artifact, which `publish.yml` does not read yet (B-063) — read it here, or leave those four to `twine` within the daily limit.
- **The skip rule.** Today's `exists` step uploads a wheel only if its project exists or is `codelaw`; the run by hand uploads exactly the named project, and refuses a name with no pending publisher.
- **One at a time.** PyPI keeps one pending publisher per repository · workflow · environment; the next is registered only after the last run created its project.

## Definition of done

- The ten projects exist on PyPI, each trusting `arpitarya/codelaw` · `publish.yml` · `pypi`; `@codelaw/rules-spell-fux-index` on npm, trusted.

## Record affected

[SR-LAW-6](../../records/0008_LAW-6-python-and-node-packages.md) decision 9 (first uploads) · [SR-PACKAGES](../../records/0107_packages.md) · RELEASING.md.

## Hazards

- If warehouse ever rate-limits personal pending publishers, a run fails with 429 like `twine` — the fallback is the old four-a-day `twine` days (W-88/W-89, archived).
- A pending publisher does not reserve the name: register it right before the run.
