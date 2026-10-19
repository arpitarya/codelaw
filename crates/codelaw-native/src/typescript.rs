//! Native rules behind `ts/recommended`, and the core rules
//! typescript-eslint switches on for TypeScript files (`no-var`,
//! `prefer-const`).

use crate::util::{children, named_children, text};
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::{Language, TextEdit};
use codelaw_lang::SyntaxNode;
use codelaw_semantic::SymbolKind;
use std::collections::BTreeMap;
use tree_sitter::Node;

const TS: &[Language] = &[Language::Typescript, Language::Tsx];
const JS: &[Language] = &[Language::Javascript, Language::Jsx, Language::Typescript, Language::Tsx];

fn finding(n: Node<'_>, message: impl Into<String>, edits: Vec<TextEdit>) -> NativeFinding {
    NativeFinding { start: n.start_byte(), end: n.end_byte(), message: message.into(), edits }
}

pub struct NoDuplicateEnumValues;

impl NativeRule for NoDuplicateEnumValues {
    fn id(&self) -> &'static str {
        "ts/no-duplicate-enum-values"
    }
    fn languages(&self) -> &'static [Language] {
        TS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["enum_body"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let mut seen: BTreeMap<String, ()> = BTreeMap::new();
        for m in named_children(node.get_inner_node()) {
            if m.kind() != "enum_assignment" {
                continue;
            }
            let Some(v) = m.child_by_field_name("value") else { continue };
            if !matches!(v.kind(), "number" | "string") {
                continue;
            }
            let key = if v.kind() == "string" { format!("s:{}", crate::util::string_value(v, ctx.source)) } else { format!("n:{}", text(v, ctx.source)) };
            if seen.insert(key, ()).is_some() {
                out.push(finding(v, format!("Duplicate enum value `{}`", text(v, ctx.source)), Vec::new()));
            }
        }
    }
}

pub struct NoUnsafeDeclarationMerging;

impl NativeRule for NoUnsafeDeclarationMerging {
    fn id(&self) -> &'static str {
        "ts/no-unsafe-declaration-merging"
    }
    fn languages(&self) -> &'static [Language] {
        TS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["program"]
    }
    fn needs_semantic(&self) -> bool {
        true
    }
    fn check(&self, _node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let Some(model) = ctx.semantic else { return };
        for scope in &model.scopes {
            for ids in scope.bindings.values() {
                let class = ids.iter().find(|&&i| model.symbol(i).kind == SymbolKind::Class);
                let iface = ids.iter().find(|&&i| model.symbol(i).kind == SymbolKind::Interface);
                if let (Some(&_c), Some(&i)) = (class, iface) {
                    let s = model.symbol(i);
                    let (start, end) = s.decls[0];
                    out.push(NativeFinding {
                        start,
                        end,
                        message: format!("The interface `{}` merges with a class of the same name, which the compiler can't check", s.name),
                        edits: Vec::new(),
                    });
                }
            }
        }
    }
}

pub struct NoUnusedExpressions {
    pub id: &'static str,
    pub languages: &'static [Language],
}

/// Whether an expression statement's expression does something.
fn has_effect(n: Node<'_>) -> bool {
    match n.kind() {
        "call_expression" | "new_expression" | "assignment_expression" | "augmented_assignment_expression" | "update_expression" | "await_expression"
        | "yield_expression" | "import" => true,
        "unary_expression" => matches!(n.child(0).map(|o| o.kind()), Some("delete") | Some("void")),
        // `x satisfies T;` is a compile-time type test.
        "satisfies_expression" => true,
        "parenthesized_expression" | "non_null_expression" | "as_expression" => n.named_child(0).is_some_and(|c| has_effect(c)),
        "sequence_expression" => named_children(n).last().is_some_and(|c| has_effect(*c)),
        _ => false,
    }
}

/// A string-literal statement at the start of a program or function body.
fn directive(stmt: Node<'_>) -> bool {
    let Some(parent) = stmt.parent() else { return false };
    if !matches!(parent.kind(), "program" | "statement_block") {
        return false;
    }
    if parent.kind() == "statement_block" && !parent.parent().is_some_and(|p| crate::util::is_function(p.kind()) || p.kind() == "arrow_function") {
        return false;
    }
    for s in named_children(parent) {
        if s.kind() == "comment" {
            continue;
        }
        let is_string = s.kind() == "expression_statement" && s.named_child(0).is_some_and(|e| e.kind() == "string");
        if !is_string {
            return false;
        }
        if s.id() == stmt.id() {
            return true;
        }
    }
    false
}

