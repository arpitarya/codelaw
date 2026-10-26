# sec-js/no-ssti — cases

### case: a name bound to the request in the same function

```js
function show(req, res) {
  const t = req.query.t;
  res.send(pug.compile(t)());
}
```

```expect
3:12 sec-js/no-ssti
```

### case: a constant template

```js
const fn = pug.compile(source);
```

```expect
(none)
```

### case: pug compile

```js
pug.compile(req.body.tpl);
```

```expect
1:1 sec-js/no-ssti
```

### case: handlebars

```js
Handlebars.compile(req.query.t);
```

```expect
1:1 sec-js/no-ssti
```

### case: a constant template

```js
res.render("home", { a: 1 });
```

```expect
(none)
```
