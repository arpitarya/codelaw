# js/max-lines-per-function — cases

### case: a function declaration

```js options={"max":2}
function a() {
  b();
  c();
}
```

```expect
1:10 js/max-lines-per-function
```

### case: an arrow function takes its variable's name

```js options={"max":2}
const run = () => {
  b();
  c();
};
```

```expect
1:7 js/max-lines-per-function
```

### case: a function expression

```js options={"max":2}
const run = function () {
  b();
  c();
};
```

```expect
1:7 js/max-lines-per-function
```

### case: a method

```js options={"max":2}
class A {
  run() {
    b();
    c();
  }
}
```

```expect
2:3 js/max-lines-per-function
```

### case: a getter and a setter

```js options={"max":2}
const o = {
  get x() {
    b();
    return 1;
  },
  set x(v) {
    b();
    c();
  },
};
```

```expect
2:7 js/max-lines-per-function
6:7 js/max-lines-per-function
```

### case: a generator and an async function

```js options={"max":2}
function* g() {
  yield 1;
  yield 2;
}
async function f() {
  await 1;
  await 2;
}
```

```expect
1:11 js/max-lines-per-function
5:16 js/max-lines-per-function
```

### case: a class static block

```js options={"max":2}
class A {
  static {
    b();
    c();
  }
}
```

```expect
2:3 js/max-lines-per-function
```

### case: an object property function takes its key

```js options={"max":2}
const o = {
  run: function () {
    b();
    c();
  },
};
```

```expect
2:3 js/max-lines-per-function
```

### case: an anonymous function is reported on its first line

```js options={"max":2}
use(function () {
  b();
  c();
});
```

```expect
1:5 js/max-lines-per-function
```

### case: blank lines and comments are not counted

```js options={"max":3}
function a() {

  // note
  b();
}
```

```expect
(none)
```

### case: blank lines counted

```js options={"max":3,"skip_blank_lines":false}
function a() {

  b();
}
```

```expect
1:10 js/max-lines-per-function
```

### case: comments counted

```js options={"max":3,"skip_comments":false}
function a() {
  // note
  b();
}
```

```expect
1:10 js/max-lines-per-function
```

### case: an inner function counts toward the outer one

```js options={"max":4}
function outer() {
  const inner = () => {
    b();
  };
  inner();
}
```

```expect
1:10 js/max-lines-per-function
```

### case: an inner function left out of the outer one

```js options={"max":4,"include_nested":false}
function outer() {
  const inner = () => {
    b();
  };
  inner();
}
```

```expect
(none)
```

### case: an immediately invoked function is skipped

```js options={"max":2}
(() => {
  b();
  c();
})();
```

```expect
(none)
```

### case: an immediately invoked function, included

```js options={"max":2,"include_iife":true}
(() => {
  b();
  c();
})();
```

```expect
1:2 js/max-lines-per-function
```

### case: an iife counts toward the function around it

```js options={"max":4,"include_nested":false}
function outer() {
  (() => {
    b();
  })();
  c();
}
```

```expect
1:10 js/max-lines-per-function
```

### case: ignore_names

```js options={"max":2,"ignore_names":"^(test|it)"}
function test_a() {
  b();
  c();
}
function other() {
  b();
  c();
}
```

```expect
5:10 js/max-lines-per-function
```

### case: a function at the limit passes

```js options={"max":3}
function a() {
  b();
}
```

```expect
(none)
```

### case: a TypeScript overload signature has no body

```ts options={"max":3}
function a(x: number): void;
function a(x: string): void;
function a(x: any) {
  b();
}
```

```expect
(none)
```

### case: a TypeScript method

```ts options={"max":2}
class A {
  private run(): void {
    b();
    c();
  }
}
```

```expect
2:11 js/max-lines-per-function
```

### case: a TSX component

```tsx options={"max":2}
const A = () => {
  const x = 1;
  return <b>{x}</b>;
};
```

```expect
1:7 js/max-lines-per-function
```

### case: an invalid ignore_names is reported once, and the function still measured

```js options={"max":2,"ignore_names":"("}
function a() {
  b();
  c();
}
```

```expect
1:1 js/max-lines-per-function
1:10 js/max-lines-per-function
```
