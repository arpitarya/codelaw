# Cases: css/declaration-block-no-duplicate-properties

### case: a fallback with a different syntax is allowed

```css
a {
  width: 100px;
  width: 50vw;
}
```

```expect
(none)
```

### case: the same syntax twice in a row is reported

```css
a {
  font-size: 16px;
  font-size: 14px;
}
```

```expect
2:3 css/declaration-block-no-duplicate-properties
```

```fixed
a {
  font-size: 14px;
}
```

### case: `src` repeats in @font-face

```css
@font-face {
  font-family: "Inter";
  src: url("a.eot");
  src: url("a.woff2") format("woff2");
}
```

```expect
(none)
```

### case: an earlier !important wins

```css
a {
  color: red !important;
  background: none;
  color: blue;
}
```

```expect
4:3 css/declaration-block-no-duplicate-properties
```
