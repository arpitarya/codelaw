//! Native rules behind `react/recommended`: duplicate JSX props, HTML
//! attribute names on DOM elements, a class component's `render()` that
//! doesn't return, and the two Rules of Hooks — `rules-of-hooks` (call
//! order) and `exhaustive-deps` (dependency lists).

use crate::util::{children, is_function, named_children, returns_value, text};
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::{Language, TextEdit};
use codelaw_lang::SyntaxNode;
use codelaw_semantic::{ScopeId, SemanticModel};
use std::collections::{BTreeMap, BTreeSet};
use tree_sitter::Node;

const JSX: &[Language] = &[Language::Javascript, Language::Jsx, Language::Typescript, Language::Tsx];

fn finding(n: Node<'_>, message: impl Into<String>, edits: Vec<TextEdit>) -> NativeFinding {
    NativeFinding { start: n.start_byte(), end: n.end_byte(), message: message.into(), edits }
}

fn attributes<'t>(element: Node<'t>) -> Vec<Node<'t>> {
    named_children(element).into_iter().filter(|c| c.kind() == "jsx_attribute").collect()
}

fn attribute_name<'s>(attr: Node<'_>, src: &'s str) -> Option<&'s str> {
    attr.named_child(0).filter(|n| matches!(n.kind(), "property_identifier" | "jsx_namespace_name")).map(|n| text(n, src))
}

pub struct JsxNoDuplicateProps;

impl NativeRule for JsxNoDuplicateProps {
    fn id(&self) -> &'static str {
        "react/jsx-no-duplicate-props"
    }
    fn languages(&self) -> &'static [Language] {
        JSX
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["jsx_opening_element", "jsx_self_closing_element"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let mut seen = BTreeSet::new();
        for a in attributes(node.get_inner_node()) {
            let Some(name) = attribute_name(a, ctx.source) else { continue };
            if !seen.insert(name) {
                out.push(finding(a, format!("The prop `{name}` is given twice; the last one wins"), Vec::new()));
            }
        }
    }
}

pub struct NoUnknownProperty;

impl NativeRule for NoUnknownProperty {
    fn id(&self) -> &'static str {
        "react/no-unknown-property"
    }
    fn languages(&self) -> &'static [Language] {
        JSX
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["jsx_opening_element", "jsx_self_closing_element"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let el = node.get_inner_node();
        let Some(tag) = el.child_by_field_name("name") else { return };
        let tag = text(tag, ctx.source);
        // DOM elements only: lowercase, no dash (custom elements take any attribute).
        if !tag.starts_with(|c: char| c.is_ascii_lowercase()) || tag.contains(['-', '.']) {
            return;
        }
        let Some(serde_json::Value::Object(aliases)) = ctx.data.get("react/dom-property-aliases").map(|v| v.as_ref()) else { return };
        for a in attributes(el) {
            let Some(name_node) = a.named_child(0).filter(|n| n.kind() == "property_identifier") else { continue };
            let name = text(name_node, ctx.source);
            if let Some(react) = aliases.get(name).and_then(|v| v.as_str()) {
                out.push(finding(
                    name_node,
                    format!("`{name}` is the HTML attribute; in React the prop is `{react}`"),
                    vec![TextEdit::new(name_node.start_byte(), name_node.end_byte(), react)],
                ));
            }
        }
    }
}

/// Whether a class extends `Component` or `PureComponent` (bare or on `React`).
fn is_component_class(class: Node<'_>, src: &str) -> bool {
    children(class).into_iter().any(|c| {
        c.kind() == "class_heritage" && {
            let t = text(c, src);
            t.contains("Component") || t.contains("PureComponent")
        }
    })
}

pub struct RequireRenderReturn;

impl NativeRule for RequireRenderReturn {
    fn id(&self) -> &'static str {
        "react/require-render-return"
    }
    fn languages(&self) -> &'static [Language] {
        JSX
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["class_declaration", "class"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let class = node.get_inner_node();
        if !is_component_class(class, ctx.source) {
            return;
        }
        let Some(body) = class.child_by_field_name("body") else { return };
        for m in named_children(body) {
            if m.kind() != "method_definition" || m.child_by_field_name("name").is_none_or(|n| text(n, ctx.source) != "render") {
                continue;
            }
            if let Some(b) = m.child_by_field_name("body")
                && !returns_value(&b)
            {
                let name = m.child_by_field_name("name").expect("checked");
                out.push(finding(name, "`render()` must return what the component draws, on every path", Vec::new()));
            }
        }
    }
}

