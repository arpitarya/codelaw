# Cases: css/alpha-value-notation

### case: opacity takes a number

```css
a {
  opacity: 50%;
}
```

```expect
2:12 css/alpha-value-notation
```

```fixed
a {
  opacity: 0.5;
}
```

### case: the legacy comma syntax

```css
a {
  color: rgba(0, 0, 0, .5);
}
```

```expect
2:24 css/alpha-value-notation
```

```fixed
a {
  color: rgba(0, 0, 0, 50%);
}
```
