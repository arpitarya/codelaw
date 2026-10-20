//! `html/*` rules on attributes, ids, character references and forms.

use crate::html_dom::{finding, finding_with, flag, html_hidden, inert, is_dynamic, remove_attr, tokens_with_offsets, Child, Dom, Meta, TextKind};
use crate::html_rule;
use codelaw_core::TextEdit;
use regex::Regex;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

fn lt(tag: &str) -> String {
    format!("<{tag}>")
}

fn opt_strings(ctx: &crate::NativeContext<'_>, key: &str) -> Option<Vec<String>> {
    ctx.options.get(key).and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_owned)).collect())
}

/// The offset just after the tag name of a start tag, where a new attribute
/// can be inserted.
fn after_tag_name(src: &str, start: usize) -> usize {
    let rest = &src[start + 1..];
    let n = rest.find(|c: char| c.is_ascii_whitespace() || c == '>' || c == '/').unwrap_or(rest.len());
    start + 1 + n
}

html_rule!(NoDupAttr, "html/no-dup-attr", |dom, meta, ctx, out| {
    for e in &dom.elems {
        let mut seen = BTreeSet::new();
        for a in &e.attrs {
            if !seen.insert(a.name.as_str()) {
                out.push(finding_with(
                    a.name_start,
                    a.name_end,
                    format!("`{}` is written twice on {}; browsers keep only the first", a.name, lt(&e.tag)),
                    vec![remove_attr(ctx.source, a)],
                ));
            }
        }
    }
});

html_rule!(NoDupClass, "html/no-dup-class", |dom, meta, ctx, out| {
    let src = ctx.source;
    for e in &dom.elems {
        for a in e.attrs.iter().filter(|a| a.name == "class" && !a.is_dynamic()) {
            let mut seen = BTreeSet::new();
            for (s, t, class) in tokens_with_offsets(a.value_or_empty(), a.value_start) {
                if seen.insert(class) {
                    continue;
                }
                let before = src[..s].trim_end_matches([' ', '\t', '\n', '\r']).len();
                out.push(finding_with(s, t, format!("The class `{class}` is listed twice"), vec![TextEdit::new(before, t, "")]));
            }
        }
    }
});

html_rule!(NoDuplicateId, "html/no-duplicate-id", |dom, meta, ctx, out| {
    // Ids inside a <template> are scoped to it.
    let mut seen: BTreeMap<Option<usize>, BTreeSet<String>> = BTreeMap::new();
    for i in dom.all() {
        let e = &dom.elems[i];
        let Some(a) = e.attr("id").filter(|a| !a.value_or_empty().is_empty() && !a.is_dynamic()) else { continue };
        let group = dom.ancestors(i).find(|&j| dom.elems[j].is("template"));
        let id = a.value_or_empty().to_owned();
        if !seen.entry(group).or_default().insert(id.clone()) {
            out.push(finding(a.value_start, a.value_end, format!("The id `{id}` is already used in this document")));
        }
    }
});

html_rule!(ButtonType, "html/button-type", |dom, meta, ctx, out| {
    for i in dom.by_tag("button") {
        let e = &dom.elems[i];
        if e.has("type") || e.parent.is_some_and(|p| dom.elems[p].is("select")) {
            continue;
        }
        let at = after_tag_name(ctx.source, e.start);
        out.push(finding_with(
            e.start,
            e.open_end,
            "<button> has no `type`, so it submits its form; say `type=\"button\"` or `type=\"submit\"`",
            vec![TextEdit::new(at, at, " type=\"button\"")],
        ));
    }
});

html_rule!(NoImplicitInputType, "html/no-implicit-input-type", |dom, meta, ctx, out| {
    for i in dom.by_tag("input") {
        let e = &dom.elems[i];
        if e.has("type") {
            continue;
        }
        let at = after_tag_name(ctx.source, e.start);
        out.push(finding_with(e.start, e.open_end, "<input> has no `type`; say `type=\"text\"` if that's what it is", vec![TextEdit::new(at, at, " type=\"text\"")]));
    }
});

