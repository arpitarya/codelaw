---
type: Handoff
name: W-56
description: "Arpit, 2026-10-25: the security-html pack: inline handlers, javascript: URLs, mixed content, iframe sandboxing, CSP meta, password forms, Angular/Vue HTML sinks; its preset also turns on the HTML pack's existing security rules."
item: W-56
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-25
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-56 — `security-html` — HTML security rules (CSP, mixed content, iframes, forms, template sinks)

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Write the catalogue below with cases | the rules | ✅ |
| 2 | `recommended` also enables `html/require-sri`, `html/target-blank-rel`, `html/no-inline-script`, `html/autocomplete-password`, `html/meta-refresh` | no duplicates | ✅ |
| 3 | README and corpus | adoption | ✅ |

**Arpit, 2026-10-25:** *"I want to create a pack for security issues. Security issues like Veracode, Black Duck, X-Ray, OWASP. They would have some suggestions on how to write, what to write, what not to write in JavaScript and Python. So create … one for JavaScript, one for TypeScript, one for HTML, one for Python. Research and create work items for individual languages."*

**Model:** Sonnet — tree-sitter-html matches with a written catalogue.

## Goal

| rule | flags | CWE | OWASP 2025 | fix |
|---|---|---|---|---|
| `sec-html/no-inline-event-handler` | `onclick=`, `onerror=` … attributes (blocks a strict CSP) | 79 | A05 | manual: addEventListener |
| `sec-html/no-javascript-url` | `href`/`src`/`action` = `javascript:` | 79 | A05 | manual |
| `sec-html/no-mixed-content` | `http://` in `src`, `href` of stylesheets/scripts, `action`, `iframe` | 319 | A04 | suggest: `https://` |
| `sec-html/password-form-post` | a form holding a password input with `method=get` or an `http:` action | 598 | A04 | suggest: `method=post` |
| `sec-html/iframe-sandbox` | `<iframe>` without `sandbox`, or with `allow-scripts` + `allow-same-origin` | 1021 | A02 | suggest: add `sandbox` |
| `sec-html/csp-meta-unsafe` | `<meta http-equiv=Content-Security-Policy>` with `unsafe-inline`, `unsafe-eval`, `*` or `data:` in script-src | 693 | A02 | manual |
| `sec-html/no-external-base` | `<base href>` pointing off-site | — | A05 | manual |
| `sec-html/no-plugin-embed` | `<object>`, `<embed>`, `<applet>` | — | A02 | manual |
| `sec-html/no-credentials-in-url` | `user:pass@host` in any URL attribute | 522 | A07 | manual |
| `sec-html/no-template-html-sink` | Angular `[innerHTML]`, Vue `v-html` in `.html` templates | 79 | A05 | manual |
| `sec-html/referrer-policy` | links or a page leaking full URLs (`referrerpolicy=unsafe-url`) | 200 | A02 | suggest |

### Added after the second research pass (2026-10-25) — 8 more

Sources: CodeQL's query list (read for ideas; MIT, nothing copied), Ruff's `S` rules and Bandit (Apache-2.0/MIT, portable with `ported_from:`), the OWASP cheat sheets. `preset` says where each rule starts: `recommended` (`error`) or `strict` (`warn`, heuristic).

| rule | flags | CWE | OWASP 2025 | fix | preset |
|---|---|---|---|---|---|
| `sec-html/no-srcdoc-script` | `<iframe srcdoc>` containing `<script>` or an event handler | 79 | A05 | manual | recommended |
| `sec-html/sri-crossorigin` | `integrity` on a cross-origin `script`/`link` without `crossorigin` | 353 | A08 | suggest: `crossorigin=anonymous` | recommended |
| `sec-html/no-secret-in-hidden-input` | a hidden input named password/token/secret/api-key with a literal value (CSRF tokens excluded) | 798, 200 | A07 | manual | recommended |
| `sec-html/no-sensitive-comments` | HTML comments holding passwords, keys, internal hosts or 'TODO remove' | 615 | A02 | auto: remove the comment | strict |
| `sec-html/form-csrf-token` | a `method=post` form with no hidden CSRF field (framework names configurable) | 352 | A01 | manual | strict |
| `sec-html/angularjs-url-expression` | `href="{{…}}"`/`src="{{…}}"` | 79 | A05 | suggest: `ng-href`/`ng-src` | recommended |
| `sec-html/no-unsafe-template-interpolation` | Handlebars `{{{ }}}`, Jinja/Nunjucks `|safe`, Django `{% autoescape off %}`, EJS `<%-` in `.html` templates | 79 | A05 | manual | recommended |
| `sec-html/iframe-allow-powerful` | `<iframe allow="camera; microphone; geolocation; payment">` on a cross-origin `src` | 250 | A02 | manual | strict |

Also widen `sec-html/no-javascript-url`'s attributes: `formaction`, `xlink:href`, `<object data>`; and `sec-html/no-mixed-content` covers `<a download href="http://…">`.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | `codelaw test packs/security-html` green | ✅ |
| 2 · | the preset enables the five existing `html/*` rules, shown by `codelaw rules --for x.html` | ✅ |

## Blockers

W-53 landed 2026-10-25. Closed 2026-10-26: see `work/IMPLEMENTATION.md`.

## Record affected

SR-RULE-PACKS (the pack's row; npm and PyPI).

## Hazards

- **No copyleft copying (L2):** Bandit, eslint-plugin-security and Ruff's `S` rules (Apache-2.0 / MIT) may be ported with `ported_from:` and NOTICE; eslint-plugin-no-unsanitized (MPL-2.0), njsscan (LGPL) and Semgrep's rules (Semgrep Rules License) are **reference only** — read, never copied.
- **Pattern-level, not taint analysis:** every rule fires on a dangerous sink with a non-literal argument, or with a known request source in the same function (W-53's predicate). False positives go to `security-*/strict` at `warn`; `recommended` holds only high-confidence rules at `error`.
- Every rule carries `security:` metadata (CWE, OWASP, severity) from W-53; L4: every `manual` fix names the safe API in its steps.
- Templates (Jinja, Handlebars) in `.html` files: `{{ }}` regions are not HTML — don't flag inside them.

## Reference

[OWASP HTML5 cheat sheet](https://cheatsheetseries.owasp.org/cheatsheets/HTML5_Security_Cheat_Sheet.html) · [MDN Subresource Integrity](https://developer.mozilla.org/en-US/docs/Web/Security/Subresource_Integrity) · [CodeQL JS queries](https://codeql.github.com/codeql-query-help/javascript/) · [OWASP HTML5 cheat sheet](https://cheatsheetseries.owasp.org/cheatsheets/HTML5_Security_Cheat_Sheet.html) · [OWASP CSP cheat sheet](https://cheatsheetseries.owasp.org/cheatsheets/Content_Security_Policy_Cheat_Sheet.html)
