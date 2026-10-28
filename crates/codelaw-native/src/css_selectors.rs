//! CSS rules about selectors.

use crate::css_syntax::{
    Complex, Kind, SKind, Sheet, Simple, all_simples, arg_selectors, complex, css_rule, find_of, finding, fixed, line, matches_any, max_specificity, non_standard, normalize_selector,
    opt_list, opt_str, selector_list, set, skip_comment, text, unprefixed, vendor_prefix,
};
use codelaw_core::TextEdit;
use std::collections::{BTreeMap, BTreeSet};

/// The style rules whose selectors are checked: not keyframe blocks, not
/// preprocessor syntax.
fn style_rules(sheet: &Sheet, src: &str) -> Vec<usize> {
    sheet.walk().into_iter().filter(|&i| sheet.nodes[i].kind == Kind::Rule && !sheet.in_keyframes(i, src) && !non_standard(text(src, sheet.nodes[i].name))).collect()
}

/// The nearest style rule enclosing a node.
fn parent_rule(sheet: &Sheet, i: usize) -> Option<usize> {
    sheet.ancestors(i).find(|&a| sheet.nodes[a].kind == Kind::Rule)
}

/// A rule's selectors with nesting resolved against its parents, as
/// normalised text, paired with the complex selector each comes from.
fn resolved(sheet: &Sheet, i: usize, src: &str) -> Vec<(String, Complex)> {
    let own = sheet.selectors(i, src);
    let parents: Option<Vec<String>> = parent_rule(sheet, i).map(|p| resolved(sheet, p, src).into_iter().map(|x| x.0).collect());
    let mut out = Vec::new();
    for c in own.iter().cloned() {
        let t = normalize_selector(text(src, (c.start, c.end)));
        match &parents {
            None => out.push((t, c)),
            Some(ps) => {
                for p in ps {
                    let r = if t.contains('&') {
                        t.replace('&', p)
                    } else if t.starts_with(['>', '+', '~']) {
                        format!("{p}{t}")
                    } else {
                        format!("{p} {t}")
                    };
                    out.push((normalize_selector(&r), c.clone()));
                }
            }
        }
    }
    out
}

css_rule!(NoDuplicateSelector, "css/no-duplicate-selector", |sheet, src, ctx, out| {
    let mut contexts: BTreeMap<Option<usize>, BTreeMap<String, usize>> = BTreeMap::new();
    for i in style_rules(sheet, src) {
        let n = &sheet.nodes[i];
        let list = sheet.selectors(i, src);
        // The same selector twice in one list.
        let mut in_list = BTreeSet::new();
        for c in list.iter() {
            let t = normalize_selector(text(src, (c.start, c.end)));
            if !in_list.insert(t.clone()) {
                out.push(finding(c.start, c.end, format!("`{t}` appears twice in this selector list")));
            }
        }
        let mut all: Vec<String> = resolved(sheet, i, src).into_iter().map(|x| x.0).collect();
        all.sort();
        all.dedup();
        let key = all.join(",");
        let ctx_map = contexts.entry(sheet.at_rule_context(i)).or_default();
        match ctx_map.get(&key) {
            Some(&first) => {
                let sel = text(src, n.name);
                out.push(finding(n.name.0, n.name.1, format!("`{sel}` repeats the selector of the rule on line {}; merge the two", line(src, first))));
            }
            None => {
                ctx_map.insert(key, n.start);
            }
        }
    }
});

/// Whether a block holds declarations, directly or in nested at-rules.
fn has_declaration(sheet: &Sheet, i: usize) -> bool {
    sheet.nodes[i].children.iter().any(|&c| match sheet.nodes[c].kind {
        Kind::Decl => true,
        Kind::AtRule => has_declaration(sheet, c),
        _ => false,
    })
}

/// An earlier selector: its specificity, text and offset.
type Prior = ((u32, u32, u32), String, usize);

const LEGACY_ELEMENTS: &[&str] = &["before", "after", "first-line", "first-letter"];

/// The last compound of a selector without its pseudo-classes, as text.
fn reference(sel: &str, pseudo_elements: &BTreeSet<String>) -> Option<String> {
    let c = complex(sel, 0, sel.len());
    let last = c.compounds.last()?;
    let parts: Vec<&str> = last
        .parts
        .iter()
        .filter(|p| {
            p.kind != SKind::PseudoClass || {
                let name = text(sel, p.name).to_ascii_lowercase();
                LEGACY_ELEMENTS.contains(&name.as_str()) || pseudo_elements.contains(&name)
            }
        })
        .map(|p| &sel[p.start..p.end])
        .collect();
    (!parts.is_empty()).then(|| parts.concat())
}

