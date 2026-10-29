//! The verdicts (SR-PROJECT-GRAPH decision 11): each read against the flag
//! it names — registered, expiring, expired, decided — each inline marker
//! against the registry, and each registry entry against the reads left.
//! Dates are judged on the as-of date only, never the clock (L5).

use super::extract::{FlagFacts, MARKER, Marker};
use super::patterns::{PRESETS, Pattern, preset};
use super::{Accessor, analyze};
use crate::facts::Span;
use crate::link::Graph;
use codelaw_core::{Severity, TextEdit};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// The registry, relative to the project root.
pub const REGISTRY_REL: &str = ".codelaw/flags.toml";

/// `[flags]`, and the flag rules' own options over it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlagOptions {
    pub presets: Vec<String>,
    pub accessors: Vec<String>,
    /// `inline = "allow"` (the default); `"registry-only"` forbids markers.
    pub inline: bool,
    pub warn_days: i64,
}

impl Default for FlagOptions {
    fn default() -> Self {
        FlagOptions { presets: Vec::new(), accessors: Vec::new(), inline: true, warn_days: 14 }
    }
}

fn strings(v: Option<&Value>) -> Vec<String> {
    match v {
        Some(Value::Array(a)) => a.iter().filter_map(Value::as_str).map(str::to_owned).collect(),
        Some(Value::String(s)) => vec![s.clone()],
        _ => Vec::new(),
    }
}

impl FlagOptions {
    pub fn from_options(o: &BTreeMap<String, Value>) -> Result<FlagOptions, String> {
        let mut f = FlagOptions { presets: strings(o.get("presets")), accessors: strings(o.get("accessors")), ..Default::default() };
        for p in &f.presets {
            if !PRESETS.contains(&p.as_str()) {
                return Err(format!("`[flags] presets` names `{p}`, which is not a preset ({})", PRESETS.join(", ")));
            }
        }
        match o.get("inline").and_then(Value::as_str) {
            None | Some("allow") => {}
            Some("registry-only") => f.inline = false,
            Some(other) => return Err(format!("`[flags] inline = \"{other}\"` must be \"allow\" or \"registry-only\"")),
        }
        match o.get("warn_days") {
            None => {}
            Some(Value::Number(n)) => f.warn_days = n.as_i64().filter(|n| *n >= 0).ok_or("`[flags] warn_days` must be a number of days, 0 or more")?,
            Some(Value::String(s)) => f.warn_days = parse_days(s).ok_or_else(|| format!("`[flags] warn_days = \"{s}\"` is not a number of days (`14`, `14d`, `2w`)"))?,
            Some(_) => return Err("`[flags] warn_days` must be a number of days".into()),
        }
        Ok(f)
    }

    /// Whether any accessor is configured: with none, nothing is read.
    pub fn is_on(&self) -> bool {
        !self.presets.is_empty() || !self.accessors.is_empty()
    }

    /// The patterns in force and where each comes from: `accessors` first,
    /// then the presets in the order listed.
    pub fn patterns(&self) -> Result<(Vec<Pattern>, Vec<String>), String> {
        let mut out = Vec::new();
        let mut origins = Vec::new();
        let mut seen = BTreeSet::new();
        for a in &self.accessors {
            let p = Pattern::parse(a)?;
            if seen.insert(p.text.clone()) {
                out.push(p);
                origins.push("config".to_owned());
            }
        }
        for name in &self.presets {
            let (js, py) = preset(name).ok_or_else(|| format!("unknown preset `{name}`"))?;
            for (t, lang) in js.iter().map(|t| (t, "js")).chain(py.iter().map(|t| (t, "py"))) {
                let mut p = Pattern::parse(t).expect("preset patterns parse");
                if lang == "js" {
                    p.py = false;
                } else {
                    p.js = false;
                }
                let key = format!("{lang}:{t}");
                if seen.insert(key) {
                    out.push(p);
                    origins.push(format!("preset {name}"));
                }
            }
        }
        Ok((out, origins))
    }
}

fn parse_days(s: &str) -> Option<i64> {
    let s = s.trim();
    if let Some(n) = s.strip_suffix('d') {
        return n.parse().ok();
    }
    if let Some(n) = s.strip_suffix('w') {
        return n.parse::<i64>().ok().map(|n| n * 7);
    }
    s.parse().ok()
}

