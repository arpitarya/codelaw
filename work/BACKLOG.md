---
type: Queue
description: "The named-but-unclaimed: everything a record, proposal or compare doc says is outstanding and nobody is doing. The list only — its rules live in the record below."
governed_by: SR-WORK-BACKLOG
governed_by_path: records/0052_WORK-backlog.md
---

# BACKLOG — what has been named and is not being done

**This file is the LIST.** Everything else — what the backlog is, what feeds
it, the shape of a row, the five classes, how an item leaves, and the one
obligation it puts on every session — is stated once in
**[SR-WORK-BACKLOG](../records/0052_WORK-backlog.md)** and is not repeated here.

⚠ **Its length is not a signal.** `OPEN-WORK.md`'s length is how much is
pending; this file's length is how honest the records have been. Do not trim
it, and do not read it top-down as a priority order.

**Nothing here is in [`OPEN-WORK.md`](OPEN-WORK.md).**

---

## `unbuilt` — a ratified decision the code does not implement

| id | what is outstanding | named in | what closes it |
|---|---|---|---|
| B-028 | The formatter engines, the `format` command and the format step of `fix`: codelaw lints but does not format yet, which L7 requires | [SR-FORMATTER](../records/0105_formatter.md) decisions · [SR-ARCHITECTURE](../records/0108_architecture.md) decisions 5–6 | `codelaw-format` and `codelaw format`, as a `W-nn` |
| B-029 | The project graph and `graph:` rules (`scope: project`), e.g. `graph/no-unused-exports` | [SR-PROJECT-GRAPH](../records/0103_project-graph.md) decisions · [SR-RULE-FORMAT](../records/0112_rule-format.md) decision 3 | `codelaw-graph`, as a `W-nn` |
| B-030 | `target:` + `assert:` rules for JSON/YAML/TOML, and the `sort-keys`, `ensure-key` and `when`/`else` fix primitives | [SR-RULE-FORMAT](../records/0112_rule-format.md) decisions 3 and 3c | the structured-file engine and the primitives, as a `W-nn` |
| B-031 | Baselines, the deprecations registry, `--as-of` and the engine's `graph/no-deprecated-usage` | [SR-BASELINE](../records/0104_baseline-and-deprecations.md) decisions · [SR-RULE-PACKS](../records/0101_rule-packs.md) decision 1 | `codelaw-baseline`, as a `W-nn` |
| B-032 | Adapters (oxlint, Ruff, stylelint, html-validate) and their manifests | [SR-LINTER](../records/0106_linter.md) decision 2 · [SR-ARCHITECTURE](../records/0108_architecture.md) decision 19 | `codelaw-adapters`, as a `W-nn` |
| B-033 | The MCP server and its tools | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) decision 5 | `codelaw-mcp`, as a `W-nn` |
| B-034 | The language server and the editor extension | [SR-EDITOR](../records/0114_editor.md) decisions | `codelaw-lsp` and the extension, as a `W-nn` |
| B-035 | Hook mode, `sync-agents`, the skills and `init --agents` | [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md) decisions | `codelaw-agents`, as a `W-nn` |
| B-036 | Warm processes: the daemon, warm sockets, `--prefer-warm`, incremental re-parse, the in-memory tier | [SR-ARCHITECTURE](../records/0108_architecture.md) decisions 21–24 · [SR-CACHE](../records/0109_cache.md) decision 8 | a `W-nn` after B-033 or B-034 |
| B-037 | Commands and flags not built: `config --print`/`--schema` and `codelaw.schema.json`, `match --explain`, `fix --trace`, `graph refs`, `bench`, `--changed-since`, `--staged`, `--timings`, `--prefer-warm`, `--format sarif` | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) decisions 1, 2 and 4 · [SR-CONFIG](../records/0110_config.md) decisions 6–7 · [SR-RULE-AUTHORING](../records/0113_rule-authoring.md) decision 3 | each with its feature, as a `W-nn` |
| B-038 | Packaging: npm `codelaw` (launcher, JS API, types), the eight `@codelaw/cli-*` binaries, the PyPI wheels, the pack tarballs and wheels, one release job | [SR-PACKAGES](../records/0107_packages.md) decisions 3–7 and 9 · [SR-LAW-6](../records/0008_LAW-6-python-and-node-packages.md) | the release job, as a `W-nn` (with B-012) |
| B-039 | `` css`…` `` / `` html`…` `` tagged templates as embedded regions | [SR-LINTER](../records/0106_linter.md) decision 3 | a region extractor for template literals, as a `W-nn` |
| B-040 | Native rules see one file's text, never its path or the files around it, so `md/relative-links` (starter) can't check a link target exists | [SR-ARCHITECTURE](../records/0108_architecture.md) decision 15b · [SR-RULE-PACKS](../records/0101_rule-packs.md) decision 8 | the file path, and a read-only view of the checked tree, on `NativeContext`, as a `W-nn` |
| B-041 | tree-sitter-css errors on modern CSS (range media queries, `layer()` in `@import`, percentage keyframe selectors), so `packs/css` reads CSS itself and `css/media-feature-range-notation` can't be `auto` (the fix verifier refuses output that won't parse) | [SR-LINTER](../records/0106_linter.md) decision 3 · [L4](../records/0006_LAW-4-every-rule-has-a-fix.md) | a newer CSS grammar; then the CSS rules can use `match:` and the fix can be `auto` |

