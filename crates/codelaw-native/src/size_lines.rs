//! `css/max-lines` and `html/max-lines` (W-61): lines per stylesheet and per
//! page, with the same options shape as `js/max-lines` and `py/max-lines`.
//!
//! Counting needs no tree, only a scan that knows where comments, strings and
//! (in HTML) `<script>`/`<style>` bodies are. A line holding code and a
//! comment counts as code. The rules measure the host file only: a `<style>`
//! element in a page, or a fenced block in Markdown, is a region, not a file,
//! so a rule run on a region does nothing.

use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::Language;
use codelaw_lang::SyntaxNode;
use serde_json::Value;
use std::collections::BTreeMap;

/// Which language a [`MaxLines`] counts.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Css,
    Html,
}

pub struct MaxLines {
    id: &'static str,
    lang: Lang,
}

impl MaxLines {
    pub const fn css(id: &'static str) -> MaxLines {
        MaxLines { id, lang: Lang::Css }
    }
    pub const fn html(id: &'static str) -> MaxLines {
        MaxLines { id, lang: Lang::Html }
    }
}

impl NativeRule for MaxLines {
    fn id(&self) -> &'static str {
        self.id
    }
    fn languages(&self) -> &'static [Language] {
        match self.lang {
            Lang::Css => &[Language::Css, Language::Scss],
            Lang::Html => &[Language::Html],
        }
    }
    fn kinds(&self) -> &'static [&'static str] {
        match self.lang {
            // `ERROR` is the root when tree-sitter-scss cannot recover.
            Lang::Css => &["stylesheet", "ERROR"],
            Lang::Html => &["document"],
        }
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        if node.parent().is_some() {
            return;
        }
        let ext = file_ext(ctx.file.path());
        let scss = ext == "scss";
        let whole_file = match self.lang {
            Lang::Css => matches!(ext.as_str(), "css" | "scss"),
            Lang::Html => matches!(ext.as_str(), "html" | "htm"),
        };
        // A region of another file (a `<style>` in a page, a fence in
        // Markdown) is not a file.
        if !whole_file {
            return;
        }
        let o = Opts::of(ctx.options, self.lang);
        let lines = match self.lang {
            Lang::Css => classify_css(ctx.source, scss),
            Lang::Html => classify_html(ctx.source),
        };
        if let Some(f) = measure(ctx.source, &lines, &o) {
            out.push(f);
        }
    }
    /// The measured value and the limit, parsed back from the message the
    /// rule wrote, for `--format json` and SARIF.
    fn data_of(&self, finding: &NativeFinding, _options: &BTreeMap<String, Value>) -> Option<Value> {
        let (head, tail) = finding.message.split_once(" lines (max ")?;
        let lines: u64 = head.parse().ok()?;
        let max: u64 = tail.chars().take_while(char::is_ascii_digit).collect::<String>().parse().ok()?;
        Some(serde_json::json!({ "lines": lines, "max": max }))
    }
}

fn file_ext(path: &str) -> String {
    path.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase()).unwrap_or_default()
}

// ---- options ----

struct Opts {
    max: usize,
    skip_blank_lines: bool,
    skip_comments: bool,
    skip_imports: bool,
    skip_embedded: bool,
}

impl Opts {
    fn of(o: &BTreeMap<String, Value>, lang: Lang) -> Opts {
        let flag = |k: &str, d: bool| o.get(k).and_then(Value::as_bool).unwrap_or(d);
        let default_max = match lang {
            Lang::Css => 300,
            Lang::Html => 400,
        };
        Opts {
            max: o.get("max").and_then(Value::as_u64).map_or(default_max, |v| v as usize),
            skip_blank_lines: flag("skip_blank_lines", true),
            skip_comments: flag("skip_comments", true),
            skip_imports: flag("skip_imports", false),
            skip_embedded: flag("skip_embedded", false),
        }
    }
}

// ---- line classes ----

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Class {
    Blank,
    Comment,
    Import,
    /// HTML: a line inside a `<script>` or `<style>` body and nothing else.
    Embedded,
    Code,
}

/// Lines of `src` as byte ranges, without their line break; a final line
/// break does not start a line.
fn line_ranges(src: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start = 0;
    for (i, b) in src.bytes().enumerate() {
        if b == b'\n' {
            out.push((start, i));
            start = i + 1;
        }
    }
    if start < src.len() {
        out.push((start, src.len()));
    }
    out
}

/// What one line holds, filled by the scanners.
#[derive(Clone, Copy, Default)]
struct Marks {
    code: bool,
    comment: bool,
    embedded: bool,
}

/// The line index of every byte offset, by walking the line starts once.
struct Lines {
    marks: Vec<Marks>,
    starts: Vec<usize>,
}

