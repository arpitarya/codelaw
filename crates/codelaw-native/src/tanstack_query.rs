//! Native rules behind `tanstack-query/recommended`: TanStack Query's
//! `@tanstack/eslint-plugin-query` `recommended` preset, reimplemented.
//! Each reads the option objects TanStack Query's hooks take — the query
//! key against what the query function reads, the order of an options
//! object's callbacks — or follows a hook's result through the file.

use crate::js_ast::{
    arguments, as_function, body, descendants_same_function, function_name, import_of, initial_value, is_async, is_function, is_named_import, member, member_name, member_value, members, outermost_function, root_of, unparen,
};
use crate::util::{children, named_children, text};
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::{Language, TextEdit};
use codelaw_lang::SyntaxNode;
use codelaw_semantic::{Imported, SemanticModel};
use std::collections::BTreeSet;
use tree_sitter::Node;

const JS: &[Language] = &[Language::Javascript, Language::Jsx, Language::Typescript, Language::Tsx];

fn finding(n: Node<'_>, message: impl Into<String>, edits: Vec<TextEdit>) -> NativeFinding {
    NativeFinding { start: n.start_byte(), end: n.end_byte(), message: message.into(), edits }
}

/// A TanStack Query package: `@tanstack/<framework>-query`.
fn is_tanstack(source: &str) -> bool {
    source.starts_with("@tanstack/") && source.ends_with("-query")
}

const QUERY_HOOKS: &[&str] = &["useQuery", "useQueries", "useInfiniteQuery", "useSuspenseQuery", "useSuspenseQueries", "useSuspenseInfiniteQuery"];

/// The TanStack Query hook a call calls directly, by its imported name.
fn tanstack_hook<'s>(model: &SemanticModel, call: Node<'_>, src: &'s str, hooks: &[&str]) -> Option<&'s str> {
    let f = call.child_by_field_name("function").filter(|f| f.kind() == "identifier")?;
    let name = text(f, src);
    (hooks.contains(&name) && import_of(model, f, &is_tanstack).is_some()).then_some(name)
}

// ---------------------------------------------------------------- exhaustive-deps

/// A dependency as the query key or the query function names it: `a.b.c`
/// with its root `a`, and whether it stands for all of `a`.
struct RefPath {
    path: String,
    root: String,
    whole_root: bool,
}

/// `a?.b!.c` as `a.b.c`, without whitespace; a subscript ends the path.
fn normalize(t: &str) -> String {
    let t: String = t.replace("?.", ".").chars().filter(|c| *c != '!' && !c.is_whitespace()).collect();
    match t.find('[') {
        Some(i) => t[..i].to_owned(),
        None => t,
    }
}

