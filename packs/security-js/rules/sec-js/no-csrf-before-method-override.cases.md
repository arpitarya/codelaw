# sec-js/no-csrf-before-method-override — cases

### case: csrf first is fine

```js
app.use(csurf());
app.use(methodOverride("_method"));
```

```expect
(none)
```
