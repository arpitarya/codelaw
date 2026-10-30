# port-rule fixture — three rules ported on demand (W-140)

What the `codelaw-port-rule` skill writes, kept green in CI. Each rule was ported on 2026-10-30 by
following the skill: step 2 found no codelaw rule whose `ported_from:` names the source; the source's
public documentation was read (nothing was run); the rule was scaffolded with `codelaw new-rule`, given
`ported_from:` and `stability: preview`, and its examples and cases were written fresh.

| source rule | licence | codelaw rule | fix | `codelaw test` |
|---|---|---|---|---|
| ESLint core `no-nested-ternary` | MIT | `acme/no-nested-ternary` (JS, JSX, TS, TSX) | `manual` — ESLint has no autofix | 22 checks, 0 failed |
| Pylint `global-statement` (W0603) | GPL-2.0: no text or test of its taken | `acme/no-global-statement` (Python) | `manual` | 9 checks, 0 failed |
| stylelint `declaration-no-important` | MIT | `acme/no-important` (CSS, SCSS) | `manual` — no autofix; removing it changes the cascade | 12 checks, 0 failed |

`acme/no-important` has one gap, under its `## Differences from the source`: `! important` written
apart does not parse as a declaration in codelaw's CSS grammar.

`codelaw.toml` turns on `codelaw:css/recommended`, so the step-2 lookup also has a first-party rule to
find: `css/declaration-block-no-duplicate-properties`, whose card carries
`ported_from: [stylelint/declaration-block-no-duplicate-properties]`. `src/` holds one violation of each
port.

`crates/codelaw-cli/tests/port_rule.rs` copies this folder, with `packs/css` as
`node_modules/@codelaw/rules-css`, into a temporary folder and runs:

| command | expect |
|---|---|
| `codelaw rules lint` | 0 problems |
| `codelaw test <id>`, for each port | 0 failed |
| `codelaw check --rule <id>` | the one file in `src/` for that rule |
| `codelaw rules --format json` | each port's card carries its `ported_from`, and so does the css rule's |
| MCP `list_rules` | the same cards |

Not this repo's own lint config: `acme/` is a project namespace, as a consumer's would be.
