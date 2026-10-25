---
type: Standing Record
kind: component
name: SR-RULE-PACKS
title: "SR-RULE-PACKS (0101) — eleven first-party rule packs published apart from the engine, ten on npm and six on PyPI; team packs and local rules on the same footing"
description: "codelaw's own rules ship as eleven separately published packs — js, ts, react, node, stencil, html, css, python, data, tanstack-query, redux-toolkit — ten on npm (@codelaw/rules-*) and six on PyPI (codelaw-rules-*), released with the engine at one version; the engine carries no rules of its own beyond meta diagnostics; teams publish packs the same way, and project rules live in .codelaw/rules."
status: accepted
date: 2026-10-18
feature: "predefined rules and rule packs"
owns:
  - packs
laws: [L0, L2, L3, L4, L5, L6, L7]
timestamp: 2026-10-25T09:00:00Z
content_sha: ""
---


# SR-RULE-PACKS — eleven first-party rule packs published apart from the engine, ten on npm and six on PyPI; team packs and local rules on the same footing

## §1 — For humans

**The decision in one line.** The engine package, `codelaw`, carries **no rules**. codelaw's own rules ship as **eleven rule packs**. A pack is published on **npm as `@codelaw/rules-<name>`**, on **PyPI as `codelaw-rules-<name>`**, or on both, as §1's table says, in the same release and at the same version as the engine. `codelaw init` installs the packs for the languages it finds. Teams publish their own packs the same way, and project rules stay in `.codelaw/rules/`.

**Ruled by Arpit, 2026-10-19:**
- the packs are separately published, not just separate in the source tree;
- every pack is mirrored on both registries;
- a fifth pack, `rules-data`, covers JSON, YAML, TOML and Markdown;
- `init` installs the packs it detects a need for.

This record was rewritten to that ruling and then accepted ("go with the recommendations").

**Ruled by Arpit, 2026-10-24** (W-31), superseding "every pack is mirrored on both registries": *"The ones that are going to be used in npm and the ones that are going to be used in PyPI should only be published."*
- `rules-js` splits into `js`, `ts`, `react`, `node` and `stencil`;
- **npm** carries `js`, `ts`, `react`, `node`, `stencil`, `html`, `css` and `data` (eight);
- **PyPI** carries `js`, `react`, `html`, `css`, `python` and `data` (six);
- `rules-python` leaves npm, and `ts`, `node` and `stencil` are not on PyPI.

