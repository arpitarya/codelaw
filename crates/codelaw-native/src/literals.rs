//! Rules about literals and the text between tokens: holes in arrays,
//! numbers that lose precision, irregular whitespace, a line break that
//! changes a call, and runs of spaces in a regular expression.

use crate::util::{children, line_of, named_children, text};
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::{Language, TextEdit};
use codelaw_lang::SyntaxNode;

const JS: &[Language] = &[Language::Javascript, Language::Jsx, Language::Typescript, Language::Tsx];

fn finding(start: usize, end: usize, message: impl Into<String>, edits: Vec<TextEdit>) -> NativeFinding {
    NativeFinding { start, end, message: message.into(), edits }
}

pub struct NoSparseArrays;

impl NativeRule for NoSparseArrays {
    fn id(&self) -> &'static str {
        "js/no-sparse-arrays"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["array"]
    }
    fn check(&self, node: SyntaxNode<'_>, _ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        let mut prev = "";
        for c in children(n) {
            if c.is_extra() {
                continue;
            }
            if c.kind() == "," && (prev == "[" || prev == ",") {
                out.push(finding(n.start_byte(), n.end_byte(), "Unexpected hole in an array literal", Vec::new()));
                return;
            }
            prev = c.kind();
        }
    }
}

pub struct NoLossOfPrecision;

/// Whether a numeric literal's value survives the trip through an f64.
pub fn loses_precision(literal: &str) -> bool {
    let t = literal.replace('_', "");
    let lower = t.to_ascii_lowercase();
    if lower.ends_with('n') {
        return false;
    }
    let radix = if let Some(d) = lower.strip_prefix("0x") {
        Some((16, d))
    } else if let Some(d) = lower.strip_prefix("0o") {
        Some((8, d))
    } else if let Some(d) = lower.strip_prefix("0b") {
        Some((2, d))
    } else if lower.len() > 1 && lower.starts_with('0') && lower.bytes().all(|b| b.is_ascii_digit()) && lower.bytes().all(|b| b < b'8') {
        Some((8, &lower[1..]))
    } else {
        None
    };
    if let Some((radix, digits)) = radix {
        return match u128::from_str_radix(digits, radix) {
            Ok(v) => (v as f64) as u128 != v,
            Err(_) => true,
        };
    }
    let (mantissa, exp) = match lower.split_once('e') {
        Some((m, e)) => (m, e.parse::<i64>().unwrap_or(0)),
        None => (lower.as_str(), 0),
    };
    let (int, frac) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let digits: String = format!("{int}{frac}");
    let leading = digits.bytes().take_while(|b| *b == b'0').count();
    let significant = digits[leading..].trim_end_matches('0');
    if significant.is_empty() {
        return false;
    }
    // The decimal exponent of the first significant digit.
    let exp10 = exp + int.len() as i64 - 1 - leading as i64;
    let Ok(value) = lower.parse::<f64>() else { return false };
    if !value.is_finite() {
        return true;
    }
    let printed = format!("{:.*e}", significant.len() - 1, value);
    let (m, e) = printed.split_once('e').expect("exponent form");
    let printed_digits = m.replace('.', "");
    let printed_digits = printed_digits.trim_end_matches('0');
    printed_digits != significant || e.parse::<i64>().unwrap_or(0) != exp10
}

impl NativeRule for NoLossOfPrecision {
    fn id(&self) -> &'static str {
        "js/no-loss-of-precision"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["number"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        if loses_precision(text(n, ctx.source)) {
            out.push(finding(n.start_byte(), n.end_byte(), "This number can't be represented exactly; it loses precision", Vec::new()));
        }
    }
}

pub struct NoIrregularWhitespace;

const IRREGULAR: &[char] = &[
    '\u{000B}', '\u{000C}', '\u{0085}', '\u{00A0}', '\u{180E}', '\u{FEFF}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}',
    '\u{2005}', '\u{2006}', '\u{2007}', '\u{2008}', '\u{2009}', '\u{200A}', '\u{200B}', '\u{2028}', '\u{2029}', '\u{202F}', '\u{205F}',
    '\u{3000}',
];

