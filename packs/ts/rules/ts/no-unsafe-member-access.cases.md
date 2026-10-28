# Cases: ts/no-unsafe-member-access

### case: a chain on an any value is reported once, at its first access

```ts
declare const config: any;
const host = config.server.host;
```

```expect
2:14 ts/no-unsafe-member-access
```

### case: an any property of a typed value is reported where it is read from

```ts
declare const config: { extra: any };
const host = config.extra.host;
```

```expect
2:14 ts/no-unsafe-member-access
```

### case: a computed access on an any value

```ts
declare const row: any;
const first = row[0];
```

```expect
2:15 ts/no-unsafe-member-access
```
