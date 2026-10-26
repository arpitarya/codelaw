//! The `calls:` predicate (SR-RULE-FORMAT decision 3a, W-73): the captured
//! node is a call whose callee is one of a rule's approved functions.
//!
//! The callee is a dotted path (`DOMPurify.sanitize`, `clean`,
//! `bleach.clean`). It is matched by name and by where its root binding
//! comes from: a `{ name, module }` item holds only when the root is
//! imported from that module, a bare item only when the root is undeclared
//! in the file (a global). A value is not followed through a variable.

use codelaw_rules::CallItem;
use codelaw_semantic::{Imported, SemanticModel};
use tree_sitter::Node;

fn text<'a>(n: Node<'_>, src: &'a str) -> &'a str {
    src.get(n.byte_range()).unwrap_or_default()
}

/// A callee as its segments, and where its root identifier starts.
fn path<'a>(n: Node<'_>, src: &'a str) -> Option<(Vec<&'a str>, usize)> {
    match n.kind() {
        "identifier" => Some((vec![text(n, src)], n.start_byte())),
        "member_expression" | "attribute" => {
            let object = n.child_by_field_name("object")?;
            let prop = n.child_by_field_name("property").or_else(|| n.child_by_field_name("attribute"))?;
            let (mut segs, root) = path(object, src)?;
            segs.push(text(prop, src));
            Some((segs, root))
        }
        "parenthesized_expression" | "non_null_expression" => path(n.named_child(0)?, src),
        _ => None,
    }
}

/// The callee of a call node, seen through wrappers that pass the value on.
fn callee_of<'t>(n: Node<'t>) -> Option<Node<'t>> {
    match n.kind() {
        "call_expression" | "call" => n.child_by_field_name("function"),
        "parenthesized_expression" | "non_null_expression" | "as_expression" | "satisfies_expression" | "await_expression" | "await" => {
            callee_of(n.named_child(0)?)
        }
        _ => None,
    }
}

/// One path segment against an item's: a name, `*` or `*<suffix>`.
fn segment_matches(item: &str, seg: &str) -> bool {
    match item.strip_prefix('*') {
        Some(suffix) => seg.ends_with(suffix),
        None => item == seg,
    }
}

fn names_match(item: &[&str], callee: &[&str]) -> bool {
    item.len() == callee.len() && item.iter().zip(callee).all(|(i, c)| segment_matches(i, c))
}

/// Whether `node` is a call to one of `items`.
pub fn is_call_to(node: Node<'_>, src: &str, model: &SemanticModel, items: &[CallItem]) -> bool {
    let Some(callee) = callee_of(node) else { return false };
    let Some((segs, root_at)) = path(callee, src) else { return false };
    let Some(root) = model.reference_at(root_at) else { return false };
    items.iter().any(|item| {
        let name: Vec<&str> = item.name.split('.').collect();
        if name[0].starts_with('*') {
            return item.modules.is_empty() && names_match(&name, &segs);
        }
        if item.modules.is_empty() {
            return root.symbol.is_none() && !model.has_star_import && names_match(&name, &segs);
        }
        // `const DOMPurify = createDOMPurify(window)`: an instance made by
        // calling what the module exports is the module's, under the name
        // the project gave it.
        if let Some(sym) = root.symbol
            && model.import_of(sym).is_none()
            && is_instance_of(node, model, sym, &item.modules)
        {
            return names_match(&name, &segs);
        }
        let Some(import) = root.symbol.and_then(|s| model.import_of(s)) else { return false };
        if !item.modules.contains(&import.source) || import.type_only {
            return false;
        }
        match &import.imported {
            // The export name is compared, so an alias still counts.
            Imported::Named(export) => name[0] == export && names_match(&name[1..], &segs[1..]) && name.len() == segs.len(),
            // The local name is compared for a default or namespace import.
            Imported::Default | Imported::Namespace => names_match(&name, &segs),
            // `import bleach as b`: the module's own name stands for the root.
            Imported::Module => {
                name[0] == import.source.rsplit('.').next().unwrap_or_default() && name.len() == segs.len() && names_match(&name[1..], &segs[1..])
            }
            Imported::Star | Imported::SideEffect => false,
        }
    })
}

const FUNCTIONS: &[&str] = &[
    "function_declaration",
    "function_expression",
    "function",
    "arrow_function",
    "method_definition",
    "generator_function_declaration",
    "generator_function",
    "function_definition",
    "lambda",
];

const CALLS: &[&str] = &["call_expression", "call"];

/// Whether `node` is a function, or sits in one, and that function or one
/// enclosing it contains a call to one of `items` (`function_calls:`).
/// The outermost enclosing function holds every inner one, so it alone is
/// searched.
pub fn function_calls(node: Node<'_>, src: &str, model: &SemanticModel, items: &[CallItem]) -> bool {
    let mut outer = None;
    let mut at = Some(node);
    while let Some(n) = at {
        if FUNCTIONS.contains(&n.kind()) {
            outer = Some(n);
        }
        at = n.parent();
    }
    let Some(outer) = outer else { return false };
    let mut cursor = outer.walk();
    let mut stack = vec![outer];
    while let Some(n) = stack.pop() {
        if CALLS.contains(&n.kind()) && is_call_to(n, src, model, items) {
            return true;
        }
        stack.extend(n.named_children(&mut cursor));
    }
    false
}

/// Whether the local `sym` is declared as `const x = f(…)` where `f` is an
/// import from one of `modules`.
fn is_instance_of(from: Node<'_>, model: &SemanticModel, sym: usize, modules: &[String]) -> bool {
    let mut root = from;
    while let Some(p) = root.parent() {
        root = p;
    }
    let Some(&(start, end)) = model.symbol(sym).decls.first() else { return false };
    let Some(ident) = root.descendant_for_byte_range(start, end) else { return false };
    let Some(decl) = ident.parent().filter(|p| p.kind() == "variable_declarator") else { return false };
    if decl.child_by_field_name("name") != Some(ident) {
        return false;
    }
    let Some(init) = decl.child_by_field_name("value").filter(|v| v.kind() == "call_expression") else { return false };
    let Some(f) = init.child_by_field_name("function").filter(|f| f.kind() == "identifier") else { return false };
    model
        .reference_at(f.start_byte())
        .and_then(|r| r.symbol)
        .and_then(|s| model.import_of(s))
        .is_some_and(|i| modules.contains(&i.source) && !i.type_only)
}
