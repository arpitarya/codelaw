//! The React hooks rules beyond the two Rules of Hooks, ported from
//! eslint-plugin-react-hooks 7's React Compiler rules and
//! @eslint-react's hooks rules (W-43). Each decides from syntax and the
//! semantic model alone: a component is a PascalCase function that renders
//! JSX or calls a hook, a hook is a `use…` function, a state setter is the
//! second element of a `useState` pattern and a ref is a `useRef` result,
//! all declared in the same file. What the compiler infers beyond that —
//! values flowing through other functions, aliases, types — is left out,
//! and each rule's page says so.

use crate::js_ast::{arguments, chain_root, descendants_same_function, is_async, is_function, params, unparen};
use crate::react::{hook_called, is_component_name, is_hook_name};
use crate::util::{named_children, text};
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::{Language, TextEdit};
use codelaw_lang::SyntaxNode;
use codelaw_semantic::SemanticModel;
use tree_sitter::Node;

const JSX: &[Language] = &[Language::Javascript, Language::Jsx, Language::Typescript, Language::Tsx];

fn finding(n: Node<'_>, message: impl Into<String>, edits: Vec<TextEdit>) -> NativeFinding {
    NativeFinding { start: n.start_byte(), end: n.end_byte(), message: message.into(), edits }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Component,
    Hook,
}

fn is_jsx(n: Node<'_>) -> bool {
    matches!(n.kind(), "jsx_element" | "jsx_self_closing_element")
}

/// The call a function is the first argument of, when it is `memo(…)`,
/// `forwardRef(…)` or `React.memo(…)`.
fn component_wrapper<'t>(f: Node<'t>, src: &str) -> Option<Node<'t>> {
    let args = f.parent().filter(|p| p.kind() == "arguments")?;
    let call = args.parent()?;
    let callee = text(call.child_by_field_name("function")?, src);
    (callee.ends_with("memo") || callee.ends_with("forwardRef")).then_some(call)
}

/// The name a function is known by: its own, the variable it initialises,
/// or the variable a `memo(…)` / `forwardRef(…)` around it initialises.
fn function_name<'s>(f: Node<'_>, src: &'s str) -> Option<&'s str> {
    if let Some(n) = f.child_by_field_name("name") {
        return Some(text(n, src));
    }
    let holder = component_wrapper(f, src).unwrap_or(f);
    let parent = holder.parent()?;
    (parent.kind() == "variable_declarator").then(|| parent.child_by_field_name("name")).flatten().filter(|n| n.kind() == "identifier").map(|n| text(n, src))
}

/// The function's own nodes, nested functions left out.
fn own_nodes(f: Node<'_>) -> Vec<Node<'_>> {
    match f.child_by_field_name("body") {
        Some(b) => {
            let mut v = descendants_same_function(b);
            v.insert(0, b);
            v
        }
        None => Vec::new(),
    }
}

/// Whether a function renders: a component (PascalCase, and it renders JSX,
/// calls a hook or is wrapped in `memo` / `forwardRef`) or a hook (`use…`).
fn role(f: Node<'_>, src: &str) -> Option<Role> {
    if f.kind() == "method_definition" {
        return None;
    }
    let wrapped = component_wrapper(f, src).is_some();
    let name = function_name(f, src);
    if name.is_some_and(is_hook_name) {
        return Some(Role::Hook);
    }
    if wrapped {
        return Some(Role::Component);
    }
    if !name.is_some_and(is_component_name) {
        return None;
    }
    let renders = own_nodes(f).into_iter().any(|n| is_jsx(n) || (n.kind() == "call_expression" && hook_called(n, src).is_some()));
    renders.then_some(Role::Component)
}

/// The nearest function around a node.
fn nearest_function(n: Node<'_>) -> Option<Node<'_>> {
    std::iter::successors(n.parent(), |p| p.parent()).find(|p| is_function(*p))
}

/// The function a node runs in during render: its nearest function, when
/// that function is a component or a hook.
fn render_function<'t>(n: Node<'t>, src: &str) -> Option<(Node<'t>, Role)> {
    let f = nearest_function(n)?;
    role(f, src).map(|r| (f, r))
}

/// The `variable_declarator` that declares the symbol `ident` names.
fn declaring<'t>(model: &SemanticModel, root: Node<'t>, ident: Node<'_>) -> Option<(Node<'t>, Node<'t>)> {
    let sym = model.symbol_at(ident.start_byte())?;
    let (start, end) = *model.symbol(sym).decls.first()?;
    let id = root.descendant_for_byte_range(start, end)?;
    let decl = std::iter::successors(id.parent(), |p| p.parent()).take(3).find(|p| p.kind() == "variable_declarator")?;
    Some((decl, id))
}

