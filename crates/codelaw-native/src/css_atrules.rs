//! CSS rules about at-rules: their names, preludes and descriptors,
//! `@import`, `@keyframes` and the names that at-rules define.

use crate::css_syntax::{Kind, Sheet, VKind, css_rule, finding, fixed, matches_any, opt_list, opt_str, set, significant, split_commas, table, text, values};
use crate::util::whole_lines;
use codelaw_core::TextEdit;
use std::collections::{BTreeMap, BTreeSet};

fn at_rules(sheet: &Sheet) -> impl Iterator<Item = usize> + '_ {
    sheet.walk().into_iter().filter(|&i| sheet.nodes[i].kind == Kind::AtRule)
}

/// The `@name` of an at-rule as written.
fn at_text<'s>(sheet: &Sheet, i: usize, src: &'s str) -> &'s str {
    let n = &sheet.nodes[i];
    &src[n.start..n.name.1]
}

css_rule!(AtRuleDescriptorNoUnknown, "css/at-rule-descriptor-no-unknown", |sheet, src, ctx, out| {
    let Some(desc) = table(ctx.data, "css/descriptors").and_then(|t| t.get("descriptors")).and_then(|v| v.as_object()) else { return };
    let page_props = set(ctx.data, "css/properties-groups", "page_context");
    let margin_props = set(ctx.data, "css/properties-groups", "margin_context");
    let margins = set(ctx.data, "css/at-rules", "page_margin");
    for i in sheet.decls() {
        let n = &sheet.nodes[i];
        let Some(p) = n.parent.filter(|&p| sheet.nodes[p].kind == Kind::AtRule) else { continue };
        let mut at = sheet.at_name(p, src);
        let mut allowed: BTreeSet<String> = BTreeSet::new();
        if margins.contains(&at) && sheet.nodes[p].parent.is_some_and(|g| sheet.nodes[g].kind == Kind::AtRule && sheet.at_name(g, src) == "page") {
            allowed.extend(margin_props.iter().cloned());
            at = "page".into();
        } else if at == "page" {
            allowed.extend(page_props.iter().cloned());
        }
        let Some(list) = desc.get(&at).and_then(|v| v.as_array()) else { continue };
        allowed.extend(list.iter().filter_map(|x| x.as_str().map(str::to_owned)));
        let name = text(src, n.name);
        let lower = name.to_ascii_lowercase();
        if lower.starts_with("--") || !crate::css_syntax::standard_property(name) || allowed.contains(&lower) {
            continue;
        }
        out.push(finding(n.name.0, n.name.1, format!("`{name}` is not a descriptor of `@{at}`")));
    }
});

/// `<syntax>` strings of `@property`: problems, if any.
fn syntax_problem(s: &str) -> Option<String> {
    const TYPES: &[&str] = &[
        "angle", "color", "custom-ident", "image", "integer", "length", "length-percentage", "number", "percentage", "resolution", "string", "time", "transform-function",
        "transform-list", "url",
    ];
    let s = s.trim();
    if s == "*" {
        return None;
    }
    if s.is_empty() {
        return Some("the syntax is empty".into());
    }
    for comp in s.split('|') {
        let c = comp.trim();
        let c = c.strip_suffix('+').or_else(|| c.strip_suffix('#')).unwrap_or(c);
        if c.is_empty() {
            return Some("a `|` has nothing on one side".into());
        }
        if let Some(inner) = c.strip_prefix('<') {
            let Some(name) = inner.strip_suffix('>') else { return Some(format!("`{c}` is missing its `>`")) };
            if !TYPES.contains(&name) {
                return Some(format!("`<{name}>` is not a supported type"));
            }
            if name == "transform-list" && comp.trim() != c {
                return Some("`<transform-list>` takes no multiplier".into());
            }
        } else if !c.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_') || c.as_bytes()[0].is_ascii_digit() {
            return Some(format!("`{c}` is neither a `<type>` nor a keyword"));
        } else if ["inherit", "initial", "unset", "revert", "revert-layer", "default"].contains(&c.to_ascii_lowercase().as_str()) {
            return Some(format!("`{c}` can't be a keyword of a syntax"));
        }
    }
    None
}

