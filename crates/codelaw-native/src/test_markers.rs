//! Focused and skipped tests (W-131; SR-CONFIG decision 11a).
//!
//! `js/no-focused-tests`: `.only`, `fit`, `fdescribe` leave the rest of the
//! suite unrun. `js/skipped-test` and `py/skipped-test`: a skipped test
//! carries a reason and an expiry — the framework's own reason argument where
//! it has one, else the text of a `// SKIP(ABC-123, 2027-01-31): reason`
//! comment directly above (or at the end of the line); `[tests.skip]` says
//! which of reason, expiry and ticket are required. A `SKIP` date is judged
//! like a TODO's: expired from the as-of date on.

use crate::markers::{self, Marker, Options};
use crate::util::{named_children, text};
use crate::{NativeContext, NativeFinding, NativeRule};
use codelaw_core::{Language, Severity, TextEdit};
use codelaw_lang::SyntaxNode;
use tree_sitter::Node;

const JS: &[Language] = &[Language::Javascript, Language::Jsx, Language::Typescript, Language::Tsx];
const PY: &[Language] = &[Language::Python];
const GROUPS: &[&str] = &["describe", "it", "test", "context", "suite", "specify", "bench"];

fn finding(start: usize, end: usize, message: String, edits: Vec<TextEdit>) -> NativeFinding {
    NativeFinding { start, end, message, edits }
}

/// A callee made only of identifiers and `.member`s: its segments, root
/// first, each with the byte range from the end of what precedes it (the `.`
/// included) to its own end. `None` for anything else.
fn path(n: Node<'_>, src: &str) -> Option<Vec<(String, usize, usize)>> {
    match n.kind() {
        "identifier" => Some(vec![(text(n, src).to_owned(), n.start_byte(), n.end_byte())]),
        "member_expression" => {
            let mut p = path(n.child_by_field_name("object")?, src)?;
            let prop = n.child_by_field_name("property")?;
            let from = p.last()?.2;
            p.push((text(prop, src).to_owned(), from, prop.end_byte()));
            Some(p)
        }
        _ => None,
    }
}

// ------------------------------------------------------------ focused tests

pub struct NoFocusedTests;

impl NativeRule for NoFocusedTests {
    fn id(&self) -> &'static str {
        "js/no-focused-tests"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["call_expression"]
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        let Some(callee) = n.child_by_field_name("function") else { return };
        let Some(p) = path(callee, ctx.source) else { return };
        let root = p[0].0.as_str();
        if p.len() == 1 {
            let plain = match root {
                "fit" => "it",
                "fdescribe" => "describe",
                "ftest" => "test",
                _ => return,
            };
            let edit = TextEdit::new(p[0].1, p[0].2, plain);
            out.push(finding(callee.start_byte(), callee.end_byte(), format!("`{root}` runs only this test: use `{plain}`"), vec![edit]));
            return;
        }
        if !GROUPS.contains(&root) {
            return;
        }
        if let Some((_, from, to)) = p[1..].iter().find(|s| s.0 == "only") {
            let shown = ctx.source[callee.start_byte()..callee.end_byte()].to_owned();
            out.push(finding(callee.start_byte(), callee.end_byte(), format!("`{shown}` runs only this test and skips the rest of the suite: drop `.only`"), vec![TextEdit::new(*from, *to, "")]));
        }
    }
}

// ------------------------------------------------------------ skipped tests

/// Lines directly above `start` that are comments, nearest first, as
/// `(byte offset of the line, the line)`.
fn comment_run_above(src: &str, start: usize) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let first = src[..start].rfind('\n').map_or(0, |o| o + 1);
    if first == 0 {
        return out;
    }
    let mut end = first - 1;
    loop {
        let line_start = src[..end].rfind('\n').map_or(0, |o| o + 1);
        let line = src[line_start..end].trim_end_matches('\r');
        let t = line.trim_start();
        if !(t.starts_with("//") || t.starts_with("/*") || t.starts_with('*') || t.starts_with('#')) {
            break;
        }
        out.push((line_start, line));
        if line_start == 0 {
            break;
        }
        end = line_start - 1;
    }
    out
}

