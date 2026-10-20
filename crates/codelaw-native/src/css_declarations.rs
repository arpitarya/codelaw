//! CSS rules about declarations: properties, their values, and the
//! declaration blocks that hold them.

use crate::css_syntax::{
    Kind, Sheet, VKind, VNode, css_rule, dimension, finding, fixed, matches_any, non_standard_value, opt_list, opt_str, set, significant, split_commas, standard_property,
    table, text, unprefixed, values, vendor_prefix,
};
use crate::util::{unquote, whole_lines};
use codelaw_core::TextEdit;
use std::collections::{BTreeMap, BTreeSet};

/// The direct declarations of every block (and of the top level), in order.
fn blocks(sheet: &Sheet) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    let top: Vec<usize> = sheet.top.iter().copied().filter(|&i| sheet.nodes[i].kind == Kind::Decl).collect();
    if !top.is_empty() {
        out.push(top);
    }
    for i in sheet.walk() {
        let d: Vec<usize> = sheet.nodes[i].children.iter().copied().filter(|&c| sheet.nodes[c].kind == Kind::Decl).collect();
        if !d.is_empty() {
            out.push(d);
        }
    }
    out
}

fn prop<'s>(sheet: &Sheet, i: usize, src: &'s str) -> &'s str {
    text(src, sheet.nodes[i].name)
}

fn value<'s>(sheet: &Sheet, i: usize, src: &'s str) -> &'s str {
    text(src, sheet.nodes[i].params)
}

/// The range of a declaration including its `;`, widened to whole lines when alone.
fn decl_removal(sheet: &Sheet, i: usize, src: &str) -> TextEdit {
    let n = &sheet.nodes[i];
    let end = n.semicolon.map_or(n.end, |s| s + 1);
    let (s, e) = whole_lines(src, n.start, end);
    TextEdit::delete(s, e)
}

/// Declarations whose properties are descriptors of an at-rule
/// (`@font-face`, `@counter-style`, …) rather than CSS properties.
fn in_descriptor_context(sheet: &Sheet, i: usize, src: &str, nesting: &BTreeSet<String>) -> bool {
    sheet.ancestors(i).any(|a| sheet.nodes[a].kind == Kind::AtRule && !nesting.contains(&sheet.at_name(a, src)))
}

/// Whether a node's parent is one of the named at-rules.
fn parent_is(sheet: &Sheet, i: usize, src: &str, names: &[&str]) -> bool {
    sheet.nodes[i].parent.is_some_and(|p| sheet.nodes[p].kind == Kind::AtRule && names.contains(&sheet.at_name(p, src).as_str()))
}

