---
type: Standing Record
kind: component
name: SR-RULE-FORMAT
title: "SR-RULE-FORMAT (0112) — the rule-file template: one Markdown file, four parts, a fixed frontmatter schema"
description: "Defines the exact template of a codelaw rule file — frontmatter fields, the codelaw block's three forms, the body sections and their order, the examples, and the .cases.md sidecar — and how codelaw validates its own rule files."
status: accepted
date: 2026-10-18
feature: "the rule file"
owns:
  - crates/codelaw-rules
laws: [L0, L3, L4, L5]
timestamp: 2026-10-19T14:51:23Z
content_sha: ""
---

# SR-RULE-FORMAT — the rule-file template: one Markdown file, four parts, a fixed frontmatter schema

## §1 — For humans

**The decision in one line.** A rule is **one Markdown file**, `<namespace>/<name>.md`, with **four parts in fixed order**: frontmatter, prose, a `codelaw` block, and examples. Edge cases go in a `<name>.cases.md` sidecar. codelaw lints its own rule files against this template.

````markdown
---
id: <namespace>/<name>                 # required; matches the path
title: <one line>                      # required
languages: [typescript, tsx]           # required; L3 names
scope: file                            # file | project        (default file)
files: ["src/**/*.{ts,tsx}"]           # optional glob filter
exclude: ["**/*.d.ts"]                 # optional
severity: error                        # error | warn | info   (default error)
fix: auto                              # required: auto | suggest | manual  (L4)
fix_reason: "<why not auto>"           # required unless fix: auto
fix_skill: codelaw-fix                 # optional; the skill that carries a manual fix (L4 decision 10)
stability: stable                      # stable | preview      (default preview)
host: html                             # optional: only inside regions embedded in this host
invariant: exports-unchanged           # optional; checked by codelaw test
options: { internal: { type: "string[]", default: [] } }   # optional, typed
ported_from: [eslint/no-dupe-keys]     # optional; the source rules this one reimplements
tags: [modules]
since: 0.1.0
---

One paragraph: what the rule requires.

**Why:** the reason, in one or two sentences.

```codelaw
match: …            # scope: file  → an ast-grep rule
# graph: …          # scope: project → a graph query (SR-PROJECT-GRAPH)
# target/assert: …  # structured files → path assertions
# native: <id>      # first-party packs only → a Rust implementation in the engine
message: "…$NAME…"
fix: [ … ]          # edit primitives, or { external: "…" }
```

## How to fix
1. …                # required when fix: manual

## Examples
### TypeScript
#### Don't
```ts
…
```
#### Do
```ts
…
```
````

```mermaid
flowchart TB
    FM["frontmatter: identity, scope, fix class"] --> PR["prose + Why"]
    PR --> CB["codelaw block: match | graph | assert"]
    CB --> HF["How to fix (manual)"]
    HF --> EX["Examples: Don't / Do per language"]
    EX -.-> CS["sidecar: name.cases.md"]
```

<details>
<summary><b>ASCII twin</b></summary>

```text
   frontmatter --> prose + Why --> codelaw block --> How to fix (manual) --> Examples (Don't/Do per language)
                                                                              :
                                                                   sidecar  name.cases.md
```

</details>

---

## §2 — For agents

### Context

The decider asked for a record of the rule-file template. The format so far existed only as examples in chat, the design doc and SR-RULE-PACKS.

### Decision

1. **Path and id.** The file is `<rules_dir>/<namespace>/<name>.md`, and its `id` must equal `<namespace>/<name>`. First-party namespaces (`js`, `ts`, `react`, `node`, `stencil`, `py`, `json`, `yaml`, `toml`, `md`, `data`, `css`, `html`, `graph`, `format`, `codelaw`) are reserved for the first-party rule packs and the engine ([SR-RULE-PACKS](0101_rule-packs.md)).
2. **Frontmatter schema** is §1's template, every field typed.
   - **Required:** `id`, `title`, `languages`, `fix`, plus `fix_reason` unless `fix: auto` ([L4](0006_LAW-4-every-rule-has-a-fix.md)).
   - **`ported_from:`** (optional) lists `<tool>/<rule>` ids, for example `[eslint/no-dupe-keys, ruff/F601]`: the source rules this one reimplements ([SR-RULE-PACKS](0101_rule-packs.md) decision 7). `codelaw-migrate` maps existing configs through it ([SR-AGENT-SURFACES](0102_agent-surfaces.md)).
   - **Unknown fields are a load error.**
3. **The `codelaw` block takes exactly one of four forms:**
   - `match:` (an ast-grep rule with relational clauses and the semantic-model predicates of [SR-ARCHITECTURE](0108_architecture.md) decision 15a; `scope: file`);
   - `graph:` ([SR-PROJECT-GRAPH](0103_project-graph.md) decision 6; `scope: project`);
   - `target:` + `assert:` (path assertions on JSON, YAML or TOML);
   - `native: <id>` (a Rust implementation in the engine, [SR-ARCHITECTURE](0108_architecture.md) decision 15b; accepted only in first-party packs and in an ejected copy of one of their rules).

   Then `message:` (with `$CAPTURE` interpolation) and `fix:` (edit primitives, conditional `when`/`else` branches, or `{ external: "<command>" }`, whose stdin/stdout contract is [SR-ARCHITECTURE](0108_architecture.md) decision 18).