css_rule!(AtRuleDescriptorValueNoUnknown, "css/at-rule-descriptor-value-no-unknown", |sheet, src, ctx, out| {
    let Some(keywords) = table(ctx.data, "css/descriptors").and_then(|t| t.get("keyword_values")).and_then(|v| v.as_object()) else { return };
    for i in sheet.decls() {
        let n = &sheet.nodes[i];
        let Some(p) = n.parent.filter(|&p| sheet.nodes[p].kind == Kind::AtRule) else { continue };
        let at = sheet.at_name(p, src);
        let name = text(src, n.name).to_ascii_lowercase();
        let v = text(src, n.params);
        if v.contains("var(") || crate::css_syntax::non_standard_value(v) {
            continue;
        }
        if at == "property" && name == "syntax" {
            if !v.starts_with(['"', '\'']) {
                out.push(finding(n.params.0, n.params.1, "The `syntax` of `@property` must be a string"));
            }
            continue;
        }
        let Some(allowed) = keywords.get(&format!("{at}/{name}")).and_then(|x| x.as_array()) else { continue };
        let vals = values(src, n.params.0, n.params.1);
        let sig = significant(&vals);
        // `system: fixed 3`, `font-style: oblique 10deg`: the keyword comes first.
        let Some(first) = sig.first() else { continue };
        if first.kind != VKind::Word || (at == "font-palette-values" && first.text(src).bytes().all(|b| b.is_ascii_digit())) {
            continue;
        }
        let kw = first.text(src).to_ascii_lowercase();
        if !allowed.iter().any(|a| a.as_str() == Some(kw.as_str())) {
            out.push(finding(first.start, first.end, format!("`{}` is not a value of `{name}` in `@{at}`", first.text(src))));
        }
    }
});

css_rule!(SyntaxStringNoInvalid, "css/syntax-string-no-invalid", |sheet, src, ctx, out| {
    for i in sheet.decls() {
        let n = &sheet.nodes[i];
        let Some(p) = n.parent else { continue };
        if sheet.nodes[p].kind != Kind::AtRule || sheet.at_name(p, src) != "property" || !text(src, n.name).eq_ignore_ascii_case("syntax") {
            continue;
        }
        let v = text(src, n.params);
        if !v.starts_with(['"', '\'']) || v.len() < 2 {
            continue;
        }
        if let Some(why) = syntax_problem(&v[1..v.len() - 1]) {
            out.push(finding(n.params.0, n.params.1, format!("Invalid syntax string: {why}")));
        }
    }
});

css_rule!(AtRuleNoDeprecated, "css/at-rule-no-deprecated", |sheet, src, ctx, out| {
    let deprecated = set(ctx.data, "css/at-rules", "deprecated");
    let ignore = opt_list(ctx, "ignoreAtRules");
    for i in at_rules(sheet) {
        let n = &sheet.nodes[i];
        let raw = text(src, n.name);
        let lower = raw.to_ascii_lowercase();
        if !deprecated.contains(&lower) || matches_any(&ignore, raw) {
            continue;
        }
        let at = at_text(sheet, i, src);
        if lower == "nest" {
            out.push(fixed(n.start, n.name.1, "`@nest` is deprecated; write the nested selector on its own", vec![TextEdit::delete(n.start, n.params.0)]));
        } else {
            out.push(finding(n.start, n.name.1, format!("`{at}` is deprecated")));
        }
    }
});

