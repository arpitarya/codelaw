//! Native rules behind `stencil/recommended`. Each one needs the same
//! context a pattern can't carry: whether a class is a `@Component`, and
//! which Stencil decorators each member has — a field's decorators are its
//! children, a method's are the siblings before it. `ban-side-effects` is
//! the exception: it looks at every call, component or not.

use crate::util::{children, descendants, has_token, is_function, named_children, string_value, text};
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::{Language, TextEdit};
use codelaw_lang::SyntaxNode;
use std::collections::BTreeSet;
use tree_sitter::Node;

const TS: &[Language] = &[Language::Typescript, Language::Tsx];

const STENCIL_DECORATORS: &[&str] = &["Prop", "State", "Element", "Method", "Watch", "Listen", "Event", "AttachInternals"];

const LIFECYCLE: &[&str] = &[
    "connectedCallback",
    "disconnectedCallback",
    "componentWillLoad",
    "componentDidLoad",
    "componentShouldUpdate",
    "componentWillRender",
    "componentDidRender",
    "componentWillUpdate",
    "componentDidUpdate",
    "render",
    "formAssociatedCallback",
    "formResetCallback",
    "formDisabledCallback",
    "formStateRestoreCallback",
];

/// Names HTML elements already have: global attributes and `HTMLElement`
/// members a component member must not shadow.
const RESERVED: &[&str] = &[
    "accessKey", "autocapitalize", "autofocus", "class", "className", "contentEditable", "dataset", "dir", "draggable", "enterKeyHint", "hidden",
    "id", "inert", "innerHTML", "innerText", "inputMode", "lang", "nonce", "outerHTML", "popover", "slot", "spellcheck", "style", "tabIndex",
    "title", "translate", "children", "classList", "click", "focus", "blur", "remove", "attributes", "nodeName", "parentElement", "shadowRoot",
    "textContent", "tagName",
];

const DOM_EVENTS: &[&str] = &[
    "click", "dblclick", "keydown", "keyup", "keypress", "mousedown", "mouseup", "mousemove", "mouseover", "mouseout", "mouseenter",
    "mouseleave", "focus", "blur", "focusin", "focusout", "input", "change", "submit", "touchstart", "touchend", "touchmove", "pointerdown",
    "pointerup", "pointermove", "contextmenu", "wheel", "dragstart", "drop",
];

fn finding(n: Node<'_>, message: impl Into<String>, edits: Vec<TextEdit>) -> NativeFinding {
    NativeFinding { start: n.start_byte(), end: n.end_byte(), message: message.into(), edits }
}

/// A decorator's name and its arguments node.
fn decorator_parts<'t>(d: Node<'t>, src: &str) -> Option<(String, Option<Node<'t>>)> {
    let inner = d.named_child(0)?;
    match inner.kind() {
        "call_expression" => Some((text(inner.child_by_field_name("function")?, src).to_owned(), inner.child_by_field_name("arguments"))),
        "identifier" => Some((text(inner, src).to_owned(), None)),
        _ => None,
    }
}

/// Whether the class owning this body is decorated `@Component(...)`.
fn is_component<'t>(body: Node<'t>, src: &str) -> Option<Node<'t>> {
    let class = body.parent()?;
    let mut decorators: Vec<Node> = children(class).into_iter().filter(|c| c.kind() == "decorator").collect();
    if let Some(export) = class.parent().filter(|p| p.kind() == "export_statement") {
        decorators.extend(children(export).into_iter().filter(|c| c.kind() == "decorator"));
    }
    decorators.into_iter().find(|d| decorator_parts(*d, src).is_some_and(|(n, _)| n == "Component"))
}

/// One member of a class body with its decorators.
struct Member<'t> {
    node: Node<'t>,
    decorators: Vec<(String, Option<Node<'t>>, Node<'t>)>,
    name: Option<Node<'t>>,
}

