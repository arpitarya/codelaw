# `@codelaw/rules-security-js` · `codelaw-rules-security-js`

codelaw's JavaScript (Node, the browser, Express) security rules: 66 of them, each mapped to a CWE and an OWASP Top 10:2025 category, each with
a fix class and examples in all four languages. One of codelaw's first-party rule packs
([SR-RULE-PACKS](../../records/0101_rule-packs.md)). The same files ship on npm as `@codelaw/rules-security-js` and on PyPI as
`codelaw-rules-security-js`: a Python repository with JavaScript in it gets them too.

```toml
# codelaw.toml
extends = ["codelaw:js/recommended", "codelaw:sec-js/recommended"]
preview = true   # every rule is `preview` until a corpus run promotes it
```

`codelaw init` does not add it: a security pack is chosen, not detected. The rules run on JS, JSX, TS and TSX
files; the TypeScript-only rules (Angular, NestJS, Prisma, typed casts) are in
[`security-ts`](../security-ts/README.md), which extends this pack.

## Presets

<!-- docs-sync:presets -->

| preset | rules | ported from |
|---|---|---|
| `sec-js/recommended` | 55 | high-confidence rules, at `error` |
| `sec-js/strict` | 66 | `sec-js/recommended` and the heuristic rules, at `warn` |

<!-- /docs-sync:presets -->

`recommended` holds the 55 high-confidence rules, at `error`; `strict` adds the 11 heuristic ones, at `warn`, so a team can
start without noise. 17 rules offer a mechanical fix (`auto` 0, `suggest` 17); the other
49 are `manual`, and each names the safe API in its `How to fix` steps, which the `codelaw-fix` skill walks through.

## Rules

Every rule carries `security: { cwe, owasp, severity }` ([SR-RULE-FORMAT](../../records/0112_rule-format.md) decision 2a), so

- its finding names the CWE (`sec-js/<rule> (CWE-79) …`);
- `codelaw rules --tag cwe:79` and `--tag owasp:A05` find it, and `codelaw rules --security-coverage` puts it in the OWASP Top 10:2025 matrix;
- SARIF carries its `security` and `external/cwe/cwe-<n>` tags and a `security-severity` that GitHub code scanning ranks by
  ([SR-CLI-MCP](../../records/0115_cli-and-mcp.md) decision 4).

### Injection and code execution

| rule | rule says | CWE | OWASP 2025 | fix | preset |
|---|---|---|---|---|---|
| [`sec-js/no-eval`](rules/sec-js/no-eval.md) | No eval, new Function or string timers | 94 | A05 | manual | recommended |
| [`sec-js/no-child-process-shell`](rules/sec-js/no-child-process-shell.md) | No shell command built from a value | 78 | A05 | manual | recommended |
| [`sec-js/no-sql-template`](rules/sec-js/no-sql-template.md) | No SQL built from a template or concatenation | 89 | A05 | manual | recommended |
| [`sec-js/no-nosql-injection`](rules/sec-js/no-nosql-injection.md) | No request object passed straight to a query, no $where | 943 | A05 | manual | recommended |
| [`sec-js/no-xpath-injection`](rules/sec-js/no-xpath-injection.md) | No XPath built from a template or concatenation | 643 | A05 | manual | recommended |
| [`sec-js/no-ldap-injection`](rules/sec-js/no-ldap-injection.md) | No LDAP filter built from a template or concatenation | 90 | A05 | manual | recommended |
| [`sec-js/no-ssti`](rules/sec-js/no-ssti.md) | No template compiled from a request, no template data taken whole from a request | 1336, 73 | A05 | manual | recommended |
| [`sec-js/no-header-injection`](rules/sec-js/no-header-injection.md) | No response header value taken from a request | 113 | A05 | manual | recommended |
| [`sec-js/no-log-injection`](rules/sec-js/no-log-injection.md) | No request data written to a log unchanged | 117 | A09 | manual | strict |
| [`sec-js/no-format-string`](rules/sec-js/no-format-string.md) | No request data as a format string | 134 | A05 | suggest | strict |
| [`sec-js/no-non-literal-require`](rules/sec-js/no-non-literal-require.md) | No require or import of a computed module name | 829, 95 | A08 | manual | recommended |
| [`sec-js/no-unvalidated-dynamic-method`](rules/sec-js/no-unvalidated-dynamic-method.md) | No method chosen by a request value | 470 | A05 | manual | recommended |
| [`sec-js/no-xxe`](rules/sec-js/no-xxe.md) | No XML entity expansion | 611, 776 | A05 | suggest | recommended |
| [`sec-js/no-unsafe-deserialize`](rules/sec-js/no-unsafe-deserialize.md) | No deserializer that builds objects or runs code | 502 | A08 | suggest | recommended |
| [`sec-js/no-new-buffer`](rules/sec-js/no-new-buffer.md) | No new Buffer, no noAssert | 119 | A06 | suggest | recommended |
| [`sec-js/no-unsafe-regex`](rules/sec-js/no-unsafe-regex.md) | No regular expression that can run for ever, no regular expression built from a request | 1333 | A06 | manual | strict |
| [`sec-js/no-prototype-pollution`](rules/sec-js/no-prototype-pollution.md) | No request value used as a key to write or merge | 1321 | A08 | manual | recommended |

