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
| 2026-10-19 | W-02 | The engine: 13 crates (core, cache, config, lang, rules, semantic, match, native, fix, engine, report, test, cli) and the `codelaw` binary — check, fix, test, rules, explain, tree, new-rule, init, cache; the semantic model for JS/TS and Python; `native:` rules; pack resolution from npm and Python installs; the layer, L2-licence, L3, L4 and L5 checks | done; `cargo test --workspace` green, clippy clean | `crates/`, `fixtures/l3/`, `crates/codelaw-cli/tests/`, `archive/open/W-02-engine-p0.md` |
| 2026-10-19 | W-01 | L6 (decisions 3, 4, 9–11) and SR-RULE-PACKS, SR-PACKAGES, SR-ARCHITECTURE (15a–15c), SR-CONFIG, SR-CONSUMER-LAYOUT, SR-RULE-FORMAT, SR-LINTER, SR-AGENT-SURFACES rewritten for five separately published rule packs; SR-RULE-PACKS accepted | done | `records/0101_rule-packs.md`, `records/0008_LAW-6-python-and-node-packages.md`, `archive/open/W-01-rule-pack-records.md` |
| 2026-10-19 | — | law L5 (no model in the engine) and fourteen records (SR-AGENT-SURFACES, SR-PROJECT-GRAPH, SR-BASELINE, SR-FORMATTER, SR-LINTER, SR-PACKAGES, SR-ARCHITECTURE, SR-CACHE, SR-CONFIG, SR-CONSUMER-LAYOUT, SR-RULE-FORMAT, SR-RULE-AUTHORING, SR-EDITOR, SR-CLI-MCP); L6 host floors Python ≥ 3.11, Node ≥ 22 | accepted on the decider's ruling | `records/README.md` status column |
| 2026-10-18 | — | law L2: every dependency free, open source and permissive (SR-LAW-2) | accepted on the decider's ruling | `records/0004_LAW-2-foss-dependencies.md` |
| 2026-10-18 | — | the work system: records 0001–0003, 0051–0060; `work/`; `archive/`; hooks; the commit guard | set up | the setup commit |
