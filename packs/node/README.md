# @codelaw/rules-node

codelaw's Node.js rules — one of codelaw's first-party rule packs
([SR-RULE-PACKS](../../records/0101_rule-packs.md)). It ships on
npm only, as `@codelaw/rules-node`; PyPI has no copy (L6 decision 11).

```toml
# codelaw.toml
extends = ["codelaw:js/recommended", "codelaw:node/recommended"]
preview = true   # every rule is `preview` until a corpus run promotes it
```

## Presets

<!-- docs-sync:presets -->

| preset | rules | ported from |
|---|---|---|
| `node/recommended` | 10 | eslint-plugin-n 18.5.0 `recommended-module` — 10 of its 14 rules: the 3 that need no `package.json`, `node/hashbang`, which reads the nearest `package.json`'s `bin`, and the 6 import rules, which run on the project graph |

<!-- /docs-sync:presets -->

Every rule is **reimplemented**, not vendored: the matchers, messages and docs
are written fresh, and each rule's `ported_from:` names the source rule.

## How the ports differ

Every rule matches its source's documented behaviour on the source's own
examples. Where codelaw differs, the rule's `## Differences from the source`
section says so. The larger ones:

| rule | difference |
|---|---|
| `node/no-deprecated-api` | the APIs come from the pack's `node/deprecated-api` table; the supported Node.js version is not read |
| `node/no-missing-import`, `node/no-missing-require` | `scope: project` rules on the project graph, which never reads `node_modules`: only the project's own paths are judged (relative, absolute, tsconfig `paths`), and only extensionless or JavaScript/TypeScript ones; a package name is not looked up |
| `node/no-extraneous-import`, `node/no-extraneous-require` | the nearest `package.json` only; a type-only import is satisfied by its `@types/` package |
| `node/no-unpublished-import`, `node/no-unpublished-require` | only in a package that is not `private` and lists `files`; `.npmignore` and the `convertPath` option are not read, so source files that build into `files` are not judged |
| `node/hashbang` | JavaScript and JSX only (no `convertPath` for TypeScript sources); `suggest`, since removing a hashbang can stop a script that is run directly |

## Not ported, and why

| rule | why |
|---|---|
| `node/no-unsupported-features/es-builtins`, `/es-syntax`, `/node-builtins` | need the project's Node.js version from `package.json` and per-version feature tables |
| `node/process-exit-as-throw` | reports nothing; it only changes ESLint's code-path analysis |

## Data

`data/node/deprecated-api.json` — Node.js APIs the Node.js documentation deprecates, with their replacements, written for this pack.
