---
type: Standing Record
kind: component
name: SR-FORMATTER
title: "SR-FORMATTER (0105) — codelaw is a linter and a formatter: one binary, one config, one run"
description: "codelaw formats all nine languages with embedded, permissively licensed formatters — Prettier-compatible for JS/TS/JSON, Black-compatible for Python — runs formatting after lint fixes, and refuses any lint rule that only changes layout."
status: accepted
date: 2026-10-18
feature: "formatting"
owns: []
laws: [L0, L2, L3, L4, L5, L6]
timestamp: 2026-10-18T17:00:00Z
content_sha: ""
---

# SR-FORMATTER — codelaw is a linter and a formatter: one binary, one config, one run

## §1 — For humans

**The decision.** codelaw does both jobs. **The linter owns meaning and structure.** **The formatter owns layout:** whitespace, line breaks, indentation, quotes, semicolons and trailing commas. One binary, one `codelaw.toml`, one command, and the two never fight over the same characters.

**What you run:**

| Command | Does |
|---|---|
| `codelaw format` | Formats files in place (like `prettier --write` / `ruff format`) |
| `codelaw format --check` | Changes nothing; exit 1 if any file isn't formatted (CI) |
| `codelaw fix` | Lint fixes first, **then** formats, then verifies the result |
| `codelaw check` | Lint findings plus one `format/unformatted` finding per unformatted file (`auto` fix, [L4](0006_LAW-4-every-rule-has-a-fix.md)) |

**No Node, no Python, no Prettier install.** Each language's formatter is embedded in the binary, so a Python-only repo gets Prettier-style JSON and Markdown formatting, and a JS-only repo gets Black-style Python, without either runtime ([L3](0005_LAW-3-languages.md), [L6](0008_LAW-6-python-and-node-packages.md)).

```mermaid
flowchart LR
    S["source file"] --> L["lint fixes<br/>(meaning, structure)"]
    L --> F["format<br/>(layout only)"]
    F --> V["verify: parses · idempotent · lint-clean"]
    V --> O["written"]
```

<details>
<summary><b>ASCII twin</b></summary>

```text
   source --> lint fixes (meaning) --> format (layout) --> verify (parses, idempotent, lint-clean) --> written
```

</details>

---

## §2 — For agents

### Context

Teams run a linter and a formatter as two tools: ESLint plus Prettier, or Ruff check plus Ruff format, plus yet another tool for YAML and TOML. They have two configs that can contradict each other, and agents have to know both. The decider asked for codelaw to be both.

### Decision

1. **One tool, two roles.** `codelaw format [paths] [--check]` is the formatter. `codelaw fix` runs lint fixes and then formats. `codelaw check` reports unformatted files as `format/unformatted` (fix class `auto`). Set `[format] check = false` to leave formatting out of `check`.

2. **The embedded engines.** Each one is compiled into the binary; none is invoked as an external program.

   | Language | Engine | Output style | Licence | How it's pulled in |
   |---|---|---|---|---|
   | JS · JSX · TS · TSX | Biome formatter (`biome_js_formatter`) | Prettier-compatible (Biome tracks its Prettier compatibility in a published report) | MIT / Apache-2.0 | git dependency pinned to a Biome release tag; the crates.io copies are stale |
   | JSON · JSONC | Biome formatter (`biome_json_formatter`) | Prettier-compatible | MIT / Apache-2.0 | same pin |
   | Python | Ruff formatter (`ruff_python_formatter`) | Black-compatible (> 99.9 % identical lines on Django and Zulip, per Ruff) | MIT | git dependency pinned to a Ruff release tag, the way `dprint-plugin-ruff` already does |
   | YAML | `pretty_yaml` | configurable, close to Prettier's YAML style | MIT | crates.io |
   | TOML | `taplo` formatter | taplo style | MIT | crates.io |
   | Markdown | `dprint-plugin-markdown` | CommonMark/GFM, dprint style | MIT | crates.io |
   | CSS | Biome formatter (`biome_css_formatter`) | Prettier-compatible; standard CSS only (no SCSS/Less) | MIT / Apache-2.0 | same Biome pin |
   | HTML | `markup_fmt` | Prettier-like; embedded `<script>` and `<style>` are handed to the JS and CSS engines through its callback | MIT | crates.io |

   Every engine must pass [L2](0004_LAW-2-foss-dependencies.md); the licence check is `W-20`.

3. **Layout belongs to the formatter, and the rule loader enforces it.** A lint rule whose fix only changes layout is refused. `codelaw test` checks this deterministically: if `format(Don't) == format(Do)`, the rule is a layout rule and fails. This is the formatter-versus-linter fight that ESLint's stylistic rules caused, ruled out by construction.

4. **Order of operations:**
   1. Parse.
   2. Lint fixes, multi-pass up to the fixpoint.
   3. Format.
   4. Verify: the file still parses, a second `format` changes nothing, and no lint rule fires that didn't fire before formatting.

   If verification fails, the file is left untouched and the failure is reported ([L4](0006_LAW-4-every-rule-has-a-fix.md) decision 5).