css_rule!(AtRuleNoUnknown, "css/at-rule-no-unknown", |sheet, src, ctx, out| {
    let known = set(ctx.data, "css/at-rules", "known");
    if known.is_empty() {
        return;
    }
    let ignore = opt_list(ctx, "ignoreAtRules");
    for i in at_rules(sheet) {
        let n = &sheet.nodes[i];
        let raw = text(src, n.name);
        // `@charset` is an encoding marker rather than an at-rule (stylelint skips it too).
        if raw.is_empty() || raw.eq_ignore_ascii_case("charset") || known.contains(&sheet.at_name(i, src)) || matches_any(&ignore, raw) {
            continue;
        }
        out.push(finding(n.start, n.name.1, format!("Unknown at-rule `@{raw}`")));
    }
});

fn is_ident(t: &str) -> bool {
    let b = t.as_bytes();
    !b.is_empty() && (b[0].is_ascii_alphabetic() || b[0] == b'-' || b[0] == b'_' || b[0] >= 0x80) && b.iter().all(|&c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_' || c >= 0x80)
}

fn dashed(t: &str) -> bool {
    t.starts_with("--") && t.len() > 2 && is_ident(t)
}

const WIDE: &[&str] = &["initial", "inherit", "unset", "revert", "revert-layer", "default"];

/// What a prelude must look like, when it doesn't.
fn prelude_problem(at: &str, p: &str) -> Option<&'static str> {
    let lower = p.to_ascii_lowercase();
    match at {
        "property" | "font-palette-values" | "position-try" => (!dashed(p)).then_some("a dashed identifier such as `--name`"),
        "counter-style" => (!is_ident(p) || WIDE.contains(&lower.as_str()) || matches!(lower.as_str(), "none" | "decimal" | "disc" | "square" | "circle" | "disclosure-open" | "disclosure-closed"))
            .then_some("one name that is not a CSS-wide keyword, `none`, or a predefined style that can't be redefined"),
        "keyframes" => {
            let ok = (p.starts_with(['"', '\'']) && p.len() >= 2) || (is_ident(p) && !WIDE.contains(&lower.as_str()) && lower != "none");
            (!ok).then_some("one name: an identifier other than `none` or a CSS-wide keyword, or a string")
        }
        "layer" => {
            let ok = p.is_empty() || p.split(',').all(|n| n.trim().split('.').all(|s| is_ident(s) && !WIDE.contains(&s.to_ascii_lowercase().as_str())));
            (!ok).then_some("layer names separated by commas, each made of dot-separated identifiers")
        }
        "font-face" | "starting-style" | "view-transition" => (!p.is_empty()).then_some("nothing"),
        "charset" => (!(p.starts_with('"') && p.ends_with('"') && p.len() >= 2)).then_some("one double-quoted encoding name"),
        "import" => (!(p.starts_with(['"', '\'']) || lower.starts_with("url(") || lower.starts_with("src("))).then_some("a URL: a string or `url()`"),
        "namespace" => {
            let parts: Vec<&str> = p.split_whitespace().collect();
            let url = |s: &str| s.starts_with(['"', '\'']) || s.to_ascii_lowercase().starts_with("url(");
            let ok = match parts.as_slice() {
                [u] => url(u),
                [n, u] => is_ident(n) && url(u),
                _ => p.to_ascii_lowercase().contains("url(") && parts.len() <= 3,
            };
            (!ok).then_some("an optional prefix and a URL")
        }
        _ => None,
    }
}

css_rule!(AtRulePreludeNoInvalid, "css/at-rule-prelude-no-invalid", |sheet, src, ctx, out| {
    let ignore = opt_list(ctx, "ignoreAtRules");
    for i in at_rules(sheet) {
        let n = &sheet.nodes[i];
        let raw = text(src, n.name);
        if matches_any(&ignore, raw) {
            continue;
        }
        let at = raw.to_ascii_lowercase();
        let at = crate::css_syntax::unprefixed(&at);
        let p = text(src, n.params);
        if crate::css_syntax::non_standard_value(p) {
            continue;
        }
        if let Some(want) = prelude_problem(at, p) {
            let (s, e) = if p.is_empty() { (n.start, n.name.1) } else { n.params };
            out.push(finding(s, e, format!("Invalid prelude for `@{raw}`: expected {want}")));
        }
    }
});

