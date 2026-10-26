---
type: Registry
description: "L11's list: every third-party dependency codelaw uses, one row per ratification unit, with its status. Nothing outside this list may be used."
governed_by: SR-LAW-11
governed_by_path: records/0013_LAW-11-no-dependencies.md
---

# DEPENDENCIES — every dependency, and whether the decider has ratified it

**Ruled 2026-10-26** (the decider, by voice, one card per unit): 29 ratified; `strsim`, `anyhow` and `postcard` were replaced by codelaw's own code and removed (W-67, 2026-10-26) — `strsim` and `anyhow` still appear in `Cargo.lock` only as part of the ratified trees of `clap` and `markup_fmt`. Each ratification accepts the unit's transitive tree as pinned in `Cargo.lock` / `package-lock.json` that day (L11 decision 4).

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
| D-21 | **similar** | `similar` | diffs (`--diff`, LSP range formatting) | 2.7.0 | 13 | ratified 2026-10-26 — *"Ratify."* |
| D-22 | **semver** | `semver` | version ranges (`requires`, packs, deps) | 1.0.28 | 1 | ratified 2026-10-26 — *"Ratify."* |
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

## The ledger — what the CI check compares (W-65)

`crates/codelaw-cli/tests/dependencies.rs` reads this block. Per unit: `members` are the crates, packages, actions or tools the unit covers (action names lower-case); `tree` is its transitive set as ratified, each package as `name@major` (`name@0.minor` below 1.0) — a patch or minor bump passes, a new package or a new major fails (L11 decision 4). `own` is code codelaw wrote itself, not a dependency. After the decider ratifies a unit again, regenerate with `CODELAW_DEPS_PRINT=1 cargo test -p codelaw-cli --test dependencies print_ledger -- --nocapture` and paste the output over the block. A removed unit's lines go with its row (W-67).

