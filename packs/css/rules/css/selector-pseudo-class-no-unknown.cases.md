# Cases: css/selector-pseudo-class-no-unknown

### case: scrollbar states after a vendor pseudo-element

```css
::-webkit-scrollbar-thumb:horizontal {
  color: red;
}
```

```expect
(none)
```

### case: page pseudo-classes in @page

```css
@page :first {
  margin: 1in;
}
```

```expect
(none)
```

### case: a pseudo-element with one colon

```css
::selection {
  color: red;
}

:selection {
  color: red;
}
```

```expect
5:1 css/selector-pseudo-class-no-unknown
```

### case: vendor-prefixed pseudo-classes are skipped

```css
a:-moz-focusring {
  color: red;
}
```

```expect
(none)
```
