# feat: range media features, `layer()` in `@import`, fractional keyframe selectors

## Summary

Three CSS forms that browsers have shipped for a while currently parse with
`ERROR` nodes in tree-sitter-css 0.25.0. This adds them to the grammar.

| construct | example | spec | parses as |
|---|---|---|---|
| Range media features | `@media (width >= 600px)`, `(400px <= width <= 700px)`, `(aspect-ratio > 16/9)` | Media Queries Level 4 (range context) | new `range_query` node: `feature_name` plus one or two values (`integer_value`, `float_value`, `call_expression`, or a ratio as `binary_expression`) |
| `layer` in `@import` | `@import url(a.css) layer(base.components) screen;` | CSS Cascading and Inheritance Level 5 (`@import`) | `call_expression` with `function_name` `layer`; a bare `layer` stays `keyword_query`, as today |
| Fractional keyframe percentages | `@keyframes k { 12.5% { … } }` | CSS Animations Level 1 (keyframe selectors) | `float_value` is now accepted in `keyframe_block` beside `integer_value` |

## Changes

- `grammar.js`:
  - `range_query`, added to `_query`, so it composes with `and`/`or`/`not`
    and is available to `@container` through `at_rule`. The operators are
    literal tokens (`=`, `<`, `<=`, `>`, `>=`); the two-bound form requires
    both operators to point the same way, as the spec does. Values are
    restricted to numbers, functions and ratios, so the leading
    `identifier` stays unambiguous and no `conflicts` entry is needed.
  - `import_statement` takes an optional `layer` / `layer(<name>)` between the
    URL and the media query list.
  - `keyframe_block` accepts `float_value`.
- `queries/highlights.scm`: `<`, `<=` and `>=` as `@operator`.
- `test/corpus/modern.txt`: six corpus tests covering the three constructs.
  Happy to fold them into the existing corpus files (`statements.txt`?) if
  you prefer.
- `src/` is regenerated with tree-sitter-cli 0.25.10 (the version 0.25.0 was
  generated with; from the unmodified grammar it reproduces the released
  `src/parser.c` byte for byte).

## Compatibility

- **No existing tree changes.** I parsed 564 real stylesheets (bootstrap
  5.3.3, bulma 1.0.2, normalize.css 8.0.1, animate.css 4.1.1, pico 2.0.6,
  open-props 1.7.6, plus ~200 snippets) with 0.25.0 and with this patch:
  every file that parsed without an error gives an identical tree; 6 files
  that had errors now parse cleanly (animate.css's fractional keyframes,
  range media queries).
- Parser size: `parser.c` grows from about 504 KB to 537 KB.
- **Not covered here** (they still produce `ERROR`, as before):
  `supports(…)` in `@import`, a name before the query in `@custom-media` and
  `@container`, and named-range keyframe selectors (`entry 0%`). Glad to
  follow up separately.

## Testing

```bash
tree-sitter generate
tree-sitter test
```

Run on a checkout of the `v0.25.0` tag with tree-sitter-cli 0.25.10:

- before the patch, `tree-sitter generate` reproduces the released
  `src/parser.c` byte for byte and `tree-sitter test` passes 40/40;
- with the patch, `tree-sitter test` passes 46/46 (the 40 existing corpus
  tests unchanged, plus the 6 new ones) and the highlight test still passes
  (37 assertions);
- `npm run lint` (`eslint grammar.js`) is clean.

## Context

Prepared for codelaw, a linter that
uses tree-sitter-css through ast-grep; it carries this change as an in-tree
fork until a release includes it.
