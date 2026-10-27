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
| `supports()` in `@import` (Cascade 5; `+codelaw.3`, W-62) | `@import url(a.css) supports(display: grid) screen;` | `call_expression` whose `arguments` hold a `feature_query` or a query |
| a name before an at-rule's query | `@container sidebar (min-width: 400px)`, `@custom-media --small (…)`, `@font-feature-values Font One` | `at_rule`: each comma-separated part of the prelude is one or more queries |
| page selectors | `@page :first`, `@page wide:left` | `page_selector` in `at_rule` |
| named-range keyframe selectors (Scroll-driven Animations) | `entry 0% { … }` | `keyframe_range` in `keyframe_block` |
| a nesting selector after a descendant space | `.a { .b & { … } }` | `descendant_selector` (the scanner reads `&`) |
| a custom property holding a block | `--x: { color: red; };` | a `block` under the `declaration` (the scanner takes a colon followed by a space as a declaration's, not a pseudo-class's) |
| annotations other than `!important` | `color: red !ie;` | `important_value` in `declaration` |

- **`grammar.js` was edited by hand,** plus the new `<`, `<=` and `>=`
  operators in `queries/highlights.scm` and, since `+codelaw.3`, two changes
  to `src/scanner.c` (a descendant space before `&`; no pseudo-class colon
  when a space follows it). `src/parser.c`, `src/grammar.json` and
  `src/node-types.json` are generated from `grammar.js`.
- **The scanner's character classes are the "C" locale's everywhere**
  (`+codelaw.2`, W-44, 2026-10-27). `src/scanner.c` asks `<wctype.h>`
  whether a character is a space or a letter, and C libraries disagree:
  MSVC's `iswspace` says yes to U+00A0, musl's to U+2028 and U+3000.
  `bindings/rust/build.rs` force-includes `bindings/rust/c_locale_wctype.h`
  (`/FI` on MSVC, `-include` elsewhere), which pins `iswspace`, `iswalpha`,
  `iswdigit` and `iswalnum` to ASCII, as macOS and glibc answer in the "C"
  locale; `src/scanner.c`'s character-class calls are upstream's. A unit test in
  `codelaw-lang` fails if the scanner calls a class the shim does not pin.
- **`test/corpus/modern.txt`** is new: one tree-sitter corpus test per construct.
- **Nothing else changed.** Every tree upstream 0.25.0 parses without an error
  is unchanged: checked on 564 files (bootstrap, bulma, normalize.css,
  animate.css, pico, open-props and every CSS example in `packs/`), 2026-10-22.
  For `+codelaw.3` (2026-10-27), on 738 files (Bootstrap 5.3.8's `dist/css`,
  Bulma, normalize.css, animate.css, Pico, Open Props and every CSS example in
  `packs/` and `docs/`): the files without an error went from 586 to 723, and
  every tree `+codelaw.2` parsed without an error is byte-identical.
- **Not covered:** an unquoted `url(data:…;…)` holding a `;`, the dropped
  `@nest` rule, CSS Modules' `@value` (W-63) and a backslash-newline inside a
  string. They still parse with `ERROR` nodes.
- **`tree-sitter-scss` is built on this grammar** (`../tree-sitter-scss`):
  regenerate it after any change here.

## Regenerating `src/` (manual; no build or CI step runs this)

The tree-sitter CLI (MIT) is **not** a dependency of the build: the generated
`src/parser.c` is checked in, and `cargo build` only compiles it.

```bash
cd crates/codelaw-lang/grammars/tree-sitter-css   # needs Node.js on PATH
npx -y tree-sitter-cli@0.25.10 generate   # rewrites src/parser.c, src/grammar.json, src/node-types.json
npx -y tree-sitter-cli@0.25.10 test       # runs test/corpus/*.txt
cd ../tree-sitter-scss && npx -y tree-sitter-cli@0.25.10 generate && npx -y tree-sitter-cli@0.25.10 test
```

`tree-sitter-cli` 0.25.10 is L11's D-36 (ratified 2026-10-27): exact pin, the
generated `src/` committed, run only when a grammar changes.

- **Use 0.25.10.** It is the version upstream generated 0.25.0 with, and its
  ABI 15 output loads in the workspace's `tree-sitter` 0.27 runtime.
  Regenerating the unmodified upstream `grammar.js` with it reproduces
  upstream's `src/parser.c` byte for byte.
- **After any change to `grammar.js` or to `bindings/rust/` that changes how `src/` compiles:** bump the `+codelaw.N` in `Cargo.toml`,
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
is the pull-request text, ready to paste. They cover the three forms of
2026-10-22 only; `+codelaw.3`'s forms (W-62) are not in the upstream patch
yet. Checked 2026-10-24 on a checkout of
the tag with tree-sitter-cli 0.25.10: the patch applies cleanly, upstream's
corpus passes 46/46 (40 existing, 6 new), `eslint grammar.js` is clean, and
the regenerated `src/parser.c`, `grammar.json` and `node-types.json` are
byte-identical to this folder's. It is not filed: opening the PR on
upstream's repository is the decider's to do.