fn root_of(n: Node<'_>) -> Node<'_> {
    let mut r = n;
    while let Some(p) = r.parent() {
        r = p;
    }
    r
}

/// When `ident` is a `useState` setter — the second element of
/// `const [value, setValue] = useState(…)` — the declarator.
fn state_setter<'t>(model: &SemanticModel, ident: Node<'t>, src: &str) -> Option<Node<'t>> {
    if ident.kind() != "identifier" {
        return None;
    }
    let (decl, id) = declaring(model, root_of(ident), ident)?;
    let pattern = id.parent().filter(|p| p.kind() == "array_pattern")?;
    if decl.child_by_field_name("name").map(|n| n.id()) != Some(pattern.id()) {
        return None;
    }
    let mut c = pattern.walk();
    let commas = pattern.children(&mut c).take_while(|x| x.id() != id.id()).filter(|x| x.kind() == ",").count();
    let value = unparen(decl.child_by_field_name("value")?);
    (commas == 1 && value.kind() == "call_expression" && hook_called(value, src) == Some("useState")).then_some(decl)
}

/// When `ident` names a `useRef(…)` result, the declarator.
fn ref_object<'t>(model: &SemanticModel, ident: Node<'t>, src: &str) -> Option<Node<'t>> {
    if ident.kind() != "identifier" {
        return None;
    }
    let (decl, id) = declaring(model, root_of(ident), ident)?;
    if decl.child_by_field_name("name").map(|n| n.id()) != Some(id.id()) {
        return None;
    }
    let value = unparen(decl.child_by_field_name("value")?);
    (value.kind() == "call_expression" && hook_called(value, src) == Some("useRef")).then_some(decl)
}

/// Whether `n` runs on every call of `f`: no condition, loop or `try`
/// between them, and no `return` before it.
fn unconditional(n: Node<'_>, f: Node<'_>, src: &str) -> bool {
    let mut p = n.parent();
    while let Some(a) = p {
        if a.id() == f.id() {
            break;
        }
        let conditional = match a.kind() {
            "if_statement" | "ternary_expression" | "for_statement" | "for_in_statement" | "while_statement" | "do_statement" | "switch_case" | "switch_default" | "catch_clause" | "try_statement" => true,
            "binary_expression" => {
                a.child_by_field_name("operator").is_some_and(|o| matches!(text(o, src), "&&" | "||" | "??")) && a.child_by_field_name("left").is_none_or(|l| !(l.start_byte() <= n.start_byte() && n.end_byte() <= l.end_byte()))
            }
            _ => false,
        };
        if conditional {
            return false;
        }
        p = a.parent();
    }
    !own_nodes(f).into_iter().any(|x| x.kind() == "return_statement" && x.end_byte() <= n.start_byte())
}

pub struct NoSetStateInRender;

impl NativeRule for NoSetStateInRender {
    fn id(&self) -> &'static str {
        "react/no-set-state-in-render"
    }
    fn languages(&self) -> &'static [Language] {
        JSX
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
        let Some(decl) = state_setter(model, callee, src) else { return };
        let Some((f, _)) = render_function(call, src) else { return };
        // The setter belongs to this component, and the call runs on every render.
        if nearest_function(decl).map(|d| d.id()) != Some(f.id()) || !unconditional(call, f, src) {
            return;
        }
        let name = text(callee, src);
        out.push(finding(call, format!("`{name}` is called on every render; setting state during render renders again, forever"), Vec::new()));
    }
}

