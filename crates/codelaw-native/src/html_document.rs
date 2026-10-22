//! `html/*` rules for complete documents (html-validate's `document`
//! preset): labels, headings, the doctype, id references and SRI.

use crate::html_dom::{classify_text, finding, has_alt_text, in_accessibility_tree, tokens_with_offsets, Dom, Meta, TextClass};
use crate::html_rule;
use codelaw_core::TextEdit;
use regex::Regex;

/// Whether an element has an accessible name: text, an image's `alt`,
/// `aria-label`, or `aria-labelledby` naming an element that has one.
/// `reference` is the element being named through `aria-labelledby`: it
/// counts even when hidden.
fn accessible_name(dom: &Dom, meta: Meta<'_>, i: usize, reference: Option<usize>) -> bool {
    if reference != Some(i) && !in_accessibility_tree(dom, meta, i) {
        return false;
    }
    if classify_text(dom, i, true, reference == Some(i)) != TextClass::Empty {
        return true;
    }
    let e = &dom.elems[i];
    let img_name = if e.is("img") {
        has_alt_text(e)
    } else if e.is("svg") {
        !dom.text_content(i).trim().is_empty()
    } else {
        dom.descendants(i).into_iter().filter(|&d| matches!(dom.elems[d].tag.as_str(), "img" | "svg")).any(|d| accessible_name(dom, meta, d, reference))
    };
    if img_name || e.attr("aria-label").is_some_and(|a| !a.value_or_empty().trim().is_empty()) {
        return true;
    }
    if reference.is_some() {
        return false;
    }
    let Some(ids) = e.attr("aria-labelledby") else { return false };
    if ids.is_dynamic() {
        return true;
    }
    ids.value_or_empty().split_ascii_whitespace().any(|id| dom.by_id(id).any(|r| accessible_name(dom, meta, r, Some(r))))
}

html_rule!(InputMissingLabel, "html/input-missing-label", |dom, meta, ctx, out| {
    for i in dom.all() {
        let e = &dom.elems[i];
        if !matches!(e.tag.as_str(), "input" | "textarea" | "select") || !in_accessibility_tree(dom, meta, i) {
            continue;
        }
        if e.is("input") && matches!(e.value_lower("type").as_deref(), Some("hidden" | "submit" | "reset" | "button")) {
            continue;
        }
        if accessible_name(dom, meta, i, None) {
            continue;
        }
        let id = e.attr("id").and_then(|a| a.value.clone()).filter(|v| !v.is_empty());
        let mut labels: Vec<usize> = match &id {
            Some(id) => dom.by_tag("label").filter(|&l| dom.elems[l].attr("for").is_some_and(|f| f.value_or_empty() == id)).collect(),
            None => Vec::new(),
        };
        if labels.is_empty() {
            labels = dom.ancestors(i).filter(|&a| dom.elems[a].is("label")).take(1).collect();
        }
        if let Some(&first) = labels.first() {
            let visible: Vec<usize> = labels.iter().copied().filter(|&l| in_accessibility_tree(dom, meta, l)).collect();
            if visible.is_empty() {
                out.push(dom.at_elem(first, format!("The <label> of this <{}> is hidden", e.tag)));
            } else if labels.iter().all(|&l| !accessible_name(dom, meta, l, None)) {
                out.push(dom.at_elem(first, format!("The <label> of this <{}> has no text", e.tag)));
            }
            continue;
        }
        let msg = if e.has("aria-label") {
            format!("<{}> has an empty `aria-label`", e.tag)
        } else if e.has("aria-labelledby") {
            format!("The element `aria-labelledby` points at gives this <{}> no text", e.tag)
        } else {
            format!("<{}> has no <label>", e.tag)
        };
        out.push(dom.at_elem(i, msg));
    }
});

fn heading_level(tag: &str) -> Option<u32> {
    let d = tag.strip_prefix('h')?;
    (d.len() == 1).then(|| d.parse().ok()).flatten().filter(|n| (1..=6).contains(n))
}

struct Root {
    current: u32,
    h1: u32,
}

