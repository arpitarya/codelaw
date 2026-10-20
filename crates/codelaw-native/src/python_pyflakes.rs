//! Native rules behind `py/recommended`'s Pyflakes ports (Ruff's `F`
//! rules): imports, names, scopes and statements in the wrong place.

use crate::NativeContext;
use crate::NativeFinding;
use crate::python_util::{
    call_args, decorator_target, decorators, hit, hit_fix, is_docstring_stmt, is_fstring, is_pure, kids,
    names_one_of, py_rule, remove_stmt, starts_line, statement_of, statements, string_parts, string_value, unparen,

};
use crate::util::{children, text};
use codelaw_core::TextEdit;
use codelaw_semantic::{Imported, ScopeKind, SemanticModel, SymbolId, SymbolKind};
use std::collections::{BTreeMap, BTreeSet};
use tree_sitter::Node;

/// How an identifier came to be bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Bind {
    Import,
    Def,
    Class,
    Assign,
    AnnOnly,
    Aug,
    Loop,
    With,
    Except,
    Walrus,
    Other,
}

/// The binding context of a declaring identifier, and the node that
/// binds it (the assignment, `for`, `as` pattern, …).
pub(crate) fn bind_ctx(id: Node<'_>) -> (Bind, Option<Node<'_>>) {
    let mut cur = id;
    while let Some(p) = cur.parent() {
        let kind = match p.kind() {
            "function_definition" => {
                if p.child_by_field_name("name") == Some(cur) {
                    Bind::Def
                } else {
                    Bind::Other
                }
            }
            "class_definition" => {
                if p.child_by_field_name("name") == Some(cur) {
                    Bind::Class
                } else {
                    Bind::Other
                }
            }
            "dotted_name" | "aliased_import" | "pattern_list" | "tuple_pattern" | "list_pattern" | "tuple" | "list"
            | "parenthesized_expression" | "list_splat_pattern" | "list_splat" | "as_pattern_target" | "expression_list" => {
                cur = p;
                continue;
            }
            "import_statement" | "import_from_statement" | "future_import_statement" => Bind::Import,
            "assignment" => {
                if p.child_by_field_name("left") == Some(cur) {
                    if p.child_by_field_name("right").is_none() { Bind::AnnOnly } else { Bind::Assign }
                } else {
                    Bind::Other
                }
            }
            "augmented_assignment" => Bind::Aug,
            "for_statement" | "for_in_clause" => Bind::Loop,
            "as_pattern" => match p.parent().map(|g| g.kind()) {
                Some("with_item") => Bind::With,
                Some("except_clause") => Bind::Except,
                _ => Bind::Other,
            },
            "named_expression" => Bind::Walrus,
            _ => Bind::Other,
        };
        return (kind, Some(p));
    }
    (Bind::Other, None)
}

fn ident_at(root: Node<'_>, start: usize, end: usize) -> Option<Node<'_>> {
    root.descendant_for_byte_range(start, end).filter(|n| n.kind() == "identifier")
}

fn line_of(src: &str, offset: usize) -> usize {
    crate::util::line_of(src, offset)
}

/// Whether two nodes sit in branches of one `if`, `try` or `match` that
/// can't both run.
pub(crate) fn different_forks(a: Node<'_>, b: Node<'_>) -> bool {
    let mut path_a = Vec::new();
    let mut cur = Some(a);
    while let Some(x) = cur {
        path_a.push(x);
        cur = x.parent();
    }
    let mut child_b = b;
    let mut cur = b.parent();
    while let Some(x) = cur {
        if let Some(i) = path_a.iter().position(|n| n.id() == x.id()) {
            if i == 0 {
                return false;
            }
            let child_a = path_a[i - 1];
            if child_a.id() == child_b.id() {
                return false;
            }
            return match x.kind() {
                "if_statement" => true,
                "try_statement" => {
                    let group = |c: Node<'_>| if c.kind() == "except_clause" { c.start_byte() + 1 } else { 0 };
                    group(child_a) != group(child_b)
                }
                "block" => x.parent().is_some_and(|p| p.kind() == "match_statement"),
                _ => false,
            };
        }
        child_b = x;
        cur = x.parent();
    }
    false
}

/// Names that appear inside string annotations (`x: "Foo"`), which use the
/// imports they name.
fn names_in_string_annotations(root: Node<'_>, src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if n.kind() == "string" && n.parent().is_some() && has_type_ancestor(n) {
            if let Some((_, content)) = string_parts(n, src) {
                collect_words(&content, &mut out);
            }
            continue;
        }
        stack.extend(kids(n));
    }
    out
}

fn has_type_ancestor(n: Node<'_>) -> bool {
    let mut p = n.parent();
    while let Some(x) = p {
        if x.kind() == "type" {
            return true;
        }
        if matches!(x.kind(), "block" | "module") {
            return false;
        }
        p = x.parent();
    }
    false
}

fn collect_words(s: &str, out: &mut BTreeSet<String>) {
    let mut word = String::new();
    for c in s.chars().chain(std::iter::once(' ')) {
        if c.is_alphanumeric() || c == '_' {
            word.push(c);
        } else if !word.is_empty() {
            if !word.chars().next().is_some_and(|f| f.is_ascii_digit()) {
                out.insert(std::mem::take(&mut word));
            }
            word.clear();
        }
    }
}

/// The import entry node (`dotted_name` / `aliased_import`) of an import.
fn import_entry<'t>(root: Node<'t>, start: usize, end: usize) -> Option<Node<'t>> {
    let mut n = root.descendant_for_byte_range(start, end)?;
    while n.start_byte() == start && n.end_byte() == end {
        if matches!(n.parent().map(|p| p.kind()), Some("import_statement" | "import_from_statement" | "future_import_statement")) {
            return Some(n);
        }
        n = n.parent()?;
    }
    None
}