impl Lines {
    fn new(src: &str) -> Lines {
        let ranges = line_ranges(src);
        Lines { marks: vec![Marks::default(); ranges.len()], starts: ranges.iter().map(|r| r.0).collect() }
    }
    fn line_of(&self, at: usize) -> usize {
        match self.starts.binary_search(&at) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        }
    }
    /// Mark the non-blank bytes of `src[from..to]`.
    fn mark(&mut self, src: &str, from: usize, to: usize, f: impl Fn(&mut Marks)) {
        for (off, ch) in src[from..to].char_indices() {
            if !ch.is_whitespace() {
                let i = self.line_of(from + off);
                if i < self.marks.len() {
                    f(&mut self.marks[i]);
                }
            }
        }
    }
}

fn blank_or(src: &str, range: (usize, usize), m: Marks) -> Class {
    if src[range.0..range.1].trim().is_empty() {
        Class::Blank
    } else if m.code {
        Class::Code
    } else if m.embedded {
        Class::Embedded
    } else if m.comment {
        Class::Comment
    } else {
        Class::Code
    }
}

// ---- CSS and SCSS ----

fn classify_css(src: &str, scss: bool) -> Vec<Class> {
    let mut lines = Lines::new(src);
    let b = src.as_bytes();
    let mut i = 0;
    let mut code_from: Option<usize> = None;
    let flush = |lines: &mut Lines, code_from: &mut Option<usize>, upto: usize| {
        if let Some(s) = code_from.take() {
            lines.mark(src, s, upto, |m| m.code = true);
        }
    };
    while i < b.len() {
        match b[i] {
            b'/' if b.get(i + 1) == Some(&b'*') => {
                flush(&mut lines, &mut code_from, i);
                let e = src[i + 2..].find("*/").map_or(src.len(), |p| i + 2 + p + 2);
                lines.mark(src, i, e, |m| m.comment = true);
                i = e;
            }
            b'/' if scss && b.get(i + 1) == Some(&b'/') => {
                flush(&mut lines, &mut code_from, i);
                let e = src[i..].find('\n').map_or(src.len(), |p| i + p);
                lines.mark(src, i, e, |m| m.comment = true);
                i = e;
            }
            q @ (b'"' | b'\'') => {
                code_from.get_or_insert(i);
                i += 1;
                while i < b.len() && b[i] != q && b[i] != b'\n' {
                    i += if b[i] == b'\\' { 2 } else { 1 };
                }
                i = (i + 1).min(b.len());
            }
            b'u' | b'U' if src[i..].len() >= 4 && src[i..i + 4].eq_ignore_ascii_case("url(") && !src[..i].ends_with(|c: char| c.is_alphanumeric() || c == '-' || c == '_') => {
                // `url(http://x/y)`: an unquoted URL holds `//` and `/*`.
                code_from.get_or_insert(i);
                i += 4;
                let rest = src[i..].trim_start();
                if !rest.starts_with(['"', '\'']) {
                    i = src[i..].find([')', '\n']).map_or(src.len(), |p| i + p);
                }
            }
            c => {
                if !c.is_ascii_whitespace() {
                    code_from.get_or_insert(i);
                } else if c == b'\n' {
                    flush(&mut lines, &mut code_from, i);
                }
                i += 1;
            }
        }
    }
    flush(&mut lines, &mut code_from, src.len());
    let ranges = line_ranges(src);
    let mut out = Vec::with_capacity(ranges.len());
    let mut in_import = false;
    for (n, &r) in ranges.iter().enumerate() {
        let mut class = blank_or(src, r, lines.marks[n]);
        if class == Class::Code {
            let t = src[r.0..r.1].trim_start();
            if in_import || ["@import", "@use", "@forward"].iter().any(|k| t.len() >= k.len() && t[..k.len()].eq_ignore_ascii_case(k) && !t[k.len()..].starts_with(|c: char| c.is_alphanumeric() || c == '-')) {
                class = Class::Import;
                in_import = !lines_ends_statement(&src[r.0..r.1]);
            }
        }
        out.push(class);
    }
    out
}

/// Whether the line ends the `@import`/`@use` statement it is part of.
fn lines_ends_statement(line: &str) -> bool {
    // Comments are stripped by the scanner only for classes, not text, so
    // look at the code before any comment start.
    let code = line.split("/*").next().unwrap_or(line);
    let code = code.split("//").next().filter(|c| !c.contains("://")).unwrap_or(code);
    code.trim_end().ends_with(';')
}

// ---- HTML ----

