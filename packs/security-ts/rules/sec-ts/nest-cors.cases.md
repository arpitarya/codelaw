# sec-ts/nest-cors — cases

### case: cors in the factory options

```ts
const app = await NestFactory.create(AppModule, { cors: true });
```

```expect
1:51 sec-ts/nest-cors
```

### case: reflecting origins with credentials

```ts
app.enableCors({ origin: true, credentials: true });
```

```expect
1:1 sec-ts/nest-cors
```

### case: origin star with credentials

```ts
app.enableCors({ origin: "*", credentials: true });
```

```expect
1:1 sec-ts/nest-cors
```
