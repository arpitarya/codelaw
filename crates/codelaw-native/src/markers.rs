//! TODO-style markers and skipped tests (W-131; SR-CONFIG decision 11a):
//! `TODO(ABC-123, 2027-01-31): text` in a comment, and the `SKIP(…)` comment
//! that gives a skipped test its reason and expiry.
//!
//! A marker is a configured tag at the start of a comment line, then an
//! optional parenthesised list, then an optional `:` and text. In the list a
//! `YYYY-MM-DD` item is the date, an item matching the ticket pattern is the
//! ticket, and any other item (an owner's name) is ignored; the order is free.
//! Dates are judged on the as-of date, never the clock (L5): a marker is
//! expired from its date on (SR-BASELINE decision 9).

use crate::{FileContext, NativeContext, NativeFinding, NativeRule};
use codelaw_core::{Language, Severity};
use codelaw_lang::SyntaxNode;
use regex::Regex;
use std::collections::BTreeMap;

pub type Options = BTreeMap<String, serde_json::Value>;

pub const DEFAULT_TAGS: &[&str] = &["TODO", "FIXME", "HACK"];
pub const DEFAULT_TICKET_PATTERN: &str = "[A-Z][A-Z0-9]+-\\d+";
pub const DEFAULT_WARN_DAYS: i64 = 14;

// ------------------------------------------------------------------ dates

/// Days since 1970-01-01 of a real calendar date `YYYY-MM-DD`.
pub fn days_of(date: &str) -> Option<i64> {
    if !is_date_shaped(date) {
        return None;
    }
    let y: i64 = date[..4].parse().ok()?;
    let m: i64 = date[5..7].parse().ok()?;
    let d: i64 = date[8..10].parse().ok()?;
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let last = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return None,
    };
    if d < 1 || d > last {
        return None;
    }
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

fn is_date_shaped(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10 && b.iter().enumerate().all(|(i, c)| if i == 4 || i == 7 { *c == b'-' } else { c.is_ascii_digit() })
}

// ---------------------------------------------------------------- options

pub fn opt_bool(o: &Options, key: &str, default: bool) -> bool {
    o.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
}

pub fn opt_days(o: &Options, key: &str, default: i64) -> i64 {
    o.get(key).and_then(|v| v.as_i64()).unwrap_or(default)
}

pub fn tags(o: &Options) -> Vec<String> {
    match o.get("tags").and_then(|v| v.as_array()) {
        Some(a) if !a.is_empty() => a.iter().filter_map(|t| t.as_str().map(str::to_owned)).collect(),
        _ => DEFAULT_TAGS.iter().map(|t| (*t).to_owned()).collect(),
    }
}

/// The ticket pattern, anchored to the whole item; `Err` is the pattern the
/// project wrote when it is not a regular expression.
pub fn ticket_regex(o: &Options) -> Result<Regex, String> {
    let p = o.get("ticket_pattern").and_then(|v| v.as_str()).unwrap_or(DEFAULT_TICKET_PATTERN);
    Regex::new(&format!("^(?:{p})$")).map_err(|_| p.to_owned())
}

// ----------------------------------------------------------------- parsing

/// One marker found in a comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Marker {
    pub tag: String,
    /// The tag and its parenthesised list, as byte offsets into the source.
    pub start: usize,
    pub end: usize,
    pub ticket: Option<String>,
    /// A date item that is a real calendar date, with its days count.
    pub date: Option<(String, i64)>,
    /// A date-shaped item that is not a calendar date.
    pub bad_date: Option<String>,
    /// The text after the marker (after an optional `:`), trimmed.
    pub text: String,
}

/// Strip the comment's own punctuation from the front of a line.
fn strip_leaders(line: &str) -> usize {
    let mut off = 0;
    let mut stripped = false;
    loop {
        let rest = &line[off..];
        let trimmed = rest.trim_start_matches([' ', '\t']);
        off += rest.len() - trimmed.len();
        let rest = &line[off..];
        let step = if rest.starts_with("<!--") {
            4
        } else if rest.starts_with("//") || rest.starts_with("/*") {
            2
        } else if rest.starts_with('*') || rest.starts_with('#') || (stripped && (rest.starts_with('/') || rest.starts_with('!'))) {
            1
        } else {
            return off;
        };
        off += step;
        stripped = true;
    }
}

