# Cases: css/no-unknown-property

### case: descriptors inside @font-face are not properties

```css
@font-face {
  font-family: "Inter";
  font-display: swap;
  unicode-range: U+0000-00FF;
}
```

```expect
(none)
```

### case: vendor-prefixed and custom properties are skipped

```css
a {
  -webkit-tap-highlight-color: transparent;
  --brand-colr: red;
}
```

```expect
(none)
```

### case: old IE hacks are skipped

```css
a {
  *zoom: 1;
  _height: 1px;
}
```

```expect
(none)
```

### case: no close known name, no fix

```css
a {
  wibble: 1;
}
```

```expect
2:3 css/no-unknown-property
```
