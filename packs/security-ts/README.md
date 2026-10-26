# `@codelaw/rules-security-ts`

codelaw's TypeScript-only security rules: 16 of them for Angular, NestJS, Next.js server actions, tRPC, Prisma, TypeORM, Drizzle, Kysely, MikroORM and the type assertions that
hide unchecked input. One of codelaw's first-party rule packs ([SR-RULE-PACKS](../../records/0101_rule-packs.md)). It ships on npm only, as
`@codelaw/rules-security-ts` (L6 decision 11). The `sec-js/*` rules already run on TypeScript, so this pack is thin on purpose: it holds only what a
TypeScript code base adds. Its presets **extend** `security-js`'s, so one preset gives a TypeScript file both sets, and it needs
`@codelaw/rules-security-js` installed beside it.

```toml
# codelaw.toml
extends = ["codelaw:js/recommended", "codelaw:ts/recommended", "codelaw:sec-ts/recommended"]
preview = true   # every rule is `preview` until a corpus run promotes it
```

`codelaw init` does not add it: a security pack is chosen, not detected.

## Presets

<!-- docs-sync:presets -->

| preset | rules | ported from |
|---|---|---|
| `sec-ts/recommended` | 65 | extends `sec-js/recommended`; high-confidence TypeScript-only rules, at `error` |
| `sec-ts/strict` | 82 | extends `sec-js/strict` and `sec-ts/recommended`; the heuristic rules, at `warn` |

<!-- /docs-sync:presets -->

`recommended` holds the 10 high-confidence rules, at `error`; `strict` adds the 6 heuristic ones, at `warn`, so a team can start without noise.
3 rules offer a suggested fix; the other 13 are `manual`, and each names the safe API in its `How to fix` steps, which the `codelaw-fix` skill walks through.

## Rules

Every rule carries `security: { cwe, owasp, severity }` ([SR-RULE-FORMAT](../../records/0112_rule-format.md) decision 2a), so

- its finding names the CWE (`sec-ts/<rule> (CWE-79) …`);
- `codelaw rules --tag cwe:79` and `--tag owasp:A05` find it, and `codelaw rules --security-coverage` puts it in the OWASP Top 10:2025 matrix;
- SARIF carries its `security` and `external/cwe/cwe-<n>` tags and a `security-severity` that GitHub code scanning ranks by
  ([SR-CLI-MCP](../../records/0115_cli-and-mcp.md) decision 4).

### Injection, queries and templates

| rule | rule says | CWE | OWASP 2025 | fix | preset |
|---|---|---|---|---|---|
| [`sec-ts/no-prisma-raw-unsafe`](rules/sec-ts/no-prisma-raw-unsafe.md) | No $queryRawUnsafe or $executeRawUnsafe with a value that is not a literal | 89 | A05 | suggest | recommended |
| [`sec-ts/no-typeorm-string-query`](rules/sec-ts/no-typeorm-string-query.md) | No TypeORM query built from a template or concatenation | 89 | A05 | manual | recommended |
| [`sec-ts/no-query-builder-raw`](rules/sec-ts/no-query-builder-raw.md) | No raw SQL fragment from a value in Drizzle, Kysely or MikroORM | 89 | A05 | manual | recommended |
| [`sec-ts/angular-bypass-security-trust`](rules/sec-ts/angular-bypass-security-trust.md) | No bypassSecurityTrust call on a value that is not a literal | 79 | A05 | manual | recommended |
| [`sec-ts/angular-native-element-html`](rules/sec-ts/angular-native-element-html.md) | No HTML written to a native element or through Renderer2 | 79 | A05 | manual | recommended |

### Input that nothing has checked

