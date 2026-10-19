//! Duplicates among siblings, which a pattern can't count: object keys,
//! class members, parameters, `case` values and `else if` conditions.

use crate::util::{children, has_token, named_children, property_name, text};
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::Language;
use codelaw_lang::SyntaxNode;
use std::collections::BTreeMap;

const JS: &[Language] = &[Language::Javascript, Language::Jsx, Language::Typescript, Language::Tsx];

fn finding(n: tree_sitter::Node<'_>, message: String) -> NativeFinding {
    NativeFinding { start: n.start_byte(), end: n.end_byte(), message, edits: Vec::new() }
}

/// An expression's text without whitespace, for comparing; and as shown.
fn normalised(n: tree_sitter::Node<'_>, src: &str) -> (String, String) {
    let t = text(n, src);
    (t.split_whitespace().collect::<String>(), t.split_whitespace().collect::<Vec<_>>().join(" "))
}

/// How a member is declared: getters and setters of one name pair up.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Accessor {
    Value,
    Get,
    Set,
}

fn accessor(n: tree_sitter::Node<'_>) -> Accessor {
    if has_token(n, "get") {
        Accessor::Get
    } else if has_token(n, "set") {
        Accessor::Set
    } else {
        Accessor::Value
    }
}

/// Report the second and later declaration of a name, unless the two are a
/// getter and a setter.
fn report(seen: &mut BTreeMap<String, Vec<Accessor>>, name: String, acc: Accessor, at: tree_sitter::Node<'_>, what: &str, out: &mut Vec<NativeFinding>) {
    let prev = seen.entry(name.clone()).or_default();
    let clash = prev.iter().any(|p| !matches!((p, acc), (Accessor::Get, Accessor::Set) | (Accessor::Set, Accessor::Get)));
    if clash {
        out.push(finding(at, format!("Duplicate {what} `{name}`")));
    }
    prev.push(acc);
}

pub struct NoDupeKeys;

impl NativeRule for NoDupeKeys {
    fn id(&self) -> &'static str {
        "js/no-dupe-keys"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["object"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let mut seen = BTreeMap::new();
        for member in named_children(node.get_inner_node()) {
            let (key, acc) = match member.kind() {
                "pair" => (member.child_by_field_name("key"), Accessor::Value),
                "method_definition" => (member.child_by_field_name("name"), accessor(member)),
                "shorthand_property_identifier" => (Some(member), Accessor::Value),
                _ => continue,
            };
            let Some(key) = key else { continue };
            let Some(name) = property_name(key, ctx.source) else { continue };
            report(&mut seen, name, acc, key, "key", out);
        }
    }
}

pub struct NoDupeClassMembers;

impl NativeRule for NoDupeClassMembers {
    fn id(&self) -> &'static str {
        "js/no-dupe-class-members"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["class_body"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let mut seen = BTreeMap::new();
        for member in named_children(node.get_inner_node()) {
            let (key, acc) = match member.kind() {
                // A TypeScript overload signature is not an implementation.
                "method_definition" if member.child_by_field_name("body").is_some() => (member.child_by_field_name("name"), accessor(member)),
                "field_definition" | "public_field_definition" => {
                    (member.child_by_field_name("property").or_else(|| member.child_by_field_name("name")), Accessor::Value)
                }
                _ => continue,
            };
            let Some(key) = key else { continue };
            let Some(name) = property_name(key, ctx.source) else { continue };
            let is_static = has_token(member, "static");
            let name = if is_static { format!("static {name}") } else { name };
            report(&mut seen, name, acc, key, "class member", out);
        }
    }
}

pub struct NoDupeArgs;

