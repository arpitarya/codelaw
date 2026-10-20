//! `html/*` accessibility rules: html-validate's `a11y` preset and its WCAG
//! techniques (H30, H32, H36, H37, H63, H67, H71).

use crate::html_dom::{
    aria_hidden, classify_text, finding, finding_with, flag, has_alt_text, has_aria_label, html_hidden, implicit_role, in_accessibility_tree,
    inert, naming_allowed, remove_attr, style_hidden, tokens_with_offsets, Dom, Meta, TextClass,
};
use crate::html_rule;
use std::collections::{BTreeMap, BTreeSet};

fn lt(tag: &str) -> String {
    format!("<{tag}>")
}

html_rule!(AreaAlt, "html/area-alt", |dom, meta, ctx, out| {
    let accessible = ctx.options.get("accessible").and_then(|v| v.as_bool()).unwrap_or(true);
    for map in dom.by_tag("map") {
        let areas: Vec<usize> = dom.descendants(map).into_iter().filter(|&a| dom.elems[a].is("area")).collect();
        for &a in &areas {
            let e = &dom.elems[a];
            match (e.attr("href"), e.attr("alt")) {
                (Some(href), alt) => {
                    if alt.is_some_and(|x| x.is_dynamic()) {
                        continue;
                    }
                    let ok = if accessible {
                        has_alt_text(e)
                    } else {
                        let target = href.value_or_empty();
                        areas.iter().any(|&o| dom.elems[o].attr("href").is_some_and(|h| h.value_or_empty() == target) && has_alt_text(&dom.elems[o]))
                    };
                    if !ok {
                        let at = alt.unwrap_or(href);
                        out.push(finding(at.name_start, at.name_end, "An <area> with `href` needs non-empty `alt` text"));
                    }
                }
                (None, Some(alt)) => {
                    out.push(finding(alt.name_start, alt.name_end, "`alt` on an <area> without `href` describes nothing"));
                }
                (None, None) => {}
            }
        }
    }
});

html_rule!(AriaHiddenBody, "html/aria-hidden-body", |dom, meta, ctx, out| {
    for i in dom.by_tag("body") {
        let Some(a) = dom.elems[i].attr("aria-hidden") else { continue };
        if a.value_or_empty().trim().eq_ignore_ascii_case("true") {
            out.push(finding_with(
                a.name_start,
                a.name_end,
                "`aria-hidden` on <body> hides the whole page from assistive technology",
                vec![remove_attr(ctx.source, a)],
            ));
        }
    }
});

const LABEL_ALLOWLIST: &[&str] = &[
    "article", "aside", "footer", "form", "header", "main", "nav", "search", "section", "area", "dialog", "fieldset", "figure", "iframe", "img",
    "summary", "table", "ul", "ol", "td", "th",
];

html_rule!(AriaLabelMisuse, "html/aria-label-misuse", |dom, meta, ctx, out| {
    let allow_any_namable = ctx.options.get("allowAnyNamable").and_then(|v| v.as_bool()).unwrap_or(false);
    for i in dom.all() {
        let e = &dom.elems[i];
        if e.in_foreign || meta.elem(&e.tag).is_none() {
            continue;
        }
        for key in ["aria-label", "aria-labelledby"] {
            let Some(a) = e.attr(key) else { continue };
            if a.value_or_empty().is_empty() || a.is_dynamic() {
                continue;
            }
            let explicit = meta.get(&e.tag, "attributes").and_then(|x| x.get("aria-label")).is_some();
            let valid = explicit
                || LABEL_ALLOWLIST.contains(&e.tag.as_str())
                || e.has("role")
                || e.has("tabindex")
                || flag(dom, meta, i, "interactive")
                || flag(dom, meta, i, "labelable");
            if valid {
                continue;
            }
            let namable = naming_allowed(dom, meta, i);
            if namable && allow_any_namable {
                continue;
            }
            let msg = if namable {
                format!("`{key}` is technically allowed on {} but screen readers may ignore it; label an interactive element or landmark instead", lt(&e.tag))
            } else {
                format!("`{key}` has no effect on {}: it can't have an accessible name", lt(&e.tag))
            };
            out.push(finding(a.name_start, a.name_end, msg));
        }
    }
});

fn tabindex(e: &crate::html_dom::Elem) -> Option<i64> {
    e.attr("tabindex").and_then(|a| a.value.as_deref()).and_then(|v| v.trim().parse::<i64>().ok())
}