3a. **What goes with `match:`.** Only beside `match:`: ast-grep's own `constraints:` and `utils:`, and `where:`, the semantic-model predicates.
   - **`where:` maps a capture to a predicate.** The key is a capture (`$NAME`) or the matched node itself (`$$MATCH`); a key `match:` doesn't capture is a load error, and so is a list capture (`$$$NAME`), since a predicate is about one node. Only JavaScript, TypeScript and Python have a semantic model, so `where:` in a rule for any other language is a load error rather than a rule that silently matches nothing.
   - **The predicates:** `unused`; `undeclared` (optionally `{ globals: [<name>…], tables: [<data table>…] }`); `shadowed` (optionally `{ builtins: true }`); `reassigned`; `imported-from` (a module, a list of modules, or `{ module, name }`); `write` (assigned to here); `declaration` (declares a name); `redeclared` (declares a name its scope already declared); `binding` (names a symbol declared as one of the given kinds: `var`, `let`, `const`, `function`, `class`, `param`, `import`, `catch-param`, `interface`, `type-alias`, `enum`, `namespace`, `type-param`, `variable`); `builtin` (one of the language's own globals, undeclared in the file).
   - **Combining them:** a list means all of them; `not`, `all` and `any` take predicates.
   - **Node kinds across languages:** a rule object whose `kind` a grammar doesn't have matches nothing in that grammar, so one rule can span JavaScript and TypeScript. A `kind` that none of the rule's grammars has is a load error.
3b. **Interpolation.** A message or template replaces `$NAME` with the capture's text, `$$$NAME` with the source a multi-capture spans, and `$$MATCH` with the matched text.
3c. **The edit primitives.** `fix:` is a template that replaces the match, or a list of primitives, each acting on the match or on a capture (`target:`):
   - `replace`, `insert-before`, `insert-after`;
   - `remove`: takes a separating comma with the node, and the whole line when the node is alone on it;
   - `strip-export-keyword`;
   - `collect-into-block`: `block` with `$$$NAMES`, `type_block` with `$$$TYPE_NAMES`, and `position`; computed once over every match in the file, so each diagnostic carries the whole edit set;
   - `move-to`: `end-of-file`, `start-of-file` or `after-imports`;
   - `rename-symbol-local`: renames a symbol and its references in the file, and is refused for an exported symbol;
   - `sort-keys` and `ensure-key` (for `target:` rules) and `when`/`else` branches.
3d. **YAML 1.2.** The frontmatter and the `codelaw` block are read as YAML 1.2, as ast-grep reads its rules: `on`, `n` and `yes` are strings. A key written twice in one mapping is a load error.
4. **Body sections, in this order:**
   1. One summary paragraph.
   2. `**Why:**`
   3. The `codelaw` block.
   4. `## How to fix` (required for `manual`; optional otherwise).
   5. `## Examples`.

   Any other section is allowed after `## Examples` and ignored by the engine.
5. **Examples.** Under `## Examples` there's one `### <Language>` per declared language, each with at least one `#### Don't` and one `#### Do`, and an optional `#### Fixed` when the fixed output differs from the Do. Multi-file examples use a `files` fence (`// path` sections; [SR-PROJECT-GRAPH](0103_project-graph.md) decision 6).
   - **A fence's info string is `<tag> [path=<file>]`.** With `path=`, the example is checked as that file, so the rule's `files:` and `exclude:` apply. Without it the file is `example.<ext>` and they don't.
6. **The sidecar `<name>.cases.md`** holds edge cases. Each `### case: <title>` has an input fence, an `expect` fence (`line:col rule-id` lines, or `(none)`) and an optional `fixed` fence ([SR-RULE-AUTHORING](0113_rule-authoring.md) decision 4).
7. **Self-check.** `codelaw rules lint` validates every rule file against this template (the schema, the section order, an example for each language, the id matching the path). It runs inside `codelaw test` and on every first-party pack rule in CI.
8. **Scaffold.** `codelaw new-rule <id> --lang <list> [--scope project] [--fix manual]` writes this template with every required field filled and empty examples. The id's namespace and name are each one plain path segment (letters, digits, `-`, `_`, `.`, not starting with `.`), so the file always lands inside the rules folder.
9. **Determinism ([L5](0007_LAW-5-no-model-in-the-engine.md)).** A rule file is data. A `native:` rule names engine code; it doesn't carry it. Nothing in a rule file executes, except an `external` fixer, which runs only under [SR-RULE-PACKS](0101_rule-packs.md) decision 12's rules.

### Consequences

- **Authors write more than a bare pattern;** the examples and steps are what make a rule readable by agents and testable.
- **A strict schema means template changes need a migration** (`codelaw rules migrate`) when fields are renamed.

### Alternatives considered

- **YAML-only rules (ast-grep style):** rejected. No place for prose, why, steps or examples that agents read.
- **Rules as code (ESLint style):** rejected. Not readable by agents, not portable across npm and PyPI, and not deterministic data.
- **Tests in a separate folder:** rejected for the core examples (they document the rule); kept for edge cases as the sidecar.

### Reference (required)

- [ast-grep rule configuration](https://ast-grep.github.io/reference/rule.html): the `match` grammar.
- [L4](0006_LAW-4-every-rule-has-a-fix.md): fix classes and `fix_skill`.
- [SR-RULE-PACKS](0101_rule-packs.md): where rule files come from.

### Veto condition

**Reopen this decision if:** a rule file loads that violates decision 2–5, or `codelaw new-rule` writes a file that `codelaw rules lint` rejects.

**How to check it:** `codelaw rules lint` over `rules/**` and a freshly scaffolded rule (`W-20`).

---

## References

*Every source this record cites, gathered in one place. An archived document is never listed here.*

**Records** — [SR-RULE-PACKS](0101_rule-packs.md) · [SR-PROJECT-GRAPH](0103_project-graph.md) · [SR-RULE-AUTHORING](0113_rule-authoring.md) · [SR-AGENT-SURFACES](0102_agent-surfaces.md) · [SR-ARCHITECTURE](0108_architecture.md)

**Project docs** — [`docs/DESIGN.md`](../docs/DESIGN.md)
