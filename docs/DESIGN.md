# codelaw — Design Proposal

> A rules-as-markdown linter and formatter for JS / TS / React / Stencil / Python / JSON / YAML / TOML / Markdown / CSS / HTML, with deterministic auto-fix. It runs on its own with no AI involved, and AI coding agents can also query it *before* writing code and are forced through it *after* writing.

Status: proposal · Date: 2026-10-18 · Owner: Arpit

---

## 1. TL;DR

- **One Rust core, two thin wrappers.** Ship a native binary to **npm** and **PyPI**, the way Ruff, Biome, oxlint and ast-grep do. Works in Python-only, Node-only and mixed repos with identical results, and needs neither runtime to be installed for the other.
- **Runs with or without AI.** The engine contains no model. The terminal, pre-commit, CI and the editor all work with no agent present. Agents are just one more way of calling the same engine, and they get the same answers.
- **JS, TS and Python, combined or on their own.** A Python-only repo, a JS/TS-only repo and a mixed repo all run the same binary. Each language can be switched on or off independently. See [L3](../records/0005_LAW-3-languages.md).
- **Don't write parsers.** Build on **tree-sitter + `ast-grep-core`**. One CST model covers JS, JSX, TS, TSX (React and Stencil), Python, JSON, YAML, Markdown, CSS and HTML. Add the tree-sitter TOML grammar yourself, since ast-grep doesn't ship TOML.
- **A rule is one Markdown file.** Frontmatter holds the metadata, a fenced block holds the matcher and fix, and `Do` / `Don't` code blocks hold the examples. That one file is read by humans, read by agents and run by the engine. The examples double as the rule's test suite, so the docs can't drift from the behaviour.
- **Fixes are declarative edit primitives.** Each fix is a list of LSP-style text edits on the CST: `replace`, `remove`, `insert`, `move-to-end`, `collect-into-block`, and so on. "Move exports to the end" is a built-in primitive, not custom code. For anything else there's an escape hatch: an external fixer, in any language, speaking JSON over stdin/stdout.
- **Agents get two channels:**
  1. **Guidance (before writing):** an MCP server (`codelaw mcp`), a CLI (`codelaw rules --for <file>`), and generated instruction files (AGENTS.md, CLAUDE.md, Kiro steering, Copilot instructions).
  2. **Enforcement (after writing):** hooks in Claude Code, Copilot, Codex and Kiro that run `codelaw fix` on every file the agent touches and send any remaining violations back to the agent.
- **Principle:** guidance is probabilistic and enforcement is deterministic. The agent querying rules first cuts churn; the hooks and auto-fix are what guarantee compliance.

---

## Laws and records

The hard requirements are stated as **laws** in [`records/`](../records/README.md). This document links to them and doesn't restate them.

| Law | One line |
|---|---|
| [L2](../records/0004_LAW-2-foss-dependencies.md) | Every dependency is free, open source and permissive (from the work-system setup) |
| [L3](../records/0005_LAW-3-languages.md) | JS, TS, Python, JSON, YAML, TOML, Markdown, CSS and HTML, each alone or in any combination |
| [L4](../records/0006_LAW-4-every-rule-has-a-fix.md) | Every rule has a fix: `auto` wherever safe, otherwise `suggest` (diff) or `manual` (written steps, carried by a skill) |
| [L5](../records/0007_LAW-5-no-model-in-the-engine.md) | No model in the engine; works the same with or without an agent |
| [L6](../records/0008_LAW-6-python-and-node-packages.md) | One release: `codelaw` on PyPI and npm, each a complete engine, identical output; rules as seventeen packs, fifteen on npm and ten on PyPI; npm sub-packages only as `@codelaw/*` |
| [L7](../records/0009_LAW-7-linter-and-formatter.md) | Always both a linter and a formatter, for every L3 language |
| [L8](../records/0010_LAW-8-performance-budgets.md) | Performance budgets: set from measurement, never exceeded, never loosened |