fn disabled(dom: &Dom, meta: Meta<'_>, i: usize) -> bool {
    let disablable = meta.get(&dom.elems[i].tag, "formAssociated").and_then(|f| f.get("disablable")).and_then(|d| d.as_bool()).unwrap_or(false);
    disablable && (dom.elems[i].has("disabled") || dom.ancestors(i).any(|j| dom.elems[j].is("fieldset") && dom.elems[j].has("disabled")))
}

/// Whether keyboard focus can reach the element.
pub fn focusable(dom: &Dom, meta: Meta<'_>, i: usize) -> bool {
    if html_hidden(dom, i) || inert(dom, i) || dom.self_and_ancestors(i).any(|j| style_hidden(&dom.elems[j])) {
        return false;
    }
    if let Some(t) = tabindex(&dom.elems[i]) {
        return t >= 0;
    }
    if meta.elem(&dom.elems[i].tag).is_none() || disabled(dom, meta, i) {
        return false;
    }
    flag(dom, meta, i, "focusable")
}

html_rule!(HiddenFocusable, "html/hidden-focusable", |dom, meta, ctx, out| {
    for i in dom.all() {
        if !aria_hidden(dom, i) || !focusable(dom, meta, i) {
            continue;
        }
        let e = &dom.elems[i];
        match e.attr("aria-hidden") {
            Some(a) => out.push(finding(a.name_start, a.name_end, "A focusable element can't be `aria-hidden`: keyboard users reach what screen readers can't see")),
            None => out.push(dom.at_elem(i, format!("{} can take focus but an ancestor is `aria-hidden`", lt(&e.tag)))),
        }
    }
});

html_rule!(EmptyHeading, "html/empty-heading", |dom, meta, ctx, out| {
    for i in dom.all() {
        let e = &dom.elems[i];
        if !matches!(e.tag.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
            continue;
        }
        let desc = dom.descendants(i);
        if desc.iter().any(|&d| dom.elems[d].is("img") && has_alt_text(&dom.elems[d])) {
            continue;
        }
        if desc.iter().any(|&d| dom.elems[d].is("svg") && !dom.text_content(d).trim().is_empty()) {
            continue;
        }
        if classify_text(dom, i, false, true) == TextClass::Empty {
            out.push(dom.at_elem(i, format!("{} is empty; a heading needs text", lt(&e.tag))));
        }
    }
});

html_rule!(EmptyTitle, "html/empty-title", |dom, meta, ctx, out| {
    for i in dom.by_tag("title") {
        if dom.elems[i].in_foreign {
            continue;
        }
        if classify_text(dom, i, false, false) == TextClass::Empty {
            out.push(dom.at_elem(i, "<title> is empty; the page needs a title"));
        }
    }
});

html_rule!(MetaRefresh, "html/meta-refresh", |dom, meta, ctx, out| {
    let allow_long = ctx.options.get("allowLongDelay").and_then(|v| v.as_bool()).unwrap_or(false);
    let re = regex::Regex::new(r"(?i)^(\d+)(?:\s*;\s*url=(.*))?").expect("valid regex");
    for i in dom.by_tag("meta") {
        let e = &dom.elems[i];
        if e.value_lower("http-equiv").as_deref() != Some("refresh") {
            continue;
        }
        let Some(c) = e.attr("content").filter(|a| !a.value_or_empty().is_empty() && !a.is_dynamic()) else { continue };
        let at = (c.value_start, c.value_end);
        let Some(m) = re.captures(c.value_or_empty()) else {
            out.push(finding(at.0, at.1, "This meta refresh is malformed: write `0;url=…`"));
            continue;
        };
        let delay: u64 = m[1].parse().unwrap_or(u64::MAX);
        let url = m.get(2).is_some_and(|u| !u.as_str().is_empty());
        if allow_long && delay > 72_000 {
            continue;
        }
        if delay == 0 && !url {
            out.push(finding(at.0, at.1, "An instant meta refresh without a URL reloads the page forever"));
        } else if delay != 0 {
            out.push(finding(at.0, at.1, "A timed meta refresh moves on before everyone has read the page; make it instant (`0;url=…`)"));
        }
    }
});

