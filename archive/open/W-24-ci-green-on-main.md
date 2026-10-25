---
type: Handoff
name: W-24
description: "Arpit, 2026-10-22: the repo review — CI cannot pass on main today, and a green verdict can be reused for a change that would fail."
item: W-24
filed: 2026-10-23
filed_by: "Arpit (Claude Code, Fable 5.1)"
updated: 2026-10-25
updated_by: "Claude Code (Opus 5.5): closed — CI run 40185344678 green on 76f1ad6"
ball: done
---

# W-24 — Make CI pass on `main`, and make a green verdict mean something

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Fix the Windows build: `bench/mod.rs`'s `not(unix)` `wait` returns `Ok(i32)` where `Ok((i32, Option<u64>))` is declared | the two `win32-*` builds compile; `package`, `smoke` and the attestation run again | ✅ 2026-10-24: fixed in `5e489f4` (`Ok((code, None))`); both `win32-*` builds green in CI runs 40026216188 and 40039203859; a unit test pins `wait`'s `(code, peak)` shape on every platform |
| 2 | Move `records/RULE-SINCE` forward to the commit before this item lands, with the reason written in the file: the nine failing commits are the WIP branches the decider had merged as-is (2026-10-21/22) and their two resolutions | the `lint` job's guard replay passes; nothing after the move escapes it | ✅ 2026-10-24: `c6b9f03` → `5e489f4`; ten commits failed, not nine (`d41c87f` and `434fd75` too) — all ten named in the file |
| 3 | Add every input a test reads to the verdict and wheel cache keys: `agents/**`, `scripts/**`, `packaging/**`, `.github/workflows/**`, `records/0111_consumer-layout.md` | a change to a skill, a script or a workflow runs the tests again instead of reusing an old verdict | ✅ 2026-10-24: `test` adds `agents/**`, `perf/**`, `records/[0-9]*.md` (schema.rs reads every record), `scripts/**`, `packaging/**`, `.github/workflows/**`; `clippy` adds the `include_str!` inputs `agents/**`, `perf/**`, `packs/**`; `wheel` adds `agents/**` (compiled in) and `scripts/package.py` |
| 4 | Run the law gate for real in `ci.yml` (`law-gate.py dist`, no `--report`) and drop the "until W-07–W-09 land" wording | `main` learns the day the binary loses `format`, `mcp` or `lsp`, not release day | ✅ 2026-10-24: no `--report`; CI run 40039203859 already reported the engine complete, so it passes today |
| 5 | Run the `perf` job only when `crates/**`, `Cargo.lock` or `perf/**` changed | a docs push stops building a full LTO release | ✅ 2026-10-24: a `changed` step diffs the push's `before` (a PR's base) against HEAD over `crates`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `perf`, `packs` (bench loads them) and `ci.yml`; unknown base, nightly and dispatch run it |
| 6 | `check-open-work-inbox.py`: `read_text(encoding="utf-8")` like every other script | the hook does not die on a non-UTF-8 locale (`work/MACHINE.md`) | ✅ 2026-10-24: and its error line is written as UTF-8 bytes, which also died under `LC_ALL=C` |

**Arpit, 2026-10-22:** *"review the repo, create work items to fix whatever was broken"*

| 7 | Found 2026-10-24 in run 40039203859: `smoke darwin-*` fail with "the staging index did not start" — `HTTPServer.server_bind` calls `socket.getfqdn("127.0.0.1")`, a reverse-DNS lookup that stalls on the macOS runners | the two macOS smokes pass | ✅ CI run 40124005285: `smoke darwin-arm64` and `darwin-x64` installed from the staging index and passed — the reverse-DNS diagnosis held |

| 8 | Found 2026-10-25: `main` red since run 40125039176 — `lint`'s docs-sync step: CHANGELOG *Unreleased* named neither W-18 nor W-19, which `work/IMPLEMENTATION.md` dates after 0.1.0 | `lint` green again | ✅ 2026-10-25: a line each under *Unreleased* in `76f1ad6`; CI run 40185344678 on it green, `lint` included |

**Model:** Sonnet: six concrete edits, each with a mechanical pass condition; step 2 needs care with the reason text.

## Goal

A push of `main` as it stands is green on all eight platforms, and a green verdict is only ever reused for a change that touched none of the tests' inputs.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · windows | `cargo check --target x86_64-pc-windows-msvc -p codelaw-cli` passes; the fix has a unit test of `wait`'s return shape or a `cfg(not(unix))` compile test | ✅ CI's `build win32-x64`/`win32-arm64` compile it (stronger than a check); `bench::tests::wait_returns_the_exit_code_and_peak_memory_only_where_measured` |
| 2 · replay | `for c in $(git rev-list --no-merges $(grep -v '^#' records/RULE-SINCE)..HEAD); do SR_GUARD_COMMIT=$c scripts/sr-guard.sh; done` exits 0 on `main`; `RULE-SINCE` names the reason and the date of the move | ✅ locally: 0 commits after `5e489f4` fail; CI replays the merged branches |
| 3 · keys | the `test`, `clippy` and `wheel` cache keys hash every path a test under `crates/*/tests` reads (`grep -rn "include_str!\|read_to_string\|\.\./\.\./" crates/*/tests crates/*/src | grep -v target` is the list) | ✅ |
| 4 · gate | `ci.yml`'s law-gate step fails on a binary missing `format`, `mcp` or `lsp` | ✅ (law-gate.py exits 1 without `--report`) |
| 5 · perf | a push touching only `*.md` does not run `perf` | ✅ CI run 40125039176 on `0473bcd` (work/, archive/, records/ only): "perf: skipped — nothing it times changed since ae909824" |
| 6 · hook | `LC_ALL=C python3 scripts/check-open-work-inbox.py` passes | ✅ (and its failure path, under `LC_ALL=C`) |
| 7 · green | a push of `main` with this change is green on all eight platforms | ✅ 2026-10-25: CI run 40185344678 on `76f1ad6` — all 22 jobs green (eight builds, `package`, eight smokes, `lint`, `test`, `msrv`, `corpus`, `perf`); before it, CI run 40124005285 on `483b2a1` (2026-10-24): every job green — all eight builds, packages and smokes, `lint` (guard replay), `test`, `msrv`, `corpus`, `perf` |

## Blockers

None.

## Record affected

[SR-WORK-OWNERSHIP](../../records/0058_WORK-ownership.md) decision 7 (the baseline move, step 2) · [SR-PACKAGES](../../records/0107_packages.md) decision 11 (the gate, step 4) · [SR-LAW-8](../../records/0010_LAW-8-performance-budgets.md) decision 7 (when the `perf` job runs, step 5) · [SR-CLI-MCP](../../records/0115_cli-and-mcp.md) decision 10 (`bench` on Windows, step 1) · [SR-PACKAGES](../../records/0107_packages.md) decision 11's staging index (step 7).

## Hazards

- **Changing `ci.yml` resets every cache key** (W-04 §Measured): land steps 3–5 as one change.
- **Step 2 is one-way:** commits before the new baseline are no longer re-auditable; the file's own comment says a move follows a bulk review — this review is it, and the nine shas are listed in the move's note.
- **Found by:** the 2026-10-22 review; the Windows defect landed in `e5122e1` (W-19) and was verified on a Mac only. A `cargo check` for a Windows target before merging a `cfg`-split file is a lesson for `work/LESSONS.md`.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
- `crates/codelaw-cli/src/bench/mod.rs` `wait()` under `#[cfg(not(unix))]`; `.github/workflows/ci.yml` jobs `lint`, `test`, `perf`, the law-gate step; `records/RULE-SINCE`.
