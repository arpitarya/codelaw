---
type: Handoff
name: W-129
description: "Arpit picked 2026-10-29 (Cowork, voice) four more dependency rules for the deps pack: licence, lockfile in sync, monorepo version drift, runtime version agreement."
item: W-129
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-129 — Licence, lockfile sync, version drift and runtime agreement

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Build `deps/licence` from the snapshot's licence metadata | copyleft or unknown licences caught | ⏳ |
| 2 | Build `deps/lockfile-sync` | a manifest edited without its lockfile caught | ⏳ |
| 3 | Build `deps/version-drift` | one version of a package across a monorepo | ⏳ |
| 4 | Build `deps/runtime-agreement` | Node and Python versions agree everywhere they are named | ⏳ |
| 5 | Report each text-file finding in both places, once W-136 amends L3 | the `.nvmrc` line and `package.json` both show it | ⏳ unblocked — W-136 built `NativeRule::elsewhere` 2026-10-30 |

**Arpit, 2026-10-29:** *"The license check, I like it. The lock file sync, I like it. Version drift in monorepo, I like that one as well. The runtime version agreement, I like that as well."*

**Model:** Sonnet (Claude Code): four contained rules on parsers that exist; SPDX expression parsing is the one new piece.

## Goal

[The proposal](../proposals/deps-flags-markers-reviews.md) §2.5, in the `deps` pack W-128 creates.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · licence | `[deps.licence] allow / deny / unknown`, defaults as the proposal; SPDX `OR`/`AND`/`WITH`; direct and transitive | ⏳ |
| 2 · lockfile-sync | package-lock, pnpm, yarn, uv, poetry (`content-hash` recomputed); manual fix names the lock command | ⏳ |
| 3 · version-drift | npm/pnpm/yarn workspaces, uv members; `policy = "highest"\|"root"`; suggest fix | ⏳ |
| 4 · runtime-agreement | every source in the proposal; `[deps.runtime] source`; suggest on L3 files | ⏳ |
| 5 · records, tests, docs | SR-RULE-PACKS, SR-CONFIG; fixtures per lockfile kind; corpus; L8; rule docs | ⏳ |

## Blockers

None — W-128 (the `deps` pack, licences in the snapshot) and W-136 (`elsewhere`) were built and closed 2026-10-30 (review 2026-10-30, Cowork).

## Record affected

SR-RULE-PACKS decision 15 (as W-128 leaves it) · SR-CONFIG §`[deps]`.

## Hazards

- `.nvmrc`, `.python-version`, `.tool-versions`, Dockerfile, yarn classic `yarn.lock`: Q-A ruled B (2026-10-30, *"in both the places"*) — the finding goes on the L3 file **and** the text file's line, through W-136.
- A licence field is free text on old packages — `unknown`, never a guess.
- No new dependency (L11): write the SPDX parser.

## Reference

[Proposal](../proposals/deps-flags-markers-reviews.md) §2.5, §8 · [W-128](W-128-deps-pack-and-vulnerabilities.md)
