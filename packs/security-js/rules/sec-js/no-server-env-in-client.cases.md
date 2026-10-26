# sec-js/no-server-env-in-client — cases

### case: a public name that looks secret

```js
const k = process.env.NEXT_PUBLIC_API_SECRET;
```

```expect
1:11 sec-js/no-server-env-in-client
```

### case: a server file may read any variable

```js
const k = process.env.STRIPE_SECRET_KEY;
```

```expect
(none)
```

### case: a vite secret

```js
const k = import.meta.env.VITE_SECRET_KEY;
```

```expect
1:11 sec-js/no-server-env-in-client
```

### case: a client file reading NODE_ENV

```js
"use client";
const e = process.env.NODE_ENV;
```

```expect
(none)
```
