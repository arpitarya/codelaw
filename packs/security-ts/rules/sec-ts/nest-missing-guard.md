---
id: sec-ts/nest-missing-guard
title: "A NestJS controller has a guard"
languages: [typescript, tsx]
fix: manual
fix_reason: "which guard protects the route is a decision for the author"
stability: preview
security: { cwe: [862], owasp: ["A01:2025"], severity: high }
since: 0.3.0
---

A class decorated `@Controller` carries `@UseGuards`, or the file registers a global guard.

**Why:** A route with no guard answers anyone; one forgotten controller is an open door (CWE-862). This reads one file: a guard registered in another file (`APP_GUARD` in a module) is not seen, so it warns.

```codelaw
match:
  all:
    - any:
        - kind: export_statement
        - kind: class_declaration
          not:
            inside:
              kind: export_statement
    - regex: '@Controller\('
    - not:
        regex: '@UseGuards|@Public\(|@AllowAnonymous|APP_GUARD|useGlobalGuards'
    - not:
        inside:
          kind: program
          stopBy: end
          has:
            stopBy: end
            regex: 'APP_GUARD|useGlobalGuards'
message: "This controller has no `@UseGuards`; its routes answer anyone"
```

## How to fix

1. Add `@UseGuards(AuthGuard)` to the controller (or register `APP_GUARD` once for the app and mark public routes `@Public()`).
2. Add role checks (`@Roles("admin")` with a `RolesGuard`) where the route is not for every signed-in user.

## Examples

### TypeScript

#### Don't

```ts
@Controller("users")
export class UsersController {
  @Get()
  list() {
    return [];
  }
}
```

#### Do

```ts
@Controller("users")
@UseGuards(AuthGuard)
export class UsersController {
  @Get()
  list() {
    return [];
  }
}
```

### TSX

#### Don't

```tsx
@Controller("users")
export class UsersController {
  @Get()
  list() {
    return [];
  }
}
```

#### Do

```tsx
@Controller("users")
@UseGuards(AuthGuard)
export class UsersController {
  @Get()
  list() {
    return [];
  }
}
```