impl NativeRule for NoIrregularWhitespace {
    fn id(&self) -> &'static str {
        "js/no-irregular-whitespace"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["program"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        // Strings are skipped; in a template, regex or JSX text the character
        // is part of a value, so it is reported but not rewritten.
        let mut skip = Vec::new();
        let mut keep = Vec::new();
        for n in node.dfs() {
            let r = n.range();
            match n.kind().as_ref() {
                "string" => skip.push(r),
                "template_string" | "regex" | "jsx_text" => keep.push(r),
                _ => {}
            }
        }
        for (i, c) in ctx.source.char_indices() {
            if !IRREGULAR.contains(&c) || skip.iter().any(|r| r.contains(&i)) {
                continue;
            }
            let end = i + c.len_utf8();
            let edits = if keep.iter().any(|r| r.contains(&i)) { Vec::new() } else { vec![TextEdit::new(i, end, " ")] };
            out.push(finding(i, end, format!("Irregular whitespace U+{:04X}", c as u32), edits));
        }
    }
}

pub struct NoUnexpectedMultiline;

impl NativeRule for NoUnexpectedMultiline {
    fn id(&self) -> &'static str {
        "js/no-unexpected-multiline"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["call_expression", "subscript_expression"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        let src = ctx.source;
        // A parse error inside means tree-sitter recovered by guessing; an
        // arrow function's body can't be called or indexed without parentheses,
        // so JavaScript inserts the semicolon there itself.
        if n.has_error() || children(n).iter().any(|c| c.kind() == "?." || c.kind() == "optional_chain") {
            return;
        }
        let (head, tail, what) = match n.kind() {
            "call_expression" => {
                let (Some(f), Some(a)) = (n.child_by_field_name("function"), n.child_by_field_name("arguments")) else { return };
                let what = if a.kind() == "template_string" { "a tagged template" } else { "a function call" };
                (f, a, what)
            }
            _ => {
                let Some(o) = n.child_by_field_name("object") else { return };
                let Some(open) = children(n).into_iter().find(|c| c.kind() == "[") else { return };
                (o, open, "a property access")
            }
        };
        if head.kind() == "arrow_function" {
            return;
        }
        if line_of(src, tail.start_byte()) > line_of(src, head.end_byte()) {
            out.push(finding(
                tail.start_byte(),
                tail.start_byte() + 1,
                format!("This line break joins two lines into {what}; end the first line with a semicolon if that isn't meant"),
                Vec::new(),
            ));
        }
    }
}

pub struct NoRegexSpaces;

impl NativeRule for NoRegexSpaces {
    fn id(&self) -> &'static str {
        "js/no-regex-spaces"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["regex_pattern"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        let base = n.start_byte();
        let bytes = text(n, ctx.source).as_bytes();
        let (mut i, mut in_class, mut escaped) = (0, false, false);
        while i < bytes.len() {
            let b = bytes[i];
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'[' {
                in_class = true;
            } else if b == b']' {
                in_class = false;
            } else if b == b' ' && !in_class {
                let run = bytes[i..].iter().take_while(|c| **c == b' ').count();
                let quantified = bytes.get(i + run).is_some_and(|c| matches!(c, b'*' | b'+' | b'?' | b'{'));
                if run >= 2 && !quantified {
                    out.push(finding(base + i, base + i + run, format!("{run} spaces are hard to count; write ` {{{run}}}`"), vec![TextEdit::new(base + i, base + i + run, format!(" {{{run}}}"))]));
                }
                i += run;
                continue;
            }
            i += 1;
        }
    }
}

pub struct NoEmptyCharacterClass;

