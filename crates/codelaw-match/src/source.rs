//! The `from_source` predicate (SR-RULE-FORMAT decision 3a): a capture that
//! is a known request source (`req.query`, `request.args`,
//! `location.hash` …), is built from one by concatenation or interpolation,
//! or is a local name bound to one in the same function.
//!
//! **This is not taint analysis.** It follows values only inside the one
//! function that holds the capture: an assignment (or a destructuring) in
//! that function, a few hops deep, flow-insensitively — any assignment in
//! the function counts, wherever it is. It never follows a call, a return,
//! a parameter, a closure's outer variable or another file. A value that
//! passes through any function call (a sanitizer or not) stops being a
//! source, except a method called on the source itself
//! (`request.args.get("next")`).

use codelaw_semantic::{SemanticModel, SymbolKind};
use tree_sitter::Node;

/// How many local bindings a value is followed through.
const MAX_HOPS: usize = 4;

/// One source: a dotted access path, `req.query` as `["req", "query"]`.
pub type SourcePath = Vec<String>;

/// Parse `req.query` into its segments; `None` for an empty segment.
pub fn parse_source(s: &str) -> Option<SourcePath> {
    let parts: Vec<String> = s.split('.').map(|p| p.trim().to_owned()).collect();
    (!parts.is_empty() && parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$')))
        .then_some(parts)
}

struct Ctx<'a, 't> {
    src: &'a str,
    model: &'a SemanticModel,
    root: Node<'t>,
    sources: &'a [SourcePath],
    /// The function that holds the capture (`None`: module level).
    function: Option<(usize, usize)>,
}

fn text<'a>(n: Node<'_>, src: &'a str) -> &'a str {
    src.get(n.byte_range()).unwrap_or_default()
}

fn is_function(kind: &str) -> bool {
    matches!(
        kind,
        "function_declaration"
            | "function_expression"
            | "function"
            | "arrow_function"
            | "method_definition"
            | "generator_function_declaration"
            | "generator_function"
            | "class_static_block"
            | "function_definition"
            | "lambda"
    )
}

/// The innermost function around a node, by its byte range.
fn enclosing_function(n: Node<'_>) -> Option<(usize, usize)> {
    let mut cur = n.parent();
    while let Some(p) = cur {
        if is_function(p.kind()) {
            return Some((p.start_byte(), p.end_byte()));
        }
        cur = p.parent();
    }
    None
}

fn root_of(n: Node<'_>) -> Node<'_> {
    let mut cur = n;
    while let Some(p) = cur.parent() {
        cur = p;
    }
    cur
}

/// Whether the capture is a request source (see the module doc).
pub fn from_source(node: Node<'_>, src: &str, model: &SemanticModel, sources: &[SourcePath]) -> bool {
    let ctx = Ctx { src, model, root: root_of(node), sources, function: enclosing_function(node) };
    ctx.value(node, MAX_HOPS)
}

impl<'t> Ctx<'_, 't> {
    fn has_source_prefix(&self, path: &[String]) -> bool {
        self.sources.iter().any(|s| path.len() >= s.len() && path[..s.len()] == s[..])
    }

    /// Whether a value is a source, or is built from one.
    fn value(&self, n: Node<'t>, hops: usize) -> bool {
        match n.kind() {
            // Wrappers that pass the value through.
            "parenthesized_expression" | "await_expression" | "await" | "non_null_expression" | "as_expression" | "satisfies_expression"
            | "type_assertion" | "expression_statement" => n.named_children(&mut n.walk()).any(|c| self.value(c, hops)),
            // String building and value choice: either side flows.
            "binary_expression" | "binary_operator" | "boolean_operator" => {
                let op = n.child_by_field_name("operator").map(|o| text(o, self.src)).unwrap_or_default();
                let flows = matches!(op, "+" | "||" | "??" | "%" | "or" | "and" | "&&");
                flows
                    && [n.child_by_field_name("left"), n.child_by_field_name("right")].into_iter().flatten().any(|c| self.value(c, hops))
            }
            "ternary_expression" | "conditional_expression" => {
                // JS: condition ? consequence : alternative; Python: a if c else b.
                let named: Vec<Node<'t>> = n.named_children(&mut n.walk()).collect();
                let branches: Vec<Node<'t>> = if n.kind() == "ternary_expression" {
                    [n.child_by_field_name("consequence"), n.child_by_field_name("alternative")].into_iter().flatten().collect()
                } else {
                    named.iter().enumerate().filter(|(i, _)| *i != 1).map(|(_, c)| *c).collect()
                };
                branches.into_iter().any(|c| self.value(c, hops))
            }
            "template_string" => n
                .named_children(&mut n.walk())
                .filter(|c| c.kind() == "template_substitution")
                .any(|c| c.named_children(&mut c.walk()).any(|e| self.value(e, hops))),
            "string" => n
                .named_children(&mut n.walk())
                .filter(|c| c.kind() == "interpolation")
                .any(|c| c.named_children(&mut c.walk()).any(|e| self.value(e, hops))),
            "concatenated_string" => n.named_children(&mut n.walk()).any(|c| self.value(c, hops)),
            "call" | "call_expression" if self.is_format_call(n) => n
                .child_by_field_name("arguments")
                .is_some_and(|a| a.named_children(&mut a.walk()).any(|c| self.value(c, hops))),
            _ => self.access(n, hops),
        }
    }

    /// `"…".format(x)`: the arguments flow into the string.
    fn is_format_call(&self, n: Node<'t>) -> bool {
        let Some(f) = n.child_by_field_name("function") else { return false };
        if f.kind() != "attribute" {
            return false;
        }
        let name = f.child_by_field_name("attribute").map(|a| text(a, self.src)).unwrap_or_default();
        name == "format" && f.child_by_field_name("object").is_some_and(|o| matches!(o.kind(), "string" | "concatenated_string"))
    }

    /// An access path (`a.b["c"].d()`), as segments, with its base
    /// identifier node.
    fn path(&self, n: Node<'t>) -> Option<(Node<'t>, Vec<String>)> {
        match n.kind() {
            "identifier" => Some((n, vec![text(n, self.src).to_owned()])),
            "member_expression" | "attribute" => {
                let object = n.child_by_field_name("object")?;
                let prop = n.child_by_field_name(if n.kind() == "attribute" { "attribute" } else { "property" })?;
                let (base, mut segs) = self.path(object)?;
                segs.push(text(prop, self.src).to_owned());
                Some((base, segs))
            }
            "subscript_expression" | "subscript" => {
                let object = n.child_by_field_name(if n.kind() == "subscript" { "value" } else { "object" })?;
                let index = n.child_by_field_name(if n.kind() == "subscript" { "subscript" } else { "index" });
                let (base, mut segs) = self.path(object)?;
                segs.push(index.map(|i| string_literal(i, self.src).unwrap_or_else(|| "*".to_owned())).unwrap_or_else(|| "*".to_owned()));
                Some((base, segs))
            }
            // A method called on a path: `request.args.get("next")`.
            "call_expression" | "call" => {
                let f = n.child_by_field_name("function")?;
                matches!(f.kind(), "member_expression" | "attribute").then_some(())?;
                self.path(f)
            }
            "parenthesized_expression" | "non_null_expression" => self.path(n.named_child(0)?),
            _ => None,
        }
    }

    fn access(&self, n: Node<'t>, hops: usize) -> bool {
        let Some((base, segs)) = self.path(n) else { return false };
        if self.has_source_prefix(&segs) {
            return true;
        }
        hops > 0 && self.binding(base, &segs[1..], hops - 1)
    }

    /// The base identifier is a local name of the capture's function,
    /// bound there to a source (or to a path that, with `rest`, is one).
    fn binding(&self, ident: Node<'t>, rest: &[String], hops: usize) -> bool {
        let Some(sym) = self.model.symbol_at(ident.start_byte()) else { return false };
        let symbol = self.model.symbol(sym);
        if matches!(symbol.kind, SymbolKind::Param | SymbolKind::Import | SymbolKind::Function | SymbolKind::Class | SymbolKind::CatchParam) {
            return false;
        }
        let mut values: Vec<(Node<'t>, Vec<String>)> = Vec::new();
        for &(start, end) in &symbol.decls {
            if let Some(id) = self.root.descendant_for_byte_range(start, end) {
                values.extend(self.bound_value(id));
            }
        }
        for &r in &symbol.refs {
            let reference = &self.model.references[r];
            if reference.write
                && let Some(id) = self.root.descendant_for_byte_range(reference.start, reference.end)
            {
                values.extend(self.bound_value(id));
            }
        }
        values.into_iter().any(|(v, key)| {
            // Only an assignment in the capture's own function counts.
            if enclosing_function(v) != self.function {
                return false;
            }
            if key.is_empty() && rest.is_empty() {
                return self.value(v, hops);
            }
            // `const q = req.query; q.next` or `const { next } = req.query`:
            // the bound path, then the destructured keys, then the rest.
            let Some((base, mut segs)) = self.path(v) else {
                // A member of a value built from a source is a source too.
                return self.value(v, hops);
            };
            segs.extend(key);
            segs.extend(rest.iter().cloned());
            self.has_source_prefix(&segs) || self.value(v, hops) || (hops > 0 && self.binding(base, &segs[1..], hops - 1))
        })
    }

    /// The value an identifier is bound to where it is declared or
    /// written, and the destructured keys between them.
    fn bound_value(&self, ident: Node<'t>) -> Option<(Node<'t>, Vec<String>)> {
        let mut key: Vec<String> = Vec::new();
        let mut cur = ident;
        let mut positional = false;
        loop {
            let parent = cur.parent()?;
            let is = |field: &str| parent.child_by_field_name(field) == Some(cur);
            match parent.kind() {
                "variable_declarator" => return is("name").then(|| parent.child_by_field_name("value")).flatten().map(|v| (v, key)),
                "assignment_expression" | "augmented_assignment_expression" | "assignment" | "augmented_assignment" => {
                    return is("left").then(|| parent.child_by_field_name("right")).flatten().map(|v| (v, key));
                }
                "named_expression" => return is("name").then(|| parent.child_by_field_name("value")).flatten().map(|v| (v, key)),
                // `{ next }` and `{ next = "/" }`: the name is the key.
                "object_pattern" | "object_assignment_pattern" => {
                    if !positional && cur.kind() == "shorthand_property_identifier_pattern" {
                        key.insert(0, text(cur, self.src).to_owned());
                    }
                }
                // `{ next: url }`.
                "pair_pattern" => {
                    if !positional
                        && is("value")
                        && let Some(k) = parent.child_by_field_name("key")
                    {
                        key.insert(0, string_literal(k, self.src).unwrap_or_else(|| text(k, self.src).to_owned()));
                    }
                }
                // A default (`url = "/"`) keeps the binding.
                "assignment_pattern" => {}
                // Positions and rest elements: the whole value flows, no key.
                "array_pattern" | "rest_pattern" | "rest_element" | "pattern_list" | "tuple_pattern" | "list_pattern" => {
                    key.clear();
                    positional = true;
                }
                _ => return None,
            }
            cur = parent;
        }
    }
}

/// The text of a string literal, without its quotes.
fn string_literal(n: Node<'_>, src: &str) -> Option<String> {
    matches!(n.kind(), "string").then_some(())?;
    let t = text(n, src);
    let t = t.trim_start_matches(['b', 'r', 'u', 'B', 'R', 'U']);
    let q = t.chars().next()?;
    matches!(q, '"' | '\'' | '`').then(|| t.trim_matches(q).to_owned())
}
