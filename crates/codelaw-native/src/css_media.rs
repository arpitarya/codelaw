//! CSS rules about media queries, with a small reader for their grammar
//! (Media Queries level 5).

use crate::css_syntax::{Feature, Kind, Range, Sheet, css_rule, dimension, finding, fixed, ident_end, matches_any, opt_list, set, skip_comment, skip_string, split_commas, table, text, unprefixed, vendor_prefix};
use codelaw_core::TextEdit;

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Ident(Range),
    /// `( … )`: the whole, and the inside.
    Paren(Range, Range),
    /// `name( … )`.
    Func(Range),
    Other(Range),
}

fn tokens(src: &str, s: usize, e: usize) -> Vec<Tok> {
    let b = src.as_bytes();
    let mut out = Vec::new();
    let mut i = s;
    while i < e {
        let c = b[i];
        if c.is_ascii_whitespace() {
            i += 1;
        } else if src[i..].starts_with("/*") {
            i = skip_comment(src, i).min(e);
        } else if c == b'(' {
            let close = matching(src, i, e);
            out.push(Tok::Paren((i, (close + 1).min(e)), (i + 1, close)));
            i = close + 1;
        } else if c.is_ascii_alphabetic() || c == b'-' || c == b'_' || c >= 0x80 || c == b'\\' {
            let end = ident_end(src, i).min(e).max(i + 1);
            if end < e && b[end] == b'(' {
                let close = matching(src, end, e);
                out.push(Tok::Func((i, (close + 1).min(e))));
                i = close + 1;
            } else {
                out.push(Tok::Ident((i, end)));
                i = end;
            }
        } else {
            let start = i;
            if c == b'"' || c == b'\'' {
                i = skip_string(b, i).min(e);
            } else {
                i += src[i..].chars().next().map_or(1, char::len_utf8);
            }
            out.push(Tok::Other((start, i)));
        }
    }
    out
}

/// The `)` matching the `(` at `open` (or `e`).
fn matching(src: &str, open: usize, e: usize) -> usize {
    let b = src.as_bytes();
    let mut depth = 0;
    let mut i = open;
    while i < e {
        match b[i] {
            b'"' | b'\'' => {
                i = skip_string(b, i);
                continue;
            }
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            _ => {}
        }
        i += 1;
    }
    e
}

fn word(src: &str, t: &Tok) -> Option<String> {
    match t {
        Tok::Ident(r) => Some(text(src, *r).to_ascii_lowercase()),
        _ => None,
    }
}

struct Query<'s> {
    src: &'s str,
    features: Vec<Feature>,
    types: Vec<Range>,
}