### Cross-site scripting and the browser

| rule | rule says | CWE | OWASP 2025 | fix | preset |
|---|---|---|---|---|---|
| [`sec-js/no-unsanitized-html`](rules/sec-js/no-unsanitized-html.md) | No HTML written from a value that is not a literal | 79 | A05 | suggest | recommended |
| [`sec-js/no-dangerously-set-inner-html`](rules/sec-js/no-dangerously-set-inner-html.md) | No dangerouslySetInnerHTML from a value that is not sanitized | 79 | A05 | manual | recommended |
| [`sec-js/no-javascript-url`](rules/sec-js/no-javascript-url.md) | No javascript: URL, no navigation to a URL taken from the address | 79 | A05 | manual | recommended |
| [`sec-js/postmessage-origin`](rules/sec-js/postmessage-origin.md) | postMessage names its target, a message handler checks the sender | 346 | A01 | manual | recommended |
| [`sec-js/no-jquery-html-sink`](rules/sec-js/no-jquery-html-sink.md) | No jQuery call that parses a built string as HTML | 79 | A05 | manual | recommended |
| [`sec-js/no-window-open-opener`](rules/sec-js/no-window-open-opener.md) | No window.open without noopener | 1022 | A05 | suggest | recommended |
| [`sec-js/no-dynamic-script-src`](rules/sec-js/no-dynamic-script-src.md) | No script or frame source taken from a value that is not a literal | 829 | A08 | manual | recommended |
| [`sec-js/no-incomplete-url-check`](rules/sec-js/no-incomplete-url-check.md) | No URL checked with a substring test | 20 | A01 | manual | recommended |
| [`sec-js/no-incomplete-sanitization`](rules/sec-js/no-incomplete-sanitization.md) | No escaping that stops at the first match, no one-pass stripping | 116 | A05 | suggest | recommended |
| [`sec-js/no-bad-tag-filter`](rules/sec-js/no-bad-tag-filter.md) | No regular expression that filters HTML tags | 116, 185 | A05 | manual | recommended |
| [`sec-js/angularjs-sce-disabled`](rules/sec-js/angularjs-sce-disabled.md) | No AngularJS strict contextual escaping switched off | 79 | A05 | manual | strict |
| [`sec-js/no-bidi-characters`](rules/sec-js/no-bidi-characters.md) | No hidden bidirectional control characters in code | — | A08 | suggest | recommended |
| [`sec-js/no-document-cookie-token`](rules/sec-js/no-document-cookie-token.md) | No token or session written to document.cookie | 1004 | A07 | manual | recommended |
| [`sec-js/no-token-in-web-storage`](rules/sec-js/no-token-in-web-storage.md) | No token kept in localStorage or sessionStorage | 922 | A07 | manual | recommended |
| [`sec-js/no-server-env-in-client`](rules/sec-js/no-server-env-in-client.md) | No server environment variable in a client component, no secret in a public one | 200 | A02 | manual | recommended |
| [`sec-js/no-electron-insecure`](rules/sec-js/no-electron-insecure.md) | No insecure Electron window setting | 79, 829 | A02 | suggest | recommended |

