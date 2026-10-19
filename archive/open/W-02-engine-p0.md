---
type: Handoff
name: W-02
description: "Arpit, 2026-10-19: the engine the rule packs run on — P0 of the design plus the semantic model, native rules and pack loading that porting real presets needs."
item: W-02
filed: 2026-10-19
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-19
updated_by: "Arpit (Claude Code, Opus 5.5)"
ball: agent
---

# W-02 — Build the engine the rule packs run on

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | The decider rules on the proposed engine records | a design of record to build against | ✅ accepted 2026-10-19 |
| 2 | Build P0: core, config, lang, rules loader, match, fix, engine, report, test, cli | `codelaw check`, `fix`, `test` on real files | ✅ 2026-10-19 |
| 3 | Add the per-file semantic model for JS/TS and Python | rules like `no-unused-vars` and `F401` become writable | ✅ 2026-10-19 |
| 4 | Add `native:` rules and pack loading (`extends` from npm or PyPI installs) | the five packs can plug in | ✅ 2026-10-19 |

**Arpit, 2026-10-19:** *"… and engine will be separate"*

**Model:** Opus: engine architecture where output quality is the deliverable and the layering, semantic model and determinism can't be checked by an existing test.

## Goal

A `codelaw` binary that loads rule packs from either registry and runs file rules, semantic-model rules and native rules deterministically, with `codelaw test` able to verify every pack rule.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · P0 crates | `codelaw-core`, `-cache`, `-config`, `-lang`, `-rules`, `-match`, `-fix`, `-engine`, `-report`, `-test`, `-cli` exist in the layer order of SR-PACKAGES decision 1, with the layer check (B-020) passing. | ✅ plus `-semantic` and `-native`; `crates/codelaw-cli/tests/layers.rs` |
| 2 · languages | All nine L3 languages parse, including TOML as a custom ast-grep language, and HTML/Markdown embedded regions. | ✅ `codelaw-lang`: TOML registered by codelaw; Markdown's inline grammar runs on `inline` nodes; HTML scripts, styles and `style=""`; Markdown frontmatter and fences |
| 3 · semantic model | Per file: scopes, bindings, references, imports and exports for JS/TS (incl. JSX/TSX) and Python. Exposed to rules as predicates (`unused`, `undeclared`, `shadowed`, `reassigned`, `imported-from`). | ✅ `codelaw-semantic`; `where:` syntax in SR-RULE-FORMAT decision 3a |
| 4 · native rules | A rule file whose `codelaw` block is `native: <id>` runs a Rust implementation registered in the engine; the Markdown still carries docs, fix class and examples. | ✅ `codelaw-native` with `js/no-unreachable` as the first |
| 5 · packs | `extends = ["codelaw:<namespace>/<preset>"]` resolves per [SR-RULE-PACKS](../../records/0101_rule-packs.md) decision 4 (`node_modules/@codelaw/rules-<pack>`, then the Python environment's `codelaw_rules_<pack>`); a missing pack is a clear error naming both install commands; an unmet `min_codelaw` or an unknown `native:` id is a load error. `codelaw init` installs the detected packs (decision 6). | ✅ resolution and errors tested on real folders; `init` runs the repo's package manager, tested with `--no-install` (no pack is published yet) |
| 6 · test | `codelaw test` runs examples, `.cases.md` and invariants (SR-RULE-AUTHORING levels 1–3). | ✅ `codelaw-test` |
| 7 · laws | L4 loader checks, L5 parity/determinism tests and the L3 single-language fixtures pass. | ✅ `crates/codelaw-cli/tests/laws.rs`, `fixtures/l3/`; parity covers CLI and library, the only surfaces built |

## Blockers

None. The engine records were accepted on 2026-10-19 and amended for the rule packs by W-01 the same day.

## What landed (2026-10-19)

- **13 crates** in `crates/`, each owned by a record in the register's Ownership table. `cargo test --workspace` and `cargo clippy --all-targets` are clean.
- **The binary:** `check`, `fix` (`--unsafe`, `--dry-run`), `test`, `rules` (`--for`, `--why`, `lint`, `eject`), `explain`, `tree`, `new-rule`, `init`, `cache`; text, JSON and agent output.
- **Checks:** layers (B-020), L2 licences over every Cargo dependency on every platform (B-008, Cargo part), the L3 matrix with the isolation diff (B-009), L4 fix fields and loader (B-010), L5 determinism, CLI/library parity and the no-model fence (B-011, part), cold/warm/partial cache (B-021).
- **Not in W-02, and in the backlog:** the formatter, project graph, baseline, adapters, MCP, LSP, agent surfaces, warm processes, `target:` rules and the `sort-keys`/`ensure-key`/`when` primitives, packaging.
- **Observed, not measured** (B-019 owns measurement): on the session's laptop, a release build checked 2,000 generated files in 0.73 s cold and 0.10 s warm, with identical output.

## Record affected

SR-ARCHITECTURE decisions 15a–15c (semantic model, native rules, pack loading) · SR-RULE-PACKS decisions 4–6 · SR-PACKAGES decision 1 · SR-LINTER decision 2 · `work/IMPLEMENTATION.md` on landing.

## Hazards

- **Scope analysis is the hard part.** Port the approach of `oxc_semantic` (MIT) for JS/TS and Ruff's semantic model (MIT) for Python; depending on either crate needs a record naming it (L2 permits, a record decides).
- **No type information.** Rules that need types stay out (L3 out-of-scope). Don't let a pack rule quietly approximate a type-aware check.
- **L8:** the semantic model runs on every file; it is in the hot path for `check_warm_10k`.

## Reference

- [SR-ARCHITECTURE](../../records/0108_architecture.md), [SR-PACKAGES](../../records/0107_packages.md), [SR-RULE-AUTHORING](../../records/0113_rule-authoring.md).
- [W-01](../../archive/open/W-01-rule-pack-records.md): the records this build follows. [W-03](W-03-rule-packs.md): what runs on it.