html_rule!(MultipleLabeledControls, "html/multiple-labeled-controls", |dom, meta, ctx, out| {
    for l in dom.by_tag("label") {
        let ids: Vec<Option<String>> = dom
            .descendants(l)
            .into_iter()
            .filter(|&d| meta.get(&dom.elems[d].tag, "labelable").is_some() && flag(dom, meta, d, "labelable"))
            .map(|d| dom.elems[d].attr("id").and_then(|a| a.value.clone()))
            .collect();
        let mut n = ids.len();
        if let Some(f) = dom.elems[l].attr("for").filter(|a| !a.is_dynamic() && !a.value_or_empty().is_empty())
            && !ids.iter().any(|id| id.as_deref() == f.value.as_deref())
        {
            n += 1;
        }
        if n > 1 {
            out.push(dom.at_elem(l, "A <label> labels more than one control"));
        }
    }
});

const ABSTRACT_ROLES: &[&str] =
    &["command", "composite", "input", "landmark", "range", "roletype", "section", "sectionhead", "select", "structure", "widget", "window"];

html_rule!(NoAbstractRole, "html/no-abstract-role", |dom, meta, ctx, out| {
    for e in &dom.elems {
        let Some(a) = e.attr("role").filter(|a| !a.is_dynamic()) else { continue };
        for (s, t, role) in tokens_with_offsets(a.value_or_empty(), a.value_start) {
            if ABSTRACT_ROLES.contains(&role) {
                out.push(finding(s, t, format!("`{role}` is an abstract ARIA role; use one of its concrete roles")));
            }
        }
    }
});

html_rule!(NoAutoplay, "html/no-autoplay", |dom, meta, ctx, out| {
    let include: Vec<String> = ctx
        .options
        .get("include")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect())
        .unwrap_or_else(|| vec!["audio".into(), "video".into()]);
    for e in &dom.elems {
        if !include.contains(&e.tag) {
            continue;
        }
        let Some(a) = e.attr("autoplay").filter(|a| !a.is_dynamic()) else { continue };
        out.push(finding_with(
            a.start,
            a.end,
            format!("{} plays by itself; let the user start it", lt(&e.tag)),
            vec![remove_attr(ctx.source, a)],
        ));
    }
});

html_rule!(NoRedundantAriaLabel, "html/no-redundant-aria-label", |dom, meta, ctx, out| {
    for i in dom.all() {
        let e = &dom.elems[i];
        if !matches!(e.tag.as_str(), "input" | "textarea" | "select") {
            continue;
        }
        let (Some(label_attr), Some(id)) = (e.attr("aria-label"), e.attr("id").and_then(|a| a.value.as_deref())) else { continue };
        let Some(label) = dom.by_tag("label").find(|&l| dom.elems[l].attr("for").and_then(|a| a.value.as_deref()) == Some(id)) else { continue };
        if dom.text_content(label).trim() == label_attr.value_or_empty() {
            out.push(finding_with(
                label_attr.name_start,
                label_attr.name_end,
                "`aria-label` repeats the text of the control's <label>",
                vec![remove_attr(ctx.source, label_attr)],
            ));
        }
    }
});

html_rule!(NoRedundantFor, "html/no-redundant-for", |dom, meta, ctx, out| {
    for l in dom.by_tag("label") {
        let Some(f) = dom.elems[l].attr("for").filter(|a| !a.is_dynamic() && !a.value_or_empty().is_empty()) else { continue };
        let id = f.value_or_empty();
        if dom.descendants(l).into_iter().any(|d| dom.elems[d].attr("id").is_some_and(|a| a.value_or_empty() == id)) {
            out.push(finding_with(
                f.name_start,
                f.name_end,
                "`for` is redundant: the <label> already wraps its control",
                vec![remove_attr(ctx.source, f)],
            ));
        }
    }
});

html_rule!(NoRedundantRole, "html/no-redundant-role", |dom, meta, ctx, out| {
    for i in dom.all() {
        let e = &dom.elems[i];
        let Some(a) = e.attr("role").filter(|a| !a.is_dynamic() && !a.value_or_empty().is_empty()) else { continue };
        let Some(implicit) = implicit_role(dom, meta, i) else { continue };
        if a.value_or_empty() == implicit {
            out.push(finding_with(
                a.value_start,
                a.value_end,
                format!("{} already has the role `{implicit}`", lt(&e.tag)),
                vec![remove_attr(ctx.source, a)],
            ));
        }
    }
});

