
## Conventions

Every change follows these. Review rejects a change that breaks one.

### Client (TypeScript)

1. Every request goes through `apiFetch` from `web/src/lib/http.ts`, never `fetch`
   directly: it adds the base URL, the auth header and the timeout.
2. No `console.*` calls in committed code.
3. Style a component with `className` and the stylesheet, never an inline `style` prop.
4. Use `===` and `!==`; check NaN with `Number.isNaN`, never `x === NaN`.
5. Use `Object.hasOwn(obj, key)`, never `obj.hasOwnProperty(key)`.
6. No empty blocks: a `catch` that ignores an error says so in a comment.
7. No unused variables, parameters or imports.
8. A `switch` case never falls through into the next one.
9. Never pass an `async` function to `new Promise`.
10. Prefer `const`; never reassign a parameter.
11. Named exports only; no default exports.
12. Types for every exported function's parameters and return value.
13. No `any`; use `unknown` and narrow it.
14. No `debugger` statements.
15. No non-null assertions (`!`) on values from the network.

### API (Python)

16. Outbound HTTP goes through `acme.http` (`http_get`, `http_post`), never
    `requests.get` or `requests.post` directly: it adds the timeout and the retries.
17. Read the current time only through `acme.clock.now()`, never `datetime.now()`,
    `datetime.utcnow()` or `datetime.today()`.
18. SQL is always parameterised: a constant string with `?` placeholders and a
    tuple of values, never an f-string, `%` or `.format()`.
19. Compare to `None` with `is` / `is not`.
20. Never a bare `except:`; catch the exceptions you expect.
21. Inside an `except`, raise a new exception `from` the one caught.
22. No mutable default arguments (`[]`, `{}`); default to `None`.
23. No function calls in default arguments.
24. Do not assign a `lambda` to a name; write a `def`.
25. Imports at the top of the module, sorted: standard library, third party, `acme`.
26. No `import *`.
27. No unused imports or variables.
28. f-strings only when they have a placeholder.
29. Do not use `assert` for runtime checks.
30. Use `pathlib` for paths, not string concatenation.
31. Money is in integer cents, never a float.
32. Log through `logging.getLogger(__name__)`, never `print`.
