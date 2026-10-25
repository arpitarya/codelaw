# Changelog

Every user-visible change, newest first. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/). Each line names the work item it came from (`W-nn`); `python3 scripts/docs-sync.py --check` fails when an item `work/IMPLEMENTATION.md` dates after the last release is not named under *Unreleased*.

## Unreleased

### Added

- **Git hooks in one command:** `codelaw init --git-hooks [pre-commit,post-commit]` writes a blocking `pre-commit` (fixes the fully staged files and re-stages them, checks every staged file, never fixes a partly staged one) and a post-commit that always exits 0, chains any hook already there and never overwrites one. `.pre-commit-hooks.yaml` ships hooks `codelaw-check` and `codelaw-fix` for the pre-commit framework, and USAGE has husky and lefthook recipes (SR-CLI-MCP decision 11, W-50).
- **Dependencies behind their latest version, highlighted:** `codelaw deps refresh` reads npm and the Python index (a private npm registry and token from `.npmrc`, a private index from `PIP_INDEX_URL` or `[[tool.uv.index]]`) and writes `.codelaw/deps-snapshot.json`, sorted so a refresh with nothing new writes the same bytes — the only command that uses the network (L5 decision 3, amended). `data/outdated-dependency` reads it offline on every `package.json` and `pyproject.toml`: a major version behind is `warn`, minor and patch `info`, a deprecated (npm) or yanked (PyPI) version in use `error`, a lockfile behind a range that allows the latest `info`; its `suggest` fix moves the range and keeps `^`, `~`, `>=` or `==`. `data/stale-deps-snapshot` reports a snapshot older than `[deps] max_snapshot_age` on the as-of date. `[deps]` sets each kind's severity, `min_release_age`, `prerelease` and `ignore`. Both rules are in `data/package-json` and `data/pyproject` (W-52).
- **Six TanStack Query rules from TanStack Query v5's own guides**, each where a guide states a rule a matcher can decide without types: `tanstack-query/no-renamed-v4-options` (`cacheTime` → `gcTime`, `useErrorBoundary` → `throwOnError`, fixed automatically), `no-removed-v4-options` (query `onSuccess` / `onError` / `onSettled`, `keepPreviousData`, `isDataEqual`, `context`), `object-syntax` (v4's positional `useQuery(key, fn, options)` and QueryClient calls, with a suggested rewrite), `array-query-key`, `no-mutating-set-query-data` and `no-unsupported-suspense-options` (`enabled`, `placeholderData`, `throwOnError` on a suspense query). All six are in `tanstack-query/recommended` (13 rules), and the pack now has a pinned corpus: `full-stack-fastapi-template`, `bulletproof-react` and TanStack Query's own repository (W-42).
- **Exceptions with an expiry date, a reason and an author:** a baseline entry, a deprecation and an inline `codelaw-ignore` take `expires` (the first day it no longer holds), `warn_days` and `added_by` (`by=` inline). From `warn_days` before the date (3 by default) the exception is a warning, `codelaw/expiring-exception`; on the day it is an error, `codelaw/expired-exception`, and its findings come back at their own severity. `[exceptions] require_expiry = true` makes the date mandatory (`codelaw/exception-without-expiry`); `added_by` is checked against git under `--staged` and `--changed-since` (`codelaw/exception-author`). `baseline add` and `deprecate` take `--expires`, `--warn-days` and `--added-by` (default `git config user.email`). `until` still works for one minor release, read as `expires` the day after; messages now say "expires 2027-04-01" where they said "deadline 2027-03-31" (SR-BASELINE decision 9, W-40).
- **`react/recommended` gains ten React hooks rules**, from eslint-plugin-react-hooks 7.1.1's React Compiler rules and @eslint-react 5.28.8, each decided from one file's syntax: `react/no-set-state-in-render`, `react/no-set-state-in-effect`, `react/no-ref-in-render` (`ref.current` during render, the `if (ref.current === null)` initialisation allowed), `react/purity` (`Math.random()`, `Date.now()`, `new Date()` … during render), `react/no-nested-components`, `react/use-memo` (an async, parameterised or value-less `useMemo` callback, or an unused result), `react/globals` (a module-level or global value changed during render) and `react/error-boundaries` (`try`/`catch` around returned JSX or `use(…)`) at `error`; `react/no-unnecessary-use-prefix` and `react/prefer-lazy-state-init` (`useState(compute())` → `useState(() => compute())`, a `suggest` fix) at `warn`. The preset now holds 29 rules (W-43).
- **`react/no-context`**: no `createContext` or `useContext` from `react` — named, aliased, or `React.*` — and the recipe to adopt it in a codebase that already uses context: `codelaw baseline add --rule react/no-context --warn-from <today> --on-touch keep` keeps today's uses as warnings while a new use, in a new file or an old one, fails. A team's choice, so in no preset; `codelaw explain react/no-context` shows the recipe (W-41).
- **Warn on a laptop, fail in CI:** `[profiles.<name>]` holds `rules` and `overrides` laid over the config. codelaw runs the `ci` profile when `CI` is set (not `false` or `0`), `local` otherwise; `--profile <name>` or `CODELAW_PROFILE` picks one. `codelaw hook` and the MCP server run `ci`, so an agent gets CI's verdict; the editor runs `local`. Reports name the profile in use (`profile ci (CI=true)`; `profile` in JSON; `config --print`) (W-39).
- **`remap` on an `[[overrides]]` entry:** maps each severity in a glob to another or to `off` — `{ error = "warn" }` never fails the run, `{ warn = "error" }` admits no warnings — applied last, after the baseline, never to `codelaw/*` diagnostics (W-39).
- **`js/no-console`:** calls on the global `console`, with an `allow` option and a `suggest` fix that deletes the statement; in `js/recommended` at `warn` (W-39).

