---
type: Handoff
name: W-125
description: "Arpit, 2026-10-29 10:31: publish.yml's only=pypi-new was temporary — revert it, and the main rule on the pypi environment, now that W-119 created all ten projects."
item: W-125
filed: 2026-10-29
filed_by: "Arpit (Claude Code, Opus 5.5)"
updated: 2026-10-29
updated_by: "Claude Code (Opus 5.5)"
ball: agent
---

# W-125 — revert `only=pypi-new` (W-119 step 5)

## At a glance

| step | what we do | state |
|---|---|---|
| 1 | Take `only=pypi-new` out of `publish.yml` (the input option, the `release`/`project` inputs, the job) | ✅ 2026-10-29 — the gate and `pypi` job are pre-W-119's again (W-126's parallel checks kept) |
| 2 | Take the branch rule `main` off the `pypi` environment (`gh api …/environments/pypi/deployment-branch-policies`); `v*` stays | ✅ 2026-10-29 — policy 64746051 deleted; `v*` (64202900) the only rule |
| 3 | Records and RELEASING.md back to *a release uploads only existing projects*: L6 decision 9, SR-PACKAGES decision 11, RELEASING.md steps 4 and the environment note | ✅ 2026-10-29 — and CHANGELOG's W-119 line |

**Arpit, 2026-10-29 10:31:** *"publish only one PyPI package temporarily. Once all of them are published, then you will revert the pipeline to original"*.

## Why this is its own item

The ruling was written as W-119's step 5 on the branch `lm/W-119-loop`, which never landed. W-119 was then closed in Cowork (all ten projects created, 11:25) without it, with the note *"`only=pypi-new` stays for the next new project"*. The note was written without the ruling. The branch was dropped in the 2026-10-29 worktree cleanup (W-120–W-124), so this item carries the ruling.

## Record affected

[SR-LAW-6](../../records/0008_LAW-6-python-and-node-packages.md) decision 9 and SR-PACKAGES decision 11 — both state the `only=pypi-new` path.

**Closed 2026-10-29 (Claude Code):** all three steps done; the next push of `main` carries the workflow change (the environment rule is already live).
