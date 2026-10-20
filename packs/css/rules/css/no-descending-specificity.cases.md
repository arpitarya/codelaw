# Cases: css/no-descending-specificity

### case: different media contexts are not compared

```css
#nav a {
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

### case: pseudo-elements are compared only with themselves

```css
a::before {
  content: "";
}

a {
  color: red;
}
```

```expect
(none)
```

### case: pseudo-classes are ignored when comparing

```css
a:hover {
  color: red;
}

a {
  color: blue;
}
```

```expect
5:1 css/no-descending-specificity
```

### case: nested selectors are resolved

```css
#nav {
  a {
    color: red;
  }
}

a {
  color: blue;
}
```

```expect
7:1 css/no-descending-specificity
```
