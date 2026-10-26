//! The grammars of the L3 languages, behind one ast-grep `Language` type.
//!
//! Eight grammars come through `ast-grep-language`; TOML is registered here
//! as a custom language (SR-LINTER decision 1). CSS is codelaw's in-tree
//! fork of `tree-sitter-css` (`grammars/tree-sitter-css`), patched in by the
//! workspace manifest. Every grammar is pinned in `Cargo.lock`, and
//! [`GRAMMARS_FINGERPRINT`] joins every cache key (SR-ARCHITECTURE
//! decision 3).

mod functions;
mod grammar;
mod regions;
mod tokens;

pub use ast_grep_core::Language as AgLanguage;
pub use functions::{Change, Function, classify, functions, has_functions};
pub use grammar::Grammar;
pub use regions::{Hole, Region, RegionOptions, regions};
pub use tokens::{Token, same_tokens, token_stream};

use ast_grep_core::tree_sitter::{LanguageExt, StrDoc};
use ast_grep_core::{AstGrep, Node};
use codelaw_core::Language;

/// A parsed file or region.
pub type Root = AstGrep<StrDoc<Grammar>>;
/// A node of a parsed file.
pub type SyntaxNode<'r> = Node<'r, StrDoc<Grammar>>;

/// Names every grammar and its pinned version; part of every cache key.
/// The CSS fork's version carries `+codelaw.N`, bumped with every change to
/// its `grammar.js`, so a grammar change invalidates cached results.
pub const GRAMMARS_FINGERPRINT: &str = "ast-grep-language=0.51.0;tree-sitter=0.27.0;tree-sitter-toml-ng=0.7.0;tree-sitter-md=0.5.3;tree-sitter-css=0.25.0+codelaw.1;tree-sitter-scss=1.0.0";

/// Parse source text with the grammar for `language`.
pub fn parse(language: Language, source: &str) -> Root {
    parse_grammar(Grammar::for_language(language), source)
}

/// Parse source text with one grammar.
pub fn parse_grammar(grammar: Grammar, source: &str) -> Root {
    grammar.ast_grep(source)
}

/// Replace the bytes `start..end` of a parsed text with `text` and re-parse
/// incrementally: tree-sitter reuses every subtree the edit did not touch,
/// and the tree is the one a fresh parse of the new text gives
/// (SR-ARCHITECTURE decision 23). The range must lie on char boundaries.
pub fn reparse(root: &mut Root, start: usize, end: usize, text: &str) -> Result<(), String> {
    let len = root.get_text().len();
    if start > end || end > len {
        return Err(format!("edit {start}..{end} is outside the text ({len} bytes)"));
    }
    let edit = ast_grep_core::source::Edit::<String> { position: start, deleted_length: end - start, inserted_text: text.as_bytes().to_vec() };
    root.edit(edit).map(|_| ())
}

/// The text a parsed root was parsed from.
pub fn text_of(root: &Root) -> &str {
    root.get_text()
}

/// The number of error and missing nodes; the fix verifier compares it
/// before and after an edit (L4 decision 5).
pub fn syntax_error_count(root: &Root) -> usize {
    let mut count = 0;
    let mut cursor = root.root().get_inner_node().walk();
    let mut descend = true;
    loop {
        if descend {
            let node = cursor.node();
            if node.is_error() || node.is_missing() {
                count += 1;
            }
            if node.has_error() && cursor.goto_first_child() {
                continue;
            }
        }
        if cursor.goto_next_sibling() {
            descend = true;
            continue;
        }
        if !cursor.goto_parent() {
            break;
        }
        descend = false;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_language_parses() {
        let samples = [
            (Language::Javascript, "const a = 1;\n"),
            (Language::Jsx, "const a = <div className=\"x\">{b}</div>;\n"),
            (Language::Typescript, "@Component({ tag: 'x' })\nclass A { @Prop() readonly b: string; }\n"),
            (Language::Tsx, "const a = <A b={1} />;\n"),
            (Language::Python, "def f(x):\n    return x\n"),
            (Language::Json, "{ \"a\": [1, 2] }\n"),
            (Language::Yaml, "a: 1\n---\nb: 2\n"),
            (Language::Toml, "[project]\nname = \"x\"\n"),
            (Language::Markdown, "# Title\n\ntext\n"),
            (Language::Css, "a { color: red; }\n"),
            (Language::Scss, "$a: 1px;\n.b { &-c { margin: $a; } }\n"),
            (Language::Html, "<p class=\"a\">x</p>\n"),
        ];
        for (lang, src) in samples {
            let root = parse(lang, src);
            assert_eq!(syntax_error_count(&root), 0, "{lang} failed to parse: {}", root.root().get_inner_node().to_sexp());
        }
    }

    /// The in-tree `tree-sitter-css` fork reads the modern CSS upstream 0.25.0
    /// rejects; this fails if the `[patch.crates-io]` stops applying.
    #[test]
    fn modern_css_parses() {
        let samples = [
            "@media (width >= 600px) { a { color: red; } }\n",
            "@media (400px <= width <= 700px) and (hover) { a { color: red; } }\n",
            "@media (aspect-ratio > 16/9), not (height < 300px) { a { color: red; } }\n",
            "@import url(\"a.css\") layer(base.components) screen;\n",
            "@import \"a.css\" layer;\n",
            "@keyframes k { from, 12.5% { opacity: 0; } 50% { opacity: 1; } to { opacity: 0; } }\n",
        ];
        for src in samples {
            let root = parse(Language::Css, src);
            assert_eq!(syntax_error_count(&root), 0, "{src} failed to parse: {}", root.root().get_inner_node().to_sexp());
        }
        let range = parse(Language::Css, "@media (width >= 600px) {}\n");
        assert!(range.root().get_inner_node().to_sexp().contains("range_query"));
    }

    /// The fingerprint names the CSS fork's version as `Cargo.lock` resolves
    /// it, so bumping `+codelaw.N` reaches every cache key.
    #[test]
    fn fingerprint_names_the_css_fork() {
        let lock = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.lock")).unwrap();
        let entry = lock.split("[[package]]").find(|p| p.contains("name = \"tree-sitter-css\"")).expect("tree-sitter-css in Cargo.lock");
        let version = entry.lines().find_map(|l| l.strip_prefix("version = \"")).and_then(|v| v.strip_suffix('"')).unwrap();
        assert!(version.contains("+codelaw."), "Cargo.lock resolves tree-sitter-css {version}, not the in-tree fork");
        assert!(!entry.contains("source ="), "the fork is a path crate, not a registry one");
        assert!(GRAMMARS_FINGERPRINT.contains(&format!("tree-sitter-css={version}")), "GRAMMARS_FINGERPRINT lacks tree-sitter-css={version}");
    }

    #[test]
    fn errors_are_counted() {
        let root = parse(Language::Javascript, "const = ;\n");
        assert!(syntax_error_count(&root) > 0);
    }
}
