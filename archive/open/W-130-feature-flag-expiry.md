---
type: Handoff
name: W-130
description: "Arpit asked 2026-10-29 (Cowork, voice) for feature-flag expiry that works for every flag SDK and for teams' own wrappers over them."
item: W-130
filed: 2026-10-30
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-30
updated_by: "Claude Code (Opus 5.5)"
ball: agent
---

# W-130 — Feature-flag expiry, for any SDK and any wrapper

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Read flag accessors from config, with presets for six SDKs | every flag read is found | ✅ \|
| 2 | Find wrappers through the project graph | team helpers work with no config | ✅ \|
| 3 | Read expiries from `.codelaw/flags.toml` and inline markers | one dated owner per flag | ✅ \|
| 4 | Report at each use: unregistered, soon, expired; unused registry entries | dead flags surface on time | ✅ \|
| 5 | Fix: `keep = "on"\|"off"` collapses the branch | the cleanup is one edit | ✅ \|

**Arpit, 2026-10-29:** *"We need to build it out for all of them … in a bit more generic way, because … people might be using wrapper over Unleash, wrapper over LaunchDarkly or Split.io."* · on registry or inline: *"Both."*

**Model:** Opus (Claude Code): wrapper detection is data-flow over the project graph, across JS/TS and Python.

## Goal

[The proposal](../../work/proposals/deps-flags-markers-reviews.md) §3.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · accessors | `$NAME` patterns (positional, `$$$`, object key); presets `launchdarkly`, `unleash`, `split`, `openfeature`, `growthbook`, `flagsmith` | ✅ (2026-10-30) — `[flags] accessors` (`$NAME`, `$_`, `$$$`, `{ key: $NAME }`, `key=$NAME`, `$_.` receiver); presets in `crates/codelaw-graph/src/flags/patterns.rs`, JS/TS and Python |
| 2 · wrappers | parameter-to-`$NAME` flow, transitive, deterministic; `codelaw flags accessors` | ✅ (2026-10-30) — parameter flow to a fixpoint through the graph's imports; methods only where their file is imported; `codelaw flags accessors [--format json]` |
| 3 · names | literals; constants/enums via the graph; dynamic → info, uncertain | ✅ (2026-10-30) — literals; top-level constants, TS enums, Python `Enum` attributes, object keys, imported; dynamic → `info`, uncertain |
| 4 · expiry | registry + inline, registry wins, mismatch auto fix, `inline = "registry-only"`, `warn_days`, `never` needs `reason`, as-of date | ✅ (2026-10-30) — registry wins, marker rewritten (auto), conflicting markers `error`, `registry-only`, `warn_days`, `never` needs `reason`, as-of date only |
| 5 · fixes | `keep` collapse (suggest); unused entry removal (auto); manual + skill otherwise | ✅ (2026-10-30) — `keep-branch` (suggest, refused unless the read is the whole condition with pure arguments); `remove-entry` and marker removal (auto); manual steps via `codelaw-fix` |
| 6 · records, tests, docs | rule placement (below), SR-CONFIG `[flags]`, SR-PROJECT-GRAPH; fixtures per preset and a two-level wrapper; L8; L10 template if `init` writes a starter `flags.toml` | ✅ (2026-10-30) — SR-PROJECT-GRAPH 11, SR-RULE-PACKS 17, SR-CONFIG `[flags]`, SR-CLI-MCP, SR-ARCHITECTURE, SR-RULE-AUTHORING; `fixtures/flags` (six presets × JS/Python, two-level wrappers), `tests/flags.rs`; L8 measured; `init` writes no `flags.toml`, so no L10 template |

## Blockers

None.

## Record affected

SR-CONFIG `[flags]` (new) · SR-PROJECT-GRAPH (accessor flow) · SR-RULE-PACKS (the rules) · SR-CLI-MCP (`flags accessors`).

## Hazards

- **Placement is the agent's call, not ruled:** use-site rules in the existing `js` and `python` packs (`js/flag-expiry`, `py/flag-expiry`), registry rules in `data` (`data/flag-registry`) — no new pack, no L6 change. Say so to the decider when it lands.
- Never query a flag service (L5).
- `keep` collapse must keep side effects in the condition; refuse the edit when the condition is not a bare accessor call.

## Reference

[Proposal](../../work/proposals/deps-flags-markers-reviews.md) §3