/// Edits that remove one name from an import statement, with its comma.
fn remove_entry(entry: Node<'_>, src: &str) -> Option<Vec<TextEdit>> {
    let next = entry.next_sibling();
    if let Some(comma) = next.filter(|c| c.kind() == ",") {
        let after = comma.next_sibling()?;
        let line_end = src[comma.end_byte()..].find('\n').map(|i| comma.end_byte() + i);
        if starts_line(src, entry.start_byte())
            && let Some(le) = line_end
            && src[comma.end_byte()..le].trim().is_empty()
        {
            let ls = src[..entry.start_byte()].rfind('\n').map(|i| i + 1).unwrap_or(0);
            return Some(vec![TextEdit::new(ls, le + 1, "")]);
        }
        let mut stop = comma.end_byte();
        while src[stop..].starts_with(' ') {
            stop += 1;
        }
        let stop = stop.min(after.start_byte().max(comma.end_byte()));
        return Some(vec![TextEdit::new(entry.start_byte(), stop, "")]);
    }
    let prev = entry.prev_sibling().filter(|c| c.kind() == ",")?;
    Some(vec![TextEdit::new(prev.start_byte(), entry.end_byte(), "")])
}

fn qualified_import(model: &SemanticModel, i: usize) -> String {
    let imp = &model.imports[i];
    match &imp.imported {
        Imported::Named(x) => {
            if imp.source.ends_with('.') { format!("{}{}", imp.source, x) } else { format!("{}.{}", imp.source, x) }
        }
        _ => imp.source.clone(),
    }
}

/// The last declaration of a symbol, by position.
fn last_decl(model: &SemanticModel, id: SymbolId) -> (usize, usize) {
    model.symbol(id).decls.iter().copied().max().unwrap_or((0, 0))
}

fn inside_try_body(n: Node<'_>) -> bool {
    let mut cur = n;
    while let Some(p) = cur.parent() {
        if p.kind() == "try_statement" && p.child_by_field_name("body") == Some(cur) {
            return true;
        }
        if matches!(p.kind(), "function_definition" | "class_definition") {
            return false;
        }
        cur = p;
    }
    false
}

fn unused_import(root: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(model) = ctx.semantic else { return };
    let src = ctx.source;
    let in_annotations = names_in_string_annotations(root, src);
    // Statement start → (entries, unused entries).
    let mut by_stmt: BTreeMap<usize, (usize, Vec<usize>)> = BTreeMap::new();
    let mut unused = Vec::new();
    for (i, imp) in model.imports.iter().enumerate() {
        let Some(entry) = import_entry(root, imp.start, imp.end) else { continue };
        let stmt = entry.parent().expect("an entry has a statement");
        let slot = by_stmt.entry(stmt.start_byte()).or_insert((0, Vec::new()));
        slot.0 += 1;
        let Some(sym) = imp.local else { continue };
        let s = model.symbol(sym);
        if s.implicitly_used || !model.is_unused(sym) || in_annotations.contains(&s.name) || s.name == "__all__" {
            continue;
        }
        // An import in a class body defines a class attribute.
        if model.scopes[s.scope].kind == ScopeKind::Class {
            continue;
        }
        // `import a as x` plus `import a.b`: the second loads a submodule `x.b` uses.
        if imp.imported == Imported::Module && imp.source.contains('.') && !src[imp.start..imp.end].contains(" as ") {
            let root = imp.source.split('.').next().unwrap_or("");
            let aliased = model.imports.iter().any(|o| o.imported == Imported::Module && o.source.split('.').next() == Some(root) && src[o.start..o.end].contains(" as "));
            if aliased {
                continue;
            }
        }
        let decl = s.decls.iter().copied().find(|d| d.0 >= imp.start && d.1 <= imp.end);
        if decl.is_none_or(|d| d != last_decl(model, sym)) {
            continue;
        }
        slot.1.push(i);
        unused.push((i, entry, stmt));
    }
    for (i, entry, stmt) in unused {
        let (total, ref gone) = by_stmt[&stmt.start_byte()];
        let edits = if inside_try_body(stmt) {
            None
        } else if gone.len() == total {
            remove_stmt(stmt, src)
        } else {
            remove_entry(entry, src)
        };
        out.push(hit_fix(entry, format!("`{}` is imported but never used", qualified_import(model, i)), edits.unwrap_or_default()));
    }
}
py_rule!(UnusedImport, "py/unused-import", ["module"], true, unused_import);

/// The identifiers a target binds.
fn target_names<'t>(n: Node<'t>, out: &mut Vec<Node<'t>>) {
    match n.kind() {
        "identifier" => out.push(n),
        "pattern_list" | "tuple_pattern" | "list_pattern" | "tuple" | "list" | "parenthesized_expression" | "list_splat_pattern"
        | "list_splat" => {
            for c in kids(n) {
                target_names(c, out);
            }
        }
        _ => {}
    }
}

fn import_shadowed_by_loop_var(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(model) = ctx.semantic else { return };
    let Some(left) = n.child_by_field_name("left") else { return };
    let mut names = Vec::new();
    target_names(left, &mut names);
    for id in names {
        let Some(sym) = model.declared_at(id.start_byte()) else { continue };
        let s = model.symbol(sym);
        let shadowed = if s.kind == SymbolKind::Import {
            s.decls.iter().find(|d| d.0 < id.start_byte()).copied()
        } else {
            None
        };
        if let Some(d) = shadowed {
            out.push(hit(id, format!("The loop variable `{}` replaces the import on line {}", s.name, line_of(ctx.source, d.0))));
        }
    }
}
py_rule!(ImportShadowedByLoopVar, "py/import-shadowed-by-loop-var", ["for_statement"], true, import_shadowed_by_loop_var);

