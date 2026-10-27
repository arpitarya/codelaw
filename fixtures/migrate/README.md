# fixtures/migrate — three repos `codelaw migrate` is tested on (W-94)

Each folder is a project as another tool configures it. `expected.txt` lists
what that tool reports on the rules codelaw ports, one `file:line tool-rule
codelaw-rule` per line, from the tool's own documentation of each rule; and
`unmapped.txt` the source settings the report must name as not mapped.
`crates/codelaw-cli/tests/migrate.rs` copies each folder, installs the packs,
runs `codelaw migrate <tool> --write` and `codelaw check`, and compares.

| folder | source config | what it covers |
|---|---|---|
| `eslint-prettier/` | a static flat `eslint.config.js` and `.prettierrc` | a preset (`js.configs.recommended`), rule options, a `files` group, `eslint-disable-next-line` |
| `biome/` | `biome.json` with `recommended: false` | rules by Biome name, a rule with no ESLint source, `biome-ignore`, the formatter keys |
| `ruff/` | `[tool.ruff]` in `pyproject.toml` | prefix selection, `ignore`, `per-file-ignores`, `# noqa` |
