---
id: data/flag-registry
title: "The feature-flag registry is valid and holds only flags still read"
languages: [toml]
files: [".codelaw/flags.toml"]
scope: project
severity: error
fix: auto
stability: preview
options:
  presets:
    type: "string[]"
    description: "`[flags] presets`, for this rule only: the SDKs whose reads codelaw knows (launchdarkly, unleash, split, openfeature, growthbook, flagsmith)"
  accessors:
    type: "string[]"
    description: "`[flags] accessors`, for this rule only: calls with one `$NAME` slot that read a flag"
  inline:
    type: "string"
    description: "`[flags] inline`, for this rule only: `allow` or `registry-only`"
  warn_days:
    type: "integer"
    description: "`[flags] warn_days`, for this rule only: how many days before `expires` a read warns"
since: 0.5.0
---

`.codelaw/flags.toml` registers each feature flag the code reads, with who owns it and when it goes:

```toml
[[flag]]
name    = "new-checkout"
owner   = "@payments"
ticket  = "PAY-123"
expires = "2027-02-28"   # a date, or "never" with a reason
keep    = "on"           # optional: the branch that stays — `js/flag-expiry` and `py/flag-expiry` collapse to it
reason  = "…"
```

The rule is on when the root config has a `[flags]` table (SR-PROJECT-GRAPH decision 11), and judges the registry once `[flags]` names a preset or an accessor. It reports, in the registry:

| an entry that | finding | fix |
|---|---|---|
| no read in the project names any more | `info` | `auto`: the entry is removed — `info`, uncertain and not removed while some read computes its flag's name |
| has no `name`, a key a flag does not have, or registers a flag twice | `error` | by hand |
| has no `expires`, one that is neither a date nor `never`, `never` without a `reason`, or `keep` other than `on`/`off` | `error` | by hand |
| — the file cannot be read as TOML | `error` | by hand |

**Why:** an entry for a flag the code no longer reads is a promise nobody keeps, and it hides the entries that matter.

```codelaw
graph:
  flags: registry
message: "This flag registry entry is invalid or has no read left"
fix:
  do: [remove-entry]
```

## How to fix

1. Run `codelaw fix`: an entry with no read left is removed.
2. Otherwise edit the entry: give it a `name`, an `expires` date (or `never` and a `reason`), `keep = "on"` or `"off"` only, and no other keys.

## Examples

### TOML

#### Don't

```toml files options={"accessors":["isFeatureOn($NAME)"]}
# .codelaw/flags.toml
[[flag]]
name = "new-checkout"
owner = "@payments"
expires = "2027-02-28"

[[flag]]
name = "old-banner"
owner = "@web"
expires = "2026-06-30"
# src/checkout.js
export const on = isFeatureOn("new-checkout");
```

#### Do

```toml files options={"accessors":["isFeatureOn($NAME)"]}
# .codelaw/flags.toml
[[flag]]
name = "new-checkout"
owner = "@payments"
expires = "2027-02-28"
# src/checkout.js
export const on = isFeatureOn("new-checkout");
```