css_rule!(NoDescendingSpecificity, "css/no-descending-specificity", |sheet, src, ctx, out| {
    let pseudo_elements = set(ctx.data, "css/selectors", "pseudo_elements");
    let within_list = opt_list(ctx, "ignore").iter().any(|x| x == "selectors-within-list");
    let mut contexts: BTreeMap<Option<usize>, BTreeMap<String, Vec<Prior>>> = BTreeMap::new();
    for i in style_rules(sheet, src) {
        if !has_declaration(sheet, i) {
            continue;
        }
        let n = &sheet.nodes[i];
        if within_list && sheet.selectors(i, src).len() > 1 {
            continue;
        }
        let map = contexts.entry(sheet.at_rule_context(i)).or_default();
        for (res, c) in resolved(sheet, i, src) {
            let Some(r) = reference(&res, &pseudo_elements) else { continue };
            let list = selector_list(&res, 0, res.len());
            let spec = max_specificity(&res, &list, (0, 0, 0));
            // Nested selectors are named as they resolve (`&::after` under
            // `.btn` is `.btn::after`), so the two the finding compares are plain.
            let own = normalize_selector(text(src, (c.start, c.end)));
            let shown = if own == res { format!("`{own}`") } else { format!("`{own}` (`{res}`)") };
            let prior = map.entry(r).or_default();
            if let Some((_, sel, at)) = prior.iter().find(|p| p.0 > spec) {
                out.push(finding(c.start, c.end, format!("{shown} comes after `{sel}` (line {}), which is more specific; put the less specific selector first", line(src, *at))));
            }
            prior.push((spec, res, n.start));
        }
    }
});

css_rule!(NestingSelectorNoMissingScopingRoot, "css/nesting-selector-no-missing-scoping-root", |sheet, src, ctx, out| {
    for i in sheet.walk() {
        let n = &sheet.nodes[i];
        let rooted = sheet.ancestors(i).any(|a| sheet.nodes[a].kind == Kind::Rule || (sheet.scss && sheet.nodes[a].kind == Kind::Other) || (sheet.nodes[a].kind == Kind::AtRule && sheet.at_name(a, src) == "scope" && !text(src, sheet.nodes[a].params).is_empty()));
        if rooted {
            continue;
        }
        match n.kind {
            Kind::Rule if !sheet.in_keyframes(i, src) && !non_standard(text(src, n.name)) => {}
            Kind::AtRule if sheet.at_name(i, src) == "scope" => {}
            _ => continue,
        }
        let list = sheet.selectors(i, src);
        let mut simples = Vec::new();
        all_simples(src, &list, &mut simples);
        for x in simples.iter().filter(|x| x.kind == SKind::Nesting) {
            out.push(finding(x.start, x.end, "`&` has no parent rule to stand for here"));
        }
    }
});

/// `a` and `b` of an An+B argument, when it is one.
fn anb(t: &str) -> Option<(i64, i64)> {
    let t: String = t.chars().filter(|c| !c.is_whitespace()).collect::<String>().to_ascii_lowercase();
    match t.as_str() {
        "odd" => return Some((2, 1)),
        "even" => return Some((2, 0)),
        _ => {}
    }
    if let Some(p) = t.find('n') {
        let a = match &t[..p] {
            "" | "+" => 1,
            "-" => -1,
            x => x.parse().ok()?,
        };
        let b = if t[p + 1..].is_empty() { 0 } else { t[p + 1..].parse().ok()? };
        Some((a, b))
    } else {
        Some((0, t.parse().ok()?))
    }
}

css_rule!(SelectorAnbNoUnmatchable, "css/selector-anb-no-unmatchable", |sheet, src, ctx, out| {
    for i in style_rules(sheet, src) {
        let n = &sheet.nodes[i];
        let mut simples = Vec::new();
        all_simples(src, &sheet.selectors(i, src), &mut simples);
        for x in simples {
            let name = text(src, x.name).to_ascii_lowercase();
            if x.kind != SKind::PseudoClass || !matches!(name.as_str(), "nth-child" | "nth-last-child" | "nth-of-type" | "nth-last-of-type" | "nth-col" | "nth-last-col") {
                continue;
            }
            let Some((a, b)) = x.args else { continue };
            let end = find_of(src, a, b).map_or(b, |o| o - 2);
            if anb(&src[a..end]) == Some((0, 0)) {
                out.push(finding(x.start, x.end, format!("`{}` matches no element: positions start at 1", &src[x.start..x.end])));
            }
        }
    }
});

