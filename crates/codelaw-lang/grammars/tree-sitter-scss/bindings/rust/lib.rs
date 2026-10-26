//! SCSS language support for the tree-sitter parsing library: codelaw's
//! in-tree copy of tree-sitter-scss (MIT), loaded through `tree-sitter-language`.

use tree_sitter_language::LanguageFn;

extern "C" {
    fn tree_sitter_scss() -> *const ();
}

/// The tree-sitter [`LanguageFn`] for this grammar.
pub const LANGUAGE: LanguageFn = unsafe { LanguageFn::from_raw(tree_sitter_scss) };

/// The content of the `node-types.json` file for this grammar.
pub const NODE_TYPES: &str = include_str!("../../src/node-types.json");
