//! Inline suppression (SR-LINTER decision 6): `codelaw-ignore <rule>:
//! <reason>` in the host language's comment syntax. A comment alone on its
//! line covers the next line with code; a trailing comment covers its own
//! line. No reason is an error; a suppression that suppresses nothing is
//! reported with an `auto` fix that removes it.

use codelaw_core::LineIndex;

#[derive(Clone, Debug)]
pub struct Suppression {
    pub rules: Vec<String>,
    pub reason: Option<String>,
    /// The comment, in host bytes.
    pub start: usize,
    pub end: usize,
    /// The 1-based line it covers.
    pub target_line: u32,
    pub own_line: bool,
    pub used: bool,
}

const MARKER: &str = "codelaw-ignore";

/// Parse one comment's text, if it carries the marker.
fn parse(comment: &str) -> Option<(Vec<String>, Option<String>)> {
    let at = comment.find(MARKER)?;
    let rest = &comment[at + MARKER.len()..];
    // `codelaw-ignore-file` and the like are not this marker.
    if rest.starts_with(|c: char| c.is_alphanumeric() || c == '-' || c == '_') {
        return None;
    }
    let rest = rest.trim_end().trim_end_matches("*/").trim_end_matches("-->").trim();
    let (ids, reason) = match rest.split_once(':') {
        Some((ids, reason)) => (ids, Some(reason.trim()).filter(|r| !r.is_empty()).map(str::to_owned)),
        None => (rest, None),
    };
    let rules: Vec<String> = ids.split(|c: char| c == ',' || c.is_whitespace()).filter(|s| !s.is_empty()).map(str::to_owned).collect();
    Some((rules, reason))
}

/// Find every suppression among the comment ranges (host bytes) of a file.
pub fn find(text: &str, comments: &[(usize, usize)], index: &LineIndex<'_>) -> Vec<Suppression> {
    let mut out = Vec::new();
    for &(start, end) in comments {
        let Some((rules, reason)) = parse(&text[start..end]) else { continue };
        let line_start = text[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let own_line = text[line_start..start].trim().is_empty();
        let comment_line = index.position(start).line;
        let target_line = if own_line {
            // The next line with code after the comment ends.
            let mut line = index.position(end).line + 1;
            while let Some(s) = index.line_start(line) {
                let e = index.line_start(line + 1).unwrap_or(text.len());
                if !text[s..e].trim().is_empty() {
                    break;
                }
                line += 1;
            }
            line
        } else {
            comment_line
        };
        out.push(Suppression { rules, reason, start, end, target_line, own_line, used: false });
    }
    out
}

/// The bytes removing a suppression takes out: its whole line when it is
/// alone there, otherwise the comment and the space before it.
pub fn removal(text: &str, s: &Suppression) -> (usize, usize) {
    let line_start = text[..s.start].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let rest = &text[s.end..];
    let line_end = rest.find('\n').map(|i| s.end + i + 1).unwrap_or(text.len());
    if s.own_line && text[s.end..line_end].trim().is_empty() {
        return (line_start, line_end);
    }
    let before = text[..s.start].trim_end_matches([' ', '\t']).len();
    (before.max(line_start), s.end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ids_and_reason() {
        assert_eq!(parse("// codelaw-ignore js/a, js/b: legacy API"), Some((vec!["js/a".into(), "js/b".into()], Some("legacy API".into()))));
        assert_eq!(parse("/* codelaw-ignore js/a */"), Some((vec!["js/a".into()], None)));
        assert_eq!(parse("<!-- codelaw-ignore html/x:   -->"), Some((vec!["html/x".into()], None)));
        assert_eq!(parse("# codelaw-ignore-file x: y"), None);
        assert_eq!(parse("# a note"), None);
    }

    #[test]
    fn targets_and_removal() {
        let text = "a();\n  // codelaw-ignore x/y: why\n\n  b();\nc(); // codelaw-ignore x/z: because\n";
        let idx = LineIndex::new(text);
        let c1 = text.find("// codelaw").unwrap();
        let e1 = text[c1..].find('\n').unwrap() + c1;
        let c2 = text.rfind("// codelaw").unwrap();
        let e2 = text.len() - 1;
        let s = find(text, &[(c1, e1), (c2, e2)], &idx);
        assert_eq!((s[0].target_line, s[0].own_line), (4, true));
        assert_eq!((s[1].target_line, s[1].own_line), (5, false));
        let (a, b) = removal(text, &s[0]);
        assert_eq!(&text[a..b], "  // codelaw-ignore x/y: why\n");
        let (a, b) = removal(text, &s[1]);
        assert_eq!(&text[a..b], " // codelaw-ignore x/z: because");
    }
}
