# `archive/` — the one archive, and where each thing went

**There is exactly ONE archive directory, and it is this one, at the repo
root.** Anything archived is moved here, into a directory mirroring where it
came from. This README is the **map**: every archived doc gets a row naming its
**live successor**.

## Archive is not evidence

A doc in here may be *named*. It may never be *cited as backing a live claim*
— nothing guarantees an archived file was not overwritten after it was
retired. Entries are frozen; **relative links inside them are not repaired**.
Rules: [SR-WORK-ARCHIVE](../records/0057_WORK-archive.md).

## Layout — the archive mirrors the live tree

| directory | holds |
|---|---|
| `open/` | closed work items' detail files, byte for byte |
| `records/` | retired records (L1) |
| `compare/` | decided forks whose triggers were ported into a record |
| `proposals/` | implemented or rejected proposals |
| `worklog/` | worklog slices cut at each major release |
| `bootstrap/` | the WORK-SYSTEM.md this repo's work system was set up from |

## `open/`

| file | closed | where the live answer lives now |
|---|---|---|
| [`W-01-rule-pack-records.md`](open/W-01-rule-pack-records.md) | 2026-10-19 | [SR-RULE-PACKS](../records/0101_rule-packs.md) and [SR-LAW-6](../records/0008_LAW-6-python-and-node-packages.md), the records it rewrote |
| [`W-02-engine-p0.md`](open/W-02-engine-p0.md) | 2026-10-19 | the engine in `crates/`, owned per the register's Ownership table; [SR-ARCHITECTURE](../records/0108_architecture.md) is its design of record; [`work/IMPLEMENTATION.md`](../work/IMPLEMENTATION.md) has what landed |
| [`W-08-mcp-server.md`](open/W-08-mcp-server.md) | 2026-10-20 | `crates/codelaw-mcp`; [SR-CLI-MCP](../records/0115_cli-and-mcp.md) decisions 5–6 |
| [`W-17-engine-defects.md`](open/W-17-engine-defects.md) | 2026-10-20 | the fixes and their regression tests in `crates/`; deviations in [SR-ARCHITECTURE](../records/0108_architecture.md) 15a and [SR-RULE-PACKS](../records/0101_rule-packs.md) decision 7 |
| [`W-22-record-drift.md`](open/W-22-record-drift.md) | 2026-10-20 | [SR-RULE-PACKS](../records/0101_rule-packs.md), [SR-PACKAGES](../records/0107_packages.md) decision 9, [SR-ARCHITECTURE](../records/0108_architecture.md) decision 1 |

## `records/`

| file | retired | ruling | successor |
|---|---|---|---|

## `compare/`

| file | decided | the trigger lives in |
|---|---|---|

## `proposals/`

| file | outcome | successor |
|---|---|---|

## `bootstrap/`

| file | archived | successor |
|---|---|---|
| `WORK-SYSTEM.md` | 2026-10-18 | the records `0001`–`0003` and `0051`–`0060`, which are now the only home of every rule it carried |
