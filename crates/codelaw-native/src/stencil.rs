//! Native rules behind `stencil/recommended`. Each one needs the same
//! context a pattern can't carry: whether a class is a `@Component`, and
//! which Stencil decorators each member has — a field's decorators are its
//! children, a method's are the siblings before it.

use crate::util::{children, named_children, text};
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
    let Some(prefix) = ctx.options.get("prefix").and_then(|v| v.as_str()).filter(|p| !p.is_empty()) else { return };
    let args = decorator_parts(comp, ctx.source).and_then(|(_, a)| a);
    if let Some(tag) = option_value(args, "tag", ctx.source) {
        let t = text(tag, ctx.source).trim_matches(['"', '\'', '`']);
        if !t.starts_with(&format!("{prefix}-")) {
            out.push(finding(tag, format!("The tag `{t}` doesn't start with this project's prefix `{prefix}-`"), Vec::new()));
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

#[cfg(test)]
mod tests {
    use super::*;

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
