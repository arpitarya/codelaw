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

**Rule packs** — on npm as `@codelaw/rules-<name>`: `js` · `ts` · `react` ·
`node` · `stencil` · `html` · `css` · `data`; on PyPI as `codelaw-rules-<name>`:
`js` · `react` · `html` · `css` · `python` · `data`.

## Use

```bash
codelaw check            # lint the project
codelaw fix              # apply the safe fixes
codelaw check --format json
```

**Programmatic API** — runs the bundled binary and returns its JSON report:

```js
import { check, fix } from "codelaw";
const report = check(["src"]);
```

```python
import codelaw
report = codelaw.check(["src"])
```

## Platforms

macOS arm64 and x64 · Linux x64 and arm64, glibc and musl · Windows x64 and
arm64. Any other platform fails with an error naming these — codelaw never
compiles from source at install time.

## Licence

MIT — <https://github.com/arpitarya/codelaw>
