# Cases: css/no-invalid-position-declaration

### case: nested group rules and descriptor at-rules

```css
a {
  @media print {
    color: black;
  }
}

@font-face {
  font-family: "Inter";
}

@page {
  margin: 1in;
}
```

```expect
(none)
```

### case: directly inside a top-level @media

```css
@media print {
  color: black;
}
```

```expect
2:3 css/no-invalid-position-declaration
```