/// The previous pseudo-element in the same compound.
fn prev_element<'a>(c: &'a Complex, x: &Simple) -> Option<&'a Simple> {
    let comp = c.compounds.iter().find(|k| k.start <= x.start && x.end <= k.end)?;
    comp.parts.iter().rfind(|p| p.end <= x.start && p.kind == SKind::PseudoElement)
}

fn each_selector_simple(src: &str, list: &[Complex], f: &mut dyn FnMut(&Complex, &Simple)) {
    for c in list {
        for s in c.simples() {
            f(c, s);
            if let Some((a, b)) = s.args {
                let name = text(src, s.name).to_ascii_lowercase();
                if let Some(list) = arg_selectors(src, s, unprefixed(&name), a, b) {
                    each_selector_simple(src, &list, f);
                }
            }
        }
    }
}

css_rule!(SelectorPseudoClassNoUnknown, "css/selector-pseudo-class-no-unknown", |sheet, src, ctx, out| {
    let classes = set(ctx.data, "css/selectors", "pseudo_classes");
    let elements = set(ctx.data, "css/selectors", "pseudo_elements");
    let page = set(ctx.data, "css/selectors", "page_pseudo_classes");
    let ignore = opt_list(ctx, "ignorePseudoClasses");
    for i in sheet.walk() {
        let n = &sheet.nodes[i];
        let is_page = n.kind == Kind::AtRule && sheet.at_name(i, src) == "page";
        match n.kind {
            Kind::Rule if !sheet.in_keyframes(i, src) && !non_standard(text(src, n.name)) => {}
            Kind::AtRule if is_page => {}
            _ => continue,
        }
        let list = sheet.selectors(i, src);
        each_selector_simple(src, &list, &mut |c, x| {
            if x.kind != SKind::PseudoClass {
                return;
            }
            let raw = text(src, x.name);
            let name = raw.to_ascii_lowercase();
            if name.is_empty() || name.starts_with("--") || LEGACY_ELEMENTS.contains(&name.as_str()) || matches_any(&ignore, raw) {
                return;
            }
            // CSS Modules' `:global` and `:local`.
            if sheet.module && matches!(name.as_str(), "global" | "local") {
                return;
            }
            if is_page {
                if !page.contains(&name) {
                    out.push(finding(x.start, x.name.1, format!("Unknown page pseudo-class `:{raw}`")));
                }
                return;
            }
            let prefixed = vendor_prefix(&name).is_some();
            if elements.contains(&name) && !prefixed {
                out.push(finding(x.start, x.name.1, format!("`:{raw}` is a pseudo-element; write `::{raw}`")));
                return;
            }
            if prefixed || classes.contains(&name) {
                return;
            }
            // `::-webkit-scrollbar-thumb:horizontal` and the like.
            if prev_element(c, x).is_some_and(|p| vendor_prefix(text(src, p.name)).is_some()) {
                return;
            }
            out.push(finding(x.start, x.name.1, format!("Unknown pseudo-class `:{raw}`")));
        });
    }
});

css_rule!(SelectorPseudoElementNoUnknown, "css/selector-pseudo-element-no-unknown", |sheet, src, ctx, out| {
    let elements = set(ctx.data, "css/selectors", "pseudo_elements");
    let ignore = opt_list(ctx, "ignorePseudoElements");
    for i in style_rules(sheet, src) {
        let n = &sheet.nodes[i];
        let list = sheet.selectors(i, src);
        each_selector_simple(src, &list, &mut |_, x| {
            let raw = text(src, x.name);
            if x.kind == SKind::PseudoElement && vendor_prefix(raw).is_none() && !elements.contains(&raw.to_ascii_lowercase()) && !matches_any(&ignore, raw) {
                out.push(finding(x.start, x.name.1, format!("Unknown pseudo-element `::{raw}`")));
            }
        });
    }
});

/// A valid custom element name: a lowercase letter first, and a hyphen.
fn custom_element(name: &str) -> bool {
    name.as_bytes().first().is_some_and(|c| c.is_ascii_lowercase()) && name.contains('-') && !name.bytes().any(|c| c.is_ascii_uppercase())
}