Feature decisions:
- [SR-RULE-PACKS](../records/0101_rule-packs.md): seventeen first-party rule packs (fifteen `@codelaw/rules-*` on npm, ten `codelaw-rules-*` on PyPI), team packs and local rules.
- [SR-AGENT-SURFACES](../records/0102_agent-surfaces.md): skills, steering, instructions, hooks.
- [SR-PROJECT-GRAPH](../records/0103_project-graph.md): cross-file rules over a whole-repo symbol graph.
- [SR-BASELINE](../records/0104_baseline-and-deprecations.md): grandfathering with deadlines, touch policy and deprecations.
- [SR-LINTER](../records/0106_linter.md): the lint engine: nine languages, file/project/adapter rules, embedded code in HTML and Markdown.
- [SR-FORMATTER](../records/0105_formatter.md): codelaw also formats (Prettier-style JS/TS/JSON, Black-style Python), with embedded engines.
- [SR-PACKAGES](../records/0107_packages.md): every crate and published package, and what goes in each.
- [SR-ARCHITECTURE](../records/0108_architecture.md): layers, parse-once data flow, concurrency, process model.
- [SR-CACHE](../records/0109_cache.md): content-addressed caches that never change output.
- [SR-CONFIG](../records/0110_config.md): the configuration file: homes, sections, precedence, validation.
- [SR-CONSUMER-LAYOUT](../records/0111_consumer-layout.md): what codelaw adds to a consumer repo.
- [SR-RULE-FORMAT](../records/0112_rule-format.md): the rule-file template.
- [SR-RULE-AUTHORING](../records/0113_rule-authoring.md): writing rules by hand or with an agent; debugging and testing.
- [SR-EDITOR](../records/0114_editor.md): the LSP server and the VS Code extension.
- [SR-CLI-MCP](../records/0115_cli-and-mcp.md): the CLI and the MCP server.

---

## 2. Research: existing linters

### 2.1 Landscape

| Tool | Langs | Custom rules | Autofix | Agent surface | Take-away for codelaw |
|---|---|---|---|---|---|
| **ESLint** (+ typescript-eslint) | JS/TS/JSX | JS plugins, the richest ecosystem | `fix` + `suggestions` | Official MCP server (`@eslint/mcp`) for Copilot, Cursor, Windsurf | Gold standard for the rule API (`meta.fixable`, suggestions, RuleTester). Slow. Node only. |
| **oxlint** (Oxc) | JS/TS/JSX | **JS plugins (alpha, Mar 2026), ESLint-compatible API, with fixes** | Yes | LSP | Proves a Rust core can host ESLint-style plugins at large speedups (4.8x on the Node.js repo, up to 100x reported). Custom *type-aware* rules aren't supported. |
| **Biome** | JS/TS/JSON/CSS/GraphQL; YAML coming in 2026 | **GritQL** plugins (CST queries, custom diagnostics) | Built-in rules only | LSP | Rust CST approach. No Python and no TOML. Markdown not started. |
| **Ruff** | Python | **None**: "does not yet support third-party plugins" | Safe vs unsafe fixes | LSP (`ruff server`) | Copy its **safe/unsafe fix split** and its distribution model (Rust binary on PyPI). Can't host your rules. |
| **Fixit 2** (Meta) | Python | Python rules on LibCST, with in-repo local rules | Yes, preserves formatting | – | Good model for "rules live in the repo". Python only, slower. |
| **ast-grep** | 25+ languages incl. TS, TSX, Python, JSON, YAML (**no TOML built in**) | YAML rules: pattern + relational (`inside`, `has`, `precedes`, `follows`) | `fix` templates + `transform` | **MCP server** (`find_code`, `find_code_by_rule`, `test_match_code_rule`, `dump_syntax_tree`) | **Closest fit for the engine.** Rust, tree-sitter, on npm and PyPI already. Its `fix` only rewrites the matched node, so it can't *move* code. That gap is codelaw's to fill. |
| **Semgrep** | 30+ | YAML rules | `fix` / `fix-regex` | MCP server | Security-focused and heavy (OCaml + Python). Same single-node fix limit. |
| **yamllint / taplo / tombi / markdownlint-cli2** | YAML / TOML / MD | Mostly config, not custom logic | Formatting only | – | taplo, tombi and markdownlint-cli2 can be **adapters**. yamllint is GPL-3.0, so [L2](../records/0004_LAW-2-foss-dependencies.md) rules it out; YAML checks are native. |
| **@stencil/eslint-plugin** | Stencil TSX | ESLint rules | Some | – | Use as an adapter for the Stencil-specific best-practice rules. |

