---
type: Log
description: "What shipped, when, and how it turned out — the evidence store the queue reconciles against."
---

# IMPLEMENTATION — what has landed

A row is earned by **landing**, never by being planned. The queue reconciles
against this file before an item may be closed
([SR-WORK-OPEN-QUEUE](../records/0051_WORK-open-queue.md) rule 9). Newest first.

| date | item | what landed | outcome | evidence |
|---|---|---|---|---|
| 2026-10-20 | W-04 | The release pipeline: `ci.yml` (tests, clippy, the SR-guard replay, eight maturin platform builds, assembly, the release checks, the npm-vs-PyPI install smoke) and `publish.yml` (gate, OIDC to npm and PyPI, no rebuild); the npm `codelaw` launcher and API, the PyPI module, `LICENSE-MIT` / `LICENSE-APACHE`, `RELEASING.md` | green on branch `ci/w-04-release-pipeline`, all eight platforms; not yet on `main`; nothing published | `.github/workflows/`, `packaging/`, `scripts/{version,package}.py`, `scripts/{smoke,withdraw,npm-trust}.sh` |
| 2026-10-20 | W-03 (part) | `packs/python`: 98 rules — Ruff 0.17.1's E4/E7/E9/F (55 of 59), all 39 stable flake8-bugbear rules, I001, RUF022, and `py/conventions`; the Python semantic model reads annotation subscripts as references | landed; 605 checks green; matches Ruff on three corpora except `# noqa` and documented gaps | `packs/python/`, `crates/codelaw-native/src/python_*.rs`, `crates/codelaw-semantic/src/python.rs` |
| 2026-10-20 | W-03 (part) | `packs/html`: 71 native rules — html-validate 11.16.4's `recommended`, `a11y` and `document` presets without layout rules — over a rebuilt element tree; element data from html-validate (MIT) | landed; 349 checks green | `packs/html/`, `crates/codelaw-native/src/html_*.rs` |
| 2026-10-20 | W-03 (part) | `packs/css`: 70 native rules — all 41 of stylelint-config-recommended 18.0.0 (`css/recommended`) and the 28 non-layout rules of stylelint-config-standard 40.0.0 (`css/standard`) — over a hand-written CSS reader; data tables from MIT sources only | landed; 385 checks green | `packs/css/`, `crates/codelaw-native/src/css_*.rs` |
| 2026-10-20 | W-03 (part) | `packs/data`: 30 rules in 7 presets — JSON, YAML and TOML correctness, 16 markdownlint 0.42.0 rules, `package.json`, `tsconfig`, `pyproject` and GitHub Actions presets | landed; 216 checks green; SchemaStore validation waits on B-030 | `packs/data/`, `crates/codelaw-native/src/data_*.rs` |
| 2026-10-20 | W-03 (part) | `packs/js` completed: `ts`, `react`, `node`, `stencil` presets and `js/conventions` — 132 rules | landed; 1,505 checks green | `packs/js/` |
| 2026-10-19 | W-03 (part) | `packs/js` with `js/recommended`: all 64 rules of ESLint 10.0.1's recommended preset, reimplemented; 26 native rules; five semantic predicates; the `globals` data tables | landed; the pack's tests green; W-03 stays open for the other presets and packs | `packs/js/`, `crates/codelaw-native/src/` |
| 2026-10-19 | W-02 | The engine: 13 crates (core, cache, config, lang, rules, semantic, match, native, fix, engine, report, test, cli) and the `codelaw` binary — check, fix, test, rules, explain, tree, new-rule, init, cache; the semantic model for JS/TS and Python; `native:` rules; pack resolution from npm and Python installs; the layer, L2-licence, L3, L4 and L5 checks | done; `cargo test --workspace` green, clippy clean | `crates/`, `fixtures/l3/`, `crates/codelaw-cli/tests/`, `archive/open/W-02-engine-p0.md` |
| 2026-10-19 | W-01 | L6 (decisions 3, 4, 9–11) and SR-RULE-PACKS, SR-PACKAGES, SR-ARCHITECTURE (15a–15c), SR-CONFIG, SR-CONSUMER-LAYOUT, SR-RULE-FORMAT, SR-LINTER, SR-AGENT-SURFACES rewritten for five separately published rule packs; SR-RULE-PACKS accepted | done | `records/0101_rule-packs.md`, `records/0008_LAW-6-python-and-node-packages.md`, `archive/open/W-01-rule-pack-records.md` |
| 2026-10-19 | — | law L5 (no model in the engine) and fourteen records (SR-AGENT-SURFACES, SR-PROJECT-GRAPH, SR-BASELINE, SR-FORMATTER, SR-LINTER, SR-PACKAGES, SR-ARCHITECTURE, SR-CACHE, SR-CONFIG, SR-CONSUMER-LAYOUT, SR-RULE-FORMAT, SR-RULE-AUTHORING, SR-EDITOR, SR-CLI-MCP); L6 host floors Python ≥ 3.11, Node ≥ 22 | accepted on the decider's ruling | `records/README.md` status column |
| 2026-10-18 | — | law L2: every dependency free, open source and permissive (SR-LAW-2) | accepted on the decider's ruling | `records/0004_LAW-2-foss-dependencies.md` |
| 2026-10-18 | — | the work system: records 0001–0003, 0051–0060; `work/`; `archive/`; hooks; the commit guard | set up | the setup commit |
