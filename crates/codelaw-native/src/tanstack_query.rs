//! Native rules behind `tanstack-query/recommended`: TanStack Query's
//! `@tanstack/eslint-plugin-query` `recommended` preset, reimplemented.
//! Each reads the option objects TanStack Query's hooks take — the query
//! key against what the query function reads, the order of an options
//! object's callbacks — or follows a hook's result through the file.

use crate::js_ast::{
    arguments, as_function, body, descendants_same_function, function_name, import_of, initial_value, is_async, is_function, is_named_import, member, member_name, member_value, members, outermost_function, params, root_of, unparen,
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

// ---------------------------------------------------------------- option objects (W-42)
//
// The rules below come from TanStack Query v5's guides, not from the ESLint
// plugin: they read the option objects the hooks, the `*Options` helpers,
// `new QueryClient({ defaultOptions })` and the QueryClient methods take.

/// What an option object configures.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Opts {
    Query,
    /// `useSuspenseQuery` and `useSuspenseInfiniteQuery`, and `useSuspenseQueries`' entries.
    Suspense,
    Mutation,
}

/// The QueryClient methods that take a query's options as their first argument.
const QUERY_METHODS: &[&str] = &["fetchQuery", "prefetchQuery", "ensureQueryData", "fetchInfiniteQuery", "prefetchInfiniteQuery", "ensureInfiniteQueryData"];

/// The QueryClient methods that take query filters as their first argument.
const FILTER_METHODS: &[&str] = &["isFetching", "getQueriesData", "setQueriesData", "removeQueries", "resetQueries", "cancelQueries", "invalidateQueries", "refetchQueries"];

/// The name a callee was imported under from a TanStack Query package.
fn imported_tanstack<'m>(model: &'m SemanticModel, callee: Node<'_>) -> Option<&'m str> {
    if callee.kind() != "identifier" {
        return None;
    }
    match &import_of(model, callee, &is_tanstack)?.imported {
        Imported::Named(n) => Some(n.as_str()),
        _ => None,
    }
}

/// An argument that is an object literal, through parentheses and casts.
fn object_arg<'t>(n: Option<&Node<'t>>) -> Option<Node<'t>> {
    n.map(|n| unparen(*n)).filter(|n| n.kind() == "object")
}

/// The object literals in `queries: [ … ]`, or `queries: xs.map((x) => ({ … }))`.
fn queries_entries<'t>(options: Node<'t>, src: &str) -> Vec<Node<'t>> {
    let Some(value) = member(options, "queries", src).and_then(member_value).map(unparen) else { return Vec::new() };
    match value.kind() {
        "array" => named_children(value).into_iter().map(unparen).filter(|e| e.kind() == "object").collect(),
        "call_expression" => {
            let mapped = value.child_by_field_name("function").and_then(|f| f.child_by_field_name("property")).is_some_and(|p| text(p, src) == "map");
            let f = arguments(value).first().copied().and_then(as_function);
            match (mapped, f.and_then(body)) {
                (true, Some(b)) if unparen(b).kind() == "object" => vec![unparen(b)],
                _ => Vec::new(),
            }
        }
        _ => Vec::new(),
    }
}

