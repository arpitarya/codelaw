---
type: Compare
description: "Arpit, 2026-10-25: CI-vs-local severity, per-glob severity remap, dated exceptions with an author, and grandfathered React context — the five forks and his rulings. Builds as W-39, W-40, W-41."
---

# CI profiles, severity remap and dated exceptions — the forks

> **Verdict:** decided by the decider, 2026-10-25 · auto-detected `ci`/`local` profiles, agents run `ci` · a per-glob `remap` of severities (silent, warning or error, configurable) · `expires` is the **first failing day**, warnings `warn_days` before · `added_by` must match the git author · `react/no-context` flags `createContext` **and** `useContext` · confidence high on the rulings, medium on the git-author check's cost · **reopen-trigger:** a team runs CI with `CI` unset and ships a `console.log`, or `git blame` makes `check --changed-since` miss L8's budget.

## Context

**Arpit, 2026-10-25:** *"while I'm starting my dev server or doing it in my local, having debugger or console logs is fine. But when we are running it in CI/CD pipeline, it should throw an error. In local, it should give you a warning … for certain files it should never throw an error. For certain files, it should never throw a warning … if I mark that feature as true, then every exception should have a date … till that date there shouldn't be an error … 2 days, 3 days in advance, we should start getting warnings. And on that particular day, it should start giving error … an optional field where we can add some comments. And … email ID, that is who added that exception, which can be validated on commit or in CI/CD … wherever we were using React context, that's okay. It should just throw a warning. But if there is a new implementation … it should throw an error."*

What already existed: per-rule severity and `[[overrides]]` (SR-CONFIG, SR-LINTER decision 5); the baseline with `until`, `warn_from`, `on_touch`, `reason` (SR-BASELINE); `js/no-debugger`, `py/no-print`. Missing: any environment-dependent severity, a whole-glob severity cap, a "first failing day" date, a relative warning window, an author, dates on inline ignores, `js/no-console`, a React-context rule.

## Options and rulings

| fork | options | ruling |
|---|---|---|
| 1 · how codelaw knows it is CI | auto from `CI` env · explicit `--profile` only | **auto**: `--profile` > `CODELAW_PROFILE` > `CI` set and not `false`/`0` → `ci` > `local` |
| 2 · what an agent hook sees | `ci` · `local` | **`ci`** — an agent never leaves a `console.log` behind; the LSP (a person's editor) stays `local` |
| 3 · "never warn" files | silent · warnings become errors · drop warnings, keep errors | **configurable** — one `remap` table per glob maps each severity to `off`, `info`, `warn` or `error` |
| 4 · `added_by` check | format only · must match git author · allowlist | **git author** — `--staged`: `git config user.email`; `--changed-since`: the blame author of the exception's line |
| 5 · React context "new implementation" | `createContext` + `useContext` · `createContext` only | **both** |
| 6 · the date's meaning (agent's call, from the ask) | keep `until` (last allowed day) · new `expires` (first failing day) | **`expires`**; `until` read as `expires = until + 1` for one minor release, then removed (SR-CONFIG's rename rule) |

## Consequences

- An agent and CI give the same verdict; a person's editor is more lenient than both. That is the point, and it is stated in each surface's header.
- The date comes from the as-of date (SR-BASELINE decision 4), not the wall clock: a branch with no new commit does not start failing on the day. A scheduled CI job passes `--as-of "$(date +%F)"`.
- `require_expiry = true` and grandfathered React context collide: the context baseline then needs an `expires` too (it may be far off).
- The git-author check costs one `git blame` per changed file holding an exception — only under `--changed-since`, never in hook or LSP mode.

## References

- [SR-BASELINE](../../records/0104_baseline-and-deprecations.md) decisions 1–5 · [SR-CONFIG](../../records/0110_config.md) decisions 4, 5, 9 · [SR-LINTER](../../records/0106_linter.md) decisions 5–6.
- [ESLint `no-console`](https://eslint.org/docs/latest/rules/no-console) · [ESLint bulk suppressions](https://eslint.org/docs/v9.x/use/suppressions) · [unicorn `expiring-todo-comments`](https://github.com/sindresorhus/eslint-plugin-unicorn/blob/master/docs/rules/expiring-todo-comments.md).

## Reopen-trigger

A team runs CI without `CI` set and a `console.log` ships; or `git blame` under `--changed-since` breaks an L8 budget.
