---
type: Handoff
name: W-59
description: "Arpit, 2026-10-26: named, singular config tables — [scope.utils] for a group of files, [profile.ci] for an environment — with preset-shipped scopes that a project extends by name; the old spellings stay as aliases for one minor release."
item: W-59
filed: 2026-10-26
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-26
updated_by: "Claude Code (Opus 5.5), 2026-10-26: built — every DoD met"
ball: done
---

# W-59 — `[scope.<name>]` replaces `[[overrides]]`; `[profiles.*]` becomes `[profile.*]`

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Amend SR-CONFIG decisions 3, 4, 10, 11 and §1: `[scope.<name>]` with `files`, `exclude`, `rules`, `remap`; `[profile.<name>]`; precedence unchanged (preset scopes, then the project's, in file order) | the record is the spec | ✅ |
| 2 | Loader and schema: names `[a-z0-9_-]+`; `default` reserved; same-named scope from a preset merges (project values win, `files` inherited when omitted) | named, mergeable scopes | ✅ |
| 3 | Aliases for one minor release: `[[overrides]]` (becomes `scope.override-<n>`) and `[profiles.*]` load with a warning naming the new spelling | no break on upgrade | ✅ |
| 4 | Presets ship scopes: `js/recommended` → `tests` (`**/*.test.*`, `**/*.spec.*`), `stories` (`**/*.stories.*`), `config` (`*.config.*`); `py/recommended` → `tests` (`tests/**`, `test_*.py`, `*_test.py`) | common groups for free | ✅ |
| 5 | `config --print` names the scope (`max = 50 (scope "utils", codelaw.toml:7)`); new `codelaw config --scopes` lists every scope with its files and source | traceable | ✅ |
| 6 | Update every doc, preset, fixture, `codelaw migrate`'s mapping (W-48), W-58's examples; USAGE gains the `*.utils.*` example | one spelling everywhere | ✅ |

**Arpit, 2026-10-26:** *"I don't like the idea of overrides … overrides is not the right terminology."* → *"Then we will go with scope."* — singular in the TOML file, and `profile` likewise, as discussed the same day.

**Model:** Opus — it renames user-facing config with an alias window and adds cross-preset merging; precedence mistakes pass silently.

## Goal

```toml
[rules]
"js/max-lines" = { severity = "error", options = { max = 100 } }

[scope.utils]
files = ["**/*.utils.ts", "**/*.utils.tsx", "**/*.utils.js", "**/*.utils.jsx"]
rules = { "js/max-lines" = { options = { max = 50 } } }

[scope.tests]            # extends the preset's tests scope; files come from it
rules = { "js/max-lines" = { options = { max = 400 } } }

[profile.ci]
rules = { "js/no-console" = "error" }
```

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | the example above gives `date.utils.ts` max 50, `Orders.tsx` 100, `a.test.ts` 400 | ✅ |
| 2 · | `[[overrides]]` and `[profiles.ci]` still load, with one warning each naming the new spelling | ✅ |
| 3 · | `[scope.default]` and `[scope."a b"]` are load errors (exit 2) | ✅ |
| 4 · | `config --scopes` lists preset and project scopes in precedence order | ✅ |
| 5 · | schema test green | ✅ |

## Blockers

None.

## Record affected

SR-CONFIG decisions 3, 4, 10, 11 and §1 · SR-RULE-PACKS (presets ship scopes).

## Hazards

- Inside a profile, `[profile.ci.scope.utils]` replaces `profiles.ci.overrides` — check the depth reads well in TOML.
- The alias is removed in the next minor release: say so in CHANGELOG now.

## Reference

[Cargo `[profile.*]`](https://doc.rust-lang.org/cargo/reference/profiles.html) · [ESLint flat config `files`](https://eslint.org/docs/latest/use/configure/configuration-files)