### Access, requests and the server

| rule | rule says | CWE | OWASP 2025 | fix | preset |
|---|---|---|---|---|---|
| [`sec-js/no-path-traversal`](rules/sec-js/no-path-traversal.md) | No file path taken from a request | 22 | A01 | manual | recommended |
| [`sec-js/no-archive-slip`](rules/sec-js/no-archive-slip.md) | No archive entry name written to disk unchecked | 22 | A01 | manual | recommended |
| [`sec-js/no-open-redirect`](rules/sec-js/no-open-redirect.md) | No redirect to an address taken from a request | 601 | A01 | manual | recommended |
| [`sec-js/no-ssrf`](rules/sec-js/no-ssrf.md) | No request made to an address taken from a request | 918 | A01 | manual | recommended |
| [`sec-js/no-tls-bypass`](rules/sec-js/no-tls-bypass.md) | No switching off of certificate checks | 295 | A04 | suggest | recommended |
| [`sec-js/no-cleartext-http`](rules/sec-js/no-cleartext-http.md) | No request made over http:// | 319 | A04 | manual | recommended |
| [`sec-js/no-cors-wildcard-credentials`](rules/sec-js/no-cors-wildcard-credentials.md) | No CORS that allows every origin together with credentials | 942 | A02 | manual | recommended |
| [`sec-js/express-hardening`](rules/sec-js/express-hardening.md) | An Express app turns off x-powered-by or uses helmet | 200 | A02 | suggest | strict |
| [`sec-js/helmet-config`](rules/sec-js/helmet-config.md) | helmet is not given false for a header it sets | 693 | A02 | manual | recommended |
| [`sec-js/cookie-flags`](rules/sec-js/cookie-flags.md) | A cookie sets httpOnly, secure and sameSite | 614, 1004 | A02 | suggest | recommended |
| [`sec-js/express-session-secret`](rules/sec-js/express-session-secret.md) | express-session has a secret from the environment and a secure cookie | 798, 614 | A07 | manual | recommended |
| [`sec-js/no-csrf-before-method-override`](rules/sec-js/no-csrf-before-method-override.md) | No method-override before the CSRF check | 352 | A01 | suggest | recommended |
| [`sec-js/missing-rate-limit`](rules/sec-js/missing-rate-limit.md) | A login or reset route has a rate limit | 770, 307 | A07 | manual | strict |
| [`sec-js/no-sensitive-get-query`](rules/sec-js/no-sensitive-get-query.md) | No password, token or secret read from the query string | 598 | A04 | manual | recommended |
| [`sec-js/no-mass-assignment`](rules/sec-js/no-mass-assignment.md) | No request body written straight into a model | 915 | A01 | manual | recommended |
| [`sec-js/no-user-controlled-auth-bypass`](rules/sec-js/no-user-controlled-auth-bypass.md) | No privileged branch decided by a request value | 807 | A01 | manual | strict |
| [`sec-js/upload-limits`](rules/sec-js/upload-limits.md) | multer has size limits and a file filter | 434, 770 | A06 | manual | recommended |
| [`sec-js/graphql-hardening`](rules/sec-js/graphql-hardening.md) | A GraphQL server limits query depth and cost, and hides introspection | 200, 770 | A02 | manual | strict |
| [`sec-js/no-non-literal-fs-filename`](rules/sec-js/no-non-literal-fs-filename.md) | No file path that is not a literal | 22 | A01 | manual | strict |
| [`sec-js/no-stack-trace-response`](rules/sec-js/no-stack-trace-response.md) | No error object or stack trace in a response | 209 | A10 | manual | recommended |

