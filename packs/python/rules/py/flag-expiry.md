---
id: py/flag-expiry
title: "Every feature flag read is registered, with an owner and a date it goes"
languages: [python]
scope: project
severity: error
fix: suggest
fix_reason: "Keeping one branch of a flag removes the other branch's code; a person or agent accepts the diff."
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

Each call that reads a feature flag names a flag that is registered — in `.codelaw/flags.toml` or by an inline `codelaw-flag` marker — and that has not expired on the as-of date. The rule is on when the root config has a `[flags]` table (SR-PROJECT-GRAPH decision 11), and reads nothing until `[flags]` names a preset or an accessor. It is `js/flag-expiry` for Python: the same `[flags]`, registry, markers and findings.

```toml
[flags]
presets   = ["unleash"]
accessors = ["flags.is_on($NAME)", "feature(key=$NAME)"]
```

A function whose parameter flows unchanged into an accessor's `$NAME` is an accessor too, a method included (`self` aside), found through the project graph across modules and imports; `codelaw flags accessors` lists them. A flag's name is a string literal, or a module constant or class attribute (an `Enum` member, `.value` or not) the graph resolves; a name computed at run time is reported as `info`, uncertain.

| at a read of a flag that is | finding |
|---|---|
| not registered | `error` |
| expiring within `warn_days` of the as-of date | `warn` |
| past its `expires` on the as-of date | `error` |
| decided, `keep = "on"` or `"off"` | `warn`, with the fix |
| `expires = "never"` with a `reason` | nothing |
| read with a computed name | `info`, uncertain |

Dates are judged on the as-of date, never the clock, and no flag service is ever asked (L5).

**Why:** a flag left in the code after its rollout is a branch nobody tests; a flag with an owner and a date is one someone removes.

```codelaw
graph:
  flags: reads
message: "A feature flag read here is unregistered, expiring, expired or decided"
fix:
  do: [keep-branch]
```

## How to fix

1. Unregistered: add the flag to `.codelaw/flags.toml`, or mark it above the read: `# codelaw-flag <name> expires YYYY-MM-DD owner @team`.
2. Expiring or expired: set `keep = "on"` or `"off"` on its entry and run `codelaw fix --unsafe`: an `if` or a conditional expression whose whole condition is the read keeps that branch, dedented. Elsewhere, keep the branch by hand.
3. When the flag has no read left, `codelaw fix` removes its registry entry (`data/flag-registry`).

## Examples

### Python

#### Don't

```python files options={"accessors":["is_feature_on($NAME)"]}
# .codelaw/flags.toml
[[flag]]
name = "new-checkout"
owner = "@payments"
expires = "2026-12-31"
keep = "on"
# app/checkout.py
def total(cart):
    if is_feature_on("new-checkout"):
        return new_total(cart)
    else:
        return old_total(cart)
```

#### Do

```python files options={"accessors":["is_feature_on($NAME)"]}
# .codelaw/flags.toml
[[flag]]
name = "new-checkout"
owner = "@payments"
expires = "2026-12-31"
keep = "on"
# app/checkout.py
def total(cart):
    return new_total(cart)
```