css_rule!(SelectorTypeNoUnknown, "css/selector-type-no-unknown", |sheet, src, ctx, out| {
    let mut known = set(ctx.data, "css/type-selectors", "html");
    known.extend(set(ctx.data, "css/type-selectors", "svg"));
    known.extend(set(ctx.data, "css/type-selectors", "mathml"));
    if known.is_empty() {
        return;
    }
    let ignore = opt_list(ctx, "ignore");
    let custom_ok = ignore.iter().any(|x| x == "custom-elements");
    let default_ns = ignore.iter().any(|x| x == "default-namespace");
    let ignore_types = opt_list(ctx, "ignoreTypes");
    for i in style_rules(sheet, src) {
        let n = &sheet.nodes[i];
        let mut simples = Vec::new();
        all_simples(src, &sheet.selectors(i, src), &mut simples);
        for x in simples.iter().filter(|x| x.kind == SKind::Type) {
            let raw = text(src, x.name);
            let (ns, local) = match raw.rfind('|') {
                Some(p) => (Some(&raw[..p]), &raw[p + 1..]),
                None => (None, raw),
            };
            if ns.is_some_and(|ns| !ns.is_empty() && ns != "*") || (default_ns && ns.is_none()) {
                continue;
            }
            let lower = local.to_ascii_lowercase();
            if local.is_empty() || local.contains('\\') || known.contains(&lower) || (custom_ok && custom_element(local)) || matches_any(&ignore_types, local) {
                continue;
            }
            // Percentages and keyword selectors of keyframes nested in rules.
            if local.as_bytes()[0].is_ascii_digit() || lower == "from" || lower == "to" {
                continue;
            }
            out.push(finding(x.start, x.end, format!("Unknown type selector `{local}`")));
        }
    }
});

/// Strings in a range that hold an unescaped line break.
fn strings_with_newline(src: &str, s: usize, e: usize, out: &mut Vec<crate::NativeFinding>) {
    let b = src.as_bytes();
    let mut i = s;
    while i < e {
        match b[i] {
            b'/' if b.get(i + 1) == Some(&b'*') => i = skip_comment(src, i),
            q @ (b'"' | b'\'') => {
                let mut j = i + 1;
                while j < e && b[j] != q {
                    if b[j] == b'\\' {
                        j += 2;
                        continue;
                    }
                    if b[j] == b'\n' || b[j] == b'\r' {
                        let line_end = src[j..].find('\n').map_or(j, |o| j + o);
                        out.push(finding(i, line_end.min(e), "Line break inside a string; escape it as `\\A` or end the line with `\\`"));
                        break;
                    }
                    j += 1;
                }
                i = j + 1;
            }
            _ => i += 1,
        }
    }
}

css_rule!(StringNoNewline, "css/string-no-newline", |sheet, src, ctx, out| {
    let ignore = opt_list(ctx, "ignore");
    for i in sheet.walk() {
        let n = &sheet.nodes[i];
        match n.kind {
            Kind::Rule => strings_with_newline(src, n.name.0, n.name.1, out),
            Kind::AtRule if !ignore.iter().any(|x| x == "at-rule-preludes") => strings_with_newline(src, n.params.0, n.params.1, out),
            Kind::Decl if !ignore.iter().any(|x| x == "declaration-values") => strings_with_newline(src, n.params.0, n.params.1, out),
            _ => {}
        }
    }
});

css_rule!(BlockNoRedundantNestedStyleRules, "css/block-no-redundant-nested-style-rules", |sheet, src, ctx, out| {
    for i in style_rules(sheet, src) {
        let n = &sheet.nodes[i];
        if text(src, n.name) != "&" || parent_rule(sheet, i).is_none() {
            continue;
        }
        let Some((open, close)) = n.block else { continue };
        let inner = src[open + 1..close.min(src.len())].trim();
        out.push(fixed(n.start, n.end, "`& { … }` adds nothing; put its contents straight in the parent block", vec![TextEdit::new(n.start, n.end, inner)]));
    }
});

fn pattern_rule(sheet: &Sheet, src: &str, ctx: &crate::NativeContext<'_>, kind: SKind, out: &mut Vec<crate::NativeFinding>) {
    let pattern = opt_str(ctx, "pattern", "^([a-z][a-z0-9]*)(-[a-z0-9]+)*$");
    let Ok(re) = regex::Regex::new(pattern) else { return };
    let what = if kind == SKind::Class { "Class" } else { "Id" };
    let sigil = if kind == SKind::Class { "." } else { "#" };
    for i in style_rules(sheet, src) {
        let mut simples = Vec::new();
        all_simples(src, &sheet.selectors(i, src), &mut simples);
        for x in simples.iter().filter(|x| x.kind == kind) {
            let name = text(src, x.name);
            if !name.is_empty() && !re.is_match(name) {
                out.push(finding(x.start, x.end, format!("{what} selector `{sigil}{name}` doesn't match the naming pattern `{pattern}` (kebab-case by default)")));
            }
        }
    }
}