impl NativeRule for NoEmptyCharacterClass {
    fn id(&self) -> &'static str {
        "js/no-empty-character-class"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["regex_pattern"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        let bytes = text(n, ctx.source).as_bytes();
        let (mut i, mut in_class, mut escaped) = (0, false, false);
        while i < bytes.len() {
            let b = bytes[i];
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if in_class {
                in_class = b != b']';
            } else if b == b'[' {
                // `[]` matches nothing; `[^]` matches any character and is fine.
                if bytes.get(i + 1) == Some(&b']') {
                    let at = n.start_byte() + i;
                    out.push(finding(at, at + 2, "`[]` matches nothing, so this pattern never matches", Vec::new()));
                    i += 2;
                    continue;
                }
                in_class = true;
                if bytes.get(i + 1) == Some(&b'^') {
                    i += 1;
                }
                // A `]` first in a class is a literal in some engines; JS
                // treats `[]` as empty, which the check above covers.
            }
            i += 1;
        }
    }
}

pub struct NoFallthrough;

impl NativeRule for NoFallthrough {
    fn id(&self) -> &'static str {
        "js/no-fallthrough"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["switch_body"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let cases: Vec<_> = named_children(node.get_inner_node()).into_iter().filter(|c| matches!(c.kind(), "switch_case" | "switch_default")).collect();
        let marker = regex::Regex::new(r"(?i)falls?\s?through").expect("a valid pattern");
        for pair in cases.windows(2) {
            let (case, next) = (pair[0], pair[1]);
            let value_end = case.child_by_field_name("value").map(|v| v.end_byte()).unwrap_or(case.start_byte());
            let body: Vec<_> = named_children(case).into_iter().filter(|s| s.start_byte() >= value_end && s.kind() != "comment" && Some(*s) != case.child_by_field_name("value")).collect();
            let Some(last) = body.last() else { continue };
            if crate::util::exits(last) {
                continue;
            }
            // A `// falls through` comment between the two cases says it's meant.
            if marker.is_match(&ctx.source[last.end_byte()..next.start_byte()]) {
                continue;
            }
            let kw_end = next.start_byte() + text(next, ctx.source).find([' ', ':']).unwrap_or(4);
            out.push(finding(next.start_byte(), kw_end, "The case above falls through into this one; end it with `break`, or say `// falls through`", Vec::new()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(rule: &dyn NativeRule, src: &str) -> Vec<String> {
        crate::run(rule, Language::Javascript, src).into_iter().map(|f| src[f.start..f.end].to_owned()).collect()
    }

    #[test]
    fn precision() {
        for lossy in ["9007199254740993", "5123000000000000000000000000001", "0x20000000000001", "1.230000000000000000000000001", "2e999"] {
            assert!(loses_precision(lossy), "{lossy}");
        }
        for exact in ["12345", "123.456", "0.1", "1e10", "0x1F", "1_000_000", "9007199254740992", "0.00001", "123n", "1.5e-7", "0"] {
            assert!(!loses_precision(exact), "{exact}");
        }
    }

    #[test]
    fn sparse_whitespace_multiline() {
        assert_eq!(found(&NoSparseArrays, "a = [1,, 2]; b = [, 1]; c = [1, 2,];"), ["[1,, 2]", "[, 1]"]);
        let src = "let\u{00A0}x = 1; const s = \"a\u{00A0}b\";";
        assert_eq!(crate::run(&NoIrregularWhitespace, Language::Javascript, src).len(), 1);
        assert_eq!(found(&NoUnexpectedMultiline, "let a = b\n(c || d).go();\nlet e = f\n[1, 2].map(g);\nlet ok = h(\n  1);\n"), ["(", "["]);
    }

    #[test]
    fn empty_character_classes() {
        assert_eq!(found(&NoEmptyCharacterClass, r"a = /^abc[]/; b = /^\s*[{[]x/; c = /[^]/; d = /\[\]/; e = /[\]]/;"), ["[]"]);
    }

    #[test]
    fn regex_spaces_and_fallthrough() {
        let r = crate::run(&NoRegexSpaces, Language::Javascript, "/a   b/; /[a   b]/; /c  +/;");
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].edits[0].text, " {3}");
        let src = "switch (x) {\n  case 1: a();\n  case 2: b(); break;\n  case 3: c();\n  // falls through\n  case 4:\n  case 5: d();\n  default: e();\n}";
        assert_eq!(found(&NoFallthrough, src), ["case", "default"]);
    }
}
