---
id: sec-js/no-child-process-shell
title: "No shell command built from a value"
languages: [javascript, jsx, typescript, tsx]
fix: manual
fix_reason: "the program and its arguments have to be separated by hand"
stability: preview
ported_from: [eslint-plugin-security/detect-child-process, bandit/B602, ruff/S602]
security: { cwe: [78], owasp: ["A05:2025"], severity: high }
since: 0.3.0
---

`exec` and `execSync` (from `child_process`) are called with a literal command only, and `spawn`-style calls do not set `shell: true`.

**Why:** A command string is parsed by a shell, so a value spliced into it can add commands with `;`, `&&` or `$(…)` (CWE-78).

```codelaw
match:
  any:
    - pattern: exec($CMD)
    - pattern: exec($CMD, $$$REST)
    - pattern: execSync($CMD)
    - pattern: execSync($CMD, $$$REST)
    - pattern: child_process.exec($CMD)
    - pattern: child_process.exec($CMD, $$$REST)
    - pattern: child_process.execSync($CMD)
    - pattern: child_process.execSync($CMD, $$$REST)
    - pattern: childProcess.exec($CMD)
    - pattern: childProcess.exec($CMD, $$$REST)
    - pattern: childProcess.execSync($CMD)
    - pattern: childProcess.execSync($CMD, $$$REST)
    - pattern: cp.exec($CMD)
    - pattern: cp.exec($CMD, $$$REST)
    - pattern: cp.execSync($CMD)
    - pattern: cp.execSync($CMD, $$$REST)
    - pattern: child.exec($CMD)
    - pattern: child.exec($CMD, $$$REST)
    - pattern: child.execSync($CMD)
    - pattern: child.execSync($CMD, $$$REST)
    - all:
        - any:
            - pattern: spawn($$$ARGS)
            - pattern: spawnSync($$$ARGS)
            - pattern: execFile($$$ARGS)
            - pattern: execFileSync($$$ARGS)
            - pattern: child_process.spawn($$$ARGS)
            - pattern: child_process.spawnSync($$$ARGS)
            - pattern: child_process.execFile($$$ARGS)
            - pattern: child_process.execFileSync($$$ARGS)
            - pattern: childProcess.spawn($$$ARGS)
            - pattern: childProcess.spawnSync($$$ARGS)
            - pattern: childProcess.execFile($$$ARGS)
            - pattern: childProcess.execFileSync($$$ARGS)
            - pattern: cp.spawn($$$ARGS)
            - pattern: cp.spawnSync($$$ARGS)
            - pattern: cp.execFile($$$ARGS)
            - pattern: cp.execFileSync($$$ARGS)
            - pattern: child.spawn($$$ARGS)
            - pattern: child.spawnSync($$$ARGS)
            - pattern: child.execFile($$$ARGS)
            - pattern: child.execFileSync($$$ARGS)
        - has:
            kind: pair
            stopBy: end
            regex: '^["'']?shell["'']?\s*:\s*true$'
constraints:
  CMD: { not: { any: [ { kind: string }, { regex: '^`[^`$]*`$' } ] } }
message: "A shell command is built from a value here; a value in it can add commands"
```

## How to fix

1. Call `execFile` or `spawn` with the program name first and the values as an array: no shell parses them.
2. Leave `shell` at its default (`false`); if a shell feature (a pipe, a glob) is needed, do it in the program, not in a command string.
3. A value that must reach a command is checked against an allow-list first.

## Examples

### JavaScript

#### Don't

```js
const { exec } = require("child_process");
exec("convert " + name + " out.png");
```

#### Do

```js
const { execFile } = require("child_process");
execFile("convert", [name, "out.png"]);
```

### JSX

#### Don't

```jsx
const { exec } = require("child_process");
exec("convert " + name + " out.png");
```

#### Do

```jsx
const { execFile } = require("child_process");
execFile("convert", [name, "out.png"]);
```

### TypeScript

#### Don't

```ts
const { exec } = require("child_process");
exec("convert " + name + " out.png");
```

#### Do

```ts
const { execFile } = require("child_process");
execFile("convert", [name, "out.png"]);
```

### TSX

#### Don't

```tsx
const { exec } = require("child_process");
exec("convert " + name + " out.png");
```

#### Do

```tsx
const { execFile } = require("child_process");
execFile("convert", [name, "out.png"]);
```
