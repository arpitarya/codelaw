//! CSS rules about how values are written: math operators, colour
//! notation, zero lengths, number precision.

use crate::css_syntax::{Kind, Sheet, VKind, VNode, css_rule, dimension, fixed, non_standard_value, opt_list, set, significant, text, unprefixed, values};
use codelaw_core::TextEdit;
use std::collections::BTreeSet;

/// Every declaration value and at-rule prelude, with its property (or `@name`).
fn value_sites(sheet: &Sheet, src: &str) -> Vec<(String, (usize, usize))> {
    sheet
        .walk()
        .into_iter()
        .filter_map(|i| {
            let n = &sheet.nodes[i];
            match n.kind {
                Kind::Decl => Some((text(src, n.name).to_ascii_lowercase(), n.params)),
                Kind::AtRule => Some((format!("@{}", sheet.at_name(i, src)), n.params)),
                _ => None,
            }
        })
        .filter(|(_, r)| !non_standard_value(text(src, *r)))
        .collect()
}

const MATH: &[&str] = &[
    "calc", "clamp", "min", "max", "round", "mod", "rem", "sin", "cos", "tan", "asin", "acos", "atan", "atan2", "pow", "sqrt", "hypot", "log", "exp", "abs", "sign", "calc-size",
];

fn is_math(name: &str) -> bool {
    MATH.contains(&unprefixed(&name.to_ascii_lowercase()))
}

/// The `+`/`-` operators inside a math function that lack a space on one side.
fn unspaced_operators(src: &str, s: usize, e: usize, out: &mut Vec<crate::NativeFinding>) {
    let b = src.as_bytes();
    let mut i = s;
    let mut depth_stack: Vec<bool> = Vec::new(); // is each open paren a math context
    let mut math = true;
    while i < e {
        let c = b[i];
        match c {
            b'"' | b'\'' => {
                i = crate::css_syntax::skip_string(b, i);
                continue;
            }
            b'/' if b.get(i + 1) == Some(&b'*') => {
                i = crate::css_syntax::skip_comment(src, i);
                continue;
            }
            b'(' => {
                // A function name before the paren decides the context.
                let mut j = i;
                while j > s && (b[j - 1].is_ascii_alphanumeric() || b[j - 1] == b'-' || b[j - 1] == b'_') {
                    j -= 1;
                }
                let name = &src[j..i];
                depth_stack.push(math);
                math = name.is_empty() || is_math(name);
            }
            b')' => math = depth_stack.pop().unwrap_or(true),
            b'+' | b'-' if math => {
                let before_ws = i > s && b[i - 1].is_ascii_whitespace();
                let after_ws = b.get(i + 1).is_some_and(|x| x.is_ascii_whitespace());
                let prev = src[s..i].trim_end().as_bytes().last().copied();
                let operand_before = prev.is_some_and(|p| p.is_ascii_alphanumeric() || p == b'%' || p == b')' || p == b'.');
                // `-` right after a letter or `-` is part of a name (`--x`, `a-b`, `1px-2px`).
                let in_name = c == b'-' && !before_ws && i > s && (b[i - 1].is_ascii_alphabetic() || b[i - 1] == b'-' || b[i - 1] == b'_');
                // An exponent: `1e-3`.
                let exponent = !before_ws && i > s + 1 && (b[i - 1] == b'e' || b[i - 1] == b'E') && b[i - 2].is_ascii_digit();
                if operand_before && !in_name && !exponent && (!before_ws || !after_ws) {
                    let op = c as char;
                    let mut edits = Vec::new();
                    if !before_ws {
                        edits.push(TextEdit::new(i, i, " "));
                    }
                    if !after_ws {
                        edits.push(TextEdit::new(i + 1, i + 1, " "));
                    }
                    out.push(fixed(i, i + 1, format!("Operator `{op}` needs a space on each side; without them it reads as a sign"), edits));
                }
            }
            _ => {}
        }
        i += 1;
    }
}

css_rule!(FunctionCalcNoUnspacedOperator, "css/function-calc-no-unspaced-operator", |sheet, src, ctx, out| {
    for i in sheet.decls() {
        let r = sheet.nodes[i].params;
        if non_standard_value(text(src, r)) {
            continue;
        }
        let vals = values(src, r.0, r.1);
        VNode::walk(&vals, &mut |n| {
            if n.kind == VKind::Func && is_math(n.name(src)) {
                unspaced_operators(src, n.inner.0, n.inner.1, out);
                return false;
            }
            true
        });
    }
});