fn levenshtein(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            cur.push((prev[j] + usize::from(ca != *cb)).min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    prev[b.len()]
}

css_rule!(AnnotationNoUnknown, "css/annotation-no-unknown", |sheet, src, ctx, out| {
    let ignore = opt_list(ctx, "ignoreAnnotations");
    for i in sheet.decls() {
        let Some((s, e)) = sheet.nodes[i].important else { continue };
        let word = src[s + 1..e].trim();
        if !word.eq_ignore_ascii_case("important") && !matches_any(&ignore, word) {
            out.push(finding(s, e, format!("Unknown annotation `!{word}`; the only one CSS has is `!important`")));
        }
    }
});

/// Properties whose values may name a custom property as an identifier.
const IDENT_PROPERTIES: &[&str] = &[
    "anchor-name", "anchor-scope", "animation", "animation-name", "animation-timeline", "container-name", "counter-increment", "counter-reset", "counter-set", "grid-column",
    "grid-column-end", "grid-column-start", "grid-row", "grid-row-end", "grid-row-start", "list-style", "list-style-type", "position-anchor", "position-try",
    "position-try-fallbacks", "timeline-scope", "transition", "transition-property", "view-transition-name", "will-change",
];

css_rule!(CustomPropertyNoMissingVarFunction, "css/custom-property-no-missing-var-function", |sheet, src, ctx, out| {
    let mut known = BTreeSet::new();
    for i in sheet.walk() {
        let n = &sheet.nodes[i];
        match n.kind {
            Kind::Decl if text(src, n.name).starts_with("--") => {
                known.insert(text(src, n.name).to_owned());
            }
            Kind::AtRule if sheet.at_name(i, src) == "property" => {
                known.insert(text(src, n.params).to_owned());
            }
            _ => {}
        }
    }
    if known.is_empty() {
        return;
    }
    fn check(list: &[VNode], src: &str, known: &BTreeSet<String>, out: &mut Vec<crate::NativeFinding>) {
        for n in list {
            match n.kind {
                VKind::Func => {
                    let name = n.name(src).to_ascii_lowercase();
                    match name.as_str() {
                        "var" => {
                            // The fallback after the first comma.
                            if let Some(c) = n.nodes.iter().position(|x| x.kind == VKind::Div && x.text(src) == ",") {
                                check(&n.nodes[c + 1..], src, known, out);
                            }
                        }
                        "anchor" | "anchor-size" | "style" | "env" | "running" => {}
                        // A custom function (`--theme(--name)`) takes names as arguments.
                        _ if name.starts_with("--") => {}
                        _ => check(&n.nodes, src, known, out),
                    }
                }
                VKind::Word => {
                    let t = n.text(src);
                    if t.starts_with("--") && known.contains(t) {
                        out.push(crate::css_syntax::fixed(n.start, n.end, format!("`{t}` is a custom property; read it with `var({t})`"), vec![TextEdit::new(n.start, n.end, format!("var({t})"))]));
                    }
                }
                _ => {}
            }
        }
    }
    for i in sheet.decls() {
        let p = prop(sheet, i, src).to_ascii_lowercase();
        if p.starts_with("--") || IDENT_PROPERTIES.contains(&p.as_str()) || !value(sheet, i, src).contains("--") {
            continue;
        }
        let r = sheet.nodes[i].params;
        check(&values(src, r.0, r.1), src, &known, out);
    }
});

css_rule!(DeclarationBlockNoDuplicateCustomProperties, "css/declaration-block-no-duplicate-custom-properties", |sheet, src, ctx, out| {
    for block in blocks(sheet) {
        let mut seen = BTreeSet::new();
        for i in block {
            let p = prop(sheet, i, src);
            if p.starts_with("--") && !seen.insert(p) {
                let r = sheet.nodes[i].name;
                out.push(finding(r.0, r.1, format!("`{p}` is declared twice in this block; the earlier value is never used")));
            }
        }
    }
});

/// A value's shape for comparing two declarations' syntaxes: token
/// types, function names, identifiers and units, not numbers.
fn syntax_shape(list: &[VNode], src: &str, color_prop: bool, named_colors: &BTreeSet<String>, out: &mut Vec<String>) {
    for n in significant(list) {
        match n.kind {
            VKind::Word => {
                let t = n.text(src);
                if let Some((_, unit)) = dimension(t) {
                    out.push(format!("dim:{unit}"));
                } else if t.starts_with('#') {
                    out.push("hash".into());
                } else if t.starts_with("--") {
                    out.push("custom".into());
                } else if color_prop && named_colors.contains(&t.to_ascii_lowercase()) {
                    out.push("color".into());
                } else {
                    out.push(format!("ident:{}", t.to_ascii_lowercase()));
                }
            }
            VKind::Str => out.push("str".into()),
            VKind::Div => out.push(format!("div:{}", n.text(src))),
            VKind::Func => {
                out.push(format!("fn:{}(", n.name(src).to_ascii_lowercase()));
                syntax_shape(&n.nodes, src, color_prop, named_colors, out);
                out.push(")".into());
            }
            _ => {}
        }
    }
}

css_rule!(DeclarationBlockNoDuplicateProperties, "css/declaration-block-no-duplicate-properties", |sheet, src, ctx, out| {
    let named = set(ctx.data, "css/keywords", "named_colors");
    let colors: BTreeSet<String> = set(ctx.data, "css/properties-groups", "single_value_color").into_iter().chain(["border-color".to_owned(), "scrollbar-color".to_owned()]).collect();
    let ignore = opt_list(ctx, "ignoreProperties");
    let consecutive_syntaxes = opt_list(ctx, "ignore").iter().any(|x| x == "consecutive-duplicates-with-different-syntaxes");
    for block in blocks(sheet) {
        // The declaration currently in effect for each property.
        let mut active: BTreeMap<String, usize> = BTreeMap::new();
        let mut previous: Option<String> = None;
        for i in block {
            let p = prop(sheet, i, src);
            let lower = p.to_ascii_lowercase();
            if p.starts_with("--") || !standard_property(p) || lower == "src" || matches_any(&ignore, p) {
                continue;
            }
            let consecutive = previous.as_deref() == Some(lower.as_str());
            previous = Some(lower.clone());
            let Some(&prev) = active.get(&lower) else {
                active.insert(lower, i);
                continue;
            };
            if consecutive_syntaxes && consecutive {
                let shape = |d: usize| {
                    let mut v = Vec::new();
                    let r = sheet.nodes[d].params;
                    syntax_shape(&values(src, r.0, r.1), src, colors.contains(&lower), &named, &mut v);
                    v
                };
                let same_text = value(sheet, prev, src) == value(sheet, i, src);
                if !same_text && (non_standard_value(value(sheet, i, src)) || shape(prev) != shape(i)) {
                    // A fallback: `width: 100px; width: 50vw;`.
                    active.insert(lower, i);
                    continue;
                }
            }
            // The earlier one loses, unless only it is `!important`.
            let prev_important = sheet.nodes[prev].important.is_some();
            let (loser, winner) = if prev_important && sheet.nodes[i].important.is_none() { (i, prev) } else { (prev, i) };
            active.insert(lower, winner);
            let r = sheet.nodes[loser].name;
            out.push(fixed(r.0, r.1, format!("`{p}` is declared twice in this block; this one is overridden"), vec![decl_removal(sheet, loser, src)]));
        }
    }
});

css_rule!(DeclarationBlockNoShorthandPropertyOverrides, "css/declaration-block-no-shorthand-property-overrides", |sheet, src, ctx, out| {
    let Some(longhands) = table(ctx.data, "css/shorthands").and_then(|t| t.get("longhands")).and_then(|v| v.as_object()) else { return };
    let resets = table(ctx.data, "css/shorthands").and_then(|t| t.get("resets")).and_then(|v| v.as_object());
    for block in blocks(sheet) {
        let mut seen: BTreeMap<String, String> = BTreeMap::new();
        for i in block {
            let p = prop(sheet, i, src);
            let lower = p.to_ascii_lowercase();
            let prefix = vendor_prefix(&lower).unwrap_or("").to_owned();
            let base = unprefixed(&lower).to_owned();
            let mut covered: Vec<&str> = longhands.get(&base).and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str()).collect()).unwrap_or_default();
            covered.extend(resets.and_then(|r| r.get(&base)).and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>()).unwrap_or_default());
            for long in covered {
                if let Some(original) = seen.get(&format!("{prefix}{long}")) {
                    let r = sheet.nodes[i].name;
                    out.push(finding(r.0, r.1, format!("`{p}` resets `{original}`, which is declared before it")));
                }
            }
            seen.insert(lower, p.to_owned());
        }
    }
});