/// The `SKIP(…)` marker that goes with a statement starting at `stmt`: in the
/// comment lines directly above it, or in a comment on its first line.
fn skip_marker(src: &str, stmt: usize, trailing: &str, ticket: &regex::Regex) -> Option<Marker> {
    let tags = ["SKIP".to_owned()];
    for (base, line) in comment_run_above(src, stmt) {
        if let Some(m) = markers::parse_line(base, line, &tags, ticket) {
            return Some(m);
        }
    }
    let line_start = src[..stmt].rfind('\n').map_or(0, |o| o + 1);
    let line_end = src[stmt..].find('\n').map_or(src.len(), |o| stmt + o);
    let line = &src[line_start..line_end];
    let from = stmt - line_start;
    let at = line[from..].find(trailing).map(|o| from + o)?;
    markers::parse_line(line_start + at, line[at..].trim_end_matches('\r'), &tags, ticket)
}

/// What a skipped test is missing, and the dated findings of its marker.
fn judge(
    what: &str,
    leader: &str,
    at: (usize, usize),
    reason_given: bool,
    marker: Option<&Marker>,
    ctx: &NativeContext<'_>,
    out: &mut Vec<NativeFinding>,
) {
    let o: &Options = ctx.options;
    let ticket_re = match markers::ticket_regex(o) {
        Ok(r) => r,
        Err(p) => {
            out.push(finding(at.0, at.1, format!("`[tests.skip] ticket_pattern` `{p}` is not a regular expression"), Vec::new()));
            return;
        }
    };
    let _ = ticket_re;
    let pattern = o.get("ticket_pattern").and_then(|v| v.as_str()).unwrap_or(markers::DEFAULT_TICKET_PATTERN);
    let mut missing = Vec::new();
    if markers::opt_bool(o, "require_reason", true) && !reason_given && marker.is_none_or(|m| m.text.is_empty()) {
        missing.push("a reason".to_owned());
    }
    if markers::opt_bool(o, "require_expiry", true) && marker.is_none_or(|m| m.date.is_none() && m.bad_date.is_none()) {
        missing.push("an expiry date".to_owned());
    }
    if markers::opt_bool(o, "require_ticket", false) && marker.is_none_or(|m| m.ticket.is_none()) {
        missing.push(format!("a ticket matching `{pattern}`"));
    }
    if !missing.is_empty() {
        out.push(finding(
            at.0,
            at.1,
            format!("{what} needs {}: write `{leader} SKIP(ABC-123, 2027-01-31): why` on the line above", missing.join(", ").replacen(", an expiry", " and an expiry", 1)),
            Vec::new(),
        ));
    }
    if let Some(m) = marker {
        let mut dated = Vec::new();
        let label = format!("The skip of {what}");
        markers::date_findings(m, &label, ctx.file, markers::opt_days(o, "warn_days", markers::DEFAULT_WARN_DAYS), &mut dated);
        // The finding sits on the skipped test, where the reader looks.
        out.extend(dated.into_iter().map(|f| finding(at.0, at.1, f.message, Vec::new())));
    }
}

fn skip_severity(finding: &NativeFinding) -> Option<Severity> {
    markers::severity_of_message(&finding.message)
}

pub struct JsSkippedTest;

/// The statement a call is part of: the outermost node whose parent is a
/// block, a program or a class body.
fn statement_of(n: Node<'_>) -> Node<'_> {
    let mut cur = n;
    while let Some(p) = cur.parent() {
        if matches!(p.kind(), "program" | "statement_block" | "class_body" | "switch_case" | "switch_default") {
            break;
        }
        cur = p;
    }
    cur
}

impl NativeRule for JsSkippedTest {
    fn id(&self) -> &'static str {
        "js/skipped-test"
    }
    fn languages(&self) -> &'static [Language] {
        JS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["call_expression"]
    }
    fn config_section(&self) -> Option<&'static str> {
        Some("tests.skip")
    }
    fn severity_of(&self, finding: &NativeFinding, _options: &Options) -> Option<Severity> {
        skip_severity(finding)
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        let src = ctx.source;
        let Some(callee) = n.child_by_field_name("function") else { return };
        let Some(p) = path(callee, src) else { return };
        let root = p[0].0.as_str();
        let skipped = if p.len() == 1 {
            matches!(root, "xit" | "xdescribe" | "xtest" | "xcontext" | "xspecify")
        } else {
            GROUPS.contains(&root) && p[1..].iter().any(|s| s.0 == "skip")
        };
        if !skipped {
            return;
        }
        // Playwright: `test.skip(condition, "reason")` carries its reason.
        let args = n.child_by_field_name("arguments").map(named_children).unwrap_or_default();
        let literal = |a: &Node<'_>| matches!(a.kind(), "string" | "template_string");
        let reason_given = args.len() >= 2 && !literal(&args[0]) && literal(&args[1]);
        let stmt = statement_of(n);
        let ticket = markers::ticket_regex(ctx.options).unwrap_or_else(|_| regex::Regex::new("^$").unwrap());
        let marker = skip_marker(src, stmt.start_byte(), "//", &ticket);
        let what = format!("`{}`", &src[callee.start_byte()..callee.end_byte()]);
        judge(&what, "//", (callee.start_byte(), callee.end_byte()), reason_given, marker.as_ref(), ctx, out);
    }
}

