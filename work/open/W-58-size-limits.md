---
type: Handoff
name: W-58
description: "Arpit, 2026-10-26: size limits for JS, TS and Python — max lines per file, max lines per function and max params — each with configurable options."
item: W-58
filed: 2026-10-26
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-26
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-58 — max lines per file, max lines per function, max params (JS, TS, Python)

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Write the six rule pages (below) with options, examples and cases | the spec | ⏳ |
| 2 | Implement them as `native:` rules in `codelaw-native` (SR-ARCHITECTURE decision 15b): line and parameter counting needs no pattern | the rules | ⏳ |
| 3 | Add opt-in presets `codelaw:js/size` and `codelaw:py/size`; not in `recommended` | one line to turn all three on | ⏳ |
| 4 | Put the measured value in JSON/SARIF (`data: { lines: 412, max: 300 }`) | dashboards and burn-down (W-47) can chart it | ⏳ |
| 5 | Docs: USAGE "size limits" section, both packs' READMEs, CHANGELOG | users find it | ⏳ |

**Arpit, 2026-10-26:** *"create a work item for max lines for TypeScript, JavaScript, and Python. There should be options as well to configure the number of lines and anything else that you can figure out would be useful … max lines per function and max params. Let's add those as well."*

**Model:** Sonnet — six counting rules with written options and cases; no judgment left once the pages are written.

## Goal — the six rules

JS rules live in the `js` pack and run on `.js`, `.jsx`, `.ts`, `.tsx` (as `js/no-console` does); Python rules in the `python` pack. Defaults follow ESLint and Pylint so `codelaw migrate` (W-48) maps them one to one.

### `js/max-lines` · `py/max-lines` — lines per file

| option | type | default (JS / Py) | meaning |
|---|---|---|---|
| `max` | int | 300 / 1000 | most lines a file may have |
| `skip_blank_lines` | bool | true / true | blank lines don't count |
| `skip_comments` | bool | true / true | comment-only lines don't count |
| `skip_docstrings` | bool | — / false | Python: docstring lines don't count |
| `skip_imports` | bool | false / false | the import block doesn't count |

Reported once, at the first line over the limit: "412 lines (max 300; blank lines and comments not counted)". Generated files are already skipped (`[files] lint_generated = false`).

### `js/max-lines-per-function` · `py/max-lines-per-function` — lines per function

| option | type | default (JS / Py) | meaning |
|---|---|---|---|
| `max` | int | 50 / 50 | most lines one function may span |
| `skip_blank_lines` | bool | true / true | |
| `skip_comments` | bool | true / true | |
| `skip_docstrings` | bool | — / true | Python |
| `include_nested` | bool | true / true | an inner function's lines count toward the outer one too |
| `include_iife` | bool | false / — | JS: count `(() => { … })()` bodies |
| `ignore_names` | regex | none | function names exempt, e.g. `"^(test|it|describe)"` |

Counts every function form: declarations, expressions, arrows, methods, getters/setters, class static blocks (JS); `def`, `async def` (Python). Reported on the function's name: "`renderOrders` is 84 lines (max 50)".

### `js/max-params` · `py/max-params` — parameters per function

| option | type | default (JS / Py) | meaning |
|---|---|---|---|
| `max` | int | 3 / 5 | most parameters a function may take |
| `max_positional` | int | — / none | Python: a separate cap on positional parameters (Ruff `PLR0917`) |
| `count_this` | bool | false / — | TS: count a `this:` parameter |
| `ignore_self_cls` | bool | — / true | Python: `self`, `cls` don't count |
| `count_varargs` | bool | false / false | `...rest`, `*args`, `**kwargs` don't count |
| `ignore_names` | regex | `"^_"` | parameters named like this don't count |
| `ignore_overrides` | bool | true / true | TS `override` methods; Python `@override`/`@typing.override` — a signature you don't own |
| `ignore_declarations` | bool | true / true | TS overload signatures, `declare`, `.d.ts`, abstract methods; Python `@overload` stubs |
| `ignore_constructor_injection` | bool | false / — | TS: skip a constructor whose parameters are all parameter properties (Angular/NestJS DI) |

A destructured object parameter counts as one. Reported on the function: "`createOrder` takes 6 parameters (max 3)".

### Fixes ([L4](../../records/0006_LAW-4-every-rule-has-a-fix.md))

All six are `fix: manual` — splitting a file or a function changes structure, and no edit is safe without a person. Each page's steps say how: move a cohesive group of exports to a new module; extract a well-named helper; group related parameters into an options object (JS) or a dataclass / `TypedDict` (Python). The skill that carries `manual` fixes quotes them.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | each rule's cases cover blank and comment lines, nested functions, every function form, and every option | ⏳ |
| 2 · | `codelaw test packs/js packs/python` green | ⏳ |
| 3 · | a wrong option type (e.g. `max = "300"`) is a load error, exit 2 | ⏳ |
| 4 · | `codelaw baseline add --rule js/max-lines` freezes today's long files; a new long file errors | ⏳ |
| 5 · | JSON and SARIF carry the measured value and the limit | ⏳ |
| 6 · | L8: `check_warm_10k` unchanged (counting reuses the parsed tree) | ⏳ |

## Blockers

None.

## Record affected

SR-RULE-PACKS (the two `size` presets in the js and python rows). The six rule pages are the rules' spec (SR-RULE-FORMAT).

## Hazards

- **Comment lines in JSX/TSX:** `{/* … */}` is a comment; count it as one.
- **A line holding code and a comment** counts as code.
- **Embedded regions** (a `<script>` in HTML, a fenced code block in Markdown) are not files: `max-lines` applies to the host file only.
- Not in this item, worth a later one: `max-depth` (nesting), `complexity` (cyclomatic), `max-statements`.

## Reference

[ESLint `max-lines`](https://eslint.org/docs/latest/rules/max-lines) · [ESLint `max-lines-per-function`](https://eslint.org/docs/latest/rules/max-lines-per-function) · [ESLint `max-params`](https://eslint.org/docs/latest/rules/max-params) · [typescript-eslint `max-params`](https://typescript-eslint.io/rules/max-params/) · [Pylint `too-many-lines`](https://pylint.readthedocs.io/en/stable/user_guide/messages/convention/too-many-lines.html) · [Ruff `PLR0913`](https://docs.astral.sh/ruff/rules/too-many-arguments/) · [Ruff `PLR0917`](https://docs.astral.sh/ruff/rules/too-many-positional-arguments/)
