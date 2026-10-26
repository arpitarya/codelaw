# js/max-params — cases

### case: a function declaration

```js options={"max":2}
function a(x, y, z) {}
```

```expect
1:10 js/max-params
```

### case: an arrow function

```js options={"max":2}
const f = (x, y, z) => 1;
```

```expect
1:7 js/max-params
```

### case: a single arrow parameter without parentheses

```js options={"max":0}
const f = x => x;
```

```expect
1:7 js/max-params
```

### case: a method and a constructor

```js options={"max":2}
class A {
  constructor(a, b, c) {}
  run(a, b, c) {}
}
```

```expect
2:3 js/max-params
3:3 js/max-params
```

### case: a generator and an async function

```js options={"max":2}
function* g(a, b, c) {}
async function f(a, b, c) {}
```

```expect
1:11 js/max-params
2:16 js/max-params
```

### case: a destructured parameter is one

```js options={"max":2}
function a({ x, y, z }, [b, c]) {}
```

```expect
(none)
```

### case: default values count

```js options={"max":2}
function a(x, y = 1, z = 2) {}
```

```expect
1:10 js/max-params
```

### case: a rest parameter is not counted

```js options={"max":2}
function a(x, y, ...rest) {}
```

```expect
(none)
```

### case: a rest parameter, counted

```js options={"max":2,"count_varargs":true}
function a(x, y, ...rest) {}
```

```expect
1:10 js/max-params
```

### case: ignore_names

```js options={"max":2,"ignore_names":"^_"}
function a(x, _y, _z) {}
```

```expect
(none)
```

### case: a parameter that does not match ignore_names is counted

```js options={"max":2,"ignore_names":"^_"}
function a(x, y, z, _w) {}
```

```expect
1:10 js/max-params
```

### case: a function at the limit passes

```js options={"max":2}
function a(x, y) {}
```

```expect
(none)
```

### case: a TypeScript this parameter is not counted

```ts options={"max":2}
function a(this: Foo, x: number, y: number) {}
```

```expect
(none)
```

### case: a TypeScript this parameter, counted

```ts options={"max":2,"count_this":true}
function a(this: Foo, x: number, y: number) {}
```

```expect
1:10 js/max-params
```

### case: a TypeScript override method is skipped

```ts options={"max":2}
class A extends B {
  override run(a: number, b: number, c: number) {}
}
```

```expect
(none)
```

### case: a TypeScript override method, counted

```ts options={"max":2,"ignore_overrides":false}
class A extends B {
  override run(a: number, b: number, c: number) {}
}
```

```expect
2:12 js/max-params
```

### case: a TypeScript overload signature and a declare function are skipped

```ts options={"max":2}
declare function a(x: number, y: number, z: number): void;
function b(x: number, y: number, z: number): void;
function b(x: any, y: any, z: any) {}
```

```expect
3:10 js/max-params
```

### case: a TypeScript abstract method is skipped

```ts options={"max":2}
abstract class A {
  abstract run(a: number, b: number, c: number): void;
}
```

```expect
(none)
```

### case: declarations, counted

```ts options={"max":2,"ignore_declarations":false}
declare function a(x: number, y: number, z: number): void;
abstract class A {
  abstract run(a: number, b: number, c: number): void;
}
```

```expect
1:18 js/max-params
3:12 js/max-params
```

### case: constructor injection

```ts options={"max":2}
class A {
  constructor(private a: X, private b: Y, readonly c: Z) {}
}
```

```expect
2:3 js/max-params
```

### case: constructor injection, skipped

```ts options={"max":2,"ignore_constructor_injection":true}
class A {
  constructor(private a: X, private b: Y, readonly c: Z) {}
}
```

```expect
(none)
```

### case: constructor injection mixed with a plain parameter is counted

```ts options={"max":2,"ignore_constructor_injection":true}
class A {
  constructor(private a: X, private b: Y, c: Z) {}
}
```

```expect
2:3 js/max-params
```

### case: a TSX component

```tsx options={"max":2}
const A = (a: number, b: number, c: number) => <i />;
```

```expect
1:7 js/max-params
```

### case: an invalid ignore_names is reported once, and the function still measured

```js options={"max":2,"ignore_names":"("}
function a(x, y, z) {}
```

```expect
1:1 js/max-params
1:10 js/max-params
```