/// Days since 1970-01-01 of a `YYYY-MM-DD` date.
pub(crate) fn date_days(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let y: i64 = s[0..4].parse().ok()?;
    let m: i64 = s[5..7].parse().ok()?;
    let d: i64 = s[8..10].parse().ok()?;
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let dim = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    if !(1..=12).contains(&m) || d < 1 || d > dim[(m - 1) as usize] {
        return None;
    }
    // Howard Hinnant's days_from_civil.
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146097 + doe - 719468)
}

/// Which rule a finding belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FlagCheck {
    /// At each read: `js/flag-expiry`, `py/flag-expiry`.
    Reads,
    /// At each inline marker: `js/flag-marker`, `py/flag-marker`.
    Markers,
    /// In the registry: `data/flag-registry`.
    Registry,
}

/// One finding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlagFinding {
    pub check: FlagCheck,
    pub file: String,
    pub span: Span,
    pub message: String,
    pub severity: Severity,
    pub edits: Vec<TextEdit>,
    /// Why the finding is uncertain: reported as `info`, never fixed.
    pub uncertain: Option<String>,
}

/// The accessors in force and every finding.
pub struct FlagReport {
    pub accessors: Vec<super::Accessor>,
    pub findings: Vec<FlagFinding>,
}

/// What one flag is declared to be, by the registry or by a marker.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Decl {
    expires: Option<String>,
    owner: Option<String>,
    ticket: Option<String>,
    keep: Option<String>,
    reason: Option<String>,
}

impl Decl {
    fn of_marker(m: &Marker) -> Decl {
        Decl { expires: m.expires.clone(), owner: m.owner.clone(), ticket: m.ticket.clone(), keep: m.keep.clone(), reason: m.reason.clone() }
    }

    /// The marker that says what this declaration says.
    fn marker(&self, name: &str) -> String {
        let mut s = format!("{MARKER} {name}");
        for (k, v) in [("expires", &self.expires), ("owner", &self.owner), ("ticket", &self.ticket), ("keep", &self.keep), ("reason", &self.reason)] {
            if let Some(v) = v {
                s.push_str(&format!(" {k} {v}"));
            }
        }
        s
    }

    /// `(owner @x, ticket T)`, or nothing.
    fn who(&self) -> String {
        let parts: Vec<String> = [("owner", &self.owner), ("ticket", &self.ticket)].iter().filter_map(|(k, v)| v.as_ref().map(|v| format!("{k} {v}"))).collect();
        if parts.is_empty() { String::new() } else { format!(" ({})", parts.join(", ")) }
    }

    /// What is wrong with it, if anything.
    fn problem(&self) -> Option<String> {
        match self.expires.as_deref() {
            None => return Some("has no `expires` (a date, `YYYY-MM-DD`, or `never` with a `reason`)".into()),
            Some("never") if self.reason.is_none() => return Some("expires `never` without a `reason`".into()),
            Some("never") => {}
            Some(d) if date_days(d).is_none() => return Some(format!("has `expires = {d}`, which is neither a date (`YYYY-MM-DD`) nor `never`")),
            _ => {}
        }
        match self.keep.as_deref() {
            None | Some("on") | Some("off") => None,
            Some(k) => Some(format!("has `keep = {k}`, which is neither `on` nor `off`")),
        }
    }
}

/// One `[[flag]]` of the registry.
struct Entry {
    name: Option<String>,
    decl: Decl,
    /// The `[[flag]]` line.
    header: Span,
    /// The whole entry, for its removal.
    block: Span,
    /// Each key's line.
    keys: Vec<(String, Span)>,
    unknown: Vec<String>,
}

fn line_spans(text: &str) -> Vec<Span> {
    let mut out = Vec::new();
    let mut at = 0;
    for l in text.split_inclusive('\n') {
        let content = l.trim_end_matches(['\n', '\r']);
        out.push((at, at + content.len()));
        at += l.len();
    }
    out
}

