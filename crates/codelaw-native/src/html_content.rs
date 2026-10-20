//! `html/*` rules about the document's structure: content models, required
//! and permitted attributes, element names, and how elements are closed.
//! They read the pack's `html/elements` table (html-validate's element
//! metadata).

use crate::html_dom::{finding, finding_with, in_category, permitted, remove_attr, Closed, ClosedBy, Dom, Meta};
use crate::html_rule;
use codelaw_core::TextEdit;
use regex::Regex;
use serde_json::{Map, Value};

fn lt(tag: &str) -> String {
    format!("<{tag}>")
}

/// `a`, `b` or `c`.
fn join_or(items: &[String]) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].clone(),
        n => format!("{} or {}", items[..n - 1].join(", "), items[n - 1]),
    }
}

fn describe(rule: &str) -> String {
    let raw = rule.trim_end_matches(['?', '*']);
    match raw.strip_prefix('@') {
        Some(cat) => format!("{cat} content"),
        None => raw.split(',').map(|t| lt(t.trim())).collect::<Vec<_>>().join(" or "),
    }
}

/// Elements whose content is checked: known, not inside `<svg>`/`<math>`,
/// with an element as parent.
fn checked_parent(dom: &Dom, i: usize) -> Option<usize> {
    let e = &dom.elems[i];
    if e.in_foreign {
        return None;
    }
    e.parent
}

fn is_native_template(dom: &Dom, meta: Meta<'_>, i: usize) -> bool {
    dom.elems[i].tag == "template" && meta.static_flag("template", "templateRoot")
}

/// The children of a transparent element that count as content of its parent.
fn transparent_children(dom: &Dom, meta: Meta<'_>, i: usize) -> Vec<usize> {
    match meta.get(&dom.elems[i].tag, "transparent") {
        Some(Value::Bool(true)) => dom.child_elems(i).collect(),
        Some(Value::Array(cats)) => dom
            .child_elems(i)
            .filter(|&c| cats.iter().filter_map(|x| x.as_str()).any(|cat| in_category(dom, meta, c, cat, false)))
            .collect(),
        _ => Vec::new(),
    }
}

fn content_violation(dom: &Dom, meta: Meta<'_>, child: usize, parent: usize, rules: &Value, out: &mut Vec<crate::NativeFinding>) -> bool {
    if !permitted(dom, meta, child, rules) {
        out.push(dom.at_elem(child, format!("{} is not allowed inside {}", lt(&dom.elems[child].tag), lt(&dom.elems[parent].tag))));
        return true;
    }
    let mut hit = false;
    for c in transparent_children(dom, meta, child) {
        hit |= content_violation(dom, meta, c, parent, rules, out);
    }
    hit
}

html_rule!(ElementPermittedContent, "html/element-permitted-content", |dom, meta, ctx, out| {
    for i in dom.all() {
        let Some(parent) = checked_parent(dom, i) else { continue };
        // Obsolete elements have no content model; `html/deprecated` reports them.
        if meta.get(&dom.elems[i].tag, "deprecated").is_some() {
            continue;
        }
        if let Some(rules) = meta.get(&dom.elems[parent].tag, "permittedContent")
            && content_violation(dom, meta, i, parent, rules, out)
        {
            continue;
        }
        for anc in dom.self_and_ancestors(parent) {
            if is_native_template(dom, meta, anc) {
                break;
            }
            let Some(rules) = meta.get(&dom.elems[anc].tag, "permittedDescendants") else { continue };
            if permitted(dom, meta, i, rules) {
                continue;
            }
            out.push(dom.at_elem(i, format!("{} is not allowed anywhere inside {}", lt(&dom.elems[i].tag), lt(&dom.elems[anc].tag))));
            break;
        }
    }
});

html_rule!(ElementPermittedOccurrences, "html/element-permitted-occurrences", |dom, meta, ctx, out| {
    for i in dom.all() {
        if dom.elems[i].in_foreign {
            continue;
        }
        let rules = meta.list(&dom.elems[i].tag, "permittedContent");
        let kids: Vec<usize> = dom.child_elems(i).collect();
        for rule in rules.iter().filter(|r| r.ends_with('?')) {
            let matching: Vec<usize> = kids.iter().copied().filter(|&c| in_category(dom, meta, c, rule, true)).collect();
            for &extra in matching.iter().skip(1) {
                out.push(dom.at_elem(extra, format!("{} may appear only once inside {}", describe(rule), lt(&dom.elems[i].tag))));
            }
        }
    }
});

