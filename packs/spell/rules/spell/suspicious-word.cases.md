# Cases: spell/suspicious-word

### case: a known typo, and a word one edit from a common one

```toml path="codelaw.toml"
[spell]
words = ["recieve", "lenth", "kubectl"]
```

```expect
2:11 spell/suspicious-word
2:22 spell/suspicious-word
```

### case: only the root config's [spell] words

```toml path="codelaw.toml"
[other]
words = ["recieve"]
```

```expect
(none)
```