css_rule!(DeclarationPropertyValueKeywordNoDeprecated, "css/declaration-property-value-keyword-no-deprecated", |sheet, src, ctx, out| {
    let Some(dep) = table(ctx.data, "css/deprecated") else { return };
    let keywords = dep.get("keywords").and_then(|v| v.as_object());
    let colors = dep.get("colors").and_then(|v| v.as_object());
    let mut color_props = set(ctx.data, "css/properties-groups", "single_value_color");
    color_props.extend(set(ctx.data, "css/deprecated", "color_properties"));
    let color_fns = set(ctx.data, "css/functions", "color");
    let ignore = opt_list(ctx, "ignoreKeywords");
    for i in sheet.decls() {
        let p = prop(sheet, i, src).to_ascii_lowercase();
        if !standard_property(&p) {
            continue;
        }
        let list = keywords.and_then(|k| k.get(&p)).and_then(|v| v.as_object());
        let is_color = color_props.contains(&p);
        if list.is_none() && !is_color {
            continue;
        }
        let r = sheet.nodes[i].params;
        let vals = values(src, r.0, r.1);
        VNode::walk(&vals, &mut |n| {
            if n.kind == VKind::Func {
                return !color_fns.contains(&n.name(src).to_ascii_lowercase());
            }
            if n.kind != VKind::Word {
                return true;
            }
            let word = n.text(src);
            let lower = word.to_ascii_lowercase();
            if matches_any(&ignore, word) {
                return true;
            }
            let hit = list.and_then(|l| l.get(&lower)).or_else(|| if is_color { colors.and_then(|c| c.get(&lower)) } else { None });
            if let Some(instead) = hit {
                match instead.as_str() {
                    Some(new) => out.push(fixed(n.start, n.end, format!("`{word}` is a deprecated keyword for `{p}`; use `{new}`"), vec![TextEdit::new(n.start, n.end, new)])),
                    None => out.push(finding(n.start, n.end, format!("`{word}` is a deprecated keyword for `{p}`"))),
                }
            }
            true
        });
    }
});

