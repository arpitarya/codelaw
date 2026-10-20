# Cases: css/font-family-no-missing-generic-family-keyword

### case: system fonts in the font shorthand

```css
a {
  font: caption;
}
```

```expect
(none)
```

### case: values with var()

```css
a {
  font-family: var(--font), Arial;
}
```

```expect
(none)
```

### case: @font-face names a family

```css
@font-face {
  font-family: "Inter";
  src: url("inter.woff2");
}
```

```expect
(none)
```

### case: the family part of font

```css
a {
  font: bold 12px/1.5 Helvetica, Arial;
}
```

```expect
2:34 css/font-family-no-missing-generic-family-keyword
```

### case: a quoted keyword is a family name

```css
a {
  font-family: "sans-serif";
}
```

```expect
2:16 css/font-family-no-missing-generic-family-keyword
```