<!-- l11-ledger:begin -->
```text
own cargo option-ext
own python lockmaster
D-01 cargo members biome_css_formatter biome_css_parser biome_formatter biome_js_formatter biome_js_parser biome_json_formatter biome_json_parser biome_languages
D-01 cargo tree addr2line@0.25 adler2@2 aho-corasick@1 allocator-api2@0.2 backtrace@0.3
   biome_aria@0.7 biome_aria_metadata@0.7 biome_console@0.7 biome_css_factory@0.7
   biome_css_formatter@0.7 biome_css_parser@0.7 biome_css_syntax@0.7 biome_db@0.7
   biome_deserialize@0.7 biome_deserialize_macros@0.7 biome_diagnostics@0.7
   biome_diagnostics_categories@0.7 biome_diagnostics_macros@0.7 biome_formatter@0.7 biome_fs@0.7
   biome_js_factory@0.7 biome_js_formatter@0.7 biome_js_parser@0.7 biome_js_syntax@0.7
   biome_json_factory@0.7 biome_json_formatter@0.7 biome_json_parser@0.7 biome_json_syntax@0.7
   biome_languages@0.7 biome_markup@0.7 biome_parser@0.7 biome_rowan@0.7 biome_string_case@0.7
   biome_suppression@0.7 biome_text_edit@0.7 biome_text_size@0.7 biome_unicode_table@0.7 bitflags@2
   boxcar@0.2 bstr@1 camino@1 caseless@0.2 castaway@0.2 cfg-if@1 compact_str@0.10
   crossbeam-channel@0.5 crossbeam-deque@0.8 crossbeam-epoch@0.9 crossbeam-queue@0.3
   crossbeam-utils@0.8 crossbeam@0.8 directories@6 dirs-sys@0.5 drop_bomb@0.1 either@1 enumflags2@0.7
   enumflags2_derive@0.7 equivalent@1 errno@0.3 foldhash@0.1 foldhash@0.2 gimli@0.32 hashbrown@0.15
   hashbrown@0.17 hashlink@0.12 indexmap@2 intrusive-collections@0.10 inventory@0.3 itoa@1 libc@0.2
   libredox@0.1 linux-raw-sys@0.12 lock_api@0.4 memchr@2 miniz_oxide@0.8 object@0.37 once_cell@1
   papaya@0.2 parking_lot@0.12 parking_lot_core@0.9 path-absolutize@3 path-dedot@3
   pin-project-lite@0.2 portable-atomic@1 prettyplease@0.2 proc-macro-error-attr3@3
   proc-macro-error3@3 proc-macro2@1 quote@1 rayon-core@1 rayon@1 redox_syscall@0.5 redox_users@0.5
   regex-automata@0.4 regex-syntax@0.8 rustc-demangle@0.1 rustc-hash@2 rustix@1 rustversion@1
   salsa-macro-rules@0.28 salsa-macros@0.28 salsa@0.28 scopeguard@1 seize@0.5 serde@1 serde_core@1
   serde_derive@1 serde_json@1 similar@2 smallvec@1 static_assertions@1 syn@1 syn@2 syn@3 termcolor@1
   terminal_size@0.4 thin-vec@0.2 thiserror-impl@2 thiserror@2 tinyvec@1 tracing-attributes@0.1
   tracing-core@0.1 tracing@0.1 typeid@1 unicode-bom@2 unicode-ident@1 unicode-normalization@0.1
   unicode-segmentation@1 unicode-width@0.2 winapi-util@0.1 windows-link@0.2 windows-sys@0.61 zmij@1
D-02 cargo members ruff_formatter ruff_python_ast ruff_python_formatter ruff_python_parser ruff_text_size
D-02 cargo tree aho-corasick@1 allocator-api2@0.2 anstream@1 anstyle-parse@1 anstyle-query@1
   anstyle-wincon@3 anstyle@1 anyhow@1 ar_archive_writer@0.5 arc-swap@1 arrayvec@0.7
   attribute-derive-macro@0.11 attribute-derive@0.11 bitflags@2 boxcar@0.2 bstr@1 bumpalo@3 camino@1
   castaway@0.2 cc@1 cfg-if@1 chacha20@0.10 char_str@0.0.4 clap@4 clap_builder@4 clap_derive@4
   clap_lex@1 collection_literals@1 colorchoice@1 compact_str@0.10 countme@3 cpufeatures@0.3
   crc32fast@1 crossbeam-deque@0.8 crossbeam-epoch@0.9 crossbeam-queue@0.3 crossbeam-utils@0.8
   dashmap@6 drop_bomb@0.1 dunce@1 either@1 equivalent@1 filetime@0.2 find-msvc-tools@0.1
   foldhash@0.2 futures-core@0.3 futures-task@0.3 futures-util@0.3 get-size-derive2@0.11
   get-size2@0.11 glob@0.3 globset@0.4 hashbrown@0.14 hashbrown@0.17 hashlink@0.12 heck@0.5
   indexmap@2 interpolator@0.5 intrusive-collections@0.10 inventory@0.3 is-macro@0.3
   is_terminal_polyfill@1 itertools@0.15 itoa@1 js-sys@0.3 libc@0.2 lock_api@0.4 log@0.4
   manyhow-macros@0.14 manyhow@0.14 matchit@0.9 memchr@2 object@0.39 once_cell@1 once_cell_polyfill@1
   ordermap@1 parking_lot@0.12 parking_lot_core@0.9 path-slash@0.2 pathdiff@0.2 phf@0.11
   phf_codegen@0.11 phf_generator@0.11 phf_shared@0.11 pin-project-lite@0.2 portable-atomic@1
   proc-macro-utils@0.10 proc-macro2@1 psm@0.1 quote-use-macros@0.9 quote-use@0.9 quote@1 rand@0.10
   rand@0.8 rand_core@0.10 rand_core@0.6 rayon-core@1 rayon@1 redox_syscall@0.5 regex-automata@0.4
   regex-syntax@0.8 regex@1 ruff_annotate_snippets@0.0.18 ruff_cache@0.0.18 ruff_db@0.0.18
   ruff_diagnostics@0.0.18 ruff_formatter@0.0.18 ruff_macros@0.0.18 ruff_memory_usage@0.0.18
   ruff_notebook@0.0.18 ruff_python_ast@0.0.18 ruff_python_formatter@0.0.18 ruff_python_parser@0.0.18
   ruff_python_trivia@0.0.18 ruff_source_file@0.0.18 ruff_text_size@0.0.18 ruff_unicode_names2@0.0.18
   ruff_unicode_names2_generator@0.0.18 rustc-hash@2 rustversion@1 ryu@1 salsa-macro-rules@0.28
   salsa-macros@0.28 salsa@0.28 scopeguard@1 seahash@4 serde@1 serde_core@1 serde_derive@1
   serde_json@1 shlex@2 similar@3 siphasher@1 slab@0.4 smallvec@1 stacker@0.1 static_assertions@1
   strsim@0.11 supports-hyperlinks@3 syn@2 syn@3 thin-vec@0.2 thiserror-impl@2 thiserror@2 tinyvec@1
   tracing-attributes@0.1 tracing-core@0.1 tracing@0.1 ty_static@0.0.18 typed-path@0.12 typeid@1
   unicode-ident@1 unicode-normalization@0.1 unicode-width@0.2 utf8parse@0.2 uuid@1
   wasm-bindgen-macro-support@0.2 wasm-bindgen-macro@0.2 wasm-bindgen-shared@0.2 wasm-bindgen@0.2
   web-time@1 windows-link@0.2 windows-sys@0.61 zip@8 zmij@1
D-03 cargo members ast-grep-config ast-grep-core ast-grep-language
D-03 cargo tree aho-corasick@1 allocator-api2@0.2 annotate-snippets@0.12 anstyle@1 arraydeque@0.5
   ast-grep-config@0.51 ast-grep-core@0.51 ast-grep-language@0.51 autocfg@1 base64@0.23 bit-set@0.11
   bit-vec@0.10 borsh-derive@1 borsh@1 bstr@1 bytes@1 cc@1 cfg-if@1 cfg_aliases@0.2 core_detect@1
   crossbeam-deque@0.8 crossbeam-epoch@0.9 crossbeam-utils@0.8 dyn-clone@1 encoding_rs@0.8
   encoding_rs_io@0.1 equivalent@1 find-msvc-tools@0.1 foldhash@0.2 globset@0.4 granit-parser@2
   hashbrown@0.17 ignore@0.4 indexmap@2 itoa@1 log@0.4 memchr@2 multiversion_no_op@1
   nohash-hasher@0.2 num-traits@0.2 once_cell@1 ordered-float@2 proc-macro-crate@3 proc-macro2@1
   quote@1 ref-cast-impl@1 ref-cast@1 regex-automata@0.4 regex-syntax@0.8 regex@1 rustversion@1
   same-file@1 schemars@1 schemars_derive@1 scopeguard@1 serde-saphyr@2 serde-value@0.7 serde@1
   serde_core@1 serde_derive@1 serde_derive_internals@0.30 serde_json@1 serde_regex@1 shlex@2
   simdutf8@0.1 smallvec@1 streaming-iterator@0.1 syn@3 thiserror-impl@2 thiserror@2 toml_datetime@1
   toml_edit@0.25 toml_parser@1 toml_writer@1 tree-sitter-css@0.25 tree-sitter-html@0.23
   tree-sitter-javascript@0.25 tree-sitter-json@0.24 tree-sitter-language@0.1 tree-sitter-md@0.5
   tree-sitter-python@0.25 tree-sitter-typescript@0.23 tree-sitter-yaml@0.7 tree-sitter@0.27
   unicode-ident@1 unicode-width@0.2 walkdir@2 winapi-util@0.1 windows-link@0.2 windows-sys@0.61
   winnow@1 zmij@1
D-04 cargo members tree-sitter tree-sitter-language tree-sitter-md tree-sitter-toml-ng
D-04 cargo tree aho-corasick@1 allocator-api2@0.2 cc@1 equivalent@1 find-msvc-tools@0.1 foldhash@0.2
   hashbrown@0.17 indexmap@2 itoa@1 memchr@2 proc-macro2@1 quote@1 regex-automata@0.4
   regex-syntax@0.8 regex@1 serde@1 serde_core@1 serde_derive@1 serde_json@1 shlex@2
   streaming-iterator@0.1 syn@3 tree-sitter-language@0.1 tree-sitter-md@0.5 tree-sitter-toml-ng@0.7
   tree-sitter@0.27 unicode-ident@1 zmij@1
D-05 cargo members cc tree-sitter-css
D-05 cargo tree cc@1 find-msvc-tools@0.1 shlex@2 tree-sitter-css@0.25 tree-sitter-language@0.1
D-06 cargo members taplo
D-06 cargo tree ahash@0.8 aho-corasick@1 allocator-api2@0.2 arc-swap@1 autocfg@1 beef@0.5 bstr@1
   cfg-if@1 countme@3 deranged@0.5 either@1 equivalent@1 fnv@1 foldhash@0.2 getrandom@0.3 globset@0.4
   hashbrown@0.14 hashbrown@0.17 indexmap@2 itertools@0.10 itoa@1 libc@0.2 log@0.4 logos-derive@0.12
   logos@0.12 memchr@2 memoffset@0.9 num-conv@0.2 once_cell@1 pin-project-lite@0.2 powerfmt@0.2
   proc-macro2@1 quote@1 r-efi@5 regex-automata@0.4 regex-syntax@0.6 regex-syntax@0.8 rowan@0.15
   rustc-hash@1 rustversion@1 serde@1 serde_core@1 serde_derive@1 serde_json@1 syn@1 syn@2 syn@3
   taplo@0.14 text-size@1 thiserror-impl@1 thiserror@1 time-core@0.1 time-macros@0.2 time@0.3
   tracing-attributes@0.1 tracing-core@0.1 tracing@0.1 unicode-ident@1 version_check@0.9 wasip2@1
   wit-bindgen@0.57 zerocopy-derive@0.8 zerocopy@0.8 zmij@1
D-07 cargo members pretty_yaml
D-07 cargo tree countme@3 hashbrown@0.14 memchr@2 pretty_yaml@0.6 rowan@0.16 rustc-hash@1 text-size@1
   tiny_pretty@0.2 winnow@0.7 yaml_parser@0.3
D-08 cargo members dprint-core dprint-plugin-markdown
D-08 cargo tree allocator-api2@0.2 bumpalo@3 dprint-core-macros@0.1 dprint-core@0.70
   dprint-plugin-markdown@0.26 equivalent@1 foldhash@0.1 foldhash@0.2 hashbrown@0.15 hashbrown@0.17
   indexmap@2 proc-macro2@1 quote@1 rustc-hash@2 serde@1 serde_core@1 serde_derive@1 syn@1 syn@3
   thiserror-impl@2 thiserror@2 unicode-ident@1 unicode-width@0.2
D-09 cargo members markup_fmt
D-09 cargo tree aho-corasick@1 anyhow@1 css_dataset@0.4 either@1 itertools@0.15 markup_fmt@0.27
   memchr@2 regex-automata@0.4 regex-syntax@0.8 regex@1 tiny_pretty@0.4 unicode-width@0.2
D-10 cargo members serde serde_json
D-10 cargo tree allocator-api2@0.2 equivalent@1 foldhash@0.2 hashbrown@0.17 indexmap@2 itoa@1
   memchr@2 proc-macro2@1 quote@1 serde@1 serde_core@1 serde_derive@1 serde_json@1 syn@3
   unicode-ident@1 zmij@1
D-11 cargo members toml toml_edit
D-11 cargo tree allocator-api2@0.2 equivalent@1 foldhash@0.2 hashbrown@0.17 indexmap@2 memchr@2
   proc-macro2@1 quote@1 serde@1 serde_core@1 serde_derive@1 serde_spanned@1 syn@3 toml@1
   toml_datetime@1 toml_edit@0.25 toml_parser@1 toml_writer@1 unicode-ident@1 winnow@1
D-12 cargo members serde-saphyr
D-12 cargo tree annotate-snippets@0.12 anstyle@1 arraydeque@0.5 autocfg@1 base64@0.23 cfg-if@1
   core_detect@1 encoding_rs@0.8 encoding_rs_io@0.1 granit-parser@2 memchr@2 multiversion_no_op@1
   nohash-hasher@0.2 num-traits@0.2 proc-macro2@1 quote@1 rustversion@1 scopeguard@1 serde-saphyr@2
   serde@1 serde_core@1 serde_derive@1 simdutf8@0.1 smallvec@1 syn@3 unicode-ident@1
   unicode-width@0.2 zmij@1
D-13 cargo members pulldown-cmark
D-13 cargo tree bitflags@2 memchr@2 pulldown-cmark@0.13 unicase@2
D-14 cargo members clap
D-14 cargo tree anstream@1 anstyle-parse@1 anstyle-query@1 anstyle-wincon@3 anstyle@1 clap@4
   clap_builder@4 clap_derive@4 clap_lex@1 colorchoice@1 heck@0.5 is_terminal_polyfill@1
   once_cell_polyfill@1 proc-macro2@1 quote@1 strsim@0.11 syn@3 unicode-ident@1 utf8parse@0.2
   windows-link@0.2 windows-sys@0.61
D-15 cargo members lsp-server lsp-types
D-15 cargo tree allocator-api2@0.2 bitflags@1 crossbeam-channel@0.5 crossbeam-utils@0.8 equivalent@1
   fluent-uri@0.1 foldhash@0.2 hashbrown@0.17 indexmap@2 itoa@1 log@0.4 lsp-server@0.10
   lsp-types@0.97 memchr@2 proc-macro2@1 quote@1 serde@1 serde_core@1 serde_derive@1 serde_json@1
   serde_repr@0.1 syn@3 unicode-ident@1 zmij@1
D-16 cargo members regex
D-16 cargo tree aho-corasick@1 memchr@2 regex-automata@0.4 regex-syntax@0.8 regex@1
D-17 cargo members globset ignore
D-17 cargo tree aho-corasick@1 bstr@1 crossbeam-deque@0.8 crossbeam-epoch@0.9 crossbeam-utils@0.8
   globset@0.4 ignore@0.4 log@0.4 memchr@2 proc-macro2@1 quote@1 regex-automata@0.4 regex-syntax@0.8
   same-file@1 serde_core@1 serde_derive@1 syn@3 unicode-ident@1 walkdir@2 winapi-util@0.1
   windows-link@0.2 windows-sys@0.61
D-18 cargo members rayon
D-18 cargo tree crossbeam-deque@0.8 crossbeam-epoch@0.9 crossbeam-utils@0.8 either@1 rayon-core@1
   rayon@1
D-19 cargo members blake3
D-19 cargo tree arrayvec@0.7 blake3@1 cc@1 cfg-if@1 constant_time_eq@0.4 cpufeatures@0.3
   find-msvc-tools@0.1 libc@0.2 shlex@2
D-20 cargo members postcard
D-20 cargo tree atomic-polyfill@1 byteorder@1 cobs@0.3 critical-section@1 embedded-io@0.4
   embedded-io@0.6 hash32@0.2 heapless@0.7 lock_api@0.4 postcard@1 proc-macro2@1 quote@1
   rustc_version@0.4 scopeguard@1 semver@1 serde@1 serde_core@1 serde_derive@1 spin@0.9
   stable_deref_trait@1 syn@3 thiserror-impl@2 thiserror@2 unicode-ident@1
D-21 cargo members similar
D-21 cargo tree aho-corasick@1 bstr@1 memchr@2 proc-macro2@1 quote@1 regex-automata@0.4
   regex-syntax@0.8 serde_core@1 serde_derive@1 similar@2 similar@3 syn@3 unicode-ident@1
   unicode-segmentation@1
D-22 cargo members semver
D-22 cargo tree semver@1
D-23 cargo members strsim
D-23 cargo tree strsim@0.11
D-24 cargo members anyhow
D-24 cargo tree anyhow@1
D-25 cargo members libc
D-25 cargo tree libc@0.2
D-26 cargo members boon
D-26 cargo tree ahash@0.8 aho-corasick@1 allocator-api2@0.2 appendlist@1 base64@0.22 boon@0.6
   borrow-or-share@0.2 cfg-if@1 displaydoc@0.2 equivalent@1 fluent-uri@0.3 foldhash@0.2
   form_urlencoded@1 getrandom@0.3 hashbrown@0.17 icu_collections@2 icu_locale_core@2
   icu_normalizer@2 icu_normalizer_data@2 icu_properties@2 icu_properties_data@2 icu_provider@2
   idna@1 idna_adapter@1 indexmap@2 itoa@1 libc@0.2 litemap@0.8 memchr@2 once_cell@1
   percent-encoding@2 potential_utf@0.1 proc-macro2@1 quote@1 r-efi@5 ref-cast-impl@1 ref-cast@1
   regex-automata@0.4 regex-syntax@0.8 regex@1 serde@1 serde_core@1 serde_derive@1 serde_json@1
   smallvec@1 stable_deref_trait@1 syn@2 syn@3 synstructure@0.14 tinystr@0.8 unicode-ident@1 url@2
   utf8_iter@1 version_check@0.9 wasip2@1 wit-bindgen@0.57 writeable@0.6 yoke-derive@0.8 yoke@0.8
   zerocopy-derive@0.8 zerocopy@0.8 zerofrom-derive@0.1 zerofrom@0.1 zerotrie@0.2 zerovec-derive@0.11
   zerovec@0.11 zmij@1
D-27 npm members vscode-languageclient
D-27 npm tree balanced-match@1 brace-expansion@2 minimatch@5 semver@7 vscode-jsonrpc@8
   vscode-languageclient@9 vscode-languageserver-protocol@3 vscode-languageserver-types@3
D-28 npm members @types/node @types/vscode typescript
D-28 npm tree @types/node@20 @types/vscode@1 typescript@5 undici-types@6
D-29 python members maturin zig
D-30 action members actions/attest-build-provenance actions/cache actions/checkout actions/download-artifact actions/setup-node actions/setup-python actions/upload-artifact
D-31 action members swatinem/rust-cache
D-32 action members pypa/gh-action-pypi-publish
```
<!-- l11-ledger:end -->