html_rule!(ElementPermittedOrder, "html/element-permitted-order", |dom, meta, ctx, out| {
    for i in dom.all() {
        if dom.elems[i].in_foreign {
            continue;
        }
        let rules = meta.list(&dom.elems[i].tag, "permittedOrder");
        if rules.is_empty() {
            continue;
        }
        let mut pos = 0usize;
        let mut prev: Option<usize> = None;
        for c in dom.child_elems(i).collect::<Vec<_>>() {
            let old = pos;
            while pos < rules.len() && !in_category(dom, meta, c, rules[pos], true) {
                pos += 1;
            }
            if pos >= rules.len() {
                if rules.iter().any(|r| in_category(dom, meta, c, r, true)) {
                    if let Some(p) = prev {
                        out.push(dom.at_elem(c, format!("{} must come before {} here", lt(&dom.elems[c].tag), lt(&dom.elems[p].tag))));
                    }
                    break;
                }
                pos = old;
            }
            prev = Some(c);
        }
    }
});

html_rule!(ElementPermittedParent, "html/element-permitted-parent", |dom, meta, ctx, out| {
    for i in dom.all() {
        let Some(parent) = checked_parent(dom, i) else { continue };
        if dom.elems[parent].tag == dom.elems[i].tag {
            continue;
        }
        let Some(rules) = meta.get(&dom.elems[i].tag, "permittedParent") else { continue };
        if permitted(dom, meta, parent, rules) {
            continue;
        }
        let allowed: Vec<String> = rules.as_array().into_iter().flatten().filter_map(|r| r.as_str()).map(describe).collect();
        out.push(dom.at_elem(i, format!("{} must be a child of {}, not of {}", lt(&dom.elems[i].tag), join_or(&allowed), lt(&dom.elems[parent].tag))));
    }
});

html_rule!(ElementRequiredAncestor, "html/element-required-ancestor", |dom, meta, ctx, out| {
    for i in dom.all() {
        if dom.elems[i].in_foreign {
            continue;
        }
        let rules = meta.list(&dom.elems[i].tag, "requiredAncestors");
        if rules.is_empty() || rules.iter().any(|r| dom.closest(i, r).is_some()) {
            continue;
        }
        let wanted: Vec<String> =
            rules.iter().map(|r| if r.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') { lt(r) } else { format!("`{r}`") }).collect();
        out.push(dom.at_elem(i, format!("{} must be inside {}", lt(&dom.elems[i].tag), join_or(&wanted))));
    }
});

/// Whether an attribute's `required` entry applies to this element.
fn required(dom: &Dom, i: usize, attr: &Map<String, Value>) -> bool {
    match attr.get("required") {
        Some(Value::Bool(b)) => *b,
        Some(c @ Value::Object(_)) => crate::html_dom::eval(dom, i, c),
        _ => false,
    }
}

html_rule!(ElementRequiredAttributes, "html/element-required-attributes", |dom, meta, ctx, out| {
    for i in dom.all() {
        let e = &dom.elems[i];
        if e.in_foreign || e.implied {
            continue;
        }
        let Some(Value::Object(attrs)) = meta.get(&e.tag, "attributes") else { continue };
        for (name, a) in attrs {
            let Some(a) = a.as_object() else { continue };
            if required(dom, i, a) && !e.has(name) {
                out.push(dom.at_elem(i, format!("{} needs a `{name}` attribute", lt(&e.tag))));
            }
        }
    }
});

html_rule!(ElementRequiredContent, "html/element-required-content", |dom, meta, ctx, out| {
    for i in dom.all() {
        if dom.elems[i].in_foreign {
            continue;
        }
        for rule in meta.list(&dom.elems[i].tag, "requiredContent") {
            if dom.child_elems(i).any(|c| in_category(dom, meta, c, rule, false)) {
                continue;
            }
            out.push(dom.at_elem(i, format!("{} must contain {}", lt(&dom.elems[i].tag), describe(rule))));
        }
    }
});

fn regex_matches(pattern: &str, value: &str) -> bool {
    thread_local! {
        static CACHE: std::cell::RefCell<std::collections::HashMap<String, Option<Regex>>> = std::cell::RefCell::new(std::collections::HashMap::new());
    }
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        let re = c.entry(pattern.to_owned()).or_insert_with(|| Regex::new(pattern).ok());
        re.as_ref().is_some_and(|re| re.is_match(value))
    })
}

fn enum_matches(value: &str, entries: &[Value]) -> bool {
    let lower = value.to_lowercase();
    entries.iter().any(|e| match e {
        Value::String(s) => lower == *s,
        Value::Object(o) => o.get("regex").and_then(|r| r.as_str()).is_some_and(|r| regex_matches(r, value)),
        _ => false,
    })
}