fn members<'t>(body: Node<'t>, src: &str) -> Vec<Member<'t>> {
    let mut out = Vec::new();
    let mut pending: Vec<(String, Option<Node>, Node)> = Vec::new();
    for c in named_children(body) {
        match c.kind() {
            "decorator" => {
                if let Some((n, a)) = decorator_parts(c, src) {
                    pending.push((n, a, c));
                }
            }
            "method_definition" | "public_field_definition" | "field_definition" | "method_signature" => {
                let mut decorators = std::mem::take(&mut pending);
                for d in children(c).into_iter().filter(|d| d.kind() == "decorator") {
                    if let Some((n, a)) = decorator_parts(d, src) {
                        decorators.push((n, a, d));
                    }
                }
                let name = c.child_by_field_name("name").or_else(|| c.child_by_field_name("property"));
                out.push(Member { node: c, decorators, name });
            }
            "comment" => {}
            _ => pending.clear(),
        }
    }
    out
}

impl Member<'_> {
    fn has(&self, name: &str) -> bool {
        self.decorators.iter().any(|(n, _, _)| n == name)
    }
    fn modifier<'s>(&self, src: &'s str) -> Option<&'s str> {
        children(self.node).into_iter().find(|c| c.kind() == "accessibility_modifier").map(|m| text(m, src))
    }
    fn is_static(&self) -> bool {
        children(self.node).iter().any(|c| c.kind() == "static")
    }
    fn is_readonly(&self) -> bool {
        children(self.node).iter().any(|c| c.kind() == "readonly")
    }
    fn args(&self, name: &str) -> Option<Node<'_>> {
        self.decorators.iter().find(|(n, _, _)| n == name).and_then(|(_, a, _)| *a)
    }
    fn name_text<'s>(&self, src: &'s str) -> &'s str {
        self.name.map(|n| text(n, src)).unwrap_or("")
    }
    /// The first node of the member, decorators included.
    fn start(&self) -> usize {
        self.decorators.iter().map(|(_, _, d)| d.start_byte()).chain([self.node.start_byte()]).min().unwrap_or(self.node.start_byte())
    }
}

fn first_string_arg<'s>(args: Option<Node<'_>>, src: &'s str) -> Option<&'s str> {
    let a = named_children(args?).into_iter().next()?;
    (a.kind() == "string").then(|| text(a, src).trim_matches(['"', '\'', '`']))
}

/// A `{ key: value }` property of a decorator's object argument.
fn option_value<'t>(args: Option<Node<'t>>, key: &str, src: &str) -> Option<Node<'t>> {
    let obj = named_children(args?).into_iter().find(|a| a.kind() == "object")?;
    named_children(obj).into_iter().filter(|p| p.kind() == "pair").find(|p| p.child_by_field_name("key").is_some_and(|k| text(k, src) == key)).and_then(|p| p.child_by_field_name("value"))
}

macro_rules! stencil_rule {
    ($ty:ident, $id:literal, |$body:ident, $ctx:ident, $comp:ident, $out:ident| $check:block) => {
        pub struct $ty;
        impl NativeRule for $ty {
            fn id(&self) -> &'static str {
                $id
            }
            fn languages(&self) -> &'static [Language] {
                TS
            }
            fn kinds(&self) -> &'static [&'static str] {
                &["class_body"]
            }
            #[allow(unused_variables)]
            fn check(&self, node: SyntaxNode<'_>, $ctx: &NativeContext<'_>, $out: &mut Vec<NativeFinding>) {
                let $body = node.get_inner_node();
                let $comp = is_component($body, $ctx.source);
                $check
            }
        }
    };
}

stencil_rule!(BanDefaultTrue, "stencil/ban-default-true", |body, ctx, comp, out| {
    if comp.is_none() {
        return;
    }
    for m in members(body, ctx.source).into_iter().filter(|m| m.has("Prop")) {
        if let Some(v) = m.node.child_by_field_name("value").filter(|v| v.kind() == "true") {
            out.push(finding(v, "A boolean @Prop should default to false, so the attribute's presence turns it on", Vec::new()));
        }
    }
});

