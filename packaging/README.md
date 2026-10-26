# codelaw

A linter and formatter whose rules are Markdown files — for JavaScript,
TypeScript, Python, JSON, YAML, TOML, Markdown, CSS and HTML, alone or mixed.

**One engine, two registries, identical output.** The same native binary ships
as `codelaw` on npm and on PyPI; neither needs the other.

## Install

| your repo | engine | rules for its languages |
|---|---|---|
| Node | `npm i -D codelaw` | `npx codelaw init` adds the `@codelaw/rules-*` packs |
| Python | `uv add --dev codelaw` or `pip install codelaw` | `codelaw init` adds the `codelaw-rules-*` packs |

`codelaw` installed alone lints nothing until a rule pack is installed — it
says so (`codelaw/no-rules`).

**Rule packs** — each registry carries the packs its users install:

<!-- docs-sync:registries -->

| registry | named | packs |
|---|---|---|
| npm | `@codelaw/rules-<name>` | `js` · `ts` · `react` · `node` · `stencil` · `html` · `css` · `scss` · `css-modules` · `data` · `tanstack-query` · `redux-toolkit` · `security-js` · `security-ts` · `security-html` |
| PyPI | `codelaw-rules-<name>` | `js` · `react` · `html` · `css` · `scss` · `python` · `data` · `security-js` · `security-html` · `security-python` |

<!-- /docs-sync:registries -->

## Use

```bash
codelaw check            # lint the project
codelaw fix              # apply the safe fixes
codelaw check --format json
```

**Programmatic API** — runs the bundled binary and returns its JSON report:

```js
import { check, fix, format, rulesFor } from "codelaw";
const report = check(["src"]);
const rules = rulesFor("src/app.ts");
```

```python
import codelaw
report = codelaw.check(["src"])
rules = codelaw.rules_for("app/main.py")
```

| npm | Python | what it returns |
|---|---|---|
| `check(paths, options)` | `check(paths, …)` | `codelaw check --format json` |
| `fix(paths, options)` | `fix(paths, …)` | `codelaw fix --format json`; `unsafe`, `dryRun` / `dry_run` |
| `format(paths, options)` | `format(paths, …)` | `codelaw format --format json`; `check` writes nothing |
| `rulesFor(path)` | `rules_for(path)` | the rules that apply to one file, as `codelaw rules --for` lists them |
| `binaryPath()`, `platformKey()` | `binary_path()` | the bundled binary, and the platform key it was chosen by |

## Platforms

macOS arm64 and x64 · Linux x64 and arm64, glibc and musl · Windows x64 and
arm64. Any other platform fails with an error naming these — codelaw never
compiles from source at install time.

## Licence

MIT — <https://github.com/arpitarya/codelaw>