impl NativeRule for NoUnusedExpressions {
    fn id(&self) -> &'static str {
        self.id
    }
    fn languages(&self) -> &'static [Language] {
        self.languages
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["expression_statement"]
    }
    fn check(&self, node: SyntaxNode<'_>, _ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let stmt = node.get_inner_node();
        let Some(e) = stmt.named_child(0) else { return };
        if has_effect(e) || directive(stmt) || stmt.has_error() {
            return;
        }
        out.push(finding(e, "This expression's value is unused, so the statement does nothing", Vec::new()));
    }
}

pub struct NoWrapperObjectTypes;

const WRAPPERS: &[(&str, &str)] = &[("String", "string"), ("Number", "number"), ("Boolean", "boolean"), ("Symbol", "symbol"), ("BigInt", "bigint"), ("Object", "object")];

impl NativeRule for NoWrapperObjectTypes {
    fn id(&self) -> &'static str {
        "ts/no-wrapper-object-types"
    }
    fn languages(&self) -> &'static [Language] {
        TS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["type_identifier"]
    }
    fn needs_semantic(&self) -> bool {
        true
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        let name = text(n, ctx.source);
        let Some((_, lower)) = WRAPPERS.iter().find(|(w, _)| *w == name) else { return };
        // Only the built-in: a class or interface of that name is the file's own.
        if ctx.semantic.and_then(|m| m.reference_at(n.start_byte())).is_none_or(|r| r.symbol.is_some()) {
            return;
        }
        // `class A implements Object` and `extends` clauses name values too.
        if n.parent().is_some_and(|p| matches!(p.kind(), "implements_clause" | "extends_type_clause")) {
            return;
        }
        out.push(finding(n, format!("Use the primitive type `{lower}`, not the wrapper object type `{name}`"), vec![TextEdit::new(n.start_byte(), n.end_byte(), *lower)]));
    }
}

pub struct PreferAsConst;

impl NativeRule for PreferAsConst {
    fn id(&self) -> &'static str {
        "ts/prefer-as-const"
    }
    fn languages(&self) -> &'static [Language] {
        TS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["as_expression", "type_assertion"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        let parts = named_children(n);
        let (value, ty) = if n.kind() == "as_expression" {
            (parts.first().copied(), parts.get(1).copied())
        } else {
            // `<"a">"a"`: type arguments first, then the value.
            (parts.get(1).copied(), parts.first().and_then(|t| t.named_child(0)))
        };
        let (Some(value), Some(ty)) = (value, ty) else { return };
        if ty.kind() != "literal_type" || !matches!(value.kind(), "string" | "number") || text(ty, ctx.source) != text(value, ctx.source) {
            return;
        }
        let edit = TextEdit::new(ty.start_byte(), ty.end_byte(), "const");
        out.push(finding(ty, "Use `as const` instead of repeating the literal as its type", vec![edit]));
    }
}

/// The `let`/`var`/`const` keyword token of a declaration.
fn keyword<'t>(decl: Node<'t>) -> Option<Node<'t>> {
    children(decl).into_iter().find(|c| matches!(c.kind(), "let" | "var" | "const"))
}

pub struct NoVar;

impl NativeRule for NoVar {
    fn id(&self) -> &'static str {
        "js/no-var"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["variable_declaration"]
    }
    fn check(&self, node: SyntaxNode<'_>, _ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        if crate::util::in_ambient(n) {
            return;
        }
        let Some(kw) = keyword(n) else { return };
        out.push(finding(kw, "Use `let` or `const` instead of `var`", vec![TextEdit::new(kw.start_byte(), kw.end_byte(), "let")]));
    }
}

pub struct PreferConst;