5. **One config, Prettier/Black-shaped keys.**
   ```toml
   [format]
   enabled         = true
   check           = true          # report unformatted files in `codelaw check`
   line_width      = 80
   indent_style    = "space"       # space | tab
   indent_width    = 2
   end_of_line     = "lf"
   quote_style     = "double"      # JS/TS
   semicolons      = "always"      # JS/TS: always | as-needed
   trailing_commas = "all"         # JS/TS: all | es5 | none
   exclude         = ["**/generated/**"]

   [format.python]
   line_width  = 88                # Black/Ruff default
   indent_width = 4

   [format.markdown]
   format_code_blocks = true       # fenced js/ts/py/json/yaml/toml blocks use their own formatter
   ```
   - **Defaults:** Prettier's for JS/TS/JSON/CSS/HTML/YAML/Markdown; Black/Ruff's for Python.
   - **`.editorconfig` is always read** as the base layer; `codelaw.toml` wins over it.
   - **Migrating from existing configs:** `codelaw format --import-config` reads `.prettierrc*`, `[tool.ruff.format]` and `[tool.black]` once and writes the equivalent `[format]` keys. From then on, `codelaw.toml` is the only source.

6. **Ignore markers.**
   - **codelaw's own:** `// codelaw-format-ignore` and `# codelaw-format-ignore` skip the next node.
   - **Honoured for compatibility:** `// prettier-ignore`, `<!-- prettier-ignore -->`, and Ruff/Black's `# fmt: off` / `# fmt: on` and `# fmt: skip`.
   - **Whole files:** `[format] exclude`.

7. **Deterministic and stable ([L5](0007_LAW-5-no-model-in-the-engine.md)).**
   - The same input and the same codelaw version always give byte-identical output.
   - A pinned engine upgrade that changes output for any corpus file is a **minor** release, and the changelog lists the changed styles.
   - Already-formatted files are skipped using the content-hash cache.

8. **Every surface.**
   - **LSP:** document and range formatting, and format-on-save.
   - **Agent hook ([SR-AGENT-SURFACES](0102_agent-surfaces.md)):** runs `fix` then `format` on the edited file, so an agent never hand-formats and never fights the formatter.
   - **MCP:** a `format_snippet` tool.
   - **Generated agent instructions** say "don't format by hand; codelaw formats on save".

### Consequences

- **Two engines are git-pinned with unstable APIs** (Biome, Ruff). Upgrades are deliberate, gated by a snapshot corpus, and a breaking upstream change can stall them. `dprint-plugin-ruff` shows the approach is sustainable.
- **Git dependencies mean the engine crate can't be published to crates.io.** That doesn't matter: [L6](0008_LAW-6-python-and-node-packages.md) ships through npm and PyPI.
- **"Prettier-compatible" isn't "identical"** on every edge case for JS, and YAML and Markdown follow their engines' styles. The measured gap is `W-07`.
- **The binary grows** by the size of six formatter engines. Accepted, because each package must be complete ([L6](0008_LAW-6-python-and-node-packages.md) decision 3).

### Alternatives considered

- **Shell out to Prettier and Black:** rejected. It needs Node and Python at runtime, which breaks [L3](0005_LAW-3-languages.md) independence and [L6](0008_LAW-6-python-and-node-packages.md) completeness, and it adds two more tools for users to install.
- **`dprint-plugin-typescript` for JS/TS:** kept only as a fallback. It's MIT and on crates.io, but its output isn't Prettier-compatible, which is the style teams expect.
- **Write codelaw's own formatters:** rejected. It's years of work, and mature permissive engines already exist.
- **`malva` for CSS** (MIT; handles CSS, SCSS, Sass and Less): kept as the replacement if SCSS or Less ever enter [L3](0005_LAW-3-languages.md).
- **dprint's WASM plugin host for every language:** deferred. It's one uniform plugin model, but it adds a WASM runtime and a plugin-download step that conflicts with "complete, offline" ([L5](0007_LAW-5-no-model-in-the-engine.md) decision 3).

### Reference (required)

- [Ruff formatter README](https://github.com/astral-sh/ruff/blob/main/crates/ruff_python_formatter/README.md): Black compatibility, and the crate is internal with an unstable API.
- [dprint-plugin-ruff](https://github.com/dprint/dprint-plugin-ruff): embeds `ruff_python_formatter` by pinning a Ruff tag.
- [Biome Prettier challenge hub](https://github.com/biomejs/biome/issues/720) and [`biome_js_formatter` on lib.rs](https://lib.rs/crates/biome_js_formatter): Prettier-compatibility tracking, and the crates.io copy is stale (0.5.7, March 2024).
- [`pretty_yaml` on docs.rs](https://docs.rs/pretty_yaml): an MIT, configurable YAML formatter.

### Veto condition

**Reopen this decision if:** an embedded engine's licence stops being permissive, or measured Prettier/Black compatibility on the corpus falls below what teams accept (`W-07`).

**How to check it:** `W-20`'s licence check over the engine's lockfile, and the `W-07` corpus report, once both exist.

---

## References

*Every source this record cites, gathered in one place. An archived document is never listed here.*

**Records** — [SR-AGENT-SURFACES](0102_agent-surfaces.md)

**Project docs** — [`docs/DESIGN.md`](../docs/DESIGN.md)
