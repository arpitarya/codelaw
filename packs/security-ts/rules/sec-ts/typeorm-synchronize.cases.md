# sec-ts/typeorm-synchronize — cases

### case: in a module

```ts
TypeOrmModule.forRoot({ type: "mysql", synchronize: true });
```

```expect
1:40 sec-ts/typeorm-synchronize
```

### case: false

```ts
new DataSource({ synchronize: false });
```

```expect
(none)
```