/// The marker a single comment line opens, if any. `base` is the byte offset
/// of `line` in the source.
pub fn parse_line(base: usize, line: &str, tags: &[String], ticket: &Regex) -> Option<Marker> {
    let off = strip_leaders(line);
    let rest = &line[off..];
    let tag = tags.iter().find(|t| rest.starts_with(t.as_str()) && !rest[t.len()..].starts_with(|c: char| c.is_alphanumeric() || c == '_'))?;
    let after = &rest[tag.len()..];
    let mut m = Marker { tag: tag.clone(), start: base + off, end: base + off + tag.len(), ticket: None, date: None, bad_date: None, text: String::new() };
    let mut tail = after;
    if let Some(inner) = after.strip_prefix('(')
        && let Some(close) = inner.find(')')
    {
        for item in inner[..close].split(',').map(str::trim).filter(|i| !i.is_empty()) {
            if is_date_shaped(item) {
                match days_of(item) {
                    Some(d) => m.date = Some((item.to_owned(), d)),
                    None => m.bad_date = Some(item.to_owned()),
                }
            } else if ticket.is_match(item) {
                m.ticket = Some(item.to_owned());
            }
        }
        m.end += 1 + close + 1;
        tail = &inner[close + 1..];
    }
    let tail = tail.trim_start();
    let tail = tail.strip_prefix(':').unwrap_or(tail);
    let tail = tail.trim();
    let tail = tail.strip_suffix("*/").or_else(|| tail.strip_suffix("-->")).unwrap_or(tail).trim();
    m.text = tail.to_owned();
    Some(m)
}

/// Every marker in a block of comment text that starts at byte `base`.
pub fn scan(text: &str, base: usize, tags: &[String], ticket: &Regex) -> Vec<Marker> {
    let mut out = Vec::new();
    let mut at = 0;
    for line in text.split_inclusive('\n') {
        if let Some(m) = parse_line(base + at, line.trim_end_matches(['\n', '\r']), tags, ticket) {
            out.push(m);
        }
        at += line.len();
    }
    out
}

// ---------------------------------------------------------------- verdicts

/// The as-of date in days, asked for only when a marker is dated.
pub fn as_of_days(file: &dyn FileContext) -> Option<i64> {
    file.as_of().as_deref().and_then(days_of)
}

/// The dated findings of a marker: a date that is not a date, a date that has
/// come, a date that is near. `what` names the thing (`` `TODO(ABC-1)` ``).
pub fn date_findings(m: &Marker, what: &str, file: &dyn FileContext, warn_days: i64, out: &mut Vec<NativeFinding>) {
    let at = |message: String| NativeFinding { start: m.start, end: m.end, message, edits: Vec::new() };
    if let Some(bad) = &m.bad_date {
        out.push(at(format!("{what}: `{bad}` is not a calendar date; write it as YYYY-MM-DD")));
    }
    let Some((date, day)) = &m.date else { return };
    let Some(today) = as_of_days(file) else { return };
    if today >= *day {
        let late = today - day;
        let when = if late == 0 { "today".to_owned() } else { format!("{late} day{} ago", if late == 1 { "" } else { "s" }) };
        out.push(at(format!("{what} expired on {date} ({when}): do the work, move the date with a reason, or remove the marker")));
    } else if warn_days > 0 && day - today <= warn_days {
        let left = day - today;
        out.push(at(format!("{what} expires in {left} day{} ({date})", if left == 1 { "" } else { "s" })));
    }
}

/// An expiring finding is a warning; everything else keeps the rule's severity.
pub fn severity_of_message(message: &str) -> Option<Severity> {
    message.contains(" expires in ").then_some(Severity::Warn)
}

