//! Inline suppression (SR-LINTER decision 6): `codelaw-ignore <rule>:
//! <reason>` in the host language's comment syntax. A comment alone on its
//! line covers the next line with code; a trailing comment covers its own
//! line. No reason is an error; a suppression that suppresses nothing is
//! reported with an `auto` fix that removes it.
//!
//! `[lint] ignore_aliases` (W-94) also reads another tool's line
//! suppressions: `eslint-disable-next-line` / `eslint-disable-line`,
//! Ruff's `# noqa` and `biome-ignore`. Their ids are the tool's, mapped to
//! codelaw ids through each rule's `ported_from:`; they need no reason and
//! are never reported as unknown or unused, since the old tool reads them too.

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
    /// The tool whose comment this is, for an alias; `rules` are then that
    /// tool's ids until [`translate`] maps them, and none means every rule.
    pub alias: Option<Alias>,
}

/// A tool whose line suppressions `[lint] ignore_aliases` accepts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Alias {
    Eslint,
    Ruff,
    Biome,
}

impl Alias {
    pub const ALL: [Alias; 3] = [Alias::Eslint, Alias::Ruff, Alias::Biome];

    pub fn name(self) -> &'static str {
        match self {
            Alias::Eslint => "eslint",
            Alias::Ruff => "ruff",
            Alias::Biome => "biome",
        }
    }

    pub fn parse(name: &str) -> Option<Alias> {
        Alias::ALL.into_iter().find(|a| a.name() == name)
    }
}

/// Each Biome rule and the upstream rules Biome names as its source: the
/// data `migrate biome` and the `biome-ignore` alias map through.
const BIOME_SOURCES: &str = include_str!("../data/biome-sources.json");

fn biome_sources() -> &'static std::collections::BTreeMap<String, Vec<String>> {
    static TABLE: std::sync::OnceLock<std::collections::BTreeMap<String, Vec<String>>> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| {
        #[derive(serde::Deserialize)]
        struct File {
            sources: std::collections::BTreeMap<String, Vec<String>>,
        }
        serde_json::from_str::<File>(BIOME_SOURCES).expect("data/biome-sources.json parses").sources
    })
}

/// The `ported_from:` ids a tool's rule id may appear under, most likely
/// first. ESLint plugins are named by their package (`react/jsx-key` is
/// `eslint-plugin-react/jsx-key`); a Biome rule goes through its sources;
/// a Ruff code is `ruff/<code>`. The same table serves `codelaw migrate`.
pub fn upstream_keys(tool: Alias, id: &str) -> Vec<String> {
    match tool {
        Alias::Ruff => vec![format!("ruff/{}", id.to_ascii_uppercase())],
        Alias::Biome => {
            // `lint/suspicious/noDebugger` or `noDebugger`.
            let name = id.rsplit('/').next().unwrap_or(id);
            biome_sources().get(name).cloned().unwrap_or_default()
        }
        Alias::Eslint => eslint_keys(id),
    }
}

/// The `ported_from:` ids of an ESLint rule id as a config names it.
pub fn eslint_keys(id: &str) -> Vec<String> {
    let Some((plugin, rule)) = id.rsplit_once('/') else { return vec![format!("eslint/{id}")] };
    let mut out = Vec::new();
    match plugin {
        "@typescript-eslint" => out.push(format!("typescript-eslint/{rule}")),
        "@tanstack/query" => out.push(format!("@tanstack/eslint-plugin-query/{rule}")),
        "@stencil-community" | "@stencil" | "stencil" => out.push(format!("stencil/{rule}")),
        "node" | "n" => out.push(format!("eslint-plugin-n/{rule}")),
        _ => {}
    }
    if let Some(scoped) = plugin.strip_prefix('@') {
        // `@eslint-react/x` is written as the plugin names it.
        out.push(format!("@{scoped}/{rule}"));
        match scoped.split_once('/') {
            Some((scope, name)) => out.push(format!("@{scope}/eslint-plugin-{name}/{rule}")),
            None => out.push(format!("@{scoped}/eslint-plugin/{rule}")),
        }
    } else {
        out.push(format!("eslint-plugin-{plugin}/{rule}"));
    }
    out.dedup();
    out
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

/// Where an alias comment points.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Covers {
    /// Its own line, as `eslint-disable-line` and `# noqa`.
    Own,
    /// The next line with code, as `eslint-disable-next-line`.
    Next,
}