css_rule!(DeclarationPropertyValueNoUnknown, "css/declaration-property-value-no-unknown", |sheet, src, ctx, out| {
    let Some(Some(table)) = table(ctx.data, "css/property-keywords").map(|t| t.as_object()) else { return };
    let wide = set(ctx.data, "css/keywords", "css_wide");
    let nesting = set(ctx.data, "css/at-rules", "nesting");
    for i in sheet.decls() {
        let p = prop(sheet, i, src).to_ascii_lowercase();
        let Some(allowed) = table.get(&p).and_then(|v| v.as_array()) else { continue };
        if in_descriptor_context(sheet, i, src, &nesting) {
            continue;
        }
        let r = sheet.nodes[i].params;
        let v = text(src, r);
        let b = v.as_bytes();
        // One plain identifier: no functions, numbers, strings or vendor prefixes.
        if v.is_empty() || !b[0].is_ascii_alphabetic() || !b.iter().all(|&c| c.is_ascii_alphanumeric() || c == b'-') {
            continue;
        }
        let lower = v.to_ascii_lowercase();
        if wide.contains(&lower) || allowed.iter().any(|a| a.as_str().is_some_and(|a| a.eq_ignore_ascii_case(&lower))) {
            continue;
        }
        out.push(finding(r.0, r.1, format!("`{v}` is not a value of `{p}`")));
    }
});

/// A family name as written, and whether it was quoted; the key compares
/// unquoted names case-insensitively only when they are keywords.
fn families(src: &str, r: (usize, usize)) -> Vec<((usize, usize), String, bool)> {
    split_commas(src, r.0, r.1)
        .into_iter()
        .filter(|f| f.1 > f.0)
        .map(|f| {
            let t = text(src, f);
            let quoted = t.starts_with(['"', '\'']);
            let name = if quoted && t.len() >= 2 { unquote(t).to_owned() } else { t.split_whitespace().collect::<Vec<_>>().join(" ") };
            (f, name, quoted)
        })
        .collect()
}

const SIZE_KEYWORDS: &[&str] = &["xx-small", "x-small", "small", "medium", "large", "x-large", "xx-large", "xxx-large", "larger", "smaller"];

/// The font-family part of a `font` shorthand: what follows the size and
/// optional `/ line-height`.
fn font_shorthand_families(src: &str, r: (usize, usize)) -> Option<(usize, usize)> {
    let vals = values(src, r.0, r.1);
    let sig = significant(&vals);
    let mut k = None;
    for (idx, n) in sig.iter().enumerate() {
        if n.kind == VKind::Word {
            let t = n.text(src).to_ascii_lowercase();
            let is_size = SIZE_KEYWORDS.contains(&t.as_str()) || dimension(&t).is_some_and(|(_, u)| !u.is_empty());
            if is_size {
                k = Some(idx);
                break;
            }
        } else if n.kind == VKind::Func && ["calc", "clamp", "min", "max", "var"].contains(&n.name(src).to_ascii_lowercase().as_str()) {
            k = Some(idx);
            break;
        }
    }
    let mut k = k?;
    if sig.get(k + 1).is_some_and(|n| n.kind == VKind::Div && n.text(src) == "/") {
        k += 2;
    }
    let first = sig.get(k + 1)?;
    Some((first.start, r.1))
}

css_rule!(FontFamilyNoDuplicateNames, "css/font-family-no-duplicate-names", |sheet, src, ctx, out| {
    let keywords = set(ctx.data, "css/keywords", "font_family");
    for i in sheet.decls() {
        let p = prop(sheet, i, src).to_ascii_lowercase();
        let r = sheet.nodes[i].params;
        if non_standard_value(text(src, r)) || text(src, r).contains("var(") {
            continue;
        }
        let range = match p.as_str() {
            "font-family" => r,
            "font" => match font_shorthand_families(src, r) {
                Some(f) => f,
                None => continue,
            },
            _ => continue,
        };
        let fams = families(src, range);
        let mut seen = BTreeSet::new();
        for (idx, (f, name, quoted)) in fams.iter().enumerate() {
            let key = if !quoted && keywords.contains(&name.to_ascii_lowercase()) { format!("kw:{}", name.to_ascii_lowercase()) } else { format!("name:{name}") };
            if !seen.insert(key) {
                // Remove the duplicate with the comma before it.
                let prev_end = fams[idx - 1].0.1;
                out.push(fixed(f.0, f.1, format!("Font family `{name}` is listed twice"), vec![TextEdit::delete(prev_end, f.1)]));
            }
        }
    }
});