/// The registry's entries, or why it cannot be read.
fn registry(text: &str) -> Result<Vec<Entry>, String> {
    let table: toml::Table = toml::from_str(text).map_err(|e| e.message().to_owned())?;
    let flags = match table.get("flag") {
        None => return Ok(Vec::new()),
        Some(toml::Value::Array(a)) => a,
        Some(_) => return Err("`flag` must be an array of tables (`[[flag]]`)".into()),
    };
    let lines = line_spans(text);
    let headers: Vec<usize> = lines.iter().enumerate().filter(|(_, (s, e))| text[*s..*e].trim_start().starts_with("[[flag]]")).map(|(i, _)| i).collect();
    let spans_ok = headers.len() == flags.len();
    let first = lines.first().copied().unwrap_or((0, 0));
    let mut out = Vec::new();
    for (n, f) in flags.iter().enumerate() {
        let Some(t) = f.as_table() else { return Err("every `flag` entry must be a table".into()) };
        let get = |k: &str| -> Option<String> {
            match t.get(k)? {
                toml::Value::String(s) => Some(s.clone()),
                toml::Value::Datetime(d) => Some(d.to_string()),
                other => Some(other.to_string()),
            }
        };
        let decl = Decl { expires: get("expires"), owner: get("owner"), ticket: get("ticket"), keep: get("keep"), reason: get("reason") };
        let unknown = t.keys().filter(|k| !["name", "owner", "ticket", "expires", "keep", "reason"].contains(&k.as_str())).cloned().collect();
        let (header, block, keys) = if spans_ok {
            let h = headers[n];
            let end_line = lines.iter().enumerate().skip(h + 1).find(|(_, (s, e))| text[*s..*e].trim_start().starts_with('[')).map(|(i, _)| i).unwrap_or(lines.len());
            let block_end = lines.get(end_line).map(|l| l.0).unwrap_or(text.len());
            // The last entry takes the blank lines before it, so none pile up
            // at the end of the file when it goes.
            let block_start = if end_line == lines.len() {
                let kept = text[..lines[h].0].trim_end_matches(['\n', '\r', ' ', '\t']).len();
                if kept == 0 { 0 } else { kept + 1 }
            } else {
                lines[h].0
            };
            let keys = lines[h + 1..end_line]
                .iter()
                .filter_map(|(s, e)| {
                    let l = text[*s..*e].trim_start();
                    let k = l.split(|c: char| c == '=' || c.is_whitespace()).next()?;
                    (!k.is_empty() && !k.starts_with('#')).then(|| (k.to_owned(), (*s + (text[*s..*e].len() - l.len()), *e)))
                })
                .collect();
            (lines[h], (block_start.min(lines[h].0), block_end), keys)
        } else {
            (first, first, Vec::new())
        };
        out.push(Entry { name: get("name"), decl, header, block, keys, unknown });
    }
    Ok(out)
}

impl Entry {
    fn at(&self, key: &str) -> Span {
        self.keys.iter().find(|(k, _)| k == key).map(|(_, s)| *s).unwrap_or(self.header)
    }
}

struct Out(Vec<FlagFinding>);

impl Out {
    fn push(&mut self, check: FlagCheck, file: &str, span: Span, severity: Severity, message: String, edits: Vec<TextEdit>) {
        self.0.push(FlagFinding { check, file: file.to_owned(), span, message, severity, edits, uncertain: None });
    }
}

