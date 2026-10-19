//! Scope analysis for JavaScript, TypeScript, JSX and TSX, over the
//! tree-sitter tree. The approach follows `oxc_semantic`: declare while
//! walking (with `var` and function hoisting), then resolve every
//! reference up its scope chain once the walk is done.

use crate::model::{Export, Import, Imported, ScopeId, ScopeKind, SemanticModel, SymbolKind};
use tree_sitter::Node;

pub(crate) fn build(root: Node<'_>, src: &str, typescript: bool) -> SemanticModel {
    let mut b = Builder { src, m: SemanticModel { typescript, ..Default::default() }, pending_exports: Vec::new() };
    let module = b.m.push_scope(None, ScopeKind::Module, root.start_byte(), root.end_byte());
    b.children(root, module);
    b.m.resolve();
    for (export, reference) in b.pending_exports {
        let local = reference.and_then(|r| b.m.references[r].symbol);
        if let Some(id) = local {
            b.m.symbols[id].exported = true;
        }
        b.m.exports.push(Export { local, ..export });
    }
    b.m.exports.sort_by_key(|e| e.start);
    b.m
}

struct Builder<'s> {
    src: &'s str,
    m: SemanticModel,
    /// Exports whose local symbol is known only after resolution.
    pending_exports: Vec<(Export, Option<usize>)>,
}

fn named_children<'t>(n: Node<'t>) -> Vec<Node<'t>> {
    let mut c = n.walk();
    n.named_children(&mut c).collect()
}

fn children<'t>(n: Node<'t>) -> Vec<Node<'t>> {
    let mut c = n.walk();
    n.children(&mut c).collect()
}

fn has_token(n: Node<'_>, token: &str) -> bool {
    children(n).iter().any(|c| !c.is_named() && c.kind() == token)
}