/// An `@import`'s URL and the rest of its prelude, normalised.
fn import_key(src: &str, r: (usize, usize)) -> Option<(String, String)> {
    let vals = values(src, r.0, r.1);
    let sig = significant(&vals);
    let first = sig.first()?;
    let url = match first.kind {
        VKind::Str => {
            let t = first.text(src);
            t[1..t.len().saturating_sub(1).max(1)].to_owned()
        }
        VKind::Func if first.name(src).eq_ignore_ascii_case("url") => {
            let t = text(src, first.inner).trim();
            t.trim_matches(['"', '\'']).to_owned()
        }
        _ => return None,
    };
    let rest: String = src[first.end..r.1].chars().filter(|c| !c.is_whitespace()).collect::<String>().to_ascii_lowercase();
    Some((url, rest))
}

css_rule!(NoDuplicateAtImportRules, "css/no-duplicate-at-import-rules", |sheet, src, ctx, out| {
    let mut seen: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for i in at_rules(sheet) {
        if sheet.at_name(i, src) != "import" {
            continue;
        }
        let n = &sheet.nodes[i];
        let Some((url, rest)) = import_key(src, n.params) else { continue };
        let conditions: Vec<String> = if rest.is_empty() { Vec::new() } else { rest.split(',').map(str::to_owned).collect() };
        let prior = seen.get(&url);
        let dup = match prior {
            None => false,
            Some(p) if conditions.is_empty() => p.is_empty() || p.iter().any(|c| c.is_empty()),
            Some(p) => conditions.iter().any(|c| p.contains(c)),
        };
        if dup {
            let end = n.semicolon.map_or(n.end, |s| s + 1);
            let (s, e) = whole_lines(src, n.start, end);
            out.push(fixed(n.start, n.end, format!("`{url}` is already imported above"), vec![TextEdit::delete(s, e)]));
            continue;
        }
        let entry = seen.entry(url).or_default();
        if conditions.is_empty() {
            entry.push(String::new());
        } else {
            entry.extend(conditions);
        }
    }
});

css_rule!(NoInvalidPositionAtImportRule, "css/no-invalid-position-at-import-rule", |sheet, src, ctx, out| {
    let ignore = opt_list(ctx, "ignoreAtRules");
    let mut late = false;
    for i in sheet.walk() {
        let n = &sheet.nodes[i];
        match n.kind {
            Kind::AtRule => {
                let name = sheet.at_name(i, src);
                if name == "import" {
                    if late || n.parent.is_some() {
                        out.push(finding(n.start, n.end, "`@import` must come before every other rule except `@charset` and `@layer` statements; browsers ignore it here"));
                    }
                } else if name != "charset" && !(name == "layer" && n.block.is_none()) && !matches_any(&ignore, text(src, n.name)) {
                    late = true;
                }
            }
            Kind::Rule => late = true,
            _ => {}
        }
    }
});

css_rule!(AtRuleNoVendorPrefix, "css/at-rule-no-vendor-prefix", |sheet, src, ctx, out| {
    let prefixed = set(ctx.data, "css/prefixed", "at_rules");
    let ignore = opt_list(ctx, "ignoreAtRules");
    for i in at_rules(sheet) {
        let n = &sheet.nodes[i];
        let raw = text(src, n.name);
        if !raw.starts_with('-') || !prefixed.contains(&raw.to_ascii_lowercase()) || matches_any(&ignore, raw) {
            continue;
        }
        let base = crate::css_syntax::unprefixed(raw);
        out.push(fixed(n.start, n.name.1, format!("`@{raw}` has a vendor prefix; write `@{base}`"), vec![TextEdit::new(n.name.0, n.name.1, base)]));
    }
});