/// Whether an attribute value is allowed by its metadata.
fn value_allowed(name: &str, value: Option<&str>, rule: &Map<String, Value>) -> bool {
    let empty = value.is_none_or(str::is_empty);
    if rule.get("boolean") == Some(&Value::Bool(true)) {
        return empty || value.is_some_and(|v| v.eq_ignore_ascii_case(name));
    }
    if rule.get("omit") == Some(&Value::Bool(true)) && empty {
        return true;
    }
    let Some(Value::Array(entries)) = rule.get("enum") else { return true };
    let Some(value) = value else { return false };
    if rule.get("list") == Some(&Value::Bool(true)) {
        return value.split_ascii_whitespace().all(|t| enum_matches(t, entries));
    }
    enum_matches(value, entries)
}

html_rule!(AttributeAllowedValues, "html/attribute-allowed-values", |dom, meta, ctx, out| {
    for i in dom.all() {
        let e = &dom.elems[i];
        if e.in_foreign || meta.elem(&e.tag).is_none() {
            continue;
        }
        for a in &e.attrs {
            if a.is_dynamic() {
                continue;
            }
            let Some(rule) = meta.attr(&e.tag, &a.name) else { continue };
            if value_allowed(&a.name, a.value.as_deref(), rule) {
                continue;
            }
            let msg = match &a.value {
                Some(v) if rule.get("boolean") == Some(&Value::Bool(true)) => {
                    format!("`{}` is a boolean attribute: write it without a value, not `\"{v}\"`", a.name)
                }
                Some(v) => format!("`\"{v}\"` is not a valid value for `{}` on {}", a.name, lt(&e.tag)),
                None => format!("`{}` on {} needs a value", a.name, lt(&e.tag)),
            };
            let (s, t) = if a.value.is_some() { (a.value_start, a.value_end.max(a.value_start)) } else { (a.name_start, a.name_end) };
            let (s, t) = if s == t { (a.start, a.end) } else { (s, t) };
            out.push(finding(s, t, msg));
        }
    }
});

fn strs(v: &Value) -> Vec<&str> {
    v.as_array().map(|a| a.iter().filter_map(|x| x.as_str()).collect()).unwrap_or_default()
}

fn quoted(items: &[&str]) -> String {
    join_or(&items.iter().map(|s| format!("`{s}`")).collect::<Vec<_>>())
}

/// Why an attribute can't be used here, from its `allowed` entry.
fn misuse(dom: &Dom, i: usize, value: Option<&str>, allowed: &Value) -> Option<String> {
    let e = &dom.elems[i];
    let o = allowed.as_object()?;
    if let Some(names) = o.get("if-attribute-present") {
        let names = strs(names);
        return (!names.iter().any(|n| e.has(n))).then(|| format!("it needs {} too", quoted(&names)));
    }
    if let Some(names) = o.get("if-attribute-absent") {
        let present: Vec<&str> = strs(names).into_iter().filter(|n| e.has(n)).collect();
        return (!present.is_empty()).then(|| format!("it can't be combined with {}", quoted(&present)));
    }
    if let Some(v) = o.get("if-attribute-value") {
        let key = v.get("key").and_then(|k| k.as_str()).unwrap_or("");
        let values = v.get("values").map(strs).unwrap_or_default();
        let actual = e.attr(key).map(|a| a.value_or_empty().to_ascii_lowercase()).or_else(|| v.get("default").and_then(|d| d.as_str()).map(str::to_owned));
        if e.attr(key).is_some_and(|a| a.is_dynamic()) {
            return None;
        }
        return (!actual.is_some_and(|a| values.contains(&a.as_str()))).then(|| format!("`{key}` must be {}", quoted(&values)));
    }
    if let Some(tags) = o.get("if-parent-present") {
        let tags = strs(tags);
        return (!tags.iter().any(|t| dom.closest(i, t).is_some()))
            .then(|| format!("it only works inside {}", join_or(&tags.iter().map(|t| lt(t)).collect::<Vec<_>>())));
    }
    if let Some(list) = o.get("rel-disallowed") {
        if let Some(req) = o.get("requires").and_then(|r| r.as_str())
            && !e.has(req)
        {
            return Some(format!("it needs `{req}` too"));
        }
        let list = strs(list);
        let prefix = o.get("disallowed-prefix").and_then(|p| p.as_str());
        let value = value?.to_ascii_lowercase();
        let bad = value.split_ascii_whitespace().find(|t| list.contains(t) || prefix.is_some_and(|p| t.starts_with(p)))?;
        return Some(format!("!{} doesn't take `rel=\"{bad}\"`", lt(&e.tag)));
    }
    if let Some(v) = o.get("unless-attr-value") {
        let key = v.get("name").and_then(|k| k.as_str()).unwrap_or("");
        let values = v.get("values").map(strs).unwrap_or_default();
        let actual = e.value_lower(key)?;
        return values.contains(&actual.as_str()).then(|| format!("it has no effect when `{key}` is `\"{actual}\"`"));
    }
    if o.contains_key("area-shape-needs-coords") {
        let shape = e.value_lower("shape").unwrap_or_else(|| "rect".into());
        let needs = matches!(shape.as_str(), "circ" | "circle" | "poly" | "polygon" | "rect" | "rectangle");
        return (needs && !e.has("coords")).then(|| "it needs `coords` too".to_owned());
    }
    if o.contains_key("link-image-preload") {
        let rel_ok = e.value_lower("rel").is_some_and(|r| r.split_ascii_whitespace().any(|t| t == "preload"));
        if !rel_ok {
            return Some("`rel` must be `preload`".into());
        }
        return (e.value_lower("as").as_deref() != Some("image")).then(|| "`as` must be `image`".to_owned());
    }
    None
}

