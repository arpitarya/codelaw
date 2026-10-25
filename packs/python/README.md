# codelaw-rules-python

codelaw's rules for Python — one of the first-party rule packs
([SR-RULE-PACKS](../../records/0101_rule-packs.md)). It ships on
PyPI only, as `codelaw-rules-python`; npm has no copy (L6 decision 11).

```toml
# codelaw.toml
extends = ["codelaw:py/recommended"]
preview = true   # every rule is `preview` until a corpus run promotes it
```

## Presets

<!-- docs-sync:presets -->

| preset | rules | ported from |
|---|---|---|
| `py/recommended` | 96 | Ruff 0.17.1: its long-standing default `E4`, `E7`, `E9`, `F` (55 of 59 rules; the other four are below), all 39 stable flake8-bugbear `B` rules, isort's `I001` and `RUF022` |
| `py/conventions` | 2 | opt-in house conventions from the starter catalogue: `py/no-print`, `py/no-tunable-literals` |
| `py/project` | 1 | `graph/no-unused-exports`, the rule that reads the whole repo, from vulture's unused-function check — opt in |

<!-- /docs-sync:presets -->

The starter catalogue's other Python rules match a Ruff rule, so they sit in
`py/recommended` under their starter ids: `py/no-bare-except` (`E722`),
`py/no-mutable-default` (`B006`) and `py/all-sorted` (`RUF022`).

Every rule is **reimplemented**, not vendored: the matchers, messages and docs
are written fresh from Ruff's documented behaviour, and each rule's
`ported_from:` names the source rule. 97 rules are `native:` (Rust, in
`crates/codelaw-native/src/python_*.rs`); `py/no-tunable-literals` is a
`match:` rule.

## Ported

