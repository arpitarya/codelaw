# sec-ts/no-typeorm-string-query — cases

### case: repo.query with a template

```ts
await repo.query(`SELECT * FROM t WHERE a = ${a}`);
```

```expect
1:7 sec-ts/no-typeorm-string-query
```

### case: a concatenation

```ts
qb.andWhere("a = " + a);
```

```expect
1:1 sec-ts/no-typeorm-string-query
```

### case: a named parameter

```ts
qb.where("a = :a", { a });
```

```expect
(none)
```

### case: having

```ts
qb.having(`COUNT(x) > ${n}`);
```

```expect
1:1 sec-ts/no-typeorm-string-query
```