const EFFECTS: &[&str] = &["useEffect", "useLayoutEffect", "useInsertionEffect"];

pub struct NoSetStateInEffect;

impl NativeRule for NoSetStateInEffect {
    fn id(&self) -> &'static str {
        "react/no-set-state-in-effect"
    }
    fn languages(&self) -> &'static [Language] {
        JSX
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["call_expression"]
    }
    fn needs_semantic(&self) -> bool {
        true
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let Some(model) = ctx.semantic else { return };
        let effect = node.get_inner_node();
        let src = ctx.source;
        let Some(hook) = hook_called(effect, src).filter(|h| EFFECTS.contains(h)) else { return };
        let Some(callback) = arguments(effect).first().map(|a| unparen(*a)).filter(|a| is_function(*a)) else { return };
        for call in own_nodes(callback) {
            if call.kind() != "call_expression" {
                continue;
            }
            let Some(callee) = call.child_by_field_name("function") else { continue };
            if state_setter(model, callee, src).is_none() {
                continue;
            }
            // A value read from a ref is the escape hatch upstream allows.
            if call.child_by_field_name("arguments").is_some_and(|a| text(a, src).contains(".current")) {
                continue;
            }
            let name = text(callee, src);
            out.push(finding(call, format!("`{name}` is called synchronously in `{hook}`; the component renders again right after every commit"), Vec::new()));
        }
    }
}

/// `ref.current == null`, `ref.current === undefined`, `!ref.current`: the
/// test of a lazy initialisation.
fn is_null_check(test: Node<'_>, member: Node<'_>, src: &str) -> bool {
    let t = unparen(test);
    match t.kind() {
        "unary_expression" => t.child_by_field_name("argument").is_some_and(|a| unparen(a).id() == member.id()),
        "binary_expression" => {
            let op = t.child_by_field_name("operator").map(|o| text(o, src)).unwrap_or("");
            let (Some(l), Some(r)) = (t.child_by_field_name("left"), t.child_by_field_name("right")) else { return false };
            let nullish = |n: Node<'_>| matches!(text(n, src), "null" | "undefined");
            matches!(op, "==" | "===") && ((unparen(l).id() == member.id() && nullish(r)) || (unparen(r).id() == member.id() && nullish(l)))
        }
        _ => false,
    }
}

/// Whether `member` is part of `if (ref.current == null) { ref.current = … }`.
fn lazy_init(member: Node<'_>, root_name: &str, src: &str) -> bool {
    for a in std::iter::successors(member.parent(), |p| p.parent()) {
        if is_function(a) {
            return false;
        }
        if a.kind() != "if_statement" {
            continue;
        }
        let Some(cond) = a.child_by_field_name("condition") else { continue };
        let tests_this_ref = descendants_same_function(cond).into_iter().any(|m| {
            m.kind() == "member_expression" && m.child_by_field_name("object").is_some_and(|o| text(o, src) == root_name) && m.child_by_field_name("property").is_some_and(|p| text(p, src) == "current") && is_null_check(cond, m, src)
        });
        if !tests_this_ref {
            continue;
        }
        let in_test = cond.start_byte() <= member.start_byte() && member.end_byte() <= cond.end_byte();
        let in_body = a.child_by_field_name("consequence").is_some_and(|c| c.start_byte() <= member.start_byte() && member.end_byte() <= c.end_byte());
        return in_test || in_body;
    }
    false
}

pub struct NoRefInRender;