### 2.2 What to steal

- **ESLint:** rule metadata (`fixable`, `docs.url`); *suggestions* (unsafe fixes the agent applies only after reviewing them); multi-pass fixing until a fixpoint, capped at 10 passes.
- **Ruff:** safe vs unsafe fixes, `--fix` vs `--unsafe-fixes`, per-glob overrides, one native binary on PyPI.
- **ast-grep:** pattern + relational rule language, rule test files, an MCP tool shaped for agents.
- **Fixit:** local, in-repo rules as first-class citizens.
- **oxlint:** an ESLint-compatible plugin API on a Rust core. This is the long-term escape hatch for JS-authored rules.

### 2.3 The gap no existing tool fills

1. **One rule format across TS, Python and config files.** Today you'd need ESLint plugins + Fixit rules + yamllint config + taplo config.
2. **Rules are documents for agents**, not just executables.
3. **Fixes that restructure** (move, hoist, group), not just replace one node.
4. **First-class agent integration**: query before writing, enforce after writing, across Claude Code, Copilot, Codex and Kiro.

---

## 3. Architecture

> **The design of record is [SR-ARCHITECTURE](../records/0108_architecture.md).** This section is the original sketch; where they differ, the record wins.

```
                 ┌──────────────── .codelaw/rules/*.md ────────────────┐
                 │  frontmatter + matcher/fix block + Do/Don't examples │
                 └───────────────┬──────────────────────────────────────┘
                                 │ parse + validate (rule schema)
┌────────────────────────────────▼─────────────────────────────────────────┐
│                         codelaw-core  (Rust)                             │
│  Rule loader → Matcher (ast-grep-core over tree-sitter CST)              │
│             → Diagnostics → Fix planner (edit primitives)                │
│             → Edit applier (non-overlapping, multi-pass to fixpoint)     │
│             → Verifier (re-parse: no new syntax errors, idempotent)      │
│  Structured-file engine: JSON/YAML/TOML via CST + path assertions/schema │
│  Adapter host: oxlint/eslint, ruff, taplo, markdownlint → normalise            │
│  Cache: content-hash per file × rule-set hash                            │
└──────┬──────────────┬───────────────┬──────────────┬──────────────┬──────┘
       │ CLI          │ MCP (stdio)   │ LSP          │ hook mode    │ SARIF/JSON
       ▼              ▼               ▼              ▼              ▼
   humans, CI    Claude Code/Cowork, IDEs (VS Code,  agent hooks    GitHub code
   pre-commit    Copilot, Kiro, Codex Kiro)          (post-edit)    scanning
       │
  ┌────┴──────────────────────┐
  │ npm:  codelaw  (+ @codelaw/<platform> binaries via optionalDependencies)
  │ PyPI: codelaw  (platform wheels built with maturin, binary only)
  │ npm:  @codelaw/rules-{js,html,css,python,data}   rule packs (SR-RULE-PACKS)
  │ PyPI: codelaw-rules-{js,html,css,python,data}
  └───────────────────────────┘
```

### 3.1 Why a Rust core (option comparison)

| Option | Python-only repo | Node-only repo | Same results everywhere | Effort | Verdict |
|---|---|---|---|---|---|
| **Rust core, native binaries on npm + PyPI** | ✅ | ✅ | ✅ one engine | Medium | **Pick this.** Proven by Ruff, uv, Biome, oxlint, ast-grep. |
| TS core + Python wrapper that calls Node | ❌ needs Node | ✅ | ✅ | Low | Fails the Python-only case. |
| Python core + Node wrapper | ✅ | ❌ needs Python | ✅ | Low | Fails Kiro and VS Code hosts without Python (same pain as Fux search from Node). |
| Two native cores (TS + Python) | ✅ | ✅ | ❌ they drift | High ×2 | Reject. |
| Rust → WASM, run in both | ✅ (needs wasmtime dependency) | ✅ | ✅ | Medium | Keep as a fallback for exotic platforms. |