/// The option objects a call or a `new` expression passes, with what each configures.
fn option_objects<'t>(model: &SemanticModel, n: Node<'t>, src: &str) -> Vec<(Node<'t>, Opts)> {
    let mut out = Vec::new();
    let args = arguments(n);
    if n.kind() == "new_expression" {
        // `new QueryClient({ defaultOptions: { queries: {…}, mutations: {…} } })`
        let Some(ctor) = n.child_by_field_name("constructor") else { return out };
        if imported_tanstack(model, ctor) != Some("QueryClient") {
            return out;
        }
        let defaults = object_arg(args.first()).and_then(|o| member(o, "defaultOptions", src)).and_then(member_value).map(unparen).filter(|d| d.kind() == "object");
        if let Some(d) = defaults {
            for (key, kind) in [("queries", Opts::Query), ("mutations", Opts::Mutation)] {
                if let Some(o) = member(d, key, src).and_then(member_value).map(unparen).filter(|o| o.kind() == "object") {
                    out.push((o, kind));
                }
            }
        }
        return out;
    }
    let Some(callee) = n.child_by_field_name("function") else { return out };
    if callee.kind() == "member_expression" {
        let Some(method) = callee.child_by_field_name("property").map(|p| text(p, src)) else { return out };
        let (index, kind) = match method {
            m if QUERY_METHODS.contains(&m) => (0, Opts::Query),
            "setQueryDefaults" => (1, Opts::Query),
            "setMutationDefaults" => (1, Opts::Mutation),
            _ => return out,
        };
        if let Some(o) = object_arg(args.get(index)) {
            out.push((o, kind));
        }
        return out;
    }
    let Some(name) = imported_tanstack(model, callee) else { return out };
    let first = object_arg(args.first());
    match name {
        "useQuery" | "useInfiniteQuery" | "queryOptions" | "infiniteQueryOptions" | "usePrefetchQuery" | "usePrefetchInfiniteQuery" => out.extend(first.map(|o| (o, Opts::Query))),
        "useSuspenseQuery" | "useSuspenseInfiniteQuery" => out.extend(first.map(|o| (o, Opts::Suspense))),
        "useMutation" | "mutationOptions" => out.extend(first.map(|o| (o, Opts::Mutation))),
        "useQueries" | "useSuspenseQueries" => {
            let kind = if name == "useQueries" { Opts::Query } else { Opts::Suspense };
            if let Some(o) = first {
                out.extend(queries_entries(o, src).into_iter().map(|e| (e, kind)));
            }
        }
        _ => {}
    }
    out
}

/// The node kinds the option-object rules are called on.
const OPTION_KINDS: &[&str] = &["call_expression", "new_expression"];

// ---------------------------------------------------------------- no-renamed-v4-options

/// v4's option names v5 renamed, and the new name.
const RENAMED: &[(&str, &str)] = &[("cacheTime", "gcTime"), ("useErrorBoundary", "throwOnError")];

/// The edit that renames an object member's key, keeping its value.
fn rename_key(m: Node<'_>, new: &str, src: &str) -> Option<TextEdit> {
    match m.kind() {
        "pair" => {
            let k = m.child_by_field_name("key")?;
            Some(TextEdit::new(k.start_byte(), k.end_byte(), new))
        }
        "shorthand_property_identifier" => Some(TextEdit::new(m.start_byte(), m.end_byte(), format!("{new}: {}", text(m, src)))),
        "method_definition" => {
            let k = m.child_by_field_name("name")?;
            Some(TextEdit::new(k.start_byte(), k.end_byte(), new))
        }
        _ => None,
    }
}

pub struct NoRenamedV4Options;

impl NativeRule for NoRenamedV4Options {
    fn id(&self) -> &'static str {
        "tanstack-query/no-renamed-v4-options"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        OPTION_KINDS
    }
    fn needs_semantic(&self) -> bool {
        true
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let Some(model) = ctx.semantic else { return };
        let src = ctx.source;
        for (object, _) in option_objects(model, node.get_inner_node(), src) {
            for m in members(object) {
                let Some(name) = member_name(m, src) else { continue };
                let Some(&(old, new)) = RENAMED.iter().find(|(o, _)| *o == name) else { continue };
                // Both names written: renaming would make a duplicate key.
                let edits = if member(object, new, src).is_some() { Vec::new() } else { rename_key(m, new, src).into_iter().collect() };
                out.push(finding(m, format!("TanStack Query v5 renamed `{old}` to `{new}`; under the old name the option does nothing"), edits));
            }
        }
    }
}

// ---------------------------------------------------------------- no-removed-v4-options

/// The query options v5 removed, and what replaces each.
const REMOVED: &[(&str, &str)] = &[
    ("onSuccess", "query callbacks were removed in v5 and never run; react to the data where it is rendered, or move the work into the `queryFn`"),
    ("onError", "query callbacks were removed in v5 and never run; handle the error where the query is rendered, or with the `QueryCache`'s `onError`"),
    ("onSettled", "query callbacks were removed in v5 and never run; react to the result where it is rendered, or move the work into the `queryFn`"),
    ("keepPreviousData", "`keepPreviousData` was removed in v5; pass `placeholderData: keepPreviousData`, imported from TanStack Query"),
    ("isDataEqual", "`isDataEqual` was removed in v5; pass a function to `structuralSharing` instead"),
    ("context", "the `context` option was removed in v5; pass the `QueryClient` as the hook's second argument"),
];

pub struct NoRemovedV4Options;

