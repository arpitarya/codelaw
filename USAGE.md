# Using codelaw

**The how-to.** What codelaw is, and why: the [README](README.md).

[Install](#install) · [First run](#your-first-run) · [Configure](#configure) · [Write your own rules](#write-your-own-rules) · [With AI agents](#with-ai-agents) · [Without AI](#without-ai-editor-pre-commit-ci) · [Commands](#commands)

---

## Install

### Today: build from source

You need a Rust toolchain (1.97 or newer).

```bash
# 1. Build and install the codelaw binary
git clone https://github.com/arpitarya/codelaw.git
cargo install --path codelaw/crates/codelaw-cli

# 2. In your project: write codelaw.toml, .codelaw/rules/ and the .gitignore line
cd your-project
codelaw init --no-install

# 3. Until the packs are published, link them from the clone:
#    the npm packs into node_modules/, the PyPI-only ones where a Python install would put them
mkdir -p node_modules/@codelaw .codelaw-packs
for p in js ts react node stencil html css scss css-modules data tanstack-query redux-toolkit security-js security-ts security-html spell; do
  ln -s "$OLDPWD/codelaw/packs/$p" "node_modules/@codelaw/rules-$p"
done
for p in python security-python; do
  ln -s "$OLDPWD/codelaw/packs/$p" ".codelaw-packs/codelaw_rules_${p//-/_}"
done
export CODELAW_SITE_PACKAGES="$PWD/.codelaw-packs"

# 4. Opt in to the preview rules, then run
echo 'preview = true' >> codelaw.toml
codelaw check
```

- **`init` detects your languages** and writes the matching `recommended` presets into `codelaw.toml`.
- **Most rules are still `preview`**, so `preview = true` is needed to run them; 42 `css` rules are `stable` since the first promotion (W-100) and run without it.
- **Step 3 is temporary.** codelaw only reads the linked folders, so it works in any repo, Python-only included, and neither Node nor Python is run. `CODELAW_SITE_PACKAGES` is where codelaw looks for the PyPI packs, as an activated `.venv` would tell it. Add `node_modules/` and `.codelaw-packs/` to `.gitignore`.

### After the first release

Planned for the first release, and not published yet:

```bash
npx codelaw init      # Node projects: installs @codelaw/rules-* as dev dependencies
uvx codelaw init      # Python projects: installs codelaw-rules-* with your package manager
```

The npm and PyPI packages will be the same engine and give byte-identical output.

---

## Your first run

```bash
codelaw check                 # every finding, each with its fix
codelaw fix --dry-run         # what the safe fixes would change; writes nothing
codelaw fix                   # apply the auto fixes, then format
codelaw fix --unsafe          # also apply the suggest fixes — review the diff
codelaw format --check        # is every file formatted?
codelaw explain js/use-isnan  # what a rule wants, why, and how to fix it
```

- **`auto` fixes** are safe and meaning-preserving; **`suggest` fixes** can change behaviour, so they wait for `--unsafe`; **`manual` findings** list numbered steps.
- **Every fix is re-parsed** before it is written; a fix that would break the file is rolled back.

---

## Coming from another tool

`codelaw migrate <tool>` reads the old tool's config and prints the `codelaw.toml` that runs the same rules, with a report: **mapped**, **changed** (an option it could not carry, a preset that differs), **unmapped** (no codelaw rule ports it, by name) and **ignored** (a setting codelaw has no key for). Nothing is dropped silently. `--write` writes the file — or, when `codelaw.toml` exists, adds only what it does not set — and `--format json` gives the report as data. It maps through each rule's `ported_from:`, so install the packs first (`codelaw init`); packs that are not installed are listed, not searched. A config that is a program is never run: convert it to JSON first, or list the rules by hand.

### Coming from ESLint

```bash
codelaw migrate eslint            # read it, print the report and the file
codelaw migrate eslint --write
```

- Reads `eslint.config.*` without running it: the literal parts of the exported array (`js.configs.recommended`, `...tseslint.configs.recommended`, `{ files, rules }`); then `.eslintrc.{json,yaml,yml}`, `.eslintrc` and `package.json`'s `eslintConfig`. `.eslintrc.js` is refused.
- `eslint:recommended` becomes `codelaw:js/recommended`; typescript-eslint's `recommended` becomes `codelaw:js/typescript` and `codelaw:ts/recommended`. A `files` group becomes a `[scope.eslint-<n>]`.
- `// eslint-disable-next-line` and `// eslint-disable-line` keep working: the file it writes sets `[lint] ignore_aliases = ["eslint"]`.

### Coming from Prettier

```bash
codelaw migrate prettier --write   # after `migrate eslint --write`, it adds [format] to the same file
```

`printWidth`, `tabWidth`, `useTabs`, `endOfLine`, `singleQuote`, `semi` and `trailingComma` become `[format]` keys; every other option is listed as not imported. Same as `codelaw format --import-config`.

### Coming from Biome

```bash
codelaw migrate biome --write
```

- Each Biome rule maps through the ESLint, typescript-eslint, React or stylelint rule Biome names as its source; one with none is listed as unmapped. Biome's `recommended` becomes `codelaw:js/recommended`, which is ESLint's core and differs rule by rule — the report says so.
- `formatter` and `javascript.formatter` become `[format]`; `// biome-ignore lint/...: reason` keeps working (`ignore_aliases = ["biome"]`).

### Coming from Ruff

```bash
codelaw migrate ruff --write
```

- Reads `ruff.toml`, `.ruff.toml` or `[tool.ruff]`. `select`, `extend-select` and `ignore` expand over the codes codelaw ports (Ruff's default `E4, E7, E9, F` when `select` is unset); `per-file-ignores` becomes a `[scope.ruff-<n>]`; `line-length` becomes `[format.python] line_width`.
- `# noqa` and `# noqa: F401` keep working (`ignore_aliases = ["ruff"]`).

### Coming from stylelint

```bash
codelaw migrate stylelint --write
```

Reads `.stylelintrc{,.json,.yaml,.yml}` or `package.json`'s `stylelint`. `stylelint-config-recommended` becomes `codelaw:css/recommended`, `-standard` adds `codelaw:css/standard`, the SCSS configs add `codelaw:scss/recommended`; `scss/*` rules map to the `scss` pack. Layout rules are the formatter's in codelaw.

Run the old tool and `codelaw check` side by side before removing it; the `codelaw-migrate` agent skill walks the comparison.

---

## Configure

`codelaw.toml` at the repo root. The same keys also work as `[tool.codelaw]` in `pyproject.toml` or `"codelaw"` in `package.json`.

```toml
extends = [
  "codelaw:js/recommended",
  "codelaw:js/typescript",      # the core rules typescript-eslint switches on
  "codelaw:ts/recommended",     # @codelaw/rules-ts, on npm
  "codelaw:py/recommended",     # codelaw-rules-python, on PyPI
  "codelaw:js/conventions",     # opt-in house style
]
preview = true                  # every rule is preview in 0.1.0

[rules]
"js/use-isnan" = "warn"         # change a severity, or "off"

[scope.legacy]                  # a named group of files with its own settings
files = ["src/legacy/**"]
rules = { "ts/no-unused-vars" = "off" }
```

- **Validation is strict.** An unknown key stops the run and names the valid ones (`unknown key 'disable'; valid keys are 'files', 'exclude', 'rules', 'remap'`).
- **Scopes:** `[scope.<name>]` gives a group of files its own rule settings, applied after `[rules]`, in the order written. The old `[[overrides]]` and `[profiles.<name>]` still load, with a warning, for one more minor release.
- **Monorepos:** a nested `codelaw.toml` inherits from the root and overrides it.
- **Team packs:** `extends` also takes `npm:<package>`, `pypi:<distribution>` and `path:<folder>`.
- **Full reference:** [SR-CONFIG](records/0110_config.md).
- **Every rule, one page each:** its options with types, defaults and examples, scopes, profiles, suppression and fix steps — [`docs/rules/`](docs/rules/README.md). For agents: [`llms.txt`](llms.txt), [`llms-full.txt`](docs/rules/llms-full.txt) and the JSON catalogue [`rules.json`](docs/rules/rules.json).

### Warn on a laptop, fail in CI

A **profile** is a named set of `rules` and `scope`s laid over the config. codelaw runs the `ci` profile when the `CI` environment variable is set (and not `false` or `0`), `local` otherwise; `--profile <name>` or `CODELAW_PROFILE` picks one by name. `codelaw hook` and the MCP server run `ci` — an agent gets CI's verdict — and the editor runs `local`.

```toml
[profile.local.rules]
"js/no-console"  = "warn"
"js/no-debugger" = "warn"

[profile.ci.rules]
"js/no-console"  = "error"
"js/no-debugger" = "error"
```

```bash
codelaw check                 # profile local (default): a console.log is a warning, exit 0
CI=true codelaw check         # profile ci (CI=true): the same line is an error, exit 1
codelaw check --profile ci    # a pipeline that doesn't set CI (some Jenkins setups)
```

When the config defines a profile, every report opens with the one in use (`profile ci (CI=true)`), and `codelaw config --print` names it with its source.

### Globs that never error, or never warn

`remap` on a `[scope.<name>]` maps each severity of a finding in those files to another, or to `off`. It applies last — after the baseline — and never to codelaw's own `codelaw/*` diagnostics.

```toml
[scope.scripts]                    # never fails the run
files = ["scripts/**"]
remap = { error = "warn" }

[scope.generated]                  # silent
files = ["**/*.generated.ts"]
remap = { error = "off", warn = "off", info = "off" }

[scope.payments]                   # strict: no warnings, only errors
files = ["src/payments/**"]
remap = { warn = "error" }
```

### One limit for a kind of file

A **scope** names a group of files. `js/recommended` already ships `tests` (`**/*.test.*`, `**/*.spec.*`), `stories` and `config`, and `py/recommended` ships `tests`; a scope of yours with the same name **extends** it — the files stay, your settings win. A scope of your own needs `files`:

```toml
[rules]
"js/max-lines" = { severity = "error", options = { max = 100 } }

[scope.utils]                      # every *.utils.* file: 50 lines
files = ["**/*.utils.ts", "**/*.utils.tsx", "**/*.utils.js", "**/*.utils.jsx"]
rules = { "js/max-lines" = { options = { max = 50 } } }

[scope.tests]                      # the preset's test files: 400 lines
rules = { "js/max-lines" = { options = { max = 400 } } }
```

`codelaw config --scopes` lists every scope with its files and where it is written, in the order they apply; `codelaw config --print --for <file>` names the scope each value came from. A scope name is lowercase letters, digits, `_` and `-`, and `default` is reserved.

### Size limits

Six rules cap how big a file, a function and a parameter list may grow, for JavaScript, TypeScript and Python. They are in no default preset; `codelaw:js/size` and `codelaw:py/size` turn all three of a language on at `warn`.

| rule | limit | default (JS / Python) |
|---|---|---|
| `js/max-lines` · `py/max-lines` | lines per file | 300 / 1000 |
| `js/max-lines-per-function` · `py/max-lines-per-function` | lines per function | 50 / 50 |
| `js/max-params` · `py/max-params` | parameters per function | 3 / 5 |

```toml
extends = ["codelaw:js/size", "codelaw:py/size"]

[rules]
"js/max-lines" = { severity = "error", options = { max = 500, skip_imports = true } }
"js/max-lines-per-function" = { severity = "warn", options = { max = 80, ignore_names = "^(test|it|describe)" } }
"py/max-params" = { severity = "warn", options = { max = 6, max_positional = 4 } }
```

- **What counts:** blank lines and comment-only lines do not (`skip_blank_lines`, `skip_comments`, both on); a line of code with a comment counts as code. Python docstrings are `skip_docstrings` (off for a file, on for a function), and the import block is `skip_imports`. A function's `include_nested` makes an inner function count toward the outer one.
- **What they report:** the measured value and the limit, `renderOrders is 84 lines (max 50)`, and `--format json` and SARIF carry both as `data` (`{ "lines": 84, "max": 50 }`) for a dashboard or a burn-down.
- **An existing codebase:** `codelaw baseline add --rule js/max-lines` keeps today's long files as exceptions, and a new long file fails.
- **Every fix is yours:** splitting a file or a function changes its structure, so all six are `manual`. The rule's page, `codelaw explain js/max-lines`, lists the steps.

**HTML and CSS.** The `html/size` and `css/size` presets are opt-in too: `codelaw:html/size` and `codelaw:css/size` turn on a line limit per file. HTML and CSS have no prior art in stylelint, html-validate or Biome, so the defaults are codelaw's own. The stylesheet limit covers `.css`, `.scss` and CSS Module files; a `<style>` or `<script>` inside a page is part of that page's count, never a file of its own.

| rule | default | options |
|---|---|---|
| `html/max-lines` | 400 | `max`, `skip_blank_lines` (true), `skip_comments` (true, `<!-- -->`), `skip_embedded` (false: lines inside `<script>` and `<style>`) |
| `css/max-lines` | 300 | `max`, `skip_blank_lines` (true), `skip_comments` (true, `/* */` and SCSS `//`), `skip_imports` (false: `@import`, `@use`, `@forward`) |

```toml
extends = ["codelaw:html/size", "codelaw:css/size"]

[rules]
"css/max-lines" = { severity = "warn", options = { max = 200, skip_imports = true } }
```

A line holding code and a comment counts as code. A finding names the first line over the limit and carries the count: `412 lines (max 300; blank lines and comments not counted)`. The fix is manual: split the file.

### Your own sanitizer, your own wrappers

A rule that requires a function at a sink takes it from your config, matched by name and by where it is imported from. The five HTML-sink rules read `sanitizers`; `sanitizers_mode` is `extend` (add to the built-in list, the default) or `replace` (only yours count):

```toml
[rules."sec-js/no-dangerously-set-inner-html".options]
sanitizers      = [{ name = "sanitizeHtml", module = "@acme/security" }]
sanitizers_mode = "replace"      # a direct DOMPurify.sanitize at the sink is now reported
```

An item is a callee path (`"DOMPurify.sanitize"`, a global; `"*.createHTML"`, any receiver) or `{ name, module }`, where `module` is a string or a list. `js/use-instead` and `py/use-instead` hold the rest of the code to a wrapper, anywhere, not only at a sink; `drop_in = true` lets `codelaw fix` swap it in and add the import, and `allow_in` exempts the wrapper's own file:

```toml
[rules."js/use-instead".options.replacements]
"fetch"              = { use = "apiFetch",     from = "@acme/http",     drop_in = true, allow_in = ["src/lib/http.ts"] }
"DOMPurify.sanitize" = { use = "sanitizeHtml", from = "@acme/security", drop_in = true, allow_in = ["src/security/**"] }
"console.log"        = { use = "logger.info",  from = "@acme/log" }
```

Neither follows a value through a variable: `const c = clean(x)` and then `c` at the sink is not a call.

### SCSS and CSS Modules

`.scss` is a CSS dialect: `codelaw check` and `codelaw format` read it, and the `css/*` rules run on it. Install `@codelaw/rules-css` and `@codelaw/rules-scss` (`codelaw init` does, for `.scss` files or `sass` in `package.json`) and extend `codelaw:css/recommended` and `codelaw:scss/recommended`; `codelaw:scss/strict` adds Dart Sass's module migrations (`@import` to `@use`, `map-get()` to `map.get()`, `/` to `math.div()`), naming patterns and a nesting limit. The indented `.sass` syntax and Less are not read.

A stylesheet named `*.module.css` or `*.module.scss` is a CSS Module, and so is one matching `[css] modules`. `@codelaw/rules-css-modules` (npm only) joins its classes with the JavaScript and TypeScript that read them:

```toml
extends = ["codelaw:css/recommended", "codelaw:css-modules/recommended"]

[css]
modules = ["src/**/*.styles.css"]    # besides *.module.css and *.module.scss
modules_convention = "camelCase"     # css-loader's localsConvention: .btn-primary is also styles.btnPrimary
```

`css-modules/no-unused-class` reports a class no file reads (in the stylesheet; `codelaw fix --unsafe` deletes the rule), and `css-modules/no-undefined-class` reports `styles.x` where the module has no `.x` (in the JS or TS file). A module read by a computed name (`styles[name]`), spread, or passed to a function is never reported, and neither is one no file imports ([SR-PROJECT-GRAPH](records/0103_project-graph.md) decision 8).

### Spelling

`codelaw init` adds the spell pack (`@codelaw/rules-spell`, `codelaw-rules-spell`) and `codelaw:spell/recommended`, which fixes known typos in comments, docs, prose strings, names and keys; `--no-spell` leaves it out. Its rules are `preview`, so they run once `preview = true` is set. `codelaw:spell/strict` adds the dictionary check, `spell/unknown-word`. The words a project accepts are in two places, merged and compared case-insensitively:

```toml
[spell]
words = ["kubectl"]                       # inline
words_file = ".codelaw/spell/words.txt"   # the default: one word per line, `#` comments
locale = "en-US"                          # or en-GB; unset accepts both
```

A missing file at the default path is no error; a `words_file` you set must exist. `init` writes the file and fills it with **`codelaw spell learn`**, which runs `spell/unknown-word` over the project (whatever the config turns on) and adds each word it reports, lower case and sorted, after the lines already there. A word that is a known typo or one edit from a common word is listed for review, never added. A second run adds only new words ([SR-SPELL](records/0116_spell.md)).

### Type-aware rules

With TypeScript 7.1 or later installed in your project, the `ts` pack's `ts/type-checked` preset asks **your own TypeScript** about types — a Promise nobody awaits, an `await` on a plain value, a value typed `any` called or read from — and reports TypeScript's own errors as `ts/type-error`. They are `preview` until TypeScript 7.1 is GA:

```toml
# codelaw.toml
extends = ["codelaw:ts/recommended", "codelaw:ts/type-checked"]
preview = true
```

- **What it needs:** Node on `PATH` and `typescript` ≥ 7.1 in `node_modules`, from the file's folder up to the project root — the copy your project compiles with; codelaw installs none. It reads your `tsconfig.json`.
- **When it can't:** without Node, or with no TypeScript or an older one, the type rules are skipped, every other rule runs, and the summary line and `--format json`'s `types.notes` say why.
- **Reproducible:** the summary line and `--format json`'s `types` name the TypeScript version that answered.
- **Not cached:** a file a type rule checks depends on the files it imports, so it is checked again on every run.
- **Hooks never wait:** `codelaw hook` and `hook --pre` use a type program only when a warm process (`codelaw daemon start`) already holds one; otherwise they skip the type rules with a note. `--watch`, the daemon and the editor keep the program loaded.
- **Shown once:** each rule names its typescript-eslint rule in `ported_from:`, so the same finding from oxlint's type-aware mode (below) is shown once, as codelaw's.

The rest of typescript-eslint's type-checked rules are not built yet ([`packs/ts`](packs/ts/README.md#type-checked-rules)).

### Type rules today, through oxlint

oxlint's type-aware mode gives you typescript-eslint's type-checked rules codelaw has not built yet, through the oxlint adapter codelaw already runs. Install the tools in your project, not codelaw's:

```bash
npm add -D oxlint oxlint-tsgolint typescript@7    # tsgolint needs TypeScript 7.0+
```

and turn type-aware mode on in the root `.oxlintrc.json` (or `options.typeAware` in `oxlint.config.ts`), with the rules you want:

```json
{
  "options": { "typeAware": true },
  "rules": { "typescript/no-floating-promises": "error", "typescript/await-thenable": "warn" }
}
```

- **Nothing to set in codelaw:** `codelaw check` and `codelaw fix` run the oxlint in `node_modules/.bin`, which reads that config, so its findings arrive as `adapter/oxlint/typescript/no-floating-promises` with oxlint's severity and position. `--no-adapters` turns them off with the rest.
- **The fix is yours:** oxlint's JSON report carries no edits, so each finding is `manual`: its steps are oxlint's help line and a link to the rule's page on oxc.rs.
- **Not cached:** a type rule's verdict on a file depends on the files it imports, so while a config names `typeAware`, every check runs oxlint again instead of reusing the last result.
- **Not in the hook or editor:** like every adapter, it runs only in `check` and `fix` from the CLI and CI.
- **Limits are oxlint's:** some legacy `tsconfig.json` options, such as `baseUrl`, are not supported by `tsgolint`.
- **Where codelaw has the rule** (`ts/type-checked`, above), it names its typescript-eslint rule in `ported_from:`, so the same finding is shown once, as codelaw's.

### Tests for new code

Every new or edited function in JavaScript, TypeScript or Python must have a unit test, and — given a coverage report — at least 60% line coverage. Functions nobody touched are never judged, so a legacy codebase needs no baseline to start. Opt in:

```toml
extends = ["codelaw:tests"]

[coverage]
report = "coverage/lcov.info"   # or --coverage <path>; codelaw reads lcov itself and never runs your tests
min = 60                        # per function; mode = "patch" judges all new lines together
scope = "all"                   # "public": only exported functions (JS/TS), no leading _ (Python)
```

- **The change set is an input:** `--changed-since origin/main` in CI, `--staged` in the pre-commit hook, and the file an agent just edited in `codelaw hook`. A function is *new* when the base has none of that name and *edited* when its tokens changed — a reformat, a move or a comment outside it is not an edit. Plain `codelaw check` judges nothing and says `new code: no change set; test rules not evaluated`. See what it sees with `codelaw debug changed-functions --changed-since origin/main`.
- **`graph/untested-function`:** no test file reaches the function — a test reaches it by importing it (any alias, through a barrel or `from pkg.mod import f`), or, for a function its module does not export, by importing the module. A string such as `mock.patch("pkg.mod.f")` does not count. It needs no report, so it runs in hooks too.
- **`graph/function-coverage`:** the function's lines in an lcov report (Jest `coverageReporters: ["lcov"]`, Vitest `coverage.reporter: ["lcov"]`, c8, `coverage lcov`, pytest-cov `--cov-report=lcov`) are under `min`. **Generate the report in the same job, before `codelaw check`:** lcov has no content hash, so an old report is only mostly caught — a file it does not name, a line past the end, or a line that is now blank or a comment gives `codelaw/stale-coverage` and no verdict for that file. TypeScript compiled with plain `tsc` and measured by c8 without source maps is not supported; Jest and Vitest write lcov against the `.ts` source.
- **Exemptions live in `.codelawignore`** at the project root, one line each, every line with a reason:

  ```text
  src/utils/http.ts#get                 forever: thin wrapper over fetch, nothing to test
  scripts/*.py                          on_touch=error expires=2027-01-31: one-off migration scripts
  app/billing.py#Invoice.legacy_total   on_touch=error on_touch_scope=function expires=2027-03-01 by=lead@example.com: replaced in ORD-1500
  app/legacy/report.py#build_report     on_touch=warn expires=2027-06-30: migration in progress
  ```

  `on_touch=error` (the default) revokes an exemption when its file is edited — every function it covers in that file is judged again; `warn` brings the findings back as warnings; `keep` keeps it. `on_touch_scope=function` narrows "edited" to that function's own tokens. `forever` never expires and is never revoked. `[coverage.ignore]` sets `require_expiry` and `require_on_touch` (both `true`) and the defaults of a line; a line missing a field is `codelaw/incomplete-ignore` and exempts nothing, and a line that matches nothing is `codelaw/stale-ignore`, which `codelaw fix` removes. `codelaw exceptions` lists every line. A `// codelaw-ignore graph/untested-function: <why>` above one function works too ([SR-PROJECT-GRAPH](records/0103_project-graph.md) decision 9, [SR-BASELINE](records/0104_baseline-and-deprecations.md) decision 12).

---

## Write your own rules

```bash
codelaw new-rule acme/no-console-log --lang typescript,tsx   # write the file from the template
codelaw test acme/no-console-log                             # run it against its own examples
```

The rule file itself is shown in the [README](README.md#a-rule-is-one-markdown-file).

- **Local rules are picked up automatically** from `.codelaw/rules/<namespace>/<name>.md`, with no plugin build or publish step.
- **`codelaw test` checks more than the examples:** every `Don't` must fire and fix to its `Do`, every `Do` must pass, and every fix must be a fixpoint.
- **`codelaw tree`** shows the syntax tree the matcher sees: `codelaw tree -e 'console.log(1)' --lang typescript`.
- **`codelaw rules lint`** checks rule files against the template.
- **Matchers use ast-grep's pattern syntax** (`pattern`, `kind`, `has`, `inside`, `not`, `any`, …), plus codelaw's fix primitives for moving code: `strip-export-keyword`, `collect-into-block` and others. The full format is [SR-RULE-FORMAT](records/0112_rule-format.md).

---

## With AI agents

**Before, while and after the agent writes** — what each needs, and what ships today.

### Before: the agent asks for the rules

- **`codelaw mcp`** is an MCP server over stdio. An agent calls `rules_for_path` before writing, `check_snippet` on a draft, `explain_rule` on a finding, and `format_snippet` for the exact bytes `fix` would write.
- **The same answer from the CLI:** `codelaw rules --for src/math.ts`.
- **Only what is specific to this repo:** `brief_for_task` (MCP) or `codelaw brief --for src/math.ts [--format json]` lists this repo's own rules, the pack rules your config sets options for, the rules with baseline entries or other exceptions under those paths, and the security rules whose fix is `manual` — then one line counting the standard rules left, which the agent already knows from ESLint or Ruff ([SR-CLI-MCP](records/0115_cli-and-mcp.md) decision 14).
- **The agent files come from the skills package**, installed beside codelaw at the same version: `npm i -D @codelaw/skills` or `pip install codelaw-skills` (`uv add --dev codelaw-skills`). It depends on that exact `codelaw`, and codelaw itself carries no agent file; without it, or at another version, `init --agents` and `sync-agents` stop with exit 2 and the install line. `CODELAW_SKILLS=<folder>` points at a copy elsewhere.
- **`codelaw init --agents claude,codex,copilot,kiro,cowork`** installs it with the packs and writes each agent's instruction files, skills, MCP registration and hook from it and your rules; `codelaw sync-agents` rewrites them when the rules change, and `--check` reports drift ([SR-AGENT-SURFACES](records/0102_agent-surfaces.md)).

### While: the edit is checked before it reaches disk

- **`codelaw hook --pre`** is the pre-write hook. It reads the agent's proposed `Write`, `Edit`, `MultiEdit`, `apply_patch`, `write_file` or `replace` from stdin, rebuilds the file in memory exactly as the tool would, and checks it against the file on disk with the `ci` profile. **Only what the edit adds counts:** a new `error` refuses the write, with each rule, line and fix; new warnings are allowed and handed back as context on Claude Code and Codex.
- **It fails open.** A payload it cannot read, a broken config, a crash or a run past `--timeout-ms` (5,000) lets the write through with one line on stderr and exit 0. Shell writes (`sed -i`, `>`) skip every pre-write hook; the post-write hook below stays the backstop.
- **Every agent with a pre-write hook:** `--agent claude|codex|copilot|vscode|kiro|cursor|gemini|windsurf` picks the answer's shape (read off the payload when omitted). `codelaw init --agents` writes it for Claude Code (`PreToolUse` in `.claude/settings.json`, which Cursor and VS Code also read), the Cowork plugin, and Codex's printed snippet; Copilot's and Kiro's wait for a confirmed payload ([SR-AGENT-SURFACES](records/0102_agent-surfaces.md) decision 9).

### After: every edit is checked and fixed

```mermaid
flowchart LR
    A["agent edits a file"] --> H["hook runs<br/>codelaw fix --format agent"]
    H -->|"clean, or auto-fixed"| D["done"]
    H -->|"findings left"| F["one line per finding,<br/>with its fix, sent back"]
    F --> A
```

<details>
<summary><b>ASCII version</b></summary>

```text
   agent edits a file --> hook: codelaw fix --format agent --> clean / auto-fixed --> done
                                       |
                                       +--> findings left --> one line each, with its fix --> back to the agent
```

</details>

**Works today with Claude Code.** Add this to `.claude/settings.json`:

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Edit|Write|MultiEdit",
        "hooks": [
          {
            "type": "command",
            "command": "jq -r '.tool_input.file_path' | xargs codelaw fix --format agent 1>&2 || exit 2"
          }
        ]
      }
    ]
  }
}
```

- **`auto` fixes are applied silently**, so the file on disk is already correct.
- **What's left goes back to the agent** (exit code 2), one line per finding with its fix:

  ```text
  src/math.ts:8:5 js/use-isnan Nothing equals NaN, not even NaN; use Number.isNaN(x) — fix: manual — Replace x === NaN or x == NaN with Number.isNaN(x). …
  ```

- **The same command works in any agent with post-edit hooks** (Codex, Copilot, Kiro), and in pre-commit and CI.
- **`codelaw hook`** is that recipe built in: it reads the agent's hook payload on stdin, fixes, and returns what is left. `codelaw init --agents claude,codex,copilot,kiro,cowork` installs the hooks and writes each agent's instruction files from the rules; `codelaw sync-agents` rewrites them when the rules change, and `--check` reports drift ([SR-AGENT-SURFACES](records/0102_agent-surfaces.md)).
- **An agent that runs `codelaw check` or `codelaw fix` itself gets `--format agent` with no flag.** codelaw sees the variable the agent sets in the commands it runs: `CLAUDE_CODE_CHILD_SESSION` or `CLAUDECODE` (Claude Code; `CLAUDECODE` only when the output is not a terminal, since Claude Code's IDE extensions set it in the terminal you type in), `CODEX_THREAD_ID`, `CODEX_SANDBOX` or `CODEX_SANDBOX_NETWORK_DISABLED` (Codex) and `GEMINI_CLI` (Gemini CLI). `--format` always wins; `CODELAW_FORMAT=json` (or any format) sets the default for every run; `CI` alone never picks `agent`. `codelaw config --print` shows which applied and why. Cursor and Copilot CLI are not detected yet: pass `--format agent` there ([SR-CLI-MCP](records/0115_cli-and-mcp.md) decision 4a).

---

## Without AI: editor, pre-commit, CI

- **Editor:** `codelaw lsp` is a language server — diagnostics, quick fixes and formatting. The VS Code extension is built; its Marketplace listing comes later ([SR-EDITOR](records/0114_editor.md)).
- **Pre-commit:** `codelaw init --git-hooks` installs a `pre-commit` and a `post-commit` hook in the folder git uses (`core.hooksPath` or `.git/hooks`). An existing hook is kept as `<name>.pre-codelaw` and runs first; nothing is overwritten, and deleting the file uninstalls it ([SR-CLI-MCP](records/0115_cli-and-mcp.md) decision 11).
  - **`pre-commit` blocks:** it runs `codelaw fix --staged` on the files that are fully staged and re-stages them, then `codelaw check --staged` on every staged file. A partly staged file is checked but never fixed, so your unstaged hunks stay yours. Add flags to both with `CODELAW_HOOK_ARGS` (for example `--profile ci`); skip once with `git commit --no-verify`.
  - **`post-commit` never blocks:** it reminds you of your exceptions that expire within 7 days (`codelaw exceptions --mine --expiring 7d`) and the baseline's 30-day burn-down in one line, then prints what the scripts in `.codelaw/hooks/post-commit.d/` print, and always exits 0.
  - **By hand:** `codelaw check --staged`.
  - **The pre-commit framework** — `codelaw` must be on `PATH`:

    ```yaml
    repos:
      - repo: https://github.com/arpitarya/codelaw
        rev: v0.2.0
        hooks:
          - id: codelaw-fix
          - id: codelaw-check
    ```

  - **husky** (`.husky/pre-commit`) and **lefthook** (`lefthook.yml`):

    ```sh
    npx codelaw check --staged -- $(git diff --cached --name-only --diff-filter=ACMR)
    ```

    ```yaml
    pre-commit:
      commands:
        codelaw:
          glob: "*.{js,jsx,ts,tsx,py,json,yaml,yml,toml,md,css,html}"
          run: codelaw check --staged -- {staged_files}
          stage_fixed: true
    ```
- **In your dev server:** `@codelaw/vite-plugin` (`plugins: [codelaw()]`), `@codelaw/webpack-plugin` (`new CodelawPlugin()`, also in Rspack) and `@codelaw/next` (`export default withCodelaw(nextConfig)`) show codelaw's findings where you already look — errors in the overlay, warnings in the terminal — and make `vite build`, `webpack` and `next build` fail on an error. Each runs the `codelaw` binary your project already has; options `profile`, `buildProfile`, `failOnError`, `paths`, `binary` ([SR-PACKAGES](records/0107_packages.md) decision 3a).
- **While you work:** `codelaw check --watch` checks once, then again every time a file is saved — the saved file, and through the project rules every file that depends on it — until Ctrl-C. Editing `codelaw.toml`, a rule or the baseline takes effect on the next run, with no restart. It runs the `local` profile unless `--profile` says otherwise. `--format json` prints one JSON object per run, one per line, for tools that read it; `codelaw fix --watch` fixes each file as you save it, which can fight an editor that also fixes or formats on save ([SR-CLI-MCP](records/0115_cli-and-mcp.md) decision 2).
- **CI:** `codelaw check --max-warnings 0` fails the job on any finding; `--changed-since origin/main` checks only what a branch touched; `--format sarif` feeds GitHub code scanning. A CI job runs the `ci` profile when `CI` is set, as GitHub Actions, GitLab CI and most runners do; where it is not, pass `--profile ci` ([Configure](#warn-on-a-laptop-fail-in-ci)). To show findings inline without one, pick the format your CI reads (every one keeps the exit code):
  - **GitHub Actions:** `codelaw check --format github` — annotations on the PR diff. Or codelaw's own Action, which installs codelaw at its tag's version and runs exactly that (`codelaw init --ci github` writes `.github/workflows/codelaw.yml` with it):

    ```yaml
    - uses: actions/checkout@v5
    - uses: arpitarya/codelaw@v<version> # installs codelaw <version>
      with:
        sarif: true # also upload to code scanning; the job needs `permissions: security-events: write`
    ```

    Inputs: `version`, `registry` (`npm` or `pypi`), `working-directory` (where `codelaw.toml` is; paths are rewritten to the repository root), `args` (more `check` arguments), `changed-since` (the base commit on a pull request by default; `none` turns it off), `sarif`, `upload-sarif`, `fail-on-error`; outputs `exit-code` and `sarif-file`. Rule packs come from your repository's dependencies, so install them in a step before it ([SR-PACKAGES](records/0107_packages.md) decision 15).
  - **GitLab:** `codelaw check --format gitlab > gl-code-quality-report.json`, then `artifacts: reports: codequality: gl-code-quality-report.json` — the MR widget.
  - **Bitbucket Pipelines:** `codelaw check --format bitbucket` — the Code Insights report and its annotations, sent with `curl`; [the step to copy](#bitbucket-pipelines) below.
  - **Jenkins, Azure DevOps:** `codelaw check --format junit > codelaw-junit.xml` and publish it as test results.
  - **Jenkins (Warnings Next Generation), SonarQube generic import, Azure DevOps:** `codelaw check --format checkstyle > checkstyle.xml`.
  - `codelaw format --check --format github` (or any of the five) reports the files that would change.
- **Adopting it in an old repo:** grandfather a rule's existing findings with a deadline — `codelaw baseline add --rule <id> --expires <date> --reason "<why>"` ([SR-BASELINE](records/0104_baseline-and-deprecations.md)).
- **Exceptions with an expiry date:** a baseline entry, a deprecation or an inline ignore can carry `expires` — the first day it no longer holds. From `warn_days` before it (3 by default) it is a warning, `codelaw/expiring-exception`; on the day it is an error, `codelaw/expired-exception`, and the findings it covered come back. Set `[exceptions] require_expiry = true` in `codelaw.toml` to make the date mandatory. An optional `added_by` email (`by=` inline) is checked against git under `--staged` (your `git config user.email`) and `--changed-since` (the line's blame author):

  ```ts
  // codelaw-ignore js/no-console expires=2026-11-15 by=you@example.com: needed until the logger lands
  console.log(state);
  ```

  The day is the as-of date — `HEAD`'s commit date — not the wall clock, so a branch with no new commit does not start failing on the day by itself. A scheduled CI job passes `codelaw check --as-of "$(date +%F)"`. `until`, the last allowed day, still works for one release and reads as `expires` the day after ([SR-BASELINE](records/0104_baseline-and-deprecations.md) decision 9).
- **Old uses warn, new ones fail:** leave out `--expires` and pass `--warn-from <today> --on-touch keep`. Each file's existing findings stay warnings with no deadline; a new file with the finding, or one more finding in an old file, is an error. `react/no-context` is the worked example — turn it on, then:

  ```bash
  # codelaw.toml: [rules] "react/no-context" = "error"
  codelaw baseline add --rule react/no-context \
    --warn-from 2026-10-25 --on-touch keep --reason "legacy context, grandfathered"
  ```

- **Dependencies on their latest version:** `codelaw deps refresh` reads npm and the Python index for every dependency your `package.json` and `pyproject.toml` files name, and writes `.codelaw/deps-snapshot.json` — commit it. It is the only command that uses the network ([L5](records/0007_LAW-5-no-model-in-the-engine.md) decision 3). Then `check` reports each dependency behind its latest version from the snapshot, offline: a major version behind is a warning, a minor or patch one is info, a deprecated or yanked version in use is an error, and a lockfile behind a range that already allows the latest is info. `codelaw fix --unsafe` moves the range to the latest and keeps its `^`, `~`, `>=` or `==`. Turn the rules on with the `data/package-json` and `data/pyproject` presets (and `preview = true`), and tune them in `[deps]`:

  ```toml
  [deps]
  severity        = { major = "warn", minor = "info", patch = "off" }
  min_release_age = "7d"                  # ignore releases younger than a week
  ignore          = ["npm:typescript"]    # or { package = "npm:react", expires = "2027-01-01" }
  ```

  A private npm registry comes from `.npmrc` (`@acme:registry=…` and `//host/:_authToken=${NPM_TOKEN}`, the token read from the environment, never written); a private Python index from `PIP_INDEX_URL` or `[[tool.uv.index]]`. `data/stale-deps-snapshot` says when the snapshot is older than `[deps] max_snapshot_age` (14 days) on the as-of date ([SR-RULE-PACKS](records/0101_rule-packs.md) decision 15).

### Bitbucket Pipelines

Bitbucket has no Actions, and codelaw writes no Bitbucket file: copy this step into `bitbucket-pipelines.yml` at the repository root. It installs codelaw at the version it pins (change `0.3.0` to the one you use; `--format bitbucket` is new in 0.3.0), checks every push and pull request, fails the build on an error, and shows each finding on the pull request as a Code Insights annotation — the report `codelaw` on the commit, sent with `curl` through the pipeline's own proxy, so no token. On a pull request the files it changed count as touched (`--changed-since` its destination branch). Delete `default:` to check pull requests only.

```yaml
image: node:22

definitions:
  steps:
    - step: &codelaw
        name: codelaw check
        clone:
          depth: full
        script:
          - export CODELAW_CODE_INSIGHTS="${CODELAW_CODE_INSIGHTS:-true}"
          - export CODELAW_SARIF="${CODELAW_SARIF:-false}"
          - export CODELAW_FAIL_ON_ERROR="${CODELAW_FAIL_ON_ERROR:-true}"
          - export CODELAW_DIR="${CODELAW_DIR:-.}"
          - if [ -f pnpm-lock.yaml ]; then corepack enable && pnpm install --frozen-lockfile; elif [ -f yarn.lock ]; then corepack enable && yarn install --frozen-lockfile; elif [ -f package-lock.json ]; then npm ci; fi
          - npm install --global --no-audit --no-fund codelaw@0.3.0
          - |
            set -u
            cd "$CODELAW_DIR"
            prefix="${CODELAW_DIR#./}"; prefix="${prefix%/}"
            flags=""
            ref="${CODELAW_CHANGED_SINCE:-}"
            dest="${BITBUCKET_PR_DESTINATION_BRANCH:-}"
            if [ -z "$ref" ] && [ -n "$dest" ]; then
              git fetch --no-tags --quiet origin "+refs/heads/$dest:refs/remotes/origin/$dest" || true
              ref="origin/$dest"
            fi
            if [ -n "$ref" ] && [ "$ref" != none ]; then
              if git merge-base HEAD "$ref" > /dev/null 2>&1; then flags="--changed-since $ref"; else echo "codelaw: no merge base with $ref; ran without --changed-since"; fi
            fi
            rc=0
            codelaw check $flags ${CODELAW_ARGS:-} || rc=$?
            if [ "$CODELAW_SARIF" = true ] && [ "$rc" -lt 2 ]; then
              src=0
              codelaw check --format sarif $flags ${CODELAW_ARGS:-} > "$BITBUCKET_CLONE_DIR/codelaw.sarif" || src=$?
              if [ "$src" -ge 2 ]; then echo "codelaw: the SARIF run exited $src"; exit "$src"; fi
              echo "codelaw: wrote codelaw.sarif (paths relative to $CODELAW_DIR)"
            fi
            if [ "$CODELAW_CODE_INSIGHTS" = true ] && [ "$rc" -lt 2 ]; then
              insights="$BITBUCKET_CLONE_DIR/codelaw-insights.jsonl"
              irc=0
              codelaw check --format bitbucket $flags ${CODELAW_ARGS:-} > "$insights.raw" || irc=$?
              if [ "$irc" -ge 2 ]; then echo "codelaw: the Code Insights run exited $irc"; exit "$irc"; fi
              # Bitbucket places an annotation by its path from the repository root.
              if [ -z "$prefix" ] || [ "$prefix" = . ]; then mv "$insights.raw" "$insights"; else sed "s#\"path\":\"#\"path\":\"$prefix/#g" "$insights.raw" > "$insights"; rm "$insights.raw"; fi
              api="http://api.bitbucket.org/2.0/repositories/$BITBUCKET_WORKSPACE/$BITBUCKET_REPO_SLUG/commit/$BITBUCKET_COMMIT/reports/codelaw"
              # Line 1 is the report; each line after it is one batch of at most 100 annotations.
              curl -s -o /dev/null --proxy http://localhost:29418 --request DELETE "$api" || true
              head -n 1 "$insights" | curl -sf -o /dev/null --proxy http://localhost:29418 --request PUT "$api" --header "Content-Type: application/json" --data-binary @- || { echo "codelaw: the Code Insights report upload failed"; exit 2; }
              tail -n +2 "$insights" | while IFS= read -r batch; do
                printf '%s' "$batch" | curl -sf -o /dev/null --proxy http://localhost:29418 --request POST "$api/annotations" --header "Content-Type: application/json" --data-binary @- || { echo "codelaw: the Code Insights annotation upload failed"; exit 2; }
              done || exit 2
              echo "codelaw: uploaded the Code Insights report \`codelaw\` for $BITBUCKET_COMMIT"
            fi
            if [ "$rc" -ge 2 ] || { [ "$rc" = 1 ] && [ "$CODELAW_FAIL_ON_ERROR" = true ]; }; then exit "$rc"; fi
        artifacts:
          - codelaw.sarif

pipelines:
  default:
    - step: *codelaw
  pull-requests:
    '**':
      - step: *codelaw
```

The switches are the step's `export` lines: edit them there, or set a repository variable of the same name (*Repository settings → Repository variables*), which wins.

| variable | default | what it does |
|---|---|---|
| `CODELAW_CODE_INSIGHTS` | `true` | upload the Code Insights report and its annotations; `false` checks only, and the build still fails on an error |
| `CODELAW_SARIF` | `false` | `true` also writes `codelaw.sarif`, kept as a build artifact |
| `CODELAW_FAIL_ON_ERROR` | `true` | `false`: the step passes when codelaw finds errors (a usage or config error still fails) |
| `CODELAW_DIR` | `.` | the project folder, where `codelaw.toml` is; annotation paths are rewritten to the repository root |
| `CODELAW_ARGS` | — | more `codelaw check` arguments, split on spaces: `--max-warnings 0` |
| `CODELAW_CHANGED_SINCE` | — | the ref for `--changed-since`; empty: the destination branch on a pull request; `none`: off |

The rule packs come from the repository's own dependencies, so the step installs them first: npm packs land in `node_modules`; install PyPI packs into `.venv/` in that line (`python3 -m venv .venv && .venv/bin/pip install …`). Code Insights holds 1000 annotations per report; past that the report says how many it left out, errors first. codelaw's tests take this block and run its script, so the step shown is the step proven ([SR-CLI-MCP](records/0115_cli-and-mcp.md) decision 12b).

---

## Commands

<!-- docs-sync:commands -->

| command | what it does |
|---|---|
| `codelaw check [paths]` | Report every finding with its fix. `--format text\|json\|sarif\|agent\|github\|gitlab\|junit\|checkstyle` (default: `CODELAW_FORMAT`, else `agent` inside a coding agent, else `text`), `--max-warnings N`, `--lang`, `--rule`, `--timings`; `--no-adapters` skips the installed oxlint, Ruff, stylelint and html-validate; `--watch` checks again on every save until Ctrl-C |
| `codelaw fix [paths]` | Apply `auto` fixes, format, then report what's left. `--unsafe` adds `suggest` fixes; `--dry-run` writes nothing; `--trace` shows each pass; `--watch` fixes each file again when you save it |
| `codelaw format [paths]` | Format every file: Prettier-style JS/TS/JSON/CSS/HTML/YAML/Markdown, Black-style Python, taplo-style TOML. `--check` and `--diff` change nothing |
| `codelaw mcp` · `codelaw lsp` | The MCP server for agents and the language server for editors, from the same binary |
| `codelaw init` | Detect languages and write `codelaw.toml`, `.codelaw/rules/` and the `.gitignore` line. `--no-install` prints the pack install commands instead of running them; `--registry npm\|pypi`; `--agents` writes the agent hooks and files; `--git-hooks [pre-commit,post-commit]` installs the git hooks; `--ci github` writes the check workflow (codelaw's GitHub Action), `--ci github\|gitlab --autofix daily\|weekly [--unsafe]` the scheduled auto-fix PR |
| `codelaw config` | `--print` every resolved value and where it came from; `--schema` prints the config's JSON Schema; `--for <file>` resolves for one file |
| `codelaw rules` | List active rules. `--for <file>` shows the ones that apply to that file. `--tag cwe:79` / `--tag owasp:A05` / `--tag security` keep the rules with that security mapping; `--security-coverage` prints the OWASP Top 10:2025 × rule matrix. `rules lint` checks rule files |
| `codelaw explain <id>` | What a rule requires, why, and how to fix it |
| `codelaw test [rule\|folder] [--corpus <dir>]` | Run rules against their own examples; `--corpus` also runs them over real code |
| `codelaw new-rule <ns/name>` | Write a rule file from the template |
| `codelaw tree <file>` | Show the syntax tree the engine sees |
| `codelaw match <id> <file>` | Which clause of a rule matched or failed, and what it captured. `--explain` shows every candidate node |
| `codelaw graph refs\|why-unused <symbol>` | Query the project graph: every use of a symbol, or why `graph/no-unused-exports` held |
| `codelaw baseline add\|prune\|check` | Grandfather a rule's existing findings with a deadline, lower the counts, or fail when the baseline got looser ([SR-BASELINE](records/0104_baseline-and-deprecations.md)) |
| `codelaw exceptions` | List every exception (baseline entry, deprecation, inline ignore) with its expiry, days left and owner: `--expiring 30d`, `--expired`, `--mine`, `--rule`, `--path`, `--group-by added_by\|rule\|file\|expires-month`, `--format text\|json\|csv\|markdown`, `--fail-if-expiring 7d` ([SR-BASELINE](records/0104_baseline-and-deprecations.md) decision 11) |
| `codelaw deprecate <symbol>` | Deprecate a symbol: write the registry entry and baseline every current use, for `graph/no-deprecated-usage` |
| `codelaw hook [paths]` | Check and fix the files an agent just edited; what is left goes back to it |
| `codelaw hook --pre [--agent <name>]` | Judge the write an agent proposes before it reaches disk; refuse it when it adds an error. Fails open |
| `codelaw sync-agents` | Write every agent's instruction files from the rules and skills; `--check` reports drift |
| `codelaw daemon start\|stop\|status` | An opt-in warm process that `--prefer-warm` runs use; the output is the same |
| `codelaw deps refresh` | Read npm and the Python index for every dependency the manifests name and write `.codelaw/deps-snapshot.json`, which `data/outdated-dependency` reads offline; the only command that uses the network. `--as-of` sets the `fetched` day |
| `codelaw cache stats\|clean` | Inspect or clear the content-addressed cache in `.codelaw/cache/` |
| `codelaw bench` | Time every performance-budgeted operation on a generated 10,000-file workload; `--budgets perf/budgets.toml` fails on a budget that's been set ([L8](records/0010_LAW-8-performance-budgets.md)) |
| `codelaw debug` | Inspect what the engine sees: `changed-functions` |
| `codelaw brief` | What is specific to this repo for the files an agent is about to write (SR-CLI-MCP decision 14) |
| `codelaw migrate` | Translate an ESLint, Prettier, Biome, Ruff or stylelint config into codelaw.toml, with a report of what mapped and what did not (SR-CLI-MCP decision 14) |
| `codelaw spell` | The spell pack's words file: `learn` adds the words no dictionary knows (SR-SPELL) |

<!-- /docs-sync:commands -->

**Exit codes:** `0` clean · `1` findings remain · `2` config or rule error.
