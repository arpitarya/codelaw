# sec-js/no-prototype-pollution — cases

### case: a merge of the request body

```js
_.merge(config, req.body);
```

```expect
1:1 sec-js/no-prototype-pollution
```

### case: a literal key

```js
config["theme"] = req.body.theme;
```

```expect
(none)
```

### case: defaultsDeep

```js
_.defaultsDeep(cfg, req.body);
```

```expect
1:1 sec-js/no-prototype-pollution
```

### case: set with a key

```js
_.set(obj, req.query.path, 1);
```

```expect
1:1 sec-js/no-prototype-pollution
```
