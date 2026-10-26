# sec-html/no-srcdoc-script — cases

### case: an event handler inside srcdoc

```html
<iframe srcdoc="&lt;p onclick=x()&gt;a&lt;/p&gt;"></iframe>
```

```expect
1:9 sec-html/no-srcdoc-script
```

### case: plain markup

```html
<iframe srcdoc="&lt;p&gt;hello&lt;/p&gt;"></iframe>
```

```expect
(none)
```
