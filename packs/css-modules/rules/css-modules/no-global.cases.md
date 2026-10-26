# Cases: css-modules/no-global

### case: a stylesheet that is not a module may use :global

```css
:global(.x) { color: red; }
```

```expect
(none)
```
