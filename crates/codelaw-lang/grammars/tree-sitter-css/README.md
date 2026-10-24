# tree-sitter-css — codelaw's fork of 0.25.0

**This is a fork.** It is [tree-sitter-css](https://github.com/tree-sitter/tree-sitter-css)
0.25.0 by Max Brunsfeld and Amaan Qureshi, MIT-licensed ([`LICENSE`](LICENSE),
kept unchanged), with a grammar change codelaw needs. All credit for the
grammar is upstream's.

**Why it exists.** Upstream 0.25.0 drops to `ERROR` nodes on three modern CSS
forms, so codelaw's fix verifier refused any fix that writes them
(W-21). The workspace `Cargo.toml` puts this crate in place of the
crates.io one with `[patch.crates-io]`, so `ast-grep-language` builds it.

**The ruling.** *Arpit, 2026-10-22: fork in-tree + upstream patch; drop the
fork when upstream releases.* SR-ARCHITECTURE decision 3 records it.

## What changed

| construct | example | node |
|---|---|---|
| range media features (Media Queries 4) | `(width >= 600px)`, `(400px <= width <= 700px)`, `(aspect-ratio > 16/9)` | `range_query` |
| `layer` in `@import` (Cascade 5) | `@import url(a.css) layer(base) screen;` | `call_expression`; a bare `layer` stays `keyword_query` |
| fractional keyframe percentages | `12.5% { … }` | `float_value` in `keyframe_block` |

- **Only `grammar.js` was edited by hand,** plus the new `<`, `<=` and `>=`
  operators in `queries/highlights.scm`. `src/parser.c`, `src/grammar.json`
  and `src/node-types.json` are generated from `grammar.js`.
- **`test/corpus/modern.txt`** is new: one tree-sitter corpus test per construct.
- **Nothing else changed.** Every tree upstream 0.25.0 parses without an error
  is unchanged: checked on 564 files (bootstrap, bulma, normalize.css,
  animate.css, pico, open-props and every CSS example in `packs/`), 2026-10-22.
- **Not covered:** `@import … supports(…)`, a name before a query in
  `@custom-media` / `@container`, and named-range keyframe selectors
  (`entry 0%`). They still parse with `ERROR` nodes.

## Regenerating `src/` (manual; no build or CI step runs this)

The tree-sitter CLI (MIT) is **not** a dependency of the build: the generated
`src/parser.c` is checked in, and `cargo build` only compiles it.

```bash
cargo install tree-sitter-cli --version 0.25.10 --locked   # needs Node.js on PATH
cd crates/codelaw-lang/grammars/tree-sitter-css
tree-sitter generate      # rewrites src/parser.c, src/grammar.json, src/node-types.json
tree-sitter test          # runs test/corpus/*.txt
```

- **Use 0.25.10.** It is the version upstream generated 0.25.0 with, and its
  ABI 15 output loads in the workspace's `tree-sitter` 0.27 runtime.
  Regenerating the unmodified upstream `grammar.js` with it reproduces
  upstream's `src/parser.c` byte for byte.
- **After any change to `grammar.js`:** bump the `+codelaw.N` in `Cargo.toml`,
  run `cargo update -p tree-sitter-css`, and bump the same version in
  `codelaw_lang::GRAMMARS_FINGERPRINT`, so cached results are invalidated
  (a unit test checks the two agree).

## Dropping the fork

When upstream releases a `tree-sitter-css` that parses the three forms and
`ast-grep-language` can take it: delete this folder and its
`[patch.crates-io]` line, return `GRAMMARS_FINGERPRINT` to the released
version, and run `cargo test -p codelaw-lang` (`modern_css_parses` must still
pass).

## The upstream patch

[`upstream/grammar.patch`](upstream/grammar.patch) is this folder's change
against upstream's `v0.25.0` (`grammar.js`, `queries/highlights.scm`,
`test/corpus/modern.txt`; no `src/`), and [`upstream/PR.md`](upstream/PR.md)
is the pull-request text, ready to paste. Checked 2026-10-24 on a checkout of
the tag with tree-sitter-cli 0.25.10: the patch applies cleanly, upstream's
corpus passes 46/46 (40 existing, 6 new), `eslint grammar.js` is clean, and
the regenerated `src/parser.c`, `grammar.json` and `node-types.json` are
byte-identical to this folder's. It is not filed: opening the PR on
upstream's repository is the decider's to do.
