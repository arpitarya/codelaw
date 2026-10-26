# sec-ts/nest-missing-guard — cases

### case: a class that is not exported

```ts
@Controller("a")
class A {
  @Get()
  list() {}
}
```

```expect
1:1 sec-ts/nest-missing-guard
```

### case: a global guard in the file

```ts
@Controller("a")
export class A {}
providers: [{ provide: APP_GUARD, useClass: AuthGuard }];
```

```expect
(none)
```