const NATIVE: &[(&str, &str)] = &[
    ("article", "article"),
    ("banner", "header"),
    ("button", "button"),
    ("cell", "td"),
    ("checkbox", "input"),
    ("complementary", "aside"),
    ("contentinfo", "footer"),
    ("figure", "figure"),
    ("form", "form"),
    ("heading", "hN"),
    ("input", "input"),
    ("link", "a"),
    ("list", "ul"),
    ("listbox", "select"),
    ("listitem", "li"),
    ("main", "main"),
    ("navigation", "nav"),
    ("progressbar", "progress"),
    ("radio", "input"),
    ("region", "section"),
    ("table", "table"),
    ("textbox", "textarea"),
];

html_rule!(PreferNativeElement, "html/prefer-native-element", |dom, meta, ctx, out| {
    for e in &dom.elems {
        let Some(a) = e.attr("role").filter(|a| !a.is_dynamic()) else { continue };
        let role = a.value_or_empty().trim().to_ascii_lowercase();
        let Some((_, native)) = NATIVE.iter().find(|(r, _)| *r == role) else { continue };
        if e.tag == *native {
            continue;
        }
        out.push(finding(a.start, a.end, format!("Use a native <{native}> instead of `role=\"{role}\"`")));
    }
});

fn has_text_or_label(dom: &Dom, meta: Meta<'_>, i: usize) -> bool {
    if !in_accessibility_tree(dom, meta, i) {
        return false;
    }
    let e = &dom.elems[i];
    let direct_text = e.children.iter().any(|c| match c {
        crate::html_dom::Child::Text(t) => {
            let t = dom.text_of(&dom.texts[*t]);
            !t.trim().is_empty()
        }
        _ => false,
    });
    let nonempty = |n: &str| e.attr(n).is_some_and(|a| !a.value_or_empty().is_empty());
    if direct_text || nonempty("aria-label") || nonempty("aria-labelledby") || (e.is("img") && nonempty("alt")) || e.is("selectedcontent") {
        return true;
    }
    if e.is("input") && !e.has("value") && matches!(e.value_lower("type").as_deref(), Some("submit" | "reset")) {
        return true;
    }
    dom.child_elems(i).any(|c| has_text_or_label(dom, meta, c))
}

html_rule!(TextContent, "html/text-content", |dom, meta, ctx, out| {
    for i in dom.all() {
        let e = &dom.elems[i];
        let Some(kind) = meta.get(&e.tag, "textContent").and_then(|v| v.as_str()) else { continue };
        match kind {
            "none" if classify_text(dom, i, false, false) != TextClass::Empty => {
                out.push(dom.at_elem(i, format!("{} must not contain text", lt(&e.tag))));
            }
            "required" if classify_text(dom, i, false, false) == TextClass::Empty => {
                out.push(dom.at_elem(i, format!("{} needs text content", lt(&e.tag))));
            }
            "accessible" if in_accessibility_tree(dom, meta, i) && !has_text_or_label(dom, meta, i) => {
                out.push(dom.at_elem(i, format!("{} has no text a screen reader can announce; add text or `aria-label`", lt(&e.tag))));
            }
            _ => {}
        }
    }
});

const LANDMARK_ROLES: &[&str] = &["complementary", "contentinfo", "form", "banner", "main", "navigation", "region"];

html_rule!(UniqueLandmark, "html/unique-landmark", |dom, meta, ctx, out| {
    let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for i in dom.all() {
        let e = &dom.elems[i];
        let role = match e.value_lower("role") {
            Some(r) => Some(r),
            None if matches!(e.tag.as_str(), "aside" | "footer" | "form" | "header" | "main" | "nav" | "section") => implicit_role(dom, meta, i),
            None => None,
        };
        let Some(role) = role.filter(|r| LANDMARK_ROLES.contains(&r.as_str())) else { continue };
        groups.entry(role).or_default().push(i);
    }
    for nodes in groups.values().filter(|n| n.len() > 1) {
        // (element, name, where to report)
        let entries: Vec<(usize, Option<String>, (usize, usize))> = nodes
            .iter()
            .map(|&i| {
                let e = &dom.elems[i];
                if let Some(a) = e.attr("aria-label") {
                    return (i, a.value.clone(), (a.name_start, a.name_end));
                }
                if let Some(a) = e.attr("aria-labelledby") {
                    let id = a.value_or_empty();
                    let text = dom.by_id(id).next().map(|r| dom.text_content(r)).unwrap_or_else(|| format!("#{id}"));
                    return (i, Some(text), (a.name_start, a.name_end));
                }
                (i, None, dom.tag_range(i))
            })
            .collect();
        for (i, text, at) in &entries {
            let e = &dom.elems[*i];
            if text.is_none() && (e.is("form") || e.is("section")) {
                continue;
            }
            if text.as_deref().is_some_and(crate::html_dom::is_dynamic) {
                continue;
            }
            let dup = entries.iter().filter(|(_, t, _)| t == text).count() > 1;
            if text.as_deref().is_some_and(|t| !t.is_empty()) && !dup {
                continue;
            }
            out.push(finding(at.0, at.1, "Repeated landmarks each need a unique, non-empty name (`aria-label` or `aria-labelledby`)"));
        }
    }
});