// --------------------------------------------------------- the TODO rules

/// Where a language keeps its comments.
#[derive(Clone, Copy)]
pub enum Source {
    /// `comment` nodes of the syntax tree (JS, TS, Python, HTML, YAML, TOML).
    Tree,
    /// `<!-- … -->` in a Markdown `html_block`.
    Markdown,
    /// `/* … */` in the hand-read stylesheet.
    Css,
    /// `/* … */` and `//` in the hand-read stylesheet.
    Scss,
}

pub struct TodoMarker {
    pub id: &'static str,
    pub languages: &'static [Language],
    pub source: Source,
}

/// Comment spans `(start, end)` of a stylesheet: strings and `url(…)` are
/// not scanned, `//` is a comment only in SCSS.
fn stylesheet_comments(src: &str, scss: bool) -> Vec<(usize, usize)> {
    let b = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'"' | b'\'' => i = crate::css_syntax::skip_string(b, i),
            b'/' if b.get(i + 1) == Some(&b'*') => {
                let e = crate::css_syntax::skip_comment(src, i);
                out.push((i, e));
                i = e;
            }
            b'/' if scss && b.get(i + 1) == Some(&b'/') => {
                let e = src[i..].find('\n').map_or(b.len(), |o| i + o);
                out.push((i, e));
                i = e;
            }
            b'u' if src[i..].starts_with("url(") => {
                let inner = src[i + 4..].trim_start();
                if inner.starts_with(['"', '\'']) {
                    i += 4;
                } else {
                    i = src[i..].find(')').map_or(b.len(), |o| i + o + 1);
                }
            }
            _ => i += 1,
        }
    }
    out
}

/// `<!-- … -->` spans inside `text`, which starts at byte `base`.
fn html_comments(text: &str, base: usize) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut at = 0;
    while let Some(o) = text[at..].find("<!--") {
        let s = at + o;
        let e = text[s + 4..].find("-->").map_or(text.len(), |c| s + 4 + c + 3);
        out.push((base + s, base + e));
        at = e;
    }
    out
}

