//! Native rules for format strings checked against their arguments:
//! `"…" % args` (Ruff's F501–F509) and `"…".format(…)` (F521–F525).

use crate::python_util::{call_args, hit, hit_fix, is_fstring, is_pure, kids, py_rule, string_value, unparen};
use crate::util::text;
use crate::{NativeContext, NativeFinding};
use codelaw_core::TextEdit;
use std::collections::BTreeSet;
use tree_sitter::Node;

/// One `%` placeholder.
struct Percent {
    key: Option<String>,
    /// `*` width or precision, each taking a positional argument.
    stars: usize,
}

enum PercentError {
    /// The string ends inside a placeholder or a key.
    Invalid(&'static str),
    /// A conversion character Python doesn't know.
    Unsupported(char),
}

fn parse_percent(s: &str) -> Result<Vec<Percent>, PercentError> {
    let chars: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '%' {
            i += 1;
            continue;
        }
        i += 1;
        let mut key = None;
        if chars.get(i) == Some(&'(') {
            let mut depth = 1;
            let start = i + 1;
            i += 1;
            while i < chars.len() && depth > 0 {
                match chars[i] {
                    '(' => depth += 1,
                    ')' => depth -= 1,
                    _ => {}
                }
                i += 1;
            }
            if depth > 0 {
                return Err(PercentError::Invalid("a key is never closed"));
            }
            key = Some(chars[start..i - 1].iter().collect());
        }
        while i < chars.len() && "#0- +".contains(chars[i]) {
            i += 1;
        }
        let mut stars = 0;
        if chars.get(i) == Some(&'*') {
            stars += 1;
            i += 1;
        } else {
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
        }
        if chars.get(i) == Some(&'.') {
            i += 1;
            if chars.get(i) == Some(&'*') {
                stars += 1;
                i += 1;
            } else {
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
            }
        }
        while i < chars.len() && "hlL".contains(chars[i]) {
            i += 1;
        }
        let Some(&c) = chars.get(i) else { return Err(PercentError::Invalid("the string ends inside a placeholder")) };
        i += 1;
        if c == '%' && key.is_none() && stars == 0 {
            continue;
        }
        if !"diouxXeEfFgGcrsa%".contains(c) {
            return Err(PercentError::Unsupported(c));
        }
        out.push(Percent { key, stars });
    }
    Ok(out)
}

/// What a `%` right-hand side is, as far as the syntax shows.
enum Rhs<'t> {
    Tuple(usize),
    Dict(Vec<(String, Node<'t>)>, bool),
    /// A single literal value: one substitution.
    Single,
    Unknown,
}

fn rhs_kind<'t>(n: Node<'t>, src: &str) -> Rhs<'t> {
    let n = unparen(n);
    match n.kind() {
        "tuple" => {
            let items = kids(n);
            if items.iter().any(|c| c.kind() == "list_splat") { Rhs::Unknown } else { Rhs::Tuple(items.len()) }
        }
        "dictionary" => {
            let mut keys = Vec::new();
            let mut all_str = true;
            for p in kids(n) {
                if p.kind() != "pair" {
                    return Rhs::Dict(keys, false);
                }
                match p.child_by_field_name("key").and_then(|k| string_value(unparen(k), src)) {
                    Some(k) => keys.push((k, p)),
                    None => all_str = false,
                }
            }
            Rhs::Dict(keys, all_str)
        }
        "string" | "concatenated_string" | "integer" | "float" | "true" | "false" | "none" => Rhs::Single,
        _ => Rhs::Unknown,
    }
}

/// The `%` rule a finding belongs to.
#[derive(Clone, Copy, PartialEq, Eq)]
enum P {
    Invalid,
    ExpectedMapping,
    ExpectedSequence,
    ExtraNamed,
    Missing,
    Mixed,
    CountMismatch,
    StarNeedsSequence,
    Unsupported,
}

