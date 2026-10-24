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
| [`W-06-consumer-readme.md`](open/W-06-consumer-readme.md) | 2026-10-22 | `README.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md` and `.github/ISSUE_TEMPLATE/`, `.github/pull_request_template.md`; [SR-WORK-REGISTRY](../records/0060_WORK-registry.md) §3 |
| [`W-07-formatter.md`](open/W-07-formatter.md) | 2026-10-21 | `crates/codelaw-format`; [SR-FORMATTER](../records/0105_formatter.md) §What is built and its parity table; [SR-LAW-7](../records/0009_LAW-7-linter-and-formatter.md) §How L7 is enforced; the rest is W-23 |
| [`W-08-mcp-server.md`](open/W-08-mcp-server.md) | 2026-10-20 | `crates/codelaw-mcp`; [SR-CLI-MCP](../records/0115_cli-and-mcp.md) decisions 5–6 |
| [`W-09-lsp-and-extension.md`](open/W-09-lsp-and-extension.md) | 2026-10-21 | `crates/codelaw-lsp`, `packaging/vscode`; [SR-EDITOR](../records/0114_editor.md); what it did not build is W-23, W-15, W-04 and B-043–B-044 |
| [`W-17-engine-defects.md`](open/W-17-engine-defects.md) | 2026-10-20 | the fixes and their regression tests in `crates/`; deviations in [SR-ARCHITECTURE](../records/0108_architecture.md) 15a and [SR-RULE-PACKS](../records/0101_rule-packs.md) decision 7 |
| [`W-11-project-graph.md`](open/W-11-project-graph.md) | 2026-10-22 | `crates/codelaw-graph`; [SR-PROJECT-GRAPH](../records/0103_project-graph.md), its Consequences holding the measured false-positive rate |
| [`W-10-agent-surfaces.md`](open/W-10-agent-surfaces.md) | 2026-10-22 | `crates/codelaw-agents` and `agents/`; [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) |
| [`W-12-structured-file-rules.md`](open/W-12-structured-file-rules.md) | 2026-10-22 | `crates/codelaw-match` (`data`, `target`, `schema`) and `crates/codelaw-fix/src/structured.rs`; [SR-RULE-FORMAT](../records/0112_rule-format.md) decision 3e |
| [`W-13-baseline-and-time.md`](open/W-13-baseline-and-time.md) | 2026-10-22 | `crates/codelaw-baseline`, `crates/codelaw-engine/src/{baseline,git}.rs`, `crates/codelaw-graph/src/deprecated.rs`; [SR-BASELINE](../records/0104_baseline-and-deprecations.md) |
| [`W-14-adapters.md`](open/W-14-adapters.md) | 2026-10-22 | `crates/codelaw-adapters` and `crates/codelaw-engine/src/adapters.rs`; [SR-LINTER](../records/0106_linter.md) decision 2 and [SR-ARCHITECTURE](../records/0108_architecture.md) decision 19 |
| [`W-15-warm-processes.md`](open/W-15-warm-processes.md) | 2026-10-22 | `crates/codelaw-cli/src/warm.rs`, `crates/codelaw-engine/src/incremental.rs`, the LSP's tree cache; [SR-ARCHITECTURE](../records/0108_architecture.md) decisions 21–24 |
| [`W-16-commands-and-api.md`](open/W-16-commands-and-api.md) | 2026-10-22 | `crates/codelaw-cli`, `codelaw.schema.json` and the npm/Python API; [SR-CLI-MCP](../records/0115_cli-and-mcp.md), [SR-CONFIG](../records/0110_config.md) and [SR-PACKAGES](../records/0107_packages.md) |
| [`W-23-formatter-rest.md`](open/W-23-formatter-rest.md) | 2026-10-22 | `crates/codelaw-format`, `crates/codelaw-test`'s layout refusal, `crates/codelaw-lsp` range formatting; [SR-FORMATTER](../records/0105_formatter.md) §What is built and decision 3; B-045 |
| [`W-22-record-drift.md`](open/W-22-record-drift.md) | 2026-10-20 | [SR-RULE-PACKS](../records/0101_rule-packs.md), [SR-PACKAGES](../records/0107_packages.md) decision 9, [SR-ARCHITECTURE](../records/0108_architecture.md) decision 1 |
| [`W-21-pack-engine-gaps.md`](open/W-21-pack-engine-gaps.md) | 2026-10-24 | the engine in `crates/codelaw-lang` (regions, the tree-sitter-css fork) and `crates/codelaw-engine` (file context); [SR-ARCHITECTURE](../records/0108_architecture.md) decisions 3 and 15b; the upstream PR is [B-046](../work/BACKLOG.md), the CSS-tree port [B-047](../work/BACKLOG.md) |
| [`W-28-layout-check-by-tokens.md`](open/W-28-layout-check-by-tokens.md) | 2026-10-24 | `codelaw_lang::token_stream` and `codelaw-test`'s layout check; [SR-FORMATTER](../records/0105_formatter.md) decision 3, [SR-RULE-AUTHORING](../records/0113_rule-authoring.md) decision 4 |
| [`W-26-public-docs-from-the-tree.md`](open/W-26-public-docs-from-the-tree.md) | 2026-10-24 | `scripts/docs-sync.py` and the README/CHANGELOG regions it keeps; [SR-WORK-REGISTRY](../records/0060_WORK-registry.md) Consequences |
| [`W-25-governance-drift.md`](open/W-25-governance-drift.md) | 2026-10-24 | the work system's checks: `scripts/check-work-system.py` (rules 40, 64; SR-WORK-OWNERSHIP decisions 2, 9); [SR-WORK-OPEN-QUEUE](../records/0051_WORK-open-queue.md), [SR-WORK-OWNERSHIP](../records/0058_WORK-ownership.md) |