/// The member chain an identifier starts, up to the node its value leaves.
fn chain_top(ident: Node<'_>) -> Node<'_> {
    let mut cur = ident;
    while let Some(p) = cur.parent() {
        let up = match p.kind() {
            "member_expression" | "subscript_expression" => p.child_by_field_name("object").is_some_and(|o| o.id() == cur.id()),
            "non_null_expression" => true,
            _ => false,
        };
        if !up {
            break;
        }
        cur = p;
    }
    cur
}

fn is_callee(n: Node<'_>) -> bool {
    n.parent().is_some_and(|p| p.kind() == "call_expression" && p.child_by_field_name("function").is_some_and(|f| f.id() == n.id()))
}

fn ref_path(ident: Node<'_>, src: &str) -> RefPath {
    let name = text(ident, src).to_owned();
    let top = chain_top(ident);
    let full = normalize(text(top, src));
    let mut path = if full.is_empty() { name.clone() } else { full };
    let mut whole = path == name;
    if is_callee(top) {
        // `a.b.method()` depends on `a.b`, not on the method.
        if let Some((head, _)) = path.rsplit_once('.') {
            path = head.to_owned();
        }
        whole = false;
    }
    RefPath { whole_root: whole && path == name, root: name, path }
}

/// The query key, followed through `const key = […]` and `as` casts.
fn dereference<'t>(model: &SemanticModel, root: Node<'t>, mut n: Node<'t>) -> Node<'t> {
    let mut seen = BTreeSet::new();
    while seen.insert(n.id()) {
        n = unparen(n);
        if !matches!(n.kind(), "identifier" | "shorthand_property_identifier") {
            break;
        }
        match initial_value(model, root, n) {
            Some(v) => n = v,
            None => break,
        }
    }
    n
}

/// The roots and the full paths a query key names.
fn key_deps(key: Node<'_>, src: &str) -> (BTreeSet<String>, BTreeSet<String>) {
    let (mut roots, mut paths) = (BTreeSet::new(), BTreeSet::new());
    let mut stack = vec![key];
    while let Some(n) = stack.pop() {
        stack.extend(named_children(n));
        if !matches!(n.kind(), "identifier" | "shorthand_property_identifier") {
            continue;
        }
        let top = chain_top(n);
        // `a.method()` in the key covers all of `a`.
        let direct_method = is_callee(top) && top.kind() == "member_expression" && top.child_by_field_name("object").is_some_and(|o| o.id() == n.id());
        let r = ref_path(n, src);
        if r.whole_root || direct_method {
            roots.insert(r.root.clone());
            paths.insert(r.root);
        } else {
            paths.insert(r.path);
        }
    }
    (roots, paths)
}

/// The type names a symbol's declaration is annotated with.
fn annotated_types(model: &SemanticModel, root: Node<'_>, ident: Node<'_>, src: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let Some(sym) = model.symbol_at(ident.start_byte()) else { return out };
    let Some(&(start, end)) = model.symbol(sym).decls.first() else { return out };
    let Some(id) = root.descendant_for_byte_range(start, end) else { return out };
    let Some(holder) = id.parent().filter(|p| matches!(p.kind(), "variable_declarator" | "required_parameter" | "optional_parameter")) else { return out };
    if let Some(t) = holder.child_by_field_name("type") {
        let mut stack = vec![t];
        while let Some(n) = stack.pop() {
            if n.kind() == "type_identifier" {
                out.insert(text(n, src).to_owned());
            }
            stack.extend(named_children(n));
        }
    }
    out
}

/// `allowlist.<key>`, upstream's option shape: `{ variables: [], types: [] }`.
fn allowlist(ctx: &NativeContext<'_>, key: &str) -> BTreeSet<String> {
    ctx.options.get("allowlist").and_then(|a| a.get(key)).and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect()).unwrap_or_default()
}

pub struct ExhaustiveDeps;

