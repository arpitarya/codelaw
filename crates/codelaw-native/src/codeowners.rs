//! `data/codeowners-coverage` (W-132): every file of the project has an
//! owner in CODEOWNERS, every pattern matches a file, and every pattern
//! names an owner.
//!
//! The file is GitHub's or GitLab's: `.github/CODEOWNERS`, `CODEOWNERS`,
//! `docs/CODEOWNERS` or `.gitlab/CODEOWNERS`, or `[codeowners] path`.
//! Patterns are gitignore-style and the last match wins; GitLab's
//! `[Section]` lines start a section of their own, and a file is owned when
//! any section owns it. The rule reads only files (L5): the project's file
//! list comes from the context, the CODEOWNERS text from `read`.
//!
//! CODEOWNERS is not one of L3's nine languages, so the findings sit on the
//! project's root manifest — `package.json`, else `pyproject.toml`, else
//! `codelaw.toml` — once per project, each naming the CODEOWNERS line.

use crate::{Elsewhere, NativeContext, NativeFinding, NativeRule};
use codelaw_core::Language;
use codelaw_lang::SyntaxNode;
use regex::{Regex, RegexSet};
use serde_json::Value;
use std::collections::BTreeMap;

const MANIFESTS: &[Language] = &[Language::Json, Language::Toml];
const ROOT_MANIFESTS: [&str; 3] = ["package.json", "pyproject.toml", "codelaw.toml"];
/// Where CODEOWNERS may live, in the order they are tried.
const LOCATIONS: [&str; 4] = [".github/CODEOWNERS", "CODEOWNERS", "docs/CODEOWNERS", ".gitlab/CODEOWNERS"];
/// How many unowned paths a message names.
const LISTED: usize = 10;

type Options = BTreeMap<String, Value>;

/// One pattern line of a CODEOWNERS file.
#[derive(Debug, PartialEq, Eq)]
struct Entry {
    /// 1-based.
    line: usize,
    pattern: String,
    owners: Vec<String>,
    section: usize,
}

/// A parsed CODEOWNERS file.
#[derive(Debug, Default, PartialEq, Eq)]
struct Codeowners {
    entries: Vec<Entry>,
    /// The number of sections: 1 for GitHub's file, one more per `[Name]`.
    sections: usize,
}

/// Split a line at its unescaped whitespace; a `#` starts a comment.
fn tokens(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' if chars.peek().is_some_and(|n| matches!(n, ' ' | '#')) => cur.push(chars.next().unwrap_or(' ')),
            '#' if cur.is_empty() => break,
            c if c.is_whitespace() => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            c => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// A GitLab section header: `[Name]`, `^[Name]`, `[Name][2]`, any of them
/// followed by default owners. The owners, when it is one.
fn section_header(line: &str) -> Option<Vec<String>> {
    let t = line.trim_start();
    let t = t.strip_prefix('^').unwrap_or(t);
    let rest = t.strip_prefix('[')?;
    let close = rest.find(']')?;
    if close == 0 {
        return None;
    }
    let mut after = &rest[close + 1..];
    if let Some(r) = after.strip_prefix('[')
        && let Some(end) = r.find(']')
        && r[..end].chars().all(|c| c.is_ascii_digit())
    {
        after = &r[end + 1..];
    }
    Some(tokens(after))
}

fn parse(text: &str) -> Codeowners {
    let mut out = Codeowners { entries: Vec::new(), sections: 1 };
    let mut section = 0;
    let mut defaults: Vec<String> = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(owners) = section_header(line) {
            section = out.sections;
            out.sections += 1;
            defaults = owners;
            continue;
        }
        let mut t = tokens(line);
        if t.is_empty() {
            continue;
        }
        let pattern = t.remove(0);
        // A section's default owners apply to a pattern that names none.
        let owners = if t.is_empty() && section > 0 { defaults.clone() } else { t };
        out.entries.push(Entry { line: i + 1, pattern, owners, section });
    }
    out
}