impl NativeRule for NoRefInRender {
    fn id(&self) -> &'static str {
        "react/no-ref-in-render"
    }
    fn languages(&self) -> &'static [Language] {
        JSX
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["member_expression"]
    }
    fn needs_semantic(&self) -> bool {
        true
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let Some(model) = ctx.semantic else { return };
        let member = node.get_inner_node();
        let src = ctx.source;
        if member.child_by_field_name("property").is_none_or(|p| text(p, src) != "current") {
            return;
        }
        let Some(object) = member.child_by_field_name("object").map(unparen) else { return };
        let Some(decl) = ref_object(model, object, src) else { return };
        let Some((f, _)) = render_function(member, src) else { return };
        if nearest_function(decl).map(|d| d.id()) != Some(f.id()) {
            return;
        }
        let name = text(object, src);
        if lazy_init(member, name, src) {
            return;
        }
        let parent = member.parent();
        let written = parent.is_some_and(|p| matches!(p.kind(), "assignment_expression" | "augmented_assignment_expression") && p.child_by_field_name("left").map(|l| l.id()) == Some(member.id()));
        let message = if written {
            format!("`{name}.current` is written during render; write it in an effect or an event handler")
        } else {
            format!("`{name}.current` is read during render; React doesn't re-render when a ref changes, so read it in an effect or an event handler")
        };
        out.push(finding(member, message, Vec::new()));
    }
}

/// The impure calls eslint-plugin-react-hooks 7 knows (`Math.random`,
/// `Date.now`, `performance.now`), and `crypto`'s two random sources.
const IMPURE: &[(&str, &str)] = &[("Math", "random"), ("Date", "now"), ("performance", "now"), ("crypto", "randomUUID"), ("crypto", "getRandomValues")];

/// Whether an identifier names a global: no declaration in the file binds it.
fn is_global(model: &SemanticModel, ident: Node<'_>) -> bool {
    ident.kind() == "identifier" && model.reference_at(ident.start_byte()).is_some_and(|r| r.symbol.is_none())
}

pub struct Purity;

impl NativeRule for Purity {
    fn id(&self) -> &'static str {
        "react/purity"
    }
    fn languages(&self) -> &'static [Language] {
        JSX
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["call_expression", "new_expression"]
    }
    fn needs_semantic(&self) -> bool {
        true
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let Some(model) = ctx.semantic else { return };
        let n = node.get_inner_node();
        let src = ctx.source;
        let what = if n.kind() == "new_expression" {
            // `new Date()` with no argument is now; `new Date(x)` is not.
            let Some(ctor) = n.child_by_field_name("constructor") else { return };
            if text(ctor, src) != "Date" || !is_global(model, ctor) || n.child_by_field_name("arguments").is_some_and(|a| a.named_child_count() > 0) {
                return;
            }
            "new Date()".to_owned()
        } else {
            let Some(f) = n.child_by_field_name("function").map(unparen).filter(|f| f.kind() == "member_expression") else { return };
            let (Some(o), Some(p)) = (f.child_by_field_name("object"), f.child_by_field_name("property")) else { return };
            let (o_text, p_text) = (text(o, src), text(p, src));
            if !IMPURE.contains(&(o_text, p_text)) || !is_global(model, o) {
                return;
            }
            format!("{o_text}.{p_text}()")
        };
        if render_function(n, src).is_none() {
            return;
        }
        out.push(finding(n, format!("`{what}` gives a different result on every render; call it in an effect, an event handler or a `useState` initializer"), Vec::new()));
    }
}

const FUNCTION_NODES: &[&str] = &["function_declaration", "function_expression", "function", "arrow_function"];

/// Whether a class extends `Component` or `PureComponent`.
fn is_component_class(class: Node<'_>, src: &str) -> bool {
    named_children(class).into_iter().any(|c| c.kind() == "class_heritage" && text(c, src).contains("Component"))
}

pub struct NoNestedComponents;