css_rule!(ImportNotation, "css/import-notation", |sheet, src, ctx, out| {
    for i in at_rules(sheet) {
        if sheet.at_name(i, src) != "import" {
            continue;
        }
        let r = sheet.nodes[i].params;
        let vals = values(src, r.0, r.1);
        if let Some(first) = significant(&vals).first()
            && first.kind == VKind::Str
        {
            let t = first.text(src);
            out.push(fixed(first.start, first.end, format!("Write the import URL as `url({t})`"), vec![TextEdit::new(first.start, first.end, format!("url({t})"))]));
        }
    }
});

/// The keyframe selectors of each `@keyframes`: (rule, range, lowercased text).
fn keyframe_selectors(sheet: &Sheet, src: &str) -> Vec<Vec<((usize, usize), String)>> {
    at_rules(sheet)
        .filter(|&i| sheet.at_name(i, src) == "keyframes")
        .map(|i| {
            sheet.nodes[i]
                .children
                .iter()
                .filter(|&&c| sheet.nodes[c].kind == Kind::Rule)
                .flat_map(|&c| split_commas(src, sheet.nodes[c].name.0, sheet.nodes[c].name.1))
                .map(|r| (r, text(src, r).to_ascii_lowercase()))
                .collect()
        })
        .collect()
}

css_rule!(KeyframeBlockNoDuplicateSelectors, "css/keyframe-block-no-duplicate-selectors", |sheet, src, ctx, out| {
    for block in keyframe_selectors(sheet, src) {
        let mut seen = BTreeSet::new();
        for (r, t) in block {
            if !seen.insert(t.clone()) {
                out.push(finding(r.0, r.1, format!("Keyframe `{}` is declared twice in this `@keyframes`", text(src, r))));
            }
        }
    }
});

css_rule!(KeyframeDeclarationNoImportant, "css/keyframe-declaration-no-important", |sheet, src, ctx, out| {
    for i in sheet.decls() {
        let n = &sheet.nodes[i];
        let in_kf = n.parent.is_some_and(|p| sheet.nodes[p].kind == Kind::Rule && sheet.in_keyframes(p, src));
        if let (true, Some((s, e))) = (in_kf, n.important) {
            let v_end = n.params.1;
            out.push(fixed(s, e, "Browsers ignore a declaration marked `!important` inside a keyframe", vec![TextEdit::delete(v_end, e)]));
        }
    }
});

css_rule!(KeyframeSelectorNotation, "css/keyframe-selector-notation", |sheet, src, ctx, out| {
    for block in keyframe_selectors(sheet, src) {
        if block.iter().all(|(_, t)| t == "from" || t == "to") {
            continue;
        }
        for (r, t) in block {
            let new = match t.as_str() {
                "from" => "0%",
                "to" => "100%",
                _ => continue,
            };
            out.push(fixed(r.0, r.1, format!("Write `{}` as `{new}`, like the other keyframes of this block", text(src, r)), vec![TextEdit::new(r.0, r.1, new)]));
        }
    }
});

fn kebab<'a>(ctx: &'a crate::NativeContext<'_>, default: &'a str) -> Option<(regex::Regex, &'a str)> {
    let p = opt_str(ctx, "pattern", default);
    regex::Regex::new(p).ok().map(|r| (r, p))
}

const KEBAB: &str = "^([a-z][a-z0-9]*)(-[a-z0-9]+)*$";

css_rule!(KeyframesNamePattern, "css/keyframes-name-pattern", |sheet, src, ctx, out| {
    let Some((re, pattern)) = kebab(ctx, KEBAB) else { return };
    for i in at_rules(sheet) {
        if sheet.at_name(i, src) != "keyframes" {
            continue;
        }
        let r = sheet.nodes[i].params;
        let name = text(src, r).trim_matches(['"', '\'']);
        if !name.is_empty() && !re.is_match(name) {
            out.push(finding(r.0, r.1, format!("Keyframes name `{name}` doesn't match the naming pattern `{pattern}` (kebab-case by default)")));
        }
    }
});

