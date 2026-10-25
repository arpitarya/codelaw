# `codelaw-rules-security-python`

codelaw's Python security rules (Bandit's set, Django, Flask, FastAPI, ML loaders) — one of codelaw's first-party rule packs
([SR-RULE-PACKS](../../records/0101_rule-packs.md)). It ships on PyPI only, as `codelaw-rules-security-python` (L6 decision 11).

```toml
# codelaw.toml
extends = ["codelaw:py/recommended", "codelaw:sec-py/recommended"]
preview = true   # every rule is `preview` until a corpus run promotes it
```

`codelaw init` does not add it: a security pack is chosen, not detected.

## Presets

<!-- docs-sync:presets -->

| preset | rules | ported from |
|---|---|---|
| `sec-py/recommended` | 0 | high-confidence rules, at `error` |
| `sec-py/strict` | 0 | `sec-py/recommended` and the heuristic rules, at `warn` |

<!-- /docs-sync:presets -->

`recommended` holds only high-confidence rules, at `error`; `strict` adds the
heuristic ones, at `warn`, so a team can start without noise.

## Rules

None yet: this is the pack's scaffold (W-53). The catalogue, each rule with
its CWE, OWASP 2025 category and fix class, is [W-57](../../work/open/W-57-security-python-pack.md).

Every rule here carries `security: { cwe, owasp, severity }`
([SR-RULE-FORMAT](../../records/0112_rule-format.md) decision 2a), so

- its finding names the CWE (`sec-py/<rule> (CWE-79) …`);
- `codelaw rules --tag cwe:79` and `--tag owasp:A05` find it, and
  `codelaw rules --security-coverage` puts it in the OWASP Top 10:2025 matrix;
- SARIF carries its `security` and `external/cwe/cwe-<n>` tags and a
  `security-severity` that GitHub code scanning ranks by
  ([SR-CLI-MCP](../../records/0115_cli-and-mcp.md) decision 4).

## `from_source:` is not taint analysis

A rule here that asks whether a sink's argument comes from a request
uses the `from_source:` predicate ([SR-RULE-FORMAT](../../records/0112_rule-format.md)
decision 3a). It holds when the argument **is** a listed request source
(or a method called on one), is built from one by concatenation,
interpolation or `?:`, or is a local name bound to one **in the same
function** — one or more assignments or destructurings away. It never
follows a call, a return value, a parameter, a closure's outer variable or
another file, so a value that crosses a function boundary is not seen
(cross-function flow waits on a deeper analysis, as B-049's type-aware
rules do). Every rule
page that uses it says so. The sources are listed per framework in this
pack's `data/sources/<framework>.json`.
