# Cases: css/function-calc-no-unspaced-operator

### case: signs, names and exponents

```css
a {
  width: calc(1px - -2px);
  height: calc(var(--a-b) * 2);
  top: calc(1e-3px + 1px);
}
```

```expect
(none)
```

### case: a sign after a value reads as a second value

```css
a {
  width: calc(100% -10px);
}
```

```expect
2:20 css/function-calc-no-unspaced-operator
```

```fixed
a {
  width: calc(100% - 10px);
}
```
