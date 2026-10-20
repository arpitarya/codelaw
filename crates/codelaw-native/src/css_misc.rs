//! CSS rules about the sheet as text: empty blocks, comments and sources,
//! `//` comments, irregular whitespace.

use crate::css_syntax::{Kind, css_rule, finding, fixed, skip_comment, skip_string, text};
use crate::util::whole_lines;
use codelaw_core::TextEdit;

css_rule!(NoEmptyBlock, "css/no-empty-block", |sheet, src, ctx, out| {
    for i in sheet.walk() {
        let n = &sheet.nodes[i];
        let Some((open, close)) = n.block else { continue };
        if !n.children.is_empty() || !src[open + 1..close.min(src.len())].trim().is_empty() {
            continue;
        }
        // `@layer name {}` fixes the layer's place in the cascade order.
        if n.kind == Kind::AtRule && sheet.at_name(i, src) == "layer" && !text(src, n.params).is_empty() {
            continue;
        }
        let (s, e) = whole_lines(src, n.start, n.end);
        out.push(fixed(n.start, n.end, "Empty block; remove it or give it declarations", vec![TextEdit::delete(s, e)]));
    }
});

css_rule!(CommentNoEmpty, "css/comment-no-empty", |sheet, src, ctx, out| {
    for i in sheet.walk() {
        let n = &sheet.nodes[i];
        if n.kind == Kind::Comment && text(src, n.params).trim().is_empty() && text(src, (n.start, n.end)).ends_with("*/") {
            let (s, e) = whole_lines(src, n.start, n.end);
            out.push(fixed(n.start, n.end, "Empty comment", vec![TextEdit::delete(s, e)]));
        }
    }
    // Comments inside values and selectors.
    let b = src.as_bytes();
    let mut i = 0;
    let statement_comments: Vec<usize> = sheet.walk().into_iter().filter(|&i| sheet.nodes[i].kind == Kind::Comment).map(|i| sheet.nodes[i].start).collect();
    while i < b.len() {
        match b[i] {
            b'"' | b'\'' => i = skip_string(b, i),
            b'/' if b.get(i + 1) == Some(&b'*') => {
                let e = skip_comment(src, i);
                if !statement_comments.contains(&i) && e - i >= 4 && src[i + 2..e - 2].trim().is_empty() {
                    out.push(fixed(i, e, "Empty comment", vec![TextEdit::delete(i, e)]));
                }
                i = e;
            }
            _ => i += 1,
        }
    }
});

css_rule!(NoEmptySource, "css/no-empty-source", |sheet, src, ctx, out| {
    if src.trim().is_empty() {
        out.push(finding(0, src.len(), "Empty stylesheet"));
    }
});

css_rule!(NoInvalidDoubleSlashComments, "css/no-invalid-double-slash-comments", |sheet, src, ctx, out| {
    for i in sheet.walk() {
        let n = &sheet.nodes[i];
        let starts = match n.kind {
            Kind::Decl | Kind::Rule | Kind::Other => text(src, n.name).starts_with("//") || (n.kind == Kind::Other && text(src, (n.start, n.end)).starts_with("//")),
            _ => false,
        };
        if starts {
            let line_end = src[n.start..n.end].find('\n').map_or(n.end, |o| n.start + o);
            let what = if n.kind == Kind::Rule { "the rule after it" } else { "this declaration" };
            out.push(finding(n.start, line_end.max(n.start + 2), format!("`//` does not start a comment in CSS; it breaks {what}. Use `/* … */`")));
        }
    }
});

const IRREGULAR: &[char] = &[
    '\u{000B}', '\u{000C}', '\u{00A0}', '\u{0085}', '\u{1680}', '\u{180E}', '\u{FEFF}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}',
    '\u{2006}', '\u{2007}', '\u{2008}', '\u{2009}', '\u{200A}', '\u{200B}', '\u{2028}', '\u{2029}', '\u{202F}', '\u{205F}', '\u{3000}',
];

css_rule!(NoIrregularWhitespace, "css/no-irregular-whitespace", |sheet, src, ctx, out| {
    let b = src.as_bytes();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'"' | b'\'' => i = skip_string(b, i),
            b'/' if b.get(i + 1) == Some(&b'*') => i = skip_comment(src, i),
            c if c < 0x80 && c != 0x0B && c != 0x0C => i += 1,
            _ => {
                let ch = src[i..].chars().next().unwrap_or(' ');
                let len = ch.len_utf8();
                // A byte-order mark at the very start is an encoding marker.
                if IRREGULAR.contains(&ch) && !(i == 0 && ch == '\u{FEFF}') {
                    out.push(fixed(i, i + len, format!("Irregular whitespace U+{:04X}; use a plain space", ch as u32), vec![TextEdit::new(i, i + len, " ")]));
                }
                i += len;
            }
        }
    }
});

#[cfg(test)]
mod tests {
    use super::*;
    use codelaw_core::Language;

    fn spans(rule: &dyn crate::NativeRule, src: &str) -> Vec<String> {
        crate::run(rule, Language::Css, src).into_iter().map(|f| src[f.start..f.end].to_owned()).collect()
    }

    #[test]
    fn empty_blocks() {
        assert_eq!(spans(&NoEmptyBlock, "a {}\nb { /* x */ }\n@media print {}\n@layer base {}\nc { top: 0 }"), ["a {}", "@media print {}"]);
    }

    #[test]
    fn comments() {
        assert_eq!(spans(&CommentNoEmpty, "/**/\n/* x */\na { top: 0 /* */; }"), ["/**/", "/* */"]);
        assert_eq!(spans(&NoInvalidDoubleSlashComments, "// a\nb { // c: d;\n top: 0 }"), ["// a", "// c: d"]);
        assert_eq!(spans(&NoIrregularWhitespace, ".a\u{00A0}.b { content: \"\u{00A0}\" }"), ["\u{00A0}"]);
    }
}