impl NativeRule for ExhaustiveDeps {
    fn id(&self) -> &'static str {
        "tanstack-query/exhaustive-deps"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["object"]
    }
    fn needs_semantic(&self) -> bool {
        true
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let Some(model) = ctx.semantic else { return };
        let object = node.get_inner_node();
        let src = ctx.source;
        let (Some(key_member), Some(fn_member)) = (member(object, "queryKey", src), member(object, "queryFn", src)) else { return };
        let Some(key_value) = member_value(key_member) else { return };
        let Some(fn_value) = member_value(fn_member) else { return };
        // The query function, or both branches of `cond ? fn : skipToken`.
        let fns: Vec<Node> = match unparen(fn_value) {
            f if is_function(f) => vec![f],
            t if t.kind() == "ternary_expression" => {
                let skip = |n: Option<Node>| n.is_some_and(|n| n.kind() == "identifier" && text(n, src) == "skipToken");
                let (c, a) = (t.child_by_field_name("consequence"), t.child_by_field_name("alternative"));
                [c, a].into_iter().flatten().filter(|n| !skip(Some(*n))).collect()
            }
            _ => return,
        };
        let Some(component) = outermost_function(object) else { return };
        let root = root_of(object);
        let allow_vars = allowlist(ctx, "variables");
        let allow_types = allowlist(ctx, "types");
        let mut required: Vec<RefPath> = Vec::new();
        for f in &fns {
            let mut idents = Vec::new();
            let mut stack = vec![*f];
            while let Some(n) = stack.pop() {
                stack.extend(named_children(n));
                if matches!(n.kind(), "identifier" | "shorthand_property_identifier") {
                    idents.push(n);
                }
            }
            idents.sort_by_key(|n| n.start_byte());
            for n in idents {
                let Some(r) = model.reference_at(n.start_byte()) else { continue };
                let Some(sym) = r.symbol.filter(|_| !r.type_only) else { continue };
                let Some(&(decl, _)) = model.symbol(sym).decls.first() else { continue };
                // Declared in the component, outside the query function.
                let in_component = component.start_byte() <= decl && decl < component.end_byte();
                let in_fn = f.start_byte() <= decl && decl < f.end_byte();
                if !in_component || in_fn {
                    continue;
                }
                let name = text(n, src);
                let top = chain_top(n);
                let instanceof = top.parent().is_some_and(|p| p.kind() == "binary_expression" && p.child_by_field_name("operator").is_some_and(|o| text(o, src) == "instanceof"));
                let constructed = top.parent().is_some_and(|p| p.kind() == "new_expression");
                if name == "undefined" || is_callee(top) || constructed || instanceof {
                    continue;
                }
                if !allow_types.is_empty() && !annotated_types(model, root, n, src).is_disjoint(&allow_types) {
                    continue;
                }
                required.push(ref_path(n, src));
            }
        }
        if required.is_empty() {
            return;
        }
        let key = dereference(model, root, key_value);
        let (roots, paths) = key_deps(key, src);
        let mut missing: Vec<String> = Vec::new();
        for r in &required {
            if roots.contains(&r.root) || allow_vars.contains(&r.root) || paths.contains(&r.path) || missing.contains(&r.path) {
                continue;
            }
            missing.push(r.path.clone());
        }
        // A missing root covers its members.
        let all = missing.clone();
        missing.retain(|p| p.split_once('.').is_none_or(|(root, _)| !all.iter().any(|m| m == root)));
        if missing.is_empty() {
            return;
        }
        let list = missing.join(", ");
        let mut edits = Vec::new();
        if key.kind() == "array" {
            let elements: Vec<Node> = named_children(key).into_iter().filter(|e| e.kind() != "comment").collect();
            if elements.is_empty() {
                edits.push(TextEdit::new(key.start_byte(), key.end_byte(), format!("[{list}]")));
            } else {
                let cs = children(key);
                if let Some(close) = cs.last().filter(|c| c.kind() == "]") {
                    let trailing = cs.iter().rev().nth(1).is_some_and(|c| c.kind() == ",");
                    edits.push(TextEdit::new(close.start_byte(), close.start_byte(), format!("{}{list}", if trailing { " " } else { ", " })));
                }
            }
        }
        let named = missing.iter().map(|m| format!("`{m}`")).collect::<Vec<_>>().join(", ");
        out.push(finding(key_member, format!("The query key is missing {named}: the query function reads it, so the cache can't tell those queries apart"), edits));
    }
}

// ---------------------------------------------------------------- stable-query-client

pub struct StableQueryClient;