stencil_rule!(BanPrefix, "stencil/ban-prefix", |body, ctx, comp, out| {
    let Some(comp) = comp else { return };
    let banned: Vec<String> = ctx
        .options
        .get("prefixes")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect())
        .unwrap_or_else(|| vec!["stencil".into(), "stnl".into(), "st".into()]);
    let args = decorator_parts(comp, ctx.source).and_then(|(_, a)| a);
    if let Some(tag) = option_value(args, "tag", ctx.source) {
        let t = text(tag, ctx.source).trim_matches(['"', '\'', '`']);
        if let Some(p) = banned.iter().find(|p| t.starts_with(&format!("{p}-"))) {
            out.push(finding(tag, format!("The tag prefix `{p}-` is reserved; choose your own"), Vec::new()));
        }
    }
});

stencil_rule!(DecoratorsContext, "stencil/decorators-context", |body, ctx, comp, out| {
    if comp.is_some() {
        return;
    }
    for m in members(body, ctx.source) {
        for (n, _, d) in &m.decorators {
            if STENCIL_DECORATORS.contains(&n.as_str()) {
                out.push(finding(*d, format!("`@{n}()` only works inside a `@Component` class"), Vec::new()));
            }
        }
    }
});

stencil_rule!(HostDataDeprecated, "stencil/host-data-deprecated", |body, ctx, comp, out| {
    if comp.is_none() {
        return;
    }
    for m in members(body, ctx.source).into_iter().filter(|m| m.node.kind() == "method_definition" && m.name_text(ctx.source) == "hostData") {
        out.push(finding(m.name.expect("named"), "`hostData()` is deprecated; render a `<Host>` element instead", Vec::new()));
    }
});

stencil_rule!(MethodsMustBePublic, "stencil/methods-must-be-public", |body, ctx, comp, out| {
    if comp.is_none() {
        return;
    }
    for m in members(body, ctx.source).into_iter().filter(|m| m.has("Method")) {
        if let Some(md) = m.modifier(ctx.source).filter(|x| *x != "public") {
            out.push(finding(m.name.unwrap_or(m.node), format!("A @Method() is part of the component's public API; it can't be {md}"), Vec::new()));
        }
    }
});

stencil_rule!(PropsMustBePublic, "stencil/props-must-be-public", |body, ctx, comp, out| {
    if comp.is_none() {
        return;
    }
    for m in members(body, ctx.source).into_iter().filter(|m| m.has("Prop")) {
        if let Some(md) = m.modifier(ctx.source).filter(|x| *x != "public") {
            out.push(finding(m.name.unwrap_or(m.node), format!("A @Prop() is set from outside the component; it can't be {md}"), Vec::new()));
        }
    }
});

stencil_rule!(NoUnusedWatch, "stencil/no-unused-watch", |body, ctx, comp, out| {
    if comp.is_none() {
        return;
    }
    let ms = members(body, ctx.source);
    let watchable: BTreeSet<&str> = ms.iter().filter(|m| m.has("Prop") || m.has("State")).map(|m| m.name_text(ctx.source)).collect();
    for m in &ms {
        for (n, args, d) in &m.decorators {
            if n != "Watch" {
                continue;
            }
            if let Some(w) = first_string_arg(*args, ctx.source)
                && !watchable.contains(w)
            {
                out.push(finding(*d, format!("@Watch('{w}') watches nothing: `{w}` is not a @Prop or @State of this component"), Vec::new()));
            }
        }
    }
});

stencil_rule!(OwnMethodsMustBePrivate, "stencil/own-methods-must-be-private", |body, ctx, comp, out| {
    if comp.is_none() {
        return;
    }
    for m in members(body, ctx.source).into_iter().filter(|m| m.node.kind() == "method_definition" && m.decorators.is_empty() && !m.is_static()) {
        let name = m.name_text(ctx.source);
        if LIFECYCLE.contains(&name) || name == "constructor" || name == "hostData" || m.modifier(ctx.source).is_some_and(|x| x != "public") {
            continue;
        }
        let edit = match children(m.node).into_iter().find(|c| c.kind() == "accessibility_modifier") {
            Some(public) => TextEdit::new(public.start_byte(), public.end_byte(), "private"),
            None => TextEdit::insert(m.node.start_byte(), "private "),
        };
        out.push(finding(m.name.unwrap_or(m.node), format!("`{name}` is not part of the component's API; make it private"), vec![edit]));
    }
});

