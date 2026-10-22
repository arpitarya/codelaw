---
type: Register
description: "The SR register: the convention, the ownership table, and the state of every record."
---

# SRs — the decision records

**How to use this file.** The naming convention, the record shape, the
ownership table and the rules every record obeys. Read it before writing a
record, before citing one, and before adding a module under `scripts/` and `.claude/hooks/`.

One record per completed feature or ruled measurement. **Every record is cited
by NAME in prose, never by number.**

## What an SR is — and what it is not

A Standing Record is a **maintained specification**, not a decision-log entry:
amended in the same change as the code (law zero); superseded records are
rewritten or deleted; retired records are archived (L1); cited by name because
the content moves; it **owns** components and carries a veto condition to check
*today*. The decision-log role lives in `work/compare/` (the fork, its verdict,
its reopen-trigger) and `work/WORKLOG.md`.

## One directory, one state

`records/` is the only home a LIVE record has. A superseded record is rewritten
or deleted in the change that supersedes it; a **retired** record is moved to
`archive/records/` with `status: archived`, a dated banner and an
`archive/README.md` row (L1), and from then on is named, never cited.

Three files here are NOT records and carry no number: this register,
`TEMPLATE.md`, and `RULE-SINCE`.

## The three kinds

Every record declares a `kind`; it answers *what would have to change for this
record to become wrong?*

| `kind` | becomes wrong when | the gate the kind selects |
|---|---|---|
| `law` | The decider rules differently | precedence: a record conflicting with a law is void in the conflicting part |
| `component` | the code changes | `scripts/sr-guard.sh` — the owning record is touched in the same change |
| `process` | the way work is done changes | it owns its enforcement — a hook or script today, a test once tests exist |

## The convention

**Path.** `records/NNNN_<short-name>.md`, on three ranges:

| range | holds | a new record takes |
|---|---|---|
| `0001`–`0050` | the **Law** records only — `0001` SR-LAWS, `0002` SR-LAW-0, `0003` SR-LAW-1, `0004` SR-LAW-2, `0005` SR-LAW-3, `0006` SR-LAW-4, `0007` SR-LAW-5, `0008` SR-LAW-6, `0009` SR-LAW-7, `0010` SR-LAW-8 | the next free number after the highest law |
| `0051`–`0100` | the **WORK** records only — how work is done | the next free number after the highest |
| `0101`– | every other record | the next free number after the highest |

Gaps are reservations; no placeholder files. The number is a filename ordinal,
**not an identity**. **Cite by NAME, never by number.**

### Frontmatter is the metadata, and the body never restates it

Keys, in this order: `type` · `kind` · `name` · `title` · `description` ·
`status` · `date` · `feature` · `owns` · `laws` · `timestamp` · `content_sha`;
optional `supersedes`, `ratifies`. See `TEMPLATE.md` for each value. **Quote
any value containing `: `.** The body opens at §1 and restates none of it.

### A record states what is true now. It carries no history.

No `Amended` sections. When a decision changes, rewrite the sentence in place,
in the same commit. The one exception is an argument that still binds — it
belongs in *Alternatives considered*.

### Two sections and a bibliography

**§1 — For humans**: one screen; a diagram (Mermaid and a collapsed ASCII
twin, updated together). **§2 — For agents**: Context · Decision (numbered) ·
Consequences · Alternatives considered · Reference (required) · Veto
condition. **References**: every source cited, grouped; nothing new; never an
archived document.

**The reference is grounded** — never an archived doc. **The veto condition is
a condition, not an event** — checkable today with a command.

**A record that restates a cross-cutting principle is a bug.** Each law lives
in its own `SR-LAW-n` record; every other record names it in `laws:` and moves
on.

### Records are kept current by a check, not by good intentions

| where | what it does |
|---|---|
| `scripts/sr-guard.sh` | a `commit-msg` hook: refuses a commit that changes an owned component without staging its owning record and every describer — unless the message carries `no SR affected` on its own line. Install: `ln -sf ../../scripts/sr-guard.sh .git/hooks/commit-msg` |
| `records/RULE-SINCE` | the baseline commit sha from which the rule applies, for a CI replay once one exists |

**The escape hatch is a claim in git history under your name**, not a silent
skip. What none of this proves: that a touched record is **coherent**.

Start from `TEMPLATE.md`.

---

## The register

