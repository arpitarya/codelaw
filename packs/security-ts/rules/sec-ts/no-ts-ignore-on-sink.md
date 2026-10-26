---
id: sec-ts/no-ts-ignore-on-sink
title: "No type error silenced on a line that holds a security sink"
languages: [typescript, tsx]
fix: manual
fix_reason: "what the right type is depends on the value"
stability: preview
security: { cwe: [20], owasp: ["A06:2025"], severity: medium }
since: 0.3.0
---

A `@ts-ignore`, a `@ts-expect-error` or an `as any` is not placed on a line that calls a dangerous API (`eval`, `exec`, `innerHTML`, a raw query, a redirect …).

**Why:** The type error is often the compiler saying the value is not what the sink needs; silencing it on the sink line removes the one check that was running (CWE-20).

```codelaw
match:
  any:
    - all:
        - kind: comment
        - regex: '@ts-(ignore|expect-error)'
        - precedes:
            regex: '\b(eval|execSync|spawnSync|innerHTML|outerHTML|dangerouslySetInnerHTML|insertAdjacentHTML|document\.write|bypassSecurityTrust\w+|createContextualFragment|new Function)\b|\$(query|execute)RawUnsafe'
    - all:
        - kind: as_expression
        - regex: '\bas any$'
        - inside:
            stopBy: end
            any:
              - kind: expression_statement
              - kind: lexical_declaration
              - kind: return_statement
            regex: '\b(eval|execSync|spawnSync|innerHTML|outerHTML|dangerouslySetInnerHTML|insertAdjacentHTML|document\.write|bypassSecurityTrust\w+|createContextualFragment|new Function)\b|\$(query|execute)RawUnsafe'
message: "A type error is silenced next to a security-sensitive call"
```

## How to fix

1. Fix the type instead: narrow the value, or convert it with a function that returns the type the sink wants.
2. If the sink really needs a value of another type, write a small typed helper for it and keep the cast inside the helper, with a comment that says why it is safe.

## Examples

### TypeScript

#### Don't

```ts
// @ts-ignore
element.innerHTML = html;
```

#### Do

```ts
element.textContent = html;
```

### TSX

#### Don't

```tsx
// @ts-ignore
element.innerHTML = html;
```

#### Do

```tsx
element.textContent = html;
```
