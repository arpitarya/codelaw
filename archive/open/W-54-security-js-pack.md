---
type: Handoff
name: W-54
description: "Arpit, 2026-10-25: the security-js pack: injection, XSS, path traversal, SSRF, crypto, secrets, cookies, TLS, ReDoS and prototype-pollution rules for JS/JSX (and TS files), mapped to CWE and OWASP 2025."
item: W-54
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-26
updated_by: "Claude Code (Sonnet 5.5), lockmaster"
ball: done
---

# W-54 — `security-js` — JavaScript (Node, browser, Express) security rules

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Port the eslint-plugin-security set (Apache-2.0) and write the rest below, each with cases | the catalogue | ✅ 2026-10-26 — 66 rules in `packs/security-js/rules/sec-js/`, each with Do/Don't in four languages and a `.cases.md` |
| 2 | Framework sources for `from_source:`: Express, Koa, Fastify, Next.js route handlers, browser `location`/`postMessage` | precise rules | ✅ 2026-10-26 — `data/sources/{express,koa,nextjs,browser,llm}.json` (Fastify's `request.*` is in `express`) |
| 3 | `recommended` and `strict` presets; README with the 'not ported, and why' table | adoption | ✅ 2026-10-26 — 55 `recommended`, 11 `strict` |
| 4 | Corpus run on three real repos; every `recommended` rule under 5% false positives on them | trustworthy defaults | ✅ 2026-10-26 — 744 files, 31 findings; the false positives it showed (client-side `confirm password` compares, an `api.post` client call, a cookie-name constant) were fixed and became cases |

**Arpit, 2026-10-25:** *"I want to create a pack for security issues. Security issues like Veracode, Black Duck, X-Ray, OWASP. They would have some suggestions on how to write, what to write, what not to write in JavaScript and Python. So create … one for JavaScript, one for TypeScript, one for HTML, one for Python. Research and create work items for individual languages."*

**Model:** Sonnet — a written catalogue with CWE, OWASP and fix class per rule, each proven by cases.

## Goal

| rule | flags | CWE | OWASP 2025 | fix |
|---|---|---|---|---|
| `sec-js/no-eval` | `eval`, `new Function`, `setTimeout`/`setInterval` with a string | 94 | A05 | manual |
| `sec-js/no-child-process-shell` | `exec`/`execSync` with a non-literal; `spawn(…, {shell: true})` | 78 | A05 | manual: `execFile` + args array |
| `sec-js/no-sql-template` | `.query(`…${x}`)`, `.raw(` concatenation (pg, mysql2, knex, sequelize) | 89 | A05 | manual: placeholders |
| `sec-js/no-nosql-injection` | `$where`; a request object passed straight to `find`/`findOne` | 943 | A05 | manual |
| `sec-js/no-unsanitized-html` | `innerHTML`, `outerHTML`, `insertAdjacentHTML`, `document.write` with a non-literal | 79 | A05 | suggest: `textContent` |
| `sec-js/no-dangerously-set-inner-html` | JSX `dangerouslySetInnerHTML` not wrapped in a known sanitizer (DOMPurify) | 79 | A05 | manual |
| `sec-js/no-javascript-url` | assigning `location`/`href` a non-literal or a `javascript:` literal | 79 | A05 | manual |
| `sec-js/postmessage-origin` | `postMessage(…, "*")`; a `message` listener that never reads `event.origin` | 346 | A01 | manual |
| `sec-js/no-path-traversal` | `fs.*`, `path.join`, `res.sendFile` with a request source | 22 | A01 | manual: resolve + prefix check |
| `sec-js/no-archive-slip` | archive entry names written to disk unchecked | 22 | A01 | manual |
| `sec-js/no-open-redirect` | `res.redirect(` with a request source | 601 | A01 | manual: allow-list |
| `sec-js/no-ssrf` | `fetch`/`axios`/`http.get` URL from a request source | 918 | A01 | manual |
| `sec-js/no-tls-bypass` | `rejectUnauthorized: false`; `NODE_TLS_REJECT_UNAUTHORIZED = "0"` | 295 | A04 | suggest: remove |
| `sec-js/no-cors-wildcard-credentials` | `cors({ origin: "*" | true, credentials: true })` | 942 | A02 | manual |
| `sec-js/express-hardening` | an Express app with no `helmet()` and `x-powered-by` left on | 200 | A02 | suggest |
| `sec-js/cookie-flags` | `res.cookie`/session config without `httpOnly`, `secure`, `sameSite` | 614, 1004 | A02 | suggest: add flags |
| `sec-js/no-csrf-before-method-override` | `methodOverride` registered before `csrf` | 352 | A01 | suggest: reorder |
| `sec-js/no-weak-hash` | `createHash("md5"|"sha1")` | 327 | A04 | manual |
| `sec-js/no-weak-cipher` | `createCipher` (no IV), DES/RC4/ECB modes | 327 | A04 | manual |
| `sec-js/no-insecure-random` | `Math.random()` bound to a name like token/secret/password/nonce/id | 338 | A04 | suggest: `crypto.randomUUID()` / `getRandomValues` |
| `sec-js/no-hardcoded-secret` | literal AWS/GCP/Stripe/GitHub keys, private-key headers, `password = "…"` | 798 | A07 | manual: env/secret store |
| `sec-js/timing-safe-compare` | `===` on a token/hash/signature | 208 | A07 | suggest: `crypto.timingSafeEqual` |
| `sec-js/jwt-verify` | `jwt.verify` without `algorithms`; `"none"`; `jwt.decode` used for auth | 347 | A07 | manual |
| `sec-js/no-token-in-web-storage` | `localStorage`/`sessionStorage` set with a token | 922 | A07 | manual: httpOnly cookie |
| `sec-js/no-unsafe-regex` | nested quantifiers (ReDoS); `new RegExp(` from a request source | 1333 | A06 | manual |
| `sec-js/no-prototype-pollution` | `obj[key] =` with a request-sourced key; `__proto__`/`constructor` keys; deep-merge of `req.body` | 1321 | A08 | manual |
| `sec-js/no-unsafe-deserialize` | `node-serialize` `unserialize`; js-yaml < 4 `load` | 502 | A08 | suggest: safe loader |
| `sec-js/no-stack-trace-response` | `res.send(err)`/`err.stack` in a response | 209 | A10 | manual |
| `sec-js/no-secret-in-log` | `console.*`/logger with a password/token/secret binding | 532 | A09 | manual |
| `sec-js/no-new-buffer` | `new Buffer(`, `noAssert` | 119 | A06 | suggest: `Buffer.from`/`alloc` |
| `sec-js/no-bidi-characters` | Trojan Source bidi control characters in code | — | A08 | auto: remove |
| `sec-js/no-llm-output-to-sink` | a model-SDK response (OpenAI, Anthropic, Vercel AI) reaching `eval`, `exec`, `innerHTML`, SQL | 94, 79 | A05 | manual |

### Added after the second research pass (2026-10-25) — 34 more

Sources: CodeQL's query list (read for ideas; MIT, nothing copied), Ruff's `S` rules and Bandit (Apache-2.0/MIT, portable with `ported_from:`), the OWASP cheat sheets. `preset` says where each rule starts: `recommended` (`error`) or `strict` (`warn`, heuristic).

| rule | flags | CWE | OWASP 2025 | fix | preset |
|---|---|---|---|---|---|
| `sec-js/no-electron-insecure` | `webSecurity: false`, `allowRunningInsecureContent: true`, `nodeIntegration: true`, `contextIsolation: false`, `sandbox: false` | 79, 829 | A02 | suggest: flip the flag | recommended |
| `sec-js/no-xxe` | libxmljs `parseXml(…, { noent: true })` and similar entity expansion | 611, 776 | A05 | suggest: `noent: false` | recommended |
| `sec-js/no-xpath-injection` | `xpath.select`/`evaluate` with a template or concatenation | 643 | A05 | manual | recommended |
| `sec-js/no-ldap-injection` | ldapjs search filter built from a template | 90 | A05 | manual: escape filter | recommended |
| `sec-js/no-ssti` | `ejs.render`, `pug.compile`, `Handlebars.compile`, `nunjucks.renderString` of a request source; `res.render(view, req.query)` (template object injection) | 1336, 73 | A05 | manual | recommended |
| `sec-js/no-header-injection` | `res.setHeader`/`writeHead` value from a request source (CRLF) | 113 | A05 | manual | recommended |
| `sec-js/no-log-injection` | logging a request source without stripping newlines | 117 | A09 | manual | strict |
| `sec-js/no-format-string` | `util.format`/`console.*` first argument from a request source | 134 | A05 | suggest: `%s` | strict |
| `sec-js/no-non-literal-require` | `require(x)` / `import(x)` with a non-literal | 829, 95 | A08 | manual | recommended |
| `sec-js/no-non-literal-fs-filename` | any `fs.*` path that is not a literal | 22 | A01 | manual | strict |
| `sec-js/no-weak-key-size` | `generateKeyPair("rsa", { modulusLength: < 2048 })`, weak EC curves | 326 | A04 | suggest: 2048/P-256 | recommended |
| `sec-js/password-hash-effort` | `createHash` on a password binding; `pbkdf2` < 600 000 iterations; bcrypt cost < 10 | 916 | A04 | manual: argon2id/bcrypt | recommended |
| `sec-js/no-biased-random` | `crypto.randomBytes(n)[i] % k` | 327 | A04 | suggest: `crypto.randomInt` | recommended |
| `sec-js/no-hardcoded-crypto-key` | `createCipheriv`/`createHmac`/`jwt.sign` with a literal key or IV; a reused static IV | 321, 329 | A04 | manual | recommended |
| `sec-js/no-cleartext-http` | `fetch`/`axios`/`http.get` to `http://` other than localhost | 319 | A04 | suggest: `https://` | recommended |
| `sec-js/express-session-secret` | `express-session` with a literal `secret` or `cookie.secure: false` | 798, 614 | A07 | manual | recommended |
| `sec-js/helmet-config` | `helmet({ contentSecurityPolicy: false, frameguard: false, hsts: false })` | 693 | A02 | manual | recommended |
| `sec-js/missing-rate-limit` | login/reset/OTP routes with no rate-limit middleware | 770, 307 | A07 | manual | strict |
| `sec-js/no-sensitive-get-query` | reading `req.query.password|token|secret` | 598 | A04 | manual: POST body | recommended |
| `sec-js/no-unvalidated-dynamic-method` | `obj[req.query.fn]()` | 470 | A05 | manual: allow-list | recommended |
| `sec-js/no-incomplete-url-check` | `url.includes("example.com")`, `startsWith("https://example.com")` with no `/`, unanchored hostname regex | 20 | A01 | manual: parse with `URL` | recommended |
| `sec-js/no-incomplete-sanitization` | `.replace("<", …)` with a string (first match only); one-pass multi-char stripping | 116 | A05 | suggest: `replaceAll`/regex `g` | recommended |
| `sec-js/no-bad-tag-filter` | regexes like `/<script>/` used to strip HTML | 116, 185 | A05 | manual: a sanitizer | recommended |
| `sec-js/no-jquery-html-sink` | `$(x)`, `.html(x)`, `.append(x)`, `.after(x)` with a non-literal | 79 | A05 | manual: `.text()` | recommended |
| `sec-js/no-window-open-opener` | `window.open(url)` without `noopener` | 1022 | A05 | suggest: add `noopener` | recommended |
| `sec-js/no-dynamic-script-src` | `script.src =`, `iframe.src =` from a non-literal | 829 | A08 | manual | recommended |
| `sec-js/graphql-hardening` | Apollo/Yoga with `introspection: true` in production or no depth/cost limit | 200, 770 | A02 | manual | strict |
| `sec-js/no-mass-assignment` | `Model.create(req.body)`, `new User(req.body)`, `Object.assign(entity, req.body)` | 915 | A01 | manual: pick fields | recommended |
| `sec-js/no-user-controlled-auth-bypass` | `if (req.cookies.isAdmin)` / `req.query.debug` gating a privileged branch | 807 | A01 | manual | strict |
| `sec-js/no-server-env-in-client` | `process.env.X` (not `NEXT_PUBLIC_`) in a `"use client"` file; `NEXT_PUBLIC_*SECRET*`/`*KEY*` names | 200 | A02 | manual | recommended |
| `sec-js/no-document-cookie-token` | `document.cookie = "token=…"` | 1004 | A07 | manual: server-set httpOnly | recommended |
| `sec-js/no-prompt-injection` | a request source concatenated into an LLM system prompt | 1427 | A05 | manual: separate roles | recommended |
| `sec-js/upload-limits` | `multer()` without `limits` and `fileFilter` | 434, 770 | A06 | manual | recommended |
| `sec-js/angularjs-sce-disabled` | `$sceProvider.enabled(false)`, permissive `aHrefSanitizationWhitelist` | 79 | A05 | manual | strict |

Also widen `sec-js/no-unsanitized-html`'s sinks: `Range.createContextualFragment`, `iframe.srcdoc`, `setHTMLUnsafe`, `DOMParser.parseFromString` inserted into the DOM; fold `crypto.pseudoRandomBytes` into `sec-js/no-insecure-random`.

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | every rule has Do/Don't examples and cases, `security:` metadata and a fix | ✅ 2026-10-26 |
| 2 · | `codelaw test packs/security-js` green | ✅ 2026-10-26 — 66 rules · 1482 checks · 0 failed |
| 3 · | `recommended` false-positive rate under 5% on the corpus | ✅ 2026-10-26 — see step 4 |
| 4 · | README's not-ported table covers every eslint-plugin-security rule | ✅ 2026-10-26 — all 14 |

## Blockers

none: W-53 landed 2026-10-25.

## Record affected

SR-RULE-PACKS (the pack's row and presets); rule pages are the rules' spec (SR-RULE-FORMAT).

## Hazards

- **No copyleft copying (L2):** Bandit, eslint-plugin-security and Ruff's `S` rules (Apache-2.0 / MIT) may be ported with `ported_from:` and NOTICE; eslint-plugin-no-unsanitized (MPL-2.0), njsscan (LGPL) and Semgrep's rules (Semgrep Rules License) are **reference only** — read, never copied.
- **Pattern-level, not taint analysis:** every rule fires on a dangerous sink with a non-literal argument, or with a known request source in the same function (W-53's predicate). False positives go to `security-*/strict` at `warn`; `recommended` holds only high-confidence rules at `error`.
- Every rule carries `security:` metadata (CWE, OWASP, severity) from W-53; L4: every `manual` fix names the safe API in its steps.
- Overlap: `react/no-danger` style rules belong here, not in `react`; check the react pack before adding.

## Reference

[CodeQL JS queries](https://codeql.github.com/codeql-query-help/javascript/) · [OWASP Node.js cheat sheet](https://cheatsheetseries.owasp.org/cheatsheets/Nodejs_Security_Cheat_Sheet.html) · [OWASP Password Storage](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html) · [Electron security checklist](https://www.electronjs.org/docs/latest/tutorial/security) · [eslint-plugin-security](https://github.com/eslint-community/eslint-plugin-security) · [OWASP Node.js cheat sheet](https://cheatsheetseries.owasp.org/cheatsheets/Nodejs_Security_Cheat_Sheet.html) · [JFrog SAST rule list](https://docs.jfrog.com/security/docs/list-of-sast-rules)

## What landed, and where it differs from the catalogue (2026-10-26)

- 66 rules, 55 in `sec-js/recommended`, 11 more in `sec-js/strict`. Fix classes: 17 `suggest`, 49 `manual`, none `auto`.
- **Differences, each said on the pack README:** `sec-js/no-bidi-characters` is `suggest`, not `auto` (a comment is removed whole; a hidden character in a string or a regular expression has no mechanical fix — a new edit primitive or a native rule would make it `auto`); `sec-js/no-cleartext-http` is `manual`, not `suggest` (an edit primitive cannot rewrite the text inside a URL); `sec-js/no-prototype-pollution` does not flag a literal `__proto__` write, only a request value used as a key or merged; `sec-js/no-javascript-url` reads the address by its text, not through a local name; `sec-js/no-electron-insecure` fixes the flags set to `false` only; `sec-js/no-unsafe-deserialize` flags a bare js-yaml `load`, safe in js-yaml 4, with a harmless fix.
- **Engine behaviour learned, for the next pack:** `from_source:` is one `where:` for the whole rule, so every branch of the rule must capture the same name, and an `any:` runs the first branch that matches the shape, so two branches of one shape with different `constraints:` do not both run (use `all:` with `regex:` per branch); `has:` looks at direct children unless `stopBy: end`; `$$$REST` after a comma does not match a call with one argument.
- **Not done here:** a release-build timing of the pack (the debug binary took 15 s over the 744 files; the `perf` job decides L8); promotion of any rule to `stable` (needs a corpus run with no false positives the decider has read).
