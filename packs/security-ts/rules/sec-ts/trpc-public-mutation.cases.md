# sec-ts/trpc-public-mutation — cases

### case: a public query is fine

```ts
export const list = publicProcedure.query(() => all());
```

```expect
(none)
```

### case: a protected mutation

```ts
export const del = protectedProcedure.mutation(() => remove());
```

```expect
(none)
```
