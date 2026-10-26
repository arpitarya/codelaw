---
type: Registry
description: "L11's list: every third-party dependency codelaw uses, one row per ratification unit, with its status. Nothing outside this list may be used."
governed_by: SR-LAW-11
governed_by_path: records/0013_LAW-11-no-dependencies.md
---

# DEPENDENCIES — every dependency, and whether the decider has ratified it

**Ruled 2026-10-26** (the decider, by voice, one card per unit): 29 ratified, 3 to remove — `strsim`, `anyhow`, `postcard` (W-67). Each ratification accepts the unit's transitive tree as pinned in `Cargo.lock` / `package-lock.json` that day (L11 decision 4).

**The rule is [L11](records/0013_LAW-11-no-dependencies.md):** no dependency unless ratified here by name. Statuses: `pending` (today's pin, awaiting the decider) · `ratified YYYY-MM-DD` (with his words) · `remove` (a work item replaces it).

**Already dependency-free:** the published `codelaw` packages on npm and PyPI (zero runtime dependencies; the npm one names only its own `@codelaw/cli-*` binaries) and every script in `scripts/` and `.claude/hooks/` (Python standard library only).

## Rust — compiled into the `codelaw` binary (runtime)

| id | unit | crates | what it does for codelaw | pin | crates incl. transitive | status |
|---|---|---|---|---|---|---|
| D-01 | **Biome** | `biome_formatter`, `biome_languages`, `biome_js_parser`, `biome_js_formatter`, `biome_json_parser`, `biome_json_formatter`, `biome_css_parser`, `biome_css_formatter` | formats JS, TS, JSX, JSON, CSS | git `0b4c11f` (Biome 2.5.15) | 133 | ratified 2026-10-26 — *"Ratify."* |
| D-02 | **Ruff** | `ruff_formatter`, `ruff_python_formatter`, `ruff_python_parser`, `ruff_python_ast`, `ruff_text_size` | formats Python | `=0.0.18` | 165 | ratified 2026-10-26 — *"Ratify."* |
| D-03 | **ast-grep** | `ast-grep-core`, `ast-grep-config`, `ast-grep-language` | the `match:` rule engine; brings the tree-sitter grammars for JS, TS, Python, JSON, YAML, Markdown, CSS, HTML | `=0.51.0` | 91 | ratified 2026-10-26 — *"Ratify."* |
| D-04 | **tree-sitter** | `tree-sitter`, `tree-sitter-language`, `tree-sitter-md`, `tree-sitter-toml-ng` | the parser runtime every lint rule stands on; Markdown and TOML grammars | `=0.27.0` | 26 | ratified 2026-10-26 — *"Ratify."* |
| D-05 | **tree-sitter-css fork** | in-tree copy of `tree-sitter-css` 0.25.0 (+ `cc`, build) | the CSS grammar (B-047: SCSS joins via tree-sitter-scss, W-62) | vendored | 3 | ratified 2026-10-26 — *"Ratify."* |
| D-06 | **taplo** | `taplo` | formats TOML | `=0.14.0` | 64 | ratified 2026-10-26 — *"Ratify."* |
| D-07 | **pretty_yaml** | `pretty_yaml` | formats YAML | `=0.6.0` | 10 | ratified 2026-10-26 — *"Ratify."* |
| D-08 | **dprint Markdown** | `dprint-plugin-markdown`, `dprint-core` | formats Markdown | `=0.26.0` | 23 | ratified 2026-10-26 — *"Ratify."* |
| D-09 | **markup_fmt** | `markup_fmt` | formats HTML | `=0.27.6` | 12 | ratified 2026-10-26 — *"Ratify."* |
| D-10 | **serde** | `serde`, `serde_json` | reads and writes JSON and Rust data everywhere | 1.0.229 / 1.0.154 | 16 | ratified 2026-10-26 — *"Ratify."* |
| D-11 | **toml** | `toml`, `toml_edit` | reads `codelaw.toml`; edits it keeping the user's formatting | 1.1.8 / 0.25.17 | 19 | ratified 2026-10-26 — *"Ratify."* |
| D-12 | **serde-saphyr** | `serde-saphyr` | reads YAML for the data rules | 2.0.0 | 28 | ratified 2026-10-26 — *"Ratify."* |
| D-13 | **pulldown-cmark** | `pulldown-cmark` | reads rule files (Markdown) | 0.13.4 | 4 | ratified 2026-10-26 — *"Ratify."* |
| D-14 | **clap** | `clap` | command-line parsing | 4.6.7 | 21 | ratified 2026-10-26 — *"Ratify."* |
| D-15 | **LSP** | `lsp-server`, `lsp-types` | the editor server | 0.10.0 / 0.97.0 | 20 | ratified 2026-10-26 — *"Ratify."* |
| D-16 | **regex** | `regex` | regular expressions in rules and config | 1.13.1 | 5 | ratified 2026-10-26 — *"Ratify."* |
| D-17 | **file walking** | `ignore`, `globset` | `.gitignore`-aware discovery, globs | 0.4.33 / 0.4.20 | 22 | ratified 2026-10-26 — *"Ratify."* |
| D-18 | **rayon** | `rayon` | checks files in parallel (L8 budgets) | 1.12.0 | 6 | ratified 2026-10-26 — *"Ratify."* |
| D-19 | **blake3** | `blake3` | content hashes for the cache | 1.8.7 | 9 | ratified 2026-10-26 — *"Ratify."* |
| D-20 | **postcard** | `postcard` | the cache's on-disk format | 1.1.3 | 24 | **remove** — *"Replace."* (2026-10-26) → W-67 |
| D-21 | **similar** | `similar` | diffs (`--diff`, LSP range formatting) | 2.7.0 | 13 | ratified 2026-10-26 — *"Ratify."* |
| D-22 | **semver** | `semver` | version ranges (`requires`, packs, deps) | 1.0.28 | 1 | ratified 2026-10-26 — *"Ratify."* |
| D-23 | **strsim** | `strsim` | "did you mean …?" suggestions | 0.11.1 | 1 | **remove** — *"Remove"* (2026-10-26, voice) → W-67 |
| D-24 | **anyhow** | `anyhow` | error type in the format crate | 1.0.104 | 1 | **remove** — *"Remove"* (2026-10-26, voice) → W-67 |
| D-25 | **libc** | `libc` | OS calls (signals, Unix sockets) | 0.2.191 | 1 | ratified 2026-10-26 — *"Ratify."* |

## Rust — tests and build only

| id | unit | what it does | pin | status |
|---|---|---|---|---|
| D-26 | **boon** | validates JSON Schema and SARIF in tests | 0.6.1 | ratified 2026-10-26 — *"Ratify."* |

## VS Code extension (`packaging/vscode`)

| id | unit | scope | what it does | pin | status |
|---|---|---|---|---|---|
| D-27 | **vscode-languageclient** | runtime | connects VS Code to `codelaw lsp` | 9.0.1 | ratified 2026-10-26 — *"Ratify."* |
| D-28 | **typescript, @types/node, @types/vscode** | dev | compile the extension | 5.6.3 / 20.17.6 / 1.82.0 | ratified 2026-10-26 — *"Ratify."* |

## Python packaging

| id | unit | scope | what it does | status |
|---|---|---|---|---|
| D-29 | **maturin** (+ `zig` on some CI legs) | build | builds the PyPI wheels around the binary | ratified 2026-10-26 — *"Ratify."* |

## CI — Fux (build tool of the index packages)

| id | unit | what it does | pin | status |
|---|---|---|---|---|
| D-33 | **fux-engine** (+ its three pinned dependencies `fux-core`, `fux-reader`, `fux-writer`, all 5.1.0), MIT | builds the optional Fux index packages in CI (`scripts/fux-index.py`, L6 decision 12); never in the engine, a package or a test | `==5.1.0` (`FUX_VERSION`) | **pending** — named as a build-time tool of CI by the decider's W-34 ruling of 2026-10-25, before L11; ratify it here by name, or W-34's build is removed |

## CI — GitHub Actions (`.github/workflows/`)

| id | unit | what it does | status |
|---|---|---|---|
| D-30 | **GitHub's own actions** — `checkout`, `cache/restore`, `cache/save`, `upload-artifact`, `download-artifact`, `setup-node`, `setup-python`, `attest-build-provenance` | CI plumbing and build provenance; pinned by SHA | ratified 2026-10-26 — *"Ratify."* |
| D-31 | **Swatinem/rust-cache** | caches Rust builds in CI | ratified 2026-10-26 — *"Ratify."* |
| D-32 | **pypa/gh-action-pypi-publish** | uploads to PyPI with trusted publishing | ratified 2026-10-26 — *"Ratify."* |

## How it was taken

2026-10-26, from `Cargo.toml` of every crate (47 direct crates; 376 external packages in `Cargo.lock`), the transitive counts from `Cargo.lock`'s graph, `packaging/vscode/package.json`, `packaging/python/pyproject.toml`, the npm manifests `scripts/package.py` generates, `.github/workflows/*.yml`, and the imports of `scripts/` and `.claude/hooks/`. W-65 turns this into a CI check.