- The wrappers have **zero runtime dependencies**. They just pick and run the right binary, in line with your zero-dependency principle.
- The one dependency we take on purpose is **tree-sitter grammars**. They're deterministic and auditable, and writing parsers ourselves isn't worth it.

### 3.2 Rule format: Markdown as the single source of truth

`.codelaw/rules/ts-exports-at-end.md`

````markdown
---
id: ts/exports-at-end
title: Exports go at the end of the file
languages: [javascript, jsx, typescript, tsx]
files: ["src/**/*.{js,jsx,ts,tsx}"]
exclude: ["**/*.stories.tsx"]
severity: error            # error | warn | info
fix: auto                  # auto | suggest | manual  (L4)
tags: [style, modules]
---

All exported symbols are declared without `export` and re-exported in a single
`export { … }` block (plus `export type { … }`) at the bottom of the file.

**Why:** readers and agents see the module's public API in one place.

```codelaw
match:
  kind: export_statement
  has: { field: declaration, kind: [function_declaration, class_declaration, lexical_declaration, interface_declaration, type_alias_declaration, enum_declaration] }
fix:
  - strip-export-keyword: $DECL
  - collect-into-block:
      block: "export { $$$NAMES };"
      type_block: "export type { $$$TYPE_NAMES };"
      position: end-of-file
```

## Do
```ts
function add(a: number, b: number) { return a + b; }
interface Point { x: number }
export { add };
export type { Point };
```

## Don't
```ts
export function add(a: number, b: number) { return a + b; }
export interface Point { x: number }
```
````

- **Frontmatter:** identity, scoping, severity, fix safety.
- **The `codelaw` block:** the machine part. It uses ast-grep rule syntax for matching, plus codelaw's fix primitives.
- **Prose, `Do` and `Don't`:** what agents read.
- **`codelaw test`:** runs every rule against its own examples. `Don't` must trigger the rule; `Do` must pass and must be a fixpoint. Fixing `Don't` should produce something that also passes. This is ESLint's RuleTester, written in the doc itself.

### 3.3 Fix model

- A fix is an ordered list of **text edits on the CST**: `{range, newText}`, the same shape as an LSP `TextEdit`. Because only the edited ranges change, comments and formatting elsewhere survive.
- **Built-in primitives** (deterministic and unit-tested):
  - `replace`
  - `remove`
  - `insert-before` / `insert-after`
  - `strip-export-keyword`
  - `move-to` (`end-of-file`, `start-of-file`, `after-imports`)
  - `collect-into-block`
  - `sort-keys` (JSON/YAML/TOML)
  - `ensure-key`
  - `rename-symbol-local`
- **Apply loop:**
  1. Gather fixes from all rules.
  2. Drop overlapping edits; they're picked up on the next pass.
  3. Apply the rest.
  4. Re-parse.
  5. Repeat until a fixpoint, or stop after 10 passes.
- **Verifier:** if a fix introduces a parse error, or a second run changes the file again, roll back that rule's fix and report it as a non-fixable diagnostic. The tool must never corrupt a file.
- **Fix classes** ([L4](../records/0006_LAW-4-every-rule-has-a-fix.md)): `auto` fixes are applied by `codelaw fix`; `suggest` fixes are applied only with `--unsafe` or offered to agents as a diff; `manual` rules carry written steps.
- **Escape hatch:** `fix: { external: "node scripts/my-fixer.mjs" }`. The fixer gets JSON (file, matches, captures) on stdin and returns `TextEdit[]` on stdout. It can be written in any language, and the core still verifies the result.
- **Formatting:** codelaw formats too ([SR-FORMATTER](../records/0105_formatter.md)): `codelaw fix` runs lint fixes, then the built-in formatter, then verifies.

### 3.4 Worked example: exports-at-end fix