impl NativeRule for StableQueryClient {
    fn id(&self) -> &'static str {
        "tanstack-query/stable-query-client"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["new_expression"]
    }
    fn needs_semantic(&self) -> bool {
        true
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let Some(model) = ctx.semantic else { return };
        let new = node.get_inner_node();
        let src = ctx.source;
        let Some(ctor) = new.child_by_field_name("constructor").filter(|c| c.kind() == "identifier" && text(*c, src) == "QueryClient") else { return };
        if !is_named_import(model, ctor, "QueryClient", &|s| s == "@tanstack/react-query") {
            return;
        }
        let Some(decl) = new.parent().filter(|p| p.kind() == "variable_declarator") else { return };
        let Some(func) = outermost_function(new) else { return };
        let Some(name) = function_name(func, src) else { return };
        let component_or_hook = name.starts_with("use") || name.starts_with(|c: char| c.is_ascii_uppercase());
        // An async component is a React Server Component: it runs once.
        if !component_or_hook || is_async(func) {
            return;
        }
        let mut edits = Vec::new();
        if let Some(var) = decl.child_by_field_name("name").filter(|n| n.kind() == "identifier") {
            let use_state = if model.imports.iter().any(|i| i.source == "react" && matches!(&i.imported, Imported::Named(n) if n == "useState") && i.local.is_some_and(|s| model.symbol(s).name == "useState")) {
                "useState"
            } else {
                "React.useState"
            };
            // A `: QueryClient` annotation goes too: the state's type is inferred.
            edits.push(TextEdit::new(decl.start_byte(), new.end_byte(), format!("[{}] = {use_state}(() => {})", text(var, src), text(new, src))));
        }
        out.push(finding(decl, "A `QueryClient` made during render is a new, empty cache on every render; make it once, outside the component or in `useState`", edits));
    }
}

// ---------------------------------------------------------------- no-rest-destructuring

fn has_rest(pattern: Node<'_>) -> bool {
    pattern.kind() == "object_pattern" && named_children(pattern).iter().any(|c| c.kind() == "rest_pattern")
}

/// The direct TanStack query hook a binding was initialised with.
fn query_result_hook<'s>(model: &SemanticModel, root: Node<'_>, ident: Node<'_>, src: &'s str, hooks: &[&str]) -> Option<&'s str> {
    let value = initial_value(model, root, ident)?;
    let call = unparen(value);
    (call.kind() == "call_expression").then(|| tanstack_hook(model, call, src, hooks)).flatten()
}

const REST_MESSAGE: &str = "Rest destructuring a query result reads every field, so the component re-renders on every change to the query; destructure the fields you use";

pub struct NoRestDestructuring;

impl NativeRule for NoRestDestructuring {
    fn id(&self) -> &'static str {
        "tanstack-query/no-rest-destructuring"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["variable_declarator", "spread_element"]
    }
    fn needs_semantic(&self) -> bool {
        true
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let Some(model) = ctx.semantic else { return };
        let n = node.get_inner_node();
        let src = ctx.source;
        let root = root_of(n);
        if n.kind() == "spread_element" {
            // `{ ...query }` in an object; a JSX spread is a prop list, as upstream.
            if n.parent().is_some_and(|p| p.kind() == "jsx_expression") {
                return;
            }
            if let Some(arg) = n.named_child(0).filter(|a| a.kind() == "identifier")
                && query_result_hook(model, root, arg, src, QUERY_HOOKS).is_some()
            {
                out.push(finding(n, REST_MESSAGE, Vec::new()));
            }
            return;
        }
        let (Some(name), Some(value)) = (n.child_by_field_name("name"), n.child_by_field_name("value")) else { return };
        let value = unparen(value);
        if value.kind() == "identifier" {
            // `const q = useQuery(…); const { data, ...rest } = q;`
            if has_rest(name) && query_result_hook(model, root, value, src, QUERY_HOOKS).is_some() {
                out.push(finding(n, REST_MESSAGE, Vec::new()));
            }
            return;
        }
        if value.kind() != "call_expression" {
            return;
        }
        let Some(hook) = tanstack_hook(model, value, src, QUERY_HOOKS) else { return };
        if matches!(hook, "useQueries" | "useSuspenseQueries") {
            if name.kind() == "array_pattern" {
                for element in named_children(name).into_iter().filter(|e| has_rest(*e)) {
                    out.push(finding(element, REST_MESSAGE, Vec::new()));
                }
            }
        } else if has_rest(name) {
            out.push(finding(n, REST_MESSAGE, Vec::new()));
        }
    }
}

