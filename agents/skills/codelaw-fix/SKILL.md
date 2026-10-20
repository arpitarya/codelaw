---
name: codelaw-fix
description: Fix codelaw findings until the check is clean. Use when the user says "fix lint errors", "fix the codelaw errors", "codelaw is failing", "make the check pass", or when a codelaw hook, a CI job or a codelaw report has listed violations to resolve. Applies auto fixes, shows suggest fixes as diffs and applies them only once accepted, works through manual fixes step by step, and re-checks until clean. This is also the skill that carries every manual fix a rule does not assign to a more specific skill.
---

# codelaw-fix — from findings to a clean check

Every codelaw finding says how to fix it, in one of three classes: `auto`,
`suggest` or `manual`. Your job is to apply each class correctly and let
codelaw — not you — decide when the code is clean.

## 1. Get the findings

Scope the work to what was asked:

- the files a hook or the user named, or
- the files changed on this branch, or
- the whole project when the user asks for that.

Run, with those paths:

- **MCP:** `check_paths` `{"paths": [...]}` (empty `paths` = the project).
- **CLI:** `codelaw check <paths> --format json`.

If a report was pasted (a hook message, a CI log, a SARIF file), still re-run
the check locally: the report may be stale, and the local JSON carries the
edits and steps you need. For a CI failure, the `codelaw-ci-triage` skill
reproduces it with the same flags first.

Group the diagnostics by `fix.class`.

## 2. `auto` — apply them all

`fix_paths` `{"paths": [...]}` (CLI: `codelaw fix <paths>`). codelaw applies
every `auto` fix, re-parses, verifies the file still parses, and loops to a
fixpoint; it never leaves a file broken. Read the `fixed[]` list it returns so
you can tell the user what changed. Do not re-do these edits by hand.

## 3. `suggest` — show, ask, then apply

A suggestion is mechanical but may change behaviour (for example `==` to
`===`). For each one:

1. Show the user the change as a diff: the file, the line, the old text and
   the replacement from `fix.edits` (byte offsets into the current file).
   Say which rule asks for it and why (`explain_rule` if needed).
2. Apply only what the user accepts.
   - All suggestions in a set of files accepted: `fix_paths`
     `{"paths": [...], "unsafe": true}` (CLI: `codelaw fix --unsafe <paths>`).
   - Only some accepted: apply those edits yourself, exactly as given.
3. A rejected suggestion stays a finding. Tell the user; if they say the
   current code is intended, switch to `codelaw-suppress`.

To preview without writing, add `"dry_run": true` (CLI: `--dry-run`).

## 4. `manual` — follow the steps exactly

Each manual diagnostic has `fix.steps`, numbered. For each one:

1. Call `explain_rule` `{"id": "<rule>"}` (CLI: `codelaw explain <id>`) and
   read the summary, the **Why**, the steps and the Do/Don't examples.
2. If `explain_rule`'s `fix.skill` names a skill other than `codelaw-fix`,
   follow that skill for this finding.
3. Do the steps in order, changing only what they require. When a step needs a
   decision only the user can make (a name, an API choice), ask.
4. Re-check that file before moving to the next finding.

## 5. Re-check until clean

Run the same check as in step 1. New findings can appear after a fix (a
moved block exposes another rule); handle them the same way. Stop when:

- no `error` or `warn` remains in scope, or
- what remains is only rejected suggestions or findings the user chose to
  keep — list them, with rule ids, in your summary.

If the same finding survives two honest attempts, stop and report it with the
rule id and what you tried; a rule that seems wrong is a job for
`codelaw-debug-rule`, not for a workaround.

## Never

- **Never disable, loosen, eject or delete a rule**, and never change
  `codelaw.toml` severities to make the check pass.
- **Never add a `codelaw-ignore` comment to clear a finding** unless the user
  said the code is intended — then use `codelaw-suppress`, with a reason.
- **Never apply a `suggest` fix the user has not accepted.**
- **Never declare the code clean yourself.** Only a clean codelaw run does.
- **Never edit generated agent files**; they come from `codelaw sync-agents`.
