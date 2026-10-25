# Changelog

Every user-visible change, newest first. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/). Each line names the work item it came from (`W-nn`); `python3 scripts/docs-sync.py --check` fails when an item `work/IMPLEMENTATION.md` dates after the last release is not named under *Unreleased*.

## Unreleased

### Added

- **Two npm-only library packs** (W-32): `@codelaw/rules-tanstack-query` (`tanstack-query/recommended`, 7 rules: @tanstack/eslint-plugin-query's `recommended`, reimplemented) and `@codelaw/rules-redux-toolkit` (`redux-toolkit/recommended`, 12 rules codelaw wrote from the Redux Style Guide's Priority A and B rules). `codelaw init` adds them when `package.json` depends on `@tanstack/react-query` or `react-query`, and on `@reduxjs/toolkit` or `redux`. npm now carries ten packs.
- **`codelaw bench --repo <dir>`** times `check`, cold and warm, on a copy of a real repository with the reference workload's config and packs; CI's `perf` job runs it on L8's three pinned repos (W-19).
- **`js/project`, `py/project` and `css/project`**, opt-in presets for the rules that read the whole repo: `graph/no-unused-exports` and `graph/no-unused-css-classes` were in no preset, so nothing turned them on but a `[rules]` line (W-26).
- **The README's and the pack READMEs' counts, tables and roadmap are checked against the tree** by `scripts/docs-sync.py` in CI (W-26).
- **A pinned corpus of ten permissively licensed repositories** (`scripts/corpus.json`) that every pack's rules run over in CI, holding each rule to its invariants on real files (W-20).

### Changed

- **Nine rule packs, each on the registry its users install from** (W-31). `rules-js` is split into `rules-js`, `rules-ts`, `rules-react`, `rules-node` and `rules-stencil`. npm carries eight packs (`js`, `ts`, `react`, `node`, `stencil`, `html`, `css`, `data`); PyPI carries six (`js`, `react`, `html`, `css`, `python`, `data`). `@codelaw/rules-python` is no longer on npm, and `ts`, `node` and `stencil` are not on PyPI. The rules themselves are unchanged (419).
  - **Before you upgrade:** `codelaw:ts/recommended` no longer turns on `js/no-var`, `js/prefer-const`, `js/prefer-rest-params` and `js/prefer-spread`; add `codelaw:js/typescript` for them (`codelaw init` writes it for a TypeScript repo). `codelaw:js/conventions` holds only the `js/*` conventions; the others are `ts/conventions`, `react/conventions` and `stencil/conventions`.
  - **`codelaw init`** installs `rules-ts` for TypeScript and `rules-react` or `rules-stencil` when `package.json` depends on them, and leaves out, with a `note:`, a pack the repo's registry does not publish.
- **Release builds are verified and attested:** every release step runs under `pipefail`, every action is pinned by SHA, `SHA256SUMS` is checked and the CI build attested before anything is published, and each of the eight platforms is smoke-installed (W-18); CI runs the law gate for real and skips `perf` when nothing it times changed (W-24).
- **The minimum Rust version is 1.97**, not 1.85: the pinned `ruff_*` crates need it. It matters only when you build codelaw from source (W-27).
- **A rule's example counts as layout-only when codelaw's own tokens are the same before and after**, not when an external formatter says so; that formatter comparison is now only a warning (W-28).
- **The records name an owner for every module and say which are built**, and the inbox's age is computed, not stored (W-25). This is project bookkeeping and changes nothing you run.

## 0.1.0 — 2026-10-24, preview, on npm only

The first release: the engine and the five rule packs of that layout, published to npm by hand from CI's build of `5e489f4` (W-29). Nothing is on PyPI yet.

### Added

- **The engine:** one Rust binary covering JavaScript, JSX, TypeScript, TSX, Python, JSON, YAML, TOML, Markdown, CSS and HTML, alone or in any combination.
- **Commands:** `check`, `fix` (with `--unsafe` and `--dry-run`), `test`, `rules` (`--for`, `lint`), `explain`, `tree`, `new-rule`, `init` and `cache`.
- **Output formats:** `text`, `json` and `agent`.
- **Rules as Markdown files:** front matter, a `codelaw` matcher and fix block, and `Do` / `Don't` examples that double as tests.
- **Fix classes:** `auto`, `suggest` and `manual`. Fixes run in passes until nothing more changes, and a fix that would add a syntax error is rolled back.
- **Five rule packs, 419 rules, all `preview`:**
  - `js`: 140 rules, from ESLint, typescript-eslint, eslint-plugin-react, react-hooks, eslint-plugin-n and @stencil/eslint-plugin.
  - `python`: 99 rules, from Ruff's defaults, flake8-bugbear, isort and `RUF022`.
  - `html`: 72 rules, from html-validate.
  - `css`: 71 rules, from stylelint's recommended and standard configs.
  - `data`: 37 rules, JSON/YAML/TOML correctness and markdownlint.
