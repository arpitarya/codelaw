---
id: spell/suspicious-word
title: "No accepted word hides a typo"
languages: [toml]
fix: manual
fix_reason: "only the team knows whether an accepted word is a real name or a misspelling it waved through"
stability: preview
since: 0.3.0
---

An entry of `[spell] words` in the root `codelaw.toml` is itself on the known-typo list (`recieve`), or is no dictionary word yet one edit from a common one (`lenth`, one letter from `length`). Every `spell/*` rule accepts the words in that list, so a misspelling there is accepted everywhere it appears.

**Why:** a word list grows by people pasting in whatever the checker flagged; one typo in it silences the check for the whole project.

```codelaw
native: spell/suspicious-word
```

## How to fix

1. Search the project for the word (`git grep -n -i recieve`).
2. If it is a misspelling, fix it where it appears and take it out of `[spell] words`.
3. If it is right (a name that looks like a typo), keep it and excuse the line: `# codelaw-ignore spell/suspicious-word: <reason>`.

## Examples

### TOML

#### Don't

```toml path="codelaw.toml"
[spell]
words = ["recieve", "kubectl"]
```

#### Do

```toml path="codelaw.toml"
[spell]
words = ["kubectl"]
```
