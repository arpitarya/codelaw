# sec-py/no-xpath-injection — cases

### case: an f-string

```py
tree.xpath(f"//user[@name='{name}']")
```

```expect
1:1 sec-py/no-xpath-injection
```

### case: variables

```py
tree.xpath("//user[@name=$n]", n=name)
```

```expect
(none)
```

### case: a concatenation

```py
root.findall("./user[@id='" + uid + "']")
```

```expect
1:1 sec-py/no-xpath-injection
```

### case: a plain path

```py
root.findall("./user")
```

```expect
(none)
```