| Input pattern | Output |
|---|---|
| `export function f() {}` | `function f() {}` … `export { f };` |
| `export const a = 1, b = 2;` | `const a = 1, b = 2;` … `export { a, b };` |
| `export interface P {}` / `export type T = …` | declaration kept; `export type { P, T };` |
| `export default function () {}` (anonymous) | **suggest**: needs a generated name, so it is a suggestion only |
| `export default foo;` / `export * from './x'` / `export { a } from './x'` | left alone; re-exports are already "export statements", configurable |
| An existing `export { x }` mid-file | merged into the end block |
| JSDoc above `export function` | stays attached to the declaration (the edit only strips the `export ` token) |
| TS `export =` / `declare module` / namespaces | skipped (rule `exclude` kinds) |

> ⚠️ **Stencil:** the Stencil compiler is strict about how `@Component` classes are exported. Check whether `class X {}; export { X }` compiles. Until that's confirmed, ship an override that excludes files with `@Component` from this rule.

### 3.5 Structured files (JSON / YAML / TOML)

- Parse to a CST (tree-sitter: `json`, `yaml`, `toml`) so fixes keep comments in YAML and TOML.
- Rule matchers for these files are **path assertions** plus optional **JSON Schema**:

```codelaw
target: package.json
assert:
  - path: $.engines.node
    exists: true
  - path: $.scripts
    keys-sorted: true
fix:
  - sort-keys: $.scripts
  - ensure-key: { path: $.engines.node, value: ">=20" }
```

- The same model works for `pyproject.toml` (`$.project.requires-python`), CI YAML, `tsconfig.json`, and so on.

### 3.6 Adapters (don't rebuild the world)

- codelaw runs these if they're installed and **normalises** their output into one diagnostic stream: oxlint or ESLint, Ruff, `@stencil/eslint-plugin`, taplo or tombi, markdownlint-cli2. Every adapter must pass [L2](../records/0004_LAW-2-foss-dependencies.md) (permissive licence only).
- Agents then query **one tool** for everything.
- codelaw-native rules are for *your* conventions. Standard hygiene stays with the specialist tools.
- Type-aware rules (for example, floating promises) need a type checker; tree-sitter has none. For TypeScript, codelaw asks the project's own TypeScript ≥ 7.1 through its Node helper ([SR-TYPES](../records/0117_types.md); not built yet, W-105), and oxlint's type-aware mode fills the gap until then. Python's stay with pyright/mypy.

### 3.7 Configuration

`codelaw.toml`, or `[tool.codelaw]` in `pyproject.toml`, or `"codelaw"` in `package.json`:

```toml
rules_dir = ".codelaw/rules"
languages = ["javascript", "jsx", "typescript", "tsx", "python"]   # omit = all detected
extends = ["codelaw:js/recommended", "npm:@acme/codelaw-rules"]   # first-party and team rule packs
adapters = ["oxlint", "ruff", "markdownlint"]
# formatting is built in — see [format] in SR-FORMATTER; no external tools

[scope.components]
files = ["src/components/**/*.tsx"]       # Stencil components
rules = { "ts/exports-at-end" = "off" }
```

---

## 4. Agent integration

### 4.1 Before writing: guidance

**MCP server** (`codelaw mcp`, stdio, same binary). One config works in Claude Code, Cowork (as a plugin), Copilot agent mode, Kiro (local or remote MCP) and Codex.

| Tool | Input | Returns | Why it exists |
|---|---|---|---|
| `rules_for_path` | file path or glob | compact rule cards (id, one-line rule, Do example) after overrides are resolved | the agent calls this **before** creating or editing a file |
| `explain_rule` | rule id | full markdown | handles "why did this fail?" |
| `check_snippet` | code + language + intended path | diagnostics + fixed code | validate code **before** writing it to disk |
| `check_paths` | paths | diagnostics (JSON) | review existing code |
| `fix_paths` | paths, `unsafe?` | applied diff + remaining diagnostics | let the agent self-heal |
| `list_rules` | filter by tag or language | ids + titles | discovery |

- Output is **token-budgeted**: rule cards are under about 60 tokens each, details come on demand, and `rules_for_path` caches by rule-set hash.

