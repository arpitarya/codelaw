# Cases: spell/known-typo-key

### case: a JSON key is reported once per file, a value is not

```json
{ "recieve": { "adress": "x" }, "list": [{ "adress": 1 }] }
```

```expect
1:4 spell/known-typo-key
1:17 spell/known-typo-key
```

### case: TOML table headers and quoted keys

```toml
[recieve]
"lenght" = 1
```

```expect
1:2 spell/known-typo-key
2:2 spell/known-typo-key
```

### case: CSS classes, ids and custom properties, not property values

```css
.recieve-all #adress { --lenght: 1; font-family: adress; }
```

```expect
1:2 spell/known-typo-key
1:15 spell/known-typo-key
1:24 spell/known-typo-key
```

### case: an HTML id, not a class or a title

```html
<p id="adress" class="recieve" title="lenght">x</p>
```

```expect
1:8 spell/known-typo-key
```