css_rule!(FontFamilyNoMissingGenericFamilyKeyword, "css/font-family-no-missing-generic-family-keyword", |sheet, src, ctx, out| {
    let keywords = set(ctx.data, "css/keywords", "font_family");
    let system = set(ctx.data, "css/keywords", "system_font");
    let ignore = opt_list(ctx, "ignoreFontFamilies");
    for i in sheet.decls() {
        let p = prop(sheet, i, src).to_ascii_lowercase();
        if parent_is(sheet, i, src, &["font-face", "font-palette-values"]) {
            continue;
        }
        let r = sheet.nodes[i].params;
        let v = text(src, r);
        if non_standard_value(v) || v.contains('(') {
            continue;
        }
        let range = match p.as_str() {
            "font-family" => r,
            "font" => {
                if system.contains(&v.to_ascii_lowercase()) {
                    continue;
                }
                match font_shorthand_families(src, r) {
                    Some(f) => f,
                    None => continue,
                }
            }
            _ => continue,
        };
        let fams = families(src, range);
        let Some(last) = fams.last() else { continue };
        if fams.iter().any(|(_, name, quoted)| (!quoted && keywords.contains(&name.to_ascii_lowercase())) || matches_any(&ignore, name)) {
            continue;
        }
        out.push(finding(last.0.0, last.0.1, "No generic family (`sans-serif`, `serif`, `monospace`, …) ends this font list"));
    }
});

/// The cells of each row string of `grid-template-areas`.
fn grid_rows(src: &str, list: &[VNode]) -> Vec<Vec<String>> {
    significant(list)
        .into_iter()
        .filter(|n| n.kind == VKind::Str)
        .map(|n| {
            let t = n.text(src);
            let inner = if t.len() >= 2 { unquote(t) } else { "" };
            let mut cells = Vec::new();
            let mut cur = String::new();
            for c in inner.chars() {
                if c.is_whitespace() {
                    if !cur.is_empty() {
                        cells.push(std::mem::take(&mut cur));
                    }
                } else if c == '.' {
                    if !cur.is_empty() && !cur.starts_with('.') {
                        cells.push(std::mem::take(&mut cur));
                    }
                    cur.push('.');
                } else {
                    if cur.starts_with('.') {
                        cells.push(std::mem::take(&mut cur));
                    }
                    cur.push(c);
                }
            }
            if !cur.is_empty() {
                cells.push(cur);
            }
            cells
        })
        .collect()
}

css_rule!(NamedGridAreasNoInvalid, "css/named-grid-areas-no-invalid", |sheet, src, ctx, out| {
    for i in sheet.decls() {
        let p = prop(sheet, i, src).to_ascii_lowercase();
        if !matches!(p.as_str(), "grid-template-areas" | "grid-template" | "grid") {
            continue;
        }
        let r = sheet.nodes[i].params;
        let vals = values(src, r.0, r.1);
        let rows = grid_rows(src, &vals);
        if rows.is_empty() {
            continue;
        }
        if rows.iter().any(|row| row.is_empty()) {
            out.push(finding(r.0, r.1, "A grid row string must name at least one cell"));
            continue;
        }
        if rows.iter().any(|row| row.len() != rows[0].len()) {
            out.push(finding(r.0, r.1, "Every grid row string must have the same number of cells"));
            continue;
        }
        let mut areas: BTreeMap<&str, Vec<(usize, usize)>> = BTreeMap::new();
        for (y, row) in rows.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                if !cell.starts_with('.') {
                    areas.entry(cell.as_str()).or_default().push((y, x));
                }
            }
        }
        for (name, cells) in areas {
            let (y0, y1) = (cells.iter().map(|c| c.0).min().unwrap_or(0), cells.iter().map(|c| c.0).max().unwrap_or(0));
            let (x0, x1) = (cells.iter().map(|c| c.1).min().unwrap_or(0), cells.iter().map(|c| c.1).max().unwrap_or(0));
            if cells.len() != (y1 - y0 + 1) * (x1 - x0 + 1) {
                out.push(finding(r.0, r.1, format!("Grid area `{name}` is not a single filled-in rectangle")));
            }
        }
    }
});

css_rule!(NoInvalidPositionDeclaration, "css/no-invalid-position-declaration", |sheet, src, ctx, out| {
    let nesting = set(ctx.data, "css/at-rules", "nesting");
    let containing = set(ctx.data, "css/at-rules", "declaration_containing");
    for i in sheet.decls() {
        if !standard_property(prop(sheet, i, src)) {
            continue;
        }
        let ok = match sheet.nodes[i].parent {
            None => false,
            Some(p) if sheet.nodes[p].kind == Kind::Rule => true,
            Some(p) => {
                let name = sheet.at_name(p, src);
                if containing.contains(&name) || !nesting.contains(&name) {
                    true
                } else {
                    sheet.ancestors(i).any(|a| sheet.nodes[a].kind == Kind::Rule || (sheet.nodes[a].kind == Kind::AtRule && containing.contains(&sheet.at_name(a, src))))
                }
            }
        };
        if !ok {
            let n = &sheet.nodes[i];
            out.push(finding(n.start, n.end, "A declaration here applies to nothing; put it inside a style rule"));
        }
    }
});

