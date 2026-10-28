# Cases: spell/locale-variant

### case: the other locale's spelling, fixed in its case

```md options={"locale":"en-US"}
Colour and behaviour, then the color.
```

```expect
1:1 spell/locale-variant
1:12 spell/locale-variant
```

```fixed
Color and behavior, then the color.
```

### case: under en-GB, the American spelling; -ize is never reported

```md options={"locale":"en-GB"}
The color centre and organize.
```

```expect
1:5 spell/locale-variant
```

```fixed
The colour centre and organize.
```

### case: pairs right in both locales are never reported

```md options={"locale":"en-US"}
A cheque, a programme and a licence.
```

```expect
(none)
```

### case: no locale, no finding

```md
Colour and color.
```

```expect
(none)
```
