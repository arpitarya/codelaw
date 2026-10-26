# sec-js/no-cors-wildcard-credentials — cases

### case: a star

```js
cors({ credentials: true, origin: "*" });
```

```expect
1:1 sec-js/no-cors-wildcard-credentials
```

### case: an allow-list

```js
cors({ origin: ["https://a.example"], credentials: true });
```

```expect
(none)
```
