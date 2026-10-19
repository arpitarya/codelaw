//! The per-file semantic model for JS/TS (JSX and TSX included) and Python
//! (SR-ARCHITECTURE decision 15a): scopes, bindings, references, imports
//! and exports, built in one pass over the syntax tree. No type
//! information (L3).

mod globals;
mod js;
mod model;
mod python;

pub use globals::{ES_GLOBALS, PYTHON_BUILTINS, TS_GLOBAL_TYPES};
pub use js::declared_names;
pub use model::{
    Export, Import, Imported, Redirect, RefId, Reference, Scope, ScopeId, ScopeKind, SemanticModel, Symbol, SymbolId, SymbolKind,
};

use codelaw_core::Language;

/// Whether a language has a semantic model.
pub fn supports(language: Language) -> bool {
    language.is_js_family() || language == Language::Python
}

/// Build the model for a parsed file, or `None` for a language without one.
pub fn build(language: Language, root: &codelaw_lang::Root, source: &str) -> Option<SemanticModel> {
    let node = root.root().get_inner_node();
    if language.is_js_family() {
        Some(js::build(node, source, language.is_typescript()))
    } else if language == Language::Python {
        Some(python::build(node, source))
    } else {
        None
    }
}

#[cfg(test)]
mod tests;