| Ruff | rule | fix |
|---|---|---|
| `E401` | `py/multiple-imports-on-one-line` | auto |
| `E402` | `py/module-import-not-at-top` | manual |
| `E711` | `py/none-comparison` | suggest |
| `E712` | `py/true-false-comparison` | suggest |
| `E713` | `py/not-in-test` | auto |
| `E714` | `py/not-is-test` | auto |
| `E721` | `py/type-comparison` | manual |
| `E722` | `py/no-bare-except` | suggest |
| `E731` | `py/lambda-assignment` | suggest |
| `E741` | `py/ambiguous-variable-name` | manual |
| `E742` | `py/ambiguous-class-name` | manual |
| `E743` | `py/ambiguous-function-name` | manual |
| `F401` | `py/unused-import` | suggest |
| `F402` | `py/import-shadowed-by-loop-var` | manual |
| `F403` | `py/no-star-import` | manual |
| `F404` | `py/late-future-import` | manual |
| `F405` | `py/star-import-usage` | manual |
| `F406` | `py/nested-star-import` | manual |
| `F407` | `py/undefined-future-feature` | manual |
| `F501` | `py/percent-format-invalid` | manual |
| `F502` | `py/percent-format-expected-mapping` | manual |
| `F503` | `py/percent-format-expected-sequence` | manual |
| `F504` | `py/percent-format-unused-named-argument` | auto |
| `F505` | `py/percent-format-missing-argument` | manual |
| `F506` | `py/percent-format-mixed-placeholders` | manual |
| `F507` | `py/percent-format-count-mismatch` | manual |
| `F508` | `py/percent-format-star-requires-sequence` | manual |
| `F509` | `py/percent-format-unsupported-character` | manual |
| `F521` | `py/format-invalid` | manual |
| `F522` | `py/format-unused-named-argument` | auto |
| `F523` | `py/format-unused-positional-argument` | auto |
| `F524` | `py/format-missing-argument` | manual |
| `F525` | `py/format-mixed-numbering` | manual |
| `F541` | `py/f-string-missing-placeholders` | auto |
| `F601` | `py/duplicate-dict-key` | manual |
| `F602` | `py/duplicate-dict-key-variable` | manual |
| `F621` | `py/too-many-expressions-in-star-assignment` | manual |
| `F622` | `py/multiple-starred-expressions` | manual |
| `F631` | `py/assert-tuple` | manual |
| `F632` | `py/is-literal` | suggest |
| `F633` | `py/invalid-print-syntax` | manual |
| `F634` | `py/if-tuple` | manual |
| `F701` | `py/break-outside-loop` | manual |
| `F702` | `py/continue-outside-loop` | manual |
| `F704` | `py/yield-outside-function` | manual |
| `F706` | `py/return-outside-function` | manual |
| `F707` | `py/default-except-not-last` | manual |
| `F722` | `py/forward-annotation-syntax-error` | manual |
| `F811` | `py/redefined-while-unused` | manual |
| `F821` | `py/undefined-name` | manual |
| `F822` | `py/undefined-export` | manual |
| `F823` | `py/undefined-local` | manual |
| `F841` | `py/unused-variable` | suggest |
| `F842` | `py/unused-annotation` | auto |
| `F901` | `py/raise-not-implemented` | suggest |
| `B002` | `py/unary-prefix-increment` | manual |
| `B003` | `py/assignment-to-os-environ` | manual |
| `B004` | `py/unreliable-callable-check` | suggest |
| `B005` | `py/strip-with-multi-characters` | manual |
| `B006` | `py/no-mutable-default` | suggest |
| `B007` | `py/unused-loop-control-variable` | suggest |
| `B008` | `py/function-call-in-default-argument` | manual |
| `B009` | `py/getattr-with-constant` | auto |
| `B010` | `py/setattr-with-constant` | auto |
| `B011` | `py/assert-false` | suggest |
| `B012` | `py/jump-statement-in-finally` | manual |
| `B013` | `py/redundant-tuple-in-exception-handler` | auto |
| `B014` | `py/duplicate-handler-exception` | auto |
| `B015` | `py/useless-comparison` | manual |
| `B016` | `py/raise-literal` | manual |
| `B017` | `py/assert-raises-exception` | manual |
| `B018` | `py/useless-expression` | manual |
| `B019` | `py/cached-instance-method` | manual |
| `B020` | `py/loop-variable-overrides-iterator` | manual |
| `B021` | `py/f-string-docstring` | manual |
| `B022` | `py/useless-contextlib-suppress` | manual |
| `B023` | `py/function-uses-loop-variable` | manual |
| `B024` | `py/abstract-base-class-without-abstract-method` | manual |
| `B025` | `py/duplicate-try-block-exception` | manual |
| `B026` | `py/star-arg-unpacking-after-keyword-arg` | manual |
| `B027` | `py/empty-method-without-abstract-decorator` | manual |
| `B028` | `py/no-explicit-stacklevel` | suggest |
| `B029` | `py/except-with-empty-tuple` | manual |
| `B030` | `py/except-with-non-exception-classes` | manual |
| `B031` | `py/reuse-of-groupby-generator` | manual |
| `B032` | `py/unintentional-type-annotation` | manual |
| `B033` | `py/duplicate-set-value` | auto |
| `B034` | `py/re-sub-positional-args` | manual |
| `B035` | `py/static-key-dict-comprehension` | manual |
| `B039` | `py/mutable-contextvar-default` | manual |
| `B904` | `py/raise-without-from-inside-except` | manual |
| `B905` | `py/zip-without-explicit-strict` | suggest |
| `B911` | `py/batched-without-explicit-strict` | suggest |
| `B912` | `py/map-without-explicit-strict` | suggest |
| `I001` | `py/unsorted-imports` | auto |
| `RUF022` | `py/all-sorted` | auto |

## How the ports differ

Every rule matches Ruff's documented behaviour on Ruff's own examples. Where
codelaw differs, the rule's `## Differences from the source` section says so.
The ones that shape results most:

| area | difference |
|---|---|
| name resolution | The semantic model is per scope, not per statement: a name bound anywhere in a scope counts as bound everywhere in it, and `del` is not tracked. Ruff follows the order statements run in, so it reports a few more `F821`, `F405`, `F811` and `F401` findings in code that rebinds names conditionally. |
| `py/redefined-while-unused` | A read anywhere between two definitions counts as a use, even inside a function body that only runs later, and so does a mention in a string annotation; as in Ruff, a redefinition in a different branch (`if __name__ == "__main__":`) is not reported; a parameter shadowing a module-level import is not reported. |
| `py/unused-import` | Names in string annotations, and in strings inside typing generics (`Dict["Name", int]`), `cast()` and `TypeVar(bound=)`, count as uses; `# type:` comments don't, as in Ruff 0.17. Not reported for imports in a class body, `import a.b` loading a submodule of a package also imported under an alias, or importing `__all__` (Ruff reports that one). Imports inside a `try` body are reported without an edit. The fix is `suggest`, because an import can run code. |
| `py/unused-variable` | As in Ruff, unpacking is reported only from a tuple or list literal. Outside functions only an unused `except … as name` is reported. |
| `py/unsorted-imports` | Checks order only. Blank lines after the import block and line wrapping are the formatter's; `from m import a, b as c` and `import a, b` are not reported for their shape. First-party modules come from the `known_first_party` option. |
| `py/no-star-import` | Relative star imports (`from .sub import *`) are not reported: that is how a package re-exports its submodules. |
| `py/zip-without-explicit-strict`, `py/batched-without-explicit-strict`, `py/map-without-explicit-strict` | The Python version comes from each rule's `target_version` option, not a project setting; unset, the rules run. The fix adds `strict=False`, which keeps today's behaviour (Ruff has no fix for `B911`); it is `suggest`, not `auto`, because with the version unknown the keyword may not exist and would raise `TypeError` (W-17). |
| `py/no-mutable-default` | The fix is offered for unannotated parameters whose default doesn't read the parameter's own name. |
| `# noqa` | Ruff's suppression comments are not read; codelaw has its own. |

## Not ported, and why

| rule | why |
|---|---|
| pycodestyle `E1`, `E2`, `E3`, `E5`, `W` | never: layout, which the formatter owns |
| `E701`, `E702`, `E703` | never: layout (statements per line, semicolons) |
| `E902` io-error | the engine reports files it can't read (`codelaw/not-utf8`, `codelaw/internal-error`) |
| `E999` syntax-error | removed upstream in Ruff 0.8.0 |
| `B043`, `B901`, `B903`, `B909` | preview-only in Ruff 0.17.1, not part of its stable `B` set |
| flake8-bugbear `B950` | never: line length is layout, and Ruff doesn't implement it |
| `I002` missing-required-import | needs the project's list of required imports; not part of `I001` |

The rest of Ruff 0.16's wider default (`UP`, `SIM`, `PL`, `C4`, …) is planned
for a later `py/strict` preset (B-048 in [`work/BACKLOG.md`](../../work/BACKLOG.md)).

## Data

`data/python/stdlib.json` — the standard library's top-level module names, the
union of CPython 3.10–3.14's `sys.stdlib_module_names`, so modules removed in
recent versions (`distutils`, `asynchat`, `imp`, …) still count as standard
library. `py/unsorted-imports` uses it to place imports in sections. See
`NOTICE`.

## Corpus check

Each run checks every rule and compares it line by line with Ruff 0.17.1
running the same rules (`--select E4,E7,E9,F,B,I001,RUF022`).

| corpus | files | findings | rules where codelaw and Ruff differ, and why |
|---|---|---|---|
| CPython 3.9 standard library (`lib/python3.9`, no tests) | 420 | 1,526 | I001 (blank lines after imports, left to the formatter), F821/F405/F811/F401/F841 (Ruff follows statement order and `del`; codelaw doesn't report `from io import __all__`), F401/I001 `# noqa` lines, B028 (a `warnings` name with a `None` fallback), B012 (codelaw reports every `return` in a `finally`), B024 (`class ABC` inside `abc.py`), B912 (`target_version` unset) |
| pip 21.2.4 `pip/_internal` | 141 | 310 | F401: 12 lines, every one marked `# noqa`; B912 (`target_version` unset) |
| rich 15.0.0 (fully annotated) | 100 | 98 | I001: 5 blocks missing only a blank line between sections; F401 and B007: one `# noqa` line each |

Applying every `auto` and `suggest` fix to all three corpora changed 368 files,
broke none, and left no fixable finding behind.
