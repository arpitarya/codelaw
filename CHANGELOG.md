# Changelog

Every user-visible change, newest first. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow [Semantic Versioning](https://semver.org/).

## Unreleased

### Added

- **Licence: MIT**, for the repo and every package. `LICENSE` at the root and in each of the five rule packs.
- `README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md` and the GitHub issue and pull-request templates.
- **The formatter:** `codelaw format [--check] [--diff]` for all nine languages, with six engines compiled in: Biome, Ruff's formatter, pretty_yaml, taplo, dprint-plugin-markdown and markup_fmt. `fix` now ends with a format. `check` reports unformatted files as `format/unformatted` (`auto`). A format that would not parse, would change again on a second run, or would change a JSON/YAML/TOML value is refused and reported as `format/failed`. Line endings are kept unless `[format] end_of_line` is set.
- **`codelaw mcp`**, the MCP server for agents, and **`codelaw lsp`**, the language server: diagnostics, quick fixes, fix all, hovers and document formatting.
- **Baselines and deprecations:** `codelaw baseline add|prune|check` grandfathers a rule's existing findings per file, with a deadline (`until`), a touch policy (`on_touch`) and a ratchet that lets counts only go down; `codelaw deprecate` and `.codelaw/deprecations.toml` drive the engine's own `graph/no-deprecated-usage`. Deadlines are judged on `HEAD`'s commit date or `--as-of`, never the clock; `--changed-since <ref>` and `--staged` say which files count as touched.

### Changed

- `codelaw fix` now formats every file it checks. Set `[format] enabled = false` to keep the old behaviour, or `[format] check = false` to keep formatting out of `check`.

## 0.1.0 — preview, not published

The first working engine and rule packs, built from source. Nothing has been published to npm or PyPI.

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

### Not yet built

- The formatter, the MCP server, agent hooks and `init --agents`, the language server, project-wide rules, baselines and SARIF output. See the README's roadmap.