/// Parse one comment as another tool's line suppression, when `aliases`
/// accepts that tool. Ids are the tool's; an empty list covers every rule.
fn parse_alias(comment: &str, aliases: &[Alias]) -> Option<(Alias, Vec<String>, Option<String>, Covers)> {
    let body = comment.trim_start().trim_start_matches(['/', '#', '*', '<', '!', '-']).trim_start();
    let body = body.trim_end().trim_end_matches("*/").trim_end_matches("-->").trim_end();
    let split = |ids: &str| -> Vec<String> { ids.split(|c: char| c == ',' || c.is_whitespace()).filter(|s| !s.is_empty()).map(str::to_owned).collect() };
    if aliases.contains(&Alias::Eslint) {
        for (marker, covers) in [("eslint-disable-next-line", Covers::Next), ("eslint-disable-line", Covers::Own)] {
            if let Some(rest) = body.strip_prefix(marker)
                && (rest.is_empty() || rest.starts_with(char::is_whitespace))
            {
                // `-- reason` ends the rule list.
                let (ids, reason) = match rest.split_once("--") {
                    Some((ids, why)) => (ids, Some(why.trim()).filter(|w| !w.is_empty()).map(str::to_owned)),
                    None => (rest, None),
                };
                return Some((Alias::Eslint, split(ids), reason, covers));
            }
        }
    }
    if aliases.contains(&Alias::Biome)
        && let Some(rest) = body.strip_prefix("biome-ignore")
        && rest.starts_with(char::is_whitespace)
    {
        // `biome-ignore lint/suspicious/noDebugger: reason`; `lint` alone is
        // every lint rule, and `format` is the formatter's, not a rule.
        let (ids, reason) = rest.split_once(':').map_or((rest, None), |(i, r)| (i, Some(r.trim().to_owned()).filter(|r| !r.is_empty())));
        let ids = split(ids);
        if ids.iter().any(|i| i != "lint" && !i.starts_with("lint/")) {
            return None;
        }
        let ids = ids.into_iter().filter(|i| i != "lint").collect();
        return Some((Alias::Biome, ids, reason, Covers::Next));
    }
    if aliases.contains(&Alias::Ruff) {
        // `# noqa`, `# noqa: F401, E501`, anywhere in a comment, as Ruff
        // reads it; `# ruff: noqa` is a whole-file exemption, not a line one.
        let lower = comment.to_ascii_lowercase();
        let at = lower.match_indices("noqa").map(|(i, _)| i).find(|&i| lower[..i].trim_end().ends_with('#'))?;
        if lower[..at].trim_end().trim_end_matches('#').trim_end().ends_with(':') {
            return None;
        }
        let rest = &comment[at + 4..];
        let ids = match rest.trim_start().strip_prefix(':') {
            Some(codes) => split(codes.split('#').next().unwrap_or("")),
            None if rest.is_empty() || rest.starts_with(char::is_whitespace) || rest.starts_with('#') => Vec::new(),
            None => return None,
        };
        return Some((Alias::Ruff, ids, None, Covers::Own));
    }
    None
}

/// Find every suppression among the comment ranges (host bytes) of a file.
pub fn find(text: &str, comments: &[(usize, usize)], index: &LineIndex<'_>) -> Vec<Suppression> {
    find_with(text, comments, index, &[])
}

