# data/codeowners-coverage — cases

### case: a file no pattern owns is reported on the root manifest

```json
// package.json
{ "name": "app" }
// src/a.js
// docs/b.md
// .github/CODEOWNERS
/docs/ @org/docs
/.github/ @org/docs
```

```expect
package.json:1:1 data/codeowners-coverage
```

### case: a catch-all owns everything

```json
// package.json
{ "name": "app" }
// src/a.js
// CODEOWNERS
* @org/all
```

```expect
```

### case: a stale pattern is reported on the manifest and on its own line

```json
// package.json
{ "name": "app" }
// CODEOWNERS
* @org/all
/gone/ @org/old
```

```expect
CODEOWNERS:2:1 data/codeowners-coverage
package.json:1:1 data/codeowners-coverage
```

### case: a pattern with no owner is reported

```json
// package.json
{ "name": "app" }
// src/a.js
// CODEOWNERS
* @org/all
/src/
```

```expect
CODEOWNERS:2:1 data/codeowners-coverage
package.json:1:1 data/codeowners-coverage
```

### case: GitLab sections own a file when any section does

```json
// package.json
{ "name": "app" }
// docs/a.md
// CODEOWNERS
[Docs] @org/docs
/docs/
[Root] @org/root
/package.json
/CODEOWNERS
```

```expect
```

### case: no CODEOWNERS file, no finding

```json
// package.json
{ "name": "app" }
// src/a.js
```

```expect
```

### case: only the root manifest carries the project's findings

```json
// package.json
{ "name": "app" }
// web/package.json
{ "name": "web" }
// CODEOWNERS
/web/ @org/web
```

```expect
package.json:1:1 data/codeowners-coverage
```
