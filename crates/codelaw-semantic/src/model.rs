use std::collections::{BTreeMap, BTreeSet, HashMap};

pub type ScopeId = usize;
pub type SymbolId = usize;
pub type RefId = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeKind {
    Module,
    Function,
    Class,
    Block,
    Catch,
    For,
    Comprehension,
    TypeParams,
}

#[derive(Clone, Debug)]
pub struct Scope {
    pub parent: Option<ScopeId>,
    pub kind: ScopeKind,
    pub start: usize,
    pub end: usize,
    /// Name to the symbols of that name declared here (a value and a type
    /// of one name are two symbols in TypeScript).
    pub bindings: BTreeMap<String, Vec<SymbolId>>,
    /// Python `global` / `nonlocal` names: assignments to them bind outside.
    pub redirects: BTreeMap<String, Redirect>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Redirect {
    Global,
    Nonlocal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolKind {
    Var,
    Let,
    Const,
    Function,
    Class,
    Param,
    Import,
    CatchParam,
    Interface,
    TypeAlias,
    Enum,
    Namespace,
    TypeParam,
    /// A Python name bound by assignment, `for`, `with`, `except`, …
    Variable,
}

impl SymbolKind {
    /// Whether a value reference may resolve to it.
    pub fn is_value(self) -> bool {
        !matches!(self, SymbolKind::Interface | SymbolKind::TypeAlias | SymbolKind::TypeParam)
    }

    /// Whether a type reference may resolve to it.
    pub fn is_type(self) -> bool {
        matches!(
            self,
            SymbolKind::Interface
                | SymbolKind::TypeAlias
                | SymbolKind::TypeParam
                | SymbolKind::Class
                | SymbolKind::Enum
                | SymbolKind::Import
                | SymbolKind::Namespace
        )
    }

    pub fn name(self) -> &'static str {
        match self {
            SymbolKind::Var => "var",
            SymbolKind::Let => "let",
            SymbolKind::Const => "const",
            SymbolKind::Function => "function",
            SymbolKind::Class => "class",
            SymbolKind::Param => "param",
            SymbolKind::Import => "import",
            SymbolKind::CatchParam => "catch-param",
            SymbolKind::Interface => "interface",
            SymbolKind::TypeAlias => "type-alias",
            SymbolKind::Enum => "enum",
            SymbolKind::Namespace => "namespace",
            SymbolKind::TypeParam => "type-param",
            SymbolKind::Variable => "variable",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub scope: ScopeId,
    /// Byte ranges of the declaring identifiers; more than one when a name
    /// is declared again (`var`, overloads, Python rebinding).
    pub decls: Vec<(usize, usize)>,
    pub refs: Vec<RefId>,
    /// Part of the module's public surface.
    pub exported: bool,
    /// Used by construction even without a reference (a TypeScript
    /// parameter property, a `__future__` import).
    pub implicitly_used: bool,
}

#[derive(Clone, Debug)]
pub struct Reference {
    pub name: String,
    pub start: usize,
    pub end: usize,
    pub scope: ScopeId,
    pub symbol: Option<SymbolId>,
    pub read: bool,
    pub write: bool,
    /// A TypeScript type position.
    pub type_only: bool,
    /// Either namespace may satisfy it (`export { Name }`).
    pub any_namespace: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Imported {
    /// `import x from`.
    Default,
    /// `import * as x from`.
    Namespace,
    /// `import { a as x } from`, `from m import a as x`.
    Named(String),
    /// `import m` / `import a.b` (Python).
    Module,
    /// `from m import *`.
    Star,
    /// `import "m"`.
    SideEffect,
}

#[derive(Clone, Debug)]
pub struct Import {
    pub source: String,
    pub imported: Imported,
    pub local: Option<SymbolId>,
    pub start: usize,
    pub end: usize,
    pub type_only: bool,
}

#[derive(Clone, Debug)]
pub struct Export {
    /// The exported name: `default`, `*` for `export * from`, `=` for
    /// `export =`.
    pub name: String,
    pub local: Option<SymbolId>,
    pub source: Option<String>,
    pub start: usize,
    pub end: usize,
    pub type_only: bool,
}

/// The semantic model of one file.
#[derive(Clone, Debug, Default)]
pub struct SemanticModel {
    pub scopes: Vec<Scope>,
    pub symbols: Vec<Symbol>,
    pub references: Vec<Reference>,
    pub imports: Vec<Import>,
    pub exports: Vec<Export>,
    /// A Python `from m import *`: any name may be defined.
    pub has_star_import: bool,
    pub(crate) python: bool,
    /// Built for TypeScript or TSX.
    pub typescript: bool,
    pub(crate) decl_at: HashMap<usize, SymbolId>,
    pub(crate) ref_at: HashMap<usize, RefId>,
}

impl SemanticModel {
    /// The symbol whose declaring identifier starts at `start`.
    pub fn declared_at(&self, start: usize) -> Option<SymbolId> {
        self.decl_at.get(&start).copied()
    }

    /// The reference whose identifier starts at `start`.
    pub fn reference_at(&self, start: usize) -> Option<&Reference> {
        self.ref_at.get(&start).map(|&r| &self.references[r])
    }

    /// The symbol an identifier names, whether it declares or references it.
    pub fn symbol_at(&self, start: usize) -> Option<SymbolId> {
        self.declared_at(start).or_else(|| self.reference_at(start).and_then(|r| r.symbol))
    }

    pub fn symbol(&self, id: SymbolId) -> &Symbol {
        &self.symbols[id]
    }

    /// The `unused` predicate: declared, never read, not exported.
    pub fn is_unused(&self, id: SymbolId) -> bool {
        let s = &self.symbols[id];
        if s.exported || s.implicitly_used {
            return false;
        }
        // Python module-level names other than imports are the module's
        // surface: importable from elsewhere.
        if self.python && self.scopes[s.scope].kind == ScopeKind::Module && s.kind != SymbolKind::Import {
            return false;
        }
        !s.refs.iter().any(|&r| self.references[r].read)
    }

    /// The `reassigned` predicate: written after its declaration.
    pub fn is_reassigned(&self, id: SymbolId) -> bool {
        let s = &self.symbols[id];
        s.decls.len() > 1 || s.refs.iter().any(|&r| self.references[r].write)
    }

    /// The `shadowed` predicate: an enclosing scope binds the same name.
    /// With `builtins`, a language builtin counts too.
    pub fn shadowed(&self, id: SymbolId, builtins: bool) -> bool {
        let s = &self.symbols[id];
        let mut scope = self.scopes[s.scope].parent;
        while let Some(sc) = scope {
            let skip_class = self.python && self.scopes[sc].kind == ScopeKind::Class;
            if !skip_class
                && let Some(ids) = self.scopes[sc].bindings.get(&s.name)
                && ids.iter().any(|&o| self.symbols[o].kind.is_value() == s.kind.is_value())
            {
                return true;
            }
            scope = self.scopes[sc].parent;
        }
        builtins && self.is_builtin(&s.name, false)
    }

    fn is_builtin(&self, name: &str, type_only: bool) -> bool {
        if self.python {
            crate::globals::PYTHON_BUILTINS.contains(&name)
        } else if type_only {
            crate::globals::TS_GLOBAL_TYPES.contains(&name) || crate::globals::ES_GLOBALS.contains(&name)
        } else {
            crate::globals::ES_GLOBALS.contains(&name)
        }
    }

    /// The `undeclared` predicate: a reference that resolves to nothing the
    /// file declares, the language defines, or `extra_globals` names.
    pub fn is_undeclared(&self, reference: &Reference, extra_globals: &dyn Fn(&str) -> bool) -> bool {
        if reference.symbol.is_some() || self.has_star_import {
            return false;
        }
        !self.is_builtin(&reference.name, reference.type_only) && !extra_globals(&reference.name)
    }

    /// The import a symbol was bound by, if any.
    pub fn import_of(&self, id: SymbolId) -> Option<&Import> {
        self.imports.iter().find(|i| i.local == Some(id))
    }

    /// The `imported-from` predicate.
    pub fn imported_from(&self, id: SymbolId, module: &str, name: Option<&str>) -> bool {
        self.import_of(id).is_some_and(|i| {
            i.source == module
                && match name {
                    None => true,
                    Some(n) => match &i.imported {
                        Imported::Named(x) => x == n,
                        Imported::Default => n == "default",
                        _ => false,
                    },
                }
        })
    }

    /// The names the module exports (the graph's per-file facts).
    pub fn exported_names(&self) -> BTreeSet<String> {
        self.exports.iter().map(|e| e.name.clone()).collect()
    }

    /// The innermost scope containing a byte offset.
    pub fn scope_at(&self, offset: usize) -> ScopeId {
        let mut best = 0;
        for (i, s) in self.scopes.iter().enumerate() {
            if s.start <= offset && offset < s.end.max(s.start + 1) && s.end - s.start <= self.scopes[best].end - self.scopes[best].start {
                best = i;
            }
        }
        best
    }

    pub(crate) fn push_scope(&mut self, parent: Option<ScopeId>, kind: ScopeKind, start: usize, end: usize) -> ScopeId {
        self.scopes.push(Scope { parent, kind, start, end, bindings: BTreeMap::new(), redirects: BTreeMap::new() });
        self.scopes.len() - 1
    }

    /// Declare `name` in `scope`, merging with an earlier declaration of
    /// the same name and namespace.
    pub(crate) fn declare(&mut self, scope: ScopeId, name: &str, kind: SymbolKind, start: usize, end: usize) -> SymbolId {
        let existing = self.scopes[scope]
            .bindings
            .get(name)
            .and_then(|ids| ids.iter().copied().find(|&id| self.symbols[id].kind.is_value() == kind.is_value() || self.python));
        let id = match existing {
            Some(id) => {
                self.symbols[id].decls.push((start, end));
                id
            }
            None => {
                self.symbols.push(Symbol {
                    name: name.to_owned(),
                    kind,
                    scope,
                    decls: vec![(start, end)],
                    refs: Vec::new(),
                    exported: false,
                    implicitly_used: false,
                });
                let id = self.symbols.len() - 1;
                self.scopes[scope].bindings.entry(name.to_owned()).or_default().push(id);
                id
            }
        };
        self.decl_at.insert(start, id);
        id
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn add_ref(&mut self, scope: ScopeId, name: &str, start: usize, end: usize, read: bool, write: bool, type_only: bool) -> RefId {
        self.references.push(Reference { name: name.to_owned(), start, end, scope, symbol: None, read, write, type_only, any_namespace: false });
        let id = self.references.len() - 1;
        self.ref_at.insert(start, id);
        id
    }

    /// Resolve every reference up its scope chain. Python skips class
    /// scopes other than the reference's own (LEGB).
    pub(crate) fn resolve(&mut self) {
        for r in 0..self.references.len() {
            let (name, mut scope, type_only, any, start) = {
                let r = &self.references[r];
                (r.name.clone(), Some(r.scope), r.type_only, r.any_namespace, r.scope)
            };
            let mut found = None;
            while let Some(sc) = scope {
                let s = &self.scopes[sc];
                let visible = !(self.python && s.kind == ScopeKind::Class && sc != start);
                if visible && let Some(ids) = s.bindings.get(&name) {
                    let pick = ids
                        .iter()
                        .copied()
                        .find(|&id| if type_only { self.symbols[id].kind.is_type() } else { self.symbols[id].kind.is_value() })
                        .or_else(|| ids.first().copied().filter(|_| self.python || any));
                    if pick.is_some() {
                        found = pick;
                        break;
                    }
                }
                scope = s.parent;
            }
            self.references[r].symbol = found;
            if let Some(id) = found {
                self.symbols[id].refs.push(r);
            }
        }
    }
}
