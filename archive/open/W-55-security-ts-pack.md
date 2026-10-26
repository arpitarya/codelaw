---
type: Handoff
name: W-55
description: "Arpit, 2026-10-25: the security-ts pack: extends security-js and adds only what TypeScript code bases add — unchecked casts of untrusted input, Angular sanitizer bypasses, NestJS validation, Prisma/TypeORM raw queries."
item: W-55
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-26
updated_by: "Claude Code (Sonnet 5.5), lockmaster"
ball: done
---

# W-55 — `security-ts` — TypeScript-only security rules (Angular, NestJS, ORMs, unchecked casts)

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Write the catalogue below with cases | the rules | ✅ 2026-10-26 — 16 rules in `packs/security-ts/rules/sec-ts/`, each with Do/Don't in TypeScript and TSX and a `.cases.md` |
| 2 | `recommended` extends `security-js/recommended` | TS users install one preset | ✅ 2026-10-26 — 10 `recommended`, 6 `strict` |
| 3 | README: why the pack is thin (JS rules already run on TS) and what B-049 would add | honest scope | ✅ 2026-10-26 |

**Arpit, 2026-10-25:** *"I want to create a pack for security issues. Security issues like Veracode, Black Duck, X-Ray, OWASP. They would have some suggestions on how to write, what to write, what not to write in JavaScript and Python. So create … one for JavaScript, one for TypeScript, one for HTML, one for Python. Research and create work items for individual languages."*

**Model:** Sonnet — a short written catalogue.

## Goal

| rule | flags | CWE | OWASP 2025 | fix |
|---|---|---|---|---|
| `sec-ts/no-unchecked-cast-of-input` | `JSON.parse(x) as T`, `req.body as T`, `await res.json() as T` with no runtime validator (zod, valibot, class-validator) in the function | 20 | A08 | manual: validate |
| `sec-ts/no-ts-ignore-on-sink` | `@ts-ignore` / `@ts-expect-error` / `as any` on a line holding a security sink | 20 | A06 | manual |
| `sec-ts/angular-bypass-security-trust` | `DomSanitizer.bypassSecurityTrustHtml|Script|Style|Url|ResourceUrl` | 79 | A05 | manual |
| `sec-ts/nest-validation-whitelist` | `new ValidationPipe()` without `whitelist: true` (mass assignment) | 915 | A01 | suggest: add `whitelist` |
| `sec-ts/no-prisma-raw-unsafe` | `$queryRawUnsafe` / `$executeRawUnsafe` with a non-literal | 89 | A05 | suggest: tagged `$queryRaw` |
| `sec-ts/no-typeorm-string-query` | `.query(`…${x}`)`, `createQueryBuilder().where(`…${x}`)` | 89 | A05 | manual: parameters |
| `sec-ts/no-any-request-handler` | an HTTP handler whose request body is typed `any` | 20 | A06 | manual |

### Added after the second research pass (2026-10-25) — 9 more

Sources: CodeQL's query list (read for ideas; MIT, nothing copied), Ruff's `S` rules and Bandit (Apache-2.0/MIT, portable with `ported_from:`), the OWASP cheat sheets. `preset` says where each rule starts: `recommended` (`error`) or `strict` (`warn`, heuristic).

