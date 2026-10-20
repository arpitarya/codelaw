---
name: codelaw-explain
description: Explain a codelaw rule or finding — what it requires, why, how to fix it, and why it is active here. Use when the user asks "why did this fire", "what does rule X want", "is this allowed", "why is codelaw complaining about this line", "where does this rule come from", or names a rule id such as js/eqeqeq. Answers from codelaw's own rule text and configuration, never from general knowledge of other linters. Writes no files.
---

# codelaw-explain — what a rule wants, and why it is on

You explain; codelaw is the source. Everything you say about a rule comes from
codelaw's answers below, quoted or closely paraphrased — not from what a
similarly named ESLint or Ruff rule does.

## 1. Find the rule id

- The user named it (`js/eqeqeq`), or
- it is in a finding they pasted (`file:line:col rule-id message`), or
- they point at code: run `check_paths` `{"paths": ["<file>"]}` (CLI:
  `codelaw check <file> --format json`) and take the rule ids on that line.

If the id is misspelt, the error names the nearest one ("did you mean …?") —
use it, and tell the user.

## 2. What it requires

Call `explain_rule` `{"id": "<rule>"}` (CLI: `codelaw explain <id> --format json`).
From the result, give the user:

- the **title** and **summary** — what the rule requires;
- the **Why** — the reason, in the rule's words;
- one **Don't** and one **Do** example for the language they are working in
  (`examples[]`, by `language`);
- the **fix class**, and for `manual` the numbered **steps** and the skill in
  `fix.skill`; for `suggest` or `manual`, the `fix.reason` it isn't automatic.

## 3. Why it is active here

Call `trace_rule` `{"id": "<rule>", "path": "<file>"}` (CLI:
`codelaw rules --why <id> --for <file> --format json`). Report:

- `source` — the pack and version, or `local` for `.codelaw/rules/`;
- `enabled_by` — the preset in `extends`, or the rules folder, that switched
  it on;
- `for.set_by` — each config file that sets its severity for that file;
- `for.applies` and `for.severity` — whether it runs on that file, and how
  severely;
- `for.diagnostics` — what it reports there right now;
- `shadows` — a local rule overriding a pack rule of the same id.

## 4. "Is this allowed?"

Do not answer from your reading of the rule. Put the exact code in a file
path and ask codelaw: `check_snippet` `{"path": "<file>", "code": "<text>"}`.
Report what it returns. If nothing fires, say "codelaw reports nothing for
this code under the current config" — not "this is fine".

## 5. Next steps to offer

- To fix it: the `codelaw-fix` skill.
- If the code is intended: the `codelaw-suppress` skill (a suppression always
  carries a reason).
- If the rule is wrong for this case (false positive or miss): the
  `codelaw-debug-rule` skill.

## Never

- **Never explain a rule from another tool's documentation** in place of
  codelaw's rule text.
- **Never suggest switching the rule off** as the way to make a finding go
  away; that is the user's decision, through config, and only if they ask.
- **Never assert a verdict** ("this code is fine") that codelaw did not give.
