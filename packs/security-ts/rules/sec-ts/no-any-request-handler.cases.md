# sec-ts/no-any-request-handler — cases

### case: a NestJS body

```ts
class C {
  create(@Body() body: any) {}
}
```

```expect
2:10 sec-ts/no-any-request-handler
```

### case: a DTO

```ts
class C {
  create(@Body() body: CreateDto) {}
}
```

```expect
(none)
```

### case: request typed any

```ts
function h(request: any) {}
```

```expect
1:12 sec-ts/no-any-request-handler
```

### case: a Param

```ts
class C {
  get(@Param() id: any) {}
}
```

```expect
2:7 sec-ts/no-any-request-handler
```
