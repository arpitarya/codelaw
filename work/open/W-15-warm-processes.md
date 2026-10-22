---
type: Handoff
name: W-15
description: "Arpit, 2026-10-20: what keeps codelaw fast between runs."
item: W-15
filed: 2026-10-20
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-20
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-15 — Build the warm processes — daemon, warm sockets, incremental re-parse, the compiled rule-set cache

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Cache the compiled rule set by the hash of every rule file, data table and the config | rules are compiled on every run today | ✅ 2026-10-22 |
| 2 | Register warm sockets (`.codelaw/cache/run/<pid>.sock`) from lsp, mcp and daemon; `--prefer-warm` | one-shot runs reuse a warm process | ✅ 2026-10-22 |
| 3 | Add `codelaw daemon start|stop` with idle timeout and reload | opt-in warm process | ✅ 2026-10-22 |
| 4 | Re-parse incrementally on edit; bound the tree cache | SR-ARCHITECTURE decisions 23–24 | 🟡 re-parse ✅ 2026-10-22; `[lsp] tree_cache` (the LRU bound) not read yet |

**Arpit, 2026-10-20:** *"Review the code. What are the blockers and what is pending?"* — then *"create work items for what all things are pending."*

**Model:** Opus: process lifecycle, version equality and stale-socket cleanup fail in ways tests only sample.

## Goal

A warm process serves hook and `--prefer-warm` runs with identical output to a cold run.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · rule-set cache | Compiled rules cached (SR-ARCHITECTURE decision 15, 15c). | ✅ |
| 2 · sockets | Discovery picks the newest live socket of the same version; stale ones are removed (fixture). | ✅ `tests/warm.rs` |
| 3 · daemon | Opt-in, 30-minute idle exit, reloads config/rules/baseline. | ✅ `tests/warm.rs` (baseline: none exists yet, W-13) |
| 4 · parity | Warm and cold runs give byte-identical output on every fixture. | ✅ `tests/warm.rs` |

## Blockers

None. **2026-10-22:** merged into `main` and verified (the merge found `daemon status` hanging on a socket that never answers; fixed). Left: the `[lsp] tree_cache` bound of decision 24.

## Record affected

[SR-ARCHITECTURE](../../records/0108_architecture.md) decisions 15, 21–24 · [SR-CACHE](../../records/0109_cache.md).

## Hazards

- **Windows** has no Unix sockets by default: named pipes, or the feature is Unix-only and says so.

## Reference

- [`work/OPEN-WORK.md`](../OPEN-WORK.md): this item's row.