// ------------------------------------------------------------------ WCAG

html_rule!(LinkText, "html/link-text", |dom, meta, ctx, out| {
    for a in dom.by_tag("a") {
        let e = &dom.elems[a];
        if !e.has("href") || !in_accessibility_tree(dom, meta, a) {
            continue;
        }
        if classify_text(dom, a, false, true) != TextClass::Empty {
            continue;
        }
        let desc = dom.descendants(a);
        if desc.iter().any(|&d| dom.elems[d].is("img") && has_alt_text(&dom.elems[d])) {
            continue;
        }
        if has_aria_label(e) || desc.iter().any(|&d| has_aria_label(&dom.elems[d])) {
            continue;
        }
        out.push(dom.at_elem(a, "A link needs text that says where it goes"));
    }
});

fn submit_button(dom: &Dom, meta: Meta<'_>, i: usize) -> bool {
    meta.get(&dom.elems[i].tag, "submitButton").is_some() && flag(dom, meta, i, "submitButton")
}

html_rule!(FormSubmit, "html/form-submit", |dom, meta, ctx, out| {
    for f in dom.by_tag("form") {
        let nested = dom.descendants(f).into_iter().any(|d| submit_button(dom, meta, d) && !dom.elems[d].has("form"));
        if nested {
            continue;
        }
        let id = dom.elems[f].attr("id").and_then(|a| a.value.clone()).unwrap_or_default();
        let associated = !id.is_empty()
            && dom.all().any(|d| submit_button(dom, meta, d) && dom.elems[d].attr("form").is_some_and(|a| a.value_or_empty() == id));
        if !associated {
            out.push(dom.at_elem(f, "A <form> needs a submit button"));
        }
    }
});

html_rule!(InputImageAlt, "html/input-image-alt", |dom, meta, ctx, out| {
    for i in dom.by_tag("input") {
        let e = &dom.elems[i];
        if e.value_lower("type").as_deref() != Some("image") || !in_accessibility_tree(dom, meta, i) || has_alt_text(e) {
            continue;
        }
        match e.attr("alt") {
            Some(a) => out.push(finding(a.name_start, a.name_end, "An image submit button needs non-empty `alt` text")),
            None => out.push(dom.at_elem(i, "An image submit button needs non-empty `alt` text")),
        }
    }
});

html_rule!(ImgAlt, "html/img-alt", |dom, meta, ctx, out| {
    let allow_empty = ctx.options.get("allowEmpty").and_then(|v| v.as_bool()).unwrap_or(true);
    let alias: Vec<String> =
        ctx.options.get("alias").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect()).unwrap_or_default();
    for i in dom.by_tag("img") {
        let e = &dom.elems[i];
        if !in_accessibility_tree(dom, meta, i) {
            continue;
        }
        if has_alt_text(e) || (e.has("alt") && allow_empty) || alias.iter().any(|a| e.attr(a).is_some_and(|x| !x.value_or_empty().is_empty())) {
            continue;
        }
        match e.attr("alt") {
            Some(a) => out.push(finding(a.name_start, a.name_end, "<img> has an empty `alt`; describe the image")),
            None => out.push(dom.at_elem(i, "<img> needs an `alt` attribute: a description, or `alt=\"\"` if it is decorative")),
        }
    }
});