fn entity_for(c: char) -> &'static str {
    match c {
        '"' => "&quot;",
        '&' => "&amp;",
        '\'' => "&apos;",
        '<' => "&lt;",
        '=' => "&equals;",
        '>' => "&gt;",
        '`' => "&grave;",
        _ => "",
    }
}

/// `&` that doesn't start a terminated character reference.
fn ambiguous_amp(s: &str, k: usize) -> bool {
    let rest = &s[k + 1..];
    let n = rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '#')).unwrap_or(rest.len());
    !(n > 0 && rest[n..].starts_with(';'))
}

fn template_text(t: &str) -> bool {
    let t = t.trim();
    (t.starts_with("<%") && t.ends_with("%>")) || (t.starts_with("<?") && t.ends_with("?>")) || (t.starts_with("<$") && t.ends_with("$>"))
}

html_rule!(NoRawCharacters, "html/no-raw-characters", |dom, meta, ctx, out| {
    let relaxed = ctx.options.get("relaxed").and_then(|v| v.as_bool()).unwrap_or(false);
    let src = ctx.source;
    let report = |at: usize, c: char, out: &mut Vec<crate::NativeFinding>| {
        if relaxed && c == '&' {
            return;
        }
        let ent = entity_for(c);
        out.push(finding_with(at, at + 1, format!("Write `{c}` as `{ent}`"), vec![TextEdit::new(at, at + 1, ent)]));
    };
    for t in &dom.texts {
        let s = &src[t.start..t.end];
        match t.kind {
            TextKind::Raw => {}
            TextKind::Entity => {
                if !s.ends_with(';') && ambiguous_amp(src, t.start) {
                    report(t.start, '&', out);
                }
            }
            TextKind::Plain | TextKind::Unparsed => {
                if template_text(s) || is_dynamic(s) {
                    continue;
                }
                for (k, c) in s.char_indices() {
                    if (c == '&' && ambiguous_amp(src, t.start + k)) || c == '<' {
                        report(t.start + k, c, out);
                    }
                }
            }
        }
    }
    for i in dom.all() {
        let e = &dom.elems[i];
        for a in &e.attrs {
            let Some(v) = a.value.as_deref() else { continue };
            if a.quoted || v.is_empty() || is_dynamic(v) || meta.attr(&e.tag, &a.name).is_some_and(|m| m.get("boolean") == Some(&Value::Bool(true))) {
                continue;
            }
            for (k, c) in v.char_indices() {
                let raw = matches!(c, '"' | '\'' | '<' | '=' | '>' | '`') || (c == '&' && ambiguous_amp(src, a.value_start + k));
                if raw {
                    report(a.value_start + k, c, out);
                }
            }
        }
    }
});

