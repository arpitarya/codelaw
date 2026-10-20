# Cases: css/declaration-property-value-no-unknown

### case: CSS-wide keywords, prefixes and variables

```css
a {
  position: inherit;
  position: -webkit-sticky;
  position: var(--pos);
  float: inline-start;
}
```

```expect
(none)
```
