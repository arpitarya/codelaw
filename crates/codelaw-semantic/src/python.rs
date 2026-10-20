//! Scope analysis for Python, over the tree-sitter tree. The approach
//! follows Ruff's semantic model: a name bound anywhere in a function is
//! local to all of it, class scopes are invisible to nested functions, and
//! comprehensions have their own scope (LEGB).

use crate::model::{Export, Import, Imported, Redirect, ScopeId, ScopeKind, SemanticModel, SymbolKind};
use std::collections::BTreeSet;
use tree_sitter::Node;

const MODULE: ScopeId = 0;

pub(crate) fn build(root: Node<'_>, src: &str) -> SemanticModel {
    let mut b = Builder { src, m: SemanticModel { python: true, ..Default::default() }, all: None };
    b.m.push_scope(None, ScopeKind::Module, root.start_byte(), root.end_byte());
    b.children(root, MODULE);
    b.m.resolve();
    b.finish_exports();
    b.m
}

struct Builder<'s> {
    src: &'s str,
    m: SemanticModel,
    /// The names of a module-level `__all__`, if it has one.
    all: Option<Vec<String>>,
}

fn named_children<'t>(n: Node<'t>) -> Vec<Node<'t>> {
    let mut c = n.walk();
    n.named_children(&mut c).collect()
}