## `backlog/`

A backlog row has no file, so nothing of it is archived but its id: each
deleted `B-nnn` that a document still names gets a row here
([SR-WORK-ARCHIVE](../records/0057_WORK-archive.md) decision 3). B-006, B-007,
B-043, B-044 and B-045 are live in [`work/BACKLOG.md`](../work/BACKLOG.md).

| id | deleted | what absorbed it |
|---|---|---|
| B-001–B-005 | 2026-10-20 | W-20: `scripts/check-work-system.py` — the queue's row rules, `owns:` ↔ the Ownership table, the registry, lost `W-nn` ids, one archive |
| B-008 | 2026-10-20 | W-02 (Cargo) and W-20 (npm, PyPI tools): `crates/codelaw-cli/tests/licences.rs` |
| B-009 | 2026-10-20 | W-02: `crates/codelaw-cli/tests/laws.rs` `l3_*`; the runtime fence and install matrix are W-18's (`scripts/smoke.sh`) |
| B-010 | 2026-10-19 | W-02: `crates/codelaw-cli/tests/laws.rs` `l4_*` |
| B-011 | 2026-10-20 | W-02 (determinism, CLI/library parity: `laws.rs` `l5_*`) and W-20 (`tests/network_fence.rs`) |
| B-012 | 2026-10-20 | W-04 (`scripts/version.py --check`) and W-18 (the install matrix, `scripts/smoke.sh`) |
| B-013 | 2026-10-20 | W-11: the corpus run in SR-PROJECT-GRAPH's Consequences |
| B-014 | 2026-10-20 | W-13: the veto check in SR-BASELINE, `crates/codelaw-baseline/src/tests.rs` |
| B-015 | 2026-10-20 | W-07: `scripts/format-parity.py`, the parity table in SR-FORMATTER |
| B-016 | 2026-10-20 | W-19 (`codelaw bench`) and the region tests named in SR-LINTER's veto |
| B-017 | 2026-10-20 | W-07: `crates/codelaw-cli/tests/laws.rs` `l7_*` |
| B-018 | 2026-10-20 | W-19: `codelaw bench`, `perf/budgets.toml`, `ci.yml`'s `perf` job |
| B-019 | 2026-10-20 | W-19: the proposed budgets; setting them is the decider's (the OPEN-WORK inbox) |
| B-020 | 2026-10-20 | W-02 (`crates/codelaw-cli/tests/layers.rs`) and W-04 (the names check in `scripts/package.py check`) |
| B-021 | 2026-10-19 | W-02: `crates/codelaw-engine/tests/engine.rs` `cold_warm_and_partial_cache_agree` |
| B-022 | 2026-10-20 | W-16: the schema-vs-records test |
| B-023 | 2026-10-20 | W-20: `crates/codelaw-cli/tests/consumer_layout.rs` |
| B-024 | 2026-10-20 | W-20: `codelaw rules lint`, `crates/codelaw-cli/tests/promotion.rs`; the corpus job is still W-20's |
| B-025 | 2026-10-20 | W-08: `crates/codelaw-cli/tests/mcp_parity.rs`; the LSP side is W-09's |
| B-026 | 2026-10-20 | W-17 (the panic fixture) and W-15 (stale-socket cleanup) |
| B-027 | 2026-10-20 | W-19: the cold targets measured by `codelaw bench` |
| B-028 | 2026-10-20 | W-07 and W-23: `crates/codelaw-format` |
| B-029 | 2026-10-20 | W-11: `crates/codelaw-graph` |
| B-030 | 2026-10-20 | W-12: `target:` and `assert:` rules, the structured fix primitives |
| B-031 | 2026-10-20 | W-13: `crates/codelaw-baseline` |
| B-032 | 2026-10-20 | W-14: `crates/codelaw-adapters` |
| B-033 | 2026-10-20 | W-08: `crates/codelaw-mcp` |
| B-034 | 2026-10-20 | W-09: `crates/codelaw-lsp`, `packaging/vscode` |
| B-035 | 2026-10-20 | W-10: `crates/codelaw-agents`, `agents/` |
| B-036 | 2026-10-20 | W-15: the warm processes |
| B-037 | 2026-10-20 | W-16 (the commands and flags), W-13 (`--changed-since`, `--staged`), W-19 (`bench`) |
| B-038 | 2026-10-20 | W-04: the packaging and the release pipeline |
| B-039 | 2026-10-20 | W-21: tagged templates as regions |
| B-040 | 2026-10-20 | W-21: native rules' file context |
| B-041 | 2026-10-20 | W-21: the in-tree tree-sitter-css fork |
| B-042 | 2026-10-20 | W-16: `rulesFor`/`rules_for`, `format` and `codelaw.schema.json` in both APIs |

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