fn classify_html(src: &str) -> Vec<Class> {
    let mut lines = Lines::new(src);
    let b = src.as_bytes();
    let mut i = 0;
    let mut text_from: Option<usize> = None;
    let flush = |lines: &mut Lines, text_from: &mut Option<usize>, upto: usize| {
        if let Some(s) = text_from.take() {
            lines.mark(src, s, upto, |m| m.code = true);
        }
    };
    while i < b.len() {
        if src[i..].starts_with("<!--") {
            flush(&mut lines, &mut text_from, i);
            let e = src[i + 4..].find("-->").map_or(src.len(), |p| i + 4 + p + 3);
            lines.mark(src, i, e, |m| m.comment = true);
            i = e;
            continue;
        }
        if b[i] == b'<' && let Some(name) = raw_open(src, i) {
            // The open tag is code; the body up to the close tag is embedded.
            text_from.get_or_insert(i);
            let tag_end = open_tag_end(src, i);
            flush(&mut lines, &mut text_from, tag_end);
            let close = find_close(src, tag_end, name);
            lines.mark(src, tag_end, close, |m| m.embedded = true);
            i = close;
            continue;
        }
        if b[i] == b'\n' {
            flush(&mut lines, &mut text_from, i);
        } else if !b[i].is_ascii_whitespace() {
            text_from.get_or_insert(i);
        }
        i += 1;
    }
    flush(&mut lines, &mut text_from, src.len());
    line_ranges(src).iter().enumerate().map(|(n, &r)| blank_or(src, r, lines.marks[n])).collect()
}

/// `script` or `style` when an open tag of that name starts at `at`.
fn raw_open(src: &str, at: usize) -> Option<&'static str> {
    let rest = &src[at + 1..];
    ["script", "style"].into_iter().find(|name| {
        rest.len() > name.len() && rest[..name.len()].eq_ignore_ascii_case(name) && (rest.as_bytes()[name.len()].is_ascii_whitespace() || matches!(rest.as_bytes()[name.len()], b'>' | b'/'))
    })
}

/// The offset just after the `>` that ends the open tag at `at`, quotes
/// respected.
fn open_tag_end(src: &str, at: usize) -> usize {
    let b = src.as_bytes();
    let mut i = at + 1;
    let mut quote = 0u8;
    while i < b.len() {
        match (quote, b[i]) {
            (0, b'"' | b'\'') => quote = b[i],
            (q, c) if q != 0 && c == q => quote = 0,
            (0, b'>') => return i + 1,
            _ => {}
        }
        i += 1;
    }
    src.len()
}

/// The offset of the `</name` that closes a raw element, or the end.
fn find_close(src: &str, from: usize, name: &str) -> usize {
    let lower = src[from..].to_ascii_lowercase();
    let needle = format!("</{name}");
    lower.find(&needle).map_or(src.len(), |p| from + p)
}

// ---- counting ----