impl<'s> Builder<'s> {
    fn text(&self, n: Node<'_>) -> &'s str {
        &self.src[n.byte_range()]
    }

    fn hoist_scope(&self, mut scope: ScopeId) -> ScopeId {
        loop {
            let s = &self.m.scopes[scope];
            match (s.kind, s.parent) {
                (ScopeKind::Function | ScopeKind::Module, _) | (_, None) => return scope,
                (_, Some(p)) => scope = p,
            }
        }
    }

    fn declare(&mut self, scope: ScopeId, name: Node<'_>, kind: SymbolKind) -> usize {
        let text = self.text(name);
        self.m.declare(scope, text, kind, name.start_byte(), name.end_byte())
    }

    fn reference(&mut self, scope: ScopeId, n: Node<'_>, read: bool, write: bool, type_only: bool) -> usize {
        let text = self.text(n);
        self.m.add_ref(scope, text, n.start_byte(), n.end_byte(), read, write, type_only)
    }

    fn children(&mut self, n: Node<'_>, scope: ScopeId) {
        for c in named_children(n) {
            self.visit(c, scope);
        }
    }

    fn scope(&mut self, parent: ScopeId, kind: ScopeKind, n: Node<'_>) -> ScopeId {
        self.m.push_scope(Some(parent), kind, n.start_byte(), n.end_byte())
    }

    fn visit_field(&mut self, n: Node<'_>, field: &str, scope: ScopeId) {
        if let Some(c) = n.child_by_field_name(field) {
            self.visit(c, scope);
        }
    }

    fn visit(&mut self, n: Node<'_>, scope: ScopeId) {
        match n.kind() {
            "identifier" => {
                self.reference(scope, n, true, false, false);
            }
            "shorthand_property_identifier" => {
                self.reference(scope, n, true, false, false);
            }
            "type_identifier" => {
                self.reference(scope, n, true, false, true);
            }
            "nested_type_identifier" => self.visit_field(n, "module", scope),
            "nested_identifier" => {
                if let Some(first) = n.named_child(0) {
                    self.visit(first, scope);
                }
            }
            "property_identifier" | "private_property_identifier" | "statement_identifier" | "this" | "super"
            | "predefined_type" | "jsx_text" | "comment" | "hash_bang_line" | "html_comment" | "string" | "number"
            | "regex" | "true" | "false" | "null" | "undefined" | "meta_property" | "accessibility_modifier"
            | "override_modifier" | "break_statement" | "continue_statement" | "jsx_namespace_name" | "import" => {}
            "template_string" => {
                for c in named_children(n) {
                    if c.kind() == "template_substitution" {
                        self.children(c, scope);
                    }
                }
            }
            "member_expression" => self.visit_field(n, "object", scope),
            "pair" => {
                if let Some(k) = n.child_by_field_name("key")
                    && k.kind() == "computed_property_name"
                {
                    self.children(k, scope);
                }
                self.visit_field(n, "value", scope);
            }
            "labeled_statement" => self.visit_field(n, "body", scope),
            "variable_declaration" => {
                let target = self.hoist_scope(scope);
                self.declarators(n, scope, target, SymbolKind::Var);
            }
            "lexical_declaration" => {
                let kind = match n.child_by_field_name("kind").map(|k| self.text(k)) {
                    Some("const") => SymbolKind::Const,
                    _ => SymbolKind::Let,
                };
                self.declarators(n, scope, scope, kind);
            }
            "function_declaration" | "generator_function_declaration" | "function_signature" => {
                if let Some(name) = n.child_by_field_name("name") {
                    self.declare(scope, name, SymbolKind::Function);
                }
                self.function(n, scope, None);
            }
            "function_expression" | "function" | "generator_function" => {
                let inner = self.scope(scope, ScopeKind::Function, n);
                if let Some(name) = n.child_by_field_name("name") {
                    self.declare(inner, name, SymbolKind::Function);
                }
                self.function(n, scope, Some(inner));
            }
            "arrow_function" => {
                let inner = self.scope(scope, ScopeKind::Function, n);
                if let Some(p) = n.child_by_field_name("parameter") {
                    self.declare(inner, p, SymbolKind::Param);
                }
                self.function(n, scope, Some(inner));
            }
            "method_definition" | "method_signature" | "abstract_method_signature" => {
                for c in named_children(n) {
                    if c.kind() == "decorator" {
                        self.visit(c, scope);
                    }
                }
                if let Some(name) = n.child_by_field_name("name")
                    && name.kind() == "computed_property_name"
                {
                    self.children(name, scope);
                }
                self.function(n, scope, None);
            }
            "function_type" | "constructor_type" | "call_signature" | "construct_signature" => self.function(n, scope, None),
            "class_declaration" | "abstract_class_declaration" => {
                if let Some(name) = n.child_by_field_name("name") {
                    self.declare(scope, name, SymbolKind::Class);
                }
                let inner = self.scope(scope, ScopeKind::Class, n);
                self.class_rest(n, inner);
            }
            "class" => {
                let inner = self.scope(scope, ScopeKind::Class, n);
                if let Some(name) = n.child_by_field_name("name") {
                    self.declare(inner, name, SymbolKind::Class);
                }
                self.class_rest(n, inner);
            }
            "public_field_definition" | "field_definition" | "property_signature" => {
                for c in named_children(n) {
                    match c.kind() {
                        "computed_property_name" => self.children(c, scope),
                        "property_identifier" | "private_property_identifier" => {}
                        _ => self.visit(c, scope),
                    }
                }
            }
            "statement_block" | "switch_body" | "class_static_block" => {
                let inner = self.scope(scope, ScopeKind::Block, n);
                self.children(n, inner);
            }
            "for_statement" => {
                let inner = self.scope(scope, ScopeKind::For, n);
                self.children(n, inner);
            }
            "for_in_statement" => {
                let inner = self.scope(scope, ScopeKind::For, n);
                let kind = n.child_by_field_name("kind").map(|k| self.text(k));
                if let Some(left) = n.child_by_field_name("left") {
                    match kind {
                        Some("var") => {
                            let target = self.hoist_scope(scope);
                            self.pattern(left, target, SymbolKind::Var, scope);
                        }
                        Some("const") => self.pattern(left, inner, SymbolKind::Const, inner),
                        Some(_) => self.pattern(left, inner, SymbolKind::Let, inner),
                        None => self.assign_target(left, scope),
                    }
                }
                self.visit_field(n, "right", scope);
                self.visit_field(n, "body", inner);
            }
            "catch_clause" => {
                let inner = self.scope(scope, ScopeKind::Catch, n);
                if let Some(p) = n.child_by_field_name("parameter") {
                    self.pattern(p, inner, SymbolKind::CatchParam, inner);
                }
                self.visit_field(n, "type", inner);
                self.visit_field(n, "body", inner);
            }
            "assignment_expression" => {
                if let Some(left) = n.child_by_field_name("left") {
                    self.assign_target(left, scope);
                }
                self.visit_field(n, "right", scope);
            }
            "augmented_assignment_expression" => {
                match n.child_by_field_name("left") {
                    Some(l) if l.kind() == "identifier" => {
                        self.reference(scope, l, true, true, false);
                    }
                    Some(l) => self.visit(l, scope),
                    None => {}
                }
                self.visit_field(n, "right", scope);
            }
            "update_expression" => match n.child_by_field_name("argument") {
                Some(a) if a.kind() == "identifier" => {
                    self.reference(scope, a, true, true, false);
                }
                Some(a) => self.visit(a, scope),
                None => {}
            },
            "jsx_opening_element" | "jsx_self_closing_element" => {
                if let Some(name) = n.child_by_field_name("name") {
                    match name.kind() {
                        "identifier" => {
                            let t = self.text(name);
                            let intrinsic = t.starts_with(|c: char| c.is_ascii_lowercase()) || t.contains('-');
                            if !intrinsic {
                                self.reference(scope, name, true, false, false);
                            }
                        }
                        "member_expression" => self.visit(name, scope),
                        _ => {}
                    }
                }
                for c in named_children(n) {
                    if Some(c) != n.child_by_field_name("name") {
                        self.visit(c, scope);
                    }
                }
            }
            "jsx_closing_element" => {}
            "jsx_attribute" => {
                for c in named_children(n).into_iter().skip(1) {
                    self.visit(c, scope);
                }
            }
            "import_statement" => self.import(n),
            "export_statement" => self.export(n, scope),
            "import_alias" => {
                let parts = named_children(n);
                if let Some(first) = parts.first() {
                    let id = self.declare(scope, *first, SymbolKind::Import);
                    let source = parts.get(1).map(|p| self.text(*p).to_owned()).unwrap_or_default();
                    self.m.imports.push(Import {
                        source,
                        imported: Imported::Namespace,
                        local: Some(id),
                        start: n.start_byte(),
                        end: n.end_byte(),
                        type_only: false,
                    });
                }
                for p in parts.into_iter().skip(1) {
                    self.visit(p, scope);
                }
            }
            "interface_declaration" => {
                if let Some(name) = n.child_by_field_name("name") {
                    self.declare(scope, name, SymbolKind::Interface);
                }
                let inner = self.scope(scope, ScopeKind::TypeParams, n);
                for c in named_children(n) {
                    if Some(c) != n.child_by_field_name("name") {
                        self.visit(c, inner);
                    }
                }
            }
            "type_alias_declaration" => {
                if let Some(name) = n.child_by_field_name("name") {
                    self.declare(scope, name, SymbolKind::TypeAlias);
                }
                let inner = self.scope(scope, ScopeKind::TypeParams, n);
                self.visit_field(n, "type_parameters", inner);
                self.visit_field(n, "value", inner);
            }
            "enum_declaration" => {
                if let Some(name) = n.child_by_field_name("name") {
                    self.declare(scope, name, SymbolKind::Enum);
                }
                if let Some(body) = n.child_by_field_name("body") {
                    for m in named_children(body) {
                        if m.kind() == "enum_assignment" {
                            self.visit_field(m, "value", scope);
                        }
                    }
                }
            }
            "internal_module" | "module" => {
                if let Some(name) = n.child_by_field_name("name")
                    && matches!(name.kind(), "identifier" | "nested_identifier")
                {
                    let first = if name.kind() == "nested_identifier" { name.named_child(0).unwrap_or(name) } else { name };
                    self.declare(scope, first, SymbolKind::Namespace);
                }
                if let Some(body) = n.child_by_field_name("body") {
                    let inner = self.scope(scope, ScopeKind::Block, body);
                    self.children(body, inner);
                }
            }
            "type_parameters" => {
                for p in named_children(n) {
                    if p.kind() == "type_parameter" {
                        if let Some(name) = p.child_by_field_name("name") {
                            self.declare(scope, name, SymbolKind::TypeParam);
                        }
                        self.visit_field(p, "constraint", scope);
                        self.visit_field(p, "value", scope);
                    }
                }
            }
            "index_signature" => {
                for c in named_children(n) {
                    if Some(c) != n.child_by_field_name("name") {
                        self.visit(c, scope);
                    }
                }
            }
            "type_predicate" => {
                for c in named_children(n) {
                    self.visit(c, scope);
                }
            }
            _ => self.children(n, scope),
        }
    }

    fn declarators(&mut self, n: Node<'_>, scope: ScopeId, target: ScopeId, kind: SymbolKind) {
        for d in named_children(n) {
            if d.kind() != "variable_declarator" {
                self.visit(d, scope);
                continue;
            }
            if let Some(name) = d.child_by_field_name("name") {
                self.pattern(name, target, kind, scope);
            }
            self.visit_field(d, "type", scope);
            self.visit_field(d, "value", scope);
        }
    }

    /// Parameters, type parameters, return type and body of a function-like
    /// node. `inner` is the function scope if the caller made one.
    fn function(&mut self, n: Node<'_>, outer: ScopeId, inner: Option<ScopeId>) {
        let inner = inner.unwrap_or_else(|| self.scope(outer, ScopeKind::Function, n));
        self.visit_field(n, "type_parameters", inner);
        if let Some(params) = n.child_by_field_name("parameters") {
            for p in named_children(params) {
                self.param(p, inner);
            }
        }
        self.visit_field(n, "return_type", inner);
        if let Some(body) = n.child_by_field_name("body") {
            if body.kind() == "statement_block" {
                self.children(body, inner);
            } else {
                self.visit(body, inner);
            }
        }
    }

    fn param(&mut self, p: Node<'_>, scope: ScopeId) {
        match p.kind() {
            "required_parameter" | "optional_parameter" => {
                let property = named_children(p).iter().any(|c| matches!(c.kind(), "accessibility_modifier" | "override_modifier"))
                    || has_token(p, "readonly");
                for c in named_children(p) {
                    if c.kind() == "decorator" {
                        self.visit(c, scope);
                    }
                }
                if let Some(pat) = p.child_by_field_name("pattern") {
                    let before = self.m.symbols.len();
                    self.pattern(pat, scope, SymbolKind::Param, scope);
                    if property {
                        for s in &mut self.m.symbols[before..] {
                            s.implicitly_used = true;
                        }
                    }
                }
                self.visit_field(p, "type", scope);
                self.visit_field(p, "value", scope);
            }
            "this" | "comment" => {}
            _ => self.pattern(p, scope, SymbolKind::Param, scope),
        }
    }

    fn class_rest(&mut self, n: Node<'_>, inner: ScopeId) {
        for c in named_children(n) {
            match c.kind() {
                "type_identifier" | "identifier" if Some(c) == n.child_by_field_name("name") => {}
                _ => self.visit(c, inner),
            }
        }
    }

    /// Declare every name a binding pattern introduces. Default values are
    /// evaluated in `value_scope`.
    fn pattern(&mut self, n: Node<'_>, target: ScopeId, kind: SymbolKind, value_scope: ScopeId) {
        match n.kind() {
            "identifier" | "shorthand_property_identifier_pattern" => {
                self.declare(target, n, kind);
            }
            "object_pattern" | "array_pattern" => {
                for c in named_children(n) {
                    match c.kind() {
                        "pair_pattern" => {
                            if let Some(k) = c.child_by_field_name("key")
                                && k.kind() == "computed_property_name"
                            {
                                self.children(k, value_scope);
                            }
                            if let Some(v) = c.child_by_field_name("value") {
                                self.pattern(v, target, kind, value_scope);
                            }
                        }
                        "object_assignment_pattern" | "assignment_pattern" => {
                            if let Some(l) = c.child_by_field_name("left") {
                                self.pattern(l, target, kind, value_scope);
                            }
                            self.visit_field(c, "right", value_scope);
                        }
                        "comment" => {}
                        _ => self.pattern(c, target, kind, value_scope),
                    }
                }
            }
            "assignment_pattern" => {
                if let Some(l) = n.child_by_field_name("left") {
                    self.pattern(l, target, kind, value_scope);
                }
                self.visit_field(n, "right", value_scope);
            }
            "rest_pattern" => {
                for c in named_children(n) {
                    self.pattern(c, target, kind, value_scope);
                }
            }
            "required_parameter" | "optional_parameter" => self.param(n, target),
            _ => self.visit(n, value_scope),
        }
    }

    /// The left side of an assignment: identifiers are writes.
    fn assign_target(&mut self, n: Node<'_>, scope: ScopeId) {
        match n.kind() {
            "identifier" | "shorthand_property_identifier_pattern" => {
                self.reference(scope, n, false, true, false);
            }
            "object_pattern" | "array_pattern" => {
                for c in named_children(n) {
                    match c.kind() {
                        "pair_pattern" => {
                            if let Some(k) = c.child_by_field_name("key")
                                && k.kind() == "computed_property_name"
                            {
                                self.children(k, scope);
                            }
                            if let Some(v) = c.child_by_field_name("value") {
                                self.assign_target(v, scope);
                            }
                        }
                        "object_assignment_pattern" | "assignment_pattern" => {
                            if let Some(l) = c.child_by_field_name("left") {
                                self.assign_target(l, scope);
                            }
                            self.visit_field(c, "right", scope);
                        }
                        _ => self.assign_target(c, scope),
                    }
                }
            }
            "assignment_pattern" => {
                if let Some(l) = n.child_by_field_name("left") {
                    self.assign_target(l, scope);
                }
                self.visit_field(n, "right", scope);
            }
            "rest_pattern" | "parenthesized_expression" => {
                for c in named_children(n) {
                    self.assign_target(c, scope);
                }
            }
            _ => self.visit(n, scope),
        }
    }

    fn string_value(&self, n: Node<'_>) -> String {
        let t = self.text(n);
        t.trim_matches(|c| c == '"' || c == '\'' || c == '`').to_owned()
    }

    /// Imports bind in the module scope, which is always scope 0.
    fn import(&mut self, n: Node<'_>) {
        let module: ScopeId = 0;
        let type_stmt = has_token(n, "type");
        let source = n.child_by_field_name("source").map(|s| self.string_value(s)).unwrap_or_default();
        let mut bound = false;
        for c in named_children(n) {
            match c.kind() {
                "import_clause" => {
                    for part in named_children(c) {
                        match part.kind() {
                            "identifier" => {
                                let id = self.declare(module, part, SymbolKind::Import);
                                self.push_import(&source, Imported::Default, Some(id), part, type_stmt);
                                bound = true;
                            }
                            "namespace_import" => {
                                if let Some(name) = part.named_child(0) {
                                    let id = self.declare(module, name, SymbolKind::Import);
                                    self.push_import(&source, Imported::Namespace, Some(id), part, type_stmt);
                                    bound = true;
                                }
                            }
                            "named_imports" => {
                                for spec in named_children(part) {
                                    if spec.kind() != "import_specifier" {
                                        continue;
                                    }
                                    let Some(name) = spec.child_by_field_name("name") else { continue };
                                    let local = spec.child_by_field_name("alias").unwrap_or(name);
                                    let id = self.declare(module, local, SymbolKind::Import);
                                    let imported = self.string_value(name);
                                    let type_only = type_stmt || has_token(spec, "type");
                                    self.push_import(&source, Imported::Named(imported), Some(id), spec, type_only);
                                    bound = true;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                "import_require_clause" => {
                    if let Some(name) = c.named_child(0) {
                        let src = c.child_by_field_name("source").map(|s| self.string_value(s)).unwrap_or_default();
                        let id = self.declare(module, name, SymbolKind::Import);
                        self.push_import(&src, Imported::Default, Some(id), c, type_stmt);
                        bound = true;
                    }
                }
                _ => {}
            }
        }
        if !bound {
            self.push_import(&source, Imported::SideEffect, None, n, type_stmt);
        }
    }

    fn push_import(&mut self, source: &str, imported: Imported, local: Option<usize>, n: Node<'_>, type_only: bool) {
        self.m.imports.push(Import { source: source.to_owned(), imported, local, start: n.start_byte(), end: n.end_byte(), type_only });
    }

    fn export(&mut self, n: Node<'_>, scope: ScopeId) {
        let type_stmt = has_token(n, "type");
        let is_default = has_token(n, "default");
        let source = n.child_by_field_name("source").map(|s| self.string_value(s));
        let base = Export { name: String::new(), local: None, source: source.clone(), start: n.start_byte(), end: n.end_byte(), type_only: type_stmt };

        for c in named_children(n) {
            if c.kind() == "decorator" {
                self.visit(c, scope);
            }
        }
        if let Some(decl) = n.child_by_field_name("declaration") {
            self.visit(decl, scope);
            for name_node in declared_names(decl) {
                let Some(id) = self.m.declared_at(name_node.start_byte()) else { continue };
                self.m.symbols[id].exported = true;
                let name = if is_default { "default".to_owned() } else { self.text(name_node).to_owned() };
                let type_only = !self.m.symbols[id].kind.is_value();
                self.m.exports.push(Export { name, local: Some(id), type_only, ..base.clone() });
            }
            return;
        }
        if let Some(value) = n.child_by_field_name("value") {
            let reference = (value.kind() == "identifier").then_some(self.m.references.len());
            self.visit(value, scope);
            self.pending_exports.push((Export { name: "default".into(), ..base.clone() }, reference));
            return;
        }
        let mut any = false;
        for c in named_children(n) {
            match c.kind() {
                "export_clause" => {
                    for spec in named_children(c) {
                        if spec.kind() != "export_specifier" {
                            continue;
                        }
                        let Some(name) = spec.child_by_field_name("name") else { continue };
                        let exported = spec.child_by_field_name("alias").map(|a| self.string_value(a)).unwrap_or_else(|| self.string_value(name));
                        let type_only = type_stmt || has_token(spec, "type");
                        if source.is_some() {
                            self.m.exports.push(Export { name: exported, type_only, ..base.clone() });
                        } else {
                            let r = self.reference(scope, name, true, false, false);
                            self.m.references[r].any_namespace = true;
                            self.pending_exports.push((Export { name: exported, type_only, ..base.clone() }, Some(r)));
                        }
                        any = true;
                    }
                }
                "namespace_export" => {
                    let name = c.named_child(0).map(|x| self.string_value(x)).unwrap_or_else(|| "*".into());
                    self.m.exports.push(Export { name, ..base.clone() });
                    any = true;
                }
                "identifier" => {
                    // `export = x`
                    let r = self.reference(scope, c, true, false, false);
                    self.pending_exports.push((Export { name: "=".into(), ..base.clone() }, Some(r)));
                    any = true;
                }
                _ => {}
            }
        }
        if !any && source.is_some() {
            self.m.exports.push(Export { name: "*".into(), ..base });
        }
    }
}

/// The identifiers a declaration introduces at its own level.
pub fn declared_names(decl: Node<'_>) -> Vec<Node<'_>> {
    let mut out = Vec::new();
    match decl.kind() {
        "lexical_declaration" | "variable_declaration" => {
            for d in named_children(decl) {
                if let Some(name) = d.child_by_field_name("name") {
                    pattern_names(name, &mut out);
                }
            }
        }
        "internal_module" | "module" => {
            if let Some(name) = decl.child_by_field_name("name") {
                out.push(if name.kind() == "nested_identifier" { name.named_child(0).unwrap_or(name) } else { name });
            }
        }
        "ambient_declaration" => {
            for c in named_children(decl) {
                out.extend(declared_names(c));
            }
        }
        _ => {
            if let Some(name) = decl.child_by_field_name("name") {
                out.push(name);
            }
        }
    }
    out
}

fn pattern_names<'t>(n: Node<'t>, out: &mut Vec<Node<'t>>) {
    match n.kind() {
        "identifier" | "shorthand_property_identifier_pattern" => out.push(n),
        "pair_pattern" => {
            if let Some(v) = n.child_by_field_name("value") {
                pattern_names(v, out);
            }
        }
        "assignment_pattern" | "object_assignment_pattern" => {
            if let Some(l) = n.child_by_field_name("left") {
                pattern_names(l, out);
            }
        }
        _ => {
            for c in named_children(n) {
                pattern_names(c, out);
            }
        }
    }
}
