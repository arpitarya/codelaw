# tree-sitter-scss 1.0.0 — vendored, unchanged

[tree-sitter-scss](https://github.com/tree-sitter-grammars/tree-sitter-scss) by Amaan
Qureshi, MIT ([`LICENSE`](LICENSE), kept unchanged), at commit
`5da5ba71a558b007b352c505bfcd9095ae337022`. Only `bindings/rust/` is codelaw's:
it loads the generated parser through `tree-sitter-language`, as the CSS fork does.

**Why it exists.** *Arpit, 2026-10-26 (B-047): "Own grammar / tree-sitter-scss".* SCSS
(`.scss`) is a CSS dialect under L3; `codelaw-lang` parses it with this grammar. It is a
dependency under L11 (a copy is one): `DEPENDENCIES.md` D-33.

**Not a fork yet.** `src/` is upstream's, as published. When a construct needs a grammar
change, bump the version suffix (`1.0.0+codelaw.1`) and `GRAMMARS_FINGERPRINT`, as the CSS
fork does.
