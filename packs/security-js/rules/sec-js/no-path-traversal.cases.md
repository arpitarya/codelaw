# sec-js/no-path-traversal — cases

### case: a name bound to the request in the same function

```js
function show(req, res) {
  const name = req.params.file;
  res.sendFile(name);
}
```

```expect
3:3 sec-js/no-path-traversal
```

### case: a constant path

```js
fs.readFileSync(configPath);
```

```expect
(none)
```

### case: createReadStream

```js
fs.createReadStream(req.query.f);
```

```expect
1:1 sec-js/no-path-traversal
```

### case: path.join with a source

```js
const p = path.join(root, req.params.a);
```

```expect
1:11 sec-js/no-path-traversal
```

### case: sendFile

```js
res.sendFile(req.params.f);
```

```expect
1:1 sec-js/no-path-traversal
```