/// A table whose headers are only its first row, or only its first column.
fn simple_table(dom: &Dom, t: usize) -> bool {
    let mut rows = Vec::new();
    for c in dom.child_elems(t) {
        match dom.elems[c].tag.as_str() {
            "tr" => rows.push(c),
            "thead" | "tbody" => rows.extend(dom.child_elems(c).filter(|&r| dom.elems[r].is("tr"))),
            _ => {}
        }
    }
    let has_headers_attr =
        rows.iter().any(|&r| dom.child_elems(r).any(|c| dom.elems[c].has("headers")) && dom.elems[r].parent.is_some_and(|p| !dom.elems[p].is("thead")));
    if has_headers_attr || rows.is_empty() {
        return false;
    }
    let cells: Vec<Vec<u8>> = rows.iter().map(|&r| dom.child_elems(r).map(|c| u8::from(dom.elems[c].is("th"))).collect()).collect();
    let cols = cells[0].len();
    if cols == 0 || cells.iter().any(|r| r.len() != cols) {
        return false;
    }
    let per_row: Vec<usize> = cells.iter().map(|r| r.iter().map(|&x| x as usize).sum()).collect();
    if per_row[0] == cols && per_row[1..].iter().all(|&n| n == 0) {
        return true;
    }
    let per_col: Vec<usize> = (0..cols).map(|c| cells.iter().map(|r| r[c] as usize).sum()).collect();
    let thead = dom.child_elems(t).any(|c| dom.elems[c].is("thead"));
    per_col[0] == cells.len() && per_col[1..].iter().all(|&n| n == 0) && !thead
}

html_rule!(ThScope, "html/th-scope", |dom, meta, ctx, out| {
    let strict = ctx.options.get("strict").and_then(|v| v.as_bool()).unwrap_or(false);
    const SCOPES: &[&str] = &["row", "col", "rowgroup", "colgroup"];
    for t in dom.by_tag("table") {
        if !strict && simple_table(dom, t) {
            continue;
        }
        for th in dom.descendants(t).into_iter().filter(|&d| dom.elems[d].is("th")) {
            let scope = dom.elems[th].attr("scope");
            if scope.is_some_and(|s| s.is_dynamic() || SCOPES.contains(&s.value_or_empty())) {
                continue;
            }
            let msg = "In a table with more than one row or column of headers, each <th> needs `scope`: row, col, rowgroup or colgroup";
            match scope {
                Some(s) => out.push(finding(s.start, s.end, msg)),
                None => out.push(dom.at_elem(th, msg)),
            }
        }
    }
});

html_rule!(ImgTitle, "html/img-title", |dom, meta, ctx, out| {
    for i in dom.by_tag("img") {
        let e = &dom.elems[i];
        let Some(title) = e.attr("title").filter(|a| !a.value_or_empty().is_empty()) else { continue };
        if has_alt_text(e) {
            continue;
        }
        out.push(finding(title.name_start, title.name_end, "A decorative <img> (empty `alt`) can't have a `title`: describe it in `alt` or drop the `title`"));
    }
});

html_rule!(FieldsetLegend, "html/fieldset-legend", |dom, meta, ctx, out| {
    for f in dom.by_tag("fieldset") {
        if !dom.child_elems(f).any(|c| dom.elems[c].is("legend")) {
            out.push(dom.at_elem(f, "A <fieldset> needs a <legend> naming the group"));
        }
    }
});

// ------------------------------------------------------------ autocomplete

const FIELD1: &[&str] = &[
    "name", "honorific-prefix", "given-name", "additional-name", "family-name", "honorific-suffix", "nickname", "username", "new-password",
    "current-password", "one-time-code", "organization-title", "organization", "street-address", "address-line1", "address-line2",
    "address-line3", "address-level4", "address-level3", "address-level2", "address-level1", "country", "country-name", "postal-code", "cc-name",
    "cc-given-name", "cc-additional-name", "cc-family-name", "cc-number", "cc-exp", "cc-exp-month", "cc-exp-year", "cc-csc", "cc-type",
    "transaction-currency", "transaction-amount", "language", "bday", "bday-day", "bday-month", "bday-year", "sex", "url", "photo",
];
const FIELD2: &[&str] = &["tel", "tel-country-code", "tel-national", "tel-area-code", "tel-local", "tel-local-prefix", "tel-local-suffix", "tel-extension", "email", "impp"];

/// The control group a field name belongs to (WHATWG autofill table).
fn field_group(field: &str) -> &'static str {
    match field {
        "username" | "email" => "username",
        "new-password" | "current-password" | "one-time-code" => "password",
        "street-address" => "multiline",
        "cc-exp" => "month",
        "cc-exp-month" | "cc-exp-year" | "transaction-amount" | "bday-day" | "bday-month" | "bday-year" => "numeric",
        "bday" => "date",
        "url" | "photo" | "impp" => "url",
        "tel" => "tel",
        _ => "text",
    }
}

