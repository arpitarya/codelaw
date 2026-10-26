# sec-ts/nest-validation-whitelist — cases

### case: options with no whitelist have no mechanical fix

```ts
app.useGlobalPipes(new ValidationPipe({ transform: true }));
```

```expect
1:20 sec-ts/nest-validation-whitelist
```

### case: whitelist and forbidNonWhitelisted

```ts
new ValidationPipe({ whitelist: true, forbidNonWhitelisted: true });
```

```expect
(none)
```

### case: a typed argument

```ts
new ValidationPipe({ transform: true, whitelist: true });
```

```expect
(none)
```