**Ruled by Arpit, 2026-10-24** (W-32): *"i want a redux tool kit and react query pack as well in npm"*; names and source ruled the same day at the docket, the query pack renamed `tanstack-query` on 2026-10-25.
- two npm-only library packs: `tanstack-query` (@tanstack/eslint-plugin-query's `recommended`, reimplemented) and `redux-toolkit` (codelaw's own rules, written from the Redux Style Guide's Priority A and B rules: there is no official Redux Toolkit ESLint plugin);
- **npm** carries ten packs; PyPI is unchanged.

| Pack | npm | PyPI | Languages | Presets |
|---|---|---|---|---|
| engine | `codelaw` | `codelaw` | none: this is the engine. It ships only the `codelaw/*` meta diagnostics, the formatter's `format/unformatted` and `format/failed`, and `graph/no-deprecated-usage` | — |
| `js` | `@codelaw/rules-js` | `codelaw-rules-js` | JS, JSX; its core rules also run on TS and TSX | `js/recommended`, `js/typescript`, `js/conventions`, `js/project` · *later:* `js/strict` |
| `ts` | `@codelaw/rules-ts` | — | TS, TSX | `ts/recommended`, `ts/conventions` |
| `react` | `@codelaw/rules-react` | `codelaw-rules-react` | JSX, TSX, and React in JS and TS | `react/recommended`, `react/conventions` |
| `node` | `@codelaw/rules-node` | — | JS, JSX, TS, TSX run on Node.js | `node/recommended` |
| `stencil` | `@codelaw/rules-stencil` | — | TS, TSX Stencil components | `stencil/recommended`, `stencil/conventions` |
| `html` | `@codelaw/rules-html` | `codelaw-rules-html` | HTML, incl. HTML embedded in Markdown and templates | `html/recommended`, `html/a11y`, `html/document`, `html/conventions` |
| `css` | `@codelaw/rules-css` | `codelaw-rules-css` | CSS | `css/recommended`, `css/standard`, `css/conventions`, `css/project` |
| `python` | — | `codelaw-rules-python` | Python | `py/recommended`, `py/conventions`, `py/project` · *later:* `py/strict` |
| `data` | `@codelaw/rules-data` | `codelaw-rules-data` | JSON, YAML, TOML, Markdown | `data/recommended`, `md/recommended`, `data/package-json`, `data/tsconfig`, `data/pyproject`, `data/github-actions`, `data/conventions` |
| `tanstack-query` | `@codelaw/rules-tanstack-query` | — | JS, JSX, TS, TSX using TanStack Query | `tanstack-query/recommended` |
| `redux-toolkit` | `@codelaw/rules-redux-toolkit` | — | JS, JSX, TS, TSX using Redux | `redux-toolkit/recommended` |

`FIRST_PARTY_PACKS` in `crates/codelaw-rules/src/pack.rs` is this table in code, and `scripts/package.py` reads it, so the engine and the release name the same packs on the same registries.

*Later* marks a preset the ruling names that no pack ships yet: `js/strict` and `py/strict` are planned as B-048 in [`work/BACKLOG.md`](../work/BACKLOG.md) and are not in `packs/` (checked 2026-10-20). Every other preset in the table is in its pack's `codelaw-pack.toml` and `presets/`. Each `<namespace>/project` preset is opt-in and turns on only that pack's `graph/*` rule, the one that reads the whole repository (W-26, 2026-10-24); `init` does not write it.

**What you install depends on your repo.** A Python-only repo installs `codelaw` and `codelaw-rules-python`, plus `codelaw-rules-data` if it wants `pyproject.toml`, YAML and Markdown checked, all from PyPI, and never touches npm. A React and TypeScript repo installs `codelaw`, `@codelaw/rules-js`, `@codelaw/rules-ts`, `@codelaw/rules-react` and, likely, `@codelaw/rules-data` from npm. **What each registry leaves out:** a repo that installs only from npm gets no Python rules, and one that installs only from PyPI gets no `ts/*`, `node/*`, `stencil/*`, `tanstack-query/*` or `redux-toolkit/*` rules (its TypeScript still gets the `js` rules that declare it, and `js/typescript`).

```mermaid
flowchart LR
    E["engine: codelaw<br/>(no rules)"] --> M["merged rule set"]
    P1["first-party packs<br/>@codelaw/rules-* · codelaw-rules-*"] --> M
    P2["team / third-party packs<br/>npm · PyPI · path"] --> M
    L[".codelaw/rules/ (local)"] --> M
```

<details>
<summary><b>ASCII twin</b></summary>

```text
   engine codelaw (no rules) ------------------+
   first-party packs (rules-js, -ts, -react,   |
     -node, -stencil, -html, -css, -python,    |
     -data, -tanstack-query, -redux-toolkit) --+--> merged rule set   (local wins, then packs in extends order)
   team / third-party packs (npm, PyPI, path) -+
   local .codelaw/rules/ ----------------------+
```

</details>

---

## §2 — For agents

### Context

The first version of this record (2026-10-18) embedded codelaw's predefined rules in the binary, because the decider had left the choice open. On 2026-10-19 he asked for the presets of ESLint, typescript-eslint, TSLint, Ruff, Prettier, stylelint and the HTML linters to be researched. He also asked for the rules to ship as separate packages, one per ecosystem, with the engine on its own. He then ruled on the shape (§1).

The research ([W-03](../archive/open/W-03-rule-packs.md)) found that porting the correctness core of those presets yields several hundred rules. Some of those rules need per-ecosystem data tables (CSS properties, HTML elements, JSON Schemas). Embedding all of that in the binary would make every user carry every ecosystem's rules and data.

### Decision

1. **The engine carries no rules except its own:**
   - the `codelaw/*` meta diagnostics, such as `codelaw/not-utf8`, `codelaw/internal-error` and `codelaw/no-rules` ([SR-ARCHITECTURE](0108_architecture.md));
   - `format/unformatted` and `format/failed`, the formatter's findings in `check` ([SR-FORMATTER](0105_formatter.md) decisions 1 and 4);
   - `graph/no-deprecated-usage`, which reads the consumer's deprecations registry and belongs to no language, and runs only when the project declares deprecations; and `codelaw/stale-baseline`, a baseline entry its file has outgrown ([SR-BASELINE](0104_baseline-and-deprecations.md) decisions 2 and 6); and the dated-exception rules `codelaw/expiring-exception`, `codelaw/expired-exception`, `codelaw/exception-without-expiry` and `codelaw/exception-author` (SR-BASELINE decision 9).

   Every other rule codelaw publishes lives in a pack.
2. **The eleven first-party packs, their registries and their presets** are §1's table. A library pack (`tanstack-query`, `redux-toolkit`) checks one library's API in the JS-family languages; its rules that follow a hook or a constructor check that it is imported from that library, so the same name from elsewhere is left alone.
   - Graph rules live with their language: `graph/no-unused-exports` in `js` and `python`, and `graph/no-unused-css-classes` in `css`. A graph rule two packs ship loads as one rule over both packs' languages with the later pack's body, so **the copies' `codelaw` blocks are identical** (a `codelaw-graph` test holds them so): when the `python` copy listed fewer kinds, a TypeScript repo lost its `interface` and `enum` findings whenever `python` loaded last (W-11 corpus run, 2026-10-22). A pack rule may also be `scope: project` under its own prefix: `node/recommended`'s six import rules (`node/no-{missing,extraneous,unpublished}-{import,require}`) run on the graph's `import:` query ([SR-PROJECT-GRAPH](0103_project-graph.md) decision 6). A file-scope native rule that reads the files around it stays file-scope: `node/hashbang` reads the nearest `package.json`'s `bin`, `md/relative-links` checks each relative link's target, and `html/no-utf8-bom` reads the file's bytes (SR-ARCHITECTURE decision 15b; W-21, 2026-10-22).
   - **A rule's fix class follows what the grammar can verify.** The engine refuses a fix whose output parses worse than its input (SR-ARCHITECTURE decision 5), so a rule whose fix writes syntax the grammar rejects ships `manual`. `css/media-feature-range-notation` is `auto` because codelaw's CSS grammar fork parses the range syntax it writes (SR-ARCHITECTURE decision 3; W-21, 2026-10-22).
   - There's **no umbrella preset that spans packs**, because it would require packs the repo hasn't installed.
3. **Pack layout.** A pack is the same folder in the source tree (`packs/<name>/` in the codelaw repo) and in each published form it has:

   ```text
   packs/js/
   ├─ codelaw-pack.toml               name, version, min_codelaw, languages, presets
   ├─ presets/<preset>.toml           the rules a preset turns on, with severity and options
   ├─ rules/<namespace>/<name>.md     rule files (SR-RULE-FORMAT), + <name>.cases.md
   ├─ data/                           tables rules read: CSS properties, HTML elements, JSON Schemas
   ├─ README.md                       rule count per preset; ported / not-ported table; upstream versions
   └─ LICENSE · NOTICE
   ```

   - **On npm,** the package *is* this folder.
   - **On PyPI,** it is one pure `py3-none-any` wheel whose `codelaw_rules_<name>` package holds this folder as package data.
   - **Every published form holds the folder's files, byte for byte**, so a pack on both registries is the same on both. A pack has no code to execute and no dependencies.
   - **A preset** is `presets/<namespace>/<name>.toml`: an optional `description` and a `[rules]` table, a severity or `{ severity, options }` per rule. Every rule it names is in the pack, so a preset never needs a second pack: the four core rules typescript-eslint switches on for TypeScript are `js/typescript` in `js`, not part of `ts/recommended` (2026-10-24), and each pack's starter conventions are its own `<namespace>/conventions`.
   - **A data table** is `data/**/*.json`, named by its path under `data/` without `.json` (`data/globals/browser.json` is `globals/browser`).
4. **Resolution.** First-party presets are named `codelaw:<namespace>/<preset>` in `extends`, for example `codelaw:js/recommended`.
   - **A pack is first-party** when its manifest `name` is one of §1's: the npm name when the pack is on npm, else its PyPI name (`codelaw-rules-python`).
   - **Namespace to pack** is a fixed map in the engine:
     - `js` → `rules-js`; `ts` → `rules-ts`; `react` → `rules-react`; `node` → `rules-node`; `stencil` → `rules-stencil`;
     - `html` → `rules-html`;
     - `css` → `rules-css`;
     - `py` → `rules-python`;
     - `data`, `md`, `json`, `yaml`, `toml` → `rules-data`;
     - `tanstack-query` → `rules-tanstack-query`; `redux-toolkit` → `rules-redux-toolkit`.
   - **The pack is found in this order,** each place only for a registry the pack is published on:
     1. `node_modules/@codelaw/rules-<name>`, walking up from the config file's directory.
     2. The Python environment: the site-packages paths the Python launcher passes in (`CODELAW_SITE_PACKAGES`); otherwise `$VIRTUAL_ENV`; otherwise `.venv/` at the project root.

     The first one found wins. Both copies of one version are identical (decision 3).
   - **A missing pack stops the run:** exit 2, with an error naming every install command the pack has, for example `npm i -D @codelaw/rules-js` and `pip install codelaw-rules-js`; a pack on one registry is named as that registry's only (`@codelaw/rules-ts, which is published on npm only`).
5. **One version, released together.** The engine and the eleven packs carry one version and are published by one release job ([L6](0008_LAW-6-python-and-node-packages.md) decision 9).
   - Each pack's manifest declares `min_codelaw`. An engine older than that refuses the pack, with an error naming the upgrade.
   - `codelaw rules --why <id>` and the JSON output name the pack and version each rule came from.
6. **`codelaw init` installs what it detects.**
   - **Languages to packs:** JS/JSX → `js`; TS/TSX → `js` and `ts`; HTML → `html`; CSS → `css`; Python → `python`; JSON/YAML/TOML/Markdown → `data`. With JS or TS, `react` when `package.json` depends on `react`, `stencil` when it depends on `@stencil/core`, `tanstack-query` when it depends on `@tanstack/react-query` or `react-query`, and `redux-toolkit` when it depends on `@reduxjs/toolkit` or `redux`. `node` is never added by `init`.
   - **How it installs:** it adds those packs as dev dependencies with the package manager the repo already uses: npm, pnpm, yarn or bun for a `package.json` repo; uv, pip or poetry for a `pyproject.toml` repo. A repo with both uses both — `rules-python` from PyPI, the other packs from npm — unless `--registry npm|pypi` picks one.
   - **A pack the chosen registry does not publish** (§1) comes from the other registry when the repo uses that one too, or uses neither; otherwise — and always under `--registry` — it is left out, its presets are not written, and `init` prints a `note:` naming the registry it is on. An npm repo with a few Python files gets no `codelaw:py/recommended` until `codelaw-rules-python` is installed from PyPI.
   - **What it writes:** `extends` with each installed pack's `recommended` presets, and `js/typescript` when the repo has TypeScript ([SR-CONFIG](0110_config.md) decision 8).
   - **`--no-install`** prints the install commands instead of running them.
   - **`--agents`** writes the agent files after the packs are installed, so the per-language instructions list the packs' rules; under `--no-install` they wait for `codelaw sync-agents` ([SR-AGENT-SURFACES](0102_agent-surfaces.md) decision 7).
   - **`--git-hooks`** installs the git hooks after everything above, and alone in a repo that already has a `codelaw.toml`; it picks and installs no pack ([SR-CLI-MCP](0115_cli-and-mcp.md) decision 11).
   - **With no pack and no local rule,** `codelaw check` reports one `codelaw/no-rules` info diagnostic naming `codelaw init`. The formatter works regardless ([L7](0009_LAW-7-linter-and-formatter.md)).
7. **Ported rules are reimplemented, never vendored.**
   - A rule ported from another linter is written fresh in codelaw's format, and its `ported_from:` names the source rule or rules ([SR-RULE-FORMAT](0112_rule-format.md) decision 2).
   - Only ideas are taken from GPL-licensed tools (Pylint, yamllint), never their text or code ([L2](0004_LAW-2-foss-dependencies.md)). A rule name that describes the same check may coincide with theirs (`yaml/document-start`, yamllint's `document-start`); the pack's NOTICE names each such coincidence instead of claiming no names were shared (W-17).
   - **Not ported:** rules that need type information or the React Compiler's analysis. Each pack's README lists them under "not ported, and why".
     - **A preset ported from a source config accounts for every rule that config turns on:** each is in the preset or in the README's "not ported" table, and the README's preset row counts against the source. `stencil/recommended` holds 16 of the 23 rules @stencil/eslint-plugin 1.4.0's `recommended` turns on, and the 7 left out are each named with a reason; `ban-side-effects` and `enforce-slot-jsdoc` need no type information and were ported (W-33, 2026-10-25).
   - **A library pack may also hold rules written from the library's own documentation,** where the docs state a rule a matcher can decide without types; such a rule has no `ported_from:`, its **Why** quotes the docs, and its "Differences from the source" says what the matcher leaves out. `tanstack-query` holds six from TanStack Query v5's guides — `no-renamed-v4-options`, `no-removed-v4-options`, `object-syntax`, `array-query-key`, `no-mutating-set-query-data`, `no-unsupported-suspense-options` — and the pack README lists the candidate dropped because the guide describes a choice, not a rule (W-42, 2026-10-25).
   - **Never ported:** layout-only rules, which the formatter owns ([L7](0009_LAW-7-linter-and-formatter.md), [SR-FORMATTER](0105_formatter.md) decision 3).
     - A source rule that is only partly layout is ported for the part the formatter leaves, and its "Differences from the source" says what was cut: `js/no-irregular-whitespace` reports templates, regular expressions, JSX text and comments, not whitespace between tokens (the decider, 2026-10-22, W-23).
   - **Data tables** in `data/` come only from sources under a permissive OSI-approved licence ([L2](0004_LAW-2-foss-dependencies.md)), each named in the pack's NOTICE with its version — for example `globals` and `known-css-properties` (MIT), html-validate's element metadata (MIT), SchemaStore (Apache-2.0).
     - CC0-1.0 is not OSI-approved, so MDN's `browser-compat-data` and `mdn-data` are not used; a table written for a pack from the specifications says so in its NOTICE.
8. **The v0 starter catalogue is kept** (below). Each starter rule goes to the pack named by its namespace.
   - A starter rule that matches a ported source rule carries it in `ported_from:` and sits in `recommended`.
   - The rest are house conventions (`js/exports-at-end`, `py/no-print`, …) and form each pack's opt-in `conventions` preset.
9. **Stability.** Each rule is `stable` or `preview`. Preview pack rules run only with `preview = true`; a local rule runs whatever its stability. A stable rule that starts finding new violations, or changes its fix, is a minor release and is listed in the changelog.
10. **Eject to customise.** `codelaw rules eject <id>` copies a pack rule into `.codelaw/rules/`, and a local rule with the same id shadows it. `codelaw rules --why <id>` shows which copy is active.
    - An ejected `native:` rule keeps its `native:` id: the copy can change the docs, severity, options and examples, but the matcher stays in the engine.
    - **`native:` is accepted only in first-party packs and in an ejected copy of one of their rules** ([SR-ARCHITECTURE](0108_architecture.md) decision 15b).
    - **A pack is first-party by where it was found, not only by its name:** its manifest names a first-party pack *and* it was reached through `codelaw:` or an `npm:`/`pypi:` spec naming that package. A `path:` pack that copies a first-party name is third-party.
11. **Team and third-party packs** use the same layout and manifest (decision 3) and are resolved through `extends`:
    ```toml
    extends = [
      "codelaw:js/recommended",       # first-party, from @codelaw/rules-js or codelaw-rules-js
      "npm:@acme/codelaw-rules",      # from node_modules
      "pypi:acme-codelaw-rules",      # from the active Python environment
      "path:../shared/codelaw-rules"  # monorepo or vendored
    ]
    ```
    A team pack may publish to npm, PyPI or both. The first-party namespaces are reserved ([SR-RULE-FORMAT](0112_rule-format.md) decision 1). An `npm:`, `pypi:` or `path:` entry enables every rule its pack holds, at the rule's own severity.
12. **Packs contain no code by default.** A pack's external fixers (any `fix: { external: … }`) are disabled unless the consumer sets `allow_pack_fixers = ["@acme/codelaw-rules"]`. Installing a rule pack must never mean running someone else's code by surprise. First-party packs ship no external fixers.
13. **Every rule, in any pack, obeys [L4](0006_LAW-4-every-rule-has-a-fix.md).** Every rule goes through the same load-time checks and `codelaw test`. A pack whose rules fail to load is refused as a whole, with the failing rule named.
14. **Precedence, highest first:**
    1. local `.codelaw/rules/`;
    2. packs, in `extends` order (later wins), first-party and third-party alike.

    A pack can't loosen a rule the consumer set locally.

### Starter catalogue (v0)

The fix class follows [L4](0006_LAW-4-every-rule-has-a-fix.md). The aim is `auto` wherever an edit can't change behaviour. The pack is the one the namespace maps to (decision 4).

| Id | Languages | Checks | Fix |
|---|---|---|---|
| `js/exports-at-end` | js jsx ts tsx | Exports declared inline instead of in one block at the end | **auto** |
| `js/import-groups` | js jsx ts tsx | Imports in builtin → external → internal → relative order, one blank line between groups | **auto** |
| `js/no-default-export` | js jsx ts tsx | `export default` | manual (importers in other files must change) |
| `ts/type-only-imports` | ts tsx | `import { T }` used only as a type (syntactic check within the file) | suggest |
| `react/hook-prefix` | jsx tsx | Custom hook not named `use*` | manual (callers rename) |
| `react/props-type-name` | tsx | Component props type not named `<Component>Props` | suggest (local rename) |
| `stencil/tag-prefix` | tsx | `@Component({ tag })` without the configured prefix | manual (HTML usages change) |
| `stencil/prop-readonly` | tsx | `@Prop()` not `readonly` and not `mutable: true` | **auto** |
| `py/all-sorted` | python | `__all__` not sorted | **auto** |
| `py/no-mutable-default` | python | `def f(x=[])` / `{}` / `set()` | suggest (`None` + initialise in body) |
| `py/no-bare-except` | python | `except:` | suggest (`except Exception:`) |
| `py/no-print` | python | `print()` under `src/` | suggest (`logging`) |
| `py/no-tunable-literals` | python | A threshold, timeout or limit hard-coded in code instead of config | manual (pick the config key) |
| `json/package-scripts-sorted` | json | `package.json` `scripts` not sorted | **auto** |
| `json/package-engines` | json | `package.json` missing `engines.node` | suggest (value from config) |
| `json/tsconfig-strict` | json | `tsconfig.json` without `"strict": true` | suggest (may surface type errors) |
| `json/no-duplicate-keys` | json | Same key twice in one object | manual (which value wins?) |
| `yaml/no-duplicate-keys` | yaml | Same key twice in one mapping | manual |
| `yaml/gha-pin-sha` | yaml | GitHub Action pinned to a tag, not a commit SHA | manual (needs the network, so [L5](0007_LAW-5-no-model-in-the-engine.md) forbids auto) |
| `yaml/document-start` | yaml | Missing or inconsistent `---` | **auto** |
| `toml/pyproject-requires-python` | toml | `pyproject.toml` missing `requires-python` | suggest (value from config) |
| `toml/deps-sorted` | toml | `[project].dependencies` not sorted | **auto** |
| `md/single-h1` | markdown | More than one `#` heading | manual |
| `md/heading-increment` | markdown | Heading level jumps (`#` → `###`) | suggest |
| `md/fence-language` | markdown | Fenced code block without a language | manual (which language?) |
| `md/relative-links` | markdown | Relative link to a file that doesn't exist (a native rule on the file's neighbours, [SR-ARCHITECTURE](0108_architecture.md) decision 15b; in `md/recommended`) | manual |
| `css/no-empty-block` | css | Empty rule block `a {}` | **auto** (remove it) |
| `css/no-duplicate-selector` | css | The same selector declared twice in one file | manual (merging depends on cascade order) |
| `css/no-important` | css | `!important` | manual |
| `css/no-unknown-property` | css | A property not in the pack's CSS property table | suggest (closest known name) |
| `html/img-alt` | html | `<img>` without `alt` | manual (the text needs a human) |
| `html/target-blank-rel` | html | `target="_blank"` without `rel="noopener"` | **auto** |
| `html/no-duplicate-id` | html | The same `id` twice in one document | manual |
| `html/button-type` | html | `<button>` without `type` | suggest (`type="button"`; changes form behaviour) |

That's 34 rules: 9 auto, 11 suggest and 14 manual. **33 ship today** (all but `md/relative-links`; 13 manual). Every `manual` rule carries `fix_reason:` and `## How to fix` ([L4](0006_LAW-4-every-rule-has-a-fix.md) decisions 2–3).

### Consequences

- **`codelaw` installed alone lints nothing.** It formats every language and reports `codelaw/no-rules`. `codelaw init` is the documented path, and `codelaw-setup` follows it ([SR-AGENT-SURFACES](0102_agent-surfaces.md)).
- **Each release publishes twenty-six packages:** on npm, `codelaw`, the eight `@codelaw/cli-*` binary packages and ten packs (nineteen); on PyPI, `codelaw` and six packs (seven). That is 33 files — 19 npm tarballs and 14 wheels — because PyPI's `codelaw` is eight platform wheels. One failed publish fails the release ([L6](0008_LAW-6-python-and-node-packages.md) decision 9).
- **`@codelaw/rules-python@0.1.0` and `@codelaw/rules-js@0.1.0` predate the split** (the first npm publish, 2026-10-24, W-29): `rules-python` is unpublished from npm by hand, and 0.2.0 is the first release in this layout ([W-31](../archive/open/W-31-registry-pack-split.md)).
- **A repo carries only its ecosystems' rules and data tables.** A Python-only repo stays on PyPI ([L3](0005_LAW-3-languages.md) decision 2).
- **Rules still update with engine releases** (one version). Engine and rules can't drift, which was the reason for embedding them in the first version: `min_codelaw` and the native-id check keep that guarantee.
- **The engine work this needs:** the semantic model, `native:` rules and pack loading, in [W-02](../archive/open/W-02-engine-p0.md) ([SR-ARCHITECTURE](0108_architecture.md) decisions 15a–15c).
- **The packs' content** is [W-03](../archive/open/W-03-rule-packs.md).

### Alternatives considered

- **Built-in rules embedded in the binary** (this record's 2026-10-18 version): superseded by the decider's ruling of 2026-10-19. Its argument, that rules and engine can't drift apart, is kept by one version and `min_codelaw`.
- **One `codelaw-rules` package for every language:** rejected. A Python-only repo would install JS, HTML and CSS rules and their data tables.
- **A package per language (eleven):** rejected. JS, JSX, TS and TSX share most rules, and the JSON, YAML, TOML and Markdown rules are small and share schema data. The JS family splits by framework instead (2026-10-24).
- **Every pack mirrored on both registries** (the 2026-10-19 ruling, five packs): superseded by the decider's ruling of 2026-10-24. Each registry carries only the packs its users install.
- **Packs on npm only, with a Python shim:** rejected. It breaks [L3](0005_LAW-3-languages.md) and [L6](0008_LAW-6-python-and-node-packages.md).
- **Rules written as Rust code:** still rejected as the face of a rule, because people and agents can't read them. `native:` keeps the Markdown file as the rule while the engine runs the check.

### Reference (required)

- [W-03](../archive/open/W-03-rule-packs.md): the preset research behind §1's table.
- [ESLint recommended](https://cdn.jsdelivr.net/npm/@eslint/js/src/configs/eslint-recommended.js) · [typescript-eslint configs](https://typescript-eslint.io/users/configs/) · [Ruff rules](https://docs.astral.sh/ruff/rules/) · [stylelint-config-recommended](https://cdn.jsdelivr.net/npm/stylelint-config-recommended/index.js) · [html-validate presets](https://html-validate.org/rules/presets.html) · [markdownlint](https://github.com/DavidAnson/markdownlint).
- [SR-LAW-6](0008_LAW-6-python-and-node-packages.md): the pack package names and the joint release.
- [`docs/DESIGN.md`](../docs/DESIGN.md) §3.2: the rule-file format.

### Veto condition

**Reopen this decision if:**
- a first-party pack needs code executed at load;
- a pack on both registries holds different files on npm and on PyPI;
- a Python-only or Node-only repo needs the other registry to get its rules;
- `packs/`, `FIRST_PARTY_PACKS` and the published names disagree.

**How to check it:** `python3 scripts/package.py check dist` compares every published copy of each pack with `packs/<name>/`, file by file, and the names with `FIRST_PARTY_PACKS`; `cargo test -p codelaw-rules` loads every `packs/` folder as its first-party pack.

---

## References

*Every source this record cites, gathered in one place. An archived document is never listed here.*

**Records** — [SR-LAW-2](0004_LAW-2-foss-dependencies.md) · [SR-LAW-3](0005_LAW-3-languages.md) · [SR-LAW-4](0006_LAW-4-every-rule-has-a-fix.md) · [SR-LAW-5](0007_LAW-5-no-model-in-the-engine.md) · [SR-LAW-6](0008_LAW-6-python-and-node-packages.md) · [SR-LAW-7](0009_LAW-7-linter-and-formatter.md) · [SR-AGENT-SURFACES](0102_agent-surfaces.md) · [SR-BASELINE](0104_baseline-and-deprecations.md) · [SR-FORMATTER](0105_formatter.md) · [SR-ARCHITECTURE](0108_architecture.md) · [SR-CONFIG](0110_config.md) · [SR-RULE-FORMAT](0112_rule-format.md)

**Project docs** — [`docs/DESIGN.md`](../docs/DESIGN.md) · [W-02](../archive/open/W-02-engine-p0.md) · [W-03](../archive/open/W-03-rule-packs.md) · [W-31](../archive/open/W-31-registry-pack-split.md) · [W-32](../archive/open/W-32-redux-and-query-packs.md)
