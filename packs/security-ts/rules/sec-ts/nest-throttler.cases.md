# sec-ts/nest-throttler — cases

### case: an auth controller by name

```ts
@Controller("auth")
export class A {}
```

```expect
1:1 sec-ts/nest-throttler
```

### case: a controller with no sign-in route

```ts
@Controller("posts")
export class P {
  @Get()
  list() {}
}
```

```expect
(none)
```