/// [`find`], also reading the line suppressions of the tools in `aliases`.
pub fn find_with(text: &str, comments: &[(usize, usize)], index: &LineIndex<'_>, aliases: &[Alias]) -> Vec<Suppression> {
    let mut out = Vec::new();
    for &(start, end) in comments {
        let comment = &text[start..end];
        let (rules, reason, dated, alias, covers) = match parse(comment) {
            Some((rules, reason, dated)) => (rules, reason, dated, None, None),
            None => match (!aliases.is_empty()).then(|| parse_alias(comment, aliases)).flatten() {
                // An alias needs no reason: the old tool's comment stands as it is.
                Some((tool, rules, reason, covers)) => (rules, reason.or_else(|| Some(format!("{} suppression", tool.name()))), Dated::default(), Some(tool), Some(covers)),
                None => continue,
            },
        };
        let line_start = text[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let own_line = text[line_start..start].trim().is_empty();
        let comment_line = index.position(start).line;
        let next = match covers {
            Some(c) => c == Covers::Next,
            None => own_line,
        };
        let target_line = if next {
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
        out.push(Suppression { rules, reason, start, end, target_line, own_line, used: false, dated, alias });
    }
    out
}

/// Map each alias suppression's tool ids to the codelaw rules that port
/// them; `ported` is `ported_from:` id → codelaw ids. An id no rule ports
/// covers nothing. A blanket alias (no ids) keeps an empty list.
pub fn translate(sups: &mut [Suppression], ported: &std::collections::BTreeMap<String, Vec<String>>) {
    for s in sups.iter_mut() {
        let Some(tool) = s.alias else { continue };
        let mut ids: Vec<String> = Vec::new();
        for id in &s.rules {
            for key in upstream_keys(tool, id) {
                for r in ported.get(&key).into_iter().flatten() {
                    if !ids.contains(r) {
                        ids.push(r.clone());
                    }
                }
            }
        }
        // Named ids that map to nothing must not read as a blanket.
        if ids.is_empty() && !s.rules.is_empty() {
            ids.push(String::new());
        }
        s.rules = ids;
    }
}

/// Whether a suppression covers rule `id` (on its target line).
pub fn covers(s: &Suppression, id: &str) -> bool {
    if s.alias.is_some() && s.rules.is_empty() {
        return !id.starts_with("codelaw/");
    }
    s.rules.iter().any(|r| r == id)
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

    #[test]
    fn aliases_read_the_other_tools_line_suppressions() {
        let all = &Alias::ALL[..];
        let a = |c: &str| parse_alias(c, all);
        assert_eq!(a("// eslint-disable-next-line no-console, no-debugger -- legacy"), Some((Alias::Eslint, vec!["no-console".into(), "no-debugger".into()], Some("legacy".into()), Covers::Next)));
        assert_eq!(a("/* eslint-disable-line */"), Some((Alias::Eslint, vec![], None, Covers::Own)));
        assert_eq!(a("// eslint-disable no-console"), None, "block forms are not line suppressions");
        assert_eq!(a("# noqa: F401, E501"), Some((Alias::Ruff, vec!["F401".into(), "E501".into()], None, Covers::Own)));
        assert_eq!(a("# type: ignore  # noqa"), Some((Alias::Ruff, vec![], None, Covers::Own)));
        assert_eq!(a("# ruff: noqa"), None, "a whole-file exemption");
        assert_eq!(a("# noqal"), None);
        assert_eq!(a("// biome-ignore lint/suspicious/noDebugger: the test needs it"), Some((Alias::Biome, vec!["lint/suspicious/noDebugger".into()], Some("the test needs it".into()), Covers::Next)));
        assert_eq!(a("// biome-ignore format: generated"), None);
        assert_eq!(parse_alias("# noqa", &[Alias::Eslint]), None, "only the tools the config names");
    }

    #[test]
    fn tool_ids_map_through_ported_from() {
        assert_eq!(eslint_keys("no-console"), ["eslint/no-console"]);
        assert_eq!(eslint_keys("@typescript-eslint/no-explicit-any")[0], "typescript-eslint/no-explicit-any");
        assert_eq!(eslint_keys("react-hooks/rules-of-hooks"), ["eslint-plugin-react-hooks/rules-of-hooks"]);
        assert!(eslint_keys("@eslint-react/use-state").contains(&"@eslint-react/use-state".to_owned()));
        assert_eq!(upstream_keys(Alias::Biome, "lint/suspicious/noDebugger"), ["eslint/no-debugger"]);
        assert_eq!(upstream_keys(Alias::Ruff, "f401"), ["ruff/F401"]);

        let text = "a(); // eslint-disable-line no-console\n// eslint-disable-next-line react/x\nb();\nc(); // eslint-disable-line\n";
        let idx = LineIndex::new(text);
        let comments: Vec<(usize, usize)> = text.match_indices("//").map(|(i, _)| (i, i + text[i..].find('\n').unwrap())).collect();
        let mut s = find_with(text, &comments, &idx, &[Alias::Eslint]);
        let ported = [("eslint/no-console".to_owned(), vec!["js/no-console".to_owned()])].into_iter().collect();
        translate(&mut s, &ported);
        assert_eq!((s[0].target_line, covers(&s[0], "js/no-console")), (1, true));
        assert!(!covers(&s[1], "js/no-console") && s[1].target_line == 3, "an id no rule ports covers nothing");
        assert!(covers(&s[2], "js/anything") && !covers(&s[2], "codelaw/unused-suppression"), "a blanket alias covers every rule but codelaw's own");
        assert!(find(text, &comments, &idx).is_empty(), "off unless the config names the tool");
    }
}
