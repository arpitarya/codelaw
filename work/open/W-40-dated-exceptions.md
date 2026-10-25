---
type: Handoff
name: W-40
description: "Arpit, 2026-10-25: an opt-in switch that makes every exception carry a date; warnings a few days before, an error on the day; an optional comment and an optional author email checked against git."
item: W-40
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-25
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-40 — exceptions with an expiry date, a comment and an author

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | The decider rules the forks | the design below | ✅ 2026-10-25 — [compare](../compare/ci-profiles-and-dated-exceptions.compare.md) forks 4 and 6 |
| 2 | Write SR-BASELINE decision 9 and SR-LINTER decision 6's inline form, with the schema, in one change | the record is the spec | ⏳ |
| 3 | `expires` + `warn_days` on baseline, deprecation and inline exceptions; `until` as an alias | warning before the day, error on it | ⏳ |
| 4 | `[exceptions] require_expiry` | dates become mandatory when switched on | ⏳ |
| 5 | `added_by` and its git check | who added it, verified on commit and in CI | ⏳ |
| 6 | `baseline add` / `deprecate` take `--expires`, `--warn-days`, `--added-by` (default `git config user.email`) | people never hand-write the entry | ⏳ |
| 7 | Docs: USAGE, CHANGELOG, GLOSSARY (`expires`, `exception`) | users can find it | ⏳ |

**Arpit, 2026-10-25:** *"if I mark that feature as true, then every exception should have a date. If I mark it as false, then exception is just an exception … till that date, there shouldn't be an error. As that date is coming nearby, maybe 2 days, 3 days in advance, we should start getting warnings. And on that particular day, it should start giving error and should start to fail. I want an optional field over there where we can add some comments. And … an optional field where we can add email ID, that is who added that exception, which can be validated on commit or in CI/CD pipeline."*

**Model:** Opus — it touches L5's clock and the ratchet; a wrong day boundary passes every test written from the same misreading.

## Goal

```toml
# codelaw.toml
[exceptions]
require_expiry = true      # default false: an exception without a date is fine
warn_days      = 3         # default 3

# .codelaw/baseline/js--no-console.toml
[[allow]]
file     = "src/debug/panel.ts"
count    = 2
expires  = "2026-11-15"        # first day it fails
reason   = "debug panel, removed in ORD-1600"
added_by = "arpit@example.com"
```

```ts
// codelaw-ignore js/no-console expires=2026-11-15 by=arpit@example.com: needed until the logger lands
console.log(state);
```

## Design (becomes SR-BASELINE decision 9; SR-LINTER decision 6 cites it)

**What an exception is:** a baseline entry, a deprecations-registry entry, an inline `codelaw-ignore`. An `[[overrides]]` entry may also carry `expires` (and is then ignored from that day), but `require_expiry` does not apply to overrides — they are configuration, not exceptions.

**Keys, the same on every form** (inline: `expires=`, `warn_days=`, `by=` before the colon; the reason after it):
- `expires` — `YYYY-MM-DD`, the **first day the exception no longer holds**.
- `warn_days` — optional; default `[exceptions] warn_days`. `warn_from`, where already allowed, still wins.
- `reason` — optional on files (it already exists), required inline (SR-LINTER decision 6 — unchanged).
- `added_by` / `by=` — optional email.
- `until` stays accepted for one minor release, read as `expires = until + 1`; both on one entry is a load error (exit 2).

**The timeline** (`today` = the as-of date, SR-BASELINE decision 4, never the wall clock):

| as-of date | result |
|---|---|
| before `expires − warn_days` | as today: suppressed (inline) or `info` (baseline) |
| from `expires − warn_days` | **warning**, `codelaw/expiring-exception`: "expires in 2 days (2026-11-15)" |
| on or after `expires` | **error**, `codelaw/expired-exception`, at the exception's own line, and the exception stops covering — the rule's findings come back at their own severity |

`codelaw/*` diagnostics are never remapped (W-39), so a `{ error = "warn" }` glob can't hide an expiry.

**`require_expiry = true`:** each exception without `expires` is an error, `codelaw/exception-without-expiry`, at its line. Fix `manual`: "add `expires = …`" — `baseline add --expires` rewrites it.

**`added_by` check** (`codelaw/exception-author`, error):
- Always: a malformed email.
- `--staged`: each exception added or changed in the staged diff whose `added_by` is not `git config user.email` (case-insensitive).
- `--changed-since <ref>`: each exception added or changed since the merge base whose `added_by` is not the `author-mail` of `git blame --porcelain` on that line.
- Plain `check`, hook and LSP: format only. No `git`: format only, and the header says so.
- Absent `added_by` is never an error (it's optional).

**The ratchet** (SR-BASELINE decision 5): moving `expires` later is "extending a deadline" — it fails `baseline check` like `until` does. Adding `added_by` or a `reason` never does.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | with `expires = D`, `warn_days = 3`: `--as-of D-4` exit 0, no warning · `--as-of D-3` warning · `--as-of D` exit 1 — for baseline, deprecation and inline forms | ⏳ |
| 2 · | `require_expiry = true` + an undated inline ignore → exit 1; `false` → exit 0 | ⏳ |
| 3 · | `added_by` of someone else in a staged change → exit 1; matching → exit 0; under `--changed-since` the blame author is used | ⏳ |
| 4 · | `until = D-1` and `expires = D` give byte-identical output | ⏳ |
| 5 · | `baseline check --since` fails when `expires` moves later | ⏳ |
| 6 · | SR-BASELINE's UTC+14 / UTC−12 test still byte-identical; L8 budgets unchanged | ⏳ |

## Blockers

None. Independent of W-39; DoD's "never remapped" line is checked once W-39 lands.

## Record affected

SR-BASELINE new decision 9 (and decision 2's table gains the `expires` rows; `[exceptions]` joins SR-CONFIG §1 as SR-BASELINE's) · SR-LINTER decision 6 (inline keys) · SR-CLI-MCP (the new flags on `baseline add`, `deprecate`; MCP twins unchanged).

## Hazards

- **The day boundary.** Today's code errors when `today > until`; `expires` errors when `today >= expires`. Write the off-by-one test first.
- **As-of, not wall clock:** a branch with no new commit won't fail on the day. Say so in USAGE; a scheduled CI job passes `--as-of "$(date +%F)"`.
- **`require_expiry` vs grandfathered React context (W-41):** that baseline then needs an `expires` too.
- `git blame` per file is the slow path — only under `--changed-since`, only on files that hold an exception and changed.

## Reference

[compare doc](../compare/ci-profiles-and-dated-exceptions.compare.md) · [SR-BASELINE](../../records/0104_baseline-and-deprecations.md) · [unicorn `expiring-todo-comments`](https://github.com/sindresorhus/eslint-plugin-unicorn/blob/master/docs/rules/expiring-todo-comments.md)
