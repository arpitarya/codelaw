---
type: Handoff
name: W-27
description: "Arpit, 2026-10-22: the repo review — small defects in the crates, the manifests and the ignore list."
item: W-27
filed: 2026-10-23
filed_by: "Arpit (Claude Code, Fable 5.1)"
updated: 2026-10-24
updated_by: "Arpit (Claude Code, Opus 5.5)"
ball: agent
---

# W-27 — Engine and packaging hygiene: a true MSRV, one process killer, Windows stated, manifests covered

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | `rust-version` to the compiler the workspace actually needs — **1.97**, not 1.88: the pinned `ruff_*` 0.0.18 crates declare it, the highest `rust-version` in `Cargo.lock` — and a `cargo +<msrv> check` in CI (`ci.yml` job `msrv`) | `Cargo.toml` stops promising 1.85 to a build that cannot pass on it | ✅ local · ⏳ first CI run |
| 2 | One `kill_group(child)` in `codelaw-core` (`process` module); `codelaw-adapters` and `codelaw-fix` call it and drop their own `libc` dependency | one copy of the process-group kill, one place to fix it | ✅ |
| 3 | State in SR-ARCHITECTURE (decision 21, not 24) that on Windows every command is one-shot (no `daemon`, no warm socket), and have `smoke.sh` assert `codelaw daemon status`'s message and exit 2 on `win32-*`, plus `check --prefer-warm` byte-equal to `check` | L6's "identical output" names its one documented hole and tests it | ✅ written · ⏳ first CI run on the `win32-*` legs |
| 4 | `scripts/version.py --set/--check` covers `packaging/vscode/package.json` and its `package-lock.json` (SR-PACKAGES decision 7); the extension now says 0.1.0, not 0.0.0 | no hand-edited manifest, as `CLAUDE.md` §Layout claims | ✅ |
| 5 | One pack-artefact builder: `package.py`'s `stage_pack` + `pack_tarball` + `wheel_for_pack`; `assemble` no longer runs `npm pack` for packs, and `build-packs.py` calls the same three | the tarball a contributor verifies is the tarball CI publishes | ✅ |
| 6 | `.gitignore`: `/out/`, `*.vsix` | CI's working folders cannot be committed by accident | ✅ |

**Arpit, 2026-10-22:** *"review the repo, create work items to fix whatever was broken or ... needs to be structured in a better way"*

**Model:** Sonnet: each step is a small, local change with a command that proves it; step 3 needs a Windows runner (CI's `smoke` job has one).

## Goal

The workspace's stated floors and manifests are true, duplicated code has one home, and the Windows behaviour is a decision rather than an accident.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · msrv | `cargo +<rust-version> check --workspace` passes in CI | ⏳ passes locally on 1.97 (`--all-targets --locked`); CI's `msrv` job not yet run |
| 2 · kill | `grep -rn "libc::kill" crates/` has one hit, in `codelaw-core` | ✅ |
| 3 · windows | SR-ARCHITECTURE names the Windows behaviour; the `smoke` job's `win32-*` legs assert it | ⏳ record ✅; the assertion is in `smoke.sh`, unproven until the `win32-*` legs run |
| 4 · version | `scripts/version.py --check` fails when `packaging/vscode/package.json`'s version drifts, or the record says it may | ✅ (failed on 0.0.0, passes after `--set 0.1.0`) |
| 5 · packs | `cmp dist/npm/codelaw-rules-js.tgz <build-packs.py's tarball>` is a CI step, or one builder remains | ✅ one builder; all 14 artefacts `cmp`-equal between `assemble` and `build-packs.py` locally |
| 6 · ignore | `git check-ignore out/x foo.vsix` names both | ✅ |

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

## Progress

- **2026-10-24 (Claude Code, Opus 5.5):** steps 1–6 landed on the W-27 branch.
  - **MSRV 1.97, not 1.88:** `cargo metadata` puts the highest dependency `rust-version` at 1.97 (`ruff_*` 0.0.18, which `codelaw-format` pins); `cargo` refuses any older compiler, and `cargo +1.97 check --workspace --all-targets --locked` passes. `README.md`'s "1.85 or newer" follows. SR-PACKAGES decision 13 says it is informational.
  - **One pack builder:** the npm pack tarballs are now written by `package.py pack_tarball`, not `npm pack`; `npm publish --dry-run` accepts one. With stand-in binaries, `assemble` then `check` pass every check but the (absent) engine wheels, and each of the 14 pack artefacts is byte-identical to `build-packs.py --release`'s.
  - **Left:** DoD 1 and 3 are proven only by the first CI run that carries them (`msrv` job; `smoke` on `win32-x64` and `win32-arm64`).

