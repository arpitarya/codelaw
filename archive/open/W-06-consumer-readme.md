---
type: Handoff
name: W-06
description: "Arpit, 2026-10-20: a consumer-facing README that also serves as marketing, plus the files a public repo needs."
item: W-06
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-22
updated_by: "Arpit (Cowork, Opus 5.5) — the templates placed; closed"
ball: agent
---

# W-06 — Write the README and the public-repo files

*Filed as W-04; renumbered W-06 on 2026-10-20 when the release-pipeline branch, which already used W-04, was merged. Older worklog entries say W-04.*

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Research how leading dev-tool READMEs (Ruff, Biome) are built | a structure consumers recognise | ✅ 2026-10-20 |
| 2 | Build the binary, run every README command on a sample repo | output in the README is real, not invented | ✅ 2026-10-20 |
| 3 | Write `README.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md` | a repo a consumer can adopt and contribute to | ✅ 2026-10-20 |
| 4 | Add the `.github/` issue and PR templates | structured false-positive and rule-request reports | ✅ 2026-10-22 — `false_positive.yml`, `bug_report.yml`, `rule_request.yml`, `config.yml` (blank issues off, security link) and `pull_request_template.md`; written fresh on the decider's go-ahead, placed through his linked computer |
| 5 | The decider picks the project licence | the README's licence section, and publishing (W-03 step 6) | ✅ 2026-10-20 — MIT; README, CHANGELOG and CONTRIBUTING say so |

**Arpit, 2026-10-20:** *"create the readme file and other necessary files. research and create one which is helpful for consumer and could be used for marketing as well"*

**Model:** Opus — public copy whose accuracy against the built engine is the deliverable.

## Goal

A README that sells codelaw honestly: what it does today, proved with real output, and what is still designed but not built.

## Definition of done

- Every command and output block in the README was run against the 0.1.0 binary.
- Built versus not built matches the open items W-07–W-21 in `work/OPEN-WORK.md`.
- The new docs pass `codelaw:md/recommended` and `codelaw:data/recommended`.

## Hazards

- The README states counts (419 rules, 3,937 checks, fix-class totals, refreshed 2026-10-22); they go stale when a pack changes — the registry row names that trigger.

**Closed 2026-10-22:** the decider asked for the templates to be placed; the 2026-10-20 copies never reached the repo, so they were written again to match `CONTRIBUTING.md`'s template links.

**Record affected:** SR-WORK-REGISTRY §3 (rows for the five new docs). No other SR affected.