/// `useX` with a capital after `use`; `use` itself is a hook too.
fn is_hook_name(name: &str) -> bool {
    name == "use" || name.strip_prefix("use").is_some_and(|r| r.starts_with(|c: char| c.is_ascii_uppercase()))
}

fn is_component_name(name: &str) -> bool {
    name.starts_with(|c: char| c.is_ascii_uppercase())
}

/// The hook a call calls: `useX(...)` or `React.useX(...)`.
fn hook_called<'s>(call: Node<'_>, src: &'s str) -> Option<&'s str> {
    let f = call.child_by_field_name("function")?;
    let name = match f.kind() {
        "identifier" => text(f, src),
        "member_expression" => text(f.child_by_field_name("property")?, src),
        _ => return None,
    };
    is_hook_name(name).then_some(name)
}

const FUNCTION_KINDS: &[&str] = &["function_declaration", "function_expression", "function", "arrow_function", "method_definition", "generator_function_declaration"];

/// The function a node is inside, and that function's name as the Rules
/// of Hooks see it.
fn enclosing_function<'t>(n: Node<'t>, src: &str) -> Option<(Node<'t>, Option<String>, bool)> {
    let mut p = n.parent();
    while let Some(f) = p {
        if FUNCTION_KINDS.contains(&f.kind()) {
            let named = f.child_by_field_name("name").map(|x| text(x, src).to_owned());
            let name = named.or_else(|| {
                let parent = f.parent()?;
                match parent.kind() {
                    "variable_declarator" => parent.child_by_field_name("name").map(|x| text(x, src).to_owned()),
                    // `memo(() => …)` and `forwardRef(() => …)` wrap a component.
                    "arguments" => {
                        let call = parent.parent()?;
                        let callee = call.child_by_field_name("function").map(|c| text(c, src)).unwrap_or("");
                        (callee.ends_with("memo") || callee.ends_with("forwardRef")).then(|| "Component".to_owned())
                    }
                    _ => None,
                }
            });
            let is_callback = f.parent().is_some_and(|p| p.kind() == "arguments") && name.is_none();
            return Some((f, name, is_callback));
        }
        p = f.parent();
    }
    None
}

pub struct RulesOfHooks;

impl NativeRule for RulesOfHooks {
    fn id(&self) -> &'static str {
        "react/rules-of-hooks"
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
        let Some(hook) = hook_called(call, src) else { return };
        let Some((func, name, is_callback)) = enclosing_function(call, src) else {
            out.push(finding(call, format!("`{hook}` is called at the top level; hooks run only inside components and custom hooks"), Vec::new()));
            return;
        };
        if func.kind() == "method_definition" {
            out.push(finding(call, format!("`{hook}` is called in a class; hooks work only in function components"), Vec::new()));
            return;
        }
        match name.as_deref() {
            Some(n) if is_component_name(n) || is_hook_name(n) => {}
            Some(n) => {
                out.push(finding(call, format!("`{hook}` is called in `{n}`, which is neither a component (PascalCase) nor a hook (`use…`)"), Vec::new()));
                return;
            }
            None => {
                let what = if is_callback { "inside a callback" } else { "in a function that is neither a component nor a hook" };
                out.push(finding(call, format!("`{hook}` is called {what}; hooks must be called at the top level of a component or hook"), Vec::new()));
                return;
            }
        }
        // `use` may be called conditionally; every other hook may not.
        if hook == "use" {
            return;
        }
        let mut p = call.parent();
        while let Some(a) = p {
            if a.id() == func.id() {
                break;
            }
            let conditional = match a.kind() {
                "if_statement" | "ternary_expression" | "for_statement" | "for_in_statement" | "while_statement" | "do_statement" | "switch_case" | "switch_default" => true,
                "binary_expression" => a.child_by_field_name("operator").is_some_and(|o| matches!(text(o, src), "&&" | "||" | "??")) && a.child_by_field_name("left").is_none_or(|l| !(l.start_byte() <= call.start_byte() && call.end_byte() <= l.end_byte())),
                _ => false,
            };
            if conditional {
                out.push(finding(call, format!("`{hook}` is called conditionally; hooks must run in the same order on every render"), Vec::new()));
                return;
            }
            p = a.parent();
        }
        // A return before the call makes it conditional too.
        if let Some(body) = func.child_by_field_name("body") {
            let mut stack = vec![body];
            while let Some(n) = stack.pop() {
                if n.kind() == "return_statement" && n.end_byte() <= call.start_byte() {
                    out.push(finding(call, format!("`{hook}` is called after an early return; hooks must run on every render"), Vec::new()));
                    return;
                }
                let mut c = n.walk();
                stack.extend(n.children(&mut c).filter(|x| !(FUNCTION_KINDS.contains(&x.kind()) || is_function(x.kind()))));
            }
        }
    }
}

