---
type: Proposal
title: Type-aware rules through typescript-go
description: "Rules that need TypeScript's types (no-floating-promises, no-unsafe-*, switch-exhaustiveness …) and TypeScript's own errors with its fixes, answered by a typescript-go helper; parked by the decider 2026-10-25."
status: graduated
timestamp: 2026-10-25T16:10:00Z
---

# Type-aware rules through typescript-go

**Arpit, 2026-10-25:** *"We need to build out type aware rules for sure … For type-aware rules, TypeScript Go … For now, put type aware rules and details in backlog. We will talk about it later."*

## The idea, as discussed

- **Helper:** `codelaw-types`, a Go binary built on [typescript-go](https://github.com/microsoft/typescript-go) (Apache-2.0), shipped per platform as an **optional** package (`@codelaw/types-<platform>`, and on PyPI), installed only when type rules are on. Precedent: oxlint's `tsgolint`.
- **Protocol:** codelaw starts the helper with the project's `tsconfig`; the helper loads the program once and answers a **fixed vocabulary of type questions** (promise-like, `any`, nullable, union members, `@deprecated`, never …). Rules stay declarative — a `type:` clause in a rule file — and codelaw's fix engine builds the fixes.
- **TypeScript's own errors** as `ts/type-error` findings, with TypeScript's code fixes as `suggest`.
- **Performance:** CI and `check` load the program once; daemon, LSP and watch (W-44) keep it warm; the hook uses a warm program or skips type rules with a note, so L8's hook budget holds.
- **Determinism:** codelaw pins the typescript-go version; the report names it.
- **The project's own TypeScript:** TypeScript 7 (GA 2026-07-08) ships **no programmatic API**; one is expected in 7.1. Once it lands, `[types] checker = "project"` can drive the repo's own TS 7 instead of the helper; the bundled helper stays for Python-only users and TS ≤ 6.
- **Rules to port first** (typescript-eslint `*-type-checked`, which `packs/ts/README.md` lists as out of scope): `no-floating-promises`, `no-misused-promises`, `await-thenable`, `require-await`, `return-await`, `switch-exhaustiveness-check`, `no-unnecessary-condition`, `no-unnecessary-type-assertion`, `prefer-nullish-coalescing`, `prefer-optional-chain`, `strict-boolean-expressions`, the five `no-unsafe-*`, `restrict-template-expressions`, `no-base-to-string`, `unbound-method`, `only-throw-error`, `prefer-promise-reject-errors`, `no-deprecated`, `consistent-type-exports`, `prefer-readonly`, `dot-notation`, `prefer-includes`.
- **Python twin later:** ty (Astral) or pyrefly (Meta), both Rust, MIT.

## What it needs

- **L3 amended** (decision 8 keeps type-aware checks outside the core) and **L6** (a per-platform helper package).
- SR-ARCHITECTURE's "not decided yet, by design: type-aware rules" becomes a record.

## Graduation trigger

The decider reopens it — or TypeScript 7.1 ships its programmatic API, whichever comes first.