impl NativeRule for NoRemovedV4Options {
    fn id(&self) -> &'static str {
        "tanstack-query/no-removed-v4-options"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        OPTION_KINDS
    }
    fn needs_semantic(&self) -> bool {
        true
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let Some(model) = ctx.semantic else { return };
        let src = ctx.source;
        for (object, kind) in option_objects(model, node.get_inner_node(), src) {
            if kind == Opts::Mutation {
                continue;
            }
            for m in members(object) {
                let Some(name) = member_name(m, src) else { continue };
                if let Some((_, why)) = REMOVED.iter().find(|(o, _)| *o == name) {
                    out.push(finding(m, format!("`{name}` is not a TanStack Query v5 query option: {why}"), Vec::new()));
                }
            }
        }
    }
}

// ---------------------------------------------------------------- no-unsupported-suspense-options

/// The options `useSuspenseQuery` and `useSuspenseInfiniteQuery` don't take.
const NOT_SUSPENSE: &[(&str, &str)] = &[
    ("enabled", "a suspense query can't be disabled; render the component only when the query should run"),
    ("placeholderData", "a suspense query has no placeholder; wrap the update that changes the key in `startTransition`"),
    ("throwOnError", "a suspense query decides when to throw; throw `error` yourself to send every error to the boundary"),
];

/// The edit that removes an object member with one comma next to it.
fn remove_member(object: Node<'_>, m: Node<'_>) -> TextEdit {
    let all = members(object);
    let i = all.iter().position(|x| x.id() == m.id()).unwrap_or(0);
    match (i.checked_sub(1).and_then(|p| all.get(p)), all.get(i + 1)) {
        (_, Some(next)) => TextEdit::new(m.start_byte(), next.start_byte(), ""),
        (Some(prev), None) => TextEdit::new(prev.end_byte(), m.end_byte(), ""),
        (None, None) => {
            // The only member: take a trailing comma with it.
            let end = m.next_sibling().filter(|s| s.kind() == ",").map_or(m.end_byte(), |c| c.end_byte());
            TextEdit::new(m.start_byte(), end, "")
        }
    }
}

pub struct NoUnsupportedSuspenseOptions;

impl NativeRule for NoUnsupportedSuspenseOptions {
    fn id(&self) -> &'static str {
        "tanstack-query/no-unsupported-suspense-options"
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
        let src = ctx.source;
        for (object, kind) in option_objects(model, node.get_inner_node(), src) {
            if kind != Opts::Suspense {
                continue;
            }
            for m in members(object) {
                let Some(name) = member_name(m, src) else { continue };
                if let Some((_, why)) = NOT_SUSPENSE.iter().find(|(o, _)| *o == name) {
                    out.push(finding(m, format!("`{name}` is not an option of the suspense hooks, which ignore it: {why}"), vec![remove_member(object, m)]));
                }
            }
        }
    }
}

// ---------------------------------------------------------------- object-syntax