/// Which argument is the callback and which the dependency list.
fn deps_hook(name: &str) -> Option<(usize, usize, bool)> {
    Some(match name {
        "useEffect" | "useLayoutEffect" | "useInsertionEffect" => (0, 1, true),
        "useCallback" | "useMemo" => (0, 1, false),
        "useImperativeHandle" => (1, 2, true),
        _ => return None,
    })
}

/// The root name of a dependency entry (`a` of `a.b?.c`).
fn root_name(n: Node<'_>, src: &str) -> Option<String> {
    match n.kind() {
        "identifier" => Some(text(n, src).to_owned()),
        "member_expression" => n.child_by_field_name("object").and_then(|o| root_name(o, src)),
        _ => None,
    }
}

/// Whether a binding is one React keeps stable across renders: a state
/// setter, a reducer's dispatch, a ref.
fn stable(model: &SemanticModel, root: Node<'_>, symbol: usize, src: &str) -> bool {
    let (start, end) = model.symbol(symbol).decls[0];
    let Some(id) = root.descendant_for_byte_range(start, end) else { return false };
    let Some(decl) = std::iter::successors(id.parent(), |p| p.parent()).find(|p| p.kind() == "variable_declarator") else { return false };
    let Some(value) = decl.child_by_field_name("value") else { return false };
    let Some(hook) = (value.kind() == "call_expression").then(|| hook_called(value, src)).flatten() else { return false };
    match hook {
        "useRef" => true,
        "useState" | "useReducer" | "useTransition" | "useActionState" => {
            // The second element of `[value, setValue]`.
            id.parent().is_some_and(|p| p.kind() == "array_pattern" && named_children(p).iter().position(|c| c.id() == id.id()) == Some(1))
        }
        _ => false,
    }
}

fn scope_chain(model: &SemanticModel, mut s: ScopeId) -> Vec<ScopeId> {
    let mut out = vec![s];
    while let Some(p) = model.scopes[s].parent {
        out.push(p);
        s = p;
    }
    out
}

pub struct ExhaustiveDeps;