impl<'s> Builder<'s> {
    fn text(&self, n: Node<'_>) -> &'s str {
        &self.src[n.byte_range()]
    }

    fn scope(&mut self, parent: ScopeId, kind: ScopeKind, n: Node<'_>) -> ScopeId {
        self.m.push_scope(Some(parent), kind, n.start_byte(), n.end_byte())
    }

    fn children(&mut self, n: Node<'_>, scope: ScopeId) {
        for c in named_children(n) {
            self.visit(c, scope);
        }
    }

    fn visit_field(&mut self, n: Node<'_>, field: &str, scope: ScopeId) {
        if let Some(c) = n.child_by_field_name(field) {
            self.visit(c, scope);
        }
    }

    fn reference(&mut self, scope: ScopeId, n: Node<'_>, read: bool, write: bool) -> usize {
        let t = self.text(n);
        self.m.add_ref(scope, t, n.start_byte(), n.end_byte(), read, write, false)
    }

    /// Bind a name in `scope`, honouring `global` and `nonlocal`.
    fn bind(&mut self, scope: ScopeId, n: Node<'_>, kind: SymbolKind) -> Option<usize> {
        let name = self.text(n);
        match self.m.scopes[scope].redirects.get(name).copied() {
            Some(Redirect::Global) => Some(self.m.declare(MODULE, name, kind, n.start_byte(), n.end_byte())),
            Some(Redirect::Nonlocal) => {
                let parent = self.m.scopes[scope].parent.unwrap_or(MODULE);
                self.m.add_ref(parent, name, n.start_byte(), n.end_byte(), false, true, false);
                None
            }
            None => Some(self.m.declare(scope, name, kind, n.start_byte(), n.end_byte())),
        }
    }

    /// The nearest scope that is not a comprehension: where `:=` binds.
    fn non_comprehension(&self, mut scope: ScopeId) -> ScopeId {
        while self.m.scopes[scope].kind == ScopeKind::Comprehension {
            scope = self.m.scopes[scope].parent.unwrap_or(MODULE);
        }
        scope
    }

    fn visit(&mut self, n: Node<'_>, scope: ScopeId) {
        match n.kind() {
            "identifier" => {
                self.reference(scope, n, true, false);
            }
            "attribute" => self.visit_field(n, "object", scope),
            "keyword_argument" => self.visit_field(n, "value", scope),
            "comment" | "integer" | "float" | "true" | "false" | "none" | "ellipsis" | "keyword_separator"
            | "positional_separator" | "pass_statement" | "break_statement" | "continue_statement" | "escape_sequence" => {}
            "string" => {
                for c in named_children(n) {
                    if c.kind() == "interpolation" {
                        for e in named_children(c) {
                            match e.kind() {
                                "type_conversion" => {}
                                // `f"{x:{width}}"`: the nested fields are expressions.
                                "format_specifier" => {
                                    for f in named_children(e).into_iter().filter(|f| f.kind() == "format_expression") {
                                        if let Some(x) = f.child_by_field_name("expression") {
                                            self.visit(x, scope);
                                        }
                                    }
                                }
                                _ => self.visit(e, scope),
                            }
                        }
                    }
                }
            }
            "dotted_name" => {
                if let Some(first) = n.named_child(0) {
                    self.visit(first, scope);
                }
            }
            "function_definition" => {
                if let Some(name) = n.child_by_field_name("name") {
                    self.bind(scope, name, SymbolKind::Function);
                }
                let inner = self.scope(scope, ScopeKind::Function, n);
                self.visit_field(n, "type_parameters", inner);
                if let Some(params) = n.child_by_field_name("parameters") {
                    self.params(params, scope, inner);
                }
                self.visit_field(n, "return_type", scope);
                self.visit_field(n, "body", inner);
            }
            "lambda" => {
                let inner = self.scope(scope, ScopeKind::Function, n);
                if let Some(params) = n.child_by_field_name("parameters") {
                    self.params(params, scope, inner);
                }
                self.visit_field(n, "body", inner);
            }
            "class_definition" => {
                if let Some(name) = n.child_by_field_name("name") {
                    self.bind(scope, name, SymbolKind::Class);
                }
                self.visit_field(n, "superclasses", scope);
                let inner = self.scope(scope, ScopeKind::Class, n);
                self.visit_field(n, "type_parameters", inner);
                self.visit_field(n, "body", inner);
            }
            // `List[int]` in an annotation is a subscript, not a PEP 695
            // declaration: its names are references.
            "type_parameter" if n.parent().is_some_and(|p| p.kind() == "generic_type") => self.children(n, scope),
            "type_parameter" => {
                for c in named_children(n) {
                    match c.kind() {
                        "type" => match c.named_child(0) {
                            Some(id) if id.kind() == "identifier" => {
                                self.bind(scope, id, SymbolKind::TypeParam);
                            }
                            // `T: int` binds `T`; the bound is a reference.
                            Some(ct) if ct.kind() == "constrained_type" => {
                                for (i, part) in named_children(ct).into_iter().enumerate() {
                                    match part.named_child(0).filter(|x| i == 0 && x.kind() == "identifier") {
                                        Some(id) => {
                                            self.bind(scope, id, SymbolKind::TypeParam);
                                        }
                                        None => self.visit(part, scope),
                                    }
                                }
                            }
                            _ => self.visit(c, scope),
                        },
                        _ => self.visit(c, scope),
                    }
                }
            }
            "list_comprehension" | "set_comprehension" | "dictionary_comprehension" | "generator_expression" => {
                let inner = self.scope(scope, ScopeKind::Comprehension, n);
                let mut first = true;
                for c in named_children(n) {
                    match c.kind() {
                        "for_in_clause" => {
                            if let Some(left) = c.child_by_field_name("left") {
                                self.target(left, inner, SymbolKind::Variable);
                            }
                            // The first iterable is evaluated outside.
                            let iter_scope = if first { scope } else { inner };
                            first = false;
                            for r in named_children(c) {
                                if Some(r) != c.child_by_field_name("left") {
                                    self.visit(r, iter_scope);
                                }
                            }
                        }
                        _ => self.visit(c, inner),
                    }
                }
            }
            "assignment" => {
                let left = n.child_by_field_name("left");
                if scope == MODULE
                    && let Some(l) = left
                    && self.text(l) == "__all__"
                    && let Some(r) = n.child_by_field_name("right")
                {
                    self.all = Some(self.string_list(r));
                }
                if let Some(l) = left {
                    self.target(l, scope, SymbolKind::Variable);
                }
                self.visit_field(n, "type", scope);
                self.visit_field(n, "right", scope);
            }
            "augmented_assignment" => {
                if let Some(l) = n.child_by_field_name("left") {
                    if l.kind() == "identifier" {
                        if scope == MODULE
                            && self.text(l) == "__all__"
                            && let Some(r) = n.child_by_field_name("right")
                        {
                            let more = self.string_list(r);
                            self.all.get_or_insert_with(Vec::new).extend(more);
                        }
                        self.reference(scope, l, true, true);
                        if !self.m.scopes[scope].bindings.contains_key(self.text(l)) {
                            self.bind(scope, l, SymbolKind::Variable);
                        }
                    } else {
                        self.visit(l, scope);
                    }
                }
                self.visit_field(n, "right", scope);
            }
            "for_statement" => {
                if let Some(l) = n.child_by_field_name("left") {
                    self.target(l, scope, SymbolKind::Variable);
                }
                for c in named_children(n) {
                    if Some(c) != n.child_by_field_name("left") {
                        self.visit(c, scope);
                    }
                }
            }
            "as_pattern" => {
                let alias = n.child_by_field_name("alias");
                for c in named_children(n) {
                    if Some(c) == alias {
                        for t in named_children(c) {
                            self.target(t, scope, SymbolKind::Variable);
                        }
                    } else {
                        self.visit(c, scope);
                    }
                }
            }
            "named_expression" => {
                if let Some(name) = n.child_by_field_name("name") {
                    let target = self.non_comprehension(scope);
                    self.bind(target, name, SymbolKind::Variable);
                }
                self.visit_field(n, "value", scope);
            }
            "global_statement" | "nonlocal_statement" => {
                let redirect = if n.kind() == "global_statement" { Redirect::Global } else { Redirect::Nonlocal };
                for id in named_children(n) {
                    if id.kind() == "identifier" {
                        let name = self.text(id).to_owned();
                        self.m.scopes[scope].redirects.insert(name, redirect);
                    }
                }
            }
            "import_statement" => {
                for c in named_children(n) {
                    match c.kind() {
                        "dotted_name" => {
                            if let Some(first) = c.named_child(0) {
                                let source = self.text(c).to_owned();
                                let id = self.bind(scope, first, SymbolKind::Import);
                                self.push_import(source, Imported::Module, id, c);
                            }
                        }
                        "aliased_import" => {
                            if let (Some(name), Some(alias)) = (c.child_by_field_name("name"), c.child_by_field_name("alias")) {
                                let source = self.text(name).to_owned();
                                let id = self.bind(scope, alias, SymbolKind::Import);
                                if let Some(id) = id
                                    && self.text(name) == self.text(alias)
                                {
                                    self.m.symbols[id].exported = true;
                                }
                                self.push_import(source, Imported::Module, id, c);
                            }
                        }
                        _ => {}
                    }
                }
            }
            "import_from_statement" | "future_import_statement" => {
                let future = n.kind() == "future_import_statement";
                let source = if future {
                    "__future__".to_owned()
                } else {
                    n.child_by_field_name("module_name").map(|m| self.text(m).to_owned()).unwrap_or_default()
                };
                let module_name = n.child_by_field_name("module_name");
                for c in named_children(n) {
                    if Some(c) == module_name {
                        continue;
                    }
                    match c.kind() {
                        "dotted_name" => {
                            if let Some(name) = c.named_child(c.named_child_count().saturating_sub(1) as u32) {
                                let imported = self.text(c).to_owned();
                                let id = self.bind(scope, name, SymbolKind::Import);
                                if future && let Some(id) = id {
                                    self.m.symbols[id].implicitly_used = true;
                                }
                                self.push_import(source.clone(), Imported::Named(imported), id, c);
                            }
                        }
                        "aliased_import" => {
                            if let (Some(name), Some(alias)) = (c.child_by_field_name("name"), c.child_by_field_name("alias")) {
                                let imported = self.text(name).to_owned();
                                let id = self.bind(scope, alias, SymbolKind::Import);
                                if let Some(id) = id
                                    && imported == self.text(alias)
                                {
                                    self.m.symbols[id].exported = true;
                                }
                                self.push_import(source.clone(), Imported::Named(imported), id, c);
                            }
                        }
                        "wildcard_import" => {
                            self.m.has_star_import = true;
                            self.push_import(source.clone(), Imported::Star, None, c);
                        }
                        _ => {}
                    }
                }
            }
            "type_alias_statement" => {
                if let Some(left) = n.child_by_field_name("left")
                    && let Some(id) = left.named_child(0).filter(|i| i.kind() == "identifier")
                {
                    self.bind(scope, id, SymbolKind::TypeAlias);
                }
                self.visit_field(n, "right", scope);
            }
            "case_clause" => {
                for c in named_children(n) {
                    if c.kind() == "case_pattern" {
                        self.case_pattern(c, scope);
                    } else {
                        self.visit(c, scope);
                    }
                }
            }
            _ => self.children(n, scope),
        }
    }

    fn params(&mut self, params: Node<'_>, outer: ScopeId, inner: ScopeId) {
        for p in named_children(params) {
            match p.kind() {
                "identifier" => {
                    self.bind(inner, p, SymbolKind::Param);
                }
                "default_parameter" | "typed_default_parameter" => {
                    if let Some(name) = p.child_by_field_name("name") {
                        self.param_name(name, inner);
                    }
                    self.visit_field(p, "type", outer);
                    self.visit_field(p, "value", outer);
                }
                "typed_parameter" => {
                    for c in named_children(p) {
                        if Some(c) == p.child_by_field_name("type") {
                            self.visit(c, outer);
                        } else {
                            self.param_name(c, inner);
                        }
                    }
                }
                "list_splat_pattern" | "dictionary_splat_pattern" | "tuple_pattern" => self.param_name(p, inner),
                _ => {}
            }
        }
    }

    fn param_name(&mut self, n: Node<'_>, inner: ScopeId) {
        match n.kind() {
            "identifier" => {
                self.bind(inner, n, SymbolKind::Param);
            }
            _ => {
                for c in named_children(n) {
                    self.param_name(c, inner);
                }
            }
        }
    }

    /// An assignment target: identifiers bind; attributes and subscripts
    /// are reads of their object.
    fn target(&mut self, n: Node<'_>, scope: ScopeId, kind: SymbolKind) {
        match n.kind() {
            "identifier" => {
                self.bind(scope, n, kind);
            }
            "pattern_list" | "tuple_pattern" | "list_pattern" | "tuple" | "list" | "list_splat_pattern" | "list_splat"
            | "parenthesized_expression" => {
                for c in named_children(n) {
                    self.target(c, scope, kind);
                }
            }
            _ => self.visit(n, scope),
        }
    }

    fn case_pattern(&mut self, n: Node<'_>, scope: ScopeId) {
        match n.kind() {
            "dotted_name" => {
                let parts = named_children(n);
                if parts.len() == 1 {
                    if self.text(parts[0]) != "_" {
                        self.bind(scope, parts[0], SymbolKind::Variable);
                    }
                } else if let Some(first) = parts.first() {
                    self.reference(scope, *first, true, false);
                }
            }
            "class_pattern" => {
                let parts = named_children(n);
                for (i, c) in parts.into_iter().enumerate() {
                    if i == 0 && c.kind() == "dotted_name" {
                        if let Some(first) = c.named_child(0) {
                            self.reference(scope, first, true, false);
                        }
                    } else {
                        self.case_pattern(c, scope);
                    }
                }
            }
            "keyword_pattern" => {
                for c in named_children(n).into_iter().skip(1) {
                    self.case_pattern(c, scope);
                }
            }
            "as_pattern" => {
                for c in named_children(n) {
                    if c.kind() == "identifier" {
                        self.bind(scope, c, SymbolKind::Variable);
                    } else {
                        self.case_pattern(c, scope);
                    }
                }
            }
            "splat_pattern" => {
                for c in named_children(n) {
                    if c.kind() == "identifier" && self.text(c) != "_" {
                        self.bind(scope, c, SymbolKind::Variable);
                    }
                }
            }
            "dict_pattern" => {
                for c in named_children(n) {
                    if Some(c) == n.child_by_field_name("key") || c.kind() != "case_pattern" && c.kind() != "splat_pattern" {
                        self.visit(c, scope);
                    } else {
                        self.case_pattern(c, scope);
                    }
                }
            }
            "string" | "integer" | "float" | "true" | "false" | "none" | "concatenated_string" => {}
            _ => {
                for c in named_children(n) {
                    self.case_pattern(c, scope);
                }
            }
        }
    }

    fn string_list(&self, n: Node<'_>) -> Vec<String> {
        let mut out = Vec::new();
        if matches!(n.kind(), "list" | "tuple") {
            for s in named_children(n) {
                if s.kind() == "string" {
                    let content: String =
                        named_children(s).into_iter().filter(|c| c.kind() == "string_content").map(|c| self.text(c)).collect();
                    out.push(content);
                }
            }
        }
        out
    }

    fn push_import(&mut self, source: String, imported: Imported, local: Option<usize>, n: Node<'_>) {
        self.m.imports.push(Import { source, imported, local, start: n.start_byte(), end: n.end_byte(), type_only: false });
    }

    /// `__all__` if present; otherwise every public module-level name.
    fn finish_exports(&mut self) {
        let module: Vec<(String, Vec<usize>)> =
            self.m.scopes[MODULE].bindings.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        let listed: Option<BTreeSet<String>> = self.all.as_ref().map(|a| a.iter().cloned().collect());
        for (name, ids) in module {
            for id in ids {
                let s = &self.m.symbols[id];
                let export = match &listed {
                    Some(all) => all.contains(&name),
                    None => !name.starts_with('_') && (s.kind != SymbolKind::Import || s.exported),
                };
                if export {
                    self.m.symbols[id].exported = true;
                    let (start, end) = self.m.symbols[id].decls[0];
                    self.m.exports.push(Export { name: name.clone(), local: Some(id), source: None, start, end, type_only: false });
                }
            }
        }
        if let Some(all) = &listed {
            for name in all {
                if !self.m.exports.iter().any(|e| &e.name == name) {
                    self.m.exports.push(Export { name: name.clone(), local: None, source: None, start: 0, end: 0, type_only: false });
                }
            }
        }
        self.m.exports.sort_by(|a, b| a.start.cmp(&b.start).then_with(|| a.name.cmp(&b.name)));
    }
}