html_rule!(UnrecognizedCharRef, "html/unrecognized-char-ref", |dom, meta, ctx, out| {
    let Some(Value::Array(list)) = ctx.data.get("html/entities").map(|v| v.as_ref()) else { return };
    let ignore_case = ctx.options.get("ignoreCase").and_then(|v| v.as_bool()).unwrap_or(false);
    let require_semicolon = ctx.options.get("requireSemicolon").and_then(|v| v.as_bool()).unwrap_or(true);
    let norm = |s: &str| if ignore_case { s.to_ascii_lowercase() } else { s.to_owned() };
    let entities: BTreeSet<String> = list.iter().filter_map(|x| x.as_str()).map(norm).collect();
    let mut legacy: Vec<&String> = entities.iter().filter(|e| !e.ends_with(';')).collect();
    legacy.sort_by_key(|e| std::cmp::Reverse(e.len()));
    let re = Regex::new(r"(?i)&(?:[\da-z]+|#x?[\da-f]+)").expect("valid regex");
    let src = ctx.source;
    let check = |start: usize, end: usize, is_attr: bool, out: &mut Vec<crate::NativeFinding>| {
        let s = &src[start..end];
        let delimiter = if is_attr { s.find(['#', '?']) } else { None };
        for m in re.find_iter(s) {
            let next = src[start + m.end()..].chars().next();
            if next.is_some_and(|c| c.is_ascii_alphanumeric()) {
                continue;
            }
            let terminated = next == Some(';');
            let raw = m.as_str();
            if raw.starts_with("&#") {
                continue;
            }
            if !terminated && delimiter.is_some_and(|d| m.start() > d) {
                continue;
            }
            let name = norm(raw);
            let with_semi = format!("{name};");
            let found = if terminated { entities.contains(&with_semi) } else { entities.contains(&name) };
            if found && (terminated || !require_semicolon) {
                continue;
            }
            let at = start + m.start();
            if !terminated {
                let legacy_name = if !is_attr && require_semicolon { legacy.iter().find(|l| name.starts_with(l.as_str())).map(|l| l.len()) } else { None };
                let len = legacy_name.or_else(|| (found || entities.contains(&with_semi)).then_some(raw.len()));
                if let Some(len) = len {
                    out.push(finding(at, at + len, format!("End the character reference `{}` with `;`", &raw[..len])));
                }
                continue;
            }
            out.push(finding(at, at + raw.len() + 1, format!("`{raw};` is not a character reference HTML knows")));
        }
    };
    for t in &dom.texts {
        if t.kind != TextKind::Raw {
            check(t.start, t.end, false, out);
        }
    }
    for e in &dom.elems {
        for a in e.attrs.iter().filter(|a| a.value.is_some() && !a.is_dynamic()) {
            check(a.value_start, a.value_end, true, out);
        }
    }
});

html_rule!(DoctypeHtml, "html/doctype-html", |dom, meta, ctx, out| {
    let Some((s, t)) = dom.doctype else { return };
    let text = &ctx.source[s..t];
    let lower = text.to_ascii_lowercase();
    let Some(k) = lower.find("doctype") else { return };
    let value_start = s + k + "doctype".len();
    let value_end = t - usize::from(text.ends_with('>'));
    let value = ctx.source[value_start..value_end].trim();
    if value.eq_ignore_ascii_case("html") {
        return;
    }
    out.push(finding_with(s, t, "Use the HTML5 doctype, `<!DOCTYPE html>`", vec![TextEdit::new(value_start, value_end, " html")]));
});

html_rule!(MapDupName, "html/map-dup-name", |dom, meta, ctx, out| {
    let mut seen = BTreeSet::new();
    for i in dom.by_tag("map") {
        let Some(a) = dom.elems[i].attr("name").filter(|a| !a.value_or_empty().is_empty() && !a.is_dynamic()) else { continue };
        if !seen.insert(a.value_or_empty().to_owned()) {
            out.push(finding(a.name_start, a.name_end, format!("Another <map> is already named `{}`", a.value_or_empty())));
        }
    }
});

html_rule!(MapIdName, "html/map-id-name", |dom, meta, ctx, out| {
    for i in dom.by_tag("map") {
        let e = &dom.elems[i];
        let (Some(id), Some(name)) = (e.attr("id"), e.attr("name")) else { continue };
        if id.is_dynamic() || name.is_dynamic() || id.value == name.value {
            continue;
        }
        out.push(finding(id.value_start, id.value_end.max(id.value_start), "A <map>'s `id` and `name` must be the same"));
    }
});

fn enabled(dom: &Dom, meta: Meta<'_>, i: usize) -> bool {
    if html_hidden(dom, i) || inert(dom, i) {
        return false;
    }
    let disablable = meta.get(&dom.elems[i].tag, "formAssociated").and_then(|f| f.get("disablable")).and_then(|d| d.as_bool()).unwrap_or(false);
    !(disablable && (dom.elems[i].has("disabled") || dom.ancestors(i).any(|j| dom.elems[j].is("fieldset") && dom.elems[j].has("disabled"))))
}

fn control_type(e: &crate::html_dom::Elem) -> String {
    let t = e.attr("type").map(|a| a.value_or_empty().to_owned()).unwrap_or_default();
    if e.is("button") && t.is_empty() { "submit".into() } else { t }
}