css_rule!(PropertyNoDeprecated, "css/property-no-deprecated", |sheet, src, ctx, out| {
    let Some(dep) = table(ctx.data, "css/deprecated") else { return };
    let Some(props) = dep.get("properties").and_then(|v| v.as_object()) else { return };
    let remap = dep.get("property_value_remap").and_then(|v| v.as_object());
    let ignore = opt_list(ctx, "ignoreProperties");
    for i in sheet.decls() {
        let p = prop(sheet, i, src);
        let lower = p.to_ascii_lowercase();
        let Some(instead) = props.get(&lower) else { continue };
        if matches_any(&ignore, p) || (lower == "-webkit-box-orient" && value(sheet, i, src).eq_ignore_ascii_case("vertical")) {
            continue;
        }
        let r = sheet.nodes[i].name;
        match instead.as_str() {
            Some(new) => {
                let mut edits = vec![TextEdit::new(r.0, r.1, new)];
                let v = value(sheet, i, src).to_ascii_lowercase();
                if let Some(nv) = remap.and_then(|m| m.get(&lower)).and_then(|m| m.get(&v)).and_then(|x| x.as_str()) {
                    let vr = sheet.nodes[i].params;
                    edits.push(TextEdit::new(vr.0, vr.1, nv));
                }
                out.push(fixed(r.0, r.1, format!("`{p}` is deprecated; use `{new}`"), edits));
            }
            None => out.push(finding(r.0, r.1, format!("`{p}` is deprecated"))),
        }
    }
});

css_rule!(NoUnknownProperty, "css/no-unknown-property", |sheet, src, ctx, out| {
    let known = set(ctx.data, "css/properties", "");
    if known.is_empty() {
        return;
    }
    let nesting = set(ctx.data, "css/at-rules", "nesting");
    let ignore = opt_list(ctx, "ignoreProperties");
    let check_prefixed = ctx.options.get("checkPrefixed").and_then(|v| v.as_bool()).unwrap_or(false);
    for i in sheet.decls() {
        let p = prop(sheet, i, src);
        let lower = p.to_ascii_lowercase();
        if lower.starts_with("--") || !standard_property(p) || p.contains('\\') || (!check_prefixed && vendor_prefix(&lower).is_some()) || matches_any(&ignore, p) {
            continue;
        }
        if known.contains(&lower) || in_descriptor_context(sheet, i, src, &nesting) {
            continue;
        }
        let r = sheet.nodes[i].name;
        let best = known.iter().filter(|k| !k.starts_with('-')).map(|k| (levenshtein(&lower, k), k)).min();
        match best {
            Some((d, k)) if d <= 2 && d < lower.len() / 2 + 1 => {
                out.push(fixed(r.0, r.1, format!("Unknown property `{p}`; did you mean `{k}`?"), vec![TextEdit::new(r.0, r.1, k.clone())]));
            }
            _ => out.push(finding(r.0, r.1, format!("Unknown property `{p}`"))),
        }
    }
});

css_rule!(NoImportant, "css/no-important", |sheet, src, ctx, out| {
    for i in sheet.decls() {
        if let Some((s, e)) = sheet.nodes[i].important
            && src[s + 1..e].trim().eq_ignore_ascii_case("important")
        {
            out.push(finding(s, e, "`!important` overrides the cascade; raise the selector's specificity or use a cascade layer instead"));
        }
    }
});