`built` says whether code implements the record's decisions — `yes`, `partly` or `no`; [`work/IMPLEMENTATION.md`](../work/IMPLEMENTATION.md) says what landed.

| # | name | title | status | built |
|---|------|-------|--------|-------|
| [0001](0001_LAWS.md) | **SR-LAWS** | The laws have exactly one home each; this record routes the handles | accepted | yes |
| [0002](0002_LAW-0-authority.md) | **SR-LAW-0** | **L0** — records are the only source of truth; Law records outrank every other record | accepted | yes |
| [0003](0003_LAW-1-retired-records-archived.md) | **SR-LAW-1** | **L1** — a retired record is archived, never deleted | accepted | yes |
| [0004](0004_LAW-2-foss-dependencies.md) | **SR-LAW-2** | **L2** — every dependency is free, open source and permissive | accepted | partly |
| [0005](0005_LAW-3-languages.md) | **SR-LAW-3** | **L3** — nine languages — JS, TS, Python, JSON, YAML, TOML, Markdown, CSS, HTML — each alone or in any combination | accepted | partly |
| [0006](0006_LAW-4-every-rule-has-a-fix.md) | **SR-LAW-4** | **L4** — every rule has a fix — `auto` wherever safe, otherwise `suggest` or `manual`, and a skill carries every `manual` fix | accepted | partly |
| [0007](0007_LAW-5-no-model-in-the-engine.md) | **SR-LAW-5** | **L5** — no model in the engine; works the same with or without an agent | accepted | partly |
| [0008](0008_LAW-6-python-and-node-packages.md) | **SR-LAW-6** | **L6** — one release — `codelaw` on PyPI and on npm — each a complete engine, identical output; rules as five packs on both (`@codelaw/rules-*`, `codelaw-rules-*`); npm sub-packages only as `@codelaw/*` | accepted | no |
| [0009](0009_LAW-7-linter-and-formatter.md) | **SR-LAW-7** | **L7** — always both a linter and a formatter, for every L3 language | accepted | no |
| [0010](0010_LAW-8-performance-budgets.md) | **SR-LAW-8** | **L8** — performance budgets — set from measurement, never exceeded, never loosened | accepted | no |
| [0051](0051_WORK-open-queue.md) | **SR-WORK-OPEN-QUEUE** | How `OPEN-WORK.md` works — the queue discipline in eight groups | accepted | yes |
| [0052](0052_WORK-backlog.md) | **SR-WORK-BACKLOG** | How `BACKLOG.md` works — the named-but-unclaimed in five classes | accepted | yes |
| [0053](0053_WORK-lifecycle.md) | **SR-WORK-LIFECYCLE** | How a feature travels — compare, proposal, handoff in the item file, the model line, one record | accepted | yes |
| [0054](0054_WORK-session.md) | **SR-WORK-SESSION** | What a session owes before it ends; the two-strikes rule | accepted | yes |
| [0055](0055_WORK-docs.md) | **SR-WORK-DOCS** | The documentation contract — form, steering-file edits, the documents every task updates | accepted | yes |
| [0056](0056_WORK-blockers.md) | **SR-WORK-BLOCKERS** | A blocker is a file, not a remark — `work/BLOCKED.json` and the three hooks | accepted | yes |
| [0057](0057_WORK-archive.md) | **SR-WORK-ARCHIVE** | One archive; named, never cited | accepted | yes |
| [0058](0058_WORK-ownership.md) | **SR-WORK-OWNERSHIP** | `owns` and `describes`, and the commit guard | accepted | yes |
| [0059](0059_WORK-governance.md) | **SR-WORK-GOVERNANCE** | The governance map — which file governs what; no counts | accepted | yes |
| [0060](0060_WORK-registry.md) | **SR-WORK-REGISTRY** | The doc registry — rules in §2, the table in §3 | accepted | yes |
| [0101](0101_rule-packs.md) | **SR-RULE-PACKS** | Five first-party rule packs on npm and PyPI apart from the engine; team packs; project rules in `.codelaw/rules` | accepted | partly |
| [0102](0102_agent-surfaces.md) | **SR-AGENT-SURFACES** | Skills, steering, instructions, agent files and hooks, generated from one source | accepted | no |
| [0103](0103_project-graph.md) | **SR-PROJECT-GRAPH** | A whole-repo symbol graph for cross-file rules — unused exports first | accepted | no |
| [0104](0104_baseline-and-deprecations.md) | **SR-BASELINE** | Grandfather violations with a deadline and a touch policy; the deprecations registry | accepted | no |
| [0105](0105_formatter.md) | **SR-FORMATTER** | codelaw is a linter and a formatter — embedded engines, layout owned by the formatter | accepted | no |
| [0106](0106_linter.md) | **SR-LINTER** | The linter — nine languages, file/project/adapter rules, embedded code, one diagnostic stream | accepted | partly |
| [0107](0107_packages.md) | **SR-PACKAGES** | What goes into which package — 20 internal crates; npm `codelaw` + 8 `@codelaw/cli-*`; PyPI `codelaw`; five rule packs on both; the editor extension | accepted | partly |
| [0108](0108_architecture.md) | **SR-ARCHITECTURE** | The design of record — layers, pipelines, file I/O rules, contracts for fixers and adapters, warm processes, failure handling | accepted | partly |
| [0109](0109_cache.md) | **SR-CACHE** | Content-addressed caches — safe to delete, never change output | accepted | partly |
| [0110](0110_config.md) | **SR-CONFIG** | The configuration file — three homes, sections and owners, precedence, strict validation | accepted | partly |
| [0111](0111_consumer-layout.md) | **SR-CONSUMER-LAYOUT** | What codelaw adds to a consumer repo — written, generated, derived, installed | accepted | no |
| [0112](0112_rule-format.md) | **SR-RULE-FORMAT** | The rule-file template — frontmatter schema, codelaw block, body order, examples, cases sidecar | accepted | partly |
| [0113](0113_rule-authoring.md) | **SR-RULE-AUTHORING** | Writing rules by hand or with an agent skill; the debug tools; four test levels | accepted | partly |
| [0114](0114_editor.md) | **SR-EDITOR** | The LSP server in the binary and the thin VS Code extension | accepted | partly |
| [0115](0115_cli-and-mcp.md) | **SR-CLI-MCP** | The CLI and the MCP server — one engine, two doors, same answers | accepted | partly |

