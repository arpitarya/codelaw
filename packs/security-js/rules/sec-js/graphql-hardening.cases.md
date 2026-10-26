# sec-js/graphql-hardening — cases

### case: introspection

```js
const o = { introspection: true };
```

```expect
1:13 sec-js/graphql-hardening
```

### case: yoga without limits

```js
createYoga({ schema });
```

```expect
1:1 sec-js/graphql-hardening
```