css_rule!(SelectorClassPattern, "css/selector-class-pattern", |sheet, src, ctx, out| {
    pattern_rule(sheet, src, ctx, SKind::Class, out);
});

css_rule!(SelectorIdPattern, "css/selector-id-pattern", |sheet, src, ctx, out| {
    pattern_rule(sheet, src, ctx, SKind::Id, out);
});

css_rule!(SelectorNoVendorPrefix, "css/selector-no-vendor-prefix", |sheet, src, ctx, out| {
    let prefixed = set(ctx.data, "css/prefixed", "selectors");
    let ignore = opt_list(ctx, "ignoreSelectors");
    for i in style_rules(sheet, src) {
        let n = &sheet.nodes[i];
        let list = sheet.selectors(i, src);
        each_selector_simple(src, &list, &mut |_, x| {
            if !matches!(x.kind, SKind::PseudoClass | SKind::PseudoElement) {
                return;
            }
            let full = &src[x.start..x.name.1];
            if !prefixed.contains(&full.to_ascii_lowercase()) || matches_any(&ignore, full) {
                return;
            }
            let name = text(src, x.name);
            let Some(p) = vendor_prefix(name) else { return };
            out.push(fixed(x.start, x.name.1, format!("`{full}` has a vendor prefix; write it without `{p}`"), vec![TextEdit::delete(x.name.0, x.name.0 + p.len())]));
        });
    }
});

css_rule!(SelectorNotNotation, "css/selector-not-notation", |sheet, src, ctx, out| {
    for i in style_rules(sheet, src) {
        let n = &sheet.nodes[i];
        let list = sheet.selectors(i, src);
        each_selector_simple(src, &list, &mut |c, x| {
            let is_not = |s: &Simple| s.kind == SKind::PseudoClass && text(src, s.name).eq_ignore_ascii_case("not") && s.args.is_some();
            if !is_not(x) {
                return;
            }
            let Some(comp) = c.compounds.iter().find(|k| k.start <= x.start && x.end <= k.end) else { return };
            let idx = comp.parts.iter().position(|p| p.start == x.start).unwrap_or(0);
            // Report each run of adjacent `:not()`s once, from its first.
            if idx > 0 && is_not(&comp.parts[idx - 1]) {
                return;
            }
            let first = idx;
            let mut last = idx;
            while comp.parts.get(last + 1).is_some_and(is_not) {
                last += 1;
            }
            if last == first {
                return;
            }
            let args: Vec<&str> = comp.parts[first..=last].iter().filter_map(|p| p.args).map(|a| src[a.0..a.1].trim()).collect();
            let start = comp.parts[first].start;
            let end = comp.parts[last].end;
            out.push(fixed(start, end, format!("Write `{}` as one `:not()` with a selector list", &src[start..end]), vec![TextEdit::new(start, end, format!(":not({})", args.join(", ")))]));
        });
    }
});

css_rule!(SelectorPseudoElementColonNotation, "css/selector-pseudo-element-colon-notation", |sheet, src, ctx, out| {
    for i in style_rules(sheet, src) {
        let n = &sheet.nodes[i];
        let list = sheet.selectors(i, src);
        each_selector_simple(src, &list, &mut |_, x| {
            let name = text(src, x.name);
            if x.kind == SKind::PseudoClass && LEGACY_ELEMENTS.contains(&name.to_ascii_lowercase().as_str()) {
                out.push(fixed(x.start, x.name.1, format!("`:{name}` is a pseudo-element; write `::{name}`"), vec![TextEdit::new(x.start, x.start, ":")]));
            }
        });
    }
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anb_values() {
        assert_eq!(anb("0"), Some((0, 0)));
        assert_eq!(anb("0n+0"), Some((0, 0)));
        assert_eq!(anb("-0n"), Some((0, 0)));
        assert_eq!(anb("2n + 1"), Some((2, 1)));
        assert_eq!(anb("odd"), Some((2, 1)));
        assert_eq!(anb("-n+3"), Some((-1, 3)));
    }

    #[test]
    fn references() {
        let pe = BTreeSet::new();
        assert_eq!(reference("#c a:hover", &pe).as_deref(), Some("a"));
        assert_eq!(reference("a::before", &pe).as_deref(), Some("a::before"));
        assert_eq!(reference(":hover", &pe), None);
    }
}
