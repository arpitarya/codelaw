# The reference workload

The 10,000-file repo L8's budgets are measured on
([SR-LAW-8](../../records/0010_LAW-8-performance-budgets.md) decision 2). It is
**generated, never committed**: `codelaw bench` writes it from a seed, so the
same bytes appear on every machine without 10,000 files in git.

```sh
codelaw bench --workload /tmp/workload --generate-only   # seed 20261018, 10,000 files
codelaw bench --repo <dir> ...                           # also time `check` on a pinned real repo (scripts/corpus.py fetch --repo <name>)
codelaw bench --typescript fixtures/types/node_modules/typescript   # also time check_types_cold / check_types_warm
```

- **The generator** is `crates/codelaw-cli/src/bench/workload.rs`; its digest is
  pinned there (`REFERENCE_DIGEST`), and a test fails when the bytes change.
  A change to them means every budget is re-measured.
- **The mix:** TypeScript 22 %, Python 20 %, JavaScript 13 %, Markdown 10 %,
  TSX 8 %, JSON 8 %, YAML 5 %, CSS 5 %, HTML 4 %, JSX 3 %, TOML 2 %; 25 files a
  folder, each importing a sibling; about one file in nine has a finding.
- **The config** extends every first-party pack's `recommended` presets with
  `preview = true`, and the repo's `packs/` are installed into it as
  `node_modules/@codelaw/rules-*`.
- **The type workload** (`check_types_cold`, `check_types_warm`; W-105) is
  written beside it: the workload's 2,200 `.ts` files under one strict
  `tsconfig.json`, a config extending only `codelaw:ts/type-checked`, and
  `node_modules/typescript` linked to `--typescript` (`npm ci` in
  `fixtures/types` installs the one CI uses). Its digest is pinned by a test
  beside `REFERENCE_DIGEST`.