---

## `ungated` — a stated rule nothing mechanically checks

| id | what is outstanding | named in | what closes it |
|---|---|---|---|
| B-001 | The queue's row rules (13–19, 42, 45) are enforced by nothing but the inbox-shape hook | [SR-WORK-OPEN-QUEUE](../records/0051_WORK-open-queue.md) Consequences | a test over `work/OPEN-WORK.md` rows, as a `W-nn` |
| B-002 | `owns:` ↔ Ownership table agreement is unchecked in either direction | [SR-WORK-OWNERSHIP](../records/0058_WORK-ownership.md) decision 2 | a test parsing both, as a `W-nn` |
| B-003 | The registry's decisions 5–8 are enforced by nothing | [SR-WORK-REGISTRY](../records/0060_WORK-registry.md) Consequences | a test over §3, as a `W-nn` |
| B-004 | A `W-nn` that resolves to neither `work/open/` nor `archive/open/` is caught by nobody | [SR-WORK-OPEN-QUEUE](../records/0051_WORK-open-queue.md) Consequences | a test, as a `W-nn` |
| B-005 | A second `archive/` directory anywhere is caught by nobody | [SR-WORK-ARCHIVE](../records/0057_WORK-archive.md) decision 1 | a test, as a `W-nn` |
| B-009 | L3's runtime fence is checked by nothing: no Python-only run without Node, no JS-only run without Python (the fixture matrix and isolation diff exist: `crates/codelaw-cli/tests/laws.rs`) | [SR-LAW-3](../records/0005_LAW-3-languages.md) "How L3 is enforced" | the fence in CI once B-038 gives each registry its package |
| B-011 | L5's network fence is checked by nothing, and parity covers only the CLI and the library (determinism and the no-model fence exist: `crates/codelaw-cli/tests/laws.rs`) | [SR-LAW-5](../records/0007_LAW-5-no-model-in-the-engine.md) "How L5 is enforced" | the suite run without networking in CI, and hook/MCP parity with B-033 and B-035 |
| B-012 | L6 is checked by nothing: no version-sync check across `package.json`, `pyproject.toml` and the binary, no install matrix | [SR-LAW-6](../records/0008_LAW-6-python-and-node-packages.md) Consequences | both checks in the release job, as a `W-nn` when the first manifest lands |
| B-017 | L7 is checked by nothing: no test that every L3 language has a lint engine and a formatter | [SR-LAW-7](../records/0009_LAW-7-linter-and-formatter.md) Consequences | a fixture-matrix assertion, as a `W-nn` when the engine lands |
| B-018 | L8 is checked by nothing: no `codelaw bench`, no `perf/budgets.toml`, no CI perf gate | [SR-LAW-8](../records/0010_LAW-8-performance-budgets.md) decision 7 | the bench command, the reference workload and the gate, as a `W-nn` when the engine lands |
| B-020 | No check that only the SR-PACKAGES names are published (the layer check exists: `crates/codelaw-cli/tests/layers.rs`) | [SR-PACKAGES](../records/0107_packages.md) veto | a release-manifest check in B-038's release job |
| B-022 | No test that every config key is defined in exactly one record and present in the schema | [SR-CONFIG](../records/0110_config.md) veto | a schema-vs-records test, as a `W-nn` |
| B-023 | No test that `codelaw init` writes only the paths SR-CONSUMER-LAYOUT lists | [SR-CONSUMER-LAYOUT](../records/0111_consumer-layout.md) veto | an init fixture test, as a `W-nn` |
| B-024 | No CI job that promotes a rule to stable only after all four test levels, and no corpus level (`codelaw test --corpus`); `codelaw rules lint` exists | [SR-RULE-AUTHORING](../records/0113_rule-authoring.md) decisions 4 and 6 | the corpus runner and the promotion job, as a `W-nn` |
| B-026 | No fixture for a rule that panics (run continues, exit 2 at the end) and no stale-socket cleanup test | [SR-ARCHITECTURE](../records/0108_architecture.md) decisions 21 and 25 | both fixtures, as a `W-nn` once the engine exists |
| B-025 | No parity test across CLI, MCP and LSP over every fixture | [SR-CLI-MCP](../records/0115_cli-and-mcp.md) decision 6 · [SR-EDITOR](../records/0114_editor.md) decision 3 | one parity test, as a `W-nn` |
| B-008 | L2's licence check covers the Cargo manifest only (`crates/codelaw-cli/tests/licences.rs`); the npm and PyPI manifests don't exist yet | [SR-LAW-2](../records/0004_LAW-2-foss-dependencies.md) Consequences | the same check over `package.json` and `pyproject.toml` when B-038 lands them |

