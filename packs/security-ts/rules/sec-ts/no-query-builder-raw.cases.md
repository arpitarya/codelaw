# sec-ts/no-query-builder-raw — cases

### case: em.execute with a built string

```ts
await this.em.execute(`select * from t where a = ${a}`);
```

```expect
1:7 sec-ts/no-query-builder-raw
```

### case: sql.lit of a variable

```ts
const f = sql.lit(name);
```

```expect
1:11 sec-ts/no-query-builder-raw
```

### case: a literal fragment

```ts
const f = sql.raw("now()");
```

```expect
(none)
```

### case: sql.raw of a template

```ts
const q = sql.raw(`order by ${col}`);
```

```expect
1:11 sec-ts/no-query-builder-raw
```