fn star_import(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(stmt) = n.parent() else { return };
    let module = stmt.child_by_field_name("module_name").map(|m| text(m, ctx.source)).unwrap_or("");
    // `from .sub import *` is how a package re-exports its submodules' API.
    if module.starts_with('.') {
        return;
    }
    out.push(hit(stmt, format!("`from {module} import *` hides which names the module uses")));
}
py_rule!(UndefinedLocalWithImportStar, "py/no-star-import", ["wildcard_import"], false, star_import);

fn star_import_usage(_root: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(model) = ctx.semantic else { return };
    if !model.has_star_import {
        return;
    }
    let mut extra = extra_builtins(ctx);
    // In a package's `__init__.py`, `from .sub import *` also binds `sub`.
    for i in &model.imports {
        if i.imported == Imported::Star
            && i.source.starts_with('.')
            && let Some(last) = i.source.trim_start_matches('.').rsplit('.').next().filter(|x| !x.is_empty())
        {
            extra.insert(last.to_owned());
        }
    }
    for st in dunder_all_strings(_root, ctx.source) {
        if let Some(name) = string_value(st, ctx.source)
            && !model.scopes[0].bindings.contains_key(&name)
            && !extra.contains(&name)
        {
            out.push(hit(st, format!("`{name}` in `__all__` may be undefined, or come from a star import")));
        }
    }
    for r in &model.references {
        if r.symbol.is_none() && !codelaw_semantic::PYTHON_BUILTINS.contains(&r.name.as_str()) && !extra.contains(&r.name) && !implicit_name(model, r.scope, &r.name) {
            out.push(NativeFinding {
                start: r.start,
                end: r.end,
                message: format!("`{}` may be undefined, or come from a star import", r.name),
                edits: Vec::new(),
            });
        }
    }
}
py_rule!(StarImportUsage, "py/star-import-usage", ["module"], true, star_import_usage);

fn nested_star_import(n: Node<'_>, _ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    if crate::python_util::enclosing_def(n).is_some()
        && let Some(stmt) = n.parent()
    {
        out.push(hit(stmt, "`import *` is only allowed at module level"));
    }
}
py_rule!(NestedStarImport, "py/nested-star-import", ["wildcard_import"], false, nested_star_import);

fn late_future_import(n: Node<'_>, _ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(parent) = n.parent() else { return };
    if parent.kind() != "module" {
        out.push(hit(n, "`from __future__` imports must come first in the module"));
        return;
    }
    for (i, s) in statements(parent).into_iter().enumerate() {
        if s.id() == n.id() {
            return;
        }
        if s.kind() == "future_import_statement" || (i == 0 && is_docstring_stmt(s)) {
            continue;
        }
        out.push(hit(n, "`from __future__` imports must come first in the module"));
        return;
    }
}
py_rule!(LateFutureImport, "py/late-future-import", ["future_import_statement"], false, late_future_import);

const FUTURE_FEATURES: &[&str] = &[
    "nested_scopes", "generators", "division", "absolute_import", "with_statement", "print_function", "unicode_literals",
    "barry_as_FLUFL", "generator_stop", "annotations",
];

fn future_feature_not_defined(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    for c in kids(n) {
        let name = match c.kind() {
            "dotted_name" => c,
            "aliased_import" => match c.child_by_field_name("name") {
                Some(x) => x,
                None => continue,
            },
            _ => continue,
        };
        let t = text(name, ctx.source);
        if !FUTURE_FEATURES.contains(&t) {
            out.push(hit(name, format!("`{t}` is not a `__future__` feature")));
        }
    }
}
py_rule!(FutureFeatureNotDefined, "py/undefined-future-feature", ["future_import_statement"], false, future_feature_not_defined);

fn starred_targets(n: Node<'_>, _ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>, many: bool) {
    let Some(p) = n.parent() else { return };
    let is_target = matches!(p.kind(), "assignment" | "for_statement" | "for_in_clause") && p.child_by_field_name("left") == Some(n);
    if !is_target {
        return;
    }
    let items = kids(n);
    let stars: Vec<usize> = items.iter().enumerate().filter(|(_, c)| matches!(c.kind(), "list_splat_pattern" | "list_splat")).map(|(i, _)| i).collect();
    if many {
        if stars.len() > 1 {
            out.push(hit(n, "Only one starred expression is allowed in an assignment"));
        }
    } else if let [i] = stars[..]
        && (i >= 1 << 8 || items.len() - i > 1 << 24)
    {
        out.push(hit(n, "Too many expressions around the starred target"));
    }
}
fn multiple_starred(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    starred_targets(n, ctx, out, true)
}
fn star_too_many(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    starred_targets(n, ctx, out, false)
}
py_rule!(MultipleStarredExpressions, "py/multiple-starred-expressions", ["pattern_list", "tuple_pattern", "list_pattern", "tuple", "list"], false, multiple_starred);
py_rule!(ExpressionsInStarAssignment, "py/too-many-expressions-in-star-assignment", ["pattern_list", "tuple_pattern", "list_pattern", "tuple", "list"], false, star_too_many);

fn assert_tuple(n: Node<'_>, _ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    if let Some(first) = kids(n).first()
        && first.kind() == "tuple"
        && !kids(*first).is_empty()
    {
        out.push(hit(*first, "A non-empty tuple is always true, so this assert never fails"));
    }
}
py_rule!(AssertTuple, "py/assert-tuple", ["assert_statement"], false, assert_tuple);

fn if_tuple(n: Node<'_>, _ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    if let Some(c) = n.child_by_field_name("condition") {
        let c = unparen(c);
        if c.kind() == "tuple" && !kids(c).is_empty() {
            out.push(hit(c, "A non-empty tuple is always true, so this condition never changes"));
        }
    }
}
py_rule!(IfTuple, "py/if-tuple", ["if_statement", "elif_clause"], false, if_tuple);

