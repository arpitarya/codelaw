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
| 2026-10-19 | — | law L5 (no model in the engine) and fourteen records (SR-AGENT-SURFACES, SR-PROJECT-GRAPH, SR-BASELINE, SR-FORMATTER, SR-LINTER, SR-PACKAGES, SR-ARCHITECTURE, SR-CACHE, SR-CONFIG, SR-CONSUMER-LAYOUT, SR-RULE-FORMAT, SR-RULE-AUTHORING, SR-EDITOR, SR-CLI-MCP); L6 host floors Python ≥ 3.11, Node ≥ 22 | accepted on the decider's ruling | `records/README.md` status column |
| 2026-10-18 | — | law L2: every dependency free, open source and permissive (SR-LAW-2) | accepted on the decider's ruling | `records/0004_LAW-2-foss-dependencies.md` |
| 2026-10-18 | — | the work system: records 0001–0003, 0051–0060; `work/`; `archive/`; hooks; the commit guard | set up | the setup commit |