fn percent(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>, want: P) {
    let src = ctx.source;
    let (Some(left), Some(op), Some(right)) = (n.child_by_field_name("left"), n.child_by_field_name("operator"), n.child_by_field_name("right")) else {
        return;
    };
    if text(op, src) != "%" || is_fstring(left, src) {
        return;
    }
    let Some(fmt) = string_value(left, src) else { return };
    let placeholders = match parse_percent(&fmt) {
        Ok(p) => p,
        Err(PercentError::Invalid(why)) => {
            if want == P::Invalid {
                out.push(hit(n, format!("This `%` format string is invalid: {why}")));
            }
            return;
        }
        Err(PercentError::Unsupported(c)) => {
            if want == P::Unsupported {
                out.push(hit(n, format!("`%{c}` is not a `%` conversion Python supports")));
            }
            return;
        }
    };
    let named: BTreeSet<String> = placeholders.iter().filter_map(|p| p.key.clone()).collect();
    let positional = placeholders.iter().filter(|p| p.key.is_none()).count() + placeholders.iter().map(|p| p.stars).sum::<usize>();
    let has_star = placeholders.iter().any(|p| p.stars > 0);
    let rhs = rhs_kind(right, src);
    let report = |out: &mut Vec<NativeFinding>, which: P, msg: String| {
        if which == want {
            out.push(hit(n, msg));
        }
    };
    if !named.is_empty() && positional > 0 {
        report(out, P::Mixed, "This `%` format string mixes positional and named placeholders".into());
    }
    if has_star && (!named.is_empty() || matches!(rhs, Rhs::Dict(..))) {
        report(out, P::StarNeedsSequence, "A `*` width or precision takes its value from a sequence, not a mapping".into());
    }
    match &rhs {
        Rhs::Tuple(count) => {
            if !named.is_empty() {
                report(out, P::ExpectedMapping, "Named placeholders need a mapping, but a tuple is given".into());
            } else if positional != *count {
                report(out, P::CountMismatch, format!("The format string has {positional} placeholder(s) but {count} value(s) are given"));
            }
        }
        Rhs::Single => {
            if named.is_empty() && positional != 1 {
                report(out, P::CountMismatch, format!("The format string has {positional} placeholder(s) but 1 value is given"));
            }
        }
        Rhs::Dict(keys, all_str) => {
            if positional > 1 {
                report(out, P::ExpectedSequence, "Positional placeholders need a sequence, but a dict is given".into());
                if named.is_empty() {
                    report(out, P::CountMismatch, format!("The format string has {positional} placeholder(s) but 1 value is given"));
                }
                return;
            }
            if !*all_str || positional > 0 {
                return;
            }
            let given: BTreeSet<String> = keys.iter().map(|k| k.0.clone()).collect();
            let missing: Vec<&String> = named.difference(&given).collect();
            if !missing.is_empty() {
                report(out, P::Missing, format!("No value is given for {}", missing.iter().map(|m| format!("`{m}`")).collect::<Vec<_>>().join(", ")));
            }
            let extra: Vec<&(String, Node)> = keys.iter().filter(|k| !named.contains(&k.0)).collect();
            if !extra.is_empty() && want == P::ExtraNamed {
                let pure = extra.iter().all(|(_, p)| p.child_by_field_name("value").is_some_and(is_pure));
                let edits = if pure { remove_items(&extra.iter().map(|e| e.1).collect::<Vec<_>>(), src) } else { Vec::new() };
                let names = extra.iter().map(|e| format!("`{}`", e.0)).collect::<Vec<_>>().join(", ");
                out.push(hit_fix(n, format!("The format string never uses {names}"), edits));
            }
        }
        Rhs::Unknown => {}
    }
}

/// Edits removing items from a comma-separated list, with their commas.
fn remove_items(items: &[Node<'_>], src: &str) -> Vec<TextEdit> {
    let mut edits = Vec::new();
    for it in items {
        if let Some(next) = it.next_sibling().filter(|c| c.kind() == ",") {
            let mut stop = next.end_byte();
            while src[stop..].starts_with(' ') {
                stop += 1;
            }
            edits.push(TextEdit::new(it.start_byte(), stop, ""));
        } else if let Some(prev) = it.prev_sibling().filter(|c| c.kind() == ",") {
            edits.push(TextEdit::new(prev.start_byte(), it.end_byte(), ""));
        } else {
            edits.push(TextEdit::new(it.start_byte(), it.end_byte(), ""));
        }
    }
    // Merge touching ranges so edits never overlap.
    edits.sort_by_key(|e| e.start);
    let mut merged: Vec<TextEdit> = Vec::new();
    for e in edits {
        if let Some(last) = merged.last_mut()
            && e.start <= last.end
        {
            last.end = last.end.max(e.end);
            continue;
        }
        merged.push(e);
    }
    merged
}

macro_rules! percent_rule {
    ($ty:ident, $id:literal, $which:expr, $f:ident) => {
        fn $f(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
            percent(n, ctx, out, $which)
        }
        py_rule!($ty, $id, ["binary_operator"], false, $f);
    };
}

percent_rule!(PercentFormatInvalidFormat, "py/percent-format-invalid", P::Invalid, p_invalid);
percent_rule!(PercentFormatExpectedMapping, "py/percent-format-expected-mapping", P::ExpectedMapping, p_mapping);
percent_rule!(PercentFormatExpectedSequence, "py/percent-format-expected-sequence", P::ExpectedSequence, p_sequence);
percent_rule!(PercentFormatExtraNamedArguments, "py/percent-format-unused-named-argument", P::ExtraNamed, p_extra);
percent_rule!(PercentFormatMissingArgument, "py/percent-format-missing-argument", P::Missing, p_missing);
percent_rule!(PercentFormatMixedPositionalAndNamed, "py/percent-format-mixed-placeholders", P::Mixed, p_mixed);
percent_rule!(PercentFormatPositionalCountMismatch, "py/percent-format-count-mismatch", P::CountMismatch, p_count);
percent_rule!(PercentFormatStarRequiresSequence, "py/percent-format-star-requires-sequence", P::StarNeedsSequence, p_star);
percent_rule!(PercentFormatUnsupportedFormatCharacter, "py/percent-format-unsupported-character", P::Unsupported, p_unsupported);

/// A `str.format` field's argument: automatic, an index, or a name.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Arg {
    Auto,
    Index(usize),
    Name(String),
}