// ---------------------------------------------------------------- no-unstable-deps

const DEPS_HOOKS: &[&str] = &["useEffect", "useCallback", "useMemo"];

/// The React hook with a dependency list a call calls: `useEffect`,
/// `React.useMemo`, or a local alias imported from `react`.
fn deps_hook(model: &SemanticModel, call: Node<'_>, src: &str) -> Option<&'static str> {
    let f = call.child_by_field_name("function")?;
    match f.kind() {
        "identifier" => {
            let name = text(f, src);
            if let Some(h) = DEPS_HOOKS.iter().find(|h| **h == name) {
                return Some(h);
            }
            let i = import_of(model, f, &|s| s == "react")?;
            match &i.imported {
                Imported::Named(n) => DEPS_HOOKS.iter().find(|h| *h == n).copied(),
                _ => None,
            }
        }
        "member_expression" => {
            let obj = f.child_by_field_name("object")?;
            let prop = text(f.child_by_field_name("property")?, src);
            (obj.kind() == "identifier" && text(obj, src) == "React").then(|| DEPS_HOOKS.iter().find(|h| **h == prop).copied()).flatten()
        }
        _ => None,
    }
}

/// `useQueries({ combine })` can return a stable value.
fn has_combine(call: Node<'_>, src: &str) -> bool {
    arguments(call).first().is_some_and(|a| a.kind() == "object" && member(*a, "combine", src).is_some())
}

/// The TanStack hook a call returns the result of: called directly, or
/// through a custom hook in this file that returns one.
fn unstable_source<'s>(model: &SemanticModel, root: Node<'_>, call: Node<'_>, src: &'s str, depth: usize) -> Option<&'s str> {
    let all: Vec<&str> = std::iter::once("useMutation").chain(QUERY_HOOKS.iter().copied()).collect();
    if let Some(h) = tanstack_hook(model, call, src, &all) {
        return (!(matches!(h, "useQueries" | "useSuspenseQueries") && has_combine(call, src))).then_some(h);
    }
    if depth > 0 {
        return None;
    }
    let f = call.child_by_field_name("function").filter(|f| f.kind() == "identifier")?;
    let name = text(f, src);
    if !(name.len() > 3 && name.starts_with("use") && name[3..].starts_with(|c: char| c.is_ascii_uppercase() || c.is_ascii_digit())) {
        return None;
    }
    // The custom hook's body: a call, or a block with one `return <call>`.
    let sym = model.symbol_at(f.start_byte())?;
    let &(start, end) = model.symbol(sym).decls.first()?;
    let id = root.descendant_for_byte_range(start, end)?;
    let func = match id.parent()? {
        p if p.kind() == "function_declaration" => p,
        p if p.kind() == "variable_declarator" => as_function(p.child_by_field_name("value")?)?,
        _ => return None,
    };
    let b = body(func)?;
    let returned = if b.kind() == "statement_block" {
        let returns: Vec<Node> = named_children(b).into_iter().filter(|s| s.kind() == "return_statement").collect();
        if returns.len() != 1 {
            return None;
        }
        returns[0].named_child(0)?
    } else {
        b
    };
    let returned = unparen(returned);
    (returned.kind() == "call_expression").then(|| unstable_source(model, root, returned, src, depth + 1)).flatten()
}

/// The hook whose unstable result a dependency names.
fn unstable_binding<'s>(model: &SemanticModel, root: Node<'_>, ident: Node<'_>, src: &'s str) -> Option<&'s str> {
    let sym = model.symbol_at(ident.start_byte())?;
    let &(start, end) = model.symbol(sym).decls.first()?;
    let id = root.descendant_for_byte_range(start, end)?;
    // `const q = …`, `const [a, b] = …`, `const [a, ...rest] = …`.
    let mut p = id.parent()?;
    if p.kind() == "rest_pattern" {
        p = p.parent()?;
    }
    if p.kind() == "array_pattern" {
        p = p.parent()?;
    }
    if p.kind() != "variable_declarator" {
        return None;
    }
    let value = unparen(p.child_by_field_name("value")?);
    (value.kind() == "call_expression").then(|| unstable_source(model, root, value, src, 0)).flatten()
}