css_rule!(DeclarationBlockNoRedundantLonghandProperties, "css/declaration-block-no-redundant-longhand-properties", |sheet, src, ctx, out| {
    let Some(longhands) = table(ctx.data, "css/shorthands").and_then(|t| t.get("longhands")).and_then(|v| v.as_object()) else { return };
    let wide = set(ctx.data, "css/keywords", "css_wide");
    for block in blocks(sheet) {
        let mut by_prop: BTreeMap<String, usize> = BTreeMap::new();
        let mut shorthands: BTreeMap<String, String> = BTreeMap::new();
        for &i in &block {
            let lower = prop(sheet, i, src).to_ascii_lowercase();
            let v = value(sheet, i, src);
            if wide.contains(&v.to_ascii_lowercase()) {
                continue;
            }
            if longhands.contains_key(unprefixed(&lower)) {
                shorthands.insert(lower.clone(), v.to_owned());
            }
            by_prop.insert(lower.clone(), i);
            let prefix = vendor_prefix(&lower).unwrap_or("").to_owned();
            let base = unprefixed(&lower).to_owned();
            for (short, longs) in longhands {
                let Some(longs) = longs.as_array() else { continue };
                if !longs.iter().any(|l| l.as_str() == Some(base.as_str())) {
                    continue;
                }
                let short_p = format!("{prefix}{short}");
                // A longhand repeating the shorthand before it.
                if shorthands.get(&short_p).is_some_and(|sv| sv == v) {
                    let r = sheet.nodes[i].name;
                    out.push(fixed(r.0, r.1, format!("`{}` repeats what `{short_p}` already set", prop(sheet, i, src)), vec![decl_removal(sheet, i, src)]));
                    continue;
                }
                let all: Option<Vec<usize>> = longs.iter().filter_map(|l| l.as_str()).map(|l| by_prop.get(&format!("{prefix}{l}")).copied()).collect();
                let Some(decls) = all else { continue };
                // Only when this declaration completes the set.
                if decls.iter().max() != Some(&i) {
                    continue;
                }
                let important: Vec<bool> = decls.iter().map(|&d| sheet.nodes[d].important.is_some()).collect();
                if important.iter().any(|&x| x) && !important.iter().all(|&x| x) {
                    continue;
                }
                let first = *decls.iter().min().unwrap_or(&i);
                out.push(finding(sheet.nodes[first].start, sheet.nodes[i].end, format!("These {} longhands can be one `{short_p}` declaration", decls.len())));
            }
        }
    }
});

/// The shorthands whose repeated values collapse, and whether they take four sides.
fn condensable(p: &str) -> Option<bool> {
    match p {
        "margin" | "padding" | "border-color" | "border-style" | "border-width" | "scroll-margin" | "scroll-padding" | "inset" => Some(true),
        "margin-block" | "margin-inline" | "padding-block" | "padding-inline" | "border-block-color" | "border-block-style" | "border-block-width" | "border-inline-color"
        | "border-inline-style" | "border-inline-width" | "gap" | "grid-gap" | "overflow" | "overscroll-behavior" | "scroll-margin-block" | "scroll-margin-inline"
        | "scroll-padding-block" | "scroll-padding-inline" | "inset-block" | "inset-inline" => Some(false),
        _ => None,
    }
}

fn condense(v: &[&str], four: bool) -> usize {
    let l: Vec<String> = v.iter().map(|s| s.to_ascii_lowercase()).collect();
    if !four {
        return if l.len() == 2 && l[0] == l[1] { 1 } else { l.len() };
    }
    let g = |i: usize| l.get(i).map(String::as_str);
    let (t, r, b, lf) = (g(0), g(1), g(2), g(3));
    let n = l.len();
    if n < 2 {
        return n;
    }
    if t == r && ((t == b && (b == lf || lf.is_none())) || (b.is_none() && lf.is_none())) {
        return 1;
    }
    if n >= 3 && ((t == b && r == lf) || (t == b && lf.is_none() && t != r)) {
        return 2;
    }
    if n == 4 && r == lf {
        return 3;
    }
    n
}

css_rule!(ShorthandPropertyNoRedundantValues, "css/shorthand-property-no-redundant-values", |sheet, src, ctx, out| {
    for i in sheet.decls() {
        let lower = prop(sheet, i, src).to_ascii_lowercase();
        let base = unprefixed(&lower);
        let r = sheet.nodes[i].params;
        let v = text(src, r);
        if non_standard_value(v) || v.to_ascii_lowercase().contains("var(") {
            continue;
        }
        let vals = values(src, r.0, r.1);
        let sig = significant(&vals);
        if base == "border-radius" {
            let slash = sig.iter().position(|n| n.kind == VKind::Div && n.text(src) == "/");
            if sig.iter().any(|n| n.kind == VKind::Div && n.text(src) != "/") {
                continue;
            }
            let (h, vv) = match slash {
                Some(s) => (&sig[..s], &sig[s + 1..]),
                None => (&sig[..], &sig[..0]),
            };
            let ht: Vec<&str> = h.iter().map(|n| n.text(src)).collect();
            let vt: Vec<&str> = vv.iter().map(|n| n.text(src)).collect();
            let (hn, vn) = (condense(&ht, true), condense(&vt, true));
            if hn < ht.len() || vn < vt.len() {
                let mut new = ht[..hn].join(" ");
                if slash.is_some() {
                    new = format!("{new} / {}", vt[..vn].join(" "));
                }
                out.push(fixed(r.0, r.1, format!("Repeated values; `{v}` can be `{new}`"), vec![TextEdit::new(r.0, r.1, new)]));
            }
            continue;
        }
        let Some(four) = condensable(base) else { continue };
        if sig.iter().any(|n| n.kind != VKind::Word && n.kind != VKind::Func) || sig.len() > if four { 4 } else { 2 } {
            continue;
        }
        let toks: Vec<&str> = sig.iter().map(|n| n.text(src)).collect();
        let n = condense(&toks, four);
        if n < toks.len() {
            let new = toks[..n].join(" ");
            out.push(fixed(r.0, r.1, format!("Repeated values; `{v}` can be `{new}`"), vec![TextEdit::new(r.0, r.1, new)]));
        }
    }
});

