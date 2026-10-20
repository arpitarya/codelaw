# Cases: css/shorthand-property-no-redundant-values

### case: four sides that pair up

```css
a {
  padding: 1px 2px 1px 2px;
  border-radius: 4px 4px / 2px 2px;
}
```

```expect
2:12 css/shorthand-property-no-redundant-values
3:18 css/shorthand-property-no-redundant-values
```

```fixed
a {
  padding: 1px 2px;
  border-radius: 4px / 2px;
}
```

### case: values with var() can expand

```css
a {
  margin: var(--x) var(--x);
}
```

```expect
(none)
```