| rule | flags | CWE | OWASP 2025 | fix | preset |
|---|---|---|---|---|---|
| `sec-ts/angular-native-element-html` | `ElementRef.nativeElement.innerHTML =`; `Renderer2.setProperty(el, "innerHTML", x)` | 79 | A05 | manual: bind `[innerText]` | recommended |
| `sec-ts/nest-cors` | `app.enableCors()` with no options, or `{ origin: true, credentials: true }` | 942 | A02 | manual | recommended |
| `sec-ts/nest-missing-guard` | a controller whose routes carry no `@UseGuards` and no global guard is registered | 862 | A01 | manual | strict |
| `sec-ts/nest-throttler` | auth controllers with no `@Throttle`/`ThrottlerGuard` | 770, 307 | A07 | manual | strict |
| `sec-ts/next-server-action-validation` | an exported `"use server"` function that uses its arguments with no validator call | 20, 862 | A01 | manual: zod parse + auth check | recommended |
| `sec-ts/no-permissive-schema` | request schemas with `z.any()`, `.passthrough()`; class-validator `validate()` without `forbidUnknownValues` | 915, 20 | A08 | suggest: `.strict()` | recommended |
| `sec-ts/typeorm-synchronize` | `synchronize: true` in a DataSource config | 1188 | A02 | manual: migrations | recommended |
| `sec-ts/no-query-builder-raw` | Drizzle `sql.raw(x)`, Kysely `sql.raw`/`sql.lit`, MikroORM `execute` with a non-literal | 89 | A05 | manual: placeholders | recommended |
| `sec-ts/trpc-public-mutation` | `publicProcedure.mutation(…)` | 862 | A01 | manual: `protectedProcedure` | strict |

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | `codelaw test packs/security-ts` green | ✅ 2026-10-26 — 16 rules · 198 checks · 0 failed |
| 2 · | a TS fixture gets `sec-js/*` and `sec-ts/*` findings from one preset | ✅ 2026-10-26 — `crates/codelaw-cli/tests/security.rs` already installs both packs; checked by hand on a TS file, see the foot of the item |

## Blockers

none: W-53 and W-54 landed.

## Record affected

SR-RULE-PACKS (the pack's row; npm only).

## Hazards

- **No copyleft copying (L2):** Bandit, eslint-plugin-security and Ruff's `S` rules (Apache-2.0 / MIT) may be ported with `ported_from:` and NOTICE; eslint-plugin-no-unsanitized (MPL-2.0), njsscan (LGPL) and Semgrep's rules (Semgrep Rules License) are **reference only** — read, never copied.
- **Pattern-level, not taint analysis:** every rule fires on a dangerous sink with a non-literal argument, or with a known request source in the same function (W-53's predicate). False positives go to `security-*/strict` at `warn`; `recommended` holds only high-confidence rules at `error`.
- Every rule carries `security:` metadata (CWE, OWASP, severity) from W-53; L4: every `manual` fix names the safe API in its steps.
- Without types (B-049), 'typed `any`' and 'validator in the function' are syntactic — say so on each page.

## Reference

[CodeQL JS queries](https://codeql.github.com/codeql-query-help/javascript/) · [OWASP Mass Assignment cheat sheet](https://cheatsheetseries.owasp.org/cheatsheets/Mass_Assignment_Cheat_Sheet.html) · [Next.js data security](https://nextjs.org/docs/app/guides/data-security) · [Angular security guide](https://angular.dev/best-practices/security) · [NestJS validation](https://docs.nestjs.com/techniques/validation) · [Prisma raw queries](https://www.prisma.io/docs/orm/prisma-client/using-raw-sql/raw-queries)

## What landed (2026-10-26)

- 16 rules (10 in `sec-ts/recommended`, 6 more in `strict`; 3 `suggest`, 13 `manual`), written syntax-only, each page saying what it cannot see.
- A TypeScript file with `eval(code)` and `const body = req.body as CreateUserDto;` got `sec-js/no-eval` and `sec-ts/no-unchecked-cast-of-input` from `extends = ["codelaw:sec-ts/strict"]` with both packs installed (DoD 2).
- **Engine behaviour learned:** in tree-sitter-typescript the decorators of an exported class belong to the `export_statement`, not the `class_declaration`, so a rule about a decorated class matches the export (or an unexported class), not the class alone.
- **Not done here:** `crates/codelaw-cli/tests/security_ts.rs` was not written, because a new test binary would grow `target/` past the disk ceiling; the pack's own examples and cases (`codelaw test packs/security-ts`) and the corpus counts are the tests.
