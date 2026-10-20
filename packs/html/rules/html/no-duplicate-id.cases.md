# html/no-duplicate-id — cases

### case: ids in separate templates do not clash

```html
<template><p id="a"></p></template>
<template><p id="a"></p></template>
```

```expect
(none)
```

### case: template code is not compared

```html
<p id="{{ id }}"></p>
<p id="{{ id }}"></p>
```

```expect
(none)
```
