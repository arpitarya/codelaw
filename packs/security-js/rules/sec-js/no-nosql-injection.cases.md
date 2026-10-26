# sec-js/no-nosql-injection — cases

### case: a $where key

```js
Model.find({ $where: "this.a == " + a });
```

```expect
1:14 sec-js/no-nosql-injection
```

### case: a named field is fine

```js
Model.find({ a: String(req.query.a) });
```

```expect
(none)
```

### case: find with the query

```js
Item.find(req.query);
```

```expect
1:1 sec-js/no-nosql-injection
```

### case: update filter from the body

```js
Item.updateOne(req.body, { $set: { a: 1 } });
```

```expect
1:1 sec-js/no-nosql-injection
```
