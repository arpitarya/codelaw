# js/no-console — cases

### case: a bracketed method name

```js
console["warn"]("low disk");
```

```expect
1:1 js/no-console
```

```fixed
```

### case: a local console is not the global one

```js
const console = { log() {} };
console.log("x");
```

```expect
(none)
```

### case: a used value: no edit

```js
const out = console.log("x") ?? 1;
```

```expect
1:13 js/no-console
```

### case: the body of an if without braces: no edit

```js
if (debug) console.log("x");
```

```expect
1:12 js/no-console
```

### case: a parameter named console

```js
function run(console) {
  console.log("x");
}
```

```expect
(none)
```

### case: other members of an object are not console calls

```js
logger.log("x");
console.log;
```

```expect
(none)
```
