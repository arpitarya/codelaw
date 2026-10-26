# sec-py/no-assert-for-auth — cases

### case: a permission

```py
assert user.has_perm("x")
```

```expect
1:1 sec-py/no-assert-for-auth
```

### case: an invariant

```py
assert len(items) > 0
```

```expect
(none)
```

### case: current_user

```py
assert current_user.id == owner_id
```

```expect
1:1 sec-py/no-assert-for-auth
```
