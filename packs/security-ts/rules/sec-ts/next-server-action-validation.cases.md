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

### case: a call to validate

```ts
"use server";
export async function f(input: unknown) {
  validate(input);
}
```

```expect
(none)
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