/// A gitignore-style pattern as a regular expression over a project-relative
/// path. A pattern matches the files it names and everything under a folder
/// it names; one with no `/` (but a trailing one) matches at any depth.
fn pattern_regex(pattern: &str) -> Option<Regex> {
    let mut p = pattern;
    let dir_only = p.ends_with('/');
    p = p.trim_end_matches('/');
    let anchored = p.starts_with('/') || p.contains('/');
    p = p.trim_start_matches('/');
    if p.is_empty() {
        return None;
    }
    let mut re = String::from(if anchored { "^" } else { "^(?:.*/)?" });
    let chars: Vec<char> = p.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '*' if chars.get(i + 1) == Some(&'*') => {
                let leading = i == 0 || chars[i - 1] == '/';
                let trailing = i + 2 >= chars.len() || chars[i + 2] == '/';
                if leading && trailing {
                    // `**/` is any folders (or none); a trailing `/**` is everything under.
                    if chars.get(i + 2) == Some(&'/') {
                        re.push_str("(?:.*/)?");
                        i += 3;
                    } else {
                        re.push_str(".*");
                        i += 2;
                    }
                } else {
                    re.push_str("[^/]*");
                    i += 2;
                }
            }
            '*' => {
                re.push_str("[^/]*");
                i += 1;
            }
            '?' => {
                re.push_str("[^/]");
                i += 1;
            }
            '\\' if i + 1 < chars.len() => {
                re.push_str(&regex::escape(&chars[i + 1].to_string()));
                i += 2;
            }
            c => {
                re.push_str(&regex::escape(&c.to_string()));
                i += 1;
            }
        }
    }
    // The pattern names a file, or a folder and so everything in it.
    re.push_str(if dir_only { "/.*$" } else { "(?:/.*)?$" });
    Regex::new(&re).ok()
}

/// How a file stands against one section.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Standing {
    /// No pattern of the section matches it.
    Unmatched,
    /// The last pattern that matches it names owners.
    Owned,
    /// The last pattern that matches it names none.
    Disowned,
}

struct Coverage {
    /// Files no pattern owns.
    unowned: Vec<String>,
    /// Entries (indices) that match no file.
    stale: Vec<usize>,
}

fn coverage(co: &Codeowners, files: &[String], ignore: &[Regex]) -> Coverage {
    // One set per section; `members[s]` maps a set index to an entry index.
    let mut sets: Vec<(RegexSet, Vec<usize>)> = Vec::new();
    for s in 0..co.sections {
        let mut res = Vec::new();
        let mut members = Vec::new();
        for (k, e) in co.entries.iter().enumerate().filter(|(_, e)| e.section == s) {
            // A pattern that cannot be read matches nothing.
            res.push(pattern_regex(&e.pattern).map_or_else(|| "$^".to_owned(), |r| r.as_str().to_owned()));
            members.push(k);
        }
        sets.push((RegexSet::new(&res).unwrap_or_else(|_| RegexSet::empty()), members));
    }
    let mut matched = vec![false; co.entries.len()];
    let mut unowned = Vec::new();
    for f in files {
        let mut standing = Standing::Unmatched;
        for (set, members) in &sets {
            let hits = set.matches(f);
            let Some(last) = hits.iter().max() else { continue };
            for h in hits.iter() {
                matched[members[h]] = true;
            }
            let here = if co.entries[members[last]].owners.is_empty() { Standing::Disowned } else { Standing::Owned };
            standing = match (standing, here) {
                (Standing::Owned, _) | (_, Standing::Owned) => Standing::Owned,
                _ => Standing::Disowned,
            };
        }
        // A path a line deliberately leaves without an owner is that line's
        // finding (`names no owner`), not the path's.
        if standing == Standing::Unmatched && !ignore.iter().any(|r| r.is_match(f)) {
            unowned.push(f.clone());
        }
    }
    let stale = (0..co.entries.len()).filter(|k| !matched[*k]).collect();
    Coverage { unowned, stale }
}

pub struct CodeownersCoverage;

/// The CODEOWNERS file: its path and text. `Err` names a configured path
/// that is not there; `Ok(None)` is a project with no CODEOWNERS file.
fn locate(ctx: &NativeContext<'_>) -> Result<Option<(String, String)>, String> {
    if let Some(p) = ctx.options.get("path").and_then(Value::as_str) {
        return match ctx.file.read(p) {
            Some(t) => Ok(Some((p.to_owned(), t))),
            None => Err(format!("`[codeowners] path` names `{p}`, which cannot be read")),
        };
    }
    for p in LOCATIONS {
        if let Some(t) = ctx.file.read(p) {
            return Ok(Some((p.to_owned(), t)));
        }
    }
    Ok(None)
}