**Generated instruction files** (`codelaw sync-agents`). These cover agents that don't call MCP proactively, and are regenerated from the rules so they never drift:

| Agent | File written |
|---|---|
| Codex, Kiro, generic | `AGENTS.md` (codelaw section between markers) |
| Claude Code / Cowork | `CLAUDE.md` section + a `codelaw` **skill** ("before editing TS/Python, call `rules_for_path`") |
| Kiro | `.kiro/steering/codelaw.md` (with `fileMatch` inclusion) |
| GitHub Copilot | `.github/instructions/codelaw-<lang>.instructions.md` with `applyTo` globs |

- These files are short (rule one-liners + "run `codelaw check`"). They point to the MCP or CLI for details rather than inlining every rule.

### 4.2 After writing: enforcement (the deterministic part)

`codelaw init --agents claude,copilot,codex,kiro` installs hooks that run **hook mode**:

```
codelaw hook --fix --changed <file(s) from hook payload> --format agent
```

- Auto-applies **`auto`** fixes silently, so the agent's file is already correct.
- If violations remain, it prints a compact report (rule id, line, message, Do example) and **exits non-zero**, so the agent sees it and corrects itself.
- Target latency: **under 100 ms per file**. That means one file, a cached rule set and no adapter runs; adapters run only in `check` or CI.

| Agent | Hook point |
|---|---|
| Claude Code / Cowork | `PostToolUse` matcher `Edit\|Write\|MultiEdit` in `.claude/settings.json`. Exit code 2 sends stderr back to Claude. |
| Codex CLI | `PostToolUse` hook (stable since v0.124). Can block and give feedback. |
| GitHub Copilot (CLI / cloud agent) | `.github/hooks/*.json` post-tool hook |
| Kiro | agent hook on file save or post-tool → runs `codelaw hook` |
| Everyone else | pre-commit hook + CI (`codelaw check --format sarif`) |

### 4.3 The flow for "write me a function"

1. The agent is about to edit `src/utils/math.ts`.
2. The skill or steering tells it to call `rules_for_path("src/utils/math.ts")`. It gets back "exports at end; no default exports; …".
3. It writes `function add() {}` mid-file and adds `add` to the bottom `export { … }` block.
4. The `PostToolUse` hook runs `codelaw hook --fix`. It's clean, so nothing happens.
5. If the agent forgot and wrote `export function add`, the hook **auto-moves it** and tells the agent: "ts/exports-at-end auto-fixed (moved `add` into the end export block)."
6. Result: compliance doesn't depend on the agent remembering.

---

## 5. CLI surface

```
codelaw check [paths]   [--lang js,ts,python] [--format text|json|sarif|agent|github|gitlab|junit|checkstyle]
codelaw fix   [paths]   [--unsafe] [--dry-run → unified diff]
codelaw rules [--for <path>] [--format md|json]
codelaw explain <rule-id>
codelaw new-rule <id> --lang ts         # scaffolds the markdown template
codelaw test [rule-id]                  # runs Do/Don't examples
codelaw mcp                             # stdio MCP server
codelaw lsp                             # language server
codelaw hook --changed <files>          # agent-hook mode
codelaw init --agents claude,copilot,codex,kiro
codelaw sync-agents                     # regenerate AGENTS.md / steering / instructions
```

Exit codes: `0` clean · `1` violations · `2` config or rule error.

---

## 6. Phased plan

| Phase | Scope | Exit criteria |
|---|---|---|
| **P0 — Engine spike (1–2 wks)** | Rust core on `ast-grep-core`. Markdown rule loader. `check`, `fix`, `test`. JS, JSX, TS, TSX and Python, each runnable alone. The no-AI CLI path ships first. Primitives: `replace`, `remove`, `strip-export-keyword`, `collect-into-block`. 3 rules incl. `ts/exports-at-end`. | All Do/Don't examples pass. Fix is idempotent on a real repo. |
| **P1 — Agent layer** | `codelaw mcp`, `hook` mode, `init --agents`, `sync-agents`. Claude Code skill. | Claude Code writes a function in a fresh file and the export lands at the end, with or without a hook-triggered fix. |
| **P2 — Configs + adapters** | JSON, YAML and TOML CST + path assertions. Adapters for oxlint, Ruff, taplo, markdownlint. SARIF output. | One `codelaw check` covers a mixed repo. |
| **P3 — Distribution + IDE** | npm (optionalDependencies per platform) + PyPI (maturin wheels). `codelaw lsp`. Caching. Shareable rule packs. | `npx codelaw` and `uvx codelaw` give byte-identical output. |
| **P4 — Plugins** | External fixer protocol hardening. Optional ESLint-compatible JS rule API (oxlint-style). | Your team ships a rule pack without touching Rust. |