/// The arguments a `str.format` string refers to, nested fields included.
fn parse_brace(s: &str) -> Result<Vec<Arg>, String> {
    let chars: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    parse_brace_into(&chars, &mut out, 0)?;
    Ok(out)
}

fn parse_brace_into(chars: &[char], out: &mut Vec<Arg>, depth: usize) -> Result<(), String> {
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '{' if chars.get(i + 1) == Some(&'{') && depth == 0 => i += 2,
            '}' if chars.get(i + 1) == Some(&'}') && depth == 0 => i += 2,
            '}' => return Err("a single `}` is not escaped as `}}`".into()),
            '{' => {
                let start = i + 1;
                let mut level = 1;
                i += 1;
                while i < chars.len() && level > 0 {
                    match chars[i] {
                        '{' => level += 1,
                        '}' => level -= 1,
                        _ => {}
                    }
                    i += 1;
                }
                if level > 0 {
                    return Err("a `{` is never closed".into());
                }
                let field: &[char] = &chars[start..i - 1];
                // Field name ends at `!` or `:` outside brackets.
                let mut j = 0;
                let mut in_bracket = false;
                while j < field.len() {
                    match field[j] {
                        '[' => in_bracket = true,
                        ']' => in_bracket = false,
                        '!' | ':' if !in_bracket => break,
                        _ => {}
                    }
                    j += 1;
                }
                let name: String = field[..j].iter().collect();
                let head: String = name.chars().take_while(|c| *c != '.' && *c != '[').collect();
                out.push(if head.is_empty() {
                    Arg::Auto
                } else if head.chars().all(|c| c.is_ascii_digit()) {
                    Arg::Index(head.parse().map_err(|_| "an index is too large".to_owned())?)
                } else {
                    Arg::Name(head)
                });
                if depth > 1 {
                    return Err("format fields nest too deeply".into());
                }
                if let Some(colon) = field[j..].iter().position(|c| *c == ':') {
                    parse_brace_into(&field[j + colon + 1..], out, depth + 1)?;
                }
            }
            _ => i += 1,
        }
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum B {
    Invalid,
    ExtraNamed,
    ExtraPositional,
    Missing,
    Mixing,
}

