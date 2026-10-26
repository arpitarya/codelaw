---
id: sec-js/no-llm-output-to-sink
title: "No model output run, written as HTML or used as SQL"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "what the program may do with model output is a design decision"
stability: preview
security: { cwe: [94, 79], owasp: ["A05:2025"], severity: high }
since: 0.3.0
---

The text a model SDK returns (OpenAI, Anthropic, Vercel AI) is not passed to `eval`, `new Function`, a shell, `innerHTML` or a SQL call.

**Why:** Model output follows its input, so a prompt injected through a web page, an email or a file can make it write code, markup or SQL that then runs with the program's rights (CWE-94, CWE-79).

```codelaw
match:
  any:
    - pattern: eval($V)
    - pattern: new Function($V)
    - pattern: exec($V)
    - pattern: execSync($V)
    - pattern: child_process.exec($V)
    - pattern: cp.exec($V)
    - pattern: $E.query($V)
    - pattern: $E.raw($V)
    - pattern: $E.execute($V)
    - pattern: $E.innerHTML = $V
    - pattern: $E.insertAdjacentHTML($P, $V)
where:
  $V: { from_source: { tables: [sources/llm] } }
message: "`$V` is model output and reaches a place where it runs or is parsed as markup"
```

## How to fix

1. Treat model output as untrusted input: parse it as data (`JSON.parse`) and check it against a schema before use.
2. Write it into the page as text (`textContent`) or through a sanitizer, never as HTML.
3. Never run it as code or as a shell or SQL command; let the model choose among actions the program already has.

## Examples

### JavaScript

#### Don't

```js
const out = completion.choices[0].message.content;
eval(out);
```

#### Do

```js
const out = completion.choices[0].message.content;
const data = JSON.parse(out);
```

### JSX

#### Don't

```jsx
const out = completion.choices[0].message.content;
eval(out);
```

#### Do

```jsx
const out = completion.choices[0].message.content;
const data = JSON.parse(out);
```

### TypeScript

#### Don't

```ts
const out = completion.choices[0].message.content;
eval(out);
```

#### Do

```ts
const out = completion.choices[0].message.content;
const data = JSON.parse(out);
```

### TSX

#### Don't

```tsx
const out = completion.choices[0].message.content;
eval(out);
```

#### Do

```tsx
const out = completion.choices[0].message.content;
const data = JSON.parse(out);
```
