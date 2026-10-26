---
type: Handoff
name: W-73
description: "Arpit, 2026-10-26: rules that require a certain function (a sanitizer at dangerouslySetInnerHTML, a wrapper instead of fetch) take that function from the project's config — added to the built-in list or replacing it — matched by name and import source; plus a 'use X instead of Y' rule for JS/TS and Python."
item: W-73
filed: 2026-10-26
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-26
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-73 — Configurable approved calls: your sanitizer, your wrappers

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Option type `call[]` with an `<opt>_mode` of `extend` or `replace`; the `calls:` predicate in `where:`; `$OPTION.<name>` in messages | a declarative rule can read a project's function list | ⏳ |
| 2 | The five HTML-sink rules move their hard-coded sanitizer regex onto a `sanitizers` option | `sanitizers = [{ name = "sanitizeHtml", module = "@acme/security" }]` works | ⏳ |
| 3 | Audit every first-party rule for another hard-coded approved-call list; move each | "wherever we have these kind of rules" | ⏳ |
| 4 | `js/use-instead` and `py/use-instead`: "use `apiFetch` instead of `fetch`", auto-fix when the wrapper is a drop-in | custom functions and variables everywhere, not only at sinks | ⏳ |

**Arpit, 2026-10-26:** *"I want that function to be configurable … rather than passing DOMPurify.sanitize throughout the repo, you need to use this function. I want these kind of rules."*

## Rulings (Arpit, 2026-10-26, answering four questions)

| question | ruling |
|---|---|
| a project's list vs the built-in one | **both, configurable**: *"make it configurable add to it or overwrite it"* → `<opt>_mode = "extend"` (default) or `"replace"` |
| where it is configured | **per rule options** in `[rules]` (no shared `[functions]` table) |
| how a function is matched | **name + import source**; a bare name only for a global |
| a generic "use X instead of Y" rule | **yes, JS/TS and Python** |

## Goal

A project names its own approved function once per rule, and codelaw holds the code to it. Today the five rules below carry the same regex — `^(DOMPurify\.sanitize|purify\.sanitize|sanitizeHtml|sanitize|xss|filterXSS|escapeHtml|escape|_\.escape|he\.escape|[A-Za-z_$][\w$]*\.createHTML)\s*\(` — and match by spelling, so `function sanitize(x) { return x }` passes.

```toml
[rules."sec-js/no-dangerously-set-inner-html".options]
sanitizers      = [{ name = "sanitizeHtml", module = "@acme/security" }]
sanitizers_mode = "replace"      # only our wrapper counts; "extend" (default) adds it to the built-in list

[rules."js/use-instead".options.replacements]
"fetch"              = { use = "apiFetch",     from = "@acme/http",     drop_in = true, allow_in = ["src/lib/http.ts"] }
"DOMPurify.sanitize" = { use = "sanitizeHtml", from = "@acme/security", drop_in = true, allow_in = ["src/security/**"] }
"console.log"        = { use = "logger.info",  from = "@acme/log" }
"process.env"        = { use = "env",          from = "@acme/env" }
```

## Design (the spec is SR-RULE-FORMAT decisions 2, 3a and 3b, amended 2026-10-26)

- **`call[]` option type** — an item is a callee path (`"DOMPurify.sanitize"`, `"*.createHTML"`) or `{ name, module }` (`module` a string or a list). Validated at load (exit 2 on a bad shape). Declaring it implies `<opt>_mode: extend | replace`, default `extend`.
- **`where: { $V: { not: { calls: sanitizers } } }`** — holds when `$V` is a call whose callee is in the resolved list.
  - `{ name, module }`: the callee's root binding is imported from `module` (named, aliased, default or namespace import; Python `import x` / `from x import y`). The root of `name` is compared with the **export name** for a named import and with the **local name** for a default or namespace import; the rest of the path must match exactly.
  - bare name: the root is undeclared in the file (a global). A same-named local function fails.
  - no flow: `const c = clean(x); … __html: c` is not a call — same limit as `from_source`, said on each rule page.
