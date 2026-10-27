# Cases: css/custom-property-pattern

### case: an interpolated SCSS name is not checked

```scss
.a {
  --#{$prefix}accordion-color: red;
  color: var(--#{$prefix}accordion-color);
}
```

```expect
(none)
```

### case: a plain SCSS name still is

```scss
.a {
  --Accordion-Color: red;
}
```

```expect
2:3 css/custom-property-pattern
```
