# sec-js/no-xxe — cases

### case: noent last

```js
libxmljs.parseXml(xml, { nonet: true, noent: true });
```

```expect
1:1 sec-js/no-xxe
```

### case: noent false

```js
libxmljs.parseXml(xml, { noent: false });
```

```expect
(none)
```