/// The colour functions and their arguments split into channels and alpha.
struct Color<'a> {
    func: &'a VNode,
    name: String,
    /// Channel nodes (words or functions), in order.
    channels: Vec<&'a VNode>,
    alpha: Option<&'a VNode>,
    legacy: bool,
}

fn color<'a>(n: &'a VNode, src: &str, fns: &BTreeSet<String>) -> Option<Color<'a>> {
    if n.kind != VKind::Func {
        return None;
    }
    let name = n.name(src).to_ascii_lowercase();
    if !fns.contains(&name) {
        return None;
    }
    let sig = significant(&n.nodes);
    let legacy = sig.iter().any(|x| x.kind == VKind::Div && x.text(src) == ",");
    let (channels, alpha) = if legacy {
        let items: Vec<&VNode> = sig.into_iter().filter(|x| x.kind != VKind::Div).collect();
        if items.len() == 4 { (items[..3].to_vec(), Some(items[3])) } else { (items, None) }
    } else {
        match sig.iter().position(|x| x.kind == VKind::Div && x.text(src) == "/") {
            Some(p) => (sig[..p].to_vec(), sig.get(p + 1).copied()),
            None => (sig, None),
        }
    };
    Some(Color { func: n, name, channels, alpha, legacy })
}

fn each_color<'a>(vals: &'a [VNode], src: &str, fns: &BTreeSet<String>, f: &mut dyn FnMut(Color<'a>)) {
    VNode::walk(vals, &mut |n| {
        if let Some(c) = color(n, src, fns) {
            f(c);
        }
        true
    });
}

/// A plain number, as a float.
fn number(t: &str) -> Option<f64> {
    match dimension(t) {
        Some((n, "")) => n.parse().ok(),
        _ => None,
    }
}

/// Format a float without trailing zeros.
fn fmt(x: f64) -> String {
    let s = format!("{:.6}", x);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" { "0".into() } else { s.into() }
}

const ALPHA_NUMBER_PROPS: &[&str] = &["opacity", "fill-opacity", "flood-opacity", "stop-opacity", "stroke-opacity"];

css_rule!(AlphaValueNotation, "css/alpha-value-notation", |sheet, src, ctx, out| {
    let fns = set(ctx.data, "css/functions", "color");
    let numbers: Vec<String> = if ctx.options.contains_key("exceptProperties") { opt_list(ctx, "exceptProperties") } else { ALPHA_NUMBER_PROPS.iter().map(|s| s.to_string()).collect() };
    for i in sheet.decls() {
        let p = text(src, sheet.nodes[i].name).to_ascii_lowercase();
        let r = sheet.nodes[i].params;
        if non_standard_value(text(src, r)) {
            continue;
        }
        let vals = values(src, r.0, r.1);
        if numbers.contains(&p) {
            // These properties take a number.
            let sig = significant(&vals);
            if let [one] = sig.as_slice()
                && one.kind == VKind::Word
                && let Some((n, "%")) = dimension(one.text(src))
                && let Ok(v) = n.parse::<f64>()
            {
                let new = fmt(v / 100.0);
                out.push(fixed(one.start, one.end, format!("Write the alpha value `{}` as the number `{new}`", one.text(src)), vec![TextEdit::new(one.start, one.end, new)]));
            }
            continue;
        }
        each_color(&vals, src, &fns, &mut |c| {
            let Some(a) = c.alpha else { return };
            if a.kind != VKind::Word {
                return;
            }
            if let Some(v) = number(a.text(src)) {
                let new = format!("{}%", fmt(v * 100.0));
                out.push(fixed(a.start, a.end, format!("Write the alpha value `{}` as the percentage `{new}`", a.text(src)), vec![TextEdit::new(a.start, a.end, new)]));
            }
        });
    }
});

css_rule!(ColorFunctionAliasNotation, "css/color-function-alias-notation", |sheet, src, ctx, out| {
    for (_, r) in value_sites(sheet, src) {
        let vals = values(src, r.0, r.1);
        VNode::walk(&vals, &mut |n| {
            if n.kind == VKind::Func {
                let name = n.name(src);
                let lower = name.to_ascii_lowercase();
                if lower == "rgba" || lower == "hsla" {
                    let short = &name[..3];
                    out.push(fixed(n.name.0, n.name.1, format!("`{name}()` is an alias; write `{short}()`"), vec![TextEdit::new(n.name.0, n.name.1, short)]));
                }
            }
            true
        });
    }
});

