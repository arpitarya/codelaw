# sec-js/no-bad-tag-filter — cases

### case: a generic tag

```js
html.replace(/<[^>]*>/g, "");
```

```expect
1:1 sec-js/no-bad-tag-filter
```
