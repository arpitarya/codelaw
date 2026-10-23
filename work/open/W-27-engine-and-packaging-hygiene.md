---
type: Handoff
name: W-27
description: "Arpit, 2026-10-22: the repo review — small defects in the crates, the manifests and the ignore list."
item: W-27
filed: 2026-10-23
filed_by: "Arpit (Claude Code, Fable 5.1)"
updated: 2026-10-23
updated_by: "Arpit (Claude Code, Fable 5.1)"
ball: agent
---

# W-27 — Engine and packaging hygiene: a true MSRV, one process killer, Windows stated, manifests covered

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | `rust-version` to the compiler that first stabilised what the workspace uses (let-chains → 1.88), and a `cargo +<msrv> check` in CI | `Cargo.toml` stops promising 1.85 to a build that cannot pass on it | ⏳ |
| 2 | One `kill_group(child)` in `codelaw-core`; `codelaw-adapters` and `codelaw-fix` call it and drop their own `libc` dependency | one copy of the process-group kill, one place to fix it | ⏳ |
| 3 | State in SR-ARCHITECTURE that on Windows every command is one-shot (no `daemon`, no warm socket), and have `smoke.sh` assert `codelaw daemon status`'s message on `win32-*` | L6's "identical output" names its one documented hole and tests it | ⏳ |
| 4 | `scripts/version.py --set/--check` covers `packaging/vscode/package.json`, or SR-PACKAGES decision 6 says the extension versions on its own | no hand-edited manifest, as `CLAUDE.md` §Layout claims | ⏳ |
| 5 | One pack-artefact builder: `build-packs.py` writes its tarball through `package.py`, or CI `cmp`s the two | the tarball a contributor verifies is the tarball CI publishes | ⏳ |
| 6 | `.gitignore`: `/out/`, `*.vsix` | CI's working folders cannot be committed by accident | ⏳ |

**Arpit, 2026-10-22:** *"review the repo, create work items to fix whatever was broken or ... needs to be structured in a better way"*

**Model:** Sonnet: each step is a small, local change with a command that proves it; step 3 needs a Windows runner (CI's `smoke` job has one).

## Goal

The workspace's stated floors and manifests are true, duplicated code has one home, and the Windows behaviour is a decision rather than an accident.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · msrv | `cargo +<rust-version> check --workspace` passes in CI | ⏳ |
| 2 · kill | `grep -rn "libc::kill" crates/` has one hit, in `codelaw-core` | ⏳ |
| 3 · windows | SR-ARCHITECTURE names the Windows behaviour; the `smoke` job's `win32-*` legs assert it | ⏳ |
| 4 · version | `scripts/version.py --check` fails when `packaging/vscode/package.json`'s version drifts, or the record says it may | ⏳ |
| 5 · packs | `cmp dist/npm/codelaw-rules-js.tgz <build-packs.py's tarball>` is a CI step, or one builder remains | ⏳ |
| 6 · ignore | `git check-ignore out/x foo.vsix` names both | ⏳ |

## Blockers

None.

## Record affected

[SR-PACKAGES](../../records/0107_packages.md) decisions 6, 9, 13 (steps 1, 4, 5) · [SR-ARCHITECTURE](../../records/0108_architecture.md) decision 24 (step 3) · the register's Ownership table if `codelaw-core` gains a `process` module.

## Hazards

- **Step 1 may raise the floor consumers see in `cargo install`**; SR-PACKAGES decision 13 says the release toolchain is `rust-toolchain.toml`, so the MSRV is informational — say so in the same change.
- **Found by:** the 2026-10-22 review.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
- `crates/codelaw-adapters/src/run.rs` and `crates/codelaw-fix/src/external.rs` (the duplicate); `crates/codelaw-cli/src/warm.rs` (the Windows refusal); `scripts/version.py`; `scripts/build-packs.py` vs `scripts/package.py assemble`.
