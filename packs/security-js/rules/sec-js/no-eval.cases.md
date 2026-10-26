# sec-js/no-eval — cases

### case: a function passed to a timer is fine

```js
setTimeout(() => tick(), 100);
setInterval(tick, 1000);
```

```expect
(none)
```

### case: a template string timer

```js
setInterval(`tick(${n})`, 100);
```

```expect
1:1 sec-js/no-eval
```

### case: eval of a name

```js
eval(code);
```

```expect
1:1 sec-js/no-eval
```

### case: new Function

```js
const f = new Function("a", "return a");
```

```expect
1:11 sec-js/no-eval
```

### case: Function called

```js
Function("return 1")();
```

```expect
1:1 sec-js/no-eval
```
