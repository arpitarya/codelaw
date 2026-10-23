# Cases: css/media-feature-range-notation

### case: a min- and max- pair becomes two ranges

```css
@media screen and (min-width: 600px) and (max-width: 899.5px) {
  a {
    color: red;
  }
}
```

```expect
1:19 css/media-feature-range-notation
1:42 css/media-feature-range-notation
```

```fixed
@media screen and (width >= 600px) and (width <= 899.5px) {
  a {
    color: red;
  }
}
```

### case: every query of a list, and ratios

```css
@media (min-aspect-ratio: 16/9), not (max-resolution: 2dppx) {
  a {
    color: red;
  }
}
```

```expect
1:8 css/media-feature-range-notation
1:38 css/media-feature-range-notation
```

```fixed
@media (aspect-ratio >= 16/9), not (resolution <= 2dppx) {
  a {
    color: red;
  }
}
```

### case: range syntax and features without a range are left alone

```css
@media (400px <= width <= 700px) and (hover: hover) and (orientation: landscape) {
  a {
    color: red;
  }
}
```

```expect
(none)
```
