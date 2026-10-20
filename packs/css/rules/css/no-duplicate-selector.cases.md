# Cases: css/no-duplicate-selector

### case: different at-rule contexts

```css
a {
  color: red;
}

@media print {
  a {
    color: black;
  }
}
```

```expect
(none)
```

### case: the same list in another order

```css
a, b {
  color: red;
}

b, a {
  margin: 0;
}
```

```expect
5:1 css/no-duplicate-selector
```

### case: one selector twice in a list

```css
a, b, a {
  color: red;
}
```

```expect
1:7 css/no-duplicate-selector
```

### case: keyframe selectors are not style rules

```css
@keyframes a {
  0% {
    top: 0;
  }
}

@keyframes b {
  0% {
    top: 1px;
  }
}
```

```expect
(none)
```