fn rank_option(ctx: &crate::NativeContext<'_>, key: &str, default: u32) -> u32 {
    match ctx.options.get(key) {
        Some(serde_json::Value::Bool(false)) => 6,
        Some(serde_json::Value::String(s)) if s == "any" => 6,
        Some(serde_json::Value::String(s)) => heading_level(s).unwrap_or(1),
        _ => default,
    }
}

html_rule!(HeadingLevel, "html/heading-level", |dom, meta, ctx, out| {
    let allow_multiple_h1 = ctx.options.get("allowMultipleH1").and_then(|v| v.as_bool()).unwrap_or(false);
    let min_initial = rank_option(ctx, "minInitialRank", 1);
    let roots: Vec<String> = ctx
        .options
        .get("sectioningRoots")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect())
        .unwrap_or_else(|| vec!["dialog".into(), "[role=\"dialog\"]".into(), "[role=\"alertdialog\"]".into()]);
    let mut stack = vec![Root { current: 0, h1: 0 }];
    let top: Vec<usize> = dom.roots.iter().filter_map(|c| if let crate::html_dom::Child::Elem(i) = c { Some(*i) } else { None }).collect();
    for r in top {
        visit(dom, r, &roots, &mut stack, allow_multiple_h1, min_initial, out);
    }
});

fn visit(dom: &Dom, i: usize, roots: &[String], stack: &mut Vec<Root>, allow_multiple_h1: bool, min_initial: u32, out: &mut Vec<crate::NativeFinding>) {
    let e = &dom.elems[i];
    if let Some(level) = heading_level(&e.tag).filter(|_| !e.in_foreign) {
        let depth = stack.len();
        let prev_current = if depth >= 2 { stack[depth - 2].current } else { 0 };
        let root = stack.last_mut().expect("root");
        let (s, t) = dom.tag_range(i);
        let mut done = false;
        if !allow_multiple_h1 && level == 1 {
            if root.h1 >= 1 {
                out.push(finding(s, t, "Only one <h1> per page"));
                done = true;
            } else {
                root.h1 += 1;
            }
        }
        if !done {
            if level <= root.current {
                root.current = level;
            } else {
                let expected = root.current + 1;
                if level != expected && !(depth == 1 && expected == 1 && level <= min_initial) {
                    if root.current > 0 {
                        out.push(finding(s, t, format!("Headings go down one level at a time: expected <h{expected}>, found <h{level}>")));
                    } else if depth == 1 {
                        let msg = if min_initial > 1 {
                            format!("The first heading must be <h{min_initial}> or higher, found <h{level}>")
                        } else {
                            format!("The first heading must be <h{expected}>, found <h{level}>")
                        };
                        out.push(finding(s, t, msg));
                    } else if level > prev_current + 1 {
                        out.push(finding(s, t, format!("The first heading in this dialog must be <h{}> at most, found <h{level}>", prev_current + 1)));
                    }
                }
                root.current = level;
            }
        }
    }
    let is_root = roots.iter().any(|r| dom.matches(i, r));
    if is_root {
        stack.push(Root { current: 0, h1: 0 });
    }
    for c in dom.child_elems(i).collect::<Vec<_>>() {
        visit(dom, c, roots, stack, allow_multiple_h1, min_initial, out);
    }
    if is_root {
        stack.pop();
    }
}

html_rule!(MissingDoctype, "html/missing-doctype", |dom, meta, ctx, out| {
    // A fragment or a template partial is not a document.
    if dom.doctype.is_some() || dom.by_tag("html").next().is_none() {
        return;
    }
    let Some(first) = dom.roots.iter().find_map(|c| if let crate::html_dom::Child::Elem(i) = c { Some(*i) } else { None }) else { return };
    let (s, t) = dom.tag_range(first);
    out.push(crate::html_dom::finding_with(s, t, "The document has no doctype; start it with `<!DOCTYPE html>`", vec![TextEdit::new(0, 0, "<!DOCTYPE html>\n")]));
});

/// A byte-order mark, as the file has it on disk: the text rules see never
/// holds one, so this reads the file's bytes (SR-ARCHITECTURE decision 15b).
/// Only the host document is checked: an HTML region of another file is not
/// where the mark is.
pub struct NoUtf8Bom;