/// How a v4 positional call's arguments map onto v5's one object.
#[derive(Clone, Copy)]
enum Layout {
    /// `(key, fn?, options?)` → `{ queryKey, queryFn, ...options }`.
    Query,
    /// `(fn, options?)` or `(key, fn?, options?)` → `{ mutationKey, mutationFn, ...options }`.
    Mutation,
    /// `(key, filters?, options?)` → `({ <field>: key, ...filters }, options)`.
    Filters(&'static str),
    /// `setQueriesData(key, updater, options?)` → `({ queryKey: key }, updater, options)`.
    Updater,
}

/// A v4 key as a v5 array: `'todos'` becomes `['todos']`.
fn key_text(key: Node<'_>, src: &str) -> String {
    match key.kind() {
        "string" | "template_string" => format!("[{}]", text(key, src)),
        _ => text(key, src).to_owned(),
    }
}

/// The members an argument spreads into the new object: an object
/// literal's own members, otherwise `...arg`.
fn spread(arg: Node<'_>, src: &str) -> Vec<String> {
    let a = unparen(arg);
    if a.kind() == "object" {
        members(a).into_iter().map(|m| text(m, src).to_owned()).collect()
    } else {
        vec![format!("...{}", text(arg, src))]
    }
}

fn object_text(entries: Vec<String>) -> String {
    format!("{{ {} }}", entries.join(", "))
}

/// `(key, fn, options)` with the `fn` and `options` slots read: an object
/// literal second argument is the options; anything else is the function
/// only when it is written in place or a third argument follows.
fn key_fn_options(key_field: &str, fn_field: &str, key: String, rest: &[Node<'_>], src: &str) -> Option<String> {
    let mut entries = vec![format!("{key_field}: {key}")];
    match rest {
        [] => {}
        [o] if unparen(*o).kind() == "object" => entries.extend(spread(*o, src)),
        [f] if as_function(*f).is_some() => entries.push(format!("{fn_field}: {}", text(*f, src))),
        [f, o] => {
            entries.push(format!("{fn_field}: {}", text(*f, src)));
            entries.extend(spread(*o, src));
        }
        _ => return None,
    }
    Some(object_text(entries))
}

/// v5's arguments for a v4 positional call, when they can be read off it.
fn object_syntax_args(layout: Layout, args: &[Node<'_>], src: &str) -> Option<String> {
    let first = unparen(*args.first()?);
    match layout {
        Layout::Query => key_fn_options("queryKey", "queryFn", key_text(first, src), &args[1..], src),
        Layout::Mutation if !is_key_literal(first) => match &args[1..] {
            [] => Some(object_text(vec![format!("mutationFn: {}", text(args[0], src))])),
            [o] => Some(object_text(std::iter::once(format!("mutationFn: {}", text(args[0], src))).chain(spread(*o, src)).collect())),
            _ => None,
        },
        Layout::Mutation => key_fn_options("mutationKey", "mutationFn", key_text(first, src), &args[1..], src),
        Layout::Filters(field) => {
            let mut entries = vec![format!("{field}: {}", key_text(first, src))];
            match &args[1..] {
                [] => Some(object_text(entries)),
                [filters, rest @ ..] if rest.len() <= 1 => {
                    entries.extend(spread(*filters, src));
                    Some(std::iter::once(object_text(entries)).chain(rest.iter().map(|r| text(*r, src).to_owned())).collect::<Vec<_>>().join(", "))
                }
                _ => None,
            }
        }
        Layout::Updater => match &args[1..] {
            [updater, rest @ ..] if rest.len() <= 1 => Some(
                std::iter::once(object_text(vec![format!("queryKey: {}", key_text(first, src))]))
                    .chain(std::iter::once(text(*updater, src).to_owned()))
                    .chain(rest.iter().map(|r| text(*r, src).to_owned()))
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            _ => None,
        },
    }
}

/// A first argument that is a v4 key: an array or a string.
fn is_key_literal(n: Node<'_>) -> bool {
    matches!(unparen(n).kind(), "array" | "string" | "template_string")
}

pub struct ObjectSyntax;

impl NativeRule for ObjectSyntax {
    fn id(&self) -> &'static str {
        "tanstack-query/object-syntax"
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
        let Some(callee) = call.child_by_field_name("function") else { return };
        let (name, layout) = if callee.kind() == "member_expression" {
            let Some(method) = callee.child_by_field_name("property").map(|p| text(p, src)) else { return };
            let layout = match method {
                m if QUERY_METHODS.contains(&m) => Layout::Query,
                "setQueriesData" => Layout::Updater,
                m if FILTER_METHODS.contains(&m) => Layout::Filters("queryKey"),
                _ => return,
            };
            (method, layout)
        } else {
            let Some(imported) = imported_tanstack(model, callee) else { return };
            let layout = match imported {
                "useQuery" | "useInfiniteQuery" => Layout::Query,
                "useMutation" => Layout::Mutation,
                "useIsFetching" => Layout::Filters("queryKey"),
                "useIsMutating" => Layout::Filters("mutationKey"),
                _ => return,
            };
            (text(callee, src), layout)
        };
        let args = arguments(call);
        let Some(&first) = args.first() else { return };
        // `useMutation(fn, { … })`: v5's second argument is a `QueryClient`, never an object literal.
        // Solid's and Svelte's hooks take a function that returns the options.
        let accessor_api = callee.kind() == "identifier" && import_of(model, callee, &|s| s.contains("solid") || s.contains("svelte")).is_some();
        let mutation_fn = matches!(layout, Layout::Mutation) && !accessor_api && (as_function(first).is_some() || (unparen(first).kind() == "identifier" && object_arg(args.get(1)).is_some()));
        let positional = is_key_literal(first) || mutation_fn;
        if !positional {
            return;
        }
        let edits = match (object_syntax_args(layout, &args, src), args.last()) {
            (Some(new), Some(last)) => vec![TextEdit::new(first.start_byte(), last.end_byte(), new)],
            _ => Vec::new(),
        };
        out.push(finding(call, format!("`{name}` takes one options object in TanStack Query v5; the positional arguments of v4 are not read"), edits));
    }
}

// ---------------------------------------------------------------- array-query-key

/// Whether an object literal is one TanStack Query reads: next to a
/// `queryFn`, an argument of a TanStack import or a QueryClient method, or
/// an entry of `queries`.
fn read_by_tanstack(model: &SemanticModel, object: Node<'_>, src: &str) -> bool {
    if member(object, "queryFn", src).is_some() {
        return true;
    }
    let mut p = object.parent();
    while let Some(x) = p.filter(|x| matches!(x.kind(), "parenthesized_expression" | "as_expression" | "satisfies_expression")) {
        p = x.parent();
    }
    let Some(p) = p else { return false };
    match p.kind() {
        "arguments" => {
            let Some(callee) = p.parent().and_then(|c| c.child_by_field_name("function")) else { return false };
            match callee.kind() {
                "member_expression" => callee.child_by_field_name("property").is_some_and(|m| {
                    let m = text(m, src);
                    QUERY_METHODS.contains(&m) || FILTER_METHODS.contains(&m) || matches!(m, "setQueryDefaults" | "getQueryDefaults")
                }),
                _ => imported_tanstack(model, callee).is_some(),
            }
        }
        "array" => p.parent().is_some_and(|q| q.kind() == "pair" && member_name(q, src) == Some("queries")),
        _ => false,
    }
}

pub struct ArrayQueryKey;

impl NativeRule for ArrayQueryKey {
    fn id(&self) -> &'static str {
        "tanstack-query/array-query-key"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["pair"]
    }
    fn needs_semantic(&self) -> bool {
        true
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let Some(model) = ctx.semantic else { return };
        let pair = node.get_inner_node();
        let src = ctx.source;
        if member_name(pair, src) != Some("queryKey") {
            return;
        }
        let Some(value) = pair.child_by_field_name("value") else { return };
        let v = unparen(value);
        if !matches!(v.kind(), "string" | "template_string" | "number" | "object" | "true" | "false") {
            return;
        }
        let Some(object) = pair.parent().filter(|o| o.kind() == "object") else { return };
        if !read_by_tanstack(model, object, src) {
            return;
        }
        let edit = TextEdit::new(value.start_byte(), value.end_byte(), format!("[{}]", text(value, src)));
        out.push(finding(value, "A query key is an array at the top level; wrap it: `[…]`", vec![edit]));
    }
}

// ---------------------------------------------------------------- no-mutating-set-query-data

/// The array methods that change their array in place.
const MUTATORS: &[&str] = &["push", "pop", "shift", "unshift", "splice", "sort", "reverse", "fill", "copyWithin"];

/// The identifier a single-name parameter declares.
fn param_ident(p: Node<'_>) -> Option<Node<'_>> {
    match p.kind() {
        "identifier" => Some(p),
        "assignment_pattern" => p.child_by_field_name("left").filter(|l| l.kind() == "identifier"),
        "required_parameter" | "optional_parameter" => p.child_by_field_name("pattern").and_then(param_ident),
        _ => None,
    }
}

/// The expression that changes, in place, the value a reference names:
/// `old.a = …`, `old.a++`, `delete old.a`, `old.list.push(…)`,
/// `Object.assign(old, …)`.
fn mutation_at<'t>(ident: Node<'t>, src: &str) -> Option<Node<'t>> {
    let top = chain_top(ident);
    let parent = top.parent()?;
    if top.id() == ident.id() {
        // `Object.assign(old, …)`
        let call = parent.parent().filter(|c| parent.kind() == "arguments" && c.kind() == "call_expression")?;
        let callee = call.child_by_field_name("function")?;
        let first = arguments(call).first().is_some_and(|a| a.id() == ident.id());
        return (first && normalize(text(callee, src)) == "Object.assign").then_some(call);
    }
    match parent.kind() {
        "assignment_expression" | "augmented_assignment_expression" => parent.child_by_field_name("left").filter(|l| l.id() == top.id()).map(|_| parent),
        "update_expression" => Some(parent),
        "unary_expression" if parent.child(0).is_some_and(|o| o.kind() == "delete") => Some(parent),
        "call_expression" if is_callee(top) && top.kind() == "member_expression" => {
            let method = top.child_by_field_name("property").map(|p| text(p, src))?;
            MUTATORS.contains(&method).then_some(parent)
        }
        _ => None,
    }
}

pub struct NoMutatingSetQueryData;

impl NativeRule for NoMutatingSetQueryData {
    fn id(&self) -> &'static str {
        "tanstack-query/no-mutating-set-query-data"
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
        let Some(method) = call.child_by_field_name("function").filter(|f| f.kind() == "member_expression").and_then(|f| f.child_by_field_name("property")).map(|p| text(p, src)) else { return };
        if !matches!(method, "setQueryData" | "setQueriesData") {
            return;
        }
        let Some(updater) = arguments(call).get(1).copied().and_then(as_function) else { return };
        let Some(param) = params(updater).first().copied().and_then(param_ident) else { return };
        let Some(sym) = model.symbol_at(param.start_byte()) else { return };
        let name = text(param, src);
        let mut refs = Vec::new();
        let mut stack = vec![updater];
        while let Some(n) = stack.pop() {
            stack.extend(named_children(n));
            if n.kind() == "identifier" && n.id() != param.id() && text(n, src) == name && model.reference_at(n.start_byte()).and_then(|r| r.symbol) == Some(sym) {
                refs.push(n);
            }
        }
        // `old = { ...old }` makes `old` a copy: what follows changes the copy.
        let reassigned = refs.iter().any(|r| r.parent().is_some_and(|p| p.kind() == "assignment_expression" && p.child_by_field_name("left").is_some_and(|l| l.id() == r.id())));
        if reassigned {
            return;
        }
        let mut found: Vec<Node> = refs.into_iter().filter_map(|r| mutation_at(r, src)).collect();
        found.sort_by_key(|m| m.start_byte());
        found.dedup_by_key(|m| m.id());
        for m in found {
            out.push(finding(m, format!("`{method}`'s updater changes the cached `{name}` in place; return a new value (`{{ ...{name}, … }}`, `[...{name}, …]`) so the cache and its observers see the change"), Vec::new()));
        }
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

