---
name: codelaw-debug-rule
description: Debug a codelaw rule that fires wrongly, misses a case, or fails its own tests. Use when the user says "false positive", "this rule is wrong here", "the rule didn't catch this", "codelaw missed …", "codelaw test is failing for my rule", or a rule's fix produces the wrong code. Reproduces the case as a failing test first, inspects the syntax tree, adjusts the matcher, and keeps every existing example and case green. Edits rules in .codelaw/rules/ only.
---

# codelaw-debug-rule — the case first, then the rule

**A bug becomes a case before the rule changes.** That order is the whole
discipline: the failing case proves the bug, and every existing example and
case proves you did not break something else.

## 1. Identify the rule and where it lives

From the finding or the user's description, get the rule id. Then
`trace_rule` `{"id": "<rule>", "path": "<file>"}` (CLI:
`codelaw rules --why <id> --for <file>`):

- `source: local` — the rule is in `.codelaw/rules/`; you can fix it here.
- a pack (`<name>@<version>`) — you cannot edit the pack. Either
  `codelaw rules eject <id>` copies it into `.codelaw/rules/` (the local copy
  then shadows the pack rule — tell the user this means they own it from now
  on), or report the case upstream to the pack's maintainers. Ask which.
- Also check `for.applies` and `for.severity`: a "miss" is sometimes the rule
  not applying to that path (`files:` / `exclude:`, or the language).

## 2. Reproduce as a failing case

In `<name>.cases.md` next to the rule, add:

````markdown
### case: <short description of the bug>

```<lang>
<the smallest code that shows it>
```

```expect
(none)
```
````

For a missed violation, `expect` lists `line:col <rule-id>` instead of
`(none)`; for a wrong fix, add a `fixed` fence with the correct output.

Run `codelaw test <id> --case "<short description>"`. It **must fail**. If it
passes, you have not reproduced the bug — shrink or correct the case until it
fails.

## 3. Inspect

- `dump_tree` `{"code": "<case code>", "lang": "<lang>"}` (CLI:
  `codelaw tree --lang <l> -e "<code>"`) — the node kinds and fields the
  matcher sees.
- If your codelaw has it, `codelaw match <id> <file> --explain` shows which
  clause matched or failed, and `codelaw fix --dry-run --trace <file>` what
  the fix did. If the command is unknown in this version, rely on the tree.

## 4. Change the matcher, minimally

Adjust `match:` (`kind`, `has`/`inside`/`not`, `constraints:`, `where:`
predicates) or the `fix:`, as little as needed to make the new case pass.

## 5. Everything stays green

`codelaw test <id>` — every example, every case, the new one included, and
the invariants (still parses, fix is a fixpoint, deterministic). Then
`codelaw rules lint`. If an old example now fails, your change is too broad;
go back to step 4. Never edit an old example or case to make it pass unless
the user agrees the old expectation was wrong — and then say so in your
summary.

Finally `codelaw check` the repo and compare the rule's finding count before
and after; explain any change.

## Never

- **Never change the rule before you have a failing case.**
- **Never "fix" a false positive by switching the rule off, narrowing
  `files:` to dodge the file, or adding a suppression** — unless the user,
  having seen the case, decides that is the right fix.
- **Never edit a pack's files in `node_modules/` or site-packages.**
