# sec-ts/no-ts-ignore-on-sink — cases

### case: an as any on a sink line

```ts
const rows = await el.insertAdjacentHTML("beforeend", html as any);
```

```expect
1:55 sec-ts/no-ts-ignore-on-sink
```

### case: a directive above an ordinary line

```ts
// @ts-ignore
const n: number = value;
```

```expect
(none)
```

### case: expect-error above eval

```ts
// @ts-expect-error
eval(cmd);
```

```expect
1:1 sec-ts/no-ts-ignore-on-sink
```

### case: as any on a raw query

```ts
await prisma.$queryRawUnsafe(sql as any);
```

```expect
1:30 sec-ts/no-ts-ignore-on-sink
```