impl Query<'_> {
    /// `<media-condition>`; `allow_or` is false for `<media-condition-without-or>`.
    fn condition(&mut self, toks: &[Tok], allow_or: bool) -> Result<(), String> {
        let src = self.src;
        if toks.is_empty() {
            return Err("a condition is missing".into());
        }
        if word(src, &toks[0]).as_deref() == Some("not") {
            if toks.len() != 2 {
                return Err("`not` takes exactly one parenthesised condition".into());
            }
            return self.in_parens(&toks[1]);
        }
        self.in_parens(&toks[0])?;
        let mut op: Option<String> = None;
        let mut i = 1;
        while i < toks.len() {
            let w = word(src, &toks[i]).ok_or_else(|| format!("expected `and` or `or`, found `{}`", tok_text(src, &toks[i])))?;
            if w != "and" && w != "or" {
                return Err(format!("expected `and` or `or`, found `{w}`"));
            }
            if w == "or" && !allow_or {
                return Err("`or` can't follow a media type; wrap the conditions in parentheses".into());
            }
            if op.as_deref().is_some_and(|o| o != w) {
                return Err("`and` and `or` can't be mixed at one level; add parentheses".into());
            }
            op = Some(w);
            let next = toks.get(i + 1).ok_or("a condition is missing after the last operator")?;
            self.in_parens(next)?;
            i += 2;
        }
        Ok(())
    }

    fn in_parens(&mut self, t: &Tok) -> Result<(), String> {
        let src = self.src;
        match t {
            Tok::Paren(whole, inner) => {
                let inside = tokens(src, inner.0, inner.1);
                let nested = matches!(inside.first(), Some(Tok::Paren(..))) || inside.first().and_then(|x| word(src, x)).as_deref() == Some("not") && inside.len() == 2 && matches!(inside[1], Tok::Paren(..) | Tok::Func(..));
                if nested {
                    return self.condition(&inside, true);
                }
                self.feature(*whole, *inner)
            }
            Tok::Func(r) => {
                let name = text(src, *r).split('(').next().unwrap_or("").to_ascii_lowercase();
                if matches!(name.as_str(), "not" | "and" | "or" | "only") {
                    Err(format!("`{name}(` needs a space before the parenthesis"))
                } else {
                    Ok(())
                }
            }
            other => Err(format!("expected a parenthesised condition, found `{}`", tok_text(src, other))),
        }
    }

    fn feature(&mut self, whole: Range, inner: Range) -> Result<(), String> {
        let src = self.src;
        let t = text(src, inner);
        if t.trim().is_empty() {
            return Err("empty parentheses".into());
        }
        // `name: value`
        if let Some(colon) = top_level(t, b':') {
            let name = crate::css_syntax::trim_comments(src, (inner.0, inner.0 + colon));
            let value = crate::css_syntax::trim_comments(src, (inner.0 + colon + 1, inner.1));
            if !is_ident(text(src, name)) {
                return Err(format!("`{}` is not a feature name", text(src, name)));
            }
            if value.0 == value.1 {
                return Err(format!("`{}:` has no value", text(src, name)));
            }
            self.features.push(Feature { whole, name, value: Some(value), range_values: Vec::new() });
            return Ok(());
        }
        // A range: values and names separated by comparisons.
        let b = src.as_bytes();
        let mut parts: Vec<Range> = Vec::new();
        let mut ops: Vec<String> = Vec::new();
        let mut i = inner.0;
        let mut start = inner.0;
        let mut depth = 0;
        while i < inner.1 {
            match b[i] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                b'<' | b'>' | b'=' if depth == 0 => {
                    parts.push(crate::css_syntax::trim_comments(src, (start, i)));
                    let mut j = i;
                    while j < inner.1 && matches!(b[j], b'<' | b'>' | b'=') {
                        j += 1;
                    }
                    ops.push(src[i..j].to_owned());
                    i = j;
                    start = j;
                    continue;
                }
                _ => {}
            }
            i += 1;
        }
        parts.push(crate::css_syntax::trim_comments(src, (start, inner.1)));
        if ops.is_empty() {
            let name = crate::css_syntax::trim_comments(src, inner);
            if !is_ident(text(src, name)) {
                return Err(format!("`{}` is not a media feature", text(src, name)));
            }
            self.features.push(Feature { whole, name, value: None, range_values: Vec::new() });
            return Ok(());
        }
        for op in &ops {
            if !matches!(op.as_str(), "<" | "<=" | ">" | ">=" | "=") {
                return Err(format!("`{op}` is not a comparison; use `<`, `<=`, `>`, `>=` or `=`"));
            }
        }
        if parts.iter().any(|p| p.0 == p.1) {
            return Err("a comparison is missing a side".into());
        }
        let names: Vec<usize> = (0..parts.len()).filter(|&k| is_ident(text(src, parts[k])) && dimension(text(src, parts[k])).is_none()).collect();
        match (parts.len(), names.as_slice()) {
            (2, [k]) => {
                let v = parts[1 - k];
                self.features.push(Feature { whole, name: parts[*k], value: None, range_values: vec![v] });
                Ok(())
            }
            (3, [1]) => {
                let dir = |o: &str| o.starts_with('<');
                if ops.iter().any(|o| o == "=") || dir(&ops[0]) != dir(&ops[1]) {
                    return Err("both comparisons of a range must point the same way".into());
                }
                self.features.push(Feature { whole, name: parts[1], value: None, range_values: vec![parts[0], parts[2]] });
                Ok(())
            }
            _ => Err("a range compares one feature name with values".into()),
        }
    }
}

fn tok_text<'s>(src: &'s str, t: &Tok) -> &'s str {
    match t {
        Tok::Ident(r) | Tok::Paren(r, _) | Tok::Func(r) | Tok::Other(r) => text(src, *r),
    }
}

fn top_level(t: &str, c: u8) -> Option<usize> {
    let b = t.as_bytes();
    let mut depth = 0;
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'"' | b'\'' => {
                i = skip_string(b, i);
                continue;
            }
            b'(' => depth += 1,
            b')' => depth -= 1,
            x if x == c && depth == 0 => return Some(i),
            _ => {}
        }
        i += 1;
    }
    None
}

