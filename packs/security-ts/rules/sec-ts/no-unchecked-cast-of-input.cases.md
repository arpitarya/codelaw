# sec-ts/no-unchecked-cast-of-input — cases

### case: an awaited json() cast

```ts
async function load(res: Response) {
  return (await res.json()) as Page;
}
```

```expect
2:10 sec-ts/no-unchecked-cast-of-input
```

### case: a function that validates is fine

```ts
function f(req: Request) {
  const ok = schema.parse(req.body);
  return req.body as Page;
}
```

```expect
(none)
```

### case: a validator in the enclosing function covers a callback

```ts
function f(req: Request) {
  bodySchema.safeParse(req.body);
  return [1].map(() => req.body as Page);
}
```

```expect
(none)
```

### case: a validator in another function does not count

```ts
function check(req: Request) {
  return bodySchema.safeParse(req.body);
}
function f(req: Request) {
  return req.body as Page;
}
```

```expect
5:10 sec-ts/no-unchecked-cast-of-input
```

### case: JSON.parse cast

```ts
const cfg = JSON.parse(text) as Config;
```

```expect
1:13 sec-ts/no-unchecked-cast-of-input
```

### case: JSON.parse in the function is not a validator

```ts
function f(text: string) {
  return JSON.parse(text) as Config;
}
```

```expect
2:10 sec-ts/no-unchecked-cast-of-input
```

### case: a query cast

```ts
const q = req.query as SearchQuery;
```

```expect
1:11 sec-ts/no-unchecked-cast-of-input
```

### case: class-validator imported passes

```ts
import { validateOrReject } from "class-validator";
async function f(req: Request) {
  await validateOrReject(req.body);
  return req.body as Dto;
}
```

```expect
(none)
```

### case: extend: the project validator and the built-in both pass

```ts options={"validators":[{"name":"checkInput","module":"@acme/validation"}]}
import { checkInput } from "@acme/validation";
function f(req: Request) {
  checkInput(req.body);
  return req.body as Page;
}
function g(req: Request) {
  pageSchema.parse(req.body);
  return req.body as Page;
}
```

```expect
(none)
```

### case: replace: only the project validator passes

```ts options={"validators":[{"name":"checkInput","module":"@acme/validation"}],"validators_mode":"replace"}
import { checkInput } from "@acme/validation";
function f(req: Request) {
  checkInput(req.body);
  return req.body as Page;
}
function g(req: Request) {
  pageSchema.parse(req.body);
  return req.body as Page;
}
```

```expect
8:10 sec-ts/no-unchecked-cast-of-input
```

### case: an aliased import passes

```ts options={"validators":[{"name":"checkInput","module":"@acme/validation"}],"validators_mode":"replace"}
import { checkInput as check } from "@acme/validation";
function f(req: Request) {
  check(req.body);
  return req.body as Page;
}
```

```expect
(none)
```

### case: a same-named local function is reported

```ts options={"validators":[{"name":"checkInput","module":"@acme/validation"}]}
function checkInput(x: unknown) { return x; }
function f(req: Request) {
  checkInput(req.body);
  return req.body as Page;
}
```

```expect
4:10 sec-ts/no-unchecked-cast-of-input
```

### case: the wrong module is reported

```ts options={"validators":[{"name":"checkInput","module":"@acme/validation"}]}
import { checkInput } from "@other/validation";
function f(req: Request) {
  checkInput(req.body);
  return req.body as Page;
}
```

```expect
4:10 sec-ts/no-unchecked-cast-of-input
```

### case: in TSX

```tsx
function f(req: Request) {
  const b = req.body as Body;
  return <div>{b.name}</div>;
}
```

```expect
2:13 sec-ts/no-unchecked-cast-of-input
```
