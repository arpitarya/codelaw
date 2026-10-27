# tree-sitter-scss 1.0.0 — vendored, `src/` unchanged

[tree-sitter-scss](https://github.com/tree-sitter-grammars/tree-sitter-scss) by Amaan
Qureshi, MIT ([`LICENSE`](LICENSE), kept unchanged), at commit
`5da5ba71a558b007b352c505bfcd9095ae337022`. Only `bindings/rust/` is codelaw's:
it loads the generated parser through `tree-sitter-language`, as the CSS fork does, and its
`build.rs` force-includes `c_locale_wctype.h` (`/FI` on MSVC, `-include` elsewhere), so the
scanner's `iswspace` and `iswalnum` answer with the "C" locale's ASCII classes on every
platform. Without it MSVC's `iswspace` read U+00A0 as a space, `.menu<U+00A0>.item` became a
descendant selector on Windows only, and `css/no-irregular-whitespace` failed the win32 legs
(W-44, 2026-10-27). That build change is `1.0.0+codelaw.1`; the fingerprint names it.

**Why it exists.** *Arpit, 2026-10-26 (B-047): "Own grammar / tree-sitter-scss".* SCSS
(`.scss`) is a CSS dialect under L3; `codelaw-lang` parses it with this grammar. It is a
dependency under L11 (a copy is one): [L11's list](../../../../records/0013_LAW-11-dependencies.md) D-33.

**Not a fork yet.** `src/` is upstream's, as published. When a construct needs a grammar
change, bump the version suffix (`1.0.0+codelaw.1`) and `GRAMMARS_FINGERPRINT`, as the CSS
fork does.