---

## 7. Risks and mitigations

| Risk | Mitigation |
|---|---|
| A fix corrupts code (moves comments, breaks semantics) | Re-parse + idempotency verifier, rollback per rule, auto/suggest/manual classes, `--dry-run` diffs, golden tests from Do/Don't. |
| Rule conflicts with frameworks (Stencil exports, Next.js `export default` pages, React Fast Refresh) | Per-glob overrides. Ship presets (`preset:stencil`, `preset:next`) that pre-disable conflicting rules. |
| Agents ignore guidance | Don't depend on it. Hooks + auto-fix are the guarantee; guidance only reduces churn. |
| Hook latency annoys users | Single-file mode, cached rule set, no adapters in hook mode, 100 ms budget tested in CI. |
| Two rules fight (A's fix triggers B) | Multi-pass with a cap of 10. Oscillation detection reports both rule ids. `codelaw test --all` checks the rule set as a whole. |
| tree-sitter has no types | State it plainly. TypeScript: ask the project's own TypeScript ([SR-TYPES](../records/0117_types.md)). Python: delegate to adapters. |
| Cross-platform binaries are hard to ship | Reuse ast-grep's and Ruff's release pipelines (cargo-dist / maturin / napi-style optionalDependencies). |

---

## 8. Open questions

1. **Stencil:** does Stencil accept `class C {}; export { C };` for `@Component` classes? This decides the preset.
2. **Rule authoring for non-Rust users:** is ast-grep YAML + primitives enough for your team, or is a JS rule API needed earlier than P4?
3. **Formatter:** should `fix` always chain the project formatter, or leave formatting to the user?
4. **Fux:** should codelaw rules also be indexed by Fux so `fux ask "how do we export?"` returns them? It's natural (rules are markdown) and keeps "rules bound to code" on the Fux roadmap.
5. **Naming:** keep `codelaw` (repo name) as the package name on npm and PyPI? Check availability on both.

---

## Sources

- [Oxlint JS plugins alpha (Mar 2026)](https://oxc.rs/blog/2026-03-11-oxlint-js-plugins-alpha)
- [Biome roadmap 2026](https://biomejs.dev/blog/roadmap-2026/)
- [ESLint MCP server docs](https://eslint.org/docs/head/use/mcp)
- [Ruff FAQ: no third-party plugins](https://github.com/astral-sh/ruff/blob/main/docs/faq.md)
- [Fixit 2 (Meta)](https://engineering.fb.com/2023/08/07/developer-tools/fixit-2-linter-meta/)
- [ast-grep built-in languages](https://docs.rs/ast-grep-language)
- [ast-grep MCP server](https://mcpservers.org/servers/ast-grep/ast-grep-mcp)
- [Semgrep MCP](https://pypi.org/p/semgrep-mcp)
- [Claude Code hooks](https://docs.anthropic.com/en/docs/claude-code/hooks)
- [Codex CLI hooks stable (v0.124)](https://codex.danielvaughan.com/2026/04/23/codex-cli-hooks-graduate-stable-v0124-mcp-observation-inline-config/)
- [GitHub Copilot hooks](https://docs.github.com/en/copilot/how-tos/copilot-on-github/customize-copilot/customize-cloud-agent/use-hooks)
- [Kiro: remote MCP, global steering, hooks](https://kiro.dev/changelog/remote-mcp-and-global-steering)
- [Stencil ESLint plugin](https://github.com/stenciljs/eslint-plugin)