// ------------------------------------------------------------------- Python

pub struct PySkippedTest;

/// The dotted name of an expression: `pytest.mark.skip`.
fn dotted(n: Node<'_>, src: &str) -> Option<String> {
    match n.kind() {
        "identifier" => Some(text(n, src).to_owned()),
        "attribute" => Some(format!("{}.{}", dotted(n.child_by_field_name("object")?, src)?, text(n.child_by_field_name("attribute")?, src))),
        _ => None,
    }
}

/// How a decorated test says it is skipped: `Some(positional index of the
/// reason)`, where `None` inside means the framework has no positional one.
fn skip_kind(name: &str, src: &str) -> Option<Option<usize>> {
    let imported = |n: &str| regex::Regex::new(&format!(r"(?m)^\s*from\s+unittest\s+import\b[^\n]*\b{n}\b")).is_ok_and(|r| r.is_match(src));
    match name {
        "unittest.skip" => Some(Some(0)),
        "unittest.skipIf" | "unittest.skipUnless" => Some(Some(1)),
        "skip" if imported("skip") => Some(Some(0)),
        "skipIf" if imported("skipIf") => Some(Some(1)),
        "skipUnless" if imported("skipUnless") => Some(Some(1)),
        n if n.ends_with("mark.skip") || n.ends_with("mark.skipif") || n.ends_with("mark.xfail") => Some(None),
        _ => None,
    }
}

impl NativeRule for PySkippedTest {
    fn id(&self) -> &'static str {
        "py/skipped-test"
    }
    fn languages(&self) -> &'static [Language] {
        PY
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["decorator", "call"]
    }
    fn config_section(&self) -> Option<&'static str> {
        Some("tests.skip")
    }
    fn severity_of(&self, finding: &NativeFinding, _options: &Options) -> Option<Severity> {
        skip_severity(finding)
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        let src = ctx.source;
        // A decorator is checked as a whole; a call inside one is not checked twice.
        let (expr, anchor) = if n.kind() == "decorator" {
            let Some(e) = named_children(n).into_iter().find(|c| c.kind() != "comment") else { return };
            (e, n.parent().filter(|p| p.kind() == "decorated_definition").unwrap_or(n))
        } else {
            if n.parent().is_some_and(|p| p.kind() == "decorator") {
                return;
            }
            (n, statement_of_py(n))
        };
        let (func, args) = if expr.kind() == "call" {
            (expr.child_by_field_name("function"), expr.child_by_field_name("arguments").map(named_children).unwrap_or_default())
        } else {
            (Some(expr), Vec::new())
        };
        let Some(name) = func.and_then(|f| dotted(f, src)) else { return };
        let Some(positional) = skip_kind(&name, src) else { return };
        let keyword_reason = args.iter().any(|a| {
            a.kind() == "keyword_argument" && a.child_by_field_name("name").is_some_and(|k| text(k, src) == "reason") && a.child_by_field_name("value").is_some_and(|v| !(v.kind() == "string" && text(v, src).trim_matches(['"', '\'']).trim().is_empty()))
        });
        let positional_reason = positional.is_some_and(|i| args.iter().filter(|a| a.kind() != "keyword_argument").nth(i).is_some());
        let ticket = markers::ticket_regex(ctx.options).unwrap_or_else(|_| regex::Regex::new("^$").unwrap());
        let tags = ["SKIP".to_owned()];
        let marker = skip_marker(src, anchor.start_byte(), "#", &ticket).or_else(|| {
            // A comment between decorators, directly above this one.
            (anchor.start_byte() != n.start_byte()).then(|| comment_run_above(src, n.start_byte()).into_iter().find_map(|(b, l)| markers::parse_line(b, l, &tags, &ticket))).flatten()
        });
        let what = format!("`{}`", text(expr, src).split('(').next().unwrap_or(&name).trim());
        judge(&what, "#", (expr.start_byte(), expr.end_byte()), keyword_reason || positional_reason, marker.as_ref(), ctx, out);
    }
}

