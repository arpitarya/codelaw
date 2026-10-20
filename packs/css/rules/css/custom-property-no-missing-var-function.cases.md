# Cases: css/custom-property-no-missing-var-function

### case: properties that take names

```css
:root {
  --fade: 1;
}

a {
  transition-property: --fade;
}
```

```expect
(none)
```

### case: arguments of a custom function

```css
:root {
  --base: 1rem;
}

a {
  margin: --theme(--base);
}
```

```expect
(none)
```

### case: a bare name in a var() fallback

```css
:root {
  --b: 1px;
}

a {
  margin: var(--a, --b);
}
```

```expect
6:20 css/custom-property-no-missing-var-function
```

### case: an undefined name is not checked

```css
a {
  color: --brand;
}
```

```expect
(none)
```