fn brace(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>, want: B) {
    let src = ctx.source;
    let Some(func) = n.child_by_field_name("function").filter(|f| f.kind() == "attribute") else { return };
    if func.child_by_field_name("attribute").map(|a| text(a, src)) != Some("format") {
        return;
    }
    let Some(recv) = func.child_by_field_name("object") else { return };
    if is_fstring(recv, src) {
        return;
    }
    let Some(fmt) = string_value(recv, src) else { return };
    let fields = match parse_brace(&fmt) {
        Ok(f) => f,
        Err(e) => {
            if want == B::Invalid {
                out.push(hit(n, format!("This `.format` string is invalid: {e}")));
            }
            return;
        }
    };
    let args = call_args(n, src);
    let has_star = args.list.is_some_and(|l| kids(l).iter().any(|c| c.kind() == "list_splat"));
    let has_kwstar = args.list.is_some_and(|l| kids(l).iter().any(|c| c.kind() == "dictionary_splat"));
    let auto = fields.contains(&Arg::Auto);
    let manual = fields.iter().any(|f| matches!(f, Arg::Index(_)));
    if auto && manual {
        if want == B::Mixing {
            out.push(hit(n, "This `.format` string mixes automatic `{}` and numbered `{0}` fields"));
        }
        return;
    }
    let mut used_index: BTreeSet<usize> = BTreeSet::new();
    let mut next = 0;
    let mut names: BTreeSet<String> = BTreeSet::new();
    for f in &fields {
        match f {
            Arg::Auto => {
                used_index.insert(next);
                next += 1;
            }
            Arg::Index(i) => {
                used_index.insert(*i);
            }
            Arg::Name(x) => {
                names.insert(x.clone());
            }
        }
    }
    let given_kw: BTreeSet<&str> = args.keywords.iter().map(|k| k.0.as_str()).collect();
    match want {
        B::ExtraNamed if !has_kwstar => {
            let extra: Vec<&(String, Node, Node)> = args.keywords.iter().filter(|k| !names.contains(&k.0)).collect();
            if !extra.is_empty() {
                let pure = extra.iter().all(|k| is_pure(k.1));
                let edits = if pure { remove_items(&extra.iter().map(|k| k.2).collect::<Vec<_>>(), src) } else { Vec::new() };
                let list = extra.iter().map(|k| format!("`{}`", k.0)).collect::<Vec<_>>().join(", ");
                out.push(hit_fix(n, format!("The format string never uses the keyword argument(s) {list}"), edits));
            }
        }
        B::ExtraPositional if !has_star => {
            let unused: Vec<usize> = (0..args.positional.len()).filter(|i| !used_index.contains(i)).collect();
            if !unused.is_empty() {
                let trailing = unused.iter().enumerate().all(|(k, i)| *i == args.positional.len() - unused.len() + k);
                let items: Vec<Node> = unused.iter().map(|i| args.positional[*i]).collect();
                let pure = items.iter().all(|x| is_pure(*x));
                let edits = if trailing && pure { remove_items(&items, src) } else { Vec::new() };
                let list = unused.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", ");
                out.push(hit_fix(n, format!("The format string never uses the positional argument(s) at {list}"), edits));
            }
        }
        B::Missing => {
            let mut missing: Vec<String> = Vec::new();
            if !has_star {
                missing.extend(used_index.iter().filter(|i| **i >= args.positional.len()).map(|i| i.to_string()));
            }
            if !has_kwstar {
                missing.extend(names.iter().filter(|x| !given_kw.contains(x.as_str())).map(|x| format!("`{x}`")));
            }
            if !missing.is_empty() {
                out.push(hit(n, format!("No argument is given for the field(s) {}", missing.join(", "))));
            }
        }
        _ => {}
    }
}

macro_rules! brace_rule {
    ($ty:ident, $id:literal, $which:expr, $f:ident) => {
        fn $f(n: Node<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
            brace(n, ctx, out, $which)
        }
        py_rule!($ty, $id, ["call"], false, $f);
    };
}

brace_rule!(StringDotFormatInvalidFormat, "py/format-invalid", B::Invalid, b_invalid);
brace_rule!(StringDotFormatExtraNamedArguments, "py/format-unused-named-argument", B::ExtraNamed, b_named);
brace_rule!(StringDotFormatExtraPositionalArguments, "py/format-unused-positional-argument", B::ExtraPositional, b_positional);
brace_rule!(StringDotFormatMissingArguments, "py/format-missing-argument", B::Missing, b_missing);
brace_rule!(StringDotFormatMixingAutomatic, "py/format-mixed-numbering", B::Mixing, b_mixing);

#[cfg(test)]
mod tests {
    use crate::run;
    use codelaw_core::Language;

    fn count(rule: &dyn crate::NativeRule, src: &str) -> usize {
        run(rule, Language::Python, src).len()
    }

    #[test]
    fn percent_formats() {
        assert_eq!(count(&super::PercentFormatPositionalCountMismatch, "a = '%s %s' % (1,)\nb = '%s %%' % (1,)\nc = '%s %s' % 5\n"), 2);
        assert_eq!(count(&super::PercentFormatUnsupportedFormatCharacter, "a = '%y' % (1,)\n"), 1);
        assert_eq!(count(&super::PercentFormatInvalidFormat, "a = '%(x' % {'x': 1}\n"), 1);
        assert_eq!(count(&super::PercentFormatMissingArgument, "a = '%(x)s %(z)s' % {'x': 1}\n"), 1);
    }

    #[test]
    fn brace_formats() {
        assert_eq!(count(&super::StringDotFormatMissingArguments, "'{} {}'.format(1)\n'{x.y[0]!r:>{w}}'.format(x=1, w=3)\n"), 1);
        assert_eq!(count(&super::StringDotFormatMixingAutomatic, "'{0} {}'.format(1, 2)\n"), 1);
        assert_eq!(count(&super::StringDotFormatInvalidFormat, "'{'.format(1)\n'{{}}'.format()\n"), 1);
    }
}