fn measure(src: &str, classes: &[Class], o: &Opts) -> Option<NativeFinding> {
    let ranges = line_ranges(src);
    let mut count = 0;
    let mut first_over = None;
    for (n, class) in classes.iter().enumerate() {
        let skipped = match class {
            Class::Blank => o.skip_blank_lines,
            Class::Comment => o.skip_comments,
            Class::Import => o.skip_imports,
            Class::Embedded => o.skip_embedded,
            Class::Code => false,
        };
        if skipped {
            continue;
        }
        count += 1;
        if count == o.max + 1 {
            first_over = Some(n);
        }
    }
    let n = first_over?;
    let mut not_counted = Vec::new();
    if o.skip_blank_lines {
        not_counted.push("blank lines");
    }
    if o.skip_comments {
        not_counted.push("comments");
    }
    if o.skip_imports {
        not_counted.push("imports");
    }
    if o.skip_embedded {
        not_counted.push("embedded script and style");
    }
    let tail = match not_counted.as_slice() {
        [] => String::new(),
        [a] => format!("; {a} not counted"),
        [a, b] => format!("; {a} and {b} not counted"),
        [rest @ .., last] => format!("; {} and {last} not counted", rest.join(", ")),
    };
    let (s, e) = ranges[n];
    Some(NativeFinding {
        start: s,
        end: e.max(s),
        message: format!("{count} lines (max {}{tail}); split it into smaller files", o.max),
        edits: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn run(rule: &MaxLines, path: &str, src: &str, opts: Value) -> Vec<NativeFinding> {
        let options: BTreeMap<String, Value> = opts.as_object().unwrap().iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        let lines = match rule.lang {
            Lang::Css => classify_css(src, path.ends_with(".scss")),
            Lang::Html => classify_html(src),
        };
        let o = Opts::of(&options, rule.lang);
        measure(src, &lines, &o).into_iter().collect()
    }

    fn css(src: &str, scss: bool, opts: Value) -> Vec<NativeFinding> {
        run(&MaxLines::css("css/max-lines"), if scss { "a.scss" } else { "a.css" }, src, opts)
    }

    fn html(src: &str, opts: Value) -> Vec<NativeFinding> {
        run(&MaxLines::html("html/max-lines"), "a.html", src, opts)
    }

    #[test]
    fn css_blank_and_comment_lines() {
        let src = "a {\n\n  top: 0;\n  /* note */\n}\n";
        assert!(css(src, false, json!({ "max": 3 })).is_empty());
        let f = css(src, false, json!({ "max": 2 }));
        assert_eq!(f.len(), 1);
        assert_eq!(&src[f[0].start..f[0].end], "}");
        assert!(f[0].message.starts_with("3 lines (max 2; blank lines and comments not counted)"));
        assert_eq!(css(src, false, json!({ "max": 4, "skip_blank_lines": false })).len(), 0);
        assert_eq!(css(src, false, json!({ "max": 3, "skip_blank_lines": false })).len(), 1);
        assert_eq!(css(src, false, json!({ "max": 3, "skip_comments": false })).len(), 1);
    }

    #[test]
    fn css_multiline_comment_and_mixed_line() {
        let src = "/*\n one\n two\n*/\na { top: 0 } /* tail */\n";
        assert!(css(src, false, json!({ "max": 1 })).is_empty());
        assert_eq!(css(src, false, json!({ "max": 1, "skip_comments": false })).len(), 1);
    }

    #[test]
    fn scss_double_slash_comments() {
        let src = "// one\n// two\na { top: 0 }\n";
        assert!(css(src, true, json!({ "max": 1 })).is_empty());
        // In plain CSS `//` is not a comment.
        assert_eq!(css(src, false, json!({ "max": 1 })).len(), 1);
        let url = "a { background: url(http://x.test/a.png) }\nb { top: 0 }\n";
        assert_eq!(css(url, true, json!({ "max": 1 })).len(), 1);
    }

    #[test]
    fn css_imports() {
        let src = "@use \"a\";\n@use \"b\" with (\n  $x: 1\n);\n@import url(x.css);\na { top: 0 }\n";
        assert_eq!(css(src, true, json!({ "max": 1 })).len(), 1);
        assert!(css(src, true, json!({ "max": 1, "skip_imports": true })).is_empty());
        assert_eq!(css(src, true, json!({ "max": 1, "skip_imports": true })).len(), 0);
    }

    #[test]
    fn html_comments_and_blank() {
        let src = "<!-- a\n b -->\n\n<p>x</p>\n";
        assert!(html(src, json!({ "max": 1 })).is_empty());
        assert_eq!(html(src, json!({ "max": 1, "skip_comments": false })).len(), 1);
    }

    #[test]
    fn html_embedded_script_and_style() {
        let src = "<script>\nlet a = 1;\nlet b = 2;\n</script>\n<style>\na { top: 0 }\n</style>\n";
        // 7 lines, 3 of them tags... 2 + 2 = 4 tag lines, 3 body lines.
        assert_eq!(html(src, json!({ "max": 4 })).len(), 1);
        assert!(html(src, json!({ "max": 4, "skip_embedded": true })).is_empty());
        assert_eq!(html(src, json!({ "max": 3, "skip_embedded": true })).len(), 1);
        let one = "<script>x()</script>\n<p></p>\n";
        assert_eq!(html(one, json!({ "max": 1, "skip_embedded": true })).len(), 1);
    }

    #[test]
    fn defaults() {
        let big = "a { top: 0 }\n".repeat(301);
        assert_eq!(css(&big, false, json!({})).len(), 1);
        assert!(css(&"a { top: 0 }\n".repeat(300), false, json!({})).is_empty());
        let page = "<p>x</p>\n".repeat(401);
        assert_eq!(html(&page, json!({})).len(), 1);
        assert!(html(&"<p>x</p>\n".repeat(400), json!({})).is_empty());
    }

    #[test]
    fn data_carries_the_count_and_the_limit() {
        let src = "a {\n  top: 0;\n  left: 0;\n}\n";
        for rule in [MaxLines::css("css/max-lines"), MaxLines::html("html/max-lines")] {
            let path = if rule.lang == Lang::Css { "a.css" } else { "a.html" };
            let f = run(&rule, path, src, json!({ "max": 2 }));
            assert_eq!(rule.data_of(&f[0], &BTreeMap::new()), Some(json!({ "lines": 4, "max": 2 })));
        }
        let other = NativeFinding { start: 0, end: 0, message: "x".into(), edits: Vec::new() };
        assert_eq!(MaxLines::css("css/max-lines").data_of(&other, &BTreeMap::new()), None);
    }
}
