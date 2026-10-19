//! Native rules: checks a pattern can't express, implemented in Rust and
//! registered by id (SR-ARCHITECTURE decision 15b). The rule's Markdown
//! file is still the rule — its docs, fix class, options and examples —
//! and names one of these by `native: <id>`.
//!
//! Every native rule declares the node kinds it inspects, so the engine
//! hands it only those (decision 15); it is deterministic (L5) and the
//! engine isolates it on panic (decision 25).

mod no_unreachable;

use codelaw_core::{Language, TextEdit};
use codelaw_lang::SyntaxNode;
use codelaw_semantic::SemanticModel;
use std::collections::BTreeMap;

/// What a native rule sees of one file.
pub struct NativeContext<'a> {
    pub source: &'a str,
    pub language: Language,
    pub semantic: Option<&'a SemanticModel>,
    /// The rule's options, resolved for this file.
    pub options: &'a BTreeMap<String, serde_json::Value>,
}

/// One finding of a native rule. `edits` are offered when the rule file's
/// fix class is `auto` or `suggest`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeFinding {
    pub start: usize,
    pub end: usize,
    pub message: String,
    pub edits: Vec<TextEdit>,
}

pub trait NativeRule: Send + Sync {
    fn id(&self) -> &'static str;
    fn languages(&self) -> &'static [Language];
    /// The node kinds `check` is called on.
    fn kinds(&self) -> &'static [&'static str];
    fn needs_semantic(&self) -> bool {
        false
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>);
}

static REGISTRY: &[&dyn NativeRule] = &[&no_unreachable::NoUnreachable];

/// Every registered native rule, sorted by id.
pub fn registry() -> &'static [&'static dyn NativeRule] {
    REGISTRY
}

pub fn lookup(id: &str) -> Option<&'static dyn NativeRule> {
    REGISTRY.iter().copied().find(|r| r.id() == id)
}

#[cfg(test)]
pub(crate) fn run(rule: &dyn NativeRule, language: Language, src: &str) -> Vec<NativeFinding> {
    let root = codelaw_lang::parse(language, src);
    let model = codelaw_semantic::build(language, &root, src);
    let options = BTreeMap::new();
    let ctx = NativeContext { source: src, language, semantic: model.as_ref(), options: &options };
    let mut out = Vec::new();
    for node in root.root().dfs() {
        if rule.kinds().contains(&node.kind().as_ref()) {
            rule.check(node, &ctx, &mut out);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn ids_are_unique_and_sorted() {
        let ids: Vec<&str> = super::registry().iter().map(|r| r.id()).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(ids, sorted);
    }
}
