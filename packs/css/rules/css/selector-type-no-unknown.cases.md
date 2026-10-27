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

### case: an SCSS line comment inside a selector list

```scss
.container,
// 100% wide container at all breakpoints
.container-fluid {
  color: red;
}
```

```expect
(none)
```