fn is_ident(t: &str) -> bool {
    let b = t.as_bytes();
    !b.is_empty() && (b[0].is_ascii_alphabetic() || b[0] == b'-' || b[0] == b'_') && b.iter().all(|&c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
}

/// One media query: its features and media types, or why it is invalid.
fn query(src: &str, r: Range) -> (Result<(), String>, Vec<Feature>, Vec<Range>) {
    let toks = tokens(src, r.0, r.1);
    let mut q = Query { src, features: Vec::new(), types: Vec::new() };
    let res = (|| {
        if toks.is_empty() {
            return Err("empty media query".to_owned());
        }
        let first = word(src, &toks[0]);
        let mut i = 0;
        if matches!(first.as_deref(), Some("not" | "only")) && matches!(toks.get(1), Some(Tok::Ident(_))) {
            i = 1;
        }
        if let Some(Tok::Ident(t)) = toks.get(i)
            && !matches!(word(src, &toks[i]).as_deref(), Some("not" | "and" | "or" | "only"))
        {
            q.types.push(*t);
            let rest = &toks[i + 1..];
            if rest.is_empty() {
                return Ok(());
            }
            if word(src, &rest[0]).as_deref() != Some("and") {
                return Err(format!("expected `and` after the media type, found `{}`", tok_text(src, &rest[0])));
            }
            return q.condition(&rest[1..], false);
        }
        q.condition(&toks, true)
    })();
    (res, q.features, q.types)
}

/// The media query lists of a sheet: `@media` and `@custom-media` preludes,
/// by statement.
fn media_lists(sheet: &Sheet, src: &str) -> Vec<(usize, Range)> {
    let mut out = Vec::new();
    for i in sheet.walk() {
        let n = &sheet.nodes[i];
        if n.kind != Kind::AtRule {
            continue;
        }
        let p = n.params;
        let t = text(src, p);
        if crate::css_syntax::non_standard_value(t) || t.contains('@') || t.contains('$') || t.contains("#{") {
            continue;
        }
        match sheet.at_name(i, src).as_str() {
            "media" => out.push((i, p)),
            "custom-media" => {
                let e = ident_end(src, p.0);
                out.push((i, crate::css_syntax::trim_comments(src, (e, p.1))));
            }
            _ => {}
        }
    }
    out
}

/// Each media query of the sheet: its range, whether it is valid, its
/// features and its media types. The queries, their features and types come
/// from the tree when the sheet was read from one (W-62); whether a query is
/// valid is judged on its text, which the grammar may accept where Media
/// Queries does not — and an invalid query keeps the features and types the
/// text read before it stopped.
fn each_query(sheet: &Sheet, src: &str, f: &mut dyn FnMut(Range, Result<(), String>, Vec<Feature>, Vec<Range>)) {
    for (i, list) in media_lists(sheet, src) {
        if let Some(queries) = &sheet.nodes[i].queries {
            for q in queries {
                let (res, features, types) = query(src, q.range);
                if res.is_ok() {
                    f(q.range, res, q.features.clone(), q.types.clone());
                } else {
                    f(q.range, res, features, types);
                }
            }
            continue;
        }
        for q in split_commas(src, list.0, list.1) {
            if q.0 == q.1 {
                continue;
            }
            let (res, features, types) = query(src, q);
            f(q, res, features, types);
        }
    }
}

css_rule!(MediaQueryNoInvalid, "css/media-query-no-invalid", |sheet, src, ctx, out| {
    let ignore = opt_list(ctx, "ignoreFunctions");
    each_query(sheet, src, &mut |q, res, _, _| {
        if let Err(why) = res {
            if !ignore.is_empty() && ignore.iter().any(|f| text(src, q).contains(&format!("{f}("))) {
                return;
            }
            // A custom media query, `(--name)`, is always fine.
            out.push(finding(q.0, q.1, format!("Invalid media query `{}`: {why}", text(src, q))));
        }
    });
});

css_rule!(MediaFeatureNameNoUnknown, "css/media-feature-name-no-unknown", |sheet, src, ctx, out| {
    let known = set(ctx.data, "css/media", "features");
    let ignore = opt_list(ctx, "ignoreMediaFeatureNames");
    each_query(sheet, src, &mut |_, _, features, _| {
        for f in features {
            let name = text(src, f.name);
            let lower = name.to_ascii_lowercase();
            if lower.starts_with("--") || vendor_prefix(&lower).is_some() || known.contains(&lower) || matches_any(&ignore, name) {
                continue;
            }
            out.push(finding(f.name.0, f.name.1, format!("Unknown media feature `{name}`")));
        }
    });
});

fn value_ok(v: &str, ty: &str) -> bool {
    let lower = v.to_ascii_lowercase();
    match ty {
        "length" => dimension(&lower).is_some_and(|(n, u)| (u.is_empty() && n.parse::<f64>().ok() == Some(0.0)) || (!u.is_empty() && u != "%")),
        "integer" => dimension(&lower).is_some_and(|(n, u)| u.is_empty() && !n.contains('.') && !n.contains('e')),
        "mq-boolean" => lower == "0" || lower == "1",
        "ratio" => {
            let parts: Vec<&str> = lower.split('/').map(str::trim).collect();
            parts.len() <= 2 && parts.iter().all(|p| dimension(p).is_some_and(|(_, u)| u.is_empty()))
        }
        "resolution" => dimension(&lower).is_some_and(|(_, u)| matches!(u, "dpi" | "dpcm" | "dppx" | "x")),
        _ => true,
    }
}

css_rule!(MediaFeatureNameValueNoUnknown, "css/media-feature-name-value-no-unknown", |sheet, src, ctx, out| {
    let Some(media) = table(ctx.data, "css/media") else { return };
    let keywords = media.get("keyword_values").and_then(|v| v.as_object());
    let types = media.get("value_types").and_then(|v| v.as_object());
    each_query(sheet, src, &mut |_, res, features, _| {
        if res.is_err() {
            return;
        }
        for f in features {
            let name = text(src, f.name).to_ascii_lowercase();
            let base = name.strip_prefix("min-").or_else(|| name.strip_prefix("max-")).unwrap_or(&name);
            let vals: Vec<Range> = f.value.into_iter().chain(f.range_values.iter().copied()).collect();
            for r in vals {
                let v = text(src, r);
                let lower = v.to_ascii_lowercase();
                if v.contains('(') || v.starts_with("--") {
                    continue;
                }
                let kw = keywords.and_then(|k| k.get(base)).and_then(|x| x.as_array());
                let ty = types.and_then(|t| t.get(base)).and_then(|x| x.as_array());
                if kw.is_none() && ty.is_none() {
                    continue;
                }
                let kw_ok = kw.is_some_and(|k| k.iter().any(|x| x.as_str() == Some(lower.as_str())));
                let ty_ok = ty.is_some_and(|t| t.iter().filter_map(|x| x.as_str()).any(|t| value_ok(v, t)));
                if !kw_ok && !ty_ok {
                    out.push(finding(r.0, r.1, format!("`{v}` is not a valid value for the media feature `{name}`")));
                }
            }
        }
    });
});

css_rule!(MediaTypeNoDeprecated, "css/media-type-no-deprecated", |sheet, src, ctx, out| {
    let deprecated = set(ctx.data, "css/media", "deprecated_types");
    let ignore = opt_list(ctx, "ignoreMediaTypes");
    each_query(sheet, src, &mut |_, _, _, types| {
        for t in types {
            let name = text(src, t);
            if deprecated.contains(&name.to_ascii_lowercase()) && !matches_any(&ignore, name) {
                out.push(finding(t.0, t.1, format!("The media type `{name}` is deprecated and matches nothing; use media features instead")));
            }
        }
    });
});

css_rule!(MediaFeatureNameNoVendorPrefix, "css/media-feature-name-no-vendor-prefix", |sheet, src, ctx, out| {
    each_query(sheet, src, &mut |_, _, features, _| {
        for f in features {
            let name = text(src, f.name);
            let lower = name.to_ascii_lowercase();
            if vendor_prefix(&lower).is_some() && lower.contains("device-pixel-ratio") {
                let standard = unprefixed(&lower).replace("device-pixel-ratio", "resolution");
                out.push(finding(f.name.0, f.name.1, format!("`{name}` has a vendor prefix; use `{standard}` with a `dppx` value")));
            }
        }
    });
});

css_rule!(MediaFeatureRangeNotation, "css/media-feature-range-notation", |sheet, src, ctx, out| {
    let min_max = set(ctx.data, "css/media", "min_max_features");
    each_query(sheet, src, &mut |_, res, features, _| {
        if res.is_err() {
            return;
        }
        for f in features {
            let Some(v) = f.value else { continue };
            let name = text(src, f.name);
            let lower = name.to_ascii_lowercase();
            if !min_max.contains(&lower) {
                continue;
            }
            let (op, base) = if let Some(b) = lower.strip_prefix("min-") { (">=", b) } else if let Some(b) = lower.strip_prefix("max-") { ("<=", b) } else { continue };
            let new = format!("({base} {op} {})", text(src, v));
            out.push(fixed(f.whole.0, f.whole.1, format!("Use the range notation `{new}`"), vec![TextEdit::new(f.whole.0, f.whole.1, new)]));
        }
    });
});

#[cfg(test)]
mod tests {
    use super::*;

    fn check(q: &str) -> Result<(), String> {
        query(q, (0, q.len())).0
    }

    #[test]
    fn grammar() {
        assert!(check("screen").is_ok());
        assert!(check("only screen and (min-width: 1px)").is_ok());
        assert!(check("(width >= 600px)").is_ok());
        assert!(check("(400px <= width <= 700px)").is_ok());
        assert!(check("((color) and (hover)) or (width)").is_ok());
        assert!(check("not (min-width: 300px)").is_ok());
        assert!(check("print and not (color)").is_ok());
        assert!(check("(--custom)").is_ok());
        assert!(check("not(min-width: 300px)").is_err());
        assert!(check("(width == 100px)").is_err());
        assert!(check("(color) and (hover) or (width)").is_err());
        assert!(check("screen or (color)").is_err());
        assert!(check("(1px < width > 2px)").is_err());
    }
}