impl NativeRule for NoNestedComponents {
    fn id(&self) -> &'static str {
        "react/no-nested-components"
    }
    fn languages(&self) -> &'static [Language] {
        JSX
    }
    fn kinds(&self) -> &'static [&'static str] {
        FUNCTION_NODES
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let f = node.get_inner_node();
        let src = ctx.source;
        let Some(name) = function_name(f, src).filter(|n| is_component_name(n)) else { return };
        if !own_nodes(f).into_iter().any(is_jsx) {
            return;
        }
        for a in std::iter::successors(f.parent(), |p| p.parent()) {
            let outer = if is_function(a) && role(a, src) == Some(Role::Component) {
                function_name(a, src).map(str::to_owned)
            } else if a.kind() == "method_definition" && a.child_by_field_name("name").is_some_and(|n| text(n, src) == "render") {
                std::iter::successors(a.parent(), |p| p.parent()).find(|c| matches!(c.kind(), "class_declaration" | "class")).filter(|c| is_component_class(*c, src)).map(|c| c.child_by_field_name("name").map(|n| text(n, src).to_owned()).unwrap_or_else(|| "a class component".to_owned()))
            } else {
                None
            };
            if let Some(outer) = outer {
                let at = f.child_by_field_name("name").or_else(|| f.parent().filter(|p| p.kind() == "variable_declarator").and_then(|p| p.child_by_field_name("name"))).unwrap_or(f);
                out.push(finding(at, format!("`{name}` is a component defined inside `{outer}`: it is a new component on every render, so its state resets; define it at the top level"), Vec::new()));
                return;
            }
        }
    }
}

/// Whether a function returns a value on some path: an expression body,
/// or a `return x` of its own.
fn returns_something(f: Node<'_>) -> bool {
    match f.child_by_field_name("body") {
        Some(b) if b.kind() == "statement_block" => own_nodes(f).into_iter().any(|n| n.kind() == "return_statement" && n.named_child_count() > 0),
        Some(_) => true,
        None => false,
    }
}

fn is_generator(f: Node<'_>) -> bool {
    let mut c = f.walk();
    f.kind().starts_with("generator") || f.children(&mut c).any(|x| !x.is_named() && x.kind() == "*")
}

pub struct UseMemo;

impl NativeRule for UseMemo {
    fn id(&self) -> &'static str {
        "react/use-memo"
    }
    fn languages(&self) -> &'static [Language] {
        JSX
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["call_expression"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let call = node.get_inner_node();
        let src = ctx.source;
        if hook_called(call, src) != Some("useMemo") {
            return;
        }
        let Some(callback) = arguments(call).first().map(|a| unparen(*a)).filter(|a| is_function(*a)) else { return };
        let problem = if is_async(callback) {
            "its callback is `async`, so it caches a promise, not the value"
        } else if is_generator(callback) {
            "its callback is a generator, so it caches an iterator, not the value"
        } else if !params(callback).is_empty() {
            "its callback takes parameters, but React calls it with none"
        } else if !returns_something(callback) {
            "its callback returns nothing; `useMemo` caches a value — side effects belong in `useEffect`"
        } else if call.parent().is_some_and(|p| p.kind() == "expression_statement") {
            "its result is not used; `useMemo` caches a value — side effects belong in `useEffect`"
        } else {
            return;
        };
        out.push(finding(call, format!("`useMemo` is misused: {problem}"), Vec::new()));
    }
}

/// eslint-react's `use-state` allow list: conversions too cheap to defer.
const CHEAP: &[&str] = &["Boolean", "String", "Number"];

/// The first call or `new` in a `useState` argument that runs on every
/// render: a call of a plain function name, not a hook and not inside a
/// function or a `use(…)`.
fn eager_call<'t>(arg: Node<'t>, src: &str) -> Option<Node<'t>> {
    if is_function(unparen(arg)) {
        return None;
    }
    // `descendants_same_function` doesn't enter a nested function.
    let mut nodes = vec![arg];
    nodes.extend(descendants_same_function(arg));
    nodes.into_iter().find(|n| {
        let callee = match n.kind() {
            "call_expression" => n.child_by_field_name("function"),
            "new_expression" => n.child_by_field_name("constructor"),
            _ => None,
        };
        let Some(callee) = callee.filter(|c| c.kind() == "identifier") else { return false };
        let name = text(callee, src);
        let in_use = n.id() != arg.id() && std::iter::successors(n.parent(), |p| p.parent()).take_while(|p| p.id() != arg.id()).chain(std::iter::once(arg)).any(|p| p.id() != n.id() && p.kind() == "call_expression" && p.child_by_field_name("function").is_some_and(|f| text(f, src) == "use"));
        !is_hook_name(name) && !CHEAP.contains(&name) && !in_use
    })
}

pub struct PreferLazyStateInit;