pub struct NoUnstableDeps;

impl NativeRule for NoUnstableDeps {
    fn id(&self) -> &'static str {
        "tanstack-query/no-unstable-deps"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["call_expression"]
    }
    fn needs_semantic(&self) -> bool {
        true
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let Some(model) = ctx.semantic else { return };
        let call = node.get_inner_node();
        let src = ctx.source;
        let Some(react_hook) = deps_hook(model, call, src) else { return };
        let Some(deps) = arguments(call).get(1).copied().filter(|d| d.kind() == "array") else { return };
        let root = root_of(call);
        for dep in named_children(deps).into_iter().filter(|d| d.kind() == "identifier") {
            if let Some(query_hook) = unstable_binding(model, root, dep, src) {
                out.push(finding(
                    dep,
                    format!("The result of `{query_hook}` is a new object on every render, so `{react_hook}` runs every time; destructure it and list the fields you use"),
                    Vec::new(),
                ));
            }
        }
    }
}

// ---------------------------------------------------------------- no-void-query-fn

/// Whether a function body hands back a value on some path: a `return
/// <value>`, or a `throw` (a function that only throws returns `never`).
fn may_return_value(b: Node<'_>, src: &str) -> bool {
    descendants_same_function(b).into_iter().any(|n| match n.kind() {
        "return_statement" => n.named_child(0).is_some_and(|v| !is_undefined(v, src)),
        "throw_statement" => true,
        _ => false,
    })
}

/// `undefined` or `void <expr>`.
fn is_undefined(v: Node<'_>, src: &str) -> bool {
    let v = unparen(v);
    v.kind() == "undefined" || (v.kind() == "identifier" && text(v, src) == "undefined") || (v.kind() == "unary_expression" && v.child(0).is_some_and(|o| o.kind() == "void"))
}

pub struct NoVoidQueryFn;

impl NativeRule for NoVoidQueryFn {
    fn id(&self) -> &'static str {
        "tanstack-query/no-void-query-fn"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["pair", "method_definition"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let m = node.get_inner_node();
        let src = ctx.source;
        if member_name(m, src) != Some("queryFn") || m.parent().is_none_or(|p| p.kind() != "object") {
            return;
        }
        let Some(f) = member_value(m).and_then(as_function) else { return };
        let Some(b) = body(f) else { return };
        let void = if b.kind() == "statement_block" {
            !may_return_value(b, src)
        } else {
            is_undefined(b, src)
        };
        if void {
            out.push(finding(f, "The query function returns nothing; TanStack Query caches what it returns, and `undefined` is an error", Vec::new()));
        }
    }
}

// ---------------------------------------------------------------- property order

/// Reorder the members of an options object so every member named in
/// `first` comes before every member named in `then`, keeping all other
/// members where they are. `None` when they are in order already.
fn reorder<'t>(object: Node<'t>, first: &[&str], then: &[&str], src: &str) -> Option<(Vec<Node<'t>>, Vec<Node<'t>>)> {
    let all = members(object);
    let group = |m: &Node| -> Option<usize> {
        let name = member_name(*m, src)?;
        if first.contains(&name) {
            Some(0)
        } else if then.contains(&name) {
            Some(1)
        } else {
            None
        }
    };
    let mut checked: Vec<Node> = all.iter().copied().filter(|m| group(m).is_some()).collect();
    checked.sort_by_key(|m| group(m));
    let mut it = checked.into_iter();
    let sorted: Vec<Node> = all.iter().map(|m| if group(m).is_some() { it.next().expect("one per slot") } else { *m }).collect();
    (sorted.iter().zip(&all).any(|(a, b)| a.id() != b.id())).then_some((all, sorted))
}