impl NativeRule for NoDupeArgs {
    fn id(&self) -> &'static str {
        "js/no-dupe-args"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["formal_parameters"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        fn names<'t>(n: tree_sitter::Node<'t>, out: &mut Vec<tree_sitter::Node<'t>>) {
            match n.kind() {
                "identifier" | "shorthand_property_identifier_pattern" => out.push(n),
                "assignment_pattern" | "object_assignment_pattern" => {
                    if let Some(l) = n.child_by_field_name("left") {
                        names(l, out);
                    }
                }
                "pair_pattern" => {
                    if let Some(v) = n.child_by_field_name("value") {
                        names(v, out);
                    }
                }
                "required_parameter" | "optional_parameter" => {
                    if let Some(p) = n.child_by_field_name("pattern") {
                        names(p, out);
                    }
                }
                "type_annotation" | "comment" => {}
                _ => {
                    for c in named_children(n) {
                        names(c, out);
                    }
                }
            }
        }
        let mut all = Vec::new();
        for p in named_children(node.get_inner_node()) {
            names(p, &mut all);
        }
        let mut seen: BTreeMap<&str, ()> = BTreeMap::new();
        for n in all {
            let name = text(n, ctx.source);
            if seen.insert(name, ()).is_some() {
                out.push(finding(n, format!("Duplicate parameter `{name}`")));
            }
        }
    }
}

pub struct NoDuplicateCase;

impl NativeRule for NoDuplicateCase {
    fn id(&self) -> &'static str {
        "js/no-duplicate-case"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["switch_body"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let mut seen: BTreeMap<String, ()> = BTreeMap::new();
        for case in named_children(node.get_inner_node()) {
            let Some(value) = case.child_by_field_name("value") else { continue };
            let (key, shown) = normalised(value, ctx.source);
            if seen.insert(key, ()).is_some() {
                out.push(finding(value, format!("Duplicate case label `{shown}`")));
            }
        }
    }
}

pub struct NoDupeElseIf;

impl NativeRule for NoDupeElseIf {
    fn id(&self) -> &'static str {
        "js/no-dupe-else-if"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["if_statement"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let head = node.get_inner_node();
        // Only the head of a chain walks it, so each duplicate is reported once.
        if head.parent().is_some_and(|p| p.kind() == "else_clause") {
            return;
        }
        let mut seen: BTreeMap<String, ()> = BTreeMap::new();
        let mut current = Some(head);
        while let Some(ifs) = current {
            if let Some(cond) = ifs.child_by_field_name("condition") {
                let (key, shown) = normalised(cond, ctx.source);
                let shown = shown.trim_start_matches('(').trim_end_matches(')').trim().to_owned();
                if seen.insert(key, ()).is_some() {
                    out.push(finding(cond, format!("This branch can never run: its condition `{shown}` repeats an earlier one in the chain")));
                }
            }
            current = ifs
                .child_by_field_name("alternative")
                .and_then(|alt| children(alt).into_iter().find(|c| c.kind() == "if_statement"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(rule: &dyn NativeRule, src: &str) -> Vec<String> {
        crate::run(rule, Language::Typescript, src).into_iter().map(|f| format!("{} @{}", f.message, &src[f.start..f.end])).collect()
    }

    #[test]
    fn keys() {
        assert_eq!(found(&NoDupeKeys, "x = { a: 1, 'a': 2, b, b: 3, 1: 0, 0x1: 0, get c() {}, set c(v) {}, ['d']: 1, d: 2, [e]: 1, [e]: 2 };"), [
            "Duplicate key `a` @'a'",
            "Duplicate key `b` @b",
            "Duplicate key `1` @0x1",
            "Duplicate key `d` @d"
        ]);
    }

    #[test]
    fn class_members() {
        assert_eq!(
            found(&NoDupeClassMembers, "class A { a() {} a() {} static a() {} get b() {} set b(v) {} c = 1; c() {} o(x: number): void; o(x: any) {} }"),
            ["Duplicate class member `a` @a", "Duplicate class member `c` @c"]
        );
    }

    #[test]
    fn args_cases_else_if() {
        assert_eq!(found(&NoDupeArgs, "function f(a, b, a, { c, d: [c] }) {}"), ["Duplicate parameter `a` @a", "Duplicate parameter `c` @c"]);
        assert_eq!(found(&NoDuplicateCase, "switch (x) { case 1: case 2: break; case  1 : break; case f(a): case f( a ): }"), [
            "Duplicate case label `1` @1",
            "Duplicate case label `f( a )` @f( a )"
        ]);
        let src = "if (a) {} else if (b) {} else if (a) {} else {}\nif (c) {} else { if (c) {} }";
        assert_eq!(found(&NoDupeElseIf, src).len(), 1);
    }
}
