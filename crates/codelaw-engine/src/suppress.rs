//! Inline suppression (SR-LINTER decision 6): `codelaw-ignore <rule>:
//! <reason>` in the host language's comment syntax. A comment alone on its
//! line covers the next line with code; a trailing comment covers its own
//! line. No reason is an error; a suppression that suppresses nothing is
//! reported with an `auto` fix that removes it.

use codelaw_baseline::Date;
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
    /// The dated-exception keys before the colon (SR-BASELINE decision 9).
    pub dated: Dated,
}

/// `expires=`, `warn_days=` and `by=` of one `codelaw-ignore`; `until=`
/// reads as `expires` the day after.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Dated {
    pub expires: Option<Date>,
    pub warn_days: Option<u32>,
    pub by: Option<String>,
    /// Why a key could not be read; the comment then suppresses nothing.
    pub bad: Option<String>,
}

pub(crate) const MARKER: &str = "codelaw-ignore";

/// Split the dated-exception keys (`key=value`) out of the ids.
fn dated(ids: &mut Vec<String>) -> Dated {
    let mut out = Dated::default();
    let (mut until, mut expires) = (None, None);
    let mut keys = Vec::new();
    ids.retain(|t| match t.split_once('=') {
        Some((k, v)) => {
            keys.push((k.to_owned(), v.to_owned()));
            false
        }
        None => true,
    });
    for (k, v) in keys {
        let r: Result<(), String> = match k.as_str() {
            "expires" => Date::parse(&v).map(|d| expires = Some(d)).map_err(|e| format!("`expires=`: {e}")),
            "until" => Date::parse(&v).map(|d| until = Some(d)).map_err(|e| format!("`until=`: {e}")),
            "warn_days" => v.parse::<u32>().map(|n| out.warn_days = Some(n)).map_err(|_| format!("`warn_days={v}` is not a whole number of days")),
            "by" => {
                out.by = Some(v);
                Ok(())
            }
            other => Err(format!("`{other}=` is not a key (expires, until, warn_days, by)")),
        };
        if let Err(e) = r {
            out.bad.get_or_insert(e);
        }
    }
    match codelaw_baseline::exception::expires_of(until, expires) {
        Ok(e) => out.expires = e,
        Err(e) => {
            out.bad.get_or_insert(e);
        }
    }
    out
}

/// Parse one comment's text, if it carries the marker. The marker opens
/// the comment, after its delimiter (`//`, `#`, `/*`, `/**`, `*`, `<!--`):
/// a comment that only mentions it (`// see codelaw-ignore docs`) is prose.
fn parse(comment: &str) -> Option<(Vec<String>, Option<String>, Dated)> {
    let body = comment.trim_start().trim_start_matches(['/', '#', '*', '<', '!', '-']).trim_start();
    let rest = body.strip_prefix(MARKER)?;
    // `codelaw-ignore-file` and the like are not this marker.
    if rest.starts_with(|c: char| c.is_alphanumeric() || c == '-' || c == '_') {
        return None;
    }
    let rest = rest.trim_end().trim_end_matches("*/").trim_end_matches("-->").trim();
    let (ids, reason) = match rest.split_once(':') {
        Some((ids, reason)) => (ids, Some(reason.trim()).filter(|r| !r.is_empty()).map(str::to_owned)),
        None => (rest, None),
    };
    let mut rules: Vec<String> = ids.split(|c: char| c == ',' || c.is_whitespace()).filter(|s| !s.is_empty()).map(str::to_owned).collect();
    let dated = dated(&mut rules);
    Some((rules, reason, dated))
}

/// Find every suppression among the comment ranges (host bytes) of a file.
pub fn find(text: &str, comments: &[(usize, usize)], index: &LineIndex<'_>) -> Vec<Suppression> {
    let mut out = Vec::new();
    for &(start, end) in comments {
        let Some((rules, reason, dated)) = parse(&text[start..end]) else { continue };
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
        out.push(Suppression { rules, reason, start, end, target_line, own_line, used: false, dated });
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
    fn parses_the_dated_keys() {
        let (rules, reason, d) = full("// codelaw-ignore js/no-console expires=2026-11-15 by=arpit@example.com: needed until the logger lands").unwrap();
        assert_eq!((rules, reason.as_deref()), (vec!["js/no-console".to_owned()], Some("needed until the logger lands")));
        assert_eq!(d, Dated { expires: Some(Date::parse("2026-11-15").unwrap()), warn_days: None, by: Some("arpit@example.com".into()), bad: None });
        // `until=` is the last allowed day.
        assert_eq!(full("# codelaw-ignore py/a until=2026-11-14 warn_days=5: r").unwrap().2.expires, Some(Date::parse("2026-11-15").unwrap()));
        assert_eq!(full("# codelaw-ignore py/a warn_days=5: r").unwrap().2.warn_days, Some(5));
        assert!(full("# codelaw-ignore py/a expires=2026-13-01: r").unwrap().2.bad.unwrap().contains("expires="));
        assert!(full("# codelaw-ignore py/a until=2026-11-14 expires=2026-11-15: r").unwrap().2.bad.unwrap().contains("`until` and `expires`"));
        assert!(full("# codelaw-ignore py/a owner=x: r").unwrap().2.bad.unwrap().contains("`owner=` is not a key"));
    }

    fn full(s: &str) -> Option<(Vec<String>, Option<String>, Dated)> {
        super::parse(s)
    }

    fn parse(s: &str) -> Option<(Vec<String>, Option<String>)> {
        super::parse(s).map(|(r, why, _)| (r, why))
    }

    #[test]
    fn parses_ids_and_reason() {
        assert_eq!(parse("// codelaw-ignore js/a, js/b: legacy API"), Some((vec!["js/a".into(), "js/b".into()], Some("legacy API".into()))));
        assert_eq!(parse("/* codelaw-ignore js/a */"), Some((vec!["js/a".into()], None)));
        assert_eq!(parse("<!-- codelaw-ignore html/x:   -->"), Some((vec!["html/x".into()], None)));
        assert_eq!(parse("# codelaw-ignore-file x: y"), None);
        assert_eq!(parse("# a note"), None);
        // Only a comment that opens with the marker is a suppression.
        assert_eq!(parse("// see codelaw-ignore docs"), None);
        assert_eq!(parse("# TODO: drop the codelaw-ignore x/y: z"), None);
        assert_eq!(parse("/** codelaw-ignore js/a: b */"), Some((vec!["js/a".into()], Some("b".into()))));
        assert_eq!(parse("#codelaw-ignore py/a: b"), Some((vec!["py/a".into()], Some("b".into()))));
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