css_rule!(LayerNamePattern, "css/layer-name-pattern", |sheet, src, ctx, out| {
    let Some((re, pattern)) = kebab(ctx, "^([a-z][a-z0-9]*)([.-][a-z0-9]+)*$") else { return };
    let check = |s: usize, e: usize, out: &mut Vec<crate::NativeFinding>| {
        for r in split_commas(src, s, e) {
            let name = text(src, r);
            if !name.is_empty() && !re.is_match(name) {
                out.push(finding(r.0, r.1, format!("Layer name `{name}` doesn't match the naming pattern `{pattern}` (kebab-case by default)")));
            }
        }
    };
    for i in at_rules(sheet) {
        let r = sheet.nodes[i].params;
        match sheet.at_name(i, src).as_str() {
            "layer" => check(r.0, r.1, out),
            "import" => {
                let vals = values(src, r.0, r.1);
                for v in significant(&vals) {
                    if v.kind == VKind::Func && v.name(src).eq_ignore_ascii_case("layer") {
                        check(v.inner.0, v.inner.1, out);
                    }
                }
            }
            _ => {}
        }
    }
});

css_rule!(ContainerNamePattern, "css/container-name-pattern", |sheet, src, ctx, out| {
    let Some((re, pattern)) = kebab(ctx, "^(--)?([a-z][a-z0-9]*)(-[a-z0-9]+)*$") else { return };
    let not_names = ["none", "and", "or", "not", "style", "scroll-state", "initial", "inherit", "unset", "revert", "revert-layer", "default"];
    let report = |s: usize, e: usize, out: &mut Vec<crate::NativeFinding>| {
        let name = &src[s..e];
        if !re.is_match(name) && !not_names.contains(&name.to_ascii_lowercase().as_str()) {
            out.push(finding(s, e, format!("Container name `{name}` doesn't match the naming pattern `{pattern}` (kebab-case by default)")));
        }
    };
    for i in sheet.walk() {
        let n = &sheet.nodes[i];
        match n.kind {
            Kind::Decl => {
                let p = text(src, n.name).to_ascii_lowercase();
                if p != "container-name" && p != "container" {
                    continue;
                }
                let end = if p == "container" { src[n.params.0..n.params.1].find('/').map_or(n.params.1, |o| n.params.0 + o) } else { n.params.1 };
                let vals = values(src, n.params.0, end);
                for v in significant(&vals) {
                    if v.kind == VKind::Word {
                        report(v.start, v.end, out);
                    }
                }
            }
            Kind::AtRule if sheet.at_name(i, src) == "container" => {
                let vals = values(src, n.params.0, n.params.1);
                if let Some(first) = significant(&vals).first()
                    && first.kind == VKind::Word
                {
                    report(first.start, first.end, out);
                }
            }
            _ => {}
        }
    }
});

css_rule!(CustomMediaPattern, "css/custom-media-pattern", |sheet, src, ctx, out| {
    let Some((re, pattern)) = kebab(ctx, KEBAB) else { return };
    for i in at_rules(sheet) {
        if sheet.at_name(i, src) != "custom-media" {
            continue;
        }
        let r = sheet.nodes[i].params;
        let e = crate::css_syntax::ident_end(src, r.0);
        let Some(name) = src[r.0..e].strip_prefix("--") else { continue };
        if !re.is_match(name) {
            out.push(finding(r.0, e, format!("Custom media `--{name}` doesn't match the naming pattern `{pattern}` (kebab-case by default)")));
        }
    }
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syntax_strings() {
        assert_eq!(syntax_problem("<color>"), None);
        assert_eq!(syntax_problem("<length> | <color>+"), None);
        assert_eq!(syntax_problem("*"), None);
        assert!(syntax_problem("<bar>").is_some());
        assert!(syntax_problem("<alpha-value>").is_some());
    }

    #[test]
    fn preludes() {
        assert!(prelude_problem("property", "foo").is_some());
        assert!(prelude_problem("property", "--foo").is_none());
        assert!(prelude_problem("layer", "base, theme.dark").is_none());
        assert!(prelude_problem("keyframes", "none").is_some());
    }
}