impl crate::NativeRule for NoUtf8Bom {
    fn id(&self) -> &'static str {
        "html/no-utf8-bom"
    }
    fn languages(&self) -> &'static [codelaw_core::Language] {
        &[codelaw_core::Language::Html]
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["document"]
    }
    fn check(&self, _node: codelaw_lang::SyntaxNode<'_>, ctx: &crate::NativeContext<'_>, out: &mut Vec<crate::NativeFinding>) {
        if ctx.file.bytes().strip_prefix("\u{feff}".as_bytes()) == Some(ctx.source.as_bytes()) {
            out.push(crate::NativeFinding { start: 0, end: 0, message: "The file starts with a UTF-8 byte-order mark; save it as UTF-8 without one".into(), edits: Vec::new() });
        }
    }
}

html_rule!(NoMissingReferences, "html/no-missing-references", |dom, meta, ctx, out| {
    let ids = crate::html_a11y::ids(dom);
    for i in dom.all() {
        let e = &dom.elems[i];
        if meta.elem(&e.tag).is_none() {
            continue;
        }
        for a in &e.attrs {
            let Some(m) = meta.attr(&e.tag, &a.name) else { continue };
            if m.get("reference").and_then(|r| r.as_str()) != Some("id") || a.is_dynamic() || a.value_or_empty().is_empty() {
                continue;
            }
            let list = m.get("list").and_then(|l| l.as_bool()).unwrap_or(false);
            let refs = if list { tokens_with_offsets(a.value_or_empty(), a.value_start) } else { vec![(a.value_start, a.value_end, a.value_or_empty())] };
            for (s, t, id) in refs {
                if !ids.contains(id) {
                    out.push(finding(s, t, format!("`{}` points at the id `{id}`, which no element has", a.name)));
                }
            }
        }
    }
});

html_rule!(RequireSri, "html/require-sri", |dom, meta, ctx, out| {
    let all = ctx.options.get("target").and_then(|v| v.as_str()) != Some("crossorigin");
    let cross = Regex::new(r"^(?:\w+://|//)").expect("valid regex");
    for i in dom.all() {
        let e = &dom.elems[i];
        let key = match e.tag.as_str() {
            "script" => "src",
            "link" => {
                let rel = e.attr("rel").and_then(|r| r.value.clone()).unwrap_or_default();
                let ok = match rel.as_str() {
                    "stylesheet" | "modulepreload" => true,
                    "preload" => matches!(e.attr("as").and_then(|a| a.value.as_deref()), Some("style" | "script")),
                    _ => false,
                };
                if !ok {
                    continue;
                }
                "href"
            }
            _ => continue,
        };
        let Some(url) = e.attr(key).filter(|a| !a.value_or_empty().is_empty() && !a.is_dynamic()) else { continue };
        if !(all || cross.is_match(url.value_or_empty())) || e.has("integrity") {
            continue;
        }
        out.push(dom.at_elem(i, format!("<{}> loads `{}` without an `integrity` hash", e.tag, url.value_or_empty())));
    }
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html_dom::tests::run;

    #[test]
    fn headings() {
        assert!(run(&HeadingLevel, "<h1>a</h1><h2>b</h2><h3>c</h3><h2>d</h2>").is_empty());
        assert_eq!(run(&HeadingLevel, "<h1>a</h1><h3>b</h3>").len(), 1);
        assert_eq!(run(&HeadingLevel, "<h1>a</h1><h1>b</h1>").len(), 1);
        assert_eq!(run(&HeadingLevel, "<h2>a</h2>").len(), 1);
    }

    #[test]
    fn labels() {
        assert!(run(&InputMissingLabel, r#"<label for="a">Name</label><input id="a" type="text">"#).is_empty());
        assert!(run(&InputMissingLabel, r#"<label>Name <input type="text"></label>"#).is_empty());
        assert_eq!(run(&InputMissingLabel, r#"<input type="text">"#).len(), 1);
        assert!(run(&InputMissingLabel, r#"<input type="text" aria-label="Name">"#).is_empty());
    }
}
