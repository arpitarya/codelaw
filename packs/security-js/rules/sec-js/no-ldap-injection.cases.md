# sec-js/no-ldap-injection — cases

### case: a concatenated filter

```js
client.search(base, { filter: "(uid=" + name + ")" }, cb);
```

```expect
1:23 sec-js/no-ldap-injection
```

### case: a literal filter

```js
client.search(base, { filter: "(objectClass=*)" }, cb);
```

```expect
(none)
```