css_rule!(PropertyNoVendorPrefix, "css/property-no-vendor-prefix", |sheet, src, ctx, out| {
    let props = set(ctx.data, "css/prefixed", "properties");
    let ignore = opt_list(ctx, "ignoreProperties");
    for i in sheet.decls() {
        let p = prop(sheet, i, src);
        let lower = p.to_ascii_lowercase();
        let Some(prefix) = vendor_prefix(&lower) else { continue };
        let base = &lower[prefix.len()..];
        if !props.contains(base) || matches_any(&ignore, p) {
            continue;
        }
        if lower == "-webkit-background-size" {
            // Two-value sizes mean the same prefixed and not (whatwg/compat#28).
            let safe = split_commas(src, sheet.nodes[i].params.0, sheet.nodes[i].params.1).iter().all(|r| {
                let parts: Vec<&str> = text(src, *r).split_whitespace().collect();
                parts.len() == 2 || (parts.len() == 1 && parts[0] == "auto")
            });
            if !safe {
                continue;
            }
        }
        let r = sheet.nodes[i].name;
        out.push(fixed(r.0, r.1, format!("`{p}` has a vendor prefix; write `{base}` and let a build tool add prefixes"), vec![TextEdit::new(r.0, r.1, base)]));
    }
});

css_rule!(ValueNoVendorPrefix, "css/value-no-vendor-prefix", |sheet, src, ctx, out| {
    let prefixed = set(ctx.data, "css/prefixed", "values");
    let ignore: Vec<String> = { let mut v = opt_list(ctx, "ignoreValues"); if !ctx.options.contains_key("ignoreValues") { v = vec!["box".into(), "inline-box".into()]; } v };
    for i in sheet.decls() {
        let p = prop(sheet, i, src);
        let r = sheet.nodes[i].params;
        if !standard_property(p) || !text(src, r).contains('-') {
            continue;
        }
        let vals = values(src, r.0, r.1);
        VNode::walk(&vals, &mut |n| {
            let (s, e) = match n.kind {
                VKind::Word => (n.start, n.end),
                VKind::Func => n.name,
                _ => return true,
            };
            let word = &src[s..e];
            let lower = word.to_ascii_lowercase();
            if prefixed.contains(&lower) && !matches_any(&ignore, unprefixed(&lower)) && !matches_any(&ignore, word) {
                let base = unprefixed(word);
                out.push(fixed(s, e, format!("`{word}` has a vendor prefix; write `{base}`"), vec![TextEdit::new(s, e, base)]));
            }
            true
        });
    }
});

css_rule!(CustomPropertyPattern, "css/custom-property-pattern", |sheet, src, ctx, out| {
    let pattern = opt_str(ctx, "pattern", "^([a-z][a-z0-9]*)(-[a-z0-9]+)*$");
    let Ok(re) = regex::Regex::new(pattern) else { return };
    let mut report = |s: usize, e: usize, name: &str| {
        if !re.is_match(name) {
            out.push(finding(s, e, format!("Custom property `--{name}` doesn't match the naming pattern `{pattern}` (kebab-case by default)")));
        }
    };
    for i in sheet.walk() {
        let n = &sheet.nodes[i];
        match n.kind {
            Kind::Decl => {
                let p = text(src, n.name);
                if let Some(name) = p.strip_prefix("--") {
                    report(n.name.0, n.name.1, name);
                }
                // Uses: `var(--name)`.
                let v = text(src, n.params);
                if v.contains("var(") {
                    let vals = values(src, n.params.0, n.params.1);
                    VNode::walk(&vals, &mut |x| {
                        if x.kind == VKind::Func && x.name(src).eq_ignore_ascii_case("var")
                            && let Some(first) = significant(&x.nodes).first()
                            && let Some(name) = first.text(src).strip_prefix("--")
                        {
                            report(first.start, first.end, name);
                        }
                        true
                    });
                }
            }
            Kind::AtRule if sheet.at_name(i, src) == "property" => {
                if let Some(name) = text(src, n.params).strip_prefix("--") {
                    report(n.params.0, n.params.1, name);
                }
            }
            _ => {}
        }
    }
});