/// A literal `is` must not compare against: strings, bytes, numbers and
/// tuples of them.
fn is_constant_non_singleton(n: Node<'_>) -> bool {
    let n = unparen(n);
    match n.kind() {
        "string" | "concatenated_string" | "integer" | "float" => true,
        "tuple" => kids(n).iter().all(|c| is_constant_non_singleton(*c) || matches!(c.kind(), "true" | "false" | "none")),
        "unary_operator" => n.child_by_field_name("argument").is_some_and(|a| matches!(a.kind(), "integer" | "float")),
        _ => false,
    }
}

fn is_literal(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let src = ctx.source;
    let mut operands: Vec<Node> = Vec::new();
    let mut tokens: Vec<Vec<Node>> = vec![Vec::new()];
    for c in children(n) {
        if c.kind() == "comment" {
            continue;
        }
        if c.is_named() {
            operands.push(c);
            tokens.push(Vec::new());
        } else if let Some(t) = tokens.last_mut() {
            t.push(c);
        }
    }
    // tokens[i] sits between operands[i-1] and operands[i].
    for i in 1..operands.len() {
        let toks = &tokens[i];
        let op: Vec<&str> = toks.iter().map(|t| text(*t, src)).collect();
        let op = op.join(" ").split_whitespace().collect::<Vec<_>>().join(" ");
        if op != "is" && op != "is not" {
            continue;
        }
        if !(is_constant_non_singleton(operands[i - 1]) || is_constant_non_singleton(operands[i])) {
            continue;
        }
        let (Some(first), Some(last)) = (toks.first(), toks.last()) else { continue };
        let repl = if op == "is" { "==" } else { "!=" };
        out.push(NativeFinding {
            start: operands[i - 1].start_byte(),
            end: operands[i].end_byte(),
            message: format!("Use `{repl}` to compare with a literal; `{op}` compares identity"),
            edits: vec![TextEdit::new(first.start_byte(), last.end_byte(), repl)],
        });
    }
}
py_rule!(IsLiteral, "py/is-literal", ["comparison_operator"], false, is_literal);

fn invalid_print_syntax(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    match n.kind() {
        "print_statement" => {
            if kids(n).iter().any(|c| c.kind() == "chevron") {
                out.push(hit(n, "`print >>` is Python 2 syntax; use `print(..., file=...)`"));
            }
        }
        _ => {
            if let (Some(l), Some(op)) = (n.child_by_field_name("left"), n.child_by_field_name("operator"))
                && text(op, ctx.source) == ">>"
                && l.kind() == "identifier"
                && text(l, ctx.source) == "print"
                && ctx.semantic.is_none_or(|m| m.reference_at(l.start_byte()).is_none_or(|r| r.symbol.is_none()))
            {
                out.push(hit(n, "`print >>` is Python 2 syntax; use `print(..., file=...)`"));
            }
        }
    }
}
py_rule!(InvalidPrintSyntax, "py/invalid-print-syntax", ["print_statement", "binary_operator"], true, invalid_print_syntax);

/// Whether `n` sits in the body of a loop, within the same function.
fn in_loop(n: Node<'_>) -> bool {
    let mut cur = n;
    while let Some(p) = cur.parent() {
        match p.kind() {
            "for_statement" | "while_statement" if p.child_by_field_name("body") == Some(cur) => return true,
            "function_definition" | "class_definition" | "lambda" => return false,
            _ => {}
        }
        cur = p;
    }
    false
}

fn break_outside_loop(n: Node<'_>, _ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    if !in_loop(n) {
        out.push(hit(n, "`break` outside a loop"));
    }
}
fn continue_outside_loop(n: Node<'_>, _ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    if !in_loop(n) {
        out.push(hit(n, "`continue` outside a loop"));
    }
}
py_rule!(BreakOutsideLoop, "py/break-outside-loop", ["break_statement"], false, break_outside_loop);
py_rule!(ContinueOutsideLoop, "py/continue-outside-loop", ["continue_statement"], false, continue_outside_loop);

/// The innermost function, class or module around a node.
fn frame(n: Node<'_>) -> &'static str {
    let mut p = n.parent();
    while let Some(x) = p {
        match x.kind() {
            "function_definition" | "lambda" => return "function",
            "class_definition" => return "class",
            _ => {}
        }
        p = x.parent();
    }
    "module"
}

fn yield_outside_function(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let f = frame(n);
    if f == "function" {
        return;
    }
    let what = if n.kind() == "await" {
        "await"
    } else if children(n).iter().any(|c| c.kind() == "from") {
        "yield from"
    } else {
        "yield"
    };
    let _ = ctx;
    out.push(hit(n, format!("`{what}` outside a function (at {f} level)")));
}
py_rule!(YieldOutsideFunction, "py/yield-outside-function", ["yield", "await"], false, yield_outside_function);

fn return_outside_function(n: Node<'_>, _ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    if frame(n) != "function" {
        out.push(hit(n, "`return` outside a function"));
    }
}
py_rule!(ReturnOutsideFunction, "py/return-outside-function", ["return_statement"], false, return_outside_function);

fn default_except_not_last(n: Node<'_>, _ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let handlers: Vec<Node> = kids(n).into_iter().filter(|c| c.kind() == "except_clause").collect();
    for (i, h) in handlers.iter().enumerate() {
        let bare = kids(*h).first().is_none_or(|c| c.kind() == "block");
        if bare && i + 1 < handlers.len() {
            let end = h.child(0).map(|c| c.end_byte()).unwrap_or(h.end_byte());
            out.push(NativeFinding { start: h.start_byte(), end, message: "A bare `except:` must be the last handler".into(), edits: Vec::new() });
        }
    }
}
py_rule!(DefaultExceptNotLast, "py/default-except-not-last", ["try_statement"], false, default_except_not_last);