css_rule!(ColorFunctionNotation, "css/color-function-notation", |sheet, src, ctx, out| {
    let fns: BTreeSet<String> = ["rgb", "rgba", "hsl", "hsla"].iter().map(|s| s.to_string()).collect();
    for (_, r) in value_sites(sheet, src) {
        let vals = values(src, r.0, r.1);
        each_color(&vals, src, &fns, &mut |c| {
            if !c.legacy || text(src, c.func.inner).to_ascii_lowercase().contains("var(") {
                return;
            }
            let parts: Vec<&str> = c.channels.iter().map(|x| x.text(src)).collect();
            if parts.len() != 3 {
                return;
            }
            let mut new = parts.join(" ");
            if let Some(a) = c.alpha {
                new = format!("{new} / {}", a.text(src));
            }
            let f = c.func;
            out.push(fixed(f.start, f.end, format!("Use the space-separated syntax: `{}({new})`", c.name), vec![TextEdit::new(f.inner.0, f.inner.1, new)]));
        });
    }
});

css_rule!(ColorHexLength, "css/color-hex-length", |sheet, src, ctx, out| {
    for (_, r) in value_sites(sheet, src) {
        let vals = values(src, r.0, r.1);
        VNode::walk(&vals, &mut |n| {
            if n.kind == VKind::Func && n.name(src).eq_ignore_ascii_case("url") {
                return false;
            }
            if n.kind != VKind::Word {
                return true;
            }
            let t = n.text(src);
            let Some(hex) = t.strip_prefix('#') else { return true };
            if !(hex.len() == 6 || hex.len() == 8) || !hex.bytes().all(|c| c.is_ascii_hexdigit()) {
                return true;
            }
            let b = hex.as_bytes();
            if b.chunks(2).all(|p| p[0].eq_ignore_ascii_case(&p[1])) {
                let short: String = b.chunks(2).map(|p| p[0] as char).collect();
                out.push(fixed(n.start, n.end, format!("Write `{t}` as `#{short}`"), vec![TextEdit::new(n.start, n.end, format!("#{short}"))]));
            }
            true
        });
    }
});

css_rule!(HueDegreeNotation, "css/hue-degree-notation", |sheet, src, ctx, out| {
    let fns = set(ctx.data, "css/functions", "color");
    for (_, r) in value_sites(sheet, src) {
        let vals = values(src, r.0, r.1);
        each_color(&vals, src, &fns, &mut |c| {
            let idx = match c.name.as_str() {
                "hsl" | "hsla" | "hwb" => 0,
                "lch" | "oklch" => 2,
                _ => return,
            };
            let Some(h) = c.channels.get(idx) else { return };
            if h.kind == VKind::Word && number(h.text(src)).is_some() {
                out.push(fixed(h.start, h.end, format!("Write the hue `{}` as an angle: `{}deg`", h.text(src), h.text(src)), vec![TextEdit::new(h.end, h.end, "deg")]));
            }
        });
    }
});

css_rule!(LightnessNotation, "css/lightness-notation", |sheet, src, ctx, out| {
    let fns = set(ctx.data, "css/functions", "color");
    for (_, r) in value_sites(sheet, src) {
        let vals = values(src, r.0, r.1);
        each_color(&vals, src, &fns, &mut |c| {
            let scale = match c.name.as_str() {
                "oklch" | "oklab" => 100.0,
                "lch" | "lab" => 1.0,
                _ => return,
            };
            let Some(l) = c.channels.first() else { return };
            if l.kind != VKind::Word {
                return;
            }
            if let Some(v) = number(l.text(src)) {
                let new = format!("{}%", fmt(v * scale));
                out.push(fixed(l.start, l.end, format!("Write the lightness `{}` as the percentage `{new}`", l.text(src)), vec![TextEdit::new(l.start, l.end, new)]));
            }
        });
    }
});

