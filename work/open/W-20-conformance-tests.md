---
type: Handoff
name: W-20
description: "Arpit, 2026-10-20: the checks the laws and records name that nothing runs."
item: W-20
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-20
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-20 — Add the missing law and record conformance tests

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | L5 network fence: the engine runs with networking denied | L5 checked | ⏳ |
| 2 | L2 licence check over the npm and PyPI build tools | L2 checked beyond Cargo | ⏳ |
| 3 | `init` writes only the paths SR-CONSUMER-LAYOUT lists | the consumer tree checked | ⏳ |
| 4 | Rule promotion: stable only after all four test levels, incl. a corpus run | SR-RULE-AUTHORING checked | ⏳ |
| 5 | Work-system tests: queue rows, owns ↔ ownership, the registry, lost `W-nn`s, a second `archive/` | the work system checked | ⏳ |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Model:** Sonnet: each test has a precise pass condition stated in its record.

## Goal

Each check below runs in `cargo test` or `ci.yml` and fails on a violation.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · L5 | A test runs `check` and `fix` with networking denied and expects identical output (SR-LAW-5). | ⏳ |
| 2 · L2 | The licence check covers the npm/PyPI build tools as well as Cargo. | ⏳ |
| 3 · init | An `init` fixture compares `git status` with SR-CONSUMER-LAYOUT §1. | ⏳ |
| 4 · promotion | `codelaw test --corpus` and a CI job that promotes a rule to `stable` only after levels 1–4. | ⏳ |
| 5 · work system | Tests for SR-WORK-OPEN-QUEUE rows (13–19, 42, 45), SR-WORK-OWNERSHIP decision 2, SR-WORK-REGISTRY decisions 5–8, lost `W-nn`s (the record's own check command misreports: `ls a b` fails when either is missing), one `archive/`. | ⏳ |

## Blockers

None.

## Record affected

[SR-LAW-2](../../records/0004_LAW-2-foss-dependencies.md) · [SR-LAW-5](../../records/0007_LAW-5-no-model-in-the-engine.md) · [SR-CONSUMER-LAYOUT](../../records/0111_consumer-layout.md) · [SR-RULE-AUTHORING](../../records/0113_rule-authoring.md) · the WORK records 0051–0060.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
