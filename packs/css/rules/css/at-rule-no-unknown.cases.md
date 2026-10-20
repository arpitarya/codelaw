# Cases: css/at-rule-no-unknown

### case: @charset is not an at-rule here

```css
@charset "utf-8";
```

```expect
(none)
```

### case: vendor-prefixed known at-rules

```css
@-webkit-keyframes fade {
  from {
    opacity: 0;
  }
}
```

```expect
(none)
```