impl NativeRule for PreferLazyStateInit {
    fn id(&self) -> &'static str {
        "react/prefer-lazy-state-init"
    }
    fn languages(&self) -> &'static [Language] {
        JSX
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["call_expression"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let call = node.get_inner_node();
        let src = ctx.source;
        if hook_called(call, src) != Some("useState") {
            return;
        }
        let Some(arg) = arguments(call).first().copied() else { return };
        let Some(eager) = eager_call(arg, src) else { return };
        let body = text(arg, src);
        let body = if unparen(arg).kind() == "object" { format!("({body})") } else { body.to_owned() };
        let shown = text(eager, src);
        let shown = if shown.len() > 40 { format!("{}…", &shown[..shown.char_indices().take(40).last().map(|(i, c)| i + c.len_utf8()).unwrap_or(0)]) } else { shown.to_owned() };
        out.push(finding(
            arg,
            format!("`{shown}` runs on every render, though `useState` uses only the first value; pass a function: `useState(() => …)`"),
            vec![TextEdit::new(arg.start_byte(), arg.end_byte(), format!("() => {body}"))],
        ));
    }
}

/// Hooks eslint-react exempts from `no-unnecessary-use-prefix`.
const WELL_KNOWN_HOOKS: &[&str] = &["use", "useMDXComponents"];

/// Whether a function is an argument of `vi.mock(…)`, `jest.mock(…)` or
/// `mock.module(…)`.
fn in_test_mock(f: Node<'_>, src: &str) -> bool {
    std::iter::successors(f.parent(), |p| p.parent()).any(|p| {
        p.kind() == "call_expression"
            && p.child_by_field_name("function").filter(|c| c.kind() == "member_expression").is_some_and(|c| {
                let prop = c.child_by_field_name("property").map(|x| text(x, src)).unwrap_or("");
                let obj = c.child_by_field_name("object").map(|x| text(x, src)).unwrap_or("");
                prop == "mock" || (obj == "mock" && prop == "module")
            })
    })
}

pub struct NoUnnecessaryUsePrefix;

impl NativeRule for NoUnnecessaryUsePrefix {
    fn id(&self) -> &'static str {
        "react/no-unnecessary-use-prefix"
    }
    fn languages(&self) -> &'static [Language] {
        JSX
    }
    fn kinds(&self) -> &'static [&'static str] {
        FUNCTION_NODES
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let f = node.get_inner_node();
        let src = ctx.source;
        let Some(name) = function_name(f, src).filter(|n| is_hook_name(n) && !WELL_KNOWN_HOOKS.contains(n)) else { return };
        let Some(body) = f.child_by_field_name("body") else { return };
        // An empty body is a stub; a comment naming a hook marks one to come.
        if body.kind() == "statement_block" && named_children(body).iter().all(|c| c.kind() == "comment") {
            return;
        }
        // The body itself too: `(s) => useStore(api, s)` is one call.
        let mut all = vec![body];
        crate::util::descendants(body, &|_| false, &mut all);
        let calls_hook = all.iter().any(|n| {
            (n.kind() == "call_expression" && hook_called(*n, src).is_some())
                || (n.kind() == "comment" && {
                    let t = text(*n, src);
                    t.match_indices("use").any(|(i, _)| t[i + 3..].starts_with(|c: char| c == '(' || c.is_ascii_uppercase() || c.is_ascii_digit()))
                })
        });
        if calls_hook || in_test_mock(f, src) {
            return;
        }
        let at = f.child_by_field_name("name").or_else(|| f.parent().filter(|p| p.kind() == "variable_declarator").and_then(|p| p.child_by_field_name("name"))).unwrap_or(f);
        out.push(finding(at, format!("`{name}` calls no hook, so it is a plain function; drop the `use` prefix"), Vec::new()));
    }
}

pub struct Globals;

