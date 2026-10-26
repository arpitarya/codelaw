# Cases: scss/percent-placeholder-pattern

### case: a camel-case placeholder

```scss
%btnBase {
  color: red;
}
```

```expect
1:1 scss/percent-placeholder-pattern
```

### case: a percent in a value is not a placeholder

```scss
.a {
  width: 100%;
}
```

```expect
(none)
```