---

## `unmeasured` — a live default, constant or claim with no measurement behind it

| id | what is outstanding | named in | what closes it |
|---|---|---|---|
| B-013 | The project graph's false-positive rate on real repos is unmeasured | [SR-PROJECT-GRAPH](../records/0103_project-graph.md) veto condition | a corpus run over pinned real repos, as a `W-nn` once the graph exists |
| B-027 | The cold targets (`check_cold_10k` < 20 s, `fix_one_file` < 300 ms cold, `peak_memory_10k` < 1 GiB) are unmeasured design targets | [SR-ARCHITECTURE](../records/0108_architecture.md) decision 29 | the first `codelaw bench` run (B-019) |
| B-019 | Every L8 budget is `unset`: nothing has been measured to propose numbers from | [SR-LAW-8](../records/0010_LAW-8-performance-budgets.md) decisions 1 and 4 | a first `codelaw bench` run and a budget proposal for the decider to rule on |
| B-016 | The linter's performance budget (10,000 files < 2 s warm; one file < 100 ms in hook mode) and injection location mapping are unmeasured | [SR-LINTER](../records/0106_linter.md) decision 8 and veto condition | a benchmark run and `codelaw test --injections`, as a `W-nn` once the engine exists |
| B-015 | How close codelaw's formatting is to Prettier (JS/TS/JSON/YAML/Markdown) and Black (Python) on real code is unmeasured | [SR-FORMATTER](../records/0105_formatter.md) Consequences | a corpus run diffing codelaw against Prettier and Black output, as a `W-nn` once formatting exists |
| B-014 | Whether count-based baselines let a new use slip in by offsetting a removed one is unmeasured | [SR-BASELINE](../records/0104_baseline-and-deprecations.md) veto condition | a check of CI history once baselines are in use |

---

## `unruled` — a fork named and not decided

| id | what is outstanding | named in | what closes it |
|---|---|---|---|

---

## `cost` — a consequence accepted deliberately

| id | what is outstanding | named in | what closes it |
|---|---|---|---|
| B-006 | The guard proves a record was touched, never that it is coherent; L0 obligation (iii) is judgment | [SR-LAW-0](../records/0002_LAW-0-authority.md) decision 1a | never graduates; it is the stated price of a mechanical gate |
| B-007 | The `Stop` hook relents after three refusals, so a blocker can be walked past | [SR-WORK-BLOCKERS](../records/0056_WORK-blockers.md) decision 8 | never graduates; a hook that never relents is a hang |