#[derive(Clone, Copy)]
struct Seen {
    array: bool,
    hidden_default: bool,
}

html_rule!(FormDupName, "html/form-dup-name", |dom, meta, ctx, out| {
    let allow_brackets = ctx.options.get("allowArrayBrackets").and_then(|v| v.as_bool()).unwrap_or(true);
    let allow_checkbox_default = ctx.options.get("allowCheckboxDefault").and_then(|v| v.as_bool()).unwrap_or(true);
    let shared = opt_strings(ctx, "shared").unwrap_or_else(|| vec!["radio".into(), "button".into(), "reset".into(), "submit".into()]);
    let listed = |i: usize| {
        meta.get(&dom.elems[i].tag, "formAssociated").and_then(|f| f.get("listed")).and_then(|l| l.as_bool()).unwrap_or(false)
    };
    let controls: Vec<usize> = dom.all().filter(|&i| listed(i) && enabled(dom, meta, i)).collect();
    let group_of = |i: usize| dom.ancestors(i).find(|&j| dom.elems[j].is("form") || dom.elems[j].is("template"));
    let mut unique: BTreeMap<Option<usize>, BTreeMap<String, Seen>> = BTreeMap::new();
    let mut shared_seen: BTreeMap<Option<usize>, BTreeMap<String, String>> = BTreeMap::new();
    let (shared_ctl, unique_ctl): (Vec<usize>, Vec<usize>) = controls.into_iter().partition(|&i| shared.contains(&control_type(&dom.elems[i])));
    for i in unique_ctl {
        let e = &dom.elems[i];
        let Some(a) = e.attr("name").filter(|a| !a.value_or_empty().is_empty() && !a.is_dynamic()) else { continue };
        let name = a.value_or_empty();
        let map = unique.entry(group_of(i)).or_default();
        if allow_brackets {
            let is_array = name.ends_with("[]");
            let base = name.strip_suffix("[]").unwrap_or(name);
            if let Some(d) = map.get(base)
                && d.array != is_array
            {
                out.push(finding(a.value_start, a.value_end, format!("Don't mix `{base}[]` and `{base}` as control names")));
                continue;
            }
            if is_array {
                map.entry(base.to_owned()).or_insert(Seen { array: true, hidden_default: false });
                continue;
            }
        }
        match map.get_mut(name) {
            Some(prev) => {
                let checkbox = e.is("input") && e.value_lower("type").as_deref() == Some("checkbox");
                if allow_checkbox_default && prev.hidden_default && checkbox {
                    prev.hidden_default = false;
                    continue;
                }
                out.push(finding(a.value_start, a.value_end, format!("Another control in this form is already named `{name}`")));
            }
            None => {
                let hidden = e.is("input") && e.value_lower("type").as_deref() == Some("hidden");
                map.insert(name.to_owned(), Seen { array: false, hidden_default: hidden });
            }
        }
    }
    for i in shared_ctl {
        let e = &dom.elems[i];
        let Some(a) = e.attr("name").filter(|a| !a.value_or_empty().is_empty() && !a.is_dynamic()) else { continue };
        let name = a.value_or_empty().to_owned();
        let group = group_of(i);
        let ty = control_type(e);
        let clash_unique = unique.get(&group).is_some_and(|m| m.contains_key(&name));
        let s = shared_seen.entry(group).or_default();
        if clash_unique || s.get(&name).is_some_and(|t| *t != ty) {
            out.push(finding(a.value_start, a.value_end, format!("Another control in this form is already named `{name}`")));
        }
        s.insert(name, ty);
    }
});

html_rule!(LongTitle, "html/long-title", |dom, meta, ctx, out| {
    let max = ctx.options.get("maxlength").and_then(|v| v.as_u64()).unwrap_or(70) as usize;
    for i in dom.by_tag("title") {
        if dom.elems[i].in_foreign {
            continue;
        }
        let n = dom.text_content(i).chars().count();
        if n > max {
            out.push(dom.at_elem(i, format!("The title is {n} characters long; search results cut it after {max}")));
        }
    }
});