    const V5: &str = "import { useQuery, useQueries, useMutation, useSuspenseQuery, useSuspenseQueries, useIsFetching, queryOptions, QueryClient } from '@tanstack/react-query';\n";

    #[test]
    fn renamed_v4_options() {
        let src = format!(
            "{V5}useQuery({{ queryKey: ['a'], queryFn: f, cacheTime: 1000, useErrorBoundary: true }});\nuseMutation({{ mutationFn: f, useErrorBoundary }});\nconst c = new QueryClient({{ defaultOptions: {{ queries: {{ cacheTime: 5 }} }} }});\nqueryClient.prefetchQuery({{ queryKey: ['b'], cacheTime: 1, gcTime: 2 }});\nuseQueries({{ queries: ids.map((id) => ({{ queryKey: [id], cacheTime: 1 }})) }});\nfunction useQuery2() {{ return other({{ cacheTime: 1 }}); }}"
        );
        let f = found(&NoRenamedV4Options, Language::Tsx, &src);
        let fixes: Vec<_> = f.iter().map(|(_, e)| e.clone()).collect();
        assert_eq!(fixes, [Some("gcTime".to_owned()), Some("throwOnError".to_owned()), Some("throwOnError: useErrorBoundary".to_owned()), Some("gcTime".to_owned()), None, Some("gcTime".to_owned())], "{f:#?}");
    }