fn control_groups(ty: &str) -> &'static [&'static str] {
    const ALL: &[&str] = &["text", "multiline", "password", "url", "username", "tel", "numeric", "month", "date"];
    const NO_MULTILINE: &[&str] = &["text", "password", "url", "username", "tel", "numeric", "month", "date"];
    match ty {
        "hidden" => ALL,
        "text" | "search" => NO_MULTILINE,
        "password" => &["password"],
        "url" => &["url"],
        "email" => &["username"],
        "tel" => &["tel"],
        "number" => &["numeric"],
        "month" => &["month"],
        "date" => &["date"],
        _ => &[],
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Tok {
    Section,
    Hint,
    Contact,
    Field1,
    Field2,
    Webauthn,
}

fn classify(token: &str) -> Option<Tok> {
    if token.starts_with("section-") {
        Some(Tok::Section)
    } else if token == "shipping" || token == "billing" {
        Some(Tok::Hint)
    } else if FIELD1.contains(&token) {
        Some(Tok::Field1)
    } else if FIELD2.contains(&token) {
        Some(Tok::Field2)
    } else if matches!(token, "home" | "work" | "mobile" | "fax" | "pager") {
        Some(Tok::Contact)
    } else if token == "webauthn" {
        Some(Tok::Webauthn)
    } else {
        None
    }
}

fn order_rank(t: Tok) -> u8 {
    match t {
        Tok::Section => 0,
        Tok::Hint => 1,
        Tok::Contact => 2,
        Tok::Field1 => 3,
        Tok::Field2 => 4,
        Tok::Webauthn => 5,
    }
}

html_rule!(ValidAutocomplete, "html/valid-autocomplete", |dom, meta, ctx, out| {
    for i in dom.all() {
        let e = &dom.elems[i];
        let Some(a) = e.attr("autocomplete").filter(|a| a.value.is_some() && !a.is_dynamic()) else { continue };
        let lower = a.value_or_empty().to_ascii_lowercase();
        let toks = tokens_with_offsets(&lower, a.value_start);
        if toks.is_empty() {
            continue;
        }
        match e.tag.as_str() {
            "form" => {
                let v = lower.trim();
                if v != "on" && v != "off" {
                    out.push(finding(a.value_start, a.value_end, format!("<form> takes only `on` or `off` for `autocomplete`, not `\"{v}\"`")));
                }
            }
            "input" | "textarea" | "select" => {
                let ty = if e.is("input") { e.value_lower("type").unwrap_or_else(|| "text".into()) } else { "text".into() };
                if e.is("input") && matches!(ty.as_str(), "checkbox" | "radio" | "file" | "submit" | "image" | "reset" | "button") {
                    out.push(finding(a.name_start, a.name_end, format!("`autocomplete` has no meaning on <input type=\"{ty}\">")));
                    continue;
                }
                if let Some(k) = toks.iter().position(|t| t.2 == "on" || t.2 == "off") {
                    let (s, t, v) = toks[k];
                    if toks.len() > 1 {
                        out.push(finding(s, t, format!("`{v}` must be the only token in `autocomplete`")));
                    }
                    if ty == "hidden" {
                        out.push(finding(toks[0].0, toks[0].1, format!("<input type=\"hidden\"> can't use `autocomplete=\"{v}\"`")));
                    }
                    continue;
                }
                let mut kinds = Vec::new();
                let mut bad = false;
                for &(s, t, tok) in &toks {
                    match classify(tok) {
                        Some(k) => kinds.push(k),
                        None => {
                            out.push(finding(s, t, format!("`{tok}` is not an autocomplete token or field name")));
                            bad = true;
                            break;
                        }
                    }
                }
                if bad {
                    continue;
                }
                let fields: Vec<usize> = kinds.iter().enumerate().filter(|(_, k)| matches!(k, Tok::Field1 | Tok::Field2)).map(|(n, _)| n).collect();
                if fields.is_empty() {
                    out.push(finding(a.name_start, a.name_end, "`autocomplete` needs a field name, such as `email` or `street-address`"));
                } else if fields.len() > 1 {
                    let (s, t, v) = toks[fields[fields.len() - 1]];
                    out.push(finding(s, t, format!("`{v}` can't be combined with `{}`: one field name only", toks[fields[0]].2)));
                }
                if let (Some(c), Some(f1)) = (kinds.iter().position(|k| *k == Tok::Contact), kinds.iter().position(|k| *k == Tok::Field1)) {
                    let (s, t, v) = toks[c];
                    out.push(finding(s, t, format!("`{v}` only goes with phone, email or messaging fields, not `{}`", toks[f1].2)));
                }
                for n in 0..kinds.len().saturating_sub(1) {
                    if order_rank(kinds[0]) > order_rank(kinds[n + 1]) {
                        let (s, t, v) = toks[n + 1];
                        out.push(finding(s, t, format!("`{v}` must come before `{}` in `autocomplete`", toks[n].2)));
                    }
                }
                if let Some(&f) = fields.first()
                    && e.is("input")
                    && !e.attr("type").is_some_and(|x| x.is_dynamic())
                {
                    let field = toks[f].2;
                    let group = field_group(field);
                    if !control_groups(&ty).contains(&group) {
                        let (s, t, _) = toks[f];
                        out.push(finding(s, t, format!("`{field}` doesn't fit <input type=\"{ty}\">")));
                    }
                }
            }
            _ => {}
        }
    }
});

html_rule!(AutocompletePassword, "html/autocomplete-password", |dom, meta, ctx, out| {
    let preferred = ctx.options.get("preferred").and_then(|v| v.as_str()).filter(|p| !p.is_empty()).map(str::to_ascii_lowercase);
    for i in dom.by_tag("input") {
        let e = &dom.elems[i];
        if e.value_lower("type").as_deref() != Some("password") {
            continue;
        }
        let Some(a) = e.attr("autocomplete") else {
            out.push(dom.at_elem(i, "A password field needs `autocomplete`: `new-password` or `current-password`"));
            continue;
        };
        if a.is_dynamic() || a.value.is_none() {
            continue;
        }
        let lower = a.value_or_empty().to_ascii_lowercase();
        let toks = tokens_with_offsets(&lower, a.value_start);
        let Some(&(s, t, v)) = toks.iter().find(|t| !(t.2.starts_with("section-") || t.2 == "shipping" || t.2 == "billing")) else { continue };
        if v == "off" {
            out.push(finding(s, t, "Don't turn `autocomplete` off on a password field; say `new-password` or `current-password`"));
        } else if let Some(p) = &preferred
            && v != p
        {
            out.push(finding(s, t, format!("This project's password fields use `autocomplete=\"{p}\"`")));
        }
    }
});

/// Ids every element in the file declares, for rules that check references.
pub fn ids(dom: &Dom) -> BTreeSet<String> {
    dom.elems.iter().filter_map(|e| e.attr("id").and_then(|a| a.value.clone())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html_dom::tests::run;

    #[test]
    fn redundant_role() {
        assert_eq!(run(&NoRedundantRole, r#"<ul role="list"><li>a</li></ul>"#).len(), 1);
        assert!(run(&NoRedundantRole, r#"<ul role="menu"><li>a</li></ul>"#).is_empty());
        assert_eq!(run(&NoRedundantRole, r#"<a href="/" role="link">x</a>"#).len(), 1);
        assert!(run(&NoRedundantRole, r#"<a role="link">x</a>"#).is_empty());
    }

    #[test]
    fn hidden_focusable() {
        assert_eq!(run(&HiddenFocusable, r#"<a href="/" aria-hidden="true">x</a>"#).len(), 1);
        assert!(run(&HiddenFocusable, r#"<a href="/" aria-hidden="true" tabindex="-1">x</a>"#).is_empty());
        assert!(run(&HiddenFocusable, r#"<span aria-hidden="true">x</span>"#).is_empty());
    }

    #[test]
    fn autocomplete() {
        assert!(run(&ValidAutocomplete, r#"<input type="email" autocomplete="work email">"#).is_empty());
        assert_eq!(run(&ValidAutocomplete, r#"<input type="email" autocomplete="foo">"#).len(), 1);
        assert_eq!(run(&ValidAutocomplete, r#"<input type="email" autocomplete="email shipping">"#).len(), 1);
        assert_eq!(run(&ValidAutocomplete, r#"<input type="number" autocomplete="email">"#).len(), 1);
    }

    #[test]
    fn th_scope() {
        assert!(run(&ThScope, "<table><tr><th>a</th><th>b</th></tr><tr><td>1</td><td>2</td></tr></table>").is_empty());
        let f = run(&ThScope, "<table><tr><th>a</th><th>b</th></tr><tr><th>1</th><td>2</td></tr></table>");
        assert_eq!(f.len(), 3);
    }
}
