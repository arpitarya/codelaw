//! The grammars of the L3 languages, behind one ast-grep `Language` type.
//!
//! Eight grammars come through `ast-grep-language`; TOML is registered here
//! as a custom language (SR-LINTER decision 1). Every grammar is pinned in
//! `Cargo.lock`, and [`GRAMMARS_FINGERPRINT`] joins every cache key
//! (SR-ARCHITECTURE decision 3).

mod grammar;
mod regions;

pub use ast_grep_core::Language as AgLanguage;
pub use grammar::Grammar;
pub use regions::{Hole, Region, RegionOptions, regions};

use ast_grep_core::tree_sitter::{LanguageExt, StrDoc};
use ast_grep_core::{AstGrep, Node};
use codelaw_core::Language;

/// A parsed file or region.
pub type Root = AstGrep<StrDoc<Grammar>>;
/// A node of a parsed file.
pub type SyntaxNode<'r> = Node<'r, StrDoc<Grammar>>;

/// Names every grammar and its pinned version; part of every cache key.
pub const GRAMMARS_FINGERPRINT: &str = "ast-grep-language=0.51.0;tree-sitter=0.27.0;tree-sitter-toml-ng=0.7.0;tree-sitter-md=0.5.3";

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
            (Language::Html, "<p class=\"a\">x</p>\n"),
        ];
        for (lang, src) in samples {
            let root = parse(lang, src);
            assert_eq!(syntax_error_count(&root), 0, "{lang} failed to parse: {}", root.root().get_inner_node().to_sexp());
        }
    }

    #[test]
    fn errors_are_counted() {
        let root = parse(Language::Javascript, "const = ;\n");
        assert!(syntax_error_count(&root) > 0);
    }
}