html_rule!(ScriptType, "html/script-type", |dom, meta, ctx, out| {
    const JS: &[&str] = &["", "application/ecmascript", "application/javascript", "text/ecmascript", "text/javascript"];
    for i in dom.by_tag("script") {
        let Some(a) = dom.elems[i].attr("type").filter(|a| !a.is_dynamic()) else { continue };
        let v = a.value_or_empty().split(';').next().unwrap_or("").trim().to_ascii_lowercase();
        if JS.contains(&v.as_str()) {
            out.push(finding_with(a.name_start, a.name_end, "JavaScript is the default: leave `type` out", vec![remove_attr(ctx.source, a)]));
        }
    }
});

html_rule!(ValidId, "html/valid-id", |dom, meta, ctx, out| {
    let relaxed = ctx.options.get("relaxed").and_then(|v| v.as_bool()).unwrap_or(false);
    let bad_char = Regex::new(r"[^\p{L}\p{N}_-]").expect("valid regex");
    for e in &dom.elems {
        let Some(a) = e.attr("id").filter(|a| a.value.is_some() && !a.is_dynamic()) else { continue };
        let v = a.value_or_empty();
        let at = (a.value_start, a.value_end);
        if v.is_empty() {
            out.push(finding(a.start, a.end, "An id can't be empty"));
        } else if v.chars().any(char::is_whitespace) {
            out.push(finding(at.0, at.1, format!("The id `{v}` contains whitespace")));
        } else if relaxed {
            continue;
        } else if !v.chars().next().is_some_and(char::is_alphabetic) {
            out.push(finding(at.0, at.1, format!("The id `{v}` should start with a letter")));
        } else if bad_char.is_match(v) {
            out.push(finding(at.0, at.1, format!("The id `{v}` should use only letters, digits, `-` and `_`")));
        }
    }
});

html_rule!(PreferButton, "html/prefer-button", |dom, meta, ctx, out| {
    for i in dom.by_tag("input") {
        let Some(a) = dom.elems[i].attr("type").filter(|a| !a.is_dynamic()) else { continue };
        let ty = a.value_or_empty().to_ascii_lowercase();
        if matches!(ty.as_str(), "button" | "submit" | "reset" | "image") {
            out.push(finding(a.value_start, a.value_end, format!("Use a <button> instead of <input type=\"{ty}\">")));
        }
    }
});

html_rule!(PreferTbody, "html/prefer-tbody", |dom, meta, ctx, out| {
    for t in dom.by_tag("table") {
        let kids: Vec<usize> = dom.child_elems(t).collect();
        if kids.iter().any(|&k| dom.elems[k].is("tbody")) {
            continue;
        }
        let rows: Vec<usize> = kids.iter().copied().filter(|&k| dom.elems[k].is("tr")).collect();
        let Some(&first) = rows.first() else { continue };
        let last = *rows.last().unwrap_or(&first);
        let span = kids.iter().position(|&k| k == first).zip(kids.iter().position(|&k| k == last));
        let contiguous = span.is_some_and(|(a, b)| kids[a..=b].iter().all(|&k| dom.elems[k].is("tr")));
        let edits = if contiguous {
            vec![TextEdit::new(dom.elems[first].start, dom.elems[first].start, "<tbody>"), TextEdit::new(dom.elems[last].end, dom.elems[last].end, "</tbody>")]
        } else {
            Vec::new()
        };
        let (s, e) = dom.tag_range(first);
        out.push(finding_with(s, e, "Wrap the table's rows in <tbody>", edits));
    }
});