/// Every flag finding of a project. `files` holds the facts of every JS,
/// TS and Python file; `registry` the text of `.codelaw/flags.toml`, when
/// there is one; `as_of` the as-of date.
pub fn check(graph: &Graph, files: &BTreeMap<String, FlagFacts>, registry_text: Option<&str>, options: &FlagOptions, as_of: Option<&str>) -> Result<FlagReport, String> {
    let (patterns, origins) = options.patterns()?;
    let analysis = analyze(graph, files, &patterns, &origins);
    let accessors: Vec<Accessor> = analysis.accessors;
    let mut out = Out(Vec::new());
    let today = as_of.and_then(date_days);
    let dynamic = analysis.reads.iter().any(|r| r.name.is_err());
    let used: BTreeSet<&str> = analysis.reads.iter().filter_map(|r| r.name.as_deref().ok()).collect();

    // The registry.
    let mut declared: BTreeMap<String, Decl> = BTreeMap::new();
    let mut readable = true;
    if let Some(text) = registry_text {
        match registry(text) {
            Err(why) => {
                readable = false;
                let first = line_spans(text).first().copied().unwrap_or((0, 0));
                out.push(FlagCheck::Registry, REGISTRY_REL, first, Severity::Error, format!("`{REGISTRY_REL}` cannot be read: {why}"), Vec::new());
            }
            Ok(entries) => {
                for e in &entries {
                    let Some(name) = &e.name else {
                        out.push(FlagCheck::Registry, REGISTRY_REL, e.header, Severity::Error, "This `[[flag]]` has no `name`".into(), Vec::new());
                        continue;
                    };
                    for k in &e.unknown {
                        out.push(FlagCheck::Registry, REGISTRY_REL, e.at(k), Severity::Error, format!("Flag `{name}` has `{k}`, which is not a key of a flag (name, owner, ticket, expires, keep, reason)"), Vec::new());
                    }
                    if declared.contains_key(name) {
                        out.push(FlagCheck::Registry, REGISTRY_REL, e.header, Severity::Error, format!("Flag `{name}` is registered twice; keep one entry"), Vec::new());
                        continue;
                    }
                    if let Some(p) = e.decl.problem() {
                        let key = if p.contains("keep") { "keep" } else { "expires" };
                        out.push(FlagCheck::Registry, REGISTRY_REL, e.at(key), Severity::Error, format!("Flag `{name}` {p}"), Vec::new());
                    }
                    declared.insert(name.clone(), e.decl.clone());
                    if !used.contains(name.as_str()) {
                        let message = format!("Flag `{name}` has no read left in the project: remove its entry");
                        if dynamic {
                            out.0.push(FlagFinding {
                                check: FlagCheck::Registry,
                                file: REGISTRY_REL.into(),
                                span: e.header,
                                message,
                                severity: Severity::Info,
                                edits: Vec::new(),
                                uncertain: Some("some reads compute their flag's name".into()),
                            });
                        } else {
                            out.push(FlagCheck::Registry, REGISTRY_REL, e.header, Severity::Info, message, vec![TextEdit::new(e.block.0, e.block.1, String::new())]);
                        }
                    }
                }
            }
        }
    }

    // The inline markers.
    let mut marked: BTreeMap<String, Vec<(&str, &Marker)>> = BTreeMap::new();
    for (file, facts) in files {
        for m in &facts.markers {
            if let Some(why) = &m.bad {
                out.push(FlagCheck::Markers, file, m.span, Severity::Error, format!("This `{MARKER}` marker cannot be read: {why}"), Vec::new());
                continue;
            }
            if !options.inline {
                let edits = if declared.contains_key(&m.name) { vec![TextEdit::new(m.removal.0, m.removal.1, String::new())] } else { Vec::new() };
                let tail = if edits.is_empty() { format!("; move it into `{REGISTRY_REL}`") } else { format!("; `{REGISTRY_REL}` already registers `{}`", m.name) };
                out.push(FlagCheck::Markers, file, m.span, Severity::Error, format!("Inline flag markers are off (`[flags] inline = \"registry-only\"`){tail}"), edits);
                continue;
            }
            if let Some(reg) = declared.get(&m.name) {
                if Decl::of_marker(m) != *reg {
                    let fixed = reg.marker(&m.name);
                    out.push(
                        FlagCheck::Markers,
                        file,
                        m.span,
                        Severity::Warn,
                        format!("This marker of `{}` disagrees with `{REGISTRY_REL}`, which wins", m.name),
                        vec![TextEdit::new(m.span.0, m.span.1, fixed)],
                    );
                }
            } else {
                marked.entry(m.name.clone()).or_default().push((file.as_str(), m));
            }
        }
    }
    for (name, ms) in &marked {
        let first = Decl::of_marker(ms[0].1);
        if ms.iter().any(|(_, m)| Decl::of_marker(m) != first) {
            for (file, m) in ms {
                out.push(
                    FlagCheck::Markers,
                    file,
                    m.span,
                    Severity::Error,
                    format!("Flag `{name}` is marked in {} places with different values; register it once in `{REGISTRY_REL}`", ms.len()),
                    Vec::new(),
                );
            }
            // Registered, but by no one date: the markers' findings say so.
            declared.insert(name.clone(), Decl::default());
            continue;
        }
        if let Some(p) = first.problem() {
            for (file, m) in ms {
                out.push(FlagCheck::Markers, file, m.span, Severity::Error, format!("The marker of `{name}` {p}"), Vec::new());
            }
        }
        if !used.contains(name.as_str()) {
            for (file, m) in ms {
                let message = format!("Flag `{name}` has no read left in the project: remove its marker");
                out.0.push(FlagFinding {
                    check: FlagCheck::Markers,
                    file: (*file).to_owned(),
                    span: m.span,
                    message,
                    severity: Severity::Info,
                    edits: if dynamic { Vec::new() } else { vec![TextEdit::new(m.removal.0, m.removal.1, String::new())] },
                    uncertain: dynamic.then(|| "some reads compute their flag's name".to_owned()),
                });
            }
        }
        declared.insert(name.clone(), first);
    }

    // The reads.
    if readable {
        for r in &analysis.reads {
            let name = match &r.name {
                Ok(n) => n,
                Err(why) => {
                    out.0.push(FlagFinding {
                        check: FlagCheck::Reads,
                        file: r.file.clone(),
                        span: r.span,
                        message: "A flag is read here, but its name is not known, so its expiry cannot be checked".into(),
                        severity: Severity::Info,
                        edits: Vec::new(),
                        uncertain: Some(why.clone()),
                    });
                    continue;
                }
            };
            let Some(d) = declared.get(name) else {
                let how = if options.inline { format!(", or mark it `{MARKER} {name} expires YYYY-MM-DD owner …`") } else { String::new() };
                out.push(FlagCheck::Reads, &r.file, r.span, Severity::Error, format!("Flag `{name}` is read here but is not registered: add it to `{REGISTRY_REL}` with an owner and an expiry date{how}"), Vec::new());
                continue;
            };
            let Some(exp) = d.expires.as_deref().filter(|e| *e != "never").and_then(|e| date_days(e).map(|days| (e, days))) else { continue };
            let edits = match (d.keep.as_deref(), &r.collapse) {
                (Some("on"), Some(c)) => vec![TextEdit::new(c.on.0.0, c.on.0.1, c.on.1.clone())],
                (Some("off"), Some(c)) => vec![TextEdit::new(c.off.0.0, c.off.0.1, c.off.1.clone())],
                _ => Vec::new(),
            };
            let keep = d.keep.as_deref().map(|k| format!("; it is decided (`keep = \"{k}\"`): keep that branch")).unwrap_or_default();
            let who = d.who();
            let left = today.map(|t| exp.1 - t);
            let (severity, message) = match left {
                Some(l) if l < 0 => (Severity::Error, format!("Flag `{name}` expired on {} ({} days before the as-of date){who}: remove it{keep}", exp.0, -l)),
                Some(l) if l <= options.warn_days => (Severity::Warn, format!("Flag `{name}` expires on {}, in {l} days{who}{keep}", exp.0)),
                _ if d.keep.is_some() => (Severity::Warn, format!("Flag `{name}` is decided (`keep = \"{}\"`){who}: keep that branch and remove the read", d.keep.as_deref().unwrap_or_default())),
                _ => continue,
            };
            out.push(FlagCheck::Reads, &r.file, r.span, severity, message, edits);
        }
    }
    let mut findings = out.0;
    findings.sort_by(|a, b| (a.check, &a.file, a.span).cmp(&(b.check, &b.file, b.span)));
    findings.dedup();
    Ok(FlagReport { accessors, findings })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates() {
        assert_eq!(date_days("1970-01-01"), Some(0));
        assert_eq!(date_days("2026-10-30").unwrap() - date_days("2026-10-01").unwrap(), 29);
        assert_eq!(date_days("2024-02-29").unwrap() + 1, date_days("2024-03-01").unwrap());
        assert!(date_days("2026-02-30").is_none());
        assert!(date_days("2026-1-01").is_none());
    }

    #[test]
    fn options() {
        let o: BTreeMap<String, Value> = serde_json::from_str(r#"{"presets":["unleash"],"inline":"registry-only","warn_days":"2w"}"#).unwrap();
        let f = FlagOptions::from_options(&o).unwrap();
        assert!(!f.inline && f.warn_days == 14 && f.is_on());
        let bad: BTreeMap<String, Value> = serde_json::from_str(r#"{"presets":["nope"]}"#).unwrap();
        assert!(FlagOptions::from_options(&bad).unwrap_err().contains("not a preset"));
    }

    #[test]
    fn registry_entries_and_their_lines() {
        let text = "# flags\n[[flag]]\nname = \"a\"\nexpires = 2027-01-01\n\n[[flag]]\nname = \"b\"\nexpires = \"never\"\ncolour = \"red\"\n";
        let e = registry(text).unwrap();
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].decl.expires.as_deref(), Some("2027-01-01"));
        assert_eq!(&text[e[0].block.0..e[0].block.1], "[[flag]]\nname = \"a\"\nexpires = 2027-01-01\n\n");
        assert_eq!(e[1].unknown, vec!["colour".to_owned()]);
        assert_eq!(&text[e[1].at("colour").0..e[1].at("colour").1], "colour = \"red\"");
        assert_eq!(e[1].decl.problem().as_deref(), Some("expires `never` without a `reason`"));
    }
}