stencil_rule!(OwnPropsMustBePrivate, "stencil/own-props-must-be-private", |body, ctx, comp, out| {
    if comp.is_none() {
        return;
    }
    for m in members(body, ctx.source).into_iter().filter(|m| m.node.kind() == "public_field_definition" && m.decorators.is_empty() && !m.is_static()) {
        if m.modifier(ctx.source).is_some_and(|x| x != "public") {
            continue;
        }
        let edit = match children(m.node).into_iter().find(|c| c.kind() == "accessibility_modifier") {
            Some(public) => TextEdit::new(public.start_byte(), public.end_byte(), "private"),
            None => TextEdit::insert(m.node.start_byte(), "private "),
        };
        let name = m.name_text(ctx.source);
        out.push(finding(m.name.unwrap_or(m.node), format!("`{name}` is internal state; make it private"), vec![edit]));
    }
});

stencil_rule!(PreferVdomListener, "stencil/prefer-vdom-listener", |body, ctx, comp, out| {
    if comp.is_none() {
        return;
    }
    for m in members(body, ctx.source) {
        for (n, args, d) in &m.decorators {
            if n != "Listen" || option_value(*args, "target", ctx.source).is_some() {
                continue;
            }
            if let Some(evt) = first_string_arg(*args, ctx.source).filter(|e| DOM_EVENTS.contains(e)) {
                out.push(finding(*d, format!("Listen for `{evt}` with an `on{}` handler in `render()` instead of @Listen", evt[..1].to_uppercase() + &evt[1..]), Vec::new()));
            }
        }
    }
});

stencil_rule!(PropReadonly, "stencil/prop-readonly", |body, ctx, comp, out| {
    if comp.is_none() {
        return;
    }
    for m in members(body, ctx.source).into_iter().filter(|m| m.has("Prop") && !m.is_readonly()) {
        let mutable = option_value(m.args("Prop"), "mutable", ctx.source).is_some_and(|v| v.kind() == "true");
        if mutable {
            continue;
        }
        let Some(name) = m.name else { continue };
        out.push(finding(name, format!("@Prop `{}` is set from outside; mark it `readonly`, or `mutable: true` if the component changes it", text(name, ctx.source)), vec![TextEdit::insert(name.start_byte(), "readonly ")]));
    }
});

stencil_rule!(RequiredJsdoc, "stencil/required-jsdoc", |body, ctx, comp, out| {
    if comp.is_none() {
        return;
    }
    for m in members(body, ctx.source).into_iter().filter(|m| m.has("Prop") || m.has("Method") || m.has("Event")) {
        let before = ctx.source[..m.start()].trim_end();
        if !before.ends_with("*/") || before.rfind("/**").is_none_or(|open| before[open..].contains("\n\n")) {
            out.push(finding(m.name.unwrap_or(m.node), format!("Document the public member `{}` with a `/** … */` comment", m.name_text(ctx.source)), Vec::new()));
        }
    }
});

stencil_rule!(ReservedMemberNames, "stencil/reserved-member-names", |body, ctx, comp, out| {
    if comp.is_none() {
        return;
    }
    for m in members(body, ctx.source).into_iter().filter(|m| m.has("Prop") || m.has("Method")) {
        let name = m.name_text(ctx.source);
        if RESERVED.contains(&name) {
            out.push(finding(m.name.expect("named"), format!("`{name}` is already an HTML attribute or element member; pick another name"), Vec::new()));
        }
    }
});

