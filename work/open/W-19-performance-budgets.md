---
type: Handoff
name: W-19
description: "Arpit, 2026-10-20: L8's budgets, which are all unset because nothing has been measured."
item: W-19
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-25
updated_by: "Claude (Opus 5.5), cloud docket 2026-10-25"
ball: agent
---

# W-19 — Build `codelaw bench`, measure, and propose the L8 budgets

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Build `codelaw bench` and the reference workload | a repeatable measurement | ✅ 2026-10-22 |
| 2 | Measure the SR-LINTER and SR-ARCHITECTURE targets (warm/cold 10k files, one-file hook, fix, LSP edit, startup, peak memory) | numbers to set budgets from | 🟡 measured on a loaded dev Mac; CI's runner still to measure |
| 3 | Propose `perf/budgets.toml` values | the decider sets the budgets | 🔴 proposals written; the ruling is the decider's |
| 4 | Add the CI perf gate | a budget is never exceeded | ✅ 2026-10-22 |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Model:** Opus: a workload that represents real repos and a stable measurement on shared runners are judgment calls.

## Goal

`perf/budgets.toml` holds measured budgets the decider set, and CI fails a change that exceeds one.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · bench | `codelaw bench` runs the reference workload and prints each budgeted operation. | ✅ 2026-10-22 — [SR-CLI-MCP](../../records/0115_cli-and-mcp.md) decision 10; `crates/codelaw-cli/src/bench/`, `tests/bench.rs` |
| 2 · measure | Every target in SR-ARCHITECTURE decision 29 and SR-LINTER decision 8 is measured on named hardware. | 🟡 2026-10-22 on an Apple M4 / 16 GiB Mac under load ([SR-LAW-8](../../records/0010_LAW-8-performance-budgets.md) §Measurements); the CI runner's numbers come from the `perf` job's first run on `main` |
| 3 · propose | A budget proposal goes to the decider (inbox row). | ✅ 2026-10-25: the decider ruled the method (below); left: apply it to CI's three runs and write each `budget` with `set_by`, `set_on`, `measured_p95` |
| 4 · gate | CI compares each run against `perf/budgets.toml` ([L8](../../records/0010_LAW-8-performance-budgets.md) decision 7). | ✅ 2026-10-22 — `ci.yml` `perf`; fails only on a set budget (`budget = <number>`), reports `unset` ones |

## Blockers

The budgets are the decider's to set (L8 decision 4), from the CI runner's numbers. **Arpit, 2026-10-24 (docket):** *"Both as recommended"* — the reference machine is GitHub's hosted `ubuntu-24.04` runner; the three pinned real repos are `djangoproject.com`, `full-stack-fastapi-template` and `html5-boilerplate`, at the commits in `scripts/corpus.json`. Left for an agent: pin the `perf` job to `ubuntu-24.04` and add the three repos to `codelaw bench`. **Both done:** the pin in `483b2a1` (2026-10-24); `codelaw bench --repo <dir>` (repeatable, `check_cold_repo` / `check_warm_repo` context rows on a copy with the workload's config and packs) and the `perf` job fetching the three with `scripts/corpus.py fetch --repo` on 2026-10-25 — SR-CLI-MCP decision 10 and SR-LAW-8 §as built say so. Checked locally on the three pinned commits (debug binary, loaded Mac: numbers not comparable). CI's `perf` ran three times on `main` (40124005285, 40124756694, 40124878075). **Arpit, 2026-10-25 (docket):** *"A: formula + guard"* — each budget is min(design target, 1.5 × the worst p95 across the `perf` job's three `ubuntu-24.04` runs on `main` (40124005285, 40124756694, 40124878075)), rounded up; where the design target is below 1.5 × that p95, the budget is not set and comes back to the inbox. **Left for an agent on the Mac** (the logs need a signed-in `gh`): `gh run view <id> --log --job <perf job>` for each run, the worst p95 per operation, the budget by the rule, and a new inbox row for any operation the guard holds back.

## Record affected

[SR-LAW-8](../../records/0010_LAW-8-performance-budgets.md) · [SR-ARCHITECTURE](../../records/0108_architecture.md) decision 29 · [SR-LINTER](../../records/0106_linter.md) decision 8 · [SR-CLI-MCP](../../records/0115_cli-and-mcp.md) decision 10 · [SR-PACKAGES](../../records/0107_packages.md) decision 11.

## Hazards

- **A loaded machine produces a clean, localised anomaly** (LESSONS). The 2026-10-22 numbers were taken with three other agents compiling (load average 10–18 on 10 cores); `check_cold_10k`'s p95 varied 3.4–4.4 s across sessions.
- **The shared cargo target** (`target-agents`) lets one worktree's build overwrite another's `codelaw` binary and crate artifacts. Measure a copy of the binary, never the target's own.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
- [`fixtures/perf/README.md`](../../fixtures/perf/README.md): the workload.
