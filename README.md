# codelaw

<div align="center">

**The linter and formatter for code written by people — and by AI.**

**Your rules, before, while and after the AI writes. The same rules with no AI at all.**

JavaScript · TypeScript · React · Stencil · Python · JSON · YAML · TOML · Markdown · CSS · HTML

![status: preview](https://img.shields.io/badge/status-preview-orange)
![version 0.3.0](https://img.shields.io/badge/version-0.3.0-blue)
![rules: 676](https://img.shields.io/badge/rules-676-informational)
![languages: 9](https://img.shields.io/badge/languages-9-informational)
![licence: MIT](https://img.shields.io/badge/licence-MIT-green)
![engine: Rust](https://img.shields.io/badge/engine-Rust-b7410e)
![no AI in the engine](https://img.shields.io/badge/AI%20in%20the%20engine-none-success)

[The story](#the-story) · [Four moments](#one-set-of-rules-four-moments) · [vs ESLint, Prettier, Biome](#if-you-know-eslint-and-prettier--or-biome--you-know-codelaw) · [See it](#see-it) · [Rule packs](#rule-packs) · [Get started](#get-started) · [**Usage guide**](USAGE.md) · [Roadmap](#roadmap) · [FAQ](#faq)

</div>

---

## The story

**It's Tuesday morning. An AI agent opened a pull request overnight.** It compiles, the tests pass, and it breaks four of your team's rules.

- **It used `var` and `== None`.** Your linters flagged both in CI, twenty minutes later, and the agent went round again.
- **It put an `export` in the middle of a file.** That is your team's convention. No linter knows it, so you write the same review comment for the fortieth time.
- **It left a bare `except:` in the Python service.** That half of the repo has its own linter, its own config and its own CI step.
- **Your `CLAUDE.md` said all of this.** The agent read it. Reading is not following.

Your rules live in five tools, five config files and one wiki page — and only some of those places can say no.

**codelaw puts every rule in one Markdown file that people, CI and AI agents all read, and one engine that enforces it.** The agent gets the rules before it writes, each edit is checked before it lands, and anything left is fixed after. Your editor, your pre-commit hook and your CI run the very same check — with no AI anywhere in it.

---

## One set of rules, four moments

| moment | what happens | where it stands |
|---|---|---|
| **Before** the AI writes | it asks what applies to these files and gets the handful of rules that matter here — this repo's own, the ones it configures, the ones this code has got wrong — not a wall of standard ones | ✅ today: the MCP tool `brief_for_task` (or `codelaw brief --for <path>`), `rules_for_path` for the full list, and the skills `codelaw init --agents` installs |
| **While** the AI writes | every edit is checked before it reaches disk; a bad line is refused with the reason and the fix | ✅ today: `codelaw hook --pre`, the pre-write hook of Claude Code, Codex, Copilot, VS Code, Kiro, Cursor, Gemini and Windsurf; it fails open |
| **After** the AI writes | anything that slips through is fixed on save, in the hook and in CI | ✅ today: `codelaw hook`, `codelaw check`, `codelaw fix`, and fix-on-save in the editor extension |
| **Without AI** | a person, an editor, a pre-commit hook or a CI job runs `check`, `fix` and `format` — the way you run ESLint and Prettier, or Biome | ✅ today: the CLI, `codelaw lsp`, `--format sarif` for code scanning |

*no AI inside codelaw — the same verdict on every machine, with or without an agent* ([SR-LAW-5](records/0007_LAW-5-no-model-in-the-engine.md)).

```mermaid
flowchart LR
    R["your rules<br/>one Markdown file each"] --> E["codelaw engine<br/>one binary · no AI"]
    E --> A["AI agents<br/>MCP · hooks"]
    E --> D["editors<br/>codelaw lsp"]
    E --> P["people · pre-commit<br/>codelaw check / fix / format"]
    E --> C["CI<br/>exit code · SARIF"]
```

<details>
<summary><b>ASCII version</b></summary>

```text
   your rules (one .md each) --> codelaw engine (one binary, no AI) --+--> AI agents   (MCP, hooks)
                                                                       +--> editors     (codelaw lsp)
                                                                       +--> people, pre-commit (check / fix / format)
                                                                       +--> CI          (exit code, SARIF)
```

</details>

Every door gets the same answer, because they all call the same engine.

---

> [!IMPORTANT]
> **codelaw is a preview.** 0.3.0 is on npm and PyPI — the engine, the eight platform binaries and every rule pack below, sixteen on npm and eleven on PyPI. The linter, the fixer, the formatter, the MCP server, the language server and all eighteen rule packs work today. The editor extension's Marketplace listing comes later. [Roadmap](#roadmap) lists what is built and what is not.

---

## If you know ESLint and Prettier — or Biome — you know codelaw

**The moves are the ones you already make:** a config file at the root, `check` to report, `fix` to repair, `format` to lay out, fix-on-save in the editor, a non-zero exit in CI. Like Biome, it is one Rust binary that is both the linter and the formatter.

**What changes is where the rules live and who can use them:**

| | the usual stack | codelaw |
|---|---|---|
| **Tools** | ESLint, Prettier, Ruff, stylelint, markdownlint, … | one binary |
| **Config** | one file per tool | one `codelaw.toml` (or `[tool.codelaw]` in `pyproject.toml`, `"codelaw"` in `package.json`) |
| **Languages in one run** | one family per tool | nine: JS, TS, Python, JSON, YAML, TOML, Markdown, CSS (with SCSS), HTML |
| **Your team's own rule** | code against a plugin API, where the tool has one | one Markdown file in `.codelaw/rules/`: pattern, message, fix, examples |
| **What a finding gives you** | a message, sometimes a fix | always a way out: an automatic fix, a diff to review, or numbered steps |
| **A rule's documentation** | kept apart from the code that runs it | the rule file itself, tested against its own examples |
| **AI agents** | read your instructions file | ask codelaw for the rules, and are checked after every edit |

You keep the rules you know: **492 rules are reimplemented from ESLint and its plugins, typescript-eslint, Ruff, Bandit, stylelint, html-validate and markdownlint**, and none of those tools needs to be installed.

**A warning on your laptop, an error in CI** — one config, no second file for the pipeline. codelaw runs the `ci` profile when `CI` is set, `local` otherwise, and an AI agent's hook runs `ci`, so an agent never leaves behind what CI would fail. A glob can be told never to fail the run, or never to warn ([usage guide](USAGE.md#warn-on-a-laptop-fail-in-ci)):

```toml
[profile.local.rules]
"js/no-console"  = "warn"
"js/no-debugger" = "warn"

[profile.ci.rules]
"js/no-console"  = "error"
"js/no-debugger" = "error"

[scope.scripts]                    # never fails the run
files = ["scripts/**"]
remap = { error = "warn" }
```

---

## See it

A TypeScript file and a Python file with a few common mistakes:

```console
$ codelaw check

app/main.py
  1:1  error  Import one module per `import` statement  py/multiple-imports-on-one-line [auto]
  1:8  error  `os` is imported but never used  py/unused-import [suggest]
  1:12  error  `sys` is imported but never used  py/unused-import [suggest]
  3:17  error  A mutable default is created once and shared by every call; default to `None` and create it in the body  py/no-mutable-default [suggest]
  6:5  error  A bare `except:` also catches `KeyboardInterrupt` and `SystemExit`; name the exceptions  py/no-bare-except [suggest]
  8:8  error  Compare with `None` using `is`, not `==`  py/none-comparison [suggest]

src/math.ts
  4:1  error  Use `let` or `const` instead of `var`  js/no-var [suggest]
  5:5  error  Nothing equals NaN, not even NaN; use Number.isNaN(x)  js/use-isnan [manual]
      1. Replace x === NaN or x == NaN with Number.isNaN(x).
      2. Replace x !== NaN or x != NaN with !Number.isNaN(x).

✖ 8 problems (8 errors, 0 warnings, 0 infos) · 1 auto fix with `codelaw fix` · 6 suggestions with `codelaw fix --unsafe` · 4 files checked
```

Apply the fixes, including the suggestions you have reviewed. `--dry-run` shows what would change without writing:

```console
$ codelaw fix --unsafe
fixed app/main.py (py/multiple-imports-on-one-line ×1, py/no-bare-except ×1, py/no-mutable-default ×1, py/none-comparison ×1, py/unused-import ×2)
fixed src/math.ts (js/no-var ×1, js/prefer-const ×1)

src/math.ts
  5:5  error  Nothing equals NaN, not even NaN; use Number.isNaN(x)  js/use-isnan [manual]
      1. Replace x === NaN or x == NaN with Number.isNaN(x).
      2. Replace x !== NaN or x != NaN with !Number.isNaN(x).

✖ 1 problem (1 error, 0 warnings, 0 infos) · 2 files fixed · 4 files checked
```

Seven findings fixed themselves. The one left says exactly what to do. Here is `app/main.py` before and after:

<table>
<tr><th>before</th><th>after <code>codelaw fix --unsafe</code></th></tr>
<tr><td>

```python
import os, sys

def greet(names=[]):
    try:
        print("hi")
    except:
        pass
    if names == None:
        return
```

</td><td>

```python

def greet(names=None):
    if names is None:
        names = []
    try:
        print("hi")
    except Exception:
        pass
    if names is None:
        return
```

</td></tr>
</table>

---

## Why teams pick codelaw

| | |
|---|---|
| 🧰 **One tool, nine languages** | JS, TS, Python, JSON, YAML, TOML, Markdown, CSS (and its dialect SCSS) and HTML, from one binary in one run. A Python-only repo doesn't need Node, and a JS-only repo doesn't need Python. |
| 📝 **A rule is a Markdown file** | Front matter for metadata, one fenced block for the matcher and fix, and `Do` / `Don't` examples. People read it, agents read it, and the engine runs it. |
| 🧪 **The docs are the tests** | `codelaw test` runs every rule against its own examples, so a rule's docs can't drift from what it does. |
| 🔧 **Every finding comes with a fix** | `auto` fixes apply themselves, `suggest` fixes come as a reviewable diff, and `manual` findings list the exact steps. No rule just says "this is wrong". |
| 🧱 **Fixes can restructure code** | Fix primitives move and regroup code, for example gathering every `export` into one block at the end of the file. A plain node-for-node replace can't do that. |
| 🔁 **Deterministic** | Same files and config in, same output out, with or without an AI agent. Every fix is re-parsed and checked before it's written, and a fix that would break the file is rolled back. |
| 🤖 **Built for AI coding agents** | `--format agent` gives one line per finding with its fix, ready to feed back to an agent after every edit. |
| 📦 **Standard rules included** | 676 rules reimplemented or written new: 492 ported from ESLint and its plugins, typescript-eslint, Ruff, Bandit, stylelint, html-validate and markdownlint, and 160 of codelaw's own — so your team's rules sit on top of a familiar base. |

**For the person who owns the codebase:** the review comment you keep writing becomes a rule file, once. From then on a person, a CI job and every AI agent get the same answer — and the answer can be audited, because it is a file in your repo and not a model's opinion.

---

## Rule packs

Eighteen first-party packs: sixteen on npm as `@codelaw/rules-<pack>`, eleven on PyPI as `codelaw-rules-<pack>`. A pack on both registries holds the same files on both.

<!-- docs-sync:packs -->

| pack | npm | PyPI | rules | presets | reimplemented from |
|---|---|---|---|---|---|
| [`js`](packs/js/README.md) | ✅ | ✅ | 80 | `js/recommended` · `js/typescript` · `js/conventions` · `js/project` · `js/size` · `js/markers` | ESLint 10 recommended (all 64, plus `no-console` at `warn`), and the four core rules typescript-eslint switches on for TypeScript |
| [`ts`](packs/ts/README.md) | ✅ | — | 27 | `ts/recommended` · `ts/conventions` · `ts/type-checked` | typescript-eslint 8 recommended (all 20) |
| [`react`](packs/react/README.md) | ✅ | ✅ | 32 | `react/recommended` · `react/conventions` | eslint-plugin-react, react-hooks, @eslint-react |
| [`node`](packs/node/README.md) | ✅ | — | 10 | `node/recommended` | eslint-plugin-n |
| [`stencil`](packs/stencil/README.md) | ✅ | — | 18 | `stencil/recommended` · `stencil/conventions` | @stencil/eslint-plugin |
| [`python`](packs/python/README.md) | — | ✅ | 105 | `py/recommended` · `py/conventions` · `py/project` · `py/size` · `py/markers` | Ruff 0.17's default `E4`/`E7`/`E9`/`F`, every stable flake8-bugbear rule, isort `I001`, `RUF022` |
| [`html`](packs/html/README.md) | ✅ | ✅ | 74 | `html/recommended` · `html/a11y` · `html/document` · `html/conventions` · `html/security` · `html/size` · `html/markers` | html-validate 11 recommended, a11y and document |
| [`css`](packs/css/README.md) | ✅ | ✅ | 73 | `css/recommended` · `css/standard` · `css/conventions` · `css/project` · `css/size` · `css/markers` | stylelint-config-recommended 18 (all 41), stylelint-config-standard 40 |
| [`scss`](packs/scss/README.md) | ✅ | ✅ | 16 | `scss/recommended` · `scss/strict` · `scss/markers` | stylelint-scss, and Dart Sass's module-system migrations |
| [`data`](packs/data/README.md) | ✅ | ✅ | 42 | `data/recommended` · `md/recommended` · `data/package-json` · `data/tsconfig` · `data/pyproject` · `data/github-actions` · `data/conventions` · `data/markers` | markdownlint 0.42, plus JSON/YAML/TOML correctness checks |
| [`tanstack-query`](packs/tanstack-query/README.md) | ✅ | — | 13 | `tanstack-query/recommended` | @tanstack/eslint-plugin-query 5 recommended (all 7), and 6 rules from TanStack Query v5's guides |
| [`redux-toolkit`](packs/redux-toolkit/README.md) | ✅ | — | 12 | `redux-toolkit/recommended` | codelaw's own, from the Redux Style Guide's Priority A and B rules |
| [`security-js`](packs/security-js/README.md) | ✅ | ✅ | 66 | `sec-js/recommended` · `sec-js/strict` | eslint-plugin-security, Bandit and Ruff `S` ports and codelaw's own, mapped to CWE and OWASP 2025: injection, XSS, traversal, SSRF, crypto, secrets, cookies, TLS, Electron, prompt injection |
| [`security-ts`](packs/security-ts/README.md) | ✅ | — | 16 | `sec-ts/recommended` · `sec-ts/strict` | extends `security-js`; Angular, NestJS, Next.js server actions, tRPC, Prisma/TypeORM/Drizzle raw queries and unchecked type assertions, read from syntax |
| [`security-html`](packs/security-html/README.md) | ✅ | ✅ | 19 | `sec-html/recommended` · `sec-html/strict` | 19 rules (inline handlers, `javascript:` URLs, mixed content, iframes, CSP, forms, template sinks); extends `html/security` |
| [`security-python`](packs/security-python/README.md) | — | ✅ | 61 | `sec-py/recommended` · `sec-py/strict` | Bandit and Ruff `S` ported, plus Django, Flask, FastAPI, ML-loader and LLM rules, each mapped to CWE and OWASP 2025 (W-57) |
| [`css-modules`](packs/css-modules/README.md) | ✅ | — | 5 | `css-modules/recommended` · `css-modules/strict` | eslint-plugin-css-modules, and the CSS Modules specification |
| [`spell`](packs/spell/README.md) | ✅ | ✅ | 7 | `spell/recommended` · `spell/strict` | codelaw's own spell checker: known typos from a list generated from SCOWL, fixed unattended; an opt-in dictionary (SCOWL, software terms) and locale check (VarCon); not added by `init` (W-103) |

<!-- /docs-sync:packs -->

- **Reimplemented, not wrapped.** Each rule is written fresh in codelaw's format, and its `ported_from:` field names the original. You don't need ESLint, Ruff or stylelint installed.
- **Differences are written down.** Each pack's README lists where a port behaves differently from the original, and which rules weren't ported and why.
- **Layout rules are left out on purpose.** Indentation, spacing and quotes are the formatter's job (`codelaw format`).
- **6,974 example checks, all green.** `codelaw test packs/<pack>` runs them.
- **`*/conventions` presets are opt-in**: house-style rules such as `js/exports-at-end` and `py/no-print`.
- **`*/size` presets are opt-in too**: `js/size` and `py/size` turn on max lines per file, max lines per function and max parameters for JavaScript, TypeScript and Python; `html/size` and `css/size` a line limit per file. Each rule has options for the limit and for what counts.
- **`*/project` presets are opt-in too**: the rules that read the whole repo, `graph/no-unused-exports` (`js/project`, `py/project`) and `graph/no-unused-css-classes` (`css/project`). `codelaw graph why-unused <symbol>` says why a finding held.
- **Every rule, for people and for agents:** one page per rule in [`docs/rules/`](docs/rules/README.md); [`llms.txt`](llms.txt), [`llms-full.txt`](docs/rules/llms-full.txt) (every rule in one file) and [`rules.json`](docs/rules/rules.json) (id, pack, languages, fix kind, options schema, presets, fix steps, a don't/do example) for an agent to read with nothing installed. All four are generated from the rule files and checked in CI.
- **The counts on this page are checked.** `python3 scripts/docs-sync.py --check` fails CI when a pack's rules, presets or fix classes and this page disagree.

### Fix classes across all 676 rules

<!-- docs-sync:fix-classes -->

| class | rules | what happens |
|---|---|---|
| `auto` | 80 | `codelaw fix` applies it. It's safe and meaning-preserving. |
| `suggest` | 128 | `codelaw fix --unsafe` applies it. Review the diff, because it can change behaviour. |
| `manual` | 468 | The finding lists numbered steps, for a person or an agent to follow. |

<!-- /docs-sync:fix-classes -->

---

## A rule is one Markdown file

Your convention, written once — people read it, agents read it, the engine runs it:

````markdown
---
id: acme/no-console-log
title: "Log through the app logger, not console.log"
languages: [typescript, tsx]
fix: auto
stability: preview
since: 0.1.0
---

Application code logs with `logger.info(…)`, never `console.log(…)`.

**Why:** the logger adds levels and request ids, and is silenced in tests.

```codelaw
match:
  pattern: console.log($$$ARGS)
message: "Use `logger.info` instead of `console.log`"
fix: "logger.info($$$ARGS)"
```

## Examples

### TypeScript

#### Don't

```ts
console.log("user", id);
```

#### Do

```ts
logger.info("user", id);
```
````

Then test it against its own examples:

```console
$ codelaw test acme/no-console-log
✔ acme/no-console-log  12 checks · local

1 rules · 12 checks · 0 failed
```

- **No plugin to build or publish:** a file in `.codelaw/rules/<namespace>/` is picked up on the next run.
- **The examples are the tests:** every `Don't` must fire and fix to its `Do`.
- **The full workflow** — `new-rule`, `test`, `tree`, `rules lint` — is in the [usage guide](USAGE.md#write-your-own-rules).

---

## Get started

**Three steps: install, `codelaw init`, `codelaw check`.**

- **Install.** codelaw 0.3.0 and its packs are on npm and PyPI; the [usage guide](USAGE.md#install) has the exact commands.
- **`codelaw init`** detects your languages and writes `codelaw.toml` with the matching presets.
- **`codelaw check`** reports every finding with its fix; **`codelaw fix`** applies the safe ones and formats.
- **With an AI agent:** `codelaw init --agents claude,codex,copilot,kiro,cowork` installs the skills package — `@codelaw/skills` (npm) or `codelaw-skills` (PyPI), at codelaw's exact version — and writes the hooks and instruction files from it ([usage guide](USAGE.md#with-ai-agents)).

**→ [The usage guide](USAGE.md)**: install, configure, write rules, wire up agents, editors, pre-commit and CI, and every command.

---

## Is codelaw for you?

**Built for codebases where people and AI write side by side:** years of human code, and an agent adding to it, held to the same rules.

**The more of your code an agent writes, the more you need rules it can't skip.** Good docs and search help an agent get it right the first time; a check is what proves it did.

**Choose codelaw when:**

- your repo mixes front end, Python and config files, and you want **one** tool, one config and one report;
- your team has conventions no off-the-shelf rule covers, and you want to write them **without building a plugin**;
- AI agents write some of your code, and you want their output checked **deterministically**, every time.

**Stay with what you have, for now, when:**

- you need **type-aware** rules, such as floating promises or `no-unsafe-*`. tree-sitter has no type information, so these stay with typescript-eslint, pyright or mypy;
- you need **byte-for-byte Prettier output** for HTML or Markdown. codelaw's formatter matches Prettier on every JS, TS and JSON file measured, but on only 62 % of HTML files and 42 % of Markdown files ([SR-FORMATTER](records/0105_formatter.md));
- you need a **stable, published** tool in production CI. 0.3.0 is a preview.

---

## Roadmap

<!-- docs-sync:roadmap -->

| | feature | record | state |
|---|---|---|---|
| ✅ | Engine: nine languages, one binary, parse once, cache | [SR-ARCHITECTURE](records/0108_architecture.md) | built |
| ✅ | `check`, `fix`, `format`, `test`, `rules`, `explain`, `tree`, `match`, `config`, `new-rule`, `init`, `cache` | [SR-CLI-MCP](records/0115_cli-and-mcp.md) | built |
| ✅ | Eighteen rule packs: 676 rules | [SR-RULE-PACKS](records/0101_rule-packs.md) | built, `preview` |
| ✅ | `--format agent`, `--format json` and `--format sarif`; `--timings` | [SR-CLI-MCP](records/0115_cli-and-mcp.md) | built |
| ✅ | Watch mode: `check --watch` and `fix --watch` re-check on every save | [SR-CLI-MCP](records/0115_cli-and-mcp.md) | built |
| ✅ | npm package (`npx codelaw`) | [SR-PACKAGES](records/0107_packages.md) | 0.3.0 published from CI with provenance, sixteen packs |
| ✅ | PyPI package (`uvx codelaw`) | [SR-PACKAGES](records/0107_packages.md) | 0.3.0: `codelaw` and eleven packs live, trusted; checked from a fresh venv and `uvx` (W-38) |
| ✅ | Formatter (`codelaw format`, and the last step of `fix`): Prettier-style JS/TS/JSON, Black-style Python | [SR-FORMATTER](records/0105_formatter.md) | built |
| ✅ | MCP server (`codelaw mcp`) | [SR-CLI-MCP](records/0115_cli-and-mcp.md) | built |
| ✅ | Language server (`codelaw lsp`): diagnostics, quick fixes, formatting | [SR-EDITOR](records/0114_editor.md) | built; the VS Code extension is built, not published |
| ✅ | Agent hooks (`codelaw hook`), `init --agents`, `sync-agents` | [SR-AGENT-SURFACES](records/0102_agent-surfaces.md) | built |
| ✅ | Project-wide rules (`graph/no-unused-exports`, `graph/no-unused-css-classes`, the `node/no-*-import` rules) | [SR-PROJECT-GRAPH](records/0103_project-graph.md) | built, `preview` |
| ✅ | Baselines and deprecations (`baseline`, `deprecate`) | [SR-BASELINE](records/0104_baseline-and-deprecations.md) | built |
| ✅ | External linters as adapters (oxlint, Ruff, stylelint, html-validate) and the warm `daemon` | [SR-ARCHITECTURE](records/0108_architecture.md) | built |
| 🔜 | Promotion to `stable`: the pinned corpus, its CI job, the promotion report and the ratified criterion | [SR-RULE-AUTHORING](records/0113_rule-authoring.md) | built; 42 `css` rules promoted, the other packs next |

<!-- /docs-sync:roadmap -->

The design for each item is a record in [`records/`](records/README.md); the open work is [`work/OPEN-WORK.md`](work/OPEN-WORK.md).

---

## FAQ

**How is codelaw different from ESLint and Prettier, or Biome?**
It does the same jobs — lint, fix, format — for nine languages from one binary and one config. The difference is the rule: a Markdown file your team writes without a plugin, that always carries a fix, and that an AI agent can ask for before it writes.

**Is it faster than ESLint and Prettier?**
Measured on 2026-10-28 on GitHub's `ubuntu-24.04` runner (4 CPUs), on three pinned real repos, each tool with its own recommended preset: codelaw lints JavaScript 2.7–4.2 × faster than ESLint 10.13 (3.3–5.0 × with both caches warm) and checks formatting 2.1–3.7 × faster than Prettier 3.9. Biome, oxlint and Ruff are faster than codelaw on their own languages. Every number, and how the work differs, is in [SR-LAW-8 §Measurements](records/0010_LAW-8-performance-budgets.md); `bench-rivals.yml` re-measures weekly.

**Does it work without AI?**
Yes, completely. The CLI, the editor extension, pre-commit and CI need no agent at all. Agents are one more caller of the same engine.

**Does codelaw use AI?**
No. The engine has no model and makes no network calls. Agents are one more caller of the same deterministic engine ([L5](records/0007_LAW-5-no-model-in-the-engine.md)).

**Do I need ESLint, Ruff or stylelint installed?**
No. The pack rules are reimplemented natively in codelaw.

**Will a fix ever break my code?**
Fixes run in passes until nothing more changes, up to ten passes. After each pass the file is re-parsed, and a fix that would introduce a syntax error is rolled back. Fixes that could change behaviour are `suggest`, and they only apply with `--unsafe`.

**Why are most rules `preview`?**
Because only the first batch has met the promotion criterion on the record: 42 `css` rules are `stable`, each promoted on a named CI corpus run; the rest are `preview` until they meet it. CI runs every pack over 13 pinned third-party repositories on every change and holds each rule's finding count; `scripts/corpus.py promote` then reports which preview rules pass the criterion — at least 3 repositories and 10 files, no false positive among its findings, its fixes verified, one release in preview ([SR-RULE-AUTHORING](records/0113_rule-authoring.md) decision 6a) — and why each other rule is held. A rule with findings waits on a review of them. Set `preview = true` to run the preview rules.

**Can I use it in a Python-only repo without Node?**
Yes. Nothing in codelaw runs Node or Python. Once released, the PyPI package will be a self-contained binary.

---

## Contributing

Bug reports, false positives and rule requests are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md).

- **A false positive?** Open an issue with the smallest snippet that triggers it.
- **Security issue?** See [SECURITY.md](SECURITY.md). Please don't open a public issue.
- **How the project is run:** every design decision is a record in [`records/`](records/README.md), and [`docs/DESIGN.md`](docs/DESIGN.md) is the original proposal.

## Licence

**MIT** — see [LICENSE](LICENSE). It covers the engine, the repo and every package, the rule packs included (each carries its own copy). Every dependency is free, open source and permissively licensed, and nothing paid or copyleft is used ([L2](records/0004_LAW-2-foss-dependencies.md)).

## Acknowledgements

codelaw's rule packs follow the documented behaviour of [ESLint](https://eslint.org), [typescript-eslint](https://typescript-eslint.io), [Ruff](https://docs.astral.sh/ruff/), [stylelint](https://stylelint.io), [html-validate](https://html-validate.org) and [markdownlint](https://github.com/DavidAnson/markdownlint). Its matcher is built on [ast-grep](https://ast-grep.github.io) and [tree-sitter](https://tree-sitter.github.io).
