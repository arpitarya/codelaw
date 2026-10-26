//! A text's token stream through codelaw's own parser — what decides that a
//! rule is a layout rule (SR-FORMATTER decision 3): two texts with the same
//! stream differ only in layout.
//!
//! The stream is every visible node of the tree in pre-order, each with its
//! depth and kind, and each leaf with its exact text. Whitespace that no
//! token holds is not in it; everything else is:
//!
//! - **comments** are tokens with text, although tree-sitter parses them as
//!   extras — a fix that edits a comment is not layout;
//! - **text no node holds** (Markdown's inline grammar leaves plain words
//!   outside any node) is split on whitespace into `text` tokens, so a
//!   changed word is a changed token;
//! - **depth** carries the structure that hidden tokens hold — Python's and
//!   YAML's indentation — so re-indenting a block into another is not layout;
//! - **embedded regions** (`<script>`, `<style>`, Markdown code blocks and
//!   inline content) are compared as their own language: the host leaf they
//!   stand in keeps its kind and loses its text, and each region's own
//!   stream follows the host's.

use crate::{Grammar, RegionOptions, parse_grammar, regions};
use codelaw_core::Language;

/// One entry of a token stream.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub depth: u32,
    /// The node kind; `text` for a word no node holds; `region` heads an
    /// embedded region's stream, with its language as `text`.
    pub kind: String,
    /// A leaf's exact text; empty for an inner node and for a leaf an
    /// embedded region stands in.
    pub text: String,
}

/// The token stream of `source` parsed as `language`, embedded regions
/// included.
pub fn token_stream(language: Language, source: &str) -> Vec<Token> {
    let mut out = Vec::new();
    stream(Grammar::for_language(language), Some(language), source, 0, &mut out);
    out
}

fn words(gap: &str, depth: u32, out: &mut Vec<Token>) {
    out.extend(gap.split_whitespace().map(|w| Token { depth, kind: "text".into(), text: w.to_owned() }));
}

fn stream(grammar: Grammar, language: Option<Language>, source: &str, base: u32, out: &mut Vec<Token>) {
    let root = parse_grammar(grammar, source);
    let options = RegionOptions { lint_code_blocks: true, tagged_templates: false };
    let embedded = match language {
        Some(l) if grammar == Grammar::for_language(l) => regions(l, &root, source, options),
        _ => Vec::new(),
    };
    let in_region = |start: usize, end: usize| embedded.iter().any(|r| start < r.host_end && r.host_start < end);
    // Bytes no leaf holds, outside every region, split into words.
    let gap = |from: usize, to: usize, depth: u32, out: &mut Vec<Token>| {
        let mut at = from;
        for r in embedded.iter().filter(|r| r.host_start < to && r.host_end > from) {
            if r.host_start > at {
                words(&source[at..r.host_start], depth, out);
            }
            at = at.max(r.host_end);
        }
        if at < to {
            words(&source[at..to], depth, out);
        }
    };
    let mut pos = 0;
    let mut depth = base;
    let mut cursor = root.root().get_inner_node().walk();
    loop {
        let node = cursor.node();
        let (start, end) = (node.start_byte(), node.end_byte());
        let leaf = node.child_count() == 0;
        if leaf && start > pos {
            gap(pos, start, depth, out);
        }
        let text = if !leaf || in_region(start, end) { String::new() } else { source[start..end].to_owned() };
        out.push(Token { depth, kind: node.kind().to_owned(), text });
        if leaf {
            pos = pos.max(end);
        }
        if cursor.goto_first_child() {
            depth += 1;
            continue;
        }
        loop {
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                if pos < source.len() {
                    gap(pos, source.len(), base, out);
                }
                for r in &embedded {
                    out.push(Token { depth: base, kind: "region".into(), text: r.language.to_string() });
                    let nested = (r.grammar == Grammar::for_language(r.language)).then_some(r.language);
                    stream(r.grammar, nested, &r.text, base + 1, out);
                }
                return;
            }
            depth -= 1;
        }
    }
}

