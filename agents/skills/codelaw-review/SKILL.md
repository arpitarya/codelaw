---
name: codelaw-review
description: Review a diff, branch or pull request against this repo's codelaw rules. Use when the user says "review this diff against our rules", "lint review this PR", "check my changes with codelaw", or "what does codelaw say about this branch". Runs codelaw check on the changed files only and writes review comments that quote each violation's fix (the edit or the manual steps). Adds no verdicts of its own and writes no files.
---

# codelaw-review — the rules' review, not yours

This review contains **only what codelaw reports**. It is not a general code
review; if the user also wants your opinion on design or logic, give that
separately and label it as yours.

## 1. The changed files

Work out what changed against the base the user means (default: the merge
base with the default branch):

`git diff --name-only <base>...HEAD`

Keep only files that still exist. Note the changed line ranges per file
(`git diff -U0 <base>...HEAD`) so you can tell findings on changed lines from
pre-existing ones.

## 2. Check them

`check_paths` `{"paths": [<changed files>]}` (CLI:
`codelaw check <files> --format json`). If the repo uses a baseline, findings
it covers are already filtered out; that is intended.

## 3. Write the comments

One comment per diagnostic **on a changed line**, in this form:

> **`<rule-id>`** (`<severity>`, fix: `<class>`) — `<message>`
>
> - `auto`: "`codelaw fix <file>` applies this." and the replacement text from
>   `fix.edits`.
> - `suggest`: the edit as a diff (old text → new text), and "apply with
>   `codelaw fix --unsafe <file>` once accepted; it may change behaviour
>   because: `<fix.reason>`" (`explain_rule` gives the reason).
> - `manual`: the numbered `fix.steps`, verbatim.

Use the file and `range.start.line` for the comment position. Then a summary:
counts by severity and fix class, and how many `auto` fixes one
`codelaw fix` would clear.

Findings on lines the diff did not touch go in a short separate list
("pre-existing, not introduced by this change"), not as review comments.

## 4. Posting

Post comments only where the user asked (a PR, a file, the chat). Through a
code host, use its review API with the positions above.

## Never

- **Never add a finding codelaw did not report**, and never drop one it did
  on a changed line.
- **Never soften or reword a rule's steps**; quote them.
- **Never approve or reject the PR on codelaw's behalf**; report, and let the
  user decide.