stencil_rule!(StrictMutable, "stencil/strict-mutable", |body, ctx, comp, out| {
    if comp.is_none() {
        return;
    }
    let src = ctx.source;
    let body_text = text(body, src);
    for m in members(body, src).into_iter().filter(|m| m.has("Prop")) {
        let Some(v) = option_value(m.args("Prop"), "mutable", src).filter(|v| v.kind() == "true") else { continue };
        let name = m.name_text(src);
        let assigned = regex::Regex::new(&format!(r"this\.{}\s*(=[^=]|\+=|-=|\+\+|--)", regex::escape(name))).is_ok_and(|r| r.is_match(body_text));
        if !assigned {
            out.push(finding(v, format!("`{name}` is never assigned in the component, so it doesn't need `mutable: true`"), Vec::new()));
        }
    }
});

stencil_rule!(TagPrefix, "stencil/tag-prefix", |body, ctx, comp, out| {
    let Some(comp) = comp else { return };
    let prefix = ctx.options.get("prefix").and_then(|v| v.as_str()).filter(|p| !p.is_empty());
    let args = decorator_parts(comp, ctx.source).and_then(|(_, a)| a);
    if let Some(tag) = option_value(args, "tag", ctx.source) {
        let t = text(tag, ctx.source).trim_matches(['"', '\'', '`']);
        match prefix {
            Some(p) if !t.starts_with(&format!("{p}-")) => {
                out.push(finding(tag, format!("The tag `{t}` doesn't start with this project's prefix `{p}-`"), Vec::new()));
            }
            // With no prefix configured, a tag still needs one: a custom
            // element's name must contain a dash.
            None if !t.contains('-') => {
                out.push(finding(tag, format!("The tag `{t}` has no prefix; custom element names are `<prefix>-<name>`"), Vec::new()));
            }
            _ => {}
        }
    }
});

pub struct BanExportedConstEnums;

impl NativeRule for BanExportedConstEnums {
    fn id(&self) -> &'static str {
        "stencil/ban-exported-const-enums"
    }
    fn languages(&self) -> &'static [Language] {
        TS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["export_statement"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        if let Some(e) = n.child_by_field_name("declaration").filter(|d| d.kind() == "enum_declaration")
            && text(e, ctx.source).starts_with("const")
        {
            out.push(finding(e, "An exported `const enum` disappears at compile time and breaks consumers of the bundle; use a plain `enum`", vec![TextEdit::delete(e.start_byte(), e.start_byte() + "const ".len())]));
        }
    }
}

/// The functions `stencil/ban-side-effects` lets run at the top level by
/// default: test-suite builders and store factories.
const SIDE_EFFECT_DEFAULTS: &[&str] = &["describe", "test", "bind", "createStore"];

/// A node whose contents run later, or only as part of a declaration: a
/// call inside one is not a side effect of loading the module.
fn defers(n: Node<'_>) -> bool {
    is_function(n.kind()) || n.kind() == "arrow_function" || (n.kind() == "export_statement" && !has_token(n, "default"))
}

pub struct BanSideEffects;

impl NativeRule for BanSideEffects {
    fn id(&self) -> &'static str {
        "stencil/ban-side-effects"
    }
    fn languages(&self) -> &'static [Language] {
        TS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["call_expression"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let call = node.get_inner_node();
        // Test files may run code at load time.
        if regex::Regex::new(r"\b(spec|e2e|test)\.").is_ok_and(|r| r.is_match(ctx.file.path())) {
            return;
        }
        let Some(callee) = call.child_by_field_name("function") else { return };
        // A tagged template and a dynamic `import()` are call expressions in
        // the tree, not calls in the source rule's.
        if callee.kind() == "import" || call.child_by_field_name("arguments").is_some_and(|a| a.kind() == "template_string") {
            return;
        }
        if callee.kind() == "identifier" {
            let name = text(callee, ctx.source);
            let skipped = match ctx.options.get("allow").and_then(|v| v.as_array()) {
                Some(list) => list.iter().any(|x| x.as_str() == Some(name)),
                None => SIDE_EFFECT_DEFAULTS.contains(&name),
            };
            if skipped {
                return;
            }
        }
        let mut p = call.parent();
        while let Some(x) = p {
            if defers(x) {
                return;
            }
            p = x.parent();
        }
        out.push(finding(call, "This call runs when the module loads; move it into a function or a lifecycle method", Vec::new()));
    }
}