fn in_literal_or_annotated(n: Node<'_>, src: &str) -> bool {
    let mut p = n.parent();
    while let Some(x) = p {
        if x.kind() == "generic_type" || x.kind() == "subscript" {
            let head = x.child(0).map(|h| text(h, src)).unwrap_or("");
            if head.ends_with("Literal") || head.ends_with("Annotated") {
                return true;
            }
        }
        if x.kind() == "type" {
            return false;
        }
        p = x.parent();
    }
    false
}

fn forward_annotation_syntax_error(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    if !has_type_ancestor(n) || n.parent().is_some_and(|p| p.kind() == "concatenated_string") || in_literal_or_annotated(n, ctx.source) {
        return;
    }
    let Some((prefix, content)) = string_parts(n, ctx.source) else { return };
    if prefix.contains('b') || prefix.contains('f') {
        return;
    }
    let body = content.trim();
    if body.is_empty() {
        out.push(hit(n, "This string annotation is empty"));
        return;
    }
    let wrapped = format!("__codelaw_annotation__: ({body})\n");
    let root = codelaw_lang::parse(codelaw_core::Language::Python, &wrapped);
    if codelaw_lang::syntax_error_count(&root) > 0 {
        out.push(hit(n, format!("The string annotation `{body}` is not a valid expression")));
    }
}
py_rule!(ForwardAnnotationSyntaxError, "py/forward-annotation-syntax-error", ["string"], false, forward_annotation_syntax_error);

fn is_overload(def: Node<'_>, model: &SemanticModel, src: &str) -> bool {
    decorators(def).into_iter().any(|d| {
        let t = decorator_target(d);
        text(t, src).ends_with("overload") || names_one_of(t, model, src, &["typing.overload", "typing_extensions.overload"])
    })
}

fn redefined_while_unused(root: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(model) = ctx.semantic else { return };
    let src = ctx.source;
    for (id, s) in model.symbols.iter().enumerate() {
        if s.decls.len() < 2 {
            continue;
        }
        let mut decls = s.decls.clone();
        decls.sort();
        decls.dedup();
        for w in decls.windows(2) {
            let (a, b) = (w[0], w[1]);
            let (Some(na), Some(nb)) = (ident_at(root, a.0, a.1), ident_at(root, b.0, b.1)) else { continue };
            let (ka, ctx_a) = bind_ctx(na);
            if !matches!(ka, Bind::Import | Bind::Def | Bind::Class) {
                continue;
            }
            let (kb, ctx_b) = bind_ctx(nb);
            if matches!(kb, Bind::AnnOnly | Bind::Aug | Bind::Other) {
                continue;
            }
            // A definition's own body runs later; a reference there doesn't use it now.
            let body = if matches!(ka, Bind::Def | Bind::Class) { ctx_a } else { None };
            let used = s.refs.iter().any(|&r| {
                let r = &model.references[r];
                let between = r.start >= a.1 && r.start <= b.0;
                // `x = f(x)`: the right-hand side runs before the new binding.
                let in_rhs = kb == Bind::Assign
                    && ctx_b.and_then(|c| c.child_by_field_name("right")).is_some_and(|rh| rh.start_byte() <= r.start && r.start < rh.end_byte());
                r.read && (between || in_rhs) && body.is_none_or(|d| !(d.start_byte() <= r.start && r.start < d.end_byte()))
            });
            if used || different_forks(na, nb) {
                continue;
            }
            // An optional accelerator: `try: from _speedups import f`.
            if kb == Bind::Import && in_try(nb) {
                continue;
            }
            if ka == Bind::Def && ctx_a.is_some_and(|d| is_overload(d, model, src)) {
                continue;
            }
            if ka == Bind::Import && kb == Bind::Import {
                // `import a.b` and `import a.c` both bind `a`, for different submodules.
                let entry = |id: Node<'_>| {
                    let p = id.parent().filter(|p| p.kind() == "dotted_name");
                    p.filter(|p| p.parent().is_some_and(|g| g.kind() == "import_statement")).map(|p| text(p, src).to_owned())
                };
                let (ea, eb) = (entry(na), entry(nb));
                if (ea.as_deref().is_some_and(|e| e.contains('.')) || eb.as_deref().is_some_and(|e| e.contains('.'))) && ea != eb {
                    continue;
                }
            }
            // A `global` declaration rebinds from inside a function.
            let outer = |n: Node<'_>| crate::python_util::enclosing_def(statement_of(n)).map(|d| d.id());
            if outer(na) != outer(nb) {
                continue;
            }
            if ka == Bind::Import && in_try(na) {
                continue;
            }
            let _ = id;
            out.push(hit(nb, format!("`{}` is redefined here, but the definition on line {} was never used", s.name, line_of(src, a.0))));
        }
    }
}
py_rule!(RedefinedWhileUnused, "py/redefined-while-unused", ["module"], true, redefined_while_unused);

fn extra_builtins(ctx: &NativeContext<'_>) -> BTreeSet<String> {
    let mut out: BTreeSet<String> = ["WindowsError", "__builtins__"].iter().map(|s| s.to_string()).collect();
    for b in ctx.options.get("builtins").and_then(|v| v.as_array()).into_iter().flatten().filter_map(|v| v.as_str()) {
        out.insert(b.to_owned());
    }
    out
}

/// Names Python defines implicitly in some scopes: `__class__` in methods,
/// `__module__` and `__qualname__` in class bodies.
fn implicit_name(model: &SemanticModel, scope: usize, name: &str) -> bool {
    let mut sc = Some(scope);
    while let Some(s) = sc {
        let kind = model.scopes[s].kind;
        if kind == ScopeKind::Class && matches!(name, "__module__" | "__qualname__" | "__class__") {
            return name != "__class__" || s != scope;
        }
        sc = model.scopes[s].parent;
    }
    false
}