/// The CODEOWNERS line a finding's message names: `` `path` line 7: … ``.
fn line_of(message: &str) -> Option<usize> {
    let rest = message.split_once("` line ")?.1;
    rest.split_once(':')?.0.parse().ok()
}

fn strings(o: &Options, key: &str) -> Vec<String> {
    o.get(key).and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_str).map(str::to_owned).collect()).unwrap_or_default()
}

impl NativeRule for CodeownersCoverage {
    fn id(&self) -> &'static str {
        "data/codeowners-coverage"
    }
    fn languages(&self) -> &'static [Language] {
        MANIFESTS
    }
    fn kinds(&self) -> &'static [&'static str] {
        &["document"]
    }
    fn config_section(&self) -> Option<&'static str> {
        Some("codeowners")
    }
    /// A stale pattern and a pattern with no owner are shown on the
    /// CODEOWNERS line too (SR-ARCHITECTURE decision 15d; W-136).
    fn elsewhere(&self, finding: &NativeFinding, ctx: &NativeContext<'_>) -> Vec<Elsewhere> {
        let Some(line) = line_of(&finding.message) else { return Vec::new() };
        let Ok(Some((co_path, text))) = locate(ctx) else { return Vec::new() };
        Elsewhere::line(co_path, &text, line).into_iter().collect()
    }
    fn check(&self, _node: SyntaxNode<'_>, ctx: &NativeContext<'_>, out: &mut Vec<NativeFinding>) {
        // One set of findings per project: on the root manifest.
        let path = ctx.file.path();
        let Some(root) = ROOT_MANIFESTS.iter().find(|m| ctx.file.exists(m)) else { return };
        if path != *root {
            return;
        }
        let end = ctx.source.find('\n').unwrap_or(ctx.source.len());
        let mut say = |message: String| out.push(NativeFinding { start: 0, end, message, edits: Vec::new() });
        let (co_path, text) = match locate(ctx) {
            Ok(Some(found)) => found,
            Ok(None) => return,
            Err(why) => return say(why),
        };
        let Some(files) = ctx.file.project_files() else { return };
        let allow_unowned = ctx.options.get("allow_unowned").and_then(Value::as_bool).unwrap_or(false);
        let ignore: Vec<Regex> = strings(ctx.options, "ignore").iter().filter_map(|p| pattern_regex(p)).collect();
        let co = parse(&text);
        let c = coverage(&co, &files, &ignore);
        if !c.unowned.is_empty() {
            let n = c.unowned.len();
            let shown: Vec<String> = c.unowned.iter().take(LISTED).map(|p| format!("`{p}`")).collect();
            let more = if n > LISTED { format!(", and {} more", n - LISTED) } else { String::new() };
            let noun = if n == 1 { "file has" } else { "files have" };
            say(format!("{n} {noun} no owner in `{co_path}`: {}{more}; add a pattern for them, or list them in `[codeowners] ignore`", shown.join(", ")));
        }
        for k in c.stale {
            let e = &co.entries[k];
            say(format!("`{co_path}` line {}: the pattern `{}` matches no file; delete the line, or fix the pattern", e.line, e.pattern));
        }
        if !allow_unowned {
            for e in co.entries.iter().filter(|e| e.owners.is_empty()) {
                say(format!("`{co_path}` line {}: the pattern `{}` names no owner; add one, or set `[codeowners] allow_unowned = true` if the path is meant to have none", e.line, e.pattern));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn m(pattern: &str, path: &str) -> bool {
        pattern_regex(pattern).is_some_and(|r| r.is_match(path))
    }

    #[test]
    fn patterns() {
        assert!(m("*", "a/b/c.js"));
        assert!(m("*.js", "a/b/c.js"));
        assert!(!m("*.js", "a/b/c.ts"));
        assert!(m("/docs/", "docs/a.md"));
        assert!(!m("/docs/", "src/docs/a.md"));
        assert!(m("docs/", "src/docs/a.md"));
        assert!(m("docs", "src/docs/a.md"));
        assert!(m("/src/*.js", "src/a.js"));
        assert!(!m("/src/*.js", "src/x/a.js"));
        assert!(m("src/**/test.js", "src/test.js"));
        assert!(m("src/**/test.js", "src/a/b/test.js"));
        assert!(m("/src/**", "src/a/b.js"));
        assert!(m("a?.js", "ab.js"));
        assert!(!m("a?.js", "a/.js"));
        assert!(m("/build/logs/", "build/logs/x.log"));
        assert!(!m("/build/logs/", "build/logs"));
        assert!(m("README.md", "docs/README.md"));
        assert!(!m("/README.md", "docs/README.md"));
    }

    #[test]
    fn parsing_github_and_gitlab() {
        let co = parse("# owners\n\n*  @org/all\n/docs/ @docs   # trailing\n/tmp/\n");
        assert_eq!(co.sections, 1);
        assert_eq!(co.entries.len(), 3);
        assert_eq!(co.entries[1], Entry { line: 4, pattern: "/docs/".into(), owners: vec!["@docs".into()], section: 0 });
        assert!(co.entries[2].owners.is_empty());
        let gl = parse("* @all\n[Docs] @docs\n/docs/\n^[Opt][2]\n/x/ @x\n");
        assert_eq!(gl.sections, 3);
        assert_eq!(gl.entries[1].owners, ["@docs"], "a section's default owner");
        assert_eq!(gl.entries[1].section, 1);
        assert_eq!(gl.entries[2].section, 2);
        assert_eq!(tokens(r"a\ b @o"), ["a b", "@o"]);
    }

    fn files(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn last_match_wins_and_gaps() {
        let co = parse("/src/ @a\n/src/gen/\n/docs/ @d\n");
        let f = files(&["src/a.js", "src/gen/x.js", "docs/a.md", "README.md"]);
        let c = coverage(&co, &f, &[]);
        // `/src/gen/` names no owner: that line is the finding, not its files
        assert_eq!(c.unowned, ["README.md"]);
        assert!(c.stale.is_empty());
    }

    #[test]
    fn stale_and_ignore() {
        let co = parse("* @a\n/old/ @a\n*.png @a\n");
        let f = files(&["a.js", "img/b.png", "pnpm-lock.yaml"]);
        let c = coverage(&co, &f, &[]);
        assert_eq!(c.stale, [1]);
        let none = parse("/src/ @a\n");
        let ignore = vec![pattern_regex("*.lock").unwrap()];
        let c = coverage(&none, &files(&["src/a.js", "x.lock", "y.txt"]), &ignore);
        assert_eq!(c.unowned, ["y.txt"]);
    }

    #[test]
    fn sections_any_owner() {
        let co = parse("[A] @a\n/a/\n[B] @b\n/b/\n");
        let c = coverage(&co, &files(&["a/x", "b/y", "c/z"]), &[]);
        assert_eq!(c.unowned, ["c/z"]);
        assert!(c.stale.is_empty());
    }

    struct Repo {
        path: String,
        files: Vec<(String, String)>,
    }
    impl crate::FileContext for Repo {
        fn path(&self) -> &str {
            &self.path
        }
        fn bytes(&self) -> &[u8] {
            &[]
        }
        fn exists(&self, p: &str) -> bool {
            self.files.iter().any(|(f, _)| f == p)
        }
        fn is_dir(&self, _: &str) -> bool {
            false
        }
        fn read(&self, p: &str) -> Option<String> {
            self.files.iter().find(|(f, _)| f == p).map(|(_, t)| t.clone())
        }
        fn list(&self, _: &str) -> Option<Vec<String>> {
            None
        }
        fn project_files(&self) -> Option<Vec<String>> {
            let mut v: Vec<String> = self.files.iter().map(|(f, _)| f.clone()).collect();
            v.sort();
            Some(v)
        }
    }

    fn run_rule(path: &str, files: &[(&str, &str)], options: Value) -> Vec<String> {
        let src = "{}";
        let root = codelaw_lang::parse(Language::Json, src);
        let options: Options = serde_json::from_value(options).unwrap();
        let data = BTreeMap::new();
        let repo = Repo { path: path.into(), files: files.iter().map(|(p, t)| (p.to_string(), t.to_string())).collect() };
        let ctx = NativeContext { source: src, language: Language::Json, semantic: None, options: &options, data: &data, file: &repo };
        let mut out = Vec::new();
        for node in root.root().dfs() {
            if CodeownersCoverage.kinds().contains(&node.kind().as_ref()) {
                CodeownersCoverage.check(node, &ctx, &mut out);
            }
        }
        out.into_iter().map(|f| f.message).collect()
    }

    #[test]
    fn the_rule_reports_on_the_root_manifest_only() {
        let files = [("package.json", "{}"), ("src/a.js", ""), ("docs/b.md", ""), (".github/CODEOWNERS", "/src/ @a\n/gone/ @a\n/docs/\n")];
        let got = run_rule("package.json", &files, json!({}));
        assert_eq!(got.len(), 3, "{got:?}");
        assert!(got[0].starts_with("2 files have no owner in `.github/CODEOWNERS`: `.github/CODEOWNERS`, `package.json`"), "{got:?}");
        assert!(got[1].contains("line 2") && got[1].contains("`/gone/`"), "{got:?}");
        assert!(got[2].contains("line 3") && got[2].contains("names no owner"), "{got:?}");
        assert!(run_rule("src/package.json", &files, json!({})).is_empty());
        let got = run_rule("package.json", &files, json!({ "allow_unowned": true }));
        assert_eq!(got.len(), 2, "{got:?}");
        assert!(!got.iter().any(|m| m.contains("names no owner")));
    }

    #[test]
    fn no_codeowners_no_findings_and_a_missing_configured_path() {
        let files = [("package.json", "{}"), ("a.js", "")];
        assert!(run_rule("package.json", &files, json!({})).is_empty());
        let got = run_rule("package.json", &files, json!({ "path": "OWNERS" }));
        assert_eq!(got, ["`[codeowners] path` names `OWNERS`, which cannot be read"]);
        let files = [("pyproject.toml", ""), ("a.py", ""), ("CODEOWNERS", "* @a\n")];
        let got = run_rule("pyproject.toml", &files, json!({}));
        assert!(got.is_empty(), "{got:?}");
    }

    #[test]
    fn a_line_finding_is_shown_on_the_codeowners_line() {
        assert_eq!(line_of("`.github/CODEOWNERS` line 12: the pattern `/x/` matches no file"), Some(12));
        assert_eq!(line_of("3 files have no owner in `CODEOWNERS`: `a`"), None);
        let src = "{}";
        let root = codelaw_lang::parse(Language::Json, src);
        let options: Options = BTreeMap::new();
        let data = BTreeMap::new();
        let repo = Repo { path: "package.json".into(), files: vec![("package.json".into(), "{}".into()), ("CODEOWNERS".into(), "* @a\n/gone/ @a\n".into())] };
        let ctx = NativeContext { source: src, language: Language::Json, semantic: None, options: &options, data: &data, file: &repo };
        let mut out = Vec::new();
        for node in root.root().dfs() {
            if CodeownersCoverage.kinds().contains(&node.kind().as_ref()) {
                CodeownersCoverage.check(node, &ctx, &mut out);
            }
        }
        assert_eq!(out.len(), 1, "{out:?}");
        let at = CodeownersCoverage.elsewhere(&out[0], &ctx);
        assert_eq!(at, [Elsewhere::new("CODEOWNERS", 5, 14)]);
    }

    #[test]
    fn many_unowned_paths_are_summarised() {
        let mut names: Vec<String> = (0..14).map(|i| format!("f{i:02}.js")).collect();
        names.push("package.json".into());
        names.push("CODEOWNERS".into());
        let files: Vec<(&str, &str)> = names.iter().map(|n| (n.as_str(), if n == "CODEOWNERS" { "/other/ @a\n" } else { "" })).collect();
        let got = run_rule("package.json", &files, json!({}));
        assert!(got[0].starts_with("16 files have no owner in `CODEOWNERS`: "), "{got:?}");
        assert!(got[0].contains(", and 6 more;"), "{got:?}");
    }
}
