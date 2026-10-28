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
| `spell/strict` | 3 | `spell/recommended`; the dictionary check joins it as it is built (W-103) |

<!-- /docs-sync:presets -->

## Rules

<!-- docs-sync:rules -->

| rule | fix | flags |
|---|---|---|
| `spell/known-typo` | auto | a word on the known-typo list in a comment, a doc, a prose string or Markdown |
| `spell/known-typo-identifier` | auto | a known typo in a JS, TS or Python name where it is declared; renamed only when the name is provably file-local |
| `spell/known-typo-key` | manual | a known typo in a JSON, YAML or TOML key, a CSS or SCSS class, id or custom property, or an HTML `id` |

<!-- /docs-sync:rules -->

## The word data

`data/spell/typos.json` is written by `scripts/spell-data.py` and committed;
the same sources write the same bytes. Sources and their notices: [`NOTICE`](NOTICE).
