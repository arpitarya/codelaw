# `@codelaw/rules-security-html` · `codelaw-rules-security-html`

codelaw's HTML security rules (CSP, mixed content, iframes, forms, template sinks) — one of codelaw's first-party rule packs
([SR-RULE-PACKS](../../records/0101_rule-packs.md)). The same files ship on npm as `@codelaw/rules-security-html` and on PyPI as
`codelaw-rules-security-html`. Its presets **extend** `codelaw:html/security`,
so the html pack's five security rules (`html/require-sri`,
`html/target-blank-rel`, `html/no-inline-script`, `html/autocomplete-password`,
`html/meta-refresh`) run without being copied here; it needs the html pack
(`@codelaw/rules-html` or `codelaw-rules-html`) installed beside it.

```toml
# codelaw.toml
extends = ["codelaw:html/recommended", "codelaw:sec-html/recommended"]
preview = true   # every rule is `preview` until a corpus run promotes it
```

`codelaw init` does not add it: a security pack is chosen, not detected.

## Presets

<!-- docs-sync:presets -->

| preset | rules | ported from |
|---|---|---|
| `sec-html/recommended` | 19 | extends `html/security` (the html pack's five security rules); high-confidence rules, at `error` |
| `sec-html/strict` | 24 | `sec-html/recommended` and the heuristic rules, at `warn` |

<!-- /docs-sync:presets -->

`recommended` holds only high-confidence rules, at `error`; `strict` adds the
heuristic ones, at `warn`, so a team can start without noise.

## Rules

| rule | flags | CWE | OWASP 2025 | fix | preset |
|---|---|---|---|---|---|
| `sec-html/no-inline-event-handler` | `onclick=`, `onerror=` … attributes (they block a strict CSP) | 79 | A05 | manual | `recommended` |
| `sec-html/no-javascript-url` | `javascript:` in `href`, `src`, `action`, `formaction`, `xlink:href`, `<object data>` | 79 | A05 | manual | `recommended` |
| `sec-html/no-mixed-content` | `http://` in `src`, `action`, a stylesheet/script `href`, a download link | 319 | A04 | suggest | `recommended` |
| `sec-html/password-form-post` | a password form with `method=get`, no `method`, or an `http:` action | 598 | A04 | suggest | `recommended` |
| `sec-html/iframe-sandbox` | `<iframe>` without `sandbox`, or `allow-scripts` with `allow-same-origin` | 1021 | A02 | suggest | `recommended` |
| `sec-html/csp-meta-unsafe` | a meta CSP with `'unsafe-inline'`, `'unsafe-eval'`, `*` or `data:` in script sources | 693 | A02 | manual | `recommended` |
| `sec-html/no-credentials-in-url` | `user:pass@host` in a URL attribute | 522 | A07 | manual | `recommended` |
| `sec-html/no-template-html-sink` | Angular `[innerHTML]`, Vue `v-html`, Alpine `x-html` | 79 | A05 | manual | `recommended` |
| `sec-html/referrer-policy` | `referrerpolicy` or `<meta name=referrer>` set to `unsafe-url` or `no-referrer-when-downgrade` | 200 | A02 | suggest | `recommended` |
| `sec-html/no-srcdoc-script` | `<iframe srcdoc>` holding a script, handler or `javascript:` | 79 | A05 | manual | `recommended` |
| `sec-html/sri-crossorigin` | `integrity` on a cross-origin file without `crossorigin` | 353 | A08 | suggest | `recommended` |
| `sec-html/no-secret-in-hidden-input` | a hidden `password`/`token`/`secret`/`api-key` input with a literal value (CSRF tokens excluded) | 798, 200 | A07 | manual | `recommended` |
| `sec-html/angularjs-url-expression` | `href="{{…}}"` / `src="{{…}}"` in an AngularJS page | 79 | A05 | suggest | `recommended` |
| `sec-html/no-unsafe-template-interpolation` | Handlebars `{{{ }}}`, `|safe`, `{% autoescape off %}`, EJS `<%-` | 79 | A05 | manual | `recommended` |
| `sec-html/no-external-base` | `<base href>` on another origin | — | A05 | manual | `strict` |
| `sec-html/no-plugin-embed` | `<object>`, `<embed>`, `<applet>` | — | A02 | manual | `strict` |
| `sec-html/no-sensitive-comments` | an HTML comment with a password, key, internal host or a note to remove | 615 | A02 | auto | `strict` |
| `sec-html/form-csrf-token` | a `method=post` form with no CSRF field (extra names: `names` option) | 352 | A01 | manual | `strict` |
| `sec-html/iframe-allow-powerful` | a cross-origin `<iframe allow="camera; microphone; …">` | 250 | A02 | manual | `strict` |

Each rule is native (a Rust implementation in the engine, read with the same
document model as the `html/*` rules) and is pattern-level: one start tag,
one attribute, one comment or one run of template syntax. A value that is
template code (`{{ }}`, `<% %>`) is judged by its literal prefix only, and
the AngularJS rule runs only on a page that shows AngularJS. The catalogue
and its sources are [W-56](../../archive/open/W-56-security-html-pack.md) and the
OWASP HTML5, CSP and Cross-Site Request Forgery cheat sheets.

`sec-html/no-inline-event-handler` and `sec-html/no-javascript-url` overlap
`html/no-inline-script`, which the preset also turns on: a handler is
reported twice, once with its CWE. Turn either off in `[rules]` if one report
is enough.

`suggest` fixes (`https://`, `method="post"`, `sandbox`, `crossorigin`,
`referrerpolicy`) apply with `codelaw fix --unsafe`. A fix is not applied when
it would add a finding the file did not have: under `strict`, turning a form
to POST is refused while `sec-html/form-csrf-token` has nothing to see.

Every rule here carries `security: { cwe, owasp, severity }`
([SR-RULE-FORMAT](../../records/0112_rule-format.md) decision 2a), so

- its finding names the CWE (`sec-html/<rule> (CWE-79) …`);
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
