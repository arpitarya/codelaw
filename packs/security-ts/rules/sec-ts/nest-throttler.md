---
id: sec-ts/nest-throttler
title: "A NestJS sign-in controller is rate limited"
languages: [typescript, tsx]
fix: manual
fix_reason: "the limits that fit the route are a decision for the author"
stability: preview
security: { cwe: [770, 307], owasp: ["A07:2025"], severity: medium }
since: 0.3.0
---

A controller with a login, register, reset or one-time-code route carries `@Throttle` or `ThrottlerGuard`.

**Why:** Without a limit an attacker tries passwords, codes and reset tokens as fast as the server answers (CWE-307, CWE-770).

```codelaw
match:
  all:
    - any:
        - kind: export_statement
        - kind: class_declaration
          not:
            inside:
              kind: export_statement
    - regex: '@Post\(\s*["''](login|signin|sign-in|register|signup|reset|forgot|otp|verify|refresh)|@Controller\(\s*["'']auth'
    - not:
        regex: '@Throttle|ThrottlerGuard|@SkipThrottle|ThrottlerModule'
message: "This sign-in controller has no `@Throttle`; guesses are not limited"
```

## How to fix

1. Install `@nestjs/throttler`, add `ThrottlerGuard` to the controller (or globally) and `@Throttle({ default: { limit: 5, ttl: 60000 } })` to the route.
2. Add a lock-out or a growing delay after repeated failures for one account, and log them.

## Examples

### TypeScript

#### Don't

```ts
@Controller("auth")
export class AuthController {
  @Post("login")
  login() {
    return signIn();
  }
}
```

#### Do

```ts
@Controller("auth")
@UseGuards(ThrottlerGuard)
export class AuthController {
  @Post("login")
  login() {
    return signIn();
  }
}
```

### TSX

#### Don't

```tsx
@Controller("auth")
export class AuthController {
  @Post("login")
  login() {
    return signIn();
  }
}
```

#### Do

```tsx
@Controller("auth")
@UseGuards(ThrottlerGuard)
export class AuthController {
  @Post("login")
  login() {
    return signIn();
  }
}
```