### Changed

- **`publish.yml` can publish npm on its own:** a run by hand on a `v*` tag publishes npm only, and the GitHub Release on the same tag later adds PyPI and skips what npm already has (L6 decision 9, W-37). The rest of the release work of W-03, W-04, W-29, W-30 and W-31 is W-37 (npm) and W-38 (PyPI).
- **The README tells codelaw's story** — the four moments (before, while and after an AI writes, and with no AI), and how it compares with ESLint + Prettier and Biome — and **the how-to moved to a new [`USAGE.md`](USAGE.md)**: install, configure, write rules, agents, editor, pre-commit, CI and every command (W-35).

## 0.2.0 — 2026-10-25

**The rule packs, one per ecosystem, on the registry its users install from.**
`rules-js` splits into `js`, `ts`, `react`, `node` and `stencil`; two library
packs arrive, TanStack Query and Redux Toolkit; and codelaw is on PyPI for the
first time, with the six packs a Python repo needs. 440 rules in eleven packs,
one engine, the same verdict from npm and from PyPI.

```bash
npx codelaw@0.2.0 init     # a Node repo: installs the @codelaw/rules-* it needs
uvx codelaw@0.2.0 init     # a Python repo: installs the codelaw-rules-* it needs
```

**Before you upgrade from 0.1.0:** a config that names only
`codelaw:ts/recommended` loses `js/no-var`, `js/prefer-const`,
`js/prefer-rest-params` and `js/prefer-spread` — add `codelaw:js/typescript`
(`codelaw init` writes it for a TypeScript repo). `codelaw:js/conventions` now
holds only the `js/*` conventions; the rest moved to `ts/`, `react/` and
`stencil/conventions`. On npm, the 0.1.0 `@codelaw/rules-js` keeps the rules
that moved, so pin every `@codelaw/rules-*` to 0.2.0 together.

### Added

- **`stencil/recommended` gains `stencil/ban-side-effects` and `stencil/enforce-slot-jsdoc`**, the two rules of @stencil/eslint-plugin 1.4.0's `recommended` that need no type information and were not yet ported: a call at a module's top level, and a component whose `@slot` JSDoc tags and rendered `<slot>`s disagree. The preset now holds 17 rules, and the pack README counts against the 23 upstream turns on (W-33).
- **On PyPI for the first time:** the six `codelaw-rules-*` packs (`js`, `react`, `html`, `css`, `python`, `data`) at 0.2.0, `pip install codelaw-rules-python` and the rest; `codelaw` itself follows with the release (W-30).
- **Two npm-only library packs** (W-32): `@codelaw/rules-tanstack-query` (`tanstack-query/recommended`, 7 rules: @tanstack/eslint-plugin-query's `recommended`, reimplemented) and `@codelaw/rules-redux-toolkit` (`redux-toolkit/recommended`, 12 rules codelaw wrote from the Redux Style Guide's Priority A and B rules). `codelaw init` adds them when `package.json` depends on `@tanstack/react-query` or `react-query`, and on `@reduxjs/toolkit` or `redux`. npm now carries ten packs.
- **`codelaw bench --repo <dir>`** times `check`, cold and warm, on a copy of a real repository with the reference workload's config and packs; CI's `perf` job runs it on L8's three pinned repos (W-19).
- **`js/project`, `py/project` and `css/project`**, opt-in presets for the rules that read the whole repo: `graph/no-unused-exports` and `graph/no-unused-css-classes` were in no preset, so nothing turned them on but a `[rules]` line (W-26).
- **The README's and the pack READMEs' counts, tables and roadmap are checked against the tree** by `scripts/docs-sync.py` in CI (W-26).
- **A pinned corpus of ten permissively licensed repositories** (`scripts/corpus.json`) that every pack's rules run over in CI, holding each rule to its invariants on real files (W-20).

### Changed

- **The README opens with what codelaw does for an AI agent before, while and after it writes**, each with where it stands today; the while-writing check is in design (W-35).
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