/// The slot names a component's JSDoc documents with `@slot`; `<default>`
/// for the unnamed slot. `@slot name - text` names `name`; `@slot - text`
/// and a bare `@slot` document the default slot.
fn documented_slots(jsdoc: &str) -> BTreeSet<String> {
    let Ok(tag) = regex::Regex::new(r"@(\w+)[ \t]*(.*)") else { return BTreeSet::new() };
    tag.captures_iter(jsdoc)
        .filter(|c| &c[1] == "slot")
        .map(|c| {
            let comment = c[2].trim();
            if comment.is_empty() || comment == "-" || comment.starts_with("- ") {
                "<default>".to_owned()
            } else {
                comment.find(" - ").map_or(comment, |i| comment[..i].trim()).to_owned()
            }
        })
        .collect()
}

/// The first `/** … */` comment directly before a class (before its
/// decorators and `export`), without its `/*` and `*/`.
fn class_jsdoc<'s>(class: Node<'_>, src: &'s str) -> Option<&'s str> {
    let start = class.parent().filter(|p| p.kind() == "export_statement").unwrap_or(class);
    let mut comments = Vec::new();
    let mut prev = start.prev_sibling();
    while let Some(c) = prev.filter(|c| c.kind() == "comment") {
        comments.push(c);
        prev = c.prev_sibling();
    }
    comments.into_iter().rev().map(|c| text(c, src)).find(|t| t.len() >= 5 && t.starts_with("/**") && t.ends_with("*/")).map(|t| &t[2..t.len() - 2])
}

/// The slot names a component renders: each `<slot>`'s literal `name`, or
/// `<default>` for one with none. A computed name is left out.
fn rendered_slots(body: Node<'_>, src: &str) -> BTreeSet<String> {
    let mut all = Vec::new();
    descendants(body, &|_| false, &mut all);
    let mut out = BTreeSet::new();
    for el in all.into_iter().filter(|n| matches!(n.kind(), "jsx_opening_element" | "jsx_self_closing_element")) {
        if el.child_by_field_name("name").is_none_or(|n| text(n, src) != "slot") {
            continue;
        }
        let name_attr = named_children(el).into_iter().filter(|a| a.kind() == "jsx_attribute").find(|a| a.named_child(0).is_some_and(|n| text(n, src) == "name"));
        match name_attr.map(|a| a.named_child(1)) {
            None => {
                out.insert("<default>".to_owned());
            }
            Some(Some(v)) if v.kind() == "string" => {
                out.insert(string_value(v, src));
            }
            _ => {}
        }
    }
    out
}

pub struct EnforceSlotJsdoc;

