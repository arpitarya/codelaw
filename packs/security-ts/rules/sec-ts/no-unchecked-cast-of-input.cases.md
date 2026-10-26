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

### case: JSON.parse cast

```ts
const cfg = JSON.parse(text) as Config;
```

```expect
1:13 sec-ts/no-unchecked-cast-of-input
```

### case: a query cast

```ts
const q = req.query as SearchQuery;
```

```expect
1:11 sec-ts/no-unchecked-cast-of-input
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
