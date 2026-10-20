---
name: codelaw-suppress
description: Suppress a codelaw finding that the user has decided is intended, with the narrowest scope and a recorded reason. Use when the user says "this violation is intended", "ignore this rule here", "silence codelaw on this line", "this is a false positive but leave the code", or "turn this rule off for these files". Picks an inline codelaw-ignore with a reason first, a per-glob override only when many files share the reason, and refuses a blanket disable.
---

# codelaw-suppress — narrow, explicit, with a reason

A suppression is a decision the user makes about specific code. You make it
precise and leave a reason behind; you never use it to make your own work
pass.

## 0. Is this a suppression at all?

- **The user decided.** Only suppress when the user says the flagged code is
  intended. If you are tempted to suppress something you wrote, use
  `codelaw-fix` instead.
- **A wrong rule is not a suppression.** If the rule is wrong for this kind of
  code everywhere, that is `codelaw-debug-rule`.
- **Existing violations across a codebase** are frozen with a baseline, not a
  wall of inline ignores.

Get the exact rule id and location: `check_paths` `{"paths": ["<file>"]}`.
Ask for the reason if the user has not given one; "intended" is not a reason —
"the API requires a loose comparison against null" is.

## 1. Inline, with a reason (the default)

In the file's own comment syntax, on its own line directly above the code:

- JS/TS/CSS: `// codelaw-ignore <rule-id>: <reason>` (CSS: `/* codelaw-ignore <rule-id>: <reason> */`)
- Python, YAML, TOML: `# codelaw-ignore <rule-id>: <reason>`
- HTML, Markdown: `<!-- codelaw-ignore <rule-id>: <reason> -->`

A comment alone on its line covers the next line with code; a trailing comment
covers its own line. Several rules go comma-separated before the colon. The
reason is required — codelaw reports a suppression without one as an error,
and one that suppresses nothing as `info`.

Re-run `check_paths` on the file: the finding is gone and no
`codelaw/…suppression…` finding appeared.

## 2. A per-glob override (many files, one reason)

Only when the same reason applies to a whole set of files (generated code,
fixtures, a vendored folder), add to `codelaw.toml`:

```toml
# <rule-id> is off for <files> because <reason> — <who decided>, <date>
[[overrides]]
files = ["<the narrowest glob that covers them>"]
rules = { "<rule-id>" = "off" }
```

Use the narrowest glob; list the files it matches back to the user. Then
re-run `check_paths` on one of them.

## Refuse

- **A blanket disable** — setting the rule `"off"` in `[rules]` for the whole
  repo, or a glob like `**` — is not a suppression. If the user wants the rule
  gone everywhere, say that this is a config decision for them to make and
  show the one line; do not add it as part of a suppression.
- **A suppression without a reason.**
- **Suppressing a rule id that is not reported there** (codelaw will flag it).