/// Whether two texts differ only in layout: the same token stream.
pub fn same_tokens(language: Language, a: &str, b: &str) -> bool {
    token_stream(language, a) == token_stream(language, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn same(l: Language, a: &str, b: &str) -> bool {
        same_tokens(l, a, b)
    }

    #[test]
    fn whitespace_between_tokens_is_layout() {
        assert!(same(Language::Javascript, "if (a==b) {}", "if (a == b) {\n}\n"));
        assert!(same(Language::Css, "a{color:red}", "a {\n  color: red;\n}".replace(';', "").as_str()));
        assert!(same(Language::Json, "{\"a\":[1,2]}", "{ \"a\": [ 1, 2 ] }\n"));
        assert!(same(Language::Toml, "a=1\n", "a = 1\n"));
        assert!(same(Language::Python, "f(a,b)\n", "f(a, b)\n"));
    }

    /// An irregular space is not layout: it is part of the name around it.
    /// On Windows the SCSS scanner read U+00A0 as a descendant combinator
    /// (MSVC's `iswspace`), so `.menu<U+00A0>.item` had the tokens of
    /// `.menu .item` (W-44; the grammars' `c_locale_wctype.h`).
    #[test]
    fn an_irregular_space_is_not_layout() {
        let spaces = ['\u{85}', '\u{a0}', '\u{1680}', '\u{2000}', '\u{2007}', '\u{200a}', '\u{2028}', '\u{2029}', '\u{202f}', '\u{205f}', '\u{3000}', '\u{feff}'];
        for language in [Language::Css, Language::Scss] {
            for space in spaces {
                let irregular = format!(".menu{space}.item {{\n  color: red;\n}}\n");
                assert!(!same(language, &irregular, ".menu .item {\n  color: red;\n}\n"), "{language:?}: U+{:04X} read as layout", space as u32);
            }
        }
    }

    #[test]
    fn a_changed_token_is_not_layout() {
        assert!(!same(Language::Javascript, "a == b;", "a === b;"));
        // Inside a string, whitespace is token text.
        assert!(!same(Language::Javascript, "f(\"a  b\");", "f(\"a b\");"));
        // Quotes are tokens.
        assert!(!same(Language::Javascript, "f('a');", "f(\"a\");"));
        // A comment is a token, although the grammar parses it as an extra.
        assert!(!same(Language::Javascript, "a; // x\n", "a; //x\n"));
        assert!(!same(Language::Javascript, "a; // x\n", "a;\n"));
        assert!(!same(Language::Css, "@media (a) {}\n", "@media not (a) {}\n"));
    }

    /// CSS Syntax reads `not(` as a function token and `not (` as an
    /// identifier and a parenthesis, but codelaw's CSS grammar parses both to
    /// the same tree: by codelaw's own tokens the two differ only in layout.
    /// This flips when the fork learns the function token (B-045).
    #[test]
    fn the_css_grammar_does_not_see_not_paren() {
        assert!(same(Language::Css, "@media not(a) {}\n", "@media not (a) {}\n"));
    }

    #[test]
    fn indentation_that_moves_a_block_is_not_layout() {
        assert!(!same(Language::Python, "if a:\n    b\nc\n", "if a:\n    b\n    c\n"));
        assert!(same(Language::Python, "if a:\n    b\n", "if a:\n  b\n"));
        assert!(!same(Language::Yaml, "a:\n  b: 1\nc: 2\n", "a:\n  b: 1\n  c: 2\n"));
    }

    #[test]
    fn markdown_words_and_regions_are_tokens() {
        assert!(!same(Language::Markdown, "hello world\n", "hello brave world\n"));
        assert!(!same(Language::Markdown, "# Title\n", "#Title\n"));
        // A code block is compared as its own language.
        assert!(same(Language::Markdown, "```js\na==b\n```\n", "```js\na == b\n```\n"));
        assert!(!same(Language::Markdown, "```js\na==b\n```\n", "```js\na===b\n```\n"));
        assert!(same(Language::Html, "<script>a==b</script>\n", "<script>\n  a == b\n</script>\n"));
        assert!(!same(Language::Html, "<script>a==b</script>\n", "<script>a===b</script>\n"));
    }
}