html_rule!(AttributeMisuse, "html/attribute-misuse", |dom, meta, ctx, out| {
    for i in dom.all() {
        let e = &dom.elems[i];
        if e.in_foreign {
            continue;
        }
        for a in &e.attrs {
            let Some(allowed) = meta.attr(&e.tag, &a.name).and_then(|m| m.get("allowed")) else { continue };
            if let Some(why) = misuse(dom, i, a.value.as_deref(), allowed) {
                let msg = match why.strip_prefix('!') {
                    Some(full) => full.to_owned(),
                    None => format!("`{}` has no effect on {} here: {why}", a.name, lt(&e.tag)),
                };
                out.push(finding(a.name_start, a.name_end, msg));
            }
        }
    }
});

html_rule!(Deprecated, "html/deprecated", |dom, meta, ctx, out| {
    for i in dom.all() {
        let e = &dom.elems[i];
        if e.in_foreign || e.implied {
            continue;
        }
        let Some(dep) = meta.get(&e.tag, "deprecated") else { continue };
        let hint = dep.get("message").and_then(|m| m.as_str()).map(|m| format!("; {m}")).unwrap_or_default();
        if matches!(dep, Value::Bool(false)) {
            continue;
        }
        out.push(dom.at_elem(i, format!("{} is obsolete{hint}", lt(&e.tag))));
    }
});

html_rule!(NoDeprecatedAttr, "html/no-deprecated-attr", |dom, meta, ctx, out| {
    for i in dom.all() {
        let e = &dom.elems[i];
        if e.in_foreign || meta.elem(&e.tag).is_none() {
            continue;
        }
        for a in &e.attrs {
            if meta.attr(&e.tag, &a.name).and_then(|m| m.get("deprecated")).is_some_and(|d| d != &Value::Bool(false)) {
                out.push(finding(a.name_start, a.name_end, format!("`{}` on {} is obsolete", a.name, lt(&e.tag))));
            }
        }
    }
});

html_rule!(ElementName, "html/element-name", |dom, meta, ctx, out| {
    if !meta.known() {
        return;
    }
    let allow: Vec<String> =
        ctx.options.get("allow").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_ascii_lowercase)).collect()).unwrap_or_default();
    let custom = Regex::new(r"^[a-z][a-z0-9\-._]*-[a-z0-9\-._]*$").expect("valid regex");
    for i in dom.all() {
        let e = &dom.elems[i];
        if e.in_foreign || e.implied || meta.elem(&e.tag).is_some() || e.tag.contains(':') || allow.contains(&e.tag) || custom.is_match(&e.tag) {
            continue;
        }
        out.push(dom.at_elem(i, format!("{} is not an HTML element, and a custom element's name needs a hyphen", lt(&e.tag))));
    }
});

html_rule!(VoidContent, "html/void-content", |dom, meta, ctx, out| {
    for t in &dom.void_end_tags {
        out.push(finding_with(
            t.start,
            t.end,
            format!("{} is a void element: it takes no end tag", lt(&t.name)),
            vec![TextEdit::new(t.start, t.end, "")],
        ));
    }
});

html_rule!(CloseOrder, "html/close-order", |dom, meta, ctx, out| {
    if !meta.known() {
        return;
    }
    for t in &dom.stray_end_tags {
        out.push(finding(t.start, t.end, format!("`</{}>` closes no open element", t.name)));
    }
    for i in dom.all() {
        let e = &dom.elems[i];
        if e.closed == Closed::Unclosed && !e.in_foreign {
            out.push(dom.at_elem(i, format!("{} is never closed", lt(&e.tag))));
        }
    }
});