/// Whether the reference sits in a `try` body whose handlers catch
/// `NameError`.
fn guarded_by_name_error(root: Node<'_>, start: usize, src: &str) -> bool {
    let Some(mut cur) = root.descendant_for_byte_range(start, start) else { return false };
    while let Some(p) = cur.parent() {
        if p.kind() == "try_statement" && p.child_by_field_name("body") == Some(cur) {
            let catches = kids(p).into_iter().filter(|c| c.kind() == "except_clause").any(|h| {
                kids(h).first().is_some_and(|t| {
                    let t = if t.kind() == "as_pattern" { t.named_child(0).unwrap_or(*t) } else { *t };
                    text(t, src).contains("NameError")
                })
            });
            if catches {
                return true;
            }
        }
        cur = p;
    }
    false
}

fn undefined_name(root: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(model) = ctx.semantic else { return };
    if model.has_star_import {
        return;
    }
    let extra = extra_builtins(ctx);
    for r in &model.references {
        if r.symbol.is_some() || !r.read || codelaw_semantic::PYTHON_BUILTINS.contains(&r.name.as_str()) || extra.contains(&r.name) {
            continue;
        }
        if implicit_name(model, r.scope, &r.name) || guarded_by_name_error(root, r.start, ctx.source) {
            continue;
        }
        out.push(NativeFinding { start: r.start, end: r.end, message: format!("`{}` is not defined", r.name), edits: Vec::new() });
    }
}
py_rule!(UndefinedName, "py/undefined-name", ["module"], true, undefined_name);

/// The string elements of module-level `__all__` definitions.
pub(crate) fn dunder_all_strings<'t>(root: Node<'t>, src: &str) -> Vec<Node<'t>> {
    // Module-level statements, including those inside `if` / `try` blocks.
    let mut stmts = Vec::new();
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        for c in kids(n) {
            match c.kind() {
                "function_definition" | "class_definition" | "decorated_definition" => {}
                "if_statement" | "elif_clause" | "else_clause" | "try_statement" | "except_clause" | "finally_clause" | "block" | "with_statement" => {
                    stack.push(c)
                }
                _ if n.kind() == "module" || n.kind() == "block" => stmts.push(c),
                _ => {}
            }
        }
    }
    stmts.sort_by_key(|s| s.start_byte());
    let mut out = Vec::new();
    for stmt in stmts {
        let Some(e) = stmt.named_child(0).filter(|_| stmt.kind() == "expression_statement") else { continue };
        let value = match e.kind() {
            "assignment" | "augmented_assignment" => {
                if e.child_by_field_name("left").map(|l| text(l, src)) != Some("__all__") {
                    continue;
                }
                e.child_by_field_name("right")
            }
            "call" => {
                let f = e.child_by_field_name("function").map(|f| text(f, src)).unwrap_or("");
                if f != "__all__.extend" && f != "__all__.append" {
                    continue;
                }
                let args = call_args(e, src);
                if f == "__all__.append" {
                    out.extend(args.positional.into_iter().filter(|a| a.kind() == "string"));
                    continue;
                }
                args.positional.first().copied()
            }
            _ => continue,
        };
        if let Some(v) = value
            && matches!(v.kind(), "list" | "tuple" | "expression_list")
        {
            out.extend(kids(v).into_iter().filter(|s| s.kind() == "string"));
        }
    }
    out
}

fn undefined_export(root: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(model) = ctx.semantic else { return };
    if model.has_star_import {
        return;
    }
    for s in dunder_all_strings(root, ctx.source) {
        let Some(name) = string_value(s, ctx.source) else { continue };
        if !model.scopes[0].bindings.contains_key(&name) && !codelaw_semantic::PYTHON_BUILTINS.contains(&name.as_str()) {
            out.push(hit(s, format!("`{name}` is listed in `__all__` but not defined in the module")));
        }
    }
}
py_rule!(UndefinedExport, "py/undefined-export", ["module"], true, undefined_export);

fn undefined_local(root: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(model) = ctx.semantic else { return };
    for s in &model.symbols {
        if model.scopes[s.scope].kind != ScopeKind::Function || s.kind == SymbolKind::Param {
            continue;
        }
        let Some(&first) = s.decls.iter().min() else { continue };
        let Some(decl) = ident_at(root, first.0, first.1) else { continue };
        let rhs = match bind_ctx(decl) {
            (Bind::Assign, Some(a)) => a.child_by_field_name("right"),
            _ => None,
        };
        // Defined in an enclosing function or the module.
        let mut outer = model.scopes[s.scope].parent;
        let mut defined_outside = false;
        while let Some(sc) = outer {
            if model.scopes[sc].kind != ScopeKind::Class && model.scopes[sc].bindings.contains_key(&s.name) {
                defined_outside = true;
                break;
            }
            outer = model.scopes[sc].parent;
        }
        if !defined_outside {
            continue;
        }
        for &r in &s.refs {
            let r = &model.references[r];
            if !r.read || r.scope != s.scope {
                continue;
            }
            let before = r.start <= first.0 || rhs.is_some_and(|x| x.start_byte() <= r.start && r.start < x.end_byte());
            if before {
                out.push(NativeFinding {
                    start: r.start,
                    end: r.end,
                    message: format!("`{}` is read here before this function assigns it, so it is never the outer `{}`", s.name, s.name),
                    edits: Vec::new(),
                });
                break;
            }
        }
    }
}
py_rule!(UndefinedLocal, "py/undefined-local", ["module"], true, undefined_local);

/// Whether a function calls `locals()` in its own body.
fn calls_locals(func: Node<'_>, src: &str) -> bool {
    let mut stack = kids(func);
    while let Some(n) = stack.pop() {
        if matches!(n.kind(), "function_definition" | "lambda" | "class_definition") {
            continue;
        }
        if n.kind() == "call" && n.child_by_field_name("function").is_some_and(|f| text(f, src) == "locals") {
            return true;
        }
        stack.extend(kids(n));
    }
    false
}