impl NativeRule for Globals {
    fn id(&self) -> &'static str {
        "react/globals"
    }
    fn languages(&self) -> &'static [Language] {
        JSX
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["assignment_expression", "augmented_assignment_expression", "update_expression"]
    }
    fn needs_semantic(&self) -> bool {
        true
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let Some(model) = ctx.semantic else { return };
        let n = node.get_inner_node();
        let src = ctx.source;
        let target = match n.kind() {
            "update_expression" => n.child_by_field_name("argument"),
            _ => n.child_by_field_name("left"),
        };
        let Some(target) = target.map(unparen) else { return };
        let Some(root) = chain_root(target).filter(|r| r.kind() == "identifier") else { return };
        // A global: unbound in the file, or declared at the module's top level.
        let outside = match model.reference_at(root.start_byte()) {
            Some(r) => r.symbol.is_none_or(|s| model.scopes[model.symbol(s).scope].parent.is_none()),
            None => false,
        };
        if !outside || render_function(n, src).is_none() {
            return;
        }
        let name = text(target, src);
        out.push(finding(n, format!("`{name}` lives outside the component and is changed during render; React may render any number of times, so change it in an effect or an event handler"), Vec::new()));
    }
}

/// Whether a `return` gives JSX: `<a/>`, `(<a/>)`, `x ? <a/> : <b/>`, `x && <a/>`.
fn returns_jsx(ret: Node<'_>) -> bool {
    fn jsxish(n: Node<'_>) -> bool {
        let n = unparen(n);
        match n.kind() {
            "jsx_element" | "jsx_self_closing_element" => true,
            "ternary_expression" => n.child_by_field_name("consequence").is_some_and(jsxish) || n.child_by_field_name("alternative").is_some_and(jsxish),
            "binary_expression" => n.child_by_field_name("right").is_some_and(jsxish),
            _ => false,
        }
    }
    ret.named_child(0).is_some_and(jsxish)
}

pub struct ErrorBoundaries;