impl NativeRule for EnforceSlotJsdoc {
    fn id(&self) -> &'static str {
        "stencil/enforce-slot-jsdoc"
    }
    fn languages(&self) -> &'static [Language] {
        &[Language::Tsx]
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["class_body"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let body = node.get_inner_node();
        let src = ctx.source;
        let Some(class) = body.parent().filter(|c| matches!(c.kind(), "class_declaration" | "abstract_class_declaration")) else { return };
        if is_component(body, src).is_none() {
            return;
        }
        let at = class.child_by_field_name("name").unwrap_or(class);
        let documented = class_jsdoc(class, src).map(documented_slots).unwrap_or_default();
        let rendered = rendered_slots(body, src);
        let label = |s: &str| if s == "<default>" { "The default slot".to_owned() } else { format!("The slot '{s}'") };
        for s in rendered.difference(&documented) {
            out.push(finding(at, format!("{} is rendered but has no `@slot` tag in the component's JSDoc", label(s)), Vec::new()));
        }
        for s in documented.difference(&rendered) {
            out.push(finding(at, format!("{} has a `@slot` tag but the component never renders it", label(s)), Vec::new()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ban_side_effects() {
        let src = "import { setMode } from './x';\nsetMode('md');\nconsole.log(1);\ndescribe('x', () => {});\nconst a = compute();\nexport const b = compute();\nexport default compute();\nconst t = tag`x`;\nconst m = import('./m');\nfunction f() { g(); }\nconst h = () => g();\nclass C { x = g(); m() { g(); } }\n";
        let got: Vec<&str> = crate::run(&BanSideEffects, Language::Typescript, src).into_iter().map(|f| &src[f.start..f.end]).collect();
        assert_eq!(got, ["setMode('md')", "console.log(1)", "compute()", "compute()"]);
    }

    #[test]
    fn enforce_slot_jsdoc() {
        let src = "/**\n * @slot - The body.\n * @slot start - Before the body.\n * @slot gone - Never rendered.\n */\n@Component({ tag: 'my-item' })\nexport class Item {\n  render() {\n    return <Host><slot name=\"start\" /><slot /><slot name=\"end\"></slot><slot name={this.n} /></Host>;\n  }\n}\nclass Plain { render() { return <slot name=\"x\" />; } }\n";
        let got: Vec<String> = crate::run(&EnforceSlotJsdoc, Language::Tsx, src).into_iter().map(|f| f.message).collect();
        assert_eq!(got.len(), 2, "{got:?}");
        assert!(got[0].starts_with("The slot 'end' is rendered"), "{got:?}");
        assert!(got[1].starts_with("The slot 'gone' has a `@slot` tag"), "{got:?}");
        let documented = documented_slots("* @slot\n * @slot -\n * @slot a - x - y\n * @slot b text ");
        assert_eq!(documented.into_iter().collect::<Vec<_>>(), ["<default>", "a", "b text"]);
    }

    const SRC: &str = "@Component({ tag: 'stencil-card' })\nexport class Card {\n  /** The heading. */\n  @Prop() heading: string;\n  @Prop() private open = true;\n  @Prop({ mutable: true }) count = 0;\n  @Prop() readonly title: string;\n  @State() expanded = false;\n  @Watch('heading') onHeading() {}\n  @Watch('missing') onMissing() {}\n  @Listen('click') onClick() {}\n  /** Opens it. */\n  @Method() private async show() {}\n  helper() {}\n  private done() {}\n  render() { return null; }\n  hostData() {}\n  cache = new Map();\n}\nclass Plain { @Prop() x: string; }\nexport const enum Size { S, M }\n";

    fn found(rule: &dyn NativeRule) -> Vec<String> {
        crate::run(rule, Language::Tsx, SRC).into_iter().map(|f| SRC[f.start..f.end].to_owned()).collect()
    }

    #[test]
    fn stencil_rules() {
        assert_eq!(found(&BanDefaultTrue), ["true"]);
        assert_eq!(found(&BanPrefix), ["'stencil-card'"]);
        assert_eq!(found(&DecoratorsContext), ["@Prop()"]);
        assert_eq!(found(&HostDataDeprecated), ["hostData"]);
        assert_eq!(found(&MethodsMustBePublic), ["show"]);
        assert_eq!(found(&PropsMustBePublic), ["open"]);
        assert_eq!(found(&NoUnusedWatch), ["@Watch('missing')"]);
        assert_eq!(found(&OwnMethodsMustBePrivate), ["helper"]);
        assert_eq!(found(&OwnPropsMustBePrivate), ["cache"]);
        assert_eq!(found(&PreferVdomListener), ["@Listen('click')"]);
        assert_eq!(found(&PropReadonly), ["heading", "open"]);
        assert_eq!(found(&RequiredJsdoc), ["open", "count", "title"]);
        assert_eq!(found(&ReservedMemberNames), ["title"]);
        assert_eq!(found(&StrictMutable), ["true"]);
        assert_eq!(found(&BanExportedConstEnums), ["const enum Size { S, M }"]);
    }
}
