# @codelaw/rules-data · codelaw-rules-data

codelaw's rules for JSON, YAML, TOML and Markdown — one of the
first-party rule packs ([SR-RULE-PACKS](../../records/0101_rule-packs.md)). The
same files ship on npm as `@codelaw/rules-data` and on PyPI as
`codelaw-rules-data`.

```toml
# codelaw.toml
extends = ["codelaw:data/recommended", "codelaw:md/recommended"]
preview = true   # every rule is `preview` until a corpus run promotes it
```

## Presets

<!-- docs-sync:presets -->

| preset | rules | what it holds |
|---|---|---|
| `data/recommended` | 6 | correctness for JSON, YAML and TOML: keys defined twice in each, YAML aliases without an anchor, and the YAML scalars that YAML 1.1 and 1.2 read differently |
| `md/recommended` | 17 | the non-layout rules of markdownlint 0.42.0's default set, and the starter rule `md/relative-links` |
| `data/package-json` | 6 | `json/package-scripts-sorted`, `json/package-deps-sorted`, `json/package-engines`, `json/package-json-schema`, `data/outdated-dependency`, `data/stale-deps-snapshot` — `files: **/package.json` |
| `data/tsconfig` | 2 | `json/tsconfig-strict`, `json/tsconfig-schema` — `files: **/tsconfig.json`, `**/tsconfig.*.json` |
| `data/pyproject` | 5 | `toml/deps-sorted`, `toml/pyproject-requires-python`, `toml/pyproject-schema`, `data/outdated-dependency`, `data/stale-deps-snapshot` — `files: **/pyproject.toml` |
| `data/github-actions` | 3 | `yaml/gha-pin-sha`, `yaml/gha-workflow-schema`, `yaml/gha-action-schema` — workflows and composite actions |
| `data/conventions` | 2 | opt-in house style: `yaml/document-start`, `md/no-inline-html` |
| `data/markers` | 3 | TODO comments with a ticket (and a date when asked) in YAML, TOML and Markdown; opt in (SR-CONFIG decision 11a) |

<!-- /docs-sync:presets -->

42 rules: 8 `auto`, 6 `suggest`, 28 `manual`. Every rule is
**reimplemented**, not vendored: the checks, messages and docs are written
fresh, and each ported rule's `ported_from:` names its source. Rules that need
more than a pattern are native rules in `crates/codelaw-native/src/data_*.rs`;
rules about a path in a JSON, YAML or TOML file are `target:` + `assert:` rules
(SR-RULE-FORMAT decision 3e).

The two `data/*` rules compare a manifest's dependencies with
`.codelaw/deps-snapshot.json`, which `codelaw deps refresh` writes from npm and
the Python index; the rules read only that file and the lockfiles, so `check`
stays offline (SR-RULE-PACKS decision 15).

The starter-catalogue rules are all here under their starter ids:
`json/package-scripts-sorted`, `json/package-engines`, `json/tsconfig-strict`,
`json/no-duplicate-keys`, `yaml/no-duplicate-keys`, `yaml/gha-pin-sha`,
`yaml/document-start`, `toml/pyproject-requires-python`, `toml/deps-sorted`,
`md/single-h1`, `md/heading-increment`, `md/fence-language` and
`md/relative-links`.

## Ported

| rule | from | fix |
|---|---|---|
| `md/heading-increment` | markdownlint MD001 | suggest |
| `md/no-reversed-links` | markdownlint MD011 | auto |
| `md/commands-show-output` | markdownlint MD014 | auto |
| `md/no-duplicate-heading` | markdownlint MD024 | manual |
| `md/single-h1` | markdownlint MD025 | manual |
| `md/no-inline-html` | markdownlint MD033 | manual |
| `md/no-bare-urls` | markdownlint MD034 | auto |
| `md/no-emphasis-as-heading` | markdownlint MD036 | manual |
| `md/fence-language` | markdownlint MD040 | manual |
| `md/first-line-heading` | markdownlint MD041 | manual |
| `md/no-empty-links` | markdownlint MD042 | manual |
| `md/no-alt-text` | markdownlint MD045 | manual |
| `md/link-fragments` | markdownlint MD051 | manual |
| `md/reference-links-images` | markdownlint MD052 | manual |
| `md/link-image-reference-definitions` | markdownlint MD053 | auto |
| `md/table-column-count` | markdownlint MD056 | manual |
| `md/descriptive-link-text` | markdownlint MD059 | manual |
| `md/relative-links` | codelaw's starter catalogue | manual |
| `json/package-json-schema` | SchemaStore `package.json` | manual |
| `json/tsconfig-schema` | SchemaStore `tsconfig.json` | manual |
| `toml/pyproject-schema` | SchemaStore `pyproject.json` | manual |
| `yaml/gha-workflow-schema` | SchemaStore `github-workflow.json` | manual |
| `yaml/gha-action-schema` | SchemaStore `github-action.json` | manual |