    #[test]
    fn removed_v4_options() {
        let src = format!(
            "{V5}useQuery({{ queryKey: ['a'], queryFn: f, onSuccess: (d) => log(d), keepPreviousData: true, suspense: true }});\nuseMutation({{ mutationFn: f, onSuccess: () => done() }});\nqueryOptions({{ queryKey: ['b'], onError() {{}} }});\nconst c = new QueryClient({{ defaultOptions: {{ queries: {{ onSettled: s }}, mutations: {{ onError: e }} }} }});"
        );
        let f: Vec<String> = found(&NoRemovedV4Options, Language::Tsx, &src).into_iter().map(|(t, _)| t).collect();
        assert_eq!(f, ["onSuccess: (d) => log(d)", "keepPreviousData: true", "onError() {}", "onSettled: s"]);
    }

    #[test]
    fn unsupported_suspense_options() {
        let src = format!("{V5}useSuspenseQuery({{ queryKey: ['a'], queryFn: f, enabled: !!id }});\nuseSuspenseQuery({{ placeholderData: p, queryKey: ['b'] }});\nuseSuspenseQueries({{ queries: [{{ queryKey: ['c'], throwOnError: true }}] }});\nuseQuery({{ queryKey: ['d'], enabled: false }});");
        let f = found(&NoUnsupportedSuspenseOptions, Language::Tsx, &src);
        assert_eq!(f.iter().map(|(t, _)| t.as_str()).collect::<Vec<_>>(), ["enabled: !!id", "placeholderData: p", "throwOnError: true"]);
        let fixed = |i: usize| {
            let e = crate::run(&NoUnsupportedSuspenseOptions, Language::Tsx, &src).remove(i).edits.remove(0);
            format!("{}{}{}", &src[..e.start], e.text, &src[e.end..])
        };
        assert!(fixed(0).contains("useSuspenseQuery({ queryKey: ['a'], queryFn: f });"));
        assert!(fixed(1).contains("useSuspenseQuery({ queryKey: ['b'] });"));
    }

