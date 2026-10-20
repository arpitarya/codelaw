# Cases: css/string-no-newline

### case: an escaped line break

```css
[title="one\
two"] {
  color: red;
}
```

```expect
(none)
```

### case: values are not checked by default

```css
a {
  content: "one
two";
}
```

```expect
(none)
```
