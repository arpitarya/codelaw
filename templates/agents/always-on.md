## codelaw

This repo is linted and formatted by codelaw. Its rules are Markdown files, every
finding carries its fix, and the verdict is always the engine's, never yours.

- **Ask before you write.** Call the MCP tool `rules_for_path` (or run
  `{{cmd}} rules --for <path>`) for each file you create or edit, and write code that complies.
- **Check what you changed.** Run `{{cmd}} check <files>` on every file you created or edited.
- **Handle each finding by its fix class.** `auto`: `{{cmd}} fix <files>`. `suggest`: show the
  diff and apply it only once accepted. `manual`: follow the steps `{{cmd}} explain <rule>` prints.
- **Never** disable, loosen or suppress a rule to make a check pass, and never edit a file
  codelaw generated: change the rule or the skill and run `{{cmd}} sync-agents`.

Active presets: {{presets}}.

Skills: `codelaw-write` before writing, `codelaw-fix` for findings, `codelaw-explain` for why
a rule fired, `codelaw-author-rule` for a new rule, `codelaw-suppress` when a finding is intended.
