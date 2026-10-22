//! The project graph (SR-PROJECT-GRAPH): per-file facts read once and
//! cached by content, a resolver over the project's own file set, the
//! linked graph of modules, exports, imports and uses, and the `graph:`
//! queries `scope: project` rules run on it.
//!
//! The graph is a pure function of the files and `[graph]` config (L5):
//! nothing here reads the disk, the clock or the network.

pub mod config;
pub mod facts;
pub mod link;
pub mod query;
pub mod resolve;

pub use config::GraphConfig;
pub use facts::{Decl, DeclKind, FactLang, FileFacts, Span, SpecForm, Specifier, extract};
pub use link::{Graph, GraphInput, Manifest, Module, Use, is_manifest};
pub use query::{GraphFinding, GraphRule, Query, compile, refs, run, why_unused};

/// Bumped whenever extraction changes: part of the facts cache key.
pub const FACTS_VERSION: &str = "graph-facts-4";

/// Whether the graph reads facts from a language.
pub fn reads(language: codelaw_core::Language) -> bool {
    language.is_js_family() || matches!(language, codelaw_core::Language::Python | codelaw_core::Language::Css | codelaw_core::Language::Html)
}

#[cfg(test)]
mod tests;