- **The built-in default, re-expressed with modules:** `{DOMPurify.sanitize, dompurify|isomorphic-dompurify}`, `"DOMPurify.sanitize"` (global), `{sanitizeHtml, sanitize-html}`, `{xss|filterXSS, xss}`, `{escapeHtml, escape-html}`, `{escape|_.escape, lodash|lodash-es|lodash/escape}`, `{he.escape|escape, he}`, `"*.createHTML"` (Trusted Types). The bare `sanitize`, `escape` and `xss` that pass today by spelling stop passing — the intended tightening; see Hazards.
- **Messages** name the list: `` "`$V` becomes HTML here without one of: $OPTION.sanitizers" ``.
- **`js/use-instead` / `py/use-instead`** (`native:`; one option `replacements`, type `object`, default `{}` so the rule is inert until configured). A key is a dotted path matched on a call's callee or a member read, exactly or as a prefix at a `.` boundary (`process.env` matches `process.env.API_URL`), only where its root is a global or an import, never a local shadow. A file matching the entry's `allow_in` is exempt (the wrapper itself must call `fetch`). `drop_in = true` → `fix: auto`: replace the path with `use`, add `import { apiFetch } from "@acme/http"` / `from acme.http import api_fetch` after the imports when absent; otherwise `manual` with the message. JS/TS rule in the `js` pack, Python rule in the `python` pack; both in `recommended` (inert until configured).
- **Complement, not overlap:** `sanitizers_mode = "replace"` reports the wrong sanitizer *at a sink*; `use-instead` reports it *anywhere*.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | `call[]` type, `_mode`, `calls:` predicate, `$OPTION.<name>`; load errors for a bad item, an undeclared option, `calls:` on a non-`call[]` option; unit tests in `codelaw-match` and `codelaw-rules` | ⏳ |
| 2 · | the five rules below read `sanitizers`; each `.cases.md` covers: extend passes a project fn and DOMPurify · replace passes only the project fn and reports DOMPurify · aliased import passes · same-named local fn is reported · wrong module is reported · global `DOMPurify` passes | ⏳ |
| 3 · | audit of every first-party pack for other hard-coded approved-call lists (JS, TS, Python), each listed here and moved, or listed with why not | ⏳ |
| 4 · | `js/use-instead`, `py/use-instead` with examples, cases (prefix match, `allow_in`, local shadow ignored, import added once, import already present, non-drop-in is manual) | ⏳ |
| 5 · | `codelaw test` green on every pack; corpus counts in each `corpus.toml` updated with the change explained; L8 `perf` within budget | ⏳ |
| 6 · | docs: each pack README row, the config example in `docs/`, `IMPLEMENTATION.md`, CHANGELOG (the tightened default is a behaviour change) | ⏳ |

The five rules: `sec-js/no-dangerously-set-inner-html`, `sec-js/no-unsanitized-html`, `sec-js/no-jquery-html-sink`, `sec-js/angularjs-sce-disabled`, `sec-ts/angular-native-element-html`.

## Blockers

None for the item. **L9 for the building session:** `target/` was 37 GB on 2026-10-26 against the 5 GB budget ([SR-LAW-9](../../records/0011_LAW-9-machine-budget.md)); `cargo clean` first or ask for the GBs.

## Record affected

[SR-RULE-FORMAT](../../records/0112_rule-format.md) decisions 2, 3a, 3b — **amended 2026-10-26 in this filing** (design of record before building, SR-WORK-LIFECYCLE decision 3). [SR-RULE-PACKS](../../records/0101_rule-packs.md) only if the pack rule counts it states change.

## Hazards

- **The default gets stricter.** A consumer's local `sanitize()` that passed by spelling is now reported. Rules are `stability: preview`; say it in the CHANGELOG and the message's note: *add it to `sanitizers`*.
- **Wrappers call the thing they wrap.** Without `allow_in`, `use-instead` reports the wrapper's own body; the example config must show it.
- **Prefix match on member reads** must not hit a declaration or a property key (`{ fetch: … }`); cases for both.
- **Python aliases** (`import bleach as b`) and re-exports through `__init__.py` — the root is resolved in the file only; a re-export is matched by the module the file imports from.

## Model

**Model:** Opus — it changes what five security rules accept, and a wrong default opens a hole no case catches; close to Sonnet because DoD 1–5 are checkable.

## Prompt (paste-ready, for a Claude Code lane)

> W-73. Read `CLAUDE.md`, then `work/open/W-73-configurable-approved-calls.md` and SR-RULE-FORMAT decisions 2, 3a, 3b. `lockmaster start W-73 --paths crates/codelaw-match crates/codelaw-rules crates/codelaw-native packs/security-js packs/security-ts packs/js packs/python records/0112_rule-format.md work/open/W-73-configurable-approved-calls.md` (add paths the audit finds). **Explore:** how `imported-from` and `from_source` resolve bindings (`crates/codelaw-semantic/src/model.rs`, `crates/codelaw-match/src/predicate.rs`), how options reach `native:` rules today. **Plan** DoD 1→6 in order; stop and write `work/BLOCKED.json` if the record's semantics cannot be built as written. **Implement**, then **verify**: `cargo test`, `codelaw test` on every pack, `perf`. Update this file's DoD table, `OPEN-WORK.md`, `IMPLEMENTATION.md` and the worklog in the same change; `lockmaster submit -m "W-73: …"`.

## Reference

[eslint-plugin-no-unsanitized customization](https://github.com/mozilla/eslint-plugin-no-unsanitized/blob/main/docs/rules/customization.md) (an `escape` option for allowed tagged templates — prior art, by name) · [ESLint `no-restricted-syntax` / `no-restricted-globals`](https://eslint.org/docs/latest/rules/no-restricted-globals) (report-only; no replacement fix) · [Ruff `banned-api` (TID251)](https://docs.astral.sh/ruff/rules/banned-api/) (Python: bans an import, with a configured message) · [Trusted Types `createHTML`](https://developer.mozilla.org/en-US/docs/Web/API/TrustedTypePolicy/createHTML)