### Cryptography, secrets and models

| rule | rule says | CWE | OWASP 2025 | fix | preset |
|---|---|---|---|---|---|
| [`sec-js/no-weak-hash`](rules/sec-js/no-weak-hash.md) | No MD5 or SHA-1 | 327 | A04 | manual | recommended |
| [`sec-js/no-weak-cipher`](rules/sec-js/no-weak-cipher.md) | No createCipher, no DES, RC4, Blowfish or ECB | 327 | A04 | manual | recommended |
| [`sec-js/no-weak-key-size`](rules/sec-js/no-weak-key-size.md) | No short RSA key, no weak elliptic curve | 326 | A04 | suggest | recommended |
| [`sec-js/no-hardcoded-crypto-key`](rules/sec-js/no-hardcoded-crypto-key.md) | No key or IV written into the source | 321, 329 | A04 | manual | recommended |
| [`sec-js/no-biased-random`](rules/sec-js/no-biased-random.md) | No byte taken modulo a number that does not divide 256 | 327 | A04 | suggest | recommended |
| [`sec-js/no-insecure-random`](rules/sec-js/no-insecure-random.md) | No Math.random for a token, a secret or a password | 338 | A04 | suggest | recommended |
| [`sec-js/password-hash-effort`](rules/sec-js/password-hash-effort.md) | A password is stored with a slow, salted hash | 916 | A04 | manual | recommended |
| [`sec-js/no-hardcoded-secret`](rules/sec-js/no-hardcoded-secret.md) | No secret written into the source | 798 | A07 | manual | recommended |
| [`sec-js/timing-safe-compare`](rules/sec-js/timing-safe-compare.md) | A token, hash or signature is compared in constant time | 208 | A07 | suggest | strict |
| [`sec-js/jwt-verify`](rules/sec-js/jwt-verify.md) | A JWT is verified with a named algorithm, never `none` | 347 | A07 | manual | recommended |
| [`sec-js/no-secret-in-log`](rules/sec-js/no-secret-in-log.md) | No password, token or secret written to a log | 532 | A09 | manual | strict |
| [`sec-js/no-llm-output-to-sink`](rules/sec-js/no-llm-output-to-sink.md) | No model output run, written as HTML or used as SQL | 94, 79 | A05 | manual | recommended |
| [`sec-js/no-prompt-injection`](rules/sec-js/no-prompt-injection.md) | No request data in a model's system prompt | 1427 | A05 | manual | recommended |

## How the rules know a value comes from a request

Several rules ask whether a sink's argument comes from a request (`sec-js/no-path-traversal`, `no-ssrf`, `no-open-redirect`, `no-ssti`,
`no-header-injection`, `no-log-injection`, `no-format-string`, `no-unvalidated-dynamic-method`, `no-prototype-pollution`, `no-llm-output-to-sink`,
`no-prompt-injection`). They use the `from_source:` predicate ([SR-RULE-FORMAT](../../records/0112_rule-format.md) decision 3a). It holds when the
argument **is** a listed request source (or a method called on one), is built from one by concatenation, interpolation or `?:`, or is a local name
bound to one **in the same function**, one or more assignments or destructurings away. The sources are in this pack's `data/sources/`:

