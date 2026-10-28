# W-102 results — guidance vs the check

Agent `claude` · model `claude-opus-5-5` · codelaw 0.2.0 · started 2026-10-28T08:59:51.
A run **broke a rule** when `codelaw check` after it has a new finding of a rule its task targets; **new errors** counts every rule. Rates are over the runs scored, never one run.

| arm | what it adds | runs | broke a targeted rule | added any new error | targeted findings | tampered | suppressions | turns (mean) | tokens in / out (mean) | cost (mean) |
|---|---|---|---|---|---|---|---|---|---|---|
| A | AGENTS.md only | 20 | 2/20 (10%) | 2/20 (10%) | 4 | 20 | 0 | 4.3 | 81,993 / 1,105 | $0.11 |
| B | A + the codelaw MCP server and the rule pages | 20 | 2/20 (10%) | 2/20 (10%) | 4 | 20 | 0 | 9.3 | 153,777 / 1,896 | $0.20 |
| C | B + a Fux index over the docs and the rule pages | 20 | 2/20 (10%) | 2/20 (10%) | 4 | 20 | 0 | 8.6 | 161,598 / 1,868 | $0.21 |

Total cost: A $2.21, B $3.96, C $4.30 · all $10.47.

## By task — runs that broke a targeted rule

| task | targets | A | B | C |
|---|---|---|---|---|
| t01-orders-list | `js/use-instead` | 0/1 | 0/1 | 0/1 |
| t02-cart-debug | `js/no-console` | 1/1 | 1/1 | 1/1 |
| t03-banner-alert | `acme/no-inline-style` | 0/1 | 0/1 | 0/1 |
| t04-format-price | `js/no-unused-vars` | 1/1 | 1/1 | 1/1 |
| t05-saved-prefs | `js/no-empty` | 0/1 | 0/1 | 0/1 |
| t06-feature-flags | `js/no-prototype-builtins` | 0/1 | 0/1 | 0/1 |
| t07-qty-input | `js/use-isnan` | 0/1 | 0/1 | 0/1 |
| t08-refunded-status | `js/no-fallthrough` | 0/1 | 0/1 | 0/1 |
| t09-retry | `js/use-instead` | 0/1 | 0/1 | 0/1 |
| t10-load-image | `js/no-async-promise-executor` | 0/1 | 0/1 | 0/1 |
| t11-fx-rates | `py/use-instead` | 0/1 | 0/1 | 0/1 |
| t12-created-at | `acme/no-datetime-now` | 0/1 | 0/1 | 0/1 |
| t13-user-by-email | `acme/no-sql-fstring` | 0/1 | 0/1 | 0/1 |
| t14-order-tags | `py/no-mutable-default` | 0/1 | 0/1 | 0/1 |
| t15-load-config | `py/no-bare-except` | 0/1 | 0/1 | 0/1 |
| t16-order-not-found | `py/raise-without-from-inside-except` | 0/1 | 0/1 | 0/1 |
| t17-sort-key | `py/lambda-assignment` | 0/1 | 0/1 | 0/1 |
| t18-discount | `py/none-comparison` | 0/1 | 0/1 | 0/1 |
| t19-summary | `py/function-call-in-default-argument`, `acme/no-datetime-now` | 0/1 | 0/1 | 0/1 |
| t20-backfill | `py/use-instead`, `acme/no-sql-fstring` | 0/1 | 0/1 | 0/1 |

## Step 4 — the README line

The "rules it can't skip" line is **stays**, with the measured rate: in 2 of 20 runs an agent with the best guidance still added an error.

## Pilot notes (by hand, 2026-10-28)

The pilot: `--runs 1`, 60 sessions, Claude Code 2.1.310, `claude-opus-5-5`, codelaw 0.2.0 (release
build of `7b839af`'s tree, SHA-256 `9c871735…0a503`), 26 minutes in all. Read it as a pipeline check,
not as evidence: one run per cell.

- **`tampered` is wrong in this pilot.** Every run reads tampered because the agent's own
  `codelaw check` (codelaw is on every run's `PATH`) writes `.codelaw/cache/`; the scorer now
  ignores the cache, after these runs. The findings are unaffected: the protected files were
  restored and the after-check reads no cache. One run is known to have really tampered — A t04
  says it edited `codelaw.toml` to allow `_`-prefixed parameters; restored, it scored as broken.
- **Both breaks are in every arm.** t02 (debug logging: `console.log`) and t04 (unused parameters
  kept in a signature) — the request itself needs the violation, and in t02 and C t04 the agent said
  the check would fail. The other 18 tasks tempted no arm.
- **Every arm can already run the check.** `codelaw` is on `PATH` and `AGENTS.md` names
  `codelaw check`, so arm A is "guidance + a check the agent may choose to run", not "guidance only".
