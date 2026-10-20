# Cases: css/font-family-no-duplicate-names

### case: a quoted keyword differs from the keyword

```css
a {
  font-family: "serif", serif;
}
```

```expect
(none)
```

### case: duplicates in the font shorthand

```css
a {
  font: 12px/1.5 Arial, Arial, sans-serif;
}
```

```expect
2:25 css/font-family-no-duplicate-names
```

```fixed
a {
  font: 12px/1.5 Arial, sans-serif;
}
```
