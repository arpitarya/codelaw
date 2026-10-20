# Cases: css/no-empty-block

### case: a block holding a comment is not empty

```css
a {
  /* todo */
}
```

```expect
(none)
```

### case: a named layer block orders the cascade

```css
@layer base {}
```

```expect
(none)
```

### case: an empty block nested in a rule

```css
a {
  color: red;
  &:hover {}
}
```

```expect
3:3 css/no-empty-block
```

```fixed
a {
  color: red;
}
```
