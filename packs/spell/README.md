# @codelaw/rules-spell

codelaw's spell checker — one of the first-party rule packs
([SR-RULE-PACKS](../../records/0101_rule-packs.md)), on npm and PyPI
(`codelaw-rules-spell`). It reads the prose of every L3 language from the tree
the engine parsed: comments and docstrings, prose strings, Markdown and HTML
text, the string values of JSON, YAML and TOML, and the names a file
declares — identifiers, keys, classes and ids. It is codelaw's own
code and codelaw's own word data, generated from SCOWL
([SR-SPELL](../../records/0116_spell.md)); it does not wrap CSpell.

`codelaw init` never adds it: a team installs it on purpose.

```toml
# codelaw.toml
extends = ["codelaw:spell/recommended"]
preview = true   # every rule is `preview` until a corpus run promotes it
```

```sh
npm i -D @codelaw/rules-spell      # or
pip install codelaw-rules-spell
```

## Presets

<!-- docs-sync:presets -->

| preset | rules | ported from |
|---|---|---|
| `spell/recommended` | 3 | codelaw's own: known typos from a list generated from SCOWL, fixed unattended |
| `spell/strict` | 5 | `spell/recommended` plus the dictionary check and the `[spell] words` check |

<!-- /docs-sync:presets -->

## Rules

<!-- docs-sync:rules -->

| rule | fix | flags |
|---|---|---|
| `spell/known-typo` | auto | a word on the known-typo list in a comment, a doc, a prose string or Markdown |
| `spell/known-typo-identifier` | auto | a known typo in a JS, TS or Python name where it is declared; renamed only when the name is provably file-local |
| `spell/known-typo-key` | manual | a known typo in a JSON, YAML or TOML key, a CSS or SCSS class, id or custom property, or an HTML `id` |
| `spell/unknown-word` | suggest | a word of prose in no word list; the nearest dictionary word as the edit, up to three named (`spell/strict`) |
| `spell/locale-variant` | auto | the other locale's spelling (`colour` under `en-US`), while `[spell] locale` is set (in no preset) |
| `spell/suspicious-word` | manual | a `[spell] words` entry that is a known typo or one edit from a common word (`spell/strict`) |

<!-- /docs-sync:rules -->

## Config

```toml
# codelaw.toml (the root config only)
[spell]
locale = "en-US"            # or "en-GB"; spell/locale-variant reports the other one
words = ["kubectl", "Acme"] # accepted by every spell/* rule; spell/suspicious-word checks them

[rules]
"spell/locale-variant" = "warn"
```

## The word data

`data/spell/` is written by `scripts/spell-data.py` and committed; the same
sources write the same bytes: `typos.json` (188,138 generated known typos),
`words.json` (92,758 words: SCOWL's US and British English to size 60, and the
software terms) and `locale.json` (VarCon's one-to-one US and British pairs). Sources and their notices: [`NOTICE`](NOTICE).