html_rule!(CloseAttr, "html/close-attr", |dom, meta, ctx, out| {
    let src = ctx.source;
    for e in &dom.elems {
        if let Some((s, t)) = e.end_tag_junk {
            let before = src[..s].trim_end_matches([' ', '\t', '\n', '\r']).len();
            out.push(finding_with(s, t, "An end tag can't have attributes", vec![TextEdit::new(before, t, "")]));
        }
    }
});

html_rule!(NoImplicitClose, "html/no-implicit-close", |dom, meta, ctx, out| {
    for i in dom.all() {
        let e = &dom.elems[i];
        if e.implied || e.in_foreign {
            continue;
        }
        let Closed::Implicit(by) = &e.closed else { continue };
        let why = match by {
            ClosedBy::Sibling => "the next one".to_owned(),
            ClosedBy::Adjacent(t) => format!("the {} after it", lt(t)),
            ClosedBy::Parent(t) => format!("its parent's `</{t}>`"),
            ClosedBy::Ancestor(t) => format!("an ancestor's `</{t}>`"),
            ClosedBy::EndOfDocument => "the end of the document".to_owned(),
        };
        let (s, t) = dom.tag_range(i);
        out.push(finding_with(
            s,
            t,
            format!("{} has no end tag and is closed by {why}; write `</{}>`", lt(&e.tag), e.tag),
            vec![TextEdit::new(e.end, e.end, format!("</{}>", e.tag))],
        ));
    }
});

html_rule!(ScriptElement, "html/script-element", |dom, meta, ctx, out| {
    let src = ctx.source;
    for &(s, t) in &dom.self_closed_scripts {
        let slash = src[s..t].rfind("/>").map(|k| s + k).unwrap_or(t);
        let before = src[..slash].trim_end_matches([' ', '\t', '\n', '\r']).len();
        out.push(finding_with(
            s,
            t,
            "A <script> needs its `</script>` end tag, even with `src`; `/>` doesn't close it",
            vec![TextEdit::new(before, t, "></script>")],
        ));
    }
});

html_rule!(InputAttributes, "html/input-attributes", |dom, meta, ctx, out| {
    let Some(Value::Object(table)) = ctx.data.get("html/input-attributes").and_then(|v| v.get("attributes").cloned()) else { return };
    for i in dom.by_tag("input") {
        let e = &dom.elems[i];
        let Some(ty) = e.attr("type").filter(|a| !a.is_dynamic()).and_then(|a| a.value.clone()).filter(|v| !v.is_empty()) else { continue };
        let ty_lower = ty.to_ascii_lowercase();
        for a in &e.attrs {
            let Some(types) = table.get(&a.name) else { continue };
            if strs(types).contains(&ty_lower.as_str()) {
                continue;
            }
            out.push(finding_with(
                a.name_start,
                a.name_end,
                format!("`{}` has no effect on <input type=\"{ty}\">", a.name),
                vec![remove_attr(ctx.source, a)],
            ));
        }
    }
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html_dom::tests::run;

    #[test]
    fn li_outside_a_list() {
        let f = run(&ElementPermittedParent, "<div><li>a</li></div>");
        assert_eq!(f.len(), 1, "{f:?}");
    }

    #[test]
    fn div_in_span() {
        let f = run(&ElementPermittedContent, "<span><div>a</div></span>");
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(run(&ElementPermittedContent, "<div><span>a</span></div>").is_empty());
    }

    #[test]
    fn interactive_in_a_link() {
        let f = run(&ElementPermittedContent, "<a href=x><button type=button>b</button></a>");
        assert_eq!(f.len(), 1, "{f:?}");
    }

    #[test]
    fn values() {
        assert_eq!(run(&AttributeAllowedValues, r#"<input type="txt">"#).len(), 1);
        assert!(run(&AttributeAllowedValues, r#"<input type="text" disabled>"#).is_empty());
        assert_eq!(run(&AttributeAllowedValues, r#"<input disabled="no">"#).len(), 1);
    }

    #[test]
    fn required_content_of_html() {
        assert!(run(&ElementRequiredContent, "<html><head><title>t</title></head><body></body></html>").is_empty());
        assert_eq!(run(&ElementRequiredContent, "<html><body></body></html>").len(), 1);
    }

    #[test]
    fn close_order() {
        let f = run(&CloseOrder, "<div><span>a</div></p>");
        assert_eq!(f.len(), 2, "{f:?}");
        assert!(run(&CloseOrder, "<ul><li>a<li>b</ul>").is_empty());
    }
}