impl NativeRule for ExhaustiveDeps {
    fn id(&self) -> &'static str {
        "react/exhaustive-deps"
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
        let Some(hook) = hook_called(call, src) else { return };
        let Some((cb_at, deps_at, effect)) = deps_hook(hook) else { return };
        let Some(args) = call.child_by_field_name("arguments") else { return };
        let args = named_children(args);
        let Some(callback) = args.get(cb_at).copied().filter(|c| matches!(c.kind(), "arrow_function" | "function_expression" | "function")) else { return };
        let Some(deps) = args.get(deps_at).copied() else {
            if !effect {
                out.push(finding(call, format!("`{hook}` without a dependency list does nothing; pass one"), Vec::new()));
            }
            return;
        };
        if deps.kind() != "array" {
            out.push(finding(deps, format!("`{hook}`'s dependency list must be an array literal, so its contents can be checked"), Vec::new()));
            return;
        }
        let Some((component, _, _)) = enclosing_function(call, src) else { return };
        let root = {
            let mut r = call;
            while let Some(p) = r.parent() {
                r = p;
            }
            r
        };
        let component_scope = model.scope_at(component.start_byte() + 1);
        let callback_scope = model.scope_at(callback.child_by_field_name("body").map(|b| b.start_byte()).unwrap_or(callback.start_byte()) + 1);
        // Values the callback reads that the component declares, outside the
        // callback itself.
        let mut needed: BTreeSet<String> = BTreeSet::new();
        for r in &model.references {
            if r.start < callback.start_byte() || r.end > callback.end_byte() || r.type_only {
                continue;
            }
            let Some(sym) = r.symbol else { continue };
            let s = model.symbol(sym);
            let declared_in = scope_chain(model, s.scope);
            let in_component = declared_in.contains(&component_scope) || s.scope == component_scope;
            let inside_callback = scope_chain(model, s.scope).contains(&callback_scope);
            if !in_component || inside_callback || stable(model, root, sym, src) {
                continue;
            }
            // A reference that is only `.current` of something is a ref read.
            needed.insert(s.name.clone());
        }
        let mut listed: BTreeMap<String, Node> = BTreeMap::new();
        for d in named_children(deps) {
            if let Some(name) = root_name(d, src) {
                listed.insert(name, d);
            }
        }
        let missing: Vec<&String> = needed.iter().filter(|n| !listed.contains_key(*n)).collect();
        let mut problems = Vec::new();
        if !missing.is_empty() {
            problems.push(format!("missing {}", missing.iter().map(|m| format!("`{m}`")).collect::<Vec<_>>().join(", ")));
        }
        // An outer value (an import, a module constant) never changes between
        // renders, so listing it does nothing.
        let mut unnecessary = Vec::new();
        for (name, d) in &listed {
            let sym = model.reference_at(d.start_byte()).and_then(|r| r.symbol);
            let outer = sym.is_some_and(|s| !scope_chain(model, model.symbol(s).scope).contains(&component_scope) && model.symbol(s).scope != component_scope);
            let unused_here = !needed.contains(name) && !effect;
            if outer || unused_here {
                unnecessary.push(name.clone());
            }
        }
        if !unnecessary.is_empty() {
            problems.push(format!("unnecessary {}", unnecessary.iter().map(|m| format!("`{m}`")).collect::<Vec<_>>().join(", ")));
        }
        if problems.is_empty() {
            return;
        }
        let mut wanted: Vec<String> = listed.keys().filter(|k| !unnecessary.contains(k)).map(|k| text(listed[k], src).to_owned()).collect();
        wanted.extend(missing.iter().map(|m| (*m).clone()));
        wanted.sort();
        wanted.dedup();
        out.push(finding(
            deps,
            format!("`{hook}` has {} in its dependency list", problems.join(" and ")),
            vec![TextEdit::new(deps.start_byte(), deps.end_byte(), format!("[{}]", wanted.join(", ")))],
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(rule: &dyn NativeRule, src: &str) -> Vec<String> {
        crate::run(rule, Language::Tsx, src).into_iter().map(|f| f.message).collect()
    }

    #[test]
    fn duplicate_props_and_render() {
        assert_eq!(found(&JsxNoDuplicateProps, "const a = <div id=\"x\" title=\"a\" id=\"y\" />;").len(), 1);
        assert_eq!(found(&RequireRenderReturn, "class A extends React.Component { render() { if (x) return <p />; } }\nclass B extends Component { render() { return null; } }").len(), 1);
    }

    #[test]
    fn rules_of_hooks() {
        let src = "function App() {\n  const [a] = useState(0);\n  if (a) { useEffect(() => {}); }\n  return a && useMemo(() => 1, []);\n}\nfunction helper() { useState(); }\nconst Card = memo(() => { useRef(); return null; });\nfunction useThing() { return useContext(C); }\nfunction Early({ x }) { if (!x) return null; useState(); }\nfunction Maybe() { if (x) { use(p); } }\nuseState();\nfunction List() { items.map(() => useState()); }";
        let msgs = found(&RulesOfHooks, src);
        assert_eq!(msgs.len(), 6, "{msgs:#?}");
        assert!(msgs[0].contains("conditionally"));
        assert!(msgs[1].contains("conditionally"));
        assert!(msgs[2].contains("`helper`"));
        assert!(msgs[3].contains("early return"));
        assert!(msgs[4].contains("top level"));
        assert!(msgs[5].contains("callback"));
    }

    #[test]
    fn exhaustive_deps() {
        let src = "import { api } from './api';\nfunction App({ id, name }) {\n  const [n, setN] = useState(0);\n  const ref = useRef(null);\n  useEffect(() => { setN(id); ref.current = name; }, []);\n  const f = useCallback(() => n + 1, [n, api]);\n  const g = useMemo(() => id, [id]);\n  useEffect(() => {});\n  const h = useCallback(() => 1);\n}";
        let found: Vec<(String, String)> = crate::run(&ExhaustiveDeps, Language::Jsx, src).into_iter().map(|f| (f.message, f.edits.first().map(|e| e.text.clone()).unwrap_or_default())).collect();
        assert_eq!(found, [
            ("`useEffect` has missing `id`, `name` in its dependency list".to_string(), "[id, name]".to_string()),
            ("`useCallback` has unnecessary `api` in its dependency list".to_string(), "[n]".to_string()),
            ("`useCallback` without a dependency list does nothing; pass one".to_string(), String::new()),
        ]);
    }
}