impl NativeRule for PreferConst {
    fn id(&self) -> &'static str {
        "js/prefer-const"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["lexical_declaration", "for_in_statement"]
    }
    fn needs_semantic(&self) -> bool {
        true
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let Some(model) = ctx.semantic else { return };
        let n = node.get_inner_node();
        let src = ctx.source;
        // The names a declaration binds and whether each has a value now.
        let (kw, names): (Option<Node>, Vec<(Node, bool)>) = match n.kind() {
            "lexical_declaration" => {
                if n.parent().is_some_and(|p| p.kind() == "for_statement") {
                    return;
                }
                let mut names = Vec::new();
                for d in named_children(n).into_iter().filter(|d| d.kind() == "variable_declarator") {
                    let has_value = d.child_by_field_name("value").is_some();
                    let mut ids = Vec::new();
                    if let Some(name) = d.child_by_field_name("name") {
                        collect_ids(name, &mut ids);
                    }
                    names.extend(ids.into_iter().map(|i| (i, has_value)));
                }
                (keyword(n), names)
            }
            _ => {
                let kind = n.child_by_field_name("kind");
                if kind.is_none_or(|k| text(k, src) != "let") {
                    return;
                }
                let mut ids = Vec::new();
                if let Some(left) = n.child_by_field_name("left") {
                    collect_ids(left, &mut ids);
                }
                (kind, ids.into_iter().map(|i| (i, true)).collect())
            }
        };
        let Some(kw) = kw.filter(|k| text(*k, src) == "let") else { return };
        if names.is_empty() {
            return;
        }
        // Every name must have its value now and never be written again
        // (ESLint's `destructuring: "any"` is not followed: all must be const).
        let all_const = names.iter().all(|(ident, has_value)| {
            *has_value && model.declared_at(ident.start_byte()).is_some_and(|s| !model.is_reassigned(s))
        });
        if all_const {
            let shown: Vec<&str> = names.iter().map(|(i, _)| text(*i, src)).collect();
            out.push(finding(kw, format!("`{}` is never reassigned; declare it with `const`", shown.join("`, `")), vec![TextEdit::new(kw.start_byte(), kw.end_byte(), "const")]));
        }
    }
}

fn collect_ids<'t>(n: Node<'t>, out: &mut Vec<Node<'t>>) {
    match n.kind() {
        "identifier" | "shorthand_property_identifier_pattern" => out.push(n),
        "pair_pattern" => {
            if let Some(v) = n.child_by_field_name("value") {
                collect_ids(v, out);
            }
        }
        "assignment_pattern" | "object_assignment_pattern" => {
            if let Some(l) = n.child_by_field_name("left") {
                collect_ids(l, out);
            }
        }
        "type_annotation" | "comment" => {}
        _ => named_children(n).into_iter().for_each(|c| collect_ids(c, out)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(rule: &dyn NativeRule, src: &str) -> Vec<String> {
        crate::run(rule, Language::Typescript, src).into_iter().map(|f| format!("{} @{}", f.message, &src[f.start..f.end])).collect()
    }

    #[test]
    fn enums_merging_wrappers() {
        assert_eq!(found(&NoDuplicateEnumValues, "enum E { A = 1, B = 2, C = 1, D = 'x', F = 'x', G }").len(), 2);
        assert_eq!(found(&NoUnsafeDeclarationMerging, "interface Foo { a: 1 }\nclass Foo {}\ninterface Bar {}\n").len(), 1);
        let src = "let a: String = 'x';\nlet b: number;\nclass Number2 {}\ninterface Object { x: 1 }\nlet c: Object;";
        assert_eq!(found(&NoWrapperObjectTypes, src), ["Use the primitive type `string`, not the wrapper object type `String` @String"]);
    }

    #[test]
    fn unused_expressions() {
        let src = "'use strict';\na;\nb.c;\nx satisfies T;\nf();\nx = 1;\nvoid g();\n(0, h)();\nnew K();\nfunction q() { 'use strict'; 'also a directive'; f(); 'too late'; }\n";
        assert_eq!(
            crate::run(&NoUnusedExpressions { id: "ts/no-unused-expressions", languages: TS }, Language::Typescript, src).into_iter().map(|f| src[f.start..f.end].to_owned()).collect::<Vec<_>>(),
            ["a", "b.c", "'too late'"]
        );
    }

    #[test]
    fn as_const_var_prefer_const() {
        assert_eq!(found(&PreferAsConst, "let a = 'x' as 'x'; let b = 1 as 1; let c = 'x' as string; let d = <'y'>'y';").len(), 3);
        assert_eq!(found(&NoVar, "var a = 1; let b = 2; declare var c: number;").len(), 1);
        let src = "let a = 1;\nlet b = 2; b++;\nlet c;\nc = 3;\nlet { d, e } = o; e = 4;\nlet { f, g } = o;\nfor (let h of xs) {}\nfor (let i of xs) { i = 1; }\nfor (let j = 0; j < 1; j++) {}\nuse(a, b, c, d, f, g);";
        assert_eq!(found(&PreferConst, src), [
            "`a` is never reassigned; declare it with `const` @let",
            "`f`, `g` is never reassigned; declare it with `const` @let",
            "`h` is never reassigned; declare it with `const` @let",
        ]);
    }
}