| rule | rule says | CWE | OWASP 2025 | fix | preset |
|---|---|---|---|---|---|
| [`sec-ts/no-unchecked-cast-of-input`](rules/sec-ts/no-unchecked-cast-of-input.md) | No type assertion on input that nothing has checked | 20 | A08 | manual | strict |
| [`sec-ts/no-ts-ignore-on-sink`](rules/sec-ts/no-ts-ignore-on-sink.md) | No type error silenced on a line that holds a security sink | 20 | A06 | manual | strict |
| [`sec-ts/no-any-request-handler`](rules/sec-ts/no-any-request-handler.md) | No request typed any | 20 | A06 | manual | strict |
| [`sec-ts/no-permissive-schema`](rules/sec-ts/no-permissive-schema.md) | No request schema that lets anything through | 915, 20 | A08 | suggest | recommended |
| [`sec-ts/next-server-action-validation`](rules/sec-ts/next-server-action-validation.md) | A Next.js server action validates its input | 20, 862 | A01 | manual | recommended |

### Access, configuration and rate limits

| rule | rule says | CWE | OWASP 2025 | fix | preset |
|---|---|---|---|---|---|
| [`sec-ts/nest-validation-whitelist`](rules/sec-ts/nest-validation-whitelist.md) | ValidationPipe has whitelist on | 915 | A01 | suggest | recommended |
| [`sec-ts/nest-cors`](rules/sec-ts/nest-cors.md) | NestJS CORS names its origins | 942 | A02 | manual | recommended |
| [`sec-ts/nest-missing-guard`](rules/sec-ts/nest-missing-guard.md) | A NestJS controller has a guard | 862 | A01 | manual | strict |
| [`sec-ts/nest-throttler`](rules/sec-ts/nest-throttler.md) | A NestJS sign-in controller is rate limited | 770, 307 | A07 | manual | strict |
| [`sec-ts/typeorm-synchronize`](rules/sec-ts/typeorm-synchronize.md) | TypeORM does not synchronize the schema on start | 1188 | A02 | manual | recommended |
| [`sec-ts/trpc-public-mutation`](rules/sec-ts/trpc-public-mutation.md) | A tRPC mutation is not public | 862 | A01 | manual | strict |

## Syntax, not types

codelaw reads one file's syntax; it has no type checker. So 'typed `any`' means the word `any` is written there (`sec-ts/no-any-request-handler`), and 'a validator in
the function' means a call to `parse`, `safeParse`, `validate` or a schema's method appears in the same function (`sec-ts/no-unchecked-cast-of-input`,
`sec-ts/next-server-action-validation`). An `any` that is inferred, a validator that runs in another function or another file, and a guard registered in a module are not
seen; the rules that depend on them are `strict`, and each page says what it cannot see. What type information would add (the real type of `req.body`, whether a
value flows from a request through a call) waits on [B-049](../../work/BACKLOG.md), the type-aware rules.

None of these rules uses `from_source:`: they match a call, a decorator or a setting, so the pack needs no request-source tables
(`security-js` has them in `data/sources/`).

## Fixes

- **`suggest` (3):** `sec-ts/nest-validation-whitelist` (adds `whitelist: true`), `sec-ts/no-prisma-raw-unsafe` (`$queryRawUnsafe` with one template to the tagged `$queryRaw`),
  `sec-ts/no-permissive-schema` (`.passthrough()` to `.strict()`). A shape the rule cannot rewrite safely (a built string, `z.any()`, options that need merging) is reported as `manual` with the steps.
- **`manual`:** the rest, each with the safe API in its steps.

## Not ported, and why

Nothing here is ported from another tool's rules: eslint-plugin-security has no TypeScript-specific rules, and typescript-eslint's security-relevant rules (`no-explicit-any`, `no-unsafe-*`) need type
information. The catalogue was written from the Angular security guide, NestJS's validation and security docs, Next.js's data-security guide, Prisma's raw-query docs and the OWASP cheat sheets, in codelaw's own words;
CodeQL's JavaScript queries (MIT) were read for ideas and nothing is copied. Rule sets under a copyleft or source-available licence (eslint-plugin-no-unsanitized, njsscan, Semgrep's rules) were not used (L2).