The JSON, YAML and TOML rules come from the formats' specifications (RFC 8259,
YAML 1.2.2 and its YAML 1.1 differences, TOML 1.0.0) and the Python packaging
specifications (PEP 503 names, PEP 621 `[project]`). yamllint (GPL-3.0) is not
a source: no code, documentation text, options or messages are taken from it.
One rule name coincides with one of yamllint's — `yaml/document-start` and its
`document-start`: both describe the same `---` marker.

## How the ports differ

| rule | difference |
|---|---|
| `md/heading-increment` | has a fix: it renumbers the outline, so a raised heading's subsections move with it |
| `md/single-h1` | the first level-1 heading anywhere is the title; markdownlint only checks when the file's first line is one |
| `md/no-duplicate-heading` | `siblings_only` defaults to `true` (markdownlint: `false`), so a changelog's repeated `### Fixed` under each version is fine |
| `md/no-bare-urls` | URLs only, not e-mail addresses; `git+https://…` and the URL of a link the parser rejected are left alone |
| `md/no-emphasis-as-heading` | only paragraphs at the top level, and not emphasis wrapped around a link |
| `md/no-alt-text` | Markdown images only; HTML `<img>` is the HTML pack's `html/img-alt` |
| `md/link-fragments` | no fix and no `ignored_pattern` option |
| `md/no-inline-html` | in `data/conventions`, not `md/recommended`: most READMEs use HTML for layout; no `table_allowed_elements` option |
| `md/reference-links-images`, `md/link-image-reference-definitions`, `md/link-fragments` | a definition whose destination starts with `#` counts, although the Markdown grammar reads it as a paragraph |
| `yaml/no-ambiguous-booleans` | `y` and `n` aren't reported: common YAML 1.1 loaders (PyYAML, Psych) read them as strings |
| `json/package-scripts-sorted` | a `scripts` object that uses `npm-run-all`, `run-s` or `run-p` is left alone, as sort-package-json does, because those runners expand `build:*` in written order |
| `json/tsconfig-strict` | a config that `extends` another and doesn't set `strict` is left alone |
| the five `*-schema` rules | a `$ref` to another SchemaStore file accepts any value (nothing is fetched); `oneOf` is read as `anyOf`; a `pattern` the engine's regular expressions can't read (look-around, back-references) accepts any value; `format` isn't checked; a YAML key with no value isn't checked |
| `json/package-deps-sorted` | names compare by code point, not by locale as npm's `localeCompare` does; the two orders differ only where names differ by `_` against `-` or `.`, or by capitals, which new npm names cannot have |

## Not ported, and why

| rule | why |
|---|---|
| markdownlint MD043 `required-headings` | does nothing until configured with a heading list; a later version |
| markdownlint MD044 `proper-names` | does nothing until configured with a list of names; a later version |
| markdownlint MD054 `link-image-style` | every style is allowed by default, so the default set does nothing; a style choice |
| markdownlint MD003, MD004, MD005, MD007, MD009, MD010, MD012, MD013, MD018–MD023, MD026–MD032, MD035, MD037–MD039, MD046–MD050, MD055, MD058, MD060 | **never: layout** — heading and list markers, indentation, spacing, blank lines, line length, trailing punctuation, emphasis, fence and table-pipe style belong to the formatter (L7, SR-FORMATTER decision 3) |

## Upstream versions

- markdownlint **0.42.0** (MIT) — the rule set and documented behaviour ported.
- YAML 1.2.2 and YAML 1.1 (the type repository's `bool`, `int` and base-60 forms) — the ambiguity rules.
- TOML 1.0.0 — `toml/no-duplicate-keys`.
- PEP 503, PEP 621 — `toml/deps-sorted`, `toml/pyproject-requires-python`.
- sort-package-json (MIT) — the `pre`/`post` grouping and the `npm-run-all` exemption of `json/package-scripts-sorted`, and the sorted dependency lists of `json/package-deps-sorted`.
- SchemaStore (Apache-2.0), as served at `json.schemastore.org` on 2026-10-20 — the five schemas in `data/schemas/`.

## Data

| table | from | read by |
|---|---|---|
| `schemas/package` | SchemaStore `package.json` | `json/package-json-schema` |
| `schemas/tsconfig` | SchemaStore `tsconfig.json` | `json/tsconfig-schema` |
| `schemas/pyproject` | SchemaStore `pyproject.json` | `toml/pyproject-schema` |
| `schemas/github-workflow` | SchemaStore `github-workflow.json` | `yaml/gha-workflow-schema` |
| `schemas/github-action` | SchemaStore `github-action.json` | `yaml/gha-action-schema` |

Each is the SchemaStore schema with its annotation-only keywords removed
(`description`, `markdownDescription`, `title`, `default`, `examples`,
`$comment`, `deprecationMessage` and the editor-specific `x-*`, `tsType` and
`allowTrailingCommas` keys) and written without whitespace; every validation
keyword is kept as published. The NOTICE carries SchemaStore's licence.
