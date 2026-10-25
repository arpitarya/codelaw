//! JavaScript and TypeScript syntax helpers shared by the library packs'
//! native rules (`tanstack-query/*`, `redux-toolkit/*`): finding the function
//! around a node, a parameter's name, what a `const` was initialised with
//! and where an identifier was imported from.

use crate::util::{named_children, text};
use codelaw_semantic::{Import, Imported, SemanticModel};
use tree_sitter::Node;

/// The node kinds that open a function body.
pub const FUNCTION_KINDS: &[&str] = &[
    "function_declaration",
    "function_expression",
    "function",
    "arrow_function",
    "method_definition",
    "generator_function",
    "generator_function_declaration",
];

pub fn is_function(n: Node<'_>) -> bool {
    FUNCTION_KINDS.contains(&n.kind())
}

/// The root of the tree a node is in.
pub fn root_of(n: Node<'_>) -> Node<'_> {
    let mut r = n;
    while let Some(p) = r.parent() {
        r = p;
    }
    r
}

/// The outermost function around a node.
pub fn outermost_function(n: Node<'_>) -> Option<Node<'_>> {
    std::iter::successors(n.parent(), |p| p.parent()).filter(|p| is_function(*p)).last()
}

/// A function's name: its own, or the variable it initialises.
pub fn function_name<'s>(f: Node<'_>, src: &'s str) -> Option<&'s str> {
    if let Some(n) = f.child_by_field_name("name") {
        return Some(text(n, src));
    }
    let parent = f.parent()?;
    (parent.kind() == "variable_declarator").then(|| parent.child_by_field_name("name")).flatten().filter(|n| n.kind() == "identifier").map(|n| text(n, src))
}

pub fn is_async(f: Node<'_>) -> bool {
    let mut c = f.walk();
    f.children(&mut c).any(|x| !x.is_named() && x.kind() == "async")
}

/// The parameters of a function, as the nodes that declare them.
pub fn params(f: Node<'_>) -> Vec<Node<'_>> {
    if let Some(p) = f.child_by_field_name("parameter") {
        return vec![p];
    }
    f.child_by_field_name("parameters").map(|ps| named_children(ps).into_iter().filter(|p| p.kind() != "comment").collect()).unwrap_or_default()
}

/// The identifier a parameter binds, when it binds one name: `x`,
/// `x = 1`, `x: T`, `x?: T`.
pub fn param_name<'s>(p: Node<'_>, src: &'s str) -> Option<&'s str> {
    match p.kind() {
        "identifier" => Some(text(p, src)),
        "assignment_pattern" => p.child_by_field_name("left").filter(|l| l.kind() == "identifier").map(|l| text(l, src)),
        "required_parameter" | "optional_parameter" => p.child_by_field_name("pattern").and_then(|x| param_name(x, src)),
        _ => None,
    }
}

/// `f`'s body, when it has one.
pub fn body(f: Node<'_>) -> Option<Node<'_>> {
    f.child_by_field_name("body")
}

/// The function node a value is, looking through parentheses.
pub fn as_function(n: Node<'_>) -> Option<Node<'_>> {
    let n = unparen(n);
    is_function(n).then_some(n)
}

/// `(x)`, `x as T`, `x satisfies T` and `x!` as `x`.
pub fn unparen(mut n: Node<'_>) -> Node<'_> {
    while matches!(n.kind(), "parenthesized_expression" | "as_expression" | "satisfies_expression" | "non_null_expression") {
        match n.named_child(0) {
            Some(inner) => n = inner,
            None => break,
        }
    }
    n
}

/// The `variable_declarator` that declares the symbol an identifier names,
/// when the identifier is that declarator's whole `name`.
pub fn declarator_of<'t>(model: &SemanticModel, root: Node<'t>, ident: Node<'_>) -> Option<Node<'t>> {
    let sym = model.symbol_at(ident.start_byte())?;
    let (start, end) = *model.symbol(sym).decls.first()?;
    let id = root.descendant_for_byte_range(start, end)?;
    let decl = id.parent().filter(|p| p.kind() == "variable_declarator")?;
    (decl.child_by_field_name("name").map(|n| n.id()) == Some(id.id())).then_some(decl)
}

/// What a `const x = …` identifier was initialised with.
pub fn initial_value<'t>(model: &SemanticModel, root: Node<'t>, ident: Node<'_>) -> Option<Node<'t>> {
    declarator_of(model, root, ident)?.child_by_field_name("value")
}

/// The import an identifier's symbol comes from, when `source` accepts the
/// module and it is a value import.
pub fn import_of<'m>(model: &'m SemanticModel, ident: Node<'_>, source: &dyn Fn(&str) -> bool) -> Option<&'m Import> {
    let sym = model.symbol_at(ident.start_byte())?;
    model.imports.iter().find(|i| i.local == Some(sym) && !i.type_only && source(&i.source))
}

/// Whether an identifier is the named import `name` from a module `source`
/// accepts (under any local alias).
pub fn is_named_import(model: &SemanticModel, ident: Node<'_>, name: &str, source: &dyn Fn(&str) -> bool) -> bool {
    import_of(model, ident, source).is_some_and(|i| matches!(&i.imported, Imported::Named(n) if n == name))
}

/// The name a call calls: `f(…)` gives `f`, `a.b.f(…)` gives `f`.
pub fn callee_name<'s>(call: Node<'_>, src: &'s str) -> Option<&'s str> {
    let f = call.child_by_field_name("function")?;
    match f.kind() {
        "identifier" => Some(text(f, src)),
        "member_expression" => f.child_by_field_name("property").map(|p| text(p, src)),
        _ => None,
    }
}

/// A call's arguments, comments left out.
pub fn arguments(call: Node<'_>) -> Vec<Node<'_>> {
    call.child_by_field_name("arguments").map(|a| named_children(a).into_iter().filter(|x| x.kind() != "comment").collect()).unwrap_or_default()
}

/// An object literal's members, comments left out.
pub fn members(object: Node<'_>) -> Vec<Node<'_>> {
    named_children(object).into_iter().filter(|x| x.kind() != "comment").collect()
}

/// The plain name of an object member's key: `a: …`, `"a": …`, `a`, `a() {}`.
pub fn member_name<'s>(m: Node<'_>, src: &'s str) -> Option<&'s str> {
    match m.kind() {
        "pair" => {
            let k = m.child_by_field_name("key")?;
            match k.kind() {
                "property_identifier" => Some(text(k, src)),
                "string" => Some(crate::util::unquote(text(k, src))),
                _ => None,
            }
        }
        "shorthand_property_identifier" => Some(text(m, src)),
        "method_definition" => m.child_by_field_name("name").map(|n| text(n, src)),
        _ => None,
    }
}

/// The member of an object literal with this key.
pub fn member<'t>(object: Node<'t>, name: &str, src: &str) -> Option<Node<'t>> {
    members(object).into_iter().find(|m| member_name(*m, src) == Some(name))
}

/// The value of an object member: the pair's value, the shorthand
/// identifier itself, the method.
pub fn member_value(m: Node<'_>) -> Option<Node<'_>> {
    match m.kind() {
        "pair" => m.child_by_field_name("value"),
        "shorthand_property_identifier" | "method_definition" => Some(m),
        _ => None,
    }
}

/// Every descendant of `n` (not `n`), not entering nested functions.
pub fn descendants_same_function(n: Node<'_>) -> Vec<Node<'_>> {
    let mut out = Vec::new();
    let mut stack: Vec<Node> = named_children(n);
    while let Some(x) = stack.pop() {
        out.push(x);
        if !is_function(x) {
            stack.extend(named_children(x));
        }
    }
    out.sort_by_key(|x| x.start_byte());
    out
}

/// The identifier at the root of a member chain: `a` of `a.b[c]?.d`.
pub fn chain_root(mut n: Node<'_>) -> Option<Node<'_>> {
    loop {
        match n.kind() {
            "identifier" | "this" => return Some(n),
            "member_expression" | "subscript_expression" => n = n.child_by_field_name("object")?,
            "parenthesized_expression" | "non_null_expression" => n = n.named_child(0)?,
            _ => return None,
        }
    }
}
