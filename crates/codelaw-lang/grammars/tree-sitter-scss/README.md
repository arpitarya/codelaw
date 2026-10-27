# tree-sitter-scss 1.0.0 — codelaw's fork, built on the CSS fork

[tree-sitter-scss](https://github.com/tree-sitter-grammars/tree-sitter-scss) by Amaan
Qureshi, MIT ([`LICENSE`](LICENSE), kept unchanged), at commit
`5da5ba71a558b007b352c505bfcd9095ae337022`, with codelaw's changes below. All credit for
the grammar is upstream's. `bindings/rust/` is codelaw's: it loads the generated parser
through `tree-sitter-language`, as the CSS fork does, and its `build.rs` force-includes
`c_locale_wctype.h` (`/FI` on MSVC, `-include` elsewhere), so the scanner's `iswspace` and
`iswalnum` answer with the "C" locale's ASCII classes on every platform. Without it MSVC's
`iswspace` read U+00A0 as a space, `.menu<U+00A0>.item` became a descendant selector on
Windows only, and `css/no-irregular-whitespace` failed the win32 legs (W-44, 2026-10-27).
That build change is `1.0.0+codelaw.1`.

**Why it exists.** *Arpit, 2026-10-26 (B-047): "Own grammar / tree-sitter-scss".* SCSS
(`.scss`) is a CSS dialect under L3; `codelaw-lang` parses it with this grammar. It is a
dependency under L11 (a copy is one): its list's D-33.

## What changed (`1.0.0+codelaw.2`, W-62, 2026-10-27)

Upstream 1.0.0 gave `ERROR` on most modern Sass: on Bootstrap's SCSS (5.3.8 and `main` at
`d793f28`, 236 files) it parsed 52 files without an error. This fork parses 233, and every
tree 1.0.0 parsed without an error still parses without one.

- **Built on codelaw's CSS fork.** `grammar.js` requires `../tree-sitter-css/grammar.js`
  (the in-tree fork, `+codelaw.3`) instead of the npm `tree-sitter-css` 0.20 upstream builds
  on, so SCSS gets the fork's forms (range media features, `layer()` and `supports()` in
  `@import`, `@container name (…)`, `.b &`, …) and codelaw depends on no second CSS grammar.
- **The Sass module system:** `@use "x" as *`, `as name`, `with (…)`; `@forward "x" show …`,
  `hide …`, `as prefix-*`, `with (…)` (`as_clause`, `show_clause`, `hide_clause`,
  `with_clause`).
- **Values:** maps (`map_value`, `map_pair`), the empty list `()`, `!default` / `!global`
  (`important_value`), `not` (`unary_expression`), `and` / `or` / `%` with Sass's precedence,
  signed variables and interpolations (`-$x`, `-#{$y}`), keyword and rest arguments and
  `if(sass(…): …; else: …)` in a call, interpolation followed by a number (`--#{$c}-500`),
  Sass variables and interpolation in media features (`(min-width: $bp)`, `(#{$p}: #{$v})`).
- **Statements:** `@if not …`, empty parameters and arguments (`@mixin m()`,
  `@include m()`), rest parameters (`$args...`), namespaced `@include bp.up(md)`,
  `@include … using (…)`, a space before `@include`'s arguments, `@content(…)`
  (`content_statement`), `@return a, b`, `@extend … !optional`, `@at-root <selectors>`,
  `@for … to …`, `@import "a", "b"`.
- **Selectors and declarations:** interpolated placeholders, pseudo-elements and
  pseudo-classes, the CSS fork's pseudo-classes kept (`:nth-child(n + 3)`,
  `:nth-child(#{$n})`), escaped class names (`.lg\:drawer`), nested properties and block
  values (`margin: { top: 1px; }`: a `block` under the `declaration`).
- **Lexing:** a colon followed by a space ends a plain value (`(a: 1)` is a map key, not
  `a:`); `#` before `{` ends one; `100%`'s `%` is a unit, not the modulo operator.
- **`src/scanner.c`:** no pseudo-class colon when a space follows it, and `#{…}` is skipped
  when it looks for the `{` that makes a selector.
- **`test/corpus/codelaw.txt`** holds one corpus test per change; upstream's other corpus
  files have their expected trees updated to the CSS fork's node shapes (`string_content`,
  a `class_name`'s `identifier`).
- **Not covered:** `@container #{$name} (…)` (an interpolation before a query in a generic
  at-rule), a string holding an interpolation with its own quotes
  (`'#{if($a, '-x', '')}'`), an interpolation after a pseudo-class's arguments
  (`:not(.a)#{$x}`), an escape after an interpolation, and the dropped `@nest`.

## Regenerating `src/`

`tree-sitter-cli` 0.25.10 is L11's D-36: exact pin, the generated `src/` committed, run only
when a grammar changes. After any change to the CSS fork's `grammar.js` too:

```bash
cd crates/codelaw-lang/grammars/tree-sitter-scss   # needs Node.js on PATH
npx -y tree-sitter-cli@0.25.10 generate   # rewrites src/parser.c, src/grammar.json, src/node-types.json
npx -y tree-sitter-cli@0.25.10 test       # runs test/corpus/*.txt
```

Then bump `+codelaw.N` in `Cargo.toml`, run `cargo update -p tree-sitter-scss`, and bump the
same version in `codelaw_lang::GRAMMARS_FINGERPRINT` (a unit test checks the two agree).
