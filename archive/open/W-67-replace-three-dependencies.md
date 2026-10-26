---
type: Handoff
name: W-67
description: "Arpit, 2026-10-26 (W-66, by voice): remove strsim, anyhow and postcard — replace each with codelaw's own code, then drop them from Cargo.toml and DEPENDENCIES.md."
item: W-67
filed: 2026-10-26
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-26
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-67 — replace `strsim`, `anyhow` and `postcard` with codelaw's own code (L11)

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | **`strsim` (D-23):** write `levenshtein` and `jaro_winkler` in `codelaw-core` (~60 lines, with tests against known pairs); switch `codelaw-config/src/lib.rs:54`, `codelaw-match/src/schema.rs:340–341`, `codelaw-match/src/target.rs:405` | "did you mean …?" with no crate | ⏳ |
| 2 | **`anyhow` (D-24):** declared in `crates/codelaw-format/Cargo.toml` and used nowhere in `src/` — delete the line; if a build proves a use, replace it with a plain `Result<_, String>` or a small error enum | one fewer crate, likely for free | ⏳ |
| 3 | **`postcard` (D-20):** a codelaw cache format in `codelaw-cache` — length-prefixed little-endian fields for the cached types, behind the existing `HEADER`; bump the header's version so old cache files are ignored and rebuilt | the cache with no crate; a bad file is a miss, never a crash | ⏳ |
| 4 | Drop the three from the workspace `Cargo.toml` and the crates; `Cargo.lock` shrinks; delete rows D-20, D-23, D-24 from `DEPENDENCIES.md` | L11's list matches the tree | ⏳ |
| 5 | L8: `check_warm_10k` and the cache read/write measured before and after | no slower | ⏳ |

**Arpit, 2026-10-26 (W-66, by voice):** strsim — *"Remove"*; anyhow — *"Remove"*; postcard — *"Replace."*

**Model:** Sonnet — three small replacements with exact before/after behaviour to test.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | every "did you mean" message the test suite checks is unchanged | ⏳ |
| 2 · | `cargo tree -i strsim`, `-i anyhow`, `-i postcard` find nothing | ⏳ |
| 3 · | a cache written by the old format is ignored (rebuilt), never misread; a truncated or corrupted cache file is a miss | ⏳ |
| 4 · | L8 budgets hold; `cargo test --workspace` green | ⏳ |

## Blockers

None.

## Record affected

SR-LAW-11 (`DEPENDENCIES.md` rows removed) · SR-CACHE (the on-disk format) · SR-CONFIG decision 5 (suggestion behaviour unchanged).

## Hazards

- Transitive users: another dependency may also pull `strsim` (clap can). Removing our direct use is the ruling; if `cargo tree -i strsim` still shows it under a ratified unit, it is part of that unit's ratified tree — say so in the DoD note rather than fight it.
- The cache format must stay deterministic (L5): fixed field order, no hash-map iteration.

## Reference

[`DEPENDENCIES.md`](../../DEPENDENCIES.md) · [SR-LAW-11](../../records/0013_LAW-11-no-dependencies.md) · [SR-CACHE](../../records/0109_cache.md)
