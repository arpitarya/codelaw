# sec-js/no-xpath-injection — cases

### case: evaluate on a document

```js
document.evaluate("//a[@id='" + id + "']", document);
```

```expect
1:1 sec-js/no-xpath-injection
```

### case: a literal expression

```js
xpath.select("//user", doc);
```

```expect
(none)
```
