# sec-js/no-non-literal-fs-filename — cases

### case: fs.promises

```js
fs.promises.readFile(file);
```

```expect
1:1 sec-js/no-non-literal-fs-filename
```

### case: a map get is not fs

```js
cache.readFile(file);
```

```expect
(none)
```

### case: a variable

```js
fs.readFileSync(file);
```

```expect
1:1 sec-js/no-non-literal-fs-filename
```
