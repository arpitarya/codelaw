---
type: Handoff
name: W-65
description: "Arpit, 2026-10-26: L11's mechanical check — CI fails on any dependency not in DEPENDENCIES.md, or on a ratified unit whose transitive tree grew."
item: W-65
filed: 2026-10-26
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-26
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-65 — L11's check: nothing outside `DEPENDENCIES.md`

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Give `DEPENDENCIES.md` a machine-readable block per unit (members, ecosystem, scope, transitive package list or its hash at ratification) | the check has something exact to compare | ⏳ |
| 2 | `crates/codelaw-cli/tests/dependencies.rs` (beside `licences.rs`): read `Cargo.lock` + every `Cargo.toml`, `packaging/**/package.json` + `package-lock.json`, `packaging/python/pyproject.toml` build requirements, `.github/workflows/*.yml` `uses:` | every dependency found | ⏳ |
| 3 | Fail when a dependency belongs to no unit; when a `ratified` unit's transitive set changed; when a `pending` unit's pin moved to a version adding packages | L11 decisions 1, 4, 5 enforced | ⏳ |
| 4 | Fold the scripts check: `scripts/*.py`, `.claude/hooks/*.py` import only the standard library | the dependency-free parts stay so | ⏳ |
| 5 | Run it in `ci.yml`; SR-LAW-11's `owns:` gains `DEPENDENCIES.md` and the test, the Ownership table both rows, the register `built: yes` | the law is gated | ⏳ |

**Arpit, 2026-10-26:** *"Create a law, no dependencies … We will ratify them one by one."*

**Model:** Sonnet — a parser over known manifest formats with a written pass/fail rule.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | adding an unlisted crate, npm package or action fails CI with its name and where it came from | ⏳ |
| 2 · | bumping a ratified crate so its tree gains a package fails, naming the new package | ⏳ |
| 3 · | today's tree passes: 29 units ratified 2026-10-26 (their trees as pinned that day), 3 marked `remove` until W-67 lands | ⏳ |
| 4 · | the check itself uses no new dependency (L11) | ⏳ |

## Blockers

None.

## Record affected

SR-LAW-11 decision 7 (built) · SR-LAW-2's `licences.rs` sits beside it.

## Hazards

- `Cargo.lock` holds platform-specific packages (Windows-only crates): count the whole lockfile, not one target.

## Reference

[SR-LAW-11](../../records/0013_LAW-11-no-dependencies.md) · [`DEPENDENCIES.md`](../../DEPENDENCIES.md)
