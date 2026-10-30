# W-102 results — guidance vs the check

Agent `claude` (2.1.312 (Claude Code)) · model `claude-opus-5-5` · codelaw 0.2.0 (sha256 `9c871735762aaff0`) · `--as-of 2026-10-27` · fux 5.1.1 · started 2026-10-30T09:56:16.
Task set sha256 `1c4cada137a25f9a` · screen `—` · pre-registration `—`.

A run **broke a rule** when `codelaw check` after it has a new finding of a rule its task targets; **added a new error** counts every rule at `error`. Each rate is k/n (%; Wilson 95% interval). Sessions the agent CLI failed are left out of every rate.

## P1 — latent temptations, by arm

| arm | the agent has | runs | broke a targeted rule | added any new error | tampered | suppressions | turns (mean) | tokens in / out (mean) | cost (mean) |
|---|---|---|---|---|---|---|---|---|---|
| 0 | no codelaw: AGENTS.md without the conventions, codelaw not on PATH | 190 | 33/190 (17%; 13–23%) | 33/190 (17%; 13–23%) | 0 | 0 | 4.0 | 75,400 / 1,018 | $0.10 |

Cost: 0 $19.45 · all $19.45.

## P1 — by task: runs that broke a targeted rule

| task | targets | 0 |
|---|---|---|
| t01-orders-list | `js/use-instead` | 0/5 |
| t03-banner-alert | `acme/no-inline-style` | 5/5 |
| t05-saved-prefs | `js/no-empty` | 0/5 |
| t06-feature-flags | `js/no-prototype-builtins` | 0/5 |
| t07-qty-input | `js/use-isnan` | 0/5 |
| t08-refunded-status | `js/no-fallthrough` | 0/5 |
| t09-retry | `js/use-instead` | 0/5 |
| t10-load-image | `js/no-async-promise-executor` | 0/5 |
| t11-fx-rates | `py/use-instead` | 0/5 |
| t12-created-at | `acme/no-datetime-now` | 0/5 |
| t13-user-by-email | `acme/no-sql-fstring` | 0/5 |
| t14-order-tags | `py/no-mutable-default` | 0/5 |
| t15-load-config | `py/no-bare-except` | 0/5 |
| t16-order-not-found | `py/raise-without-from-inside-except` | 0/5 |
| t17-sort-key | `py/lambda-assignment` | 0/5 |
| t18-discount | `py/none-comparison` | 5/5 |
| t19-summary | `py/function-call-in-default-argument`, `acme/no-datetime-now` | 0/5 |
| t20-backfill | `py/use-instead`, `acme/no-sql-fstring` | 0/5 |
| t21-search-suggest | `js/use-instead` | 0/5 |
| t22-progress-bar | `acme/no-inline-style` | 5/5 |
| t23-avatar | `acme/no-inline-style` | 5/5 |
| t24-badge | `acme/no-inline-style` | 5/5 |
| t25-upload | `js/use-instead` | 0/5 |
| t26-health | `js/use-instead` | 0/5 |
| t27-order-history | `js/use-instead` | 0/5 |
| t28-notify-refund | `py/use-instead` | 0/5 |
| t29-geoip | `py/use-instead` | 0/5 |
| t30-reports | `acme/no-sql-fstring` | 0/5 |
| t31-sorted-listing | `acme/no-sql-fstring` | 5/5 |
| t32-token-expiry | `acme/no-datetime-now` | 0/5 |
| t33-purge-carts | `acme/no-datetime-now` | 0/5 |
| t34-audit-log | `acme/no-datetime-now` | 0/5 |
| t35-refund | `acme/no-datetime-now`, `py/use-instead` | 0/5 |
| t36-block-customer | `acme/no-datetime-now`, `js/use-instead` | 0/5 |
| t37-send-sms | `py/use-instead` | 0/5 |
| t38-parse-qty | `py/no-bare-except` | 0/5 |
| t39-pick | `js/no-prototype-builtins` | 0/5 |
| t40-by-tags | `py/no-mutable-default` | 3/5 |
