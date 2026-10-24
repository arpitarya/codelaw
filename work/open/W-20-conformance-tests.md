---
type: Handoff
name: W-20
description: "Arpit, 2026-10-20: the checks the laws and records name that nothing runs."
item: W-20
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-24
updated_by: "Claude (Opus 5.5), cloud session 2026-10-24: corpus snapshots, W-28 re-run"
ball: agent
---

# W-20 — Add the missing law and record conformance tests

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | L5 network fence: the engine runs with networking denied | L5 checked | ✅ |
| 2 | L2 licence check over the npm and PyPI build tools | L2 checked beyond Cargo | ✅ (npm's Artistic-2.0 ruled permissive, 2026-10-22) |
| 3 | `init` writes only the paths SR-CONSUMER-LAYOUT lists | the consumer tree checked | ✅ |
| 4 | Rule promotion: stable only after all four test levels, incl. a corpus run | SR-RULE-AUTHORING checked | ⏳ the runner and the gate are done; the pinned corpus and its CI job are not |
| 5 | Work-system tests: queue rows, owns ↔ ownership, the registry, lost `W-nn`s, a second `archive/` | the work system checked | ✅ (today's queue fails it until its rows are fixed) |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Model:** Sonnet: each test has a precise pass condition stated in its record.

## Goal

Each check below runs in `cargo test` or `ci.yml` and fails on a violation.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · L5 | A test runs `check` and `fix` with networking denied and expects identical output (SR-LAW-5). | ✅ `tests/network_fence.rs`; required in CI |
| 2 · L2 | The licence check covers the npm/PyPI build tools as well as Cargo. | ✅ `tests/licences.rs`; npm's Artistic-2.0 allowed by the decider, 2026-10-22 |
| 3 · init | An `init` fixture compares `git status` with SR-CONSUMER-LAYOUT §1. | ✅ `tests/consumer_layout.rs` |
| 4 · promotion | `codelaw test --corpus` and a CI job that promotes a rule to `stable` only after levels 1–4. | ✅ 2026-10-24 here, ⏳ CI: `packs/*/corpus.toml` for all nine packs over 11 pinned repos (`ionic-framework` added for `stencil`; `ts`, `react`, `node` run on zustand, jotai and the fastapi template), written by `corpus.py run --update` and re-run clean, so level 4 is deterministic. Built with rustc 1.97 (the pinned 1.99.1 was not reachable from the cloud); CI's first `corpus` run on `main` confirms. The build-and-test leg is dropped (Arpit, 2026-10-24, docket) |
| 5 · work system | Tests for SR-WORK-OPEN-QUEUE rows (13–19, 42, 45), SR-WORK-OWNERSHIP decision 2, SR-WORK-REGISTRY decisions 5–8, lost `W-nn`s (the record's own check command misreports: `ls a b` fails when either is missing), one `archive/`. | ✅ `scripts/check-work-system.py`, in CI |

## Blockers

None. npm's `Artistic-2.0` was ruled permissive by the decider, 2026-10-22: SR-LAW-2 decision 2 names it, `licences.rs` allows it and `AWAITING_RULING` is empty.

## What landed (2026-10-22, branch `W-20-conformance`)

- **L2:** `every_build_tool_is_declared_and_free_and_permissive` failed on two scanner faults, not a missing declaration. `scripts/smoke.sh`'s error message (`echo "… pip install for the host" >&2`) read as an install, and `packaging/vscode/package.json` was demanded in `BUILD_TOOLS` though its lockfile already carries each licence. Comments, `echo`/`printf` and redirections now end a command's scan. A `package.json` with a lockfile beside it is read through the lockfile. The lockfile test covers every `package-lock.json` under the build's folders, and fails a dependency the lockfile doesn't resolve.
- **L5:** the macOS fence is proven: the sandbox must refuse a connection to a port the test itself listens on. `ci.yml` sets `CODELAW_REQUIRE_NETWORK_FENCE=1`. SR-LAW-5 owns the test. It fences `check` and `fix`, not the whole suite that SR-LAW-5's enforcement list names.
- **init:** `consumer_layout.rs` reads §1's tree from the record itself. It runs `init --no-install` and `check` in a committed repo and compares the status, ignored paths included. `init --agents` must write only §1's paths, or nothing while it is refused (W-10). It found `.codelaw/rules/README.md` missing from §1. §1 now lists it.
- **promotion:** `codelaw test --corpus <dir> [--snapshot f [--update-snapshot]]` runs levels 1–3 and then level 4. At level 4 every file must give the same result on two runs, and each fix must still parse, be a fixpoint and keep the rule's invariant. Finding counts are compared with a `[findings]` TOML snapshot. `promotion.rs` refuses a `stable` pack rule that has no count in `packs/<pack>/corpus.toml`. No pack rule is `stable` today.
- **work system:** `scripts/check-work-system.py` (it has a `--self-test`) runs in `ci.yml`'s `lint` job. The registry check found `work/WORKLOG.md` and `work/NOW.md` without rows. Both have rows now.

## Left

- **Step 4's remainder:** a pinned corpus per pack (repositories and commits). The pack READMEs name the corpora they were run on by hand. Also left: a CI job that checks the corpus out and runs `codelaw test packs/<pack> --corpus … --snapshot packs/<pack>/corpus.toml`, and — dropped by the decider, 2026-10-24 — the "repo still builds and passes its own tests after fixes" leg: CI installs no corpus repo's dependencies; level 4 stands in for it.
- **The queue fails the work-system check today:** two inbox rows lack 🔴 (rule 44), and seven rows exceed 280 rendered characters (rule 15). Whoever edits `work/OPEN-WORK.md` fixes them.

## Record affected

[SR-LAW-2](../../records/0004_LAW-2-foss-dependencies.md) · [SR-LAW-5](../../records/0007_LAW-5-no-model-in-the-engine.md) · [SR-CONSUMER-LAYOUT](../../records/0111_consumer-layout.md) · [SR-RULE-AUTHORING](../../records/0113_rule-authoring.md) · [SR-CLI-MCP](../../records/0115_cli-and-mcp.md) · [SR-PACKAGES](../../records/0107_packages.md) · the WORK records 0051, 0057, 0058, 0060.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