| table | sources |
|---|---|
| `sources/express` | `req.body`, `req.query`, `req.params`, `req.headers`, `req.cookies`, `req.url`, … and the same on `request` (Fastify, Hapi) |
| `sources/koa` | `ctx.request.body`, `ctx.query`, `ctx.params`, `ctx.headers`, `ctx.cookies.get`, … |
| `sources/nextjs` | `searchParams`, `request.nextUrl.searchParams`, `request.json`, `request.formData`, … |
| `sources/browser` | `location.hash`, `location.search`, `document.referrer`, `window.name`, `document.cookie`, `event.data`, … |
| `sources/llm` | `completion.choices`, `response.content`, `message.content`, `result.text`, … (a model SDK's answer) |

**`from_source:` is not taint analysis.** It never follows a call (a value passed through any function, a sanitizer or not, stops being a
source), a return value, a parameter, a closure's outer variable or another file. A rule page that uses it says so. Rules that need to see more
than one argument position (`path.join(a, b)`) look for a request path in the whole call instead.
Every other rule fires on a dangerous sink with a value that is not a literal, or on a setting that is unsafe in itself.

## What the rules are not

- **Not a scanner for the whole program.** They read one file at a time and one function at a time; a value that crosses a function boundary is not seen.
- **Not a proof of safety.** A clean run means no known-bad shape was found, not that the code is safe. `strict` rules are heuristics and warn.
- **Not for generated code or tests without care.** Turn rules off for a folder with an `[[overrides]]` entry in `codelaw.toml`; `sec-js/no-hardcoded-secret` and
  `sec-js/no-insecure-random` fire in test helpers too.
- **Deviations from the first catalogue (W-54).** `sec-js/no-cleartext-http` is `manual`: an edit primitive cannot rewrite the text inside a URL. `sec-js/no-bidi-characters`
  is `suggest`, not `auto`: only a comment is removed whole, and a string or regular expression with a hidden character is left for you.
  `sec-js/no-prototype-pollution` does not flag a literal `.__proto__` write, only a request value used as a key or merged; `sec-js/no-javascript-url` reads the
  address by its text (`location.hash`, `document.referrer`, …), not through a local name.

## Not ported, and why

[eslint-plugin-security](https://github.com/eslint-community/eslint-plugin-security) (Apache-2.0) is the source of the rules whose `ported_from:` names it; this
table covers each of its rules. Bandit (Apache-2.0) and Ruff's `S` rules (MIT) are named in `ported_from:` the same way.

| eslint-plugin-security rule | here | why |
|---|---|---|
| `detect-buffer-noassert` | `sec-js/no-new-buffer` | the `noAssert` argument is flagged there |
| `detect-child-process` | `sec-js/no-child-process-shell` | narrowed: a literal command is fine; `shell: true` is flagged |
| `detect-disable-mustache-escape` | not ported | the `escapeMarkup = false` switch is of one old template engine; the XSS it allows is flagged where the output is written (`no-unsanitized-html`, `no-ssti`) |
| `detect-eval-with-expression` | `sec-js/no-eval` | widened to `new Function` and string timers |
| `detect-new-buffer` | `sec-js/no-new-buffer` | with a suggested fix |
| `detect-no-csrf-before-method-override` | `sec-js/no-csrf-before-method-override` | with a suggested reorder |
| `detect-non-literal-fs-filename` | `sec-js/no-path-traversal` (a request value), `sec-js/no-non-literal-fs-filename` (`strict`) | split: the first is precise, the second is the original's breadth |
| `detect-non-literal-regexp` | `sec-js/no-unsafe-regex` | only a pattern built from a request is flagged |
| `detect-non-literal-require` | `sec-js/no-non-literal-require` | |
| `detect-object-injection` | `sec-js/no-unvalidated-dynamic-method`, `sec-js/no-prototype-pollution` | not ported as written: it flags every `obj[key]`, which is most code; only a key a request supplies is flagged |
| `detect-possible-timing-attacks` | `sec-js/timing-safe-compare` (`strict`) | limited to names that hold a secret, with a suggested fix |
| `detect-pseudoRandomBytes` | `sec-js/no-insecure-random` | folded in |
| `detect-unsafe-regex` | `sec-js/no-unsafe-regex` | nested quantifiers in a regular expression literal |
| `detect-bidi-characters` | `sec-js/no-bidi-characters` | |

Read for ideas only, never copied (L2): CodeQL's JavaScript queries (MIT), the OWASP cheat sheets, and — under licences that forbid copying —
eslint-plugin-no-unsanitized (MPL-2.0), njsscan (LGPL-3.0) and Semgrep's rules (the Semgrep Rules License).
