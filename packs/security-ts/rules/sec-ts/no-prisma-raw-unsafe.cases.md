# sec-ts/no-prisma-raw-unsafe — cases

### case: executeRawUnsafe with a concatenation has no mechanical fix

```ts
await prisma.$executeRawUnsafe("DELETE FROM t WHERE a = " + a);
```

```expect
1:7 sec-ts/no-prisma-raw-unsafe
```

### case: the tagged template

```ts
await prisma.$executeRaw`DELETE FROM t WHERE a = ${a}`;
```

```expect
(none)
```

### case: a literal string

```ts
await prisma.$queryRawUnsafe("SELECT 1");
```

```expect
(none)
```