- **Configuration:** `codelaw.toml`, `[tool.codelaw]` in `pyproject.toml`, or `"codelaw"` in `package.json`, with strict validation, `[rules]` and `[[overrides]]`.
- **Local rules** in `.codelaw/rules/`, picked up with no build step.
- **Licence: MIT**, for the repo and every package. `LICENSE` at the root and in each rule pack.
- `README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md` and the GitHub issue and pull-request templates (W-06).
- **The formatter:** `codelaw format [--check] [--diff]` for all nine languages, with six engines compiled in: Biome, Ruff's formatter, pretty_yaml, taplo, dprint-plugin-markdown and markup_fmt. `fix` ends with a format. `check` reports unformatted files as `format/unformatted` (`auto`). A format that would not parse, would change again on a second run, or would change a JSON/YAML/TOML value is refused and reported as `format/failed`. Line endings are kept unless `[format] end_of_line` is set (W-07).
  - `.editorconfig` is read under `[format]`; `codelaw format --import-config` turns a Prettier, Ruff or Black config into `[format]`; `codelaw-format-ignore` and `prettier-ignore` markers are honoured by every engine (W-23).
- **`codelaw mcp`**, the MCP server for agents (W-08), and **`codelaw lsp`**, the language server: diagnostics, quick fixes, fix all, hovers, document and range formatting; closed files' trees kept in a bounded cache, `[lsp] tree_cache` (W-09, W-15).
- **Agents:** `codelaw hook` checks and fixes the files an agent just edited and returns what is left; `codelaw init --agents` installs the hooks and writes each agent's files; `codelaw sync-agents [--check]` rewrites them from the rules and skills (W-10).
- **Project-wide rules** on a whole-repo graph: `graph/no-unused-exports` (JS, TS, Python), `graph/no-unused-css-classes`, and the six `node/no-{missing,extraneous,unpublished}-{import,require}` rules; `codelaw graph refs` and `graph why-unused` (W-11).
- **Rules on JSON, YAML and TOML paths:** `target:` + `assert:` rules, `schema:`, and the `sort-keys`, `ensure-key` and `when` fixes (W-12).
- **Baselines and deprecations:** `codelaw baseline add|prune|check` grandfathers a rule's existing findings per file, with a deadline (`until`), a touch policy (`on_touch`) and a ratchet that lets counts only go down; `codelaw deprecate` and `.codelaw/deprecations.toml` drive the engine's own `graph/no-deprecated-usage`. Deadlines are judged on `HEAD`'s commit date or `--as-of`, never the clock; `--changed-since <ref>` and `--staged` say which files count as touched (W-13).
- **Adapters:** `check` and `fix` also run the installed oxlint, Ruff, stylelint and html-validate and merge their findings; `--no-adapters` turns them off (W-14).
- **Warm processes:** `codelaw daemon start|stop|status`, warm sockets for `lsp` and `mcp`, and `--prefer-warm`; a compiled rule-set cache (W-15).
- **Debugging and output:** `codelaw config --print|--schema`, `codelaw match --explain`, `fix --trace`, `--timings`, `--format sarif`, `format --format json`; the npm and Python APIs gain `format` and `rulesFor` / `rules_for` (W-16).
- **`codelaw bench`** times every performance-budgeted operation on a generated 10,000-file workload and compares it with `perf/budgets.toml` (W-19); it builds on Windows (W-24).
- **Rules added since the packs were first built:** `md/relative-links`, `html/no-utf8-bom` and `node/hashbang`, which read the files around them; `css`/`html` tagged templates in JS and TS are checked as CSS and HTML (W-21).

### Changed

- **CSS parsing** reads range media features (`@media (400px <= width <= 700px)`), `layer()` in `@import` and fractional keyframe selectors (`12.5%`) without syntax errors, through codelaw's fork of tree-sitter-css 0.25.0. Fixes that write them are no longer refused (W-21).
- `css/media-feature-range-notation` now has an `auto` fix: `(min-width: 600px)` becomes `(width >= 600px)` (W-21).
- `js/no-irregular-whitespace` reports templates, regular expressions, JSX text and comments, not whitespace between tokens, which the formatter owns (W-23).
- `codelaw test` refuses a rule that only checks layout (W-23).

### Not yet built

- The PyPI package, the editor extension's Marketplace listing, and promotion of rules to `stable` (the pinned corpus and its CI job). See the README's roadmap.
