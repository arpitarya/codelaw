# sec-js/no-mass-assignment — cases

### case: Object.assign

```js
Object.assign(user, req.body);
```

```expect
1:1 sec-js/no-mass-assignment
```

### case: findByIdAndUpdate

```js
await User.findByIdAndUpdate(id, req.body);
```

```expect
1:7 sec-js/no-mass-assignment
```

### case: a property of the body is a value, not the object

```js
new Date(req.body.when);
```

```expect
(none)
```

### case: new

```js
const u = new User(req.body);
```

```expect
1:11 sec-js/no-mass-assignment
```

### case: build

```js
const u = User.build(req.body);
```

```expect
1:11 sec-js/no-mass-assignment
```
