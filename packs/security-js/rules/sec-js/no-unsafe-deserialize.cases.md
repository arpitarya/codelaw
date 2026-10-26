# sec-js/no-unsafe-deserialize — cases

### case: node-serialize has no safe mode

```js
const obj = serialize.unserialize(cookie);
```

```expect
1:13 sec-js/no-unsafe-deserialize
```

### case: a loader given a schema

```js
yaml.load(text, { schema: yaml.JSON_SCHEMA });
```

```expect
(none)
```

### case: loadAll

```js
yaml.loadAll(text);
```

```expect
1:1 sec-js/no-unsafe-deserialize
```

### case: unserialize bare

```js
unserialize(data);
```

```expect
1:1 sec-js/no-unsafe-deserialize
```
