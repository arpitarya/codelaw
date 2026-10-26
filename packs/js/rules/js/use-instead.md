---
id: js/use-instead
title: "Use the project's function instead of the one it wraps"
languages: [javascript, jsx, typescript, tsx]
fix: auto
stability: preview
options:
  replacements:
    type: "object"
    default: {}
    description: "a dotted path to { use, from, drop_in?, allow_in? }: `use` is the function or variable to call instead, `from` the module it comes from, `drop_in` lets `codelaw fix` swap it in and add the import, `allow_in` lists globs of files exempt (the wrapper itself)"
since: 0.3.0
---

A project names the function it wants called instead of another, once, and codelaw holds the code to it. The rule is inert until `replacements` is set. A key is a dotted path, matched on a call's callee or a member read, exactly or as a prefix at a `.` boundary (`process.env` matches `process.env.API_URL`), and only where its first name is a global or an import, never a local that shadows it. A file that matches the entry's `allow_in` is exempt.

```toml
[rules."js/use-instead".options.replacements]
"fetch"              = { use = "apiFetch",     from = "@acme/http",     drop_in = true, allow_in = ["src/lib/http.ts"] }
"DOMPurify.sanitize" = { use = "sanitizeHtml", from = "@acme/security", drop_in = true, allow_in = ["src/security/**"] }
"console.log"        = { use = "logger.info",  from = "@acme/log" }
```

**Why:** A wrapper is where a team puts what every call needs - an auth header, a timeout, an approved sanitizer, a logger. Code that calls the wrapped function directly skips all of it, and only a rule can notice. It complements the rules that name an approved function at one sink (`sanitizers`): this one reports the wrong function anywhere.

**Limits:** it matches by the spelling of a path and where its first name comes from; it does not follow a value through a variable (`const f = fetch; f(x)`), and a shorthand property (`{ fetch }`) is not a read it sees. The wrapper calls the thing it wraps, so name its file in `allow_in`.

```codelaw
native: js/use-instead
```

## How to fix

1. With `drop_in = true`, run `codelaw fix`: the path becomes `use` and the import is added when the file lacks it.
2. Without `drop_in`, change the call by hand: the replacement takes other arguments or returns another shape.
3. In the wrapper's own file, add the file to `allow_in` rather than the call to the baseline.

## Examples

### JavaScript

#### Don't

```js options={"replacements":{"fetch":{"use":"apiFetch","from":"@acme/http","drop_in":true}}}
fetch("/api/orders");
```

#### Do

```js options={"replacements":{"fetch":{"use":"apiFetch","from":"@acme/http","drop_in":true}}}
import { apiFetch } from "@acme/http";
apiFetch("/api/orders");
```

### JSX

#### Don't

```jsx options={"replacements":{"fetch":{"use":"apiFetch","from":"@acme/http","drop_in":true}}}
const Orders = () => <button onClick={() => fetch("/api/orders")}>load</button>;
```

#### Do

```jsx options={"replacements":{"fetch":{"use":"apiFetch","from":"@acme/http","drop_in":true}}}
import { apiFetch } from "@acme/http";
const Orders = () => <button onClick={() => apiFetch("/api/orders")}>load</button>;
```

### TypeScript

#### Don't

```ts options={"replacements":{"fetch":{"use":"apiFetch","from":"@acme/http","drop_in":true}}}
const load = (): Promise<Response> => fetch("/api/orders");
```

#### Do

```ts options={"replacements":{"fetch":{"use":"apiFetch","from":"@acme/http","drop_in":true}}}
import { apiFetch } from "@acme/http";
const load = (): Promise<Response> => apiFetch("/api/orders");
```

### TSX

#### Don't

```tsx options={"replacements":{"fetch":{"use":"apiFetch","from":"@acme/http","drop_in":true}}}
const Orders = () => <button onClick={() => fetch("/api/orders")}>load</button>;
```

#### Do

```tsx options={"replacements":{"fetch":{"use":"apiFetch","from":"@acme/http","drop_in":true}}}
import { apiFetch } from "@acme/http";
const Orders = () => <button onClick={() => apiFetch("/api/orders")}>load</button>;
```
