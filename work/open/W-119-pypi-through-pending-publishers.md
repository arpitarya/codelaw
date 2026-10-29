---
type: Handoff
name: W-119
description: "Arpit, 2026-10-29 (voice): create the ten PyPI projects still missing through CI, one pending publisher at a time — a project CI creates that way does not count against PyPI's four-new-projects-a-day limit. Replaces PyPI days 3 and 4 and the spell first publish."
item: W-119
filed: 2026-10-29
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-29
updated_by: "Claude Code (Opus 5.5)"
ball: arpit
---

# W-119 — The missing PyPI projects through CI, one pending publisher at a time

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Records first, then `publish.yml`: a run by hand that publishes **one named PyPI project** from a release's build, through its pending publisher | CI can create a new project without a release | ✅ 2026-10-29 — `only=pypi-new`, from `main`; `pypi` admits `main` |
| 2 | The loop, per project (§The loop): Claude in Chrome — or Arpit — registers the pending publisher (`arpitarya/codelaw` · `publish.yml` · `pypi`) → Claude gives the command → Arpit runs it → CI creates the project → the pending publisher becomes its trusted publisher → the next | all ten created today, no limit | ⏳ Arpit + Claude — next: `codelaw-rules-scss` |
| 3 | npm, separately: `@codelaw/rules-spell-fux-index`'s first version by hand (`npm publish`), then `scripts/npm-trust.sh --fux-index` | the one npm name missing | ✅ 2026-10-29 10:14 — `@codelaw/rules-spell-fux-index@0.3.0` published by Arpit from v0.3.0's CI `fux-index` artifact (run 40756572636); trust created (`arpitarya/codelaw` · `publish.yml` · `npm`, id `23e01fe0-…`); `npm view` answered 404 at 10:20, `0.3.0` at 10:22 |
| 4 | Check: each of the ten shows a version and a trusted publisher | done | ⏳ |

**Arpit, 2026-10-29 (voice):** asked whether pending publishers plus his own uploads could beat the limit; told a `twine` upload counts however the project is registered, and only a project **CI** creates through a personal pending publisher is not rate-limited (warehouse `oidc/views.py`: `ratelimited=pending_publisher.organization_id is not None`), one pending publisher per repository · workflow · environment at a time. Then: *"Okay, here is what you do. Let's do one by one. Um, one package in pending state, then you give me the command. Then another package in pending state, then you give me the command. I'll execute the command manually, and once I give it to you, you enter the next to pending state. Let's do that."* — the command each time being a workflow run, not `twine` — and *"That's all right. Let's go by that approach. Uh, let's create a work item remove all the existing publishing work items and we'll go with this approach."* Agreed: W-88, W-89 and W-115 close into this; the npm spell index stays a separate step: *"Yes, it does."*

**Model:** Opus — a `publish.yml` change with OIDC in it.

## The ten PyPI projects

| project | in the v0.3.0 `dist` | was | created |
|---|---|---|---|
| `codelaw-rules-scss` | `dist/pypi/` (0.3.0) | day 3 (W-88) | ⏳ |
| `codelaw-skills` | `dist/pypi/` (0.3.0; needs `codelaw==0.3.0`, on PyPI) | day 3 (W-88) | ⏳ |
| `codelaw-rules-security-js` | `dist/pypi/` (0.3.0) | day 4 (W-89) | ⏳ |
| `codelaw-rules-security-html` | `dist/pypi/` (0.3.0) | day 4 (W-89) | ⏳ |
| `codelaw-rules-security-python` | `dist/pypi/` (0.3.0) | day 4 (W-89) | ⏳ |
| `codelaw-rules-spell` | `dist/pypi/` (0.3.0) | W-115 | ⏳ |
| `codelaw-rules-security-html-fux-index` | the `fux-index` artifact, not `dist` | day 3 (W-88) | ⏳ |
| `codelaw-rules-security-python-fux-index` | the `fux-index` artifact | day 3 (W-88) | ⏳ |
| `codelaw-rules-security-js-fux-index` | the `fux-index` artifact | day 4 (W-89) | ⏳ |
| `codelaw-rules-spell-fux-index` | the `fux-index` artifact | W-115 | ⏳ |

## The loop — one project at a time

Order: the table above, top to bottom. For each name:

1. **Pending publisher** — <https://pypi.org/manage/account/publishing/> → *Add a new pending publisher* → GitHub: PyPI project name `<name>`, owner `arpitarya`, repository `codelaw`, workflow `publish.yml`, environment `pypi`. Right before the run: it does not reserve the name.
2. **Run** — `gh workflow run publish.yml -R arpitarya/codelaw --ref main -f only=pypi-new -f release=v0.3.0 -f project=<name>`, then `gh run watch -R arpitarya/codelaw $(gh run list -R arpitarya/codelaw -w publish.yml -L1 --json databaseId -q '.[0].databaseId')`.
3. **Say "done"** — the agent checks <https://pypi.org/pypi/<name>/json> shows 0.3.0 and the project trusts `publish.yml` · `pypi`, ticks the row, and the next name starts at 1.

## Step 1, as built (2026-10-29, Claude Code)

- **The ref — ruled by Arpit, 2026-10-29:** *"Admit main in pypi"* (offered: that, a v0.3.1 tag, or `main` admitted only for W-119). The `pypi` environment has the branch rule `main` beside the tag rule `v*`, as `vscode` has for W-95. Applied the same day (`gh api …/environments/pypi/deployment-branch-policies`).
- **`publish.yml`:** `workflow_dispatch` takes `only=pypi-new`, `release` and `project`. The gate resolves the tag's commit and runs every check on it — its ci.yml run on `main`, its `dist`, the attestation for that commit, the tag's own scripts (`checkout` at that commit). `trust` and `npm` run only for a Release or `only=npm`; `pypi` runs for a Release or `pypi-new`. In `pypi-new` the job copies exactly the named wheel, at the tag's version, refuses a name the build has no wheel of and a project already on PyPI, and uploads it through `gh-action-pypi-publish`; a `-fux-index` name comes from the same run's `fux-index` artifact, checked against its `SHA256SUMS` — not attested (ci.yml attests `dist/SHA256SUMS` only). `half-published` and `attach` do not run. actionlint clean; untested until the first run.
- **Checked:** all ten names 404 on PyPI (10:30); v0.3.0's ci.yml run 40756572636 holds `dist` and `fux-index` until 2026-11-27, and its `fux-index` has the four index wheels at 0.3.0, sums good.
- **Records:** [SR-LAW-6](../../records/0008_LAW-6-python-and-node-packages.md) decision 9 (a new PyPI project is created by CI), [SR-PACKAGES](../../records/0107_packages.md) decision 11 (the run, `main` on `pypi`); RELEASING.md step 4 and 5.

## What step 1 had to solve (found 2026-10-29)

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