fn unused_locals(root: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>, annotations: bool) {
    let Some(model) = ctx.semantic else { return };
    let src = ctx.source;
    let mut locals_cache: BTreeMap<usize, bool> = BTreeMap::new();
    for s in &model.symbols {
        let scope = &model.scopes[s.scope];
        if s.kind != SymbolKind::Variable || s.name.starts_with('_') {
            continue;
        }
        if scope.kind != ScopeKind::Function {
            // Outside functions only an unused `except … as name` is reported.
            let last = s.decls.iter().copied().max().unwrap_or((0, 0));
            let is_except = !annotations && ident_at(root, last.0, last.1).is_some_and(|i| bind_ctx(i).0 == Bind::Except);
            if !is_except || scope.kind == ScopeKind::Class {
                continue;
            }
        }
        // Read anywhere, or written from a nested function (`nonlocal`).
        if s.refs.iter().any(|&r| {
            let r = &model.references[r];
            r.read || r.scope != s.scope || model.scope_at(r.start) != s.scope
        }) {
            continue;
        }
        let uses_locals = *locals_cache.entry(s.scope).or_insert_with(|| {
            root.descendant_for_byte_range(scope.start, scope.end)
                .map(|mut f| {
                    while !matches!(f.kind(), "function_definition" | "lambda") {
                        match f.parent() {
                            Some(p) => f = p,
                            None => return false,
                        }
                    }
                    calls_locals(f, src)
                })
                .unwrap_or(false)
        });
        if uses_locals {
            continue;
        }
        let last = s.decls.iter().copied().max().unwrap_or((0, 0));
        let Some(id) = ident_at(root, last.0, last.1) else { continue };
        let (kind, at) = bind_ctx(id);
        match (kind, annotations) {
            (Bind::AnnOnly, true) => {
                let edits = at.map(statement_of).and_then(|st| remove_stmt(st, src)).unwrap_or_default();
                out.push(hit_fix(id, format!("`{}` is annotated but never used", s.name), edits));
            }
            (Bind::Assign | Bind::With | Bind::Except | Bind::Walrus, false) => {
                if unpacking_partly_used(id, at, model) {
                    continue;
                }
                let edits = unused_variable_edits(id, kind, at, src).unwrap_or_default();
                out.push(hit_fix(id, format!("`{}` is assigned but never used", s.name), edits));
            }
            _ => {}
        }
    }
}

/// For a name bound by unpacking (`a, b = …`): Ruff reports unpacking only
/// from a tuple or list literal, where each name is a separate assignment.
fn unpacking_partly_used(_id: Node<'_>, at: Option<Node<'_>>, _model: &SemanticModel) -> bool {
    let Some(a) = at else { return false };
    match a.kind() {
        "assignment" => {
            let Some(left) = a.child_by_field_name("left") else { return false };
            left.kind() != "identifier" && !a.child_by_field_name("right").is_some_and(|r| matches!(r.kind(), "expression_list" | "tuple" | "list"))
        }
        "as_pattern" => a.child_by_field_name("alias").and_then(|t| kids(t).first().copied()).is_none_or(|t| t.kind() != "identifier"),
        _ => false,
    }
}

fn in_try(n: Node<'_>) -> bool {
    let mut cur = n;
    while let Some(p) = cur.parent() {
        match p.kind() {
            "try_statement" => return true,
            "function_definition" | "class_definition" => return false,
            _ => {}
        }
        cur = p;
    }
    false
}

fn unused_variable_edits(id: Node<'_>, kind: Bind, at: Option<Node<'_>>, src: &str) -> Option<Vec<TextEdit>> {
    let at = at?;
    match kind {
        Bind::Except => {
            let value = at.named_child(0)?;
            Some(vec![TextEdit::new(value.end_byte(), at.end_byte(), "")])
        }
        Bind::Walrus => {
            let value = at.child_by_field_name("value")?;
            Some(vec![TextEdit::new(at.start_byte(), at.end_byte(), text(value, src))])
        }
        Bind::Assign => {
            if at.child_by_field_name("left") != Some(id) {
                return None;
            }
            let stmt = at.parent().filter(|p| p.kind() == "expression_statement")?;
            let right = at.child_by_field_name("right")?;
            if right.kind() == "assignment" {
                return None;
            }
            if is_pure(right) {
                remove_stmt(stmt, src)
            } else if matches!(right.kind(), "yield" | "expression_list") {
                None
            } else {
                Some(vec![TextEdit::new(at.start_byte(), right.start_byte(), "")])
            }
        }
        _ => None,
    }
}

fn unused_variable(root: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    unused_locals(root, ctx, out, false)
}
fn unused_annotation(root: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    unused_locals(root, ctx, out, true)
}
py_rule!(UnusedVariable, "py/unused-variable", ["module"], true, unused_variable);
py_rule!(UnusedAnnotation, "py/unused-annotation", ["module"], true, unused_annotation);

fn raise_not_implemented(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let Some(e) = kids(n).first().copied() else { return };
    let target = if e.kind() == "call" { e.child_by_field_name("function") } else { Some(e) };
    let Some(t) = target.filter(|t| t.kind() == "identifier" && text(*t, ctx.source) == "NotImplemented") else { return };
    if ctx.semantic.is_some_and(|m| m.reference_at(t.start_byte()).is_some_and(|r| r.symbol.is_some())) {
        return;
    }
    out.push(hit_fix(t, "`NotImplemented` is a value, not an exception; raise `NotImplementedError`", vec![TextEdit::new(t.start_byte(), t.end_byte(), "NotImplementedError")]));
}
py_rule!(RaiseNotImplemented, "py/raise-not-implemented", ["raise_statement"], true, raise_not_implemented);