impl NativeRule for ErrorBoundaries {
    fn id(&self) -> &'static str {
        "react/error-boundaries"
    }
    fn languages(&self) -> &'static [Language] {
        JSX
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["try_statement"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let t = node.get_inner_node();
        let src = ctx.source;
        if render_function(t, src).is_none() {
            return;
        }
        let Some(body) = t.child_by_field_name("body") else { return };
        let inner = descendants_same_function(body);
        let message = if inner.iter().any(|n| n.kind() == "return_statement" && returns_jsx(*n)) {
            "`try`/`catch` around returned JSX can't catch what the children throw: React renders them later; wrap them in an Error Boundary"
        } else if inner.iter().any(|n| n.kind() == "call_expression" && hook_called(*n, src) == Some("use")) {
            "`try`/`catch` around `use(…)` can't catch its error: `use` suspends the component, and only an Error Boundary catches what it throws"
        } else {
            return;
        };
        out.push(finding(t, message, Vec::new()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(rule: &dyn NativeRule, src: &str) -> Vec<String> {
        crate::run(rule, Language::Tsx, src).into_iter().map(|f| f.message).collect()
    }

    #[test]
    fn set_state_in_render() {
        let src = "function A() {\n  const [n, setN] = useState(0);\n  setN(1);\n  if (n) setN(2);\n  const go = () => setN(3);\n  return <p onClick={go}>{n}</p>;\n}\nfunction B() {\n  const [n, setN] = useState(0);\n  if (!n) return null;\n  setN(1);\n  return <p />;\n}\nfunction helper(setN) { setN(1); }";
        let msgs = found(&NoSetStateInRender, src);
        assert_eq!(msgs.len(), 1, "{msgs:#?}");
        assert!(msgs[0].starts_with("`setN` is called on every render"));
    }

    #[test]
    fn set_state_in_effect() {
        let src = "function A({ id }) {\n  const [n, setN] = useState(0);\n  const ref = useRef(0);\n  useEffect(() => { setN(id); }, [id]);\n  useEffect(() => { if (id) setN(1); }, [id]);\n  useEffect(() => { fetch(id).then(r => setN(r)); }, [id]);\n  useLayoutEffect(() => { setN(ref.current); }, []);\n  return n;\n}";
        assert_eq!(found(&NoSetStateInEffect, src).len(), 2);
    }

    #[test]
    fn ref_in_render() {
        let src = "function A() {\n  const ref = useRef(null);\n  const other = useRef(0);\n  if (ref.current === null) { ref.current = make(); }\n  other.current = 1;\n  const onClick = () => other.current;\n  return <p onClick={onClick}>{other.current}</p>;\n}";
        let msgs = found(&NoRefInRender, src);
        assert_eq!(msgs.len(), 2, "{msgs:#?}");
        assert!(msgs[0].contains("written") && msgs[1].contains("read"));
    }

    #[test]
    fn purity() {
        let src = "function A() {\n  const id = Math.random();\n  const now = new Date();\n  const then = new Date(0);\n  const onClick = () => Date.now();\n  return <p onClick={onClick}>{id}{now}</p>;\n}\nfunction helper() { return Math.random(); }\nfunction B({ Math }) { return <p>{Math.random()}</p>; }";
        assert_eq!(found(&Purity, src).len(), 2);
    }

    #[test]
    fn nested_components() {
        let src = "function List() {\n  function Item() { return <li />; }\n  const Row = () => <tr />;\n  const renderRow = () => <tr />;\n  return <ul><Item /><Row />{renderRow()}</ul>;\n}\nfunction Top() { return <p />; }\nclass C extends React.Component { render() { const Inner = () => <p />; return <Inner />; } }";
        let msgs = found(&NoNestedComponents, src);
        assert_eq!(msgs.len(), 3, "{msgs:#?}");
    }

    #[test]
    fn use_memo() {
        let src = "function A({ a }) {\n  const x = useMemo(async () => a, [a]);\n  const y = useMemo((b) => b, []);\n  const z = useMemo(() => { log(a); }, [a]);\n  useMemo(() => a + 1, [a]);\n  const ok = useMemo(() => a * 2, [a]);\n  return ok;\n}";
        assert_eq!(found(&UseMemo, src).len(), 4);
    }

    #[test]
    fn lazy_state_init() {
        let src = "function A() {\n  const [a] = useState(compute());\n  const [b] = useState(() => compute());\n  const [c] = useState(Number(x));\n  const [d] = useState({ items: load() });\n  const [e] = useState(JSON.parse(s));\n  const [f] = useState(new Map());\n  return a;\n}";
        let found: Vec<String> = crate::run(&PreferLazyStateInit, Language::Tsx, src).into_iter().flat_map(|f| f.edits.into_iter().map(|e| e.text)).collect();
        assert_eq!(found, ["() => compute()", "() => ({ items: load() })", "() => new Map()"]);
    }

    #[test]
    fn unnecessary_use_prefix() {
        let src = "function useA() { return 1; }\nfunction useB() { return useState(0); }\nconst useC = () => {};\nconst useD = () => format(1);\nconst useG: any = (s?: any) => useStore(api, s);\nfunction useE() { // useContext(X) later\n  return 1;\n}\nvi.mock('x', () => ({ useF: () => 1 }));\nfunction use() { return 1; }";
        let msgs = found(&NoUnnecessaryUsePrefix, src);
        assert_eq!(msgs.len(), 2, "{msgs:#?}");
        assert!(msgs[0].starts_with("`useA`") && msgs[1].starts_with("`useD`"));
    }

    #[test]
    fn globals() {
        let src = "let count = 0;\nconst cache = {};\nfunction A({ id }) {\n  count++;\n  cache[id] = 1;\n  window.last = id;\n  let local = 0;\n  local += 1;\n  const onClick = () => { count = 0; };\n  return <p onClick={onClick} />;\n}\nfunction helper() { count = 1; }";
        assert_eq!(found(&Globals, src).len(), 3);
    }

    #[test]
    fn error_boundaries() {
        let src = "function A() {\n  try { return <Child />; } catch { return null; }\n}\nfunction B({ p }) {\n  try { const v = use(p); return v; } catch { return null; }\n}\nfunction C() {\n  let v;\n  try { v = parse(); } catch { v = null; }\n  return <p>{v}</p>;\n}";
        assert_eq!(found(&ErrorBoundaries, src).len(), 2);
    }
}
