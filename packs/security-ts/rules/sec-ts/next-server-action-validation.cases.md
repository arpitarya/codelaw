# sec-ts/next-server-action-validation — cases

### case: a file with no use server directive

```ts
export async function f(input: FormData) {
  await save(input);
}
```

```expect
(none)
```

### case: a schema parse passes

```ts
"use server";
export async function f(input: unknown) {
  const data = postSchema.parse(input);
  await save(data);
}
```

```expect
(none)
```

### case: zod imported and used inline passes

```ts
"use server";
import { z } from "zod";
export async function f(input: unknown) {
  await save(z.object({ title: z.string() }).parse(input));
}
```

```expect
(none)
```

### case: a validator in a nested callback passes

```ts
"use server";
export async function f(items: unknown[]) {
  const ok = items.map((i) => itemSchema.safeParse(i));
  await save(ok);
}
```

```expect
(none)
```

### case: a bare validate that nothing imports is reported

```ts
"use server";
export async function f(input: unknown) {
  validate(input);
}
```

```expect
2:8 sec-ts/next-server-action-validation
```

### case: JSON.parse is not a validator

```ts
"use server";
export async function f(input: string) {
  await save(JSON.parse(input));
}
```

```expect
2:8 sec-ts/next-server-action-validation
```

### case: extend: the project validator and the built-in both pass

```ts options={"validators":[{"name":"checkInput","module":"@acme/validation"}]}
"use server";
import { checkInput } from "@acme/validation";
export async function f(input: unknown) {
  await save(checkInput(input));
}
export async function g(input: unknown) {
  await save(postSchema.parse(input));
}
```

```expect
(none)
```

### case: replace: only the project validator passes

```ts options={"validators":[{"name":"checkInput","module":"@acme/validation"}],"validators_mode":"replace"}
"use server";
import { checkInput } from "@acme/validation";
export async function f(input: unknown) {
  await save(checkInput(input));
}
export async function g(input: unknown) {
  await save(postSchema.parse(input));
}
```

```expect
6:8 sec-ts/next-server-action-validation
```

### case: an aliased import passes

```ts options={"validators":[{"name":"checkInput","module":"@acme/validation"}],"validators_mode":"replace"}
"use server";
import { checkInput as check } from "@acme/validation";
export async function f(input: unknown) {
  await save(check(input));
}
```

```expect
(none)
```

### case: a same-named local function is reported

```ts options={"validators":[{"name":"checkInput","module":"@acme/validation"}]}
"use server";
function checkInput(x: unknown) { return x; }
export async function f(input: unknown) {
  await save(checkInput(input));
}
```

```expect
3:8 sec-ts/next-server-action-validation
```

### case: the wrong module is reported

```ts options={"validators":[{"name":"checkInput","module":"@acme/validation"}]}
"use server";
import { checkInput } from "@other/validation";
export async function f(input: unknown) {
  await save(checkInput(input));
}
```

```expect
3:8 sec-ts/next-server-action-validation
```

### case: no arguments

```ts
"use server";
export async function f() {
  await run();
}
```

```expect
(none)
```