/// A dictionary key as a comparable string, for literal keys.
fn literal_key(n: Node<'_>, src: &str) -> Option<String> {
    let n = unparen(n);
    match n.kind() {
        "string" | "concatenated_string" => {
            if is_fstring(n, src) {
                return None;
            }
            let bytes = text(n, src).trim_start_matches(|c: char| c.is_ascii_alphabetic()).len() != text(n, src).len()
                && text(n, src).chars().take_while(|c| c.is_ascii_alphabetic()).any(|c| c == 'b' || c == 'B');
            let v = match n.kind() {
                "string" => {
                    let (_, c) = string_parts(n, src)?;
                    c
                }
                _ => kids(n).iter().filter_map(|p| string_parts(*p, src).map(|x| x.1)).collect(),
            };
            Some(format!("{}:{v}", if bytes { "b" } else { "s" }))
        }
        "integer" | "float" => Some(format!("n:{}", text(n, src).replace('_', "").to_ascii_lowercase())),
        "true" | "false" | "none" => Some(format!("k:{}", text(n, src))),
        "unary_operator" => {
            let a = n.child_by_field_name("argument")?;
            if matches!(a.kind(), "integer" | "float") { Some(format!("n:{}", text(n, src).replace([' ', '_'], ""))) } else { None }
        }
        "tuple" => {
            let parts: Option<Vec<String>> = kids(n).iter().map(|c| literal_key(*c, src)).collect();
            Some(format!("t:({})", parts?.join(",")))
        }
        _ => None,
    }
}

fn repeated_keys(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>, variables: bool) {
    let src = ctx.source;
    let mut seen: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for p in kids(n) {
        if p.kind() != "pair" {
            continue;
        }
        let (Some(k), Some(v)) = (p.child_by_field_name("key"), p.child_by_field_name("value")) else { continue };
        let key = if variables {
            let k = unparen(k);
            if k.kind() != "identifier" {
                continue;
            }
            text(k, src).to_owned()
        } else {
            match literal_key(k, src) {
                Some(x) => x,
                None => continue,
            }
        };
        let value = text(v, src).split_whitespace().collect::<String>();
        let prev = seen.entry(key).or_default();
        if !prev.is_empty() {
            let shown = text(k, src);
            let msg = if prev.contains(&value) {
                format!("The key `{shown}` repeats with the same value")
            } else {
                format!("The key `{shown}` repeats; only its last value is kept")
            };
            out.push(hit(k, msg));
        }
        prev.push(value);
    }
}
fn repeated_key_literal(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    repeated_keys(n, ctx, out, false)
}
fn repeated_key_variable(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    repeated_keys(n, ctx, out, true)
}
py_rule!(MultiValueRepeatedKeyLiteral, "py/duplicate-dict-key", ["dictionary"], false, repeated_key_literal);
py_rule!(MultiValueRepeatedKeyVariable, "py/duplicate-dict-key-variable", ["dictionary"], false, repeated_key_variable);

fn fstring_missing_placeholders(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
    let src = ctx.source;
    if n.kind() == "string" && n.parent().is_some_and(|p| p.kind() == "concatenated_string") {
        return;
    }
    let parts: Vec<Node> = if n.kind() == "string" { vec![n] } else { kids(n).into_iter().filter(|c| c.kind() == "string").collect() };
    if !parts.iter().any(|p| is_fstring(*p, src)) {
        return;
    }
    let has_placeholder = |p: &Node<'_>| children(*p).iter().any(|c| c.kind() == "interpolation");
    if parts.iter().any(has_placeholder) {
        return;
    }
    let mut edits = Vec::new();
    for p in parts.iter().filter(|p| is_fstring(**p, src)) {
        let Some(start) = children(*p).into_iter().find(|c| c.kind() == "string_start") else { continue };
        let st = text(start, src);
        if st.chars().any(|c| c == 't' || c == 'T') {
            return;
        }
        let new_start: String = st.chars().filter(|c| *c != 'f' && *c != 'F').collect();
        edits.push(TextEdit::new(start.start_byte(), start.end_byte(), new_start));
        let mut stack = vec![*p];
        while let Some(x) = stack.pop() {
            if x.kind() == "escape_interpolation" {
                let t = text(x, src);
                edits.push(TextEdit::new(x.start_byte(), x.end_byte(), &t[..1]));
            }
            stack.extend(children(x));
        }
    }
    edits.sort_by_key(|e| e.start);
    out.push(hit_fix(n, "This f-string has no placeholders; drop the `f` prefix", edits));
}
py_rule!(FStringMissingPlaceholders, "py/f-string-missing-placeholders", ["string", "concatenated_string"], false, fstring_missing_placeholders);

#[cfg(test)]
mod tests {
    use crate::run;
    use codelaw_core::Language;

    fn spans(rule: &dyn crate::NativeRule, src: &str) -> Vec<String> {
        run(rule, Language::Python, src).iter().map(|f| src[f.start..f.end].to_owned()).collect()
    }

    #[test]
    fn unused_imports_and_locals() {
        let src = "import os, sys\nfrom x import (a, b)\nprint(sys, a)\ndef f():\n    y = 1\n    _z = 2\n    w: int\n";
        assert_eq!(spans(&super::UnusedImport, src), ["os", "b"]);
        assert_eq!(spans(&super::UnusedVariable, src), ["y"]);
        assert_eq!(spans(&super::UnusedAnnotation, src), ["w"]);
    }

    #[test]
    fn redefinition_skips_forks_and_setters() {
        let src = "try:\n    import json\nexcept ImportError:\n    json = None\nclass C:\n    @property\n    def p(self): pass\n    @p.setter\n    def p(self, v): pass\ndef g(): pass\ndef g(): pass\n";
        assert_eq!(spans(&super::RedefinedWhileUnused, src), ["g"]);
    }

    #[test]
    fn undefined_local_reads_before_assignment() {
        let src = "x = 1\ndef f():\n    x += 1\ndef g():\n    print(x)\n    x = 2\ndef h():\n    x = 3\n    print(x)\n";
        assert_eq!(spans(&super::UndefinedLocal, src), ["x", "x"]);
    }
}
