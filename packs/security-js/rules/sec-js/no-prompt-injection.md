---
id: sec-js/no-prompt-injection
title: "No request data in a model's system prompt"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "how the prompt is split between instructions and data is a design decision"
stability: preview
security: { cwe: [1427], owasp: ["A05:2025"], severity: high }
since: 0.3.0
---

Request data is not written into the system prompt, the instructions or the system message of a model call.

**Why:** The system prompt is the model's rulebook; a client who can edit it can rewrite the rules, so keep their text in a user message where the model treats it as data (CWE-1427).

```codelaw
match:
  any:
    - pattern:
        context: '({ content: $V })'
        selector: pair
      inside:
        kind: object
        has:
          kind: pair
          regex: '^["'']?role["'']?\s*:\s*["'']system["'']$'
    - pattern:
        context: '({ system: $V })'
        selector: pair
    - pattern:
        context: '({ systemPrompt: $V })'
        selector: pair
    - pattern:
        context: '({ instructions: $V })'
        selector: pair
    - pattern:
        context: '({ system_instruction: $V })'
        selector: pair
where:
  $V: { from_source: { tables: [sources/express, sources/koa, sources/nextjs] } }
message: "`$V` comes from the request and is written into the system prompt"
```

## How to fix

1. Keep the system prompt a constant. Put the user's text in a `user` message, where the model treats it as data.
2. Pass facts the prompt needs (a name, a plan) through a validated, length-limited field, and say in the prompt that they are data.
3. Check the model's output before acting on it: see `sec-js/no-llm-output-to-sink`.

## Examples

### JavaScript

#### Don't

```js
const messages = [{ role: "system", content: `You help ${req.query.name} with billing.` }];
```

#### Do

```js
const messages = [
  { role: "system", content: "You help with billing." },
  { role: "user", content: req.query.name },
];
```

### JSX

#### Don't

```jsx
const messages = [{ role: "system", content: `You help ${req.query.name} with billing.` }];
```

#### Do

```jsx
const messages = [
  { role: "system", content: "You help with billing." },
  { role: "user", content: req.query.name },
];
```

### TypeScript

#### Don't

```ts
const messages = [{ role: "system", content: `You help ${req.query.name} with billing.` }];
```

#### Do

```ts
const messages = [
  { role: "system", content: "You help with billing." },
  { role: "user", content: req.query.name },
];
```

### TSX

#### Don't

```tsx
const messages = [{ role: "system", content: `You help ${req.query.name} with billing.` }];
```

#### Do

```tsx
const messages = [
  { role: "system", content: "You help with billing." },
  { role: "user", content: req.query.name },
];
```