impl NativeRule for TodoMarker {
    fn id(&self) -> &'static str {
        self.id
    }
    fn languages(&self) -> &'static [Language] {
        self.languages
    }
    fn kinds(&self) -> &'static [&'static str] {
        match self.source {
            Source::Tree => &["comment"],
            Source::Markdown => &["html_block"],
            Source::Css | Source::Scss => &["stylesheet", "ERROR"],
        }
    }
    fn config_section(&self) -> Option<&'static str> {
        Some("todo")
    }
    fn severity_of(&self, finding: &NativeFinding, _options: &Options) -> Option<Severity> {
        severity_of_message(&finding.message)
    }
    fn check(&self, node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        let n = node.get_inner_node();
        let spans: Vec<(usize, usize)> = match self.source {
            Source::Tree => vec![(n.start_byte(), n.end_byte())],
            Source::Markdown => html_comments(&ctx.source[n.byte_range()], n.start_byte()),
            Source::Css | Source::Scss => {
                if node.parent().is_some() {
                    return;
                }
                stylesheet_comments(ctx.source, matches!(self.source, Source::Scss))
            }
        };
        if spans.is_empty() {
            return;
        }
        let tags = tags(ctx.options);
        let ticket = match ticket_regex(ctx.options) {
            Ok(r) => r,
            Err(p) => {
                out.push(NativeFinding { start: spans[0].0, end: spans[0].1, message: format!("`[todo] ticket_pattern` `{p}` is not a regular expression"), edits: Vec::new() });
                return;
            }
        };
        let require_ticket = opt_bool(ctx.options, "require_ticket", true);
        let require_date = opt_bool(ctx.options, "require_date", false);
        let warn_days = opt_days(ctx.options, "warn_days", DEFAULT_WARN_DAYS);
        let pattern = ctx.options.get("ticket_pattern").and_then(|v| v.as_str()).unwrap_or(DEFAULT_TICKET_PATTERN);
        for (s, e) in spans {
            for m in scan(&ctx.source[s..e], s, &tags, &ticket) {
                let mut missing = Vec::new();
                if require_ticket && m.ticket.is_none() {
                    missing.push(format!("a ticket matching `{pattern}`"));
                }
                if require_date && m.date.is_none() && m.bad_date.is_none() {
                    missing.push("a date (YYYY-MM-DD)".to_owned());
                }
                if !missing.is_empty() {
                    out.push(NativeFinding {
                        start: m.start,
                        end: m.end,
                        message: format!("`{}` needs {}: write `{}(ABC-123, 2027-01-31): …`", m.tag, missing.join(" and "), m.tag),
                        edits: Vec::new(),
                    });
                }
                let what = format!("`{}{}`", m.tag, ctx.source[m.start + m.tag.len()..m.end].to_owned());
                date_findings(&m, &what, ctx.file, warn_days, out);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DetachedFile;

    struct Dated(&'static str);
    impl FileContext for Dated {
        fn path(&self) -> &str {
            "f"
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
            Some(self.0.to_owned())
        }
    }

    fn run(rule: &TodoMarker, lang: Language, src: &str, options: serde_json::Value, today: Option<&'static str>) -> Vec<NativeFinding> {
        let root = codelaw_lang::parse(lang, src);
        let options: Options = serde_json::from_value(options).unwrap();
        let data = BTreeMap::new();
        let dated = Dated(today.unwrap_or(""));
        let detached = DetachedFile::default();
        let file: &dyn FileContext = if today.is_some() { &dated } else { &detached };
        let ctx = NativeContext { source: src, language: lang, semantic: None, options: &options, data: &data, file };
        let mut out = Vec::new();
        for n in root.root().dfs() {
            if rule.kinds().iter().any(|k| n.kind() == *k) {
                rule.check(n, &ctx, &mut out);
            }
        }
        out
    }

    const JS: TodoMarker = TodoMarker { id: "js/todo-marker", languages: &[Language::Javascript], source: Source::Tree };

    fn messages(f: &[NativeFinding]) -> Vec<&str> {
        f.iter().map(|f| f.message.as_str()).collect()
    }

    #[test]
    fn dates_are_real_dates() {
        assert_eq!(days_of("1970-01-01"), Some(0));
        assert_eq!(days_of("2027-02-29"), None);
        assert_eq!(days_of("2028-02-29").map(|d| d - days_of("2028-02-28").unwrap()), Some(1));
        assert_eq!(days_of("2027-13-01"), None);
    }

    #[test]
    fn ticket_and_date_in_either_order() {
        let ok = "// TODO(ABC-12, 2027-01-31): later\n// TODO(2027-01-31, ABC-12) later\n/* FIXME(ABC-1) */\n";
        assert!(run(&JS, Language::Javascript, ok, serde_json::json!({}), Some("2026-10-30")).is_empty());
    }

    #[test]
    fn a_bare_marker_needs_a_ticket() {
        let out = run(&JS, Language::Javascript, "// TODO: later\n// HACK(arpit) shim\nlet note = 'TODO'; // not at the start: the TODO list\n", serde_json::json!({}), None);
        assert_eq!(out.len(), 2, "{out:?}");
        assert!(out[0].message.starts_with("`TODO` needs a ticket matching"), "{}", out[0].message);
        assert!(run(&JS, Language::Javascript, "// TODO: later\n", serde_json::json!({ "require_ticket": false }), None).is_empty());
    }

    #[test]
    fn a_date_is_required_when_configured_and_enforced_when_written() {
        let opts = serde_json::json!({ "require_date": true });
        let out = run(&JS, Language::Javascript, "// TODO(ABC-1): x\n", opts, None);
        assert_eq!(messages(&out), ["`TODO` needs a date (YYYY-MM-DD): write `TODO(ABC-123, 2027-01-31): …`"]);
        let src = "// TODO(ABC-1, 2026-10-30): x\n// TODO(ABC-2, 2026-11-05): y\n// TODO(ABC-3, 2027-06-01): z\n// TODO(ABC-4, 2026-02-30): w\n";
        let out = run(&JS, Language::Javascript, src, serde_json::json!({}), Some("2026-10-30"));
        assert_eq!(out.len(), 3, "{:?}", messages(&out));
        assert!(out[0].message.contains("expired on 2026-10-30 (today)"));
        assert!(out[1].message.contains("expires in 6 days (2026-11-05)"));
        assert_eq!(JS.severity_of(&out[1], &Options::new()), Some(Severity::Warn));
        assert_eq!(JS.severity_of(&out[0], &Options::new()), None);
        assert!(out[2].message.contains("not a calendar date"));
        // No as-of date, no judgment of the date.
        assert!(run(&JS, Language::Javascript, src, serde_json::json!({}), None).len() == 1);
    }

    #[test]
    fn custom_tags_and_ticket_pattern() {
        let opts = serde_json::json!({ "tags": ["XXX"], "ticket_pattern": "#\\d+" });
        let out = run(&JS, Language::Javascript, "// TODO: ignored\n// XXX(#42) ok\n// XXX(ABC-1)\n", opts, None);
        assert_eq!(out.len(), 1);
        assert!(out[0].message.contains("a ticket matching `#\\d+`"), "{}", out[0].message);
    }

    #[test]
    fn block_comments_and_leaders() {
        let src = "/**\n * TODO(ABC-1, 2020-01-01): old\n * the TODO of it\n */\n";
        let out = run(&JS, Language::Javascript, src, serde_json::json!({}), Some("2026-10-30"));
        assert_eq!(out.len(), 1);
        assert_eq!(&src[out[0].start..out[0].end], "TODO(ABC-1, 2020-01-01)");
    }

    #[test]
    fn stylesheets_and_markdown() {
        let css = TodoMarker { id: "css/todo-marker", languages: &[Language::Css], source: Source::Css };
        let out = run(&css, Language::Css, "a { background: url(http://x/y); content: \"/* TODO */\" } /* TODO: a */\n", serde_json::json!({}), None);
        assert_eq!(out.len(), 1);
        let scss = TodoMarker { id: "scss/todo-marker", languages: &[Language::Scss], source: Source::Scss };
        let out = run(&scss, Language::Scss, "// TODO(ABC-1)\na { b: c } // FIXME later\n", serde_json::json!({}), None);
        assert_eq!(out.len(), 1);
        let md = TodoMarker { id: "md/todo-marker", languages: &[Language::Markdown], source: Source::Markdown };
        let out = run(&md, Language::Markdown, "# T\n\n<!-- TODO: write -->\n\nTODO in prose is prose.\n", serde_json::json!({}), None);
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn data_languages() {
        let yaml = TodoMarker { id: "yaml/todo-marker", languages: &[Language::Yaml], source: Source::Tree };
        assert_eq!(run(&yaml, Language::Yaml, "a: 1 # TODO: x\n# FIXME(ABC-1)\n", serde_json::json!({}), None).len(), 1);
        let toml = TodoMarker { id: "toml/todo-marker", languages: &[Language::Toml], source: Source::Tree };
        assert_eq!(run(&toml, Language::Toml, "# TODO: x\na = 1\n", serde_json::json!({}), None).len(), 1);
        let py = TodoMarker { id: "py/todo-marker", languages: &[Language::Python], source: Source::Tree };
        assert_eq!(run(&py, Language::Python, "x = 1  # TODO: x\n", serde_json::json!({}), None).len(), 1, "a trailing comment is a comment");
        assert_eq!(run(&py, Language::Python, "# TODO: x\n", serde_json::json!({}), None).len(), 1);
        let html = TodoMarker { id: "html/todo-marker", languages: &[Language::Html], source: Source::Tree };
        assert_eq!(run(&html, Language::Html, "<!-- TODO: x -->\n<p>a</p>\n", serde_json::json!({}), None).len(), 1);
    }
}