html_rule!(TelNonBreaking, "html/tel-non-breaking", |dom, meta, ctx, out| {
    let ignore_classes = opt_strings(ctx, "ignoreClasses").unwrap_or_default();
    let ignore_style = ctx.options.get("ignoreStyle").and_then(|v| v.as_bool()).unwrap_or(true);
    for a in dom.by_tag("a") {
        let e = &dom.elems[a];
        if !e.attr("href").is_some_and(|h| h.value_or_empty().starts_with("tel:")) {
            continue;
        }
        if e.attr("class").is_some_and(|c| c.value_or_empty().split_ascii_whitespace().any(|x| ignore_classes.iter().any(|i| i == x))) {
            continue;
        }
        let nowrap = e.attr("style").is_some_and(|s| {
            s.value_or_empty().split(';').any(|d| {
                d.split_once(':').is_some_and(|(k, v)| k.trim().eq_ignore_ascii_case("white-space") && matches!(v.trim(), "nowrap" | "pre"))
            })
        });
        if ignore_style && nowrap {
            continue;
        }
        let mut texts = Vec::new();
        collect_plain(dom, a, &mut texts);
        for (start, end) in texts {
            for (k, c) in ctx.source[start..end].char_indices() {
                let (with, what) = match c {
                    ' ' => ("&nbsp;", "a non-breaking space"),
                    '-' => ("&#8209;", "a non-breaking hyphen"),
                    _ => continue,
                };
                let at = start + k;
                out.push(finding_with(at, at + 1, format!("A phone number can wrap at `{c}`; use {what} (`{with}`)"), vec![TextEdit::new(at, at + 1, with)]));
            }
        }
    }
});

fn collect_plain(dom: &Dom, i: usize, out: &mut Vec<(usize, usize)>) {
    for c in &dom.elems[i].children {
        match c {
            Child::Text(t) if dom.texts[*t].kind == TextKind::Plain => out.push((dom.texts[*t].start, dom.texts[*t].end)),
            Child::Elem(j) => collect_plain(dom, *j, out),
            _ => {}
        }
    }
}

html_rule!(NoConditionalComment, "html/no-conditional-comment", |dom, meta, ctx, out| {
    for &(s, t) in &dom.comments {
        let body = ctx.source[s..t].trim_start_matches("<!--").trim_start();
        if body.starts_with("[if") || body.starts_with("<![endif]") || ctx.source[s..t].starts_with("<![") {
            out.push(finding(s, t, "Conditional comments only ever worked in old Internet Explorer"));
        }
    }
});

html_rule!(NoMultipleMain, "html/no-multiple-main", |dom, meta, ctx, out| {
    for i in dom.by_tag("main").filter(|&i| !dom.elems[i].has("hidden")).skip(1) {
        out.push(dom.at_elem(i, "Only one visible <main> per document; hide the others with `hidden`"));
    }
});

html_rule!(ValidFor, "html/valid-for", |dom, meta, ctx, out| {
    for l in dom.by_tag("label") {
        let Some(f) = dom.elems[l].attr("for").filter(|a| !a.is_dynamic() && !a.value_or_empty().is_empty()) else { continue };
        let Some(target) = dom.by_id(f.value_or_empty()).next() else { continue };
        if meta.elem(&dom.elems[target].tag).is_none() || flag(dom, meta, target, "labelable") {
            continue;
        }
        out.push(finding(f.value_start, f.value_end, format!("`for` points at {}, which is not a form control a label can name", lt(&dom.elems[target].tag))));
    }
});

html_rule!(TargetBlankRel, "html/target-blank-rel", |dom, meta, ctx, out| {
    for i in dom.all() {
        let e = &dom.elems[i];
        if !matches!(e.tag.as_str(), "a" | "area" | "form") {
            continue;
        }
        let Some(target) = e.attr("target").filter(|a| a.value_or_empty().trim().eq_ignore_ascii_case("_blank")) else { continue };
        if e.is("form") && !e.has("action") || !e.is("form") && !e.has("href") {
            continue;
        }
        let edit = match e.attr("rel") {
            Some(rel) if rel.is_dynamic() => continue,
            Some(rel) => {
                let v = rel.value_or_empty().to_ascii_lowercase();
                if v.split_ascii_whitespace().any(|t| t == "noopener" || t == "noreferrer") {
                    continue;
                }
                match (&rel.value, rel.quoted) {
                    (Some(v), true) if !v.is_empty() => TextEdit::new(rel.value_end, rel.value_end, " noopener"),
                    _ => TextEdit::new(rel.start, rel.end, "rel=\"noopener\""),
                }
            }
            None => TextEdit::new(target.end, target.end, " rel=\"noopener\""),
        };
        out.push(finding_with(
            target.start,
            target.end,
            "`target=\"_blank\"` without `rel=\"noopener\"` lets the opened page control this one",
            vec![edit],
        ));
    }
});

