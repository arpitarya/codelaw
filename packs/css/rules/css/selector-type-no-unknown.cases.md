# Cases: css/selector-type-no-unknown

### case: custom elements and namespaces

```css
my-element,
svg|rect,
foreignObject {
  color: red;
}
```

```expect
(none)
```

### case: keyframe selectors are not type selectors

```css
@keyframes fade {
  from {
    opacity: 0;
  }
}
```

```expect
(none)
```

### case: inside :is()

```css
:is(tabel, div) {
  color: red;
}
```

```expect
1:5 css/selector-type-no-unknown
```
