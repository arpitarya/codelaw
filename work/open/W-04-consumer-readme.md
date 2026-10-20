---
type: Handoff
name: W-04
description: "Arpit, 2026-10-20: a consumer-facing README that also serves as marketing, plus the files a public repo needs."
item: W-04
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-20
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-04 — Write the README and the public-repo files

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Research how leading dev-tool READMEs (Ruff, Biome) are built | a structure consumers recognise | ✅ 2026-10-20 |
| 2 | Build the binary, run every README command on a sample repo | output in the README is real, not invented | ✅ 2026-10-20 |
| 3 | Write `README.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md` | a repo a consumer can adopt and contribute to | ✅ 2026-10-20 |
| 4 | Add the `.github/` issue and PR templates | structured false-positive and rule-request reports | 🔴 decider places them — `.github/` is protected from remote writes |
| 5 | The decider picks the project licence | the README's licence section, and publishing (W-03 step 6) | ✅ 2026-10-20 — MIT; README, CHANGELOG and CONTRIBUTING say so |

**Arpit, 2026-10-20:** *"create the readme file and other necessary files. research and create one which is helpful for consumer and could be used for marketing as well"*

**Model:** Opus — public copy whose accuracy against the built engine is the deliverable.

## Goal

A README that sells codelaw honestly: what it does today, proved with real output, and what is still designed but not built.

## Definition of done

- Every command and output block in the README was run against the 0.1.0 binary.
- Built versus not built matches `work/BACKLOG.md` B-028–B-041.
- The new docs pass `codelaw:md/recommended` and `codelaw:data/recommended`.

## Hazards

- The README states counts (401 rules, 3,060 checks, fix-class totals); they go stale when a pack changes — the registry row names that trigger.

**Closes when** the decider has placed the `.github/` templates (sent in the session, 2026-10-20).

**Record affected:** SR-WORK-REGISTRY §3 (rows for the five new docs). No other SR affected.
