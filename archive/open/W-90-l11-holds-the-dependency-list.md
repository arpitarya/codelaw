---
type: Handoff
name: W-90
description: "Arpit, 2026-10-27 (voice docket): rename L11 from 'no dependencies' to 'dependencies', move everything in DEPENDENCIES.md into the law, archive DEPENDENCIES.md."
item: W-90
filed: 2026-10-27
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-27
updated_by: "Claude Code (Opus 5.5), W-90 closed"
ball: done
---

# W-90 — L11 is "dependencies", and it holds the list

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Rename L11 "no dependencies" → **"dependencies"**: `records/0013_LAW-11-dependencies.md` → `records/0013_LAW-11-dependencies.md`; its decisions keep their numbers | the law's name says what it holds | ✅ 2026-10-27 |
| 2 | Move every table of `DEPENDENCIES.md` (D-01…D-36, statuses and the decider's words verbatim) into the law as its list | one home for the rule and the list | ✅ 2026-10-27 |
| 3 | `DEPENDENCIES.md` → `archive/DEPENDENCIES.md`, byte for byte plus a banner naming L11 as its successor; `archive/README.md` row (L1: archived, never deleted) | the old file is history, not a second source | ✅ 2026-10-27 |
| 4 | Repoint every reference in the same change: `CLAUDE.md` and `AGENTS.md` law lines, SR-LAWS, `records/README.md` register, SR-WORK-REGISTRY, SR-PACKAGES, SR-ARCHITECTURE, `crates/codelaw-cli/tests/dependencies.rs` and `licences.rs` (they read the file), `scripts/package.py`, `Cargo.toml` comments, the tree-sitter-scss README, `CHANGELOG.md` *Unreleased* | nothing points at a file that moved | ✅ 2026-10-27 |
| 5 | `cargo test -p codelaw-cli --test dependencies --test licences`, `check-work-system.py`, `docs-sync.py --check` green; CI's `lint` | the move broke nothing | ✅ 2026-10-27 |

**Arpit, 2026-10-27 (voice docket):** *"Remove dependencies.md and everything that goes into dependencies.md should be moved into the law that was created. Rather than having that law named no dependency, let's call that law as dependency."* Then, archive or delete: *"Move it to archive."*

## Definition of done

- ✅ L11's file and handle say "dependencies"; the law holds the list (§3: the tables D-01…D-36 and the ledger); `DEPENDENCIES.md` exists only in `archive/`, with its banner and `archive/README.md` row.
- ✅ `grep -r DEPENDENCIES.md` outside `archive/`, `work/WORKLOG.md` and `.claude/worktrees/` finds only the archive's path (the generated `.fux/index` is rebuilt by its own hook).
- ✅ The tests that read the list read it from the law: `dependencies.rs` reads `records/0013_LAW-11-dependencies.md` and takes rows only from tables headed `| id |`.

**Closed 2026-10-27 (Claude Code, W-90).** Hazard 1 answered in the law: decision 4 now names D-34 as ratified unversioned. `licences.rs` names D-34 in the law and as ratified (it still said `pending`).

## Record affected

[SR-LAW-11](../../records/0013_LAW-11-dependencies.md) (renamed, gains the list) · [SR-LAWS](../../records/0001_LAWS.md) · [SR-WORK-REGISTRY](../../records/0060_WORK-registry.md) · `records/README.md` · the agent-steering files (SR-WORK-DOCS decisions 6–8 govern that edit).

## Hazards

- L11 decision 4 accepts a unit's transitive tree "as pinned that day"; D-34 (fux-engine) is now ratified with no fixed version (W-91) — the law's text must say so, or the two disagree.
- A law record outranks every other; the rename is a law change on the decider's ruling, named in the record.
- Concurrent sessions read `DEPENDENCIES.md`; land in one change.

**Model:** Sonnet.