fn statement_of_py(n: Node<'_>) -> Node<'_> {
    let mut cur = n;
    while let Some(p) = cur.parent() {
        if matches!(p.kind(), "module" | "block") {
            break;
        }
        cur = p;
    }
    cur
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DetachedFile, FileContext};
    use std::collections::BTreeMap;

    struct Dated;
    impl FileContext for Dated {
        fn path(&self) -> &str {
            "t"
        }
        fn bytes(&self) -> &[u8] {
            &[]
        }
        fn exists(&self, _: &str) -> bool {
            false
        }
        fn is_dir(&self, _: &str) -> bool {
            false
        }
        fn read(&self, _: &str) -> Option<String> {
            None
        }
        fn list(&self, _: &str) -> Option<Vec<String>> {
            None
        }
        fn as_of(&self) -> Option<String> {
            Some("2026-10-30".into())
        }
    }

    fn run(rule: &dyn NativeRule, lang: Language, src: &str, options: serde_json::Value) -> Vec<NativeFinding> {
        let root = codelaw_lang::parse(lang, src);
        let options: BTreeMap<String, serde_json::Value> = serde_json::from_value(options).unwrap();
        let data = BTreeMap::new();
        let file = Dated;
        let ctx = NativeContext { source: src, language: lang, semantic: None, options: &options, data: &data, file: &file };
        let mut out = Vec::new();
        for n in root.root().dfs() {
            if rule.kinds().iter().any(|k| n.kind() == *k) {
                rule.check(n, &ctx, &mut out);
            }
        }
        let _ = DetachedFile::default();
        out
    }

    fn msgs(f: &[NativeFinding]) -> Vec<String> {
        f.iter().map(|f| f.message.clone()).collect()
    }

    #[test]
    fn focus_is_found_and_dropped() {
        let src = "describe.only('a', () => {});\nit.only('b', f);\nfit('c', f);\nfdescribe('d', f);\ntest.describe.only('e', f);\nit.only.each([1])('f', f);\nthing.only(1);\nit('ok', f);\n";
        let out = run(&NoFocusedTests, Language::Javascript, src, serde_json::json!({}));
        assert_eq!(out.len(), 6, "{:?}", msgs(&out));
        let mut fixed = src.to_owned();
        let mut edits: Vec<_> = out.iter().flat_map(|f| f.edits.clone()).collect();
        edits.sort_by_key(|e| std::cmp::Reverse(e.start));
        for e in edits {
            fixed.replace_range(e.start..e.end, &e.text);
        }
        assert_eq!(fixed, "describe('a', () => {});\nit('b', f);\nit('c', f);\ndescribe('d', f);\ntest.describe('e', f);\nit.each([1])('f', f);\nthing.only(1);\nit('ok', f);\n");
    }

    #[test]
    fn a_js_skip_needs_a_reason_and_an_expiry() {
        let bare = run(&JsSkippedTest, Language::Javascript, "it.skip('a', f);\nxdescribe('b', f);\n", serde_json::json!({}));
        assert_eq!(bare.len(), 2);
        assert!(bare[0].message.contains("needs a reason and an expiry date"), "{}", bare[0].message);
        let ok = "// SKIP(ABC-1, 2027-06-01): flaky on CI\nit.skip('a', f);\nxit('b', f); // SKIP(2027-06-01): later\n";
        let out = run(&JsSkippedTest, Language::Javascript, ok, serde_json::json!({}));
        assert!(out.is_empty(), "{:?}", msgs(&out));
        // Without a reason in the comment.
        let out = run(&JsSkippedTest, Language::Javascript, "// SKIP(ABC-1, 2027-06-01)\nit.skip('a', f);\n", serde_json::json!({}));
        assert_eq!(msgs(&out), ["`it.skip` needs a reason: write `// SKIP(ABC-123, 2027-01-31): why` on the line above"]);
        // Configured off.
        let out = run(&JsSkippedTest, Language::Javascript, "it.skip('a', f);\n", serde_json::json!({ "require_reason": false, "require_expiry": false }));
        assert!(out.is_empty());
        let out = run(&JsSkippedTest, Language::Javascript, "// SKIP(2027-06-01): x\nit.skip('a', f);\n", serde_json::json!({ "require_ticket": true }));
        assert!(out[0].message.contains("a ticket matching"));
    }

    #[test]
    fn playwright_reason_argument() {
        let out = run(&JsSkippedTest, Language::Typescript, "test.skip(process.env.CI, 'no GPU on CI');\n", serde_json::json!({ "require_expiry": false }));
        assert!(out.is_empty(), "{:?}", msgs(&out));
    }

    #[test]
    fn an_expired_skip_is_reported_on_the_skipped_test() {
        let src = "// SKIP(ABC-1, 2026-10-01): flaky\nit.skip('a', f);\n// SKIP(ABC-1, 2026-11-02): flaky\nit.skip('b', f);\n";
        let out = run(&JsSkippedTest, Language::Javascript, src, serde_json::json!({}));
        assert_eq!(out.len(), 2, "{:?}", msgs(&out));
        assert!(out[0].message.contains("expired on 2026-10-01 (29 days ago)"));
        assert!(out[1].message.contains("expires in 3 days"));
        assert_eq!(JsSkippedTest.severity_of(&out[1], &Options::new()), Some(Severity::Warn));
        assert_eq!(&src[out[0].start..out[0].end], "it.skip");
    }

    #[test]
    fn python_skips() {
        let src = "import pytest, unittest\n\n@pytest.mark.skip\ndef test_a(): pass\n\n@pytest.mark.skip(reason='x')\ndef test_b(): pass\n\n# SKIP(ABC-1, 2027-01-01): why\n@pytest.mark.skipif(sys.platform == 'win32')\ndef test_c(): pass\n\n@unittest.skip('later')\ndef test_d(): pass\n\n@unittest.skipIf(cond, 'reason')\ndef test_e(): pass\n\n@pytest.mark.xfail(reason='bug')\ndef test_f(): pass\n\n@pytest.mark.parametrize('a', [1])\ndef test_g(a): pass\n";
        let out = run(&PySkippedTest, Language::Python, src, serde_json::json!({}));
        assert_eq!(out.len(), 5, "{:?}", msgs(&out));
        assert!(out[0].message.contains("`pytest.mark.skip` needs a reason and an expiry date"), "{}", out[0].message);
        // b, d, e, f have a reason and no expiry; c has both, in its comment.
        assert!(out[1].message.contains("needs an expiry date"));
        assert!(!out.iter().any(|f| f.message.contains("skipif")));
        let out2 = run(&PySkippedTest, Language::Python, "@pytest.mark.skipif(sys.platform == 'win32')\ndef test_c(): pass\n", serde_json::json!({ "require_expiry": false }));
        assert_eq!(msgs(&out2), ["`pytest.mark.skipif` needs a reason: write `# SKIP(ABC-123, 2027-01-31): why` on the line above"]);
        let out = run(&PySkippedTest, Language::Python, src, serde_json::json!({ "require_expiry": false }));
        assert_eq!(out.len(), 1, "{:?}", msgs(&out));
    }

    #[test]
    fn python_module_marks_and_expiry() {
        let src = "import pytest\n# SKIP(2026-10-01): old\npytestmark = pytest.mark.skip(reason='r')\n";
        let out = run(&PySkippedTest, Language::Python, src, serde_json::json!({}));
        assert_eq!(out.len(), 1, "{:?}", msgs(&out));
        assert!(out[0].message.contains("expired on 2026-10-01"));
    }
}