/// A member whose value can be moved without changing what runs first: a
/// function, a method, or a plain name.
fn movable(m: Node<'_>) -> bool {
    match m.kind() {
        "method_definition" | "shorthand_property_identifier" => true,
        "pair" => m.child_by_field_name("value").is_some_and(|v| {
            let v = unparen(v);
            is_function(v) || matches!(v.kind(), "identifier" | "member_expression")
        }),
        _ => false,
    }
}

fn order_check(call: Node<'_>, src: &str, functions: &[&str], first: &[&str], then: &[&str], why: &str, out: &mut Vec<NativeFinding>) {
    let Some(f) = call.child_by_field_name("function").filter(|f| f.kind() == "identifier") else { return };
    let name = text(f, src);
    if !functions.contains(&name) {
        return;
    }
    let Some(object) = arguments(call).first().copied().filter(|a| a.kind() == "object") else { return };
    let Some((all, sorted)) = reorder(object, first, then, src) else { return };
    let mut edits = Vec::new();
    let names: BTreeSet<&str> = all.iter().filter_map(|m| member_name(*m, src)).filter(|n| first.contains(n) || then.contains(n)).collect();
    let duplicate = names.len() != all.iter().filter_map(|m| member_name(*m, src)).filter(|n| first.contains(n) || then.contains(n)).count();
    if !duplicate && sorted.iter().zip(&all).all(|(a, b)| a.id() == b.id() || (movable(*a) && movable(*b))) {
        let mut text_out = String::new();
        for (i, m) in sorted.iter().enumerate() {
            text_out.push_str(text(*m, src));
            if let Some(next) = all.get(i + 1) {
                text_out.push_str(&src[all[i].end_byte()..next.start_byte()]);
            }
        }
        edits.push(TextEdit::new(all[0].start_byte(), all[all.len() - 1].end_byte(), text_out));
    }
    let a = first.iter().map(|x| format!("`{x}`")).collect::<Vec<_>>().join(", ");
    let b = then.iter().map(|x| format!("`{x}`")).collect::<Vec<_>>().join(" and ");
    out.push(finding(object, format!("In `{name}`'s options, {a} comes before {b}: {why}"), edits));
}

pub struct InfiniteQueryPropertyOrder;

impl NativeRule for InfiniteQueryPropertyOrder {
    fn id(&self) -> &'static str {
        "tanstack-query/infinite-query-property-order"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["call_expression"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        order_check(
            node.get_inner_node(),
            ctx.source,
            &["infiniteQueryOptions", "useInfiniteQuery", "useSuspenseInfiniteQuery"],
            &["queryFn"],
            &["getPreviousPageParam", "getNextPageParam"],
            "TypeScript infers the page type from `queryFn`, in the order the options are written",
            out,
        );
    }
}

pub struct MutationPropertyOrder;