---

## Ownership — which record owns which component

**This table is the answer, not a judgement call.** Every component under
`scripts/`, `.claude/hooks/`, `crates/`, `fixtures/` and `packs/`, and the
workspace `Cargo.toml`, appears here exactly once. Most specific wins; a carve-out is
justified by a *different decision*. A component with no decision yet is owned
by an open `W-nn`. A record's `owns:` key and this table agree in both
directions.

<!-- OWNERSHIP-TABLE-START -->

| component | owner | note |
|---|---|---|
| `scripts/sr-guard.sh` | SR-WORK-OWNERSHIP | the guard is the executable form of decisions 3 and 6 |
| `scripts/check-open-work-inbox.py` | SR-WORK-OPEN-QUEUE | rule 45a's check |
| `scripts/build-packs.py` | SR-PACKAGES | decision 9: the pack tarballs and wheels, built from `packs/` |
| `.claude/hooks/guard-open-work-inbox.sh` | SR-WORK-OPEN-QUEUE | rule 45a, in-session |
| `.claude/hooks/inject-inbox.sh` | SR-WORK-BLOCKERS | decision 7 |
| `.claude/hooks/stop-if-blocked.sh` | SR-WORK-BLOCKERS | decision 7 |
| `.claude/hooks/session-lock.sh` | SR-WORK-BLOCKERS | decision 7 |
| `.claude/hooks/require-progress.sh` | SR-WORK-SESSION | decision 5 |
| `Cargo.toml` | SR-PACKAGES | decision 1: the workspace and its one version |
| `crates/codelaw-core` | SR-ARCHITECTURE | decision 13: offsets and positions; the shared types |
| `crates/codelaw-cache` | SR-CACHE | decisions 1–6 |
| `crates/codelaw-config` | SR-CONFIG | decisions 1–5 and 9 |
| `crates/codelaw-lang` | SR-LINTER | decisions 1 and 3: grammars and embedded regions |
| `crates/codelaw-rules` | SR-RULE-FORMAT | decisions 1–7: the rule-file parser and its load checks |
| `crates/codelaw-semantic` | SR-ARCHITECTURE | decision 15a |
| `crates/codelaw-match` | SR-LINTER | decision 2: file-scope `match:` rules |
| `crates/codelaw-native` | SR-ARCHITECTURE | decision 15b |
| `crates/codelaw-fix` | SR-ARCHITECTURE | decisions 5 and 18: the apply loop, verifier and external fixers |
| `crates/codelaw-format` | SR-FORMATTER | decisions 2, 4, 5 and 7: the embedded engines, their options and the verifier; `shims/option-ext` is SR-PACKAGES decision 1's L2 patch |
| `crates/codelaw-engine` | SR-ARCHITECTURE | decisions 4–28: the pipelines |
| `crates/codelaw-report` | SR-CLI-MCP | decisions 3 and 4 |
| `crates/codelaw-test` | SR-RULE-AUTHORING | decision 4 |
| `crates/codelaw-cli` | SR-CLI-MCP | decisions 1–3 |
| `crates/codelaw-lsp` | SR-EDITOR | decisions 1–3: the language server |
| `crates/codelaw-mcp` | SR-CLI-MCP | decisions 5–6: the MCP server and its parity with the CLI |
| `crates/codelaw-agents` | SR-AGENT-SURFACES | decisions 1, 2 and 6: hook mode, `sync-agents` and the compiled-in skills |
| `agents` | SR-AGENT-SURFACES | decisions 1, 4 and 5: the skill sources, the custom agent and the trigger evals |
| `crates/codelaw-cli/tests/layers.rs` | SR-PACKAGES | the veto's layer check (a different decision from the CLI's) |
| `crates/codelaw-cli/tests/licences.rs` | SR-LAW-2 | how L2 is enforced, for the Cargo manifest |
| `crates/codelaw-cli/tests/laws.rs` | SR-LAW-3 | the fixture matrix and isolation check; it also runs the L4 and L5 checks |
| `fixtures/l3` | SR-LAW-3 | the fixture matrix |
| `packs` | SR-RULE-PACKS | decisions 2, 3, 7 and 8: the five first-party packs' rules, presets and data |
| `LICENSE` | SR-PACKAGES | decision 10: the licence every package carries |
| `packaging` | SR-PACKAGES | decisions 3 and 5: the npm launcher and API, the PyPI module and its maturin build |
| `packaging/vscode` | SR-EDITOR | decision 4: the thin VS Code extension (SR-PACKAGES decision 6 is how it ships) |
| `scripts/version.py` | SR-PACKAGES | decision 7: the one version, written and checked everywhere |
| `scripts/format-parity.py` | SR-FORMATTER | the Prettier/Black parity measurement in its Consequences |
| `scripts/law-gate.py` | SR-PACKAGES | decision 11: refuses a release while the engine lacks `format`, `mcp` or `lsp` (L6 decision 3, L7) |
| `scripts/package.py` | SR-PACKAGES | decision 11: assembles every package and runs the release checks |
| `scripts/smoke.sh` | SR-PACKAGES | decision 11: the npm-vs-PyPI install smoke |
| `scripts/staging-index.py` | SR-PACKAGES | decision 11: the localhost staging index the install smoke publishes to and installs from |
| `scripts/withdraw.sh` | SR-PACKAGES | decision 12 |
| `scripts/npm-trust.sh` | SR-PACKAGES | decision 11: the one-time npm trusted-publisher setup |
| `.github/workflows` | SR-PACKAGES | decision 11: build once in CI, publish as built |
| `RELEASING.md` | SR-PACKAGES | decision 11: how a release is cut |
| `rust-toolchain.toml` | SR-PACKAGES | decision 13: the one pinned Rust toolchain |

<!-- OWNERSHIP-TABLE-END -->

## Describes — which record's subject REACHES INTO a component it does not own

Ownership stays one record per component; **describes is any number**, and the
guard demands the owner **and every describer**. Every row states its reason.
A component may be narrowed to symbols: `` `path::name,name` ``.

<!-- DESCRIBES-TABLE-START -->

| component | describer | why |
|---|---|---|
| `crates/codelaw-rules` | SR-RULE-PACKS | pack layout, resolution, presets and precedence (decisions 3–5, 10–14) are the loader's |
| `crates/codelaw-cli/src/init.rs` | SR-RULE-PACKS | decision 6 is what `codelaw init` does |
| `crates/codelaw-engine/src/suppress.rs` | SR-LINTER | decision 6: the suppression syntax and what it covers |

<!-- DESCRIBES-TABLE-END -->