css_rule!(LengthZeroNoUnit, "css/length-zero-no-unit", |sheet, src, ctx, out| {
    let lengths = set(ctx.data, "css/units", "length");
    let ignore_custom = opt_list(ctx, "ignore").iter().any(|x| x == "custom-properties");
    for (p, r) in value_sites(sheet, src) {
        if p == "line-height" || p == "flex" || (ignore_custom && p.starts_with("--")) || p.starts_with("@") && !matches!(p.as_str(), "@media" | "@custom-media" | "@container") {
            continue;
        }
        let vals = values(src, r.0, r.1);
        let mut prev_slash = false;
        let report = |n: &VNode, out: &mut Vec<crate::NativeFinding>| {
            let t = n.text(src);
            let Some((num, unit)) = dimension(t) else { return };
            if unit.is_empty() || unit == "%" || !lengths.contains(&unit.to_ascii_lowercase()) || unit.eq_ignore_ascii_case("fr") {
                return;
            }
            if num.parse::<f64>().ok() != Some(0.0) {
                return;
            }
            let keep = num.strip_prefix('.').unwrap_or(num);
            out.push(fixed(n.start, n.end, format!("A zero length needs no unit: write `{keep}`"), vec![TextEdit::new(n.start, n.end, keep)]));
        };
        for n in &vals {
            match n.kind {
                VKind::Div => prev_slash = n.text(src) == "/",
                VKind::Space | VKind::Comment => {}
                _ => {
                    // `font: 12px/0px` — the line height after the slash.
                    let skip = p == "font" && prev_slash;
                    prev_slash = false;
                    if skip {
                        continue;
                    }
                    if n.kind == VKind::Word {
                        report(n, out);
                    } else if n.kind == VKind::Func && !is_math(n.name(src)) && !n.name(src).eq_ignore_ascii_case("url") {
                        VNode::walk(&n.nodes, &mut |x| {
                            if x.kind == VKind::Func {
                                return !is_math(x.name(src)) && !x.name(src).eq_ignore_ascii_case("url");
                            }
                            if x.kind == VKind::Word {
                                report(x, out);
                            }
                            true
                        });
                    }
                }
            }
        }
    }
});

css_rule!(NumberMaxPrecision, "css/number-max-precision", |sheet, src, ctx, out| {
    let max = ctx.options.get("precision").and_then(|v| v.as_u64()).unwrap_or(4) as usize;
    let ignore_units = opt_list(ctx, "ignoreUnits");
    for (p, r) in value_sites(sheet, src) {
        if p.starts_with('@') && !matches!(p.as_str(), "@media" | "@custom-media" | "@container" | "@supports") {
            continue;
        }
        let vals = values(src, r.0, r.1);
        VNode::walk(&vals, &mut |n| {
            if n.kind == VKind::Func && n.name(src).eq_ignore_ascii_case("url") {
                return false;
            }
            if n.kind != VKind::Word {
                return true;
            }
            let t = n.text(src);
            let Some((num, unit)) = dimension(t) else { return true };
            if ignore_units.iter().any(|u| u.eq_ignore_ascii_case(unit)) {
                return true;
            }
            let mantissa = num.split(['e', 'E']).next().unwrap_or(num);
            let decimals = mantissa.split('.').nth(1).map_or(0, str::len);
            if decimals > max
                && let Ok(v) = mantissa.parse::<f64>()
            {
                let factor = 10f64.powi(max as i32);
                let rounded = fmt((v * factor).round() / factor);
                let exp = &num[mantissa.len()..];
                let new = format!("{rounded}{exp}{unit}");
                out.push(fixed(n.start, n.end, format!("`{t}` has {decimals} decimal places; at most {max} are allowed"), vec![TextEdit::new(n.start, n.end, new)]));
            }
            true
        });
    }
});

#[cfg(test)]
mod tests {
    use super::*;
    use codelaw_core::Language;

    #[test]
    fn calc_operators() {
        let src = "a { width: calc(1px+2px); height: calc(1px + -2px); top: calc(100% -1px); left: calc(var(--a-b) - 1e-3px); }";
        let got: Vec<String> = crate::run(&FunctionCalcNoUnspacedOperator, Language::Css, src).into_iter().map(|f| f.message).collect();
        assert_eq!(got.len(), 2, "{got:?}");
    }

    #[test]
    fn formatting() {
        assert_eq!(fmt(50.0), "50");
        assert_eq!(fmt(0.5), "0.5");
        assert_eq!(fmt(12.34500), "12.345");
    }
}
