# sec-js/no-sql-template — cases

### case: concatenation

```js
db.query("SELECT * FROM t WHERE a = " + a);
```

```expect
1:1 sec-js/no-sql-template
```

### case: a template with no SQL is not flagged

```js
cache.get(`user:${id}`);
```

```expect
(none)
```

### case: a literal template is fine

```js
db.query(`SELECT 1`);
```

```expect
(none)
```

### case: raw with a template

```js
knex.raw(`SELECT * FROM t WHERE a = ${a}`);
```

```expect
1:1 sec-js/no-sql-template
```

### case: a placeholder

```js
db.query("SELECT 1 WHERE a = $1", [a]);
```

```expect
(none)
```
