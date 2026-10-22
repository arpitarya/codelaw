//! `deprecated: registry` (SR-BASELINE decision 6): every use of a
//! deprecated symbol, by identity from the graph — an aliased import, a
//! barrel re-export and `from m import f as g` are caught, a local function
//! that happens to share the name is not.

use crate::facts::{FactLang, ImportKind, Span};
use crate::link::Graph;
use crate::query::locate;
use std::collections::BTreeSet;

/// A symbol marked deprecated in code: JSDoc `@deprecated` on an export or
/// a PEP 702 `@deprecated(...)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Marked {
    /// `<path>#<name>` or `<module>:<name>`, as the registry would name it.
    pub symbol: String,
    pub module: String,
    pub name: String,
    /// The marker's message.
    pub message: String,
}

/// One use of a deprecated symbol.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeprecatedUse {
    pub file: String,
    pub span: Span,
    /// The symbol as the registry or the marker names it.
    pub symbol: String,
}

/// Where a registry symbol is declared in the project, or `None` when it
/// is an outside package's (found by its import specifier instead).
pub fn declared(graph: &Graph, symbol: &str) -> Option<(String, String)> {
    locate(graph, symbol).ok()
}

/// Every in-code deprecation marker, sorted by symbol.
pub fn marked(graph: &Graph) -> Vec<Marked> {
    let mut out = Vec::new();
    for m in graph.modules.values() {
        for d in &m.facts.decls {
            let Some(message) = &d.deprecated else { continue };
            let symbol = match m.facts.lang {
                Some(FactLang::Python) => format!("{}:{}", graph.resolver.py_module_name(&m.path).unwrap_or_else(|| m.path.clone()), d.name),
                _ => format!("{}#{}", m.path, d.name),
            };
            out.push(Marked { symbol, module: m.path.clone(), name: d.name.clone(), message: message.clone() });
        }
    }
    out.sort_by(|a, b| a.symbol.cmp(&b.symbol));
    out
}

/// Every use, from another module, of the declaration `name` of `module`.
pub fn uses_of_decl(graph: &Graph, module: &str, name: &str, symbol: &str) -> Vec<DeprecatedUse> {
    let mut seen: BTreeSet<(String, Span)> = BTreeSet::new();
    let mut out = Vec::new();
    for u in graph.uses_of(module, name) {
        // A re-export only passes the name on, and an entry point is not
        // code; the uses through them are counted where they are.
        if u.file == module || u.how.starts_with("re-export") || u.how == "entry point" || !graph.modules.contains_key(&u.file) {
            continue;
        }
        let spans = if u.refs.is_empty() { vec![u.span] } else { u.refs.clone() };
        for s in spans {
            if seen.insert((u.file.clone(), s)) {
                out.push(DeprecatedUse { file: u.file.clone(), span: s, symbol: symbol.to_owned() });
            }
        }
    }
    out.sort();
    out
}

/// The uses of a symbol of a package outside the project, by import
/// specifier: `pkg#name` (JS/TS) or `module:name` (Python).
pub fn uses_of_external(graph: &Graph, symbol: &str) -> Vec<DeprecatedUse> {
    let (source, name, python) = if let Some((s, n)) = symbol.rsplit_once('#') {
        (s, n, false)
    } else if let Some((s, n)) = symbol.split_once(':') {
        (s, n.split('.').next().unwrap_or(n), true)
    } else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for m in graph.modules.values() {
        let lang_ok = if python { m.facts.lang == Some(FactLang::Python) } else { m.facts.lang == Some(FactLang::Js) };
        if !lang_ok {
            continue;
        }
        for imp in m.facts.imports.iter().filter(|i| i.source == source) {
            let spans: Vec<Span> = match &imp.kind {
                ImportKind::Named(n) if n == name => {
                    if imp.uses.is_empty() {
                        vec![imp.span]
                    } else {
                        imp.uses.clone()
                    }
                }
                ImportKind::Namespace | ImportKind::Module | ImportKind::Default => {
                    imp.members.iter().flatten().filter(|x| x.name == name).map(|x| x.span).collect()
                }
                _ => Vec::new(),
            };
            out.extend(spans.into_iter().map(|span| DeprecatedUse { file: m.path.clone(), span, symbol: symbol.to_owned() }));
        }
    }
    out.sort();
    out.dedup();
    out
}
