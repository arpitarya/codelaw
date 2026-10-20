# Cases: css/length-zero-no-unit

### case: where the unit matters

```css
a {
  --gap: 0px;
  line-height: 0px;
  flex: 1 1 0px;
  width: calc(0px + 1em);
  font: 12px/0px serif;
  grid-template-columns: 0fr;
}
```

```expect
(none)
```

### case: zero lengths in a media query

```css
@media (min-width: 0px) {
  a {
    margin: 0em;
  }
}
```

```expect
1:20 css/length-zero-no-unit
3:13 css/length-zero-no-unit
```

```fixed
@media (min-width: 0) {
  a {
    margin: 0;
  }
}
```
