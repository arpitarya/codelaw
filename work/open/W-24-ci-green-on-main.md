---
type: Handoff
name: W-24
description: "Arpit, 2026-10-22: the repo review — CI cannot pass on main today, and a green verdict can be reused for a change that would fail."
item: W-24
filed: 2026-10-23
filed_by: "Arpit (Claude Code, Fable 5.1)"
updated: 2026-10-23
updated_by: "Arpit (Claude Code, Fable 5.1)"
ball: agent
---

# W-24 — Make CI pass on `main`, and make a green verdict mean something

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Fix the Windows build: `bench/mod.rs`'s `not(unix)` `wait` returns `Ok(i32)` where `Ok((i32, Option<u64>))` is declared | the two `win32-*` builds compile; `package`, `smoke` and the attestation run again | ⏳ |
| 2 | Move `records/RULE-SINCE` forward to the commit before this item lands, with the reason written in the file: the nine failing commits are the WIP branches the decider had merged as-is (2026-10-21/22) and their two resolutions | the `lint` job's guard replay passes; nothing after the move escapes it | ⏳ |
| 3 | Add every input a test reads to the verdict and wheel cache keys: `agents/**`, `scripts/**`, `packaging/**`, `.github/workflows/**`, `records/0111_consumer-layout.md` | a change to a skill, a script or a workflow runs the tests again instead of reusing an old verdict | ⏳ |
| 4 | Run the law gate for real in `ci.yml` (`law-gate.py dist`, no `--report`) and drop the "until W-07–W-09 land" wording | `main` learns the day the binary loses `format`, `mcp` or `lsp`, not release day | ⏳ |
| 5 | Run the `perf` job only when `crates/**`, `Cargo.lock` or `perf/**` changed | a docs push stops building a full LTO release | ⏳ |
| 6 | `check-open-work-inbox.py`: `read_text(encoding="utf-8")` like every other script | the hook does not die on a non-UTF-8 locale (`work/MACHINE.md`) | ⏳ |

**Arpit, 2026-10-22:** *"review the repo, create work items to fix whatever was broken"*

**Model:** Sonnet: six concrete edits, each with a mechanical pass condition; step 2 needs care with the reason text.

## Goal

A push of `main` as it stands is green on all eight platforms, and a green verdict is only ever reused for a change that touched none of the tests' inputs.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · windows | `cargo check --target x86_64-pc-windows-msvc -p codelaw-cli` passes; the fix has a unit test of `wait`'s return shape or a `cfg(not(unix))` compile test | ⏳ |
| 2 · replay | `for c in $(git rev-list --no-merges $(grep -v '^#' records/RULE-SINCE)..HEAD); do SR_GUARD_COMMIT=$c scripts/sr-guard.sh; done` exits 0 on `main`; `RULE-SINCE` names the reason and the date of the move | ⏳ |
| 3 · keys | the `test`, `clippy` and `wheel` cache keys hash every path a test under `crates/*/tests` reads (`grep -rn "include_str!\|read_to_string\|\.\./\.\./" crates/*/tests crates/*/src | grep -v target` is the list) | ⏳ |
| 4 · gate | `ci.yml`'s law-gate step fails on a binary missing `format`, `mcp` or `lsp` | ⏳ |
| 5 · perf | a push touching only `*.md` does not run `perf` | ⏳ |
| 6 · hook | `LC_ALL=C python3 scripts/check-open-work-inbox.py` passes | ⏳ |

## Blockers

None.

## Record affected

[SR-WORK-OWNERSHIP](../../records/0058_WORK-ownership.md) decision 7 (the baseline move, step 2) · [SR-PACKAGES](../../records/0107_packages.md) decision 11 (the gate, step 4) · [SR-LAW-8](../../records/0010_LAW-8-performance-budgets.md) decision 7 (when the `perf` job runs, step 5) · [SR-CLI-MCP](../../records/0115_cli-and-mcp.md) decision 10 (`bench` on Windows, step 1).

## Hazards

- **Changing `ci.yml` resets every cache key** (W-04 §Measured): land steps 3–5 as one change.
- **Step 2 is one-way:** commits before the new baseline are no longer re-auditable; the file's own comment says a move follows a bulk review — this review is it, and the nine shas are listed in the move's note.
- **Found by:** the 2026-10-22 review; the Windows defect landed in `e5122e1` (W-19) and was verified on a Mac only. A `cargo check` for a Windows target before merging a `cfg`-split file is a lesson for `work/LESSONS.md`.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
- `crates/codelaw-cli/src/bench/mod.rs` `wait()` under `#[cfg(not(unix))]`; `.github/workflows/ci.yml` jobs `lint`, `test`, `perf`, the law-gate step; `records/RULE-SINCE`.