    #[test]
    fn object_syntax() {
        let src = format!(
            "{V5}useQuery(['todos'], fetchTodos, {{ staleTime: 5 }});\nuseQuery('todos', () => get());\nuseQuery(['a'], {{ queryFn: f }});\nuseMutation(addTodo, {{ onSuccess }});\nuseMutation((v) => post(v));\nqueryClient.invalidateQueries(['todos']);\nqueryClient.setQueriesData(['todos'], up);\nuseIsFetching(['todos'], {{ exact: true }});\nuseQuery(['x'], fn);\nuseQuery({{ queryKey: ['ok'] }});\nqueryClient.invalidateQueries({{ queryKey: ['ok'] }});\nuseMutation(options);\nimport {{ useMutation as useSolidMutation }} from '@tanstack/solid-query';\nuseSolidMutation(() => ({{ mutationFn: f }}));"
        );
        let f: Vec<Option<String>> = found(&ObjectSyntax, Language::Tsx, &src).into_iter().map(|(_, e)| e).collect();
        assert_eq!(
            f,
            [
                Some("{ queryKey: ['todos'], queryFn: fetchTodos, staleTime: 5 }".to_owned()),
                Some("{ queryKey: ['todos'], queryFn: () => get() }".to_owned()),
                Some("{ queryKey: ['a'], queryFn: f }".to_owned()),
                Some("{ mutationFn: addTodo, onSuccess }".to_owned()),
                Some("{ mutationFn: (v) => post(v) }".to_owned()),
                Some("{ queryKey: ['todos'] }".to_owned()),
                Some("{ queryKey: ['todos'] }, up".to_owned()),
                Some("{ queryKey: ['todos'], exact: true }".to_owned()),
                None,
            ]
        );
    }

    #[test]
    fn array_query_key() {
        let src = format!(
            "{V5}useQuery({{ queryKey: 'todos', queryFn: f }});\nqueryClient.invalidateQueries({{ queryKey: `todo-${{id}}` }});\nconst o = {{ queryKey: {{ id }}, queryFn: g }};\nuseQuery({{ queryKey: ['ok'], queryFn: f }});\nconst p = {{ queryKey: 'q' }};\nuseQuery({{ queryKey: key, queryFn: f }});"
        );
        let f = found(&ArrayQueryKey, Language::Tsx, &src);
        assert_eq!(f.into_iter().map(|(_, e)| e.unwrap_or_default()).collect::<Vec<_>>(), ["['todos']", "[`todo-${id}`]", "[{ id }]"]);
    }

    #[test]
    fn mutating_set_query_data() {
        let src = "queryClient.setQueryData(['p', id], (old) => { if (old) { old.title = 't'; } return old; });\nqc.setQueryData(k, (old) => { old.items.push(x); delete old.draft; old.count++; Object.assign(old, patch); return old; });\nqc.setQueryData(k, (old) => (old ? { ...old, title: 't' } : old));\nqc.setQueryData(k, (old) => { const copy = [...old]; copy.push(x); return copy; });\nqc.setQueryData(k, (old) => old.map((p) => { p.seen = true; return p; }));\nqc.setQueryData(k, (old) => { old = { ...old }; old.x = 1; return old; });";
        let f: Vec<String> = found(&NoMutatingSetQueryData, Language::Tsx, src).into_iter().map(|(t, _)| t).collect();
        assert_eq!(f, ["old.title = 't'", "old.items.push(x)", "delete old.draft", "old.count++", "Object.assign(old, patch)"]);
    }
}