html_rule!(NoInlineStyle, "html/no-inline-style", |dom, meta, ctx, out| {
    let allowed = opt_strings(ctx, "allowedProperties").unwrap_or_else(|| vec!["display".into()]);
    let allow_vars = ctx.options.get("allowVariables").and_then(|v| v.as_bool()).unwrap_or(true);
    for e in &dom.elems {
        let Some(a) = e.attr("style") else { continue };
        let props: Vec<String> =
            a.value_or_empty().split(';').filter_map(|d| d.split_once(':')).map(|(k, _)| k.trim().to_ascii_lowercase()).filter(|k| !k.is_empty()).collect();
        let all_allowed = !props.is_empty() && props.iter().all(|p| (allow_vars && p.starts_with("--")) || allowed.contains(p));
        if all_allowed {
            continue;
        }
        out.push(finding(a.start, a.end, "Move this inline style to a stylesheet class"));
    }
});

html_rule!(NoInlineScript, "html/no-inline-script", |dom, meta, ctx, out| {
    let js_url = Regex::new(r"(?i)^\s*javascript:").expect("valid regex");
    for e in &dom.elems {
        for a in &e.attrs {
            let handler = a.name.len() > 2 && a.name.starts_with("on") && a.name[2..].chars().all(|c| c.is_ascii_lowercase());
            let url = matches!(a.name.as_str(), "href" | "src" | "action" | "formaction") && js_url.is_match(a.value_or_empty());
            if handler {
                out.push(finding(a.start, a.end, format!("Inline event handler `{}`; attach the listener from a script file", a.name)));
            } else if url {
                out.push(finding(a.start, a.end, "A `javascript:` URL is inline script; attach a listener from a script file"));
            }
        }
    }
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html_dom::tests::run;

    #[test]
    fn raw_characters() {
        let f = run(&NoRawCharacters, "<p>a & b</p>");
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(run(&NoRawCharacters, "<p>a &amp; b &copy; &#169;</p>").is_empty());
        assert_eq!(run(&NoRawCharacters, "<a href=a?b=1&c=2>x</a>").len(), 3);
    }

    #[test]
    fn char_refs() {
        assert!(run(&UnrecognizedCharRef, "<p>&amp; &copy; &#169;</p>").is_empty());
        assert_eq!(run(&UnrecognizedCharRef, "<p>&nosuch;</p>").len(), 1);
        assert_eq!(run(&UnrecognizedCharRef, "<p>&copy 2024</p>").len(), 1);
        assert!(run(&UnrecognizedCharRef, r#"<a href="/x?a=1&copy=2">x</a>"#).is_empty());
    }

    #[test]
    fn duplicate_ids() {
        assert_eq!(run(&NoDuplicateId, r#"<p id="a"></p><p id="a"></p>"#).len(), 1);
        assert!(run(&NoDuplicateId, r#"<p id="a"></p><template><p id="a"></p></template>"#).is_empty());
    }

    #[test]
    fn form_names() {
        assert_eq!(run(&FormDupName, r#"<form><input name="a"><input name="a"></form>"#).len(), 1);
        assert!(run(&FormDupName, r#"<form><input type="radio" name="a"><input type="radio" name="a"></form>"#).is_empty());
        assert!(run(&FormDupName, r#"<form><input type="hidden" name="a" value="0"><input type="checkbox" name="a"></form>"#).is_empty());
    }
}