impl NativeRule for MutationPropertyOrder {
    fn id(&self) -> &'static str {
        "tanstack-query/mutation-property-order"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["call_expression"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        order_check(
            node.get_inner_node(),
            ctx.source,
            &["useMutation"],
            &["onMutate"],
            &["onError", "onSettled"],
            "TypeScript infers the context `onMutate` returns, in the order the options are written",
            out,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(rule: &dyn NativeRule, lang: Language, src: &str) -> Vec<(String, Option<String>)> {
        crate::run(rule, lang, src).into_iter().map(|f| (src[f.start..f.end].to_owned(), f.edits.first().map(|e| e.text.clone()))).collect()
    }

    const IMPORTS: &str = "import { useQuery, useQueries, useMutation, useInfiniteQuery, QueryClient } from '@tanstack/react-query';\n";

    #[test]
    fn exhaustive_deps() {
        let src = format!(
            "{IMPORTS}function Todo({{ id, filters }}) {{\n  const a = useQuery({{ queryKey: ['todo'], queryFn: () => fetchTodo(id, filters.page) }});\n  const b = useQuery({{ queryKey: ['todo', id, filters], queryFn: () => fetchTodo(id, filters.page) }});\n  const key = ['todo', id];\n  const c = useQuery({{ queryKey: key, queryFn: () => api.get(id) }});\n  const d = useQuery({{ queryKey: [], queryFn: enabled ? () => get(id) : skipToken }});\n  const e = useQuery({{ queryKey: ['x'], queryFn: ({{ signal }}) => get(signal) }});\n}}"
        );
        let f = found(&ExhaustiveDeps, Language::Tsx, &src);
        assert_eq!(f.len(), 2, "{f:#?}");
        assert_eq!(f[0].1.as_deref(), Some(", id, filters.page"));
        assert_eq!(f[1].1.as_deref(), Some("[id]"));
    }

    #[test]
    fn stable_query_client() {
        let src = format!("{IMPORTS}function App() {{\n  const queryClient = new QueryClient();\n}}\nconst client = new QueryClient();\nasync function Page() {{ const c = new QueryClient(); }}\nfunction helper() {{ const c = new QueryClient(); }}");
        let f = found(&StableQueryClient, Language::Tsx, &src);
        assert_eq!(f, [("queryClient = new QueryClient()".to_owned(), Some("[queryClient] = React.useState(() => new QueryClient())".to_owned()))]);
    }

    #[test]
    fn rest_destructuring() {
        let src = format!("{IMPORTS}function A() {{\n  const {{ data, ...rest }} = useQuery(o);\n  const q = useQuery(o);\n  const {{ x, ...more }} = q;\n  const all = {{ ...q }};\n  const [{{ ...r1 }}, ok] = useQueries(o);\n  const {{ data: d }} = useQuery(o);\n}}");
        assert_eq!(found(&NoRestDestructuring, Language::Tsx, &src).len(), 4);
    }

    #[test]
    fn unstable_deps() {
        let src = format!("{IMPORTS}function useTodos() {{ return useQuery(o); }}\nfunction A() {{\n  const m = useMutation(o);\n  const t = useTodos();\n  const {{ data }} = useQuery(o);\n  useEffect(() => {{}}, [m, t, data]);\n  React.useMemo(() => 1, [m]);\n}}");
        assert_eq!(found(&NoUnstableDeps, Language::Tsx, &src).len(), 3);
    }

    #[test]
    fn void_query_fn() {
        let src = "const a = { queryFn: () => { fetchTodos(); } };\nconst b = { queryFn: async () => { await save(); } };\nconst c = { queryFn: () => fetchTodos() };\nconst d = { queryFn: async () => { const r = await get(); return r.json(); } };\nconst e = { queryFn: () => undefined };\nconst f = { queryFn() { throw new Error('x'); } };";
        assert_eq!(found(&NoVoidQueryFn, Language::Tsx, src).len(), 3);
    }

    #[test]
    fn property_order() {
        let src = "useInfiniteQuery({\n  queryKey: ['p'],\n  getNextPageParam: (last) => last.next,\n  queryFn: ({ pageParam }) => get(pageParam),\n  initialPageParam: 0,\n});\nuseInfiniteQuery({ queryKey: ['p'], queryFn: get, getNextPageParam: next });";
        let f = found(&InfiniteQueryPropertyOrder, Language::Tsx, src);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].1.as_deref(), Some("queryKey: ['p'],\n  queryFn: ({ pageParam }) => get(pageParam),\n  getNextPageParam: (last) => last.next,\n  initialPageParam: 0"));
        let src = "useMutation({ onError: (e, v, ctx) => undo(ctx), mutationFn: save, onMutate: () => snapshot() });";
        let f = found(&MutationPropertyOrder, Language::Tsx, src);
        assert_eq!(f[0].1.as_deref(), Some("onMutate: () => snapshot(), mutationFn: save, onError: (e, v, ctx) => undo(ctx)"));
    }
}
