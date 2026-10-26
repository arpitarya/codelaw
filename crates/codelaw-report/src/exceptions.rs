//! The exceptions report (SR-BASELINE decision 11): filter, group and
//! render the list the engine collects, as text, JSON, CSV or a GitHub
//! table. No logic of its own beyond that: both `codelaw exceptions` and
//! the MCP `list_exceptions` call `build` and `json`.

use crate::SCHEMA_VERSION;
use codelaw_engine::{Date, Exception, Exceptions};
use serde_json::{Value, json};

/// What `--expiring`, `--expired`, `--mine`, `--rule` and `--path` ask for.
#[derive(Clone, Debug, Default)]
pub struct Filter {
    /// Expires within this many days after the as-of date.
    pub expiring: Option<u32>,
    /// Already expired: `expires` on or before the as-of date.
    pub expired: bool,
    /// Added by `git config user.email`.
    pub mine: bool,
    /// Only these rule ids.
    pub rules: Vec<String>,
    /// Only exceptions covering a file at or under this project-relative path.
    pub path: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupBy {
    AddedBy,
    Rule,
    File,
    ExpiresMonth,
}

impl GroupBy {
    pub fn parse(s: &str) -> Result<GroupBy, String> {
        match s {
            "added_by" | "added-by" => Ok(GroupBy::AddedBy),
            "rule" => Ok(GroupBy::Rule),
            "file" => Ok(GroupBy::File),
            "expires-month" | "expires_month" => Ok(GroupBy::ExpiresMonth),
            other => Err(format!("unknown --group-by `{other}` (added_by, rule, file or expires-month)")),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            GroupBy::AddedBy => "added_by",
            GroupBy::Rule => "rule",
            GroupBy::File => "file",
            GroupBy::ExpiresMonth => "expires-month",
        }
    }

    fn none(self) -> &'static str {
        match self {
            GroupBy::AddedBy => "(no added_by)",
            GroupBy::Rule => "(no rule)",
            GroupBy::File => "(project)",
            GroupBy::ExpiresMonth => "(no expiry)",
        }
    }

    fn key(self, e: &Exception) -> Option<String> {
        match self {
            GroupBy::AddedBy => e.added_by.clone(),
            GroupBy::Rule => Some(e.rule.clone()),
            GroupBy::File => (e.files.len() == 1).then(|| e.files[0].clone()),
            GroupBy::ExpiresMonth => e.expires.map(|d| d.to_string()[..7].to_owned()),
        }
    }
}

/// A duration: `30d`, `4w` or a bare number of days.
pub fn parse_days(flag: &str, s: &str) -> Result<u32, String> {
    let (n, mul) = match s.strip_suffix('w') {
        Some(n) => (n, 7),
        None => (s.strip_suffix('d').unwrap_or(s), 1),
    };
    n.parse::<u32>().ok().and_then(|n| n.checked_mul(mul)).ok_or_else(|| format!("--{flag}: `{s}` is not a number of days (30d, 4w)"))
}

/// A row: an exception and the days left on the as-of date.
#[derive(Clone, Debug)]
pub struct Row<'a> {
    pub e: &'a Exception,
    /// From the as-of date to `expires`; 0 or less is expired.
    pub days_left: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct Group<'a> {
    pub key: String,
    pub rows: Vec<Row<'a>>,
}

impl Group<'_> {
    pub fn uses(&self) -> usize {
        self.rows.iter().map(|r| r.e.uses).sum()
    }
}

/// The report: the as-of line, the filtered rows, and groups when asked.
#[derive(Clone, Debug)]
pub struct Report<'a> {
    pub as_of: String,
    pub as_of_date: Option<Date>,
    pub rows: Vec<Row<'a>>,
    pub group_by: Option<GroupBy>,
    pub groups: Vec<Group<'a>>,
}

/// Filter and group. An error is a usage error: a flag the data cannot
/// answer (no as-of date for `--expiring`, no `user.email` for `--mine`).
pub fn build<'a>(all: &'a Exceptions, filter: &Filter, group_by: Option<GroupBy>) -> Result<Report<'a>, String> {
    let today = all.as_of.date;
    if (filter.expiring.is_some() || filter.expired) && today.is_none() {
        return Err(format!("--expiring and --expired need an as-of date: {}", all.as_of.source));
    }
    let me = match filter.mine {
        true => Some(all.git_email.clone().ok_or("--mine needs `git config user.email`; none is set")?),
        false => None,
    };
    let mut rows = Vec::new();
    for e in &all.items {
        let days_left = today.zip(e.expires).map(|(t, x)| t.days_until(x));
        if filter.expiring.is_some() || filter.expired {
            let Some(d) = days_left else { continue };
            let expiring = filter.expiring.is_some_and(|n| d >= 1 && d <= i64::from(n));
            if !(expiring || (filter.expired && d <= 0)) {
                continue;
            }
        }
        if me.as_ref().is_some_and(|m| !e.added_by.as_deref().is_some_and(|a| a.eq_ignore_ascii_case(m))) {
            continue;
        }
        if !filter.rules.is_empty() && !filter.rules.contains(&e.rule) {
            continue;
        }
        if let Some(p) = &filter.path {
            let p = p.trim_start_matches("./").trim_end_matches('/');
            if !p.is_empty() && p != "." && !e.files.iter().any(|f| f == p || f.starts_with(&format!("{p}/"))) {
                continue;
            }
        }
        rows.push(Row { e, days_left });
    }
    let groups = match group_by {
        None => Vec::new(),
        Some(g) => {
            let mut by: std::collections::BTreeMap<(bool, String), Vec<Row<'a>>> = Default::default();
            for r in &rows {
                let key = g.key(r.e);
                by.entry((key.is_none(), key.unwrap_or_else(|| g.none().to_owned()))).or_default().push(r.clone());
            }
            by.into_iter().map(|((_, key), rows)| Group { key, rows }).collect()
        }
    };
    Ok(Report { as_of: all.as_of.line(), as_of_date: today, rows, group_by, groups })
}

/// Whether any row has `days_left` at or under `n` (expired included):
/// `--fail-if-expiring`.
pub fn any_within(report: &Report<'_>, n: u32) -> bool {
    report.rows.iter().any(|r| r.days_left.is_some_and(|d| d <= i64::from(n)))
}

fn when(days: Option<i64>, has_expiry: bool) -> String {
    match (days, has_expiry) {
        (_, false) => "no expiry".into(),
        (None, _) => String::new(),
        (Some(d), _) if d > 1 => format!("in {d} days"),
        (Some(1), _) => "in 1 day".into(),
        (Some(0), _) => "expired today".into(),
        (Some(-1), _) => "expired 1 day ago".into(),
        (Some(d), _) => format!("expired {} days ago", -d),
    }
}

fn subject(e: &Exception) -> String {
    use codelaw_engine::ExceptionKind::*;
    match e.kind {
        Baseline => match &e.symbol {
            Some(s) => format!("{}#{}", e.files[0], s),
            None => e.files[0].clone(),
        },
        Deprecation => {
            let s = e.symbol.clone().unwrap_or_default();
            match e.files.len() {
                0 => s,
                1 => format!("{s} ({})", e.files[0]),
                n => format!("{s} ({n} files)"),
            }
        }
        Inline => format!("{}:{}", e.files[0], e.line),
    }
}

fn plural(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

impl Report<'_> {
    pub fn text(&self) -> String {
        let mut out = format!("{}\n", self.as_of);
        if self.rows.is_empty() {
            out.push_str("no exceptions\n");
            return out;
        }
        let cells = |r: &Row<'_>| {
            let e = r.e;
            let uses = if e.kind == codelaw_engine::ExceptionKind::Inline { String::new() } else { format!(" ({})", e.uses) };
            [
                e.expires.map(|d| d.to_string()).unwrap_or_default(),
                when(r.days_left, e.expires.is_some()),
                e.rule.clone(),
                format!("{}{uses}", subject(e)),
                e.reason.clone().unwrap_or_default(),
            ]
        };
        let all: Vec<[String; 5]> = self.rows.iter().map(cells).collect();
        let w: Vec<usize> = (0..4).map(|c| all.iter().map(|r| r[c].chars().count()).max().unwrap_or(0)).collect();
        let line = |r: &[String; 5]| {
            let pad = |c: usize| " ".repeat(w[c] - r[c].chars().count());
            format!("{}{}  {}{}  {}{}  {}{}  {}", r[0], pad(0), r[1], pad(1), r[2], pad(2), r[3], pad(3), r[4]).trim_end().to_owned()
        };
        if self.group_by.is_none() {
            for r in &all {
                out.push_str(&format!("{}\n", line(r)));
            }
            return out;
        }
        for g in &self.groups {
            out.push_str(&format!("{}  {}, {}\n", g.key, plural(g.rows.len(), "exception", "exceptions"), plural(g.uses(), "use", "uses")));
            for r in &g.rows {
                out.push_str(&format!("  {}\n", line(&cells(r))));
            }
        }
        out
    }

    const COLUMNS: [&'static str; 11] = ["kind", "rule", "file", "symbol", "expires", "days_left", "added_by", "reason", "uses", "at", "line"];

    fn fields(r: &Row<'_>) -> [String; 11] {
        let e = r.e;
        [
            e.kind.name().to_owned(),
            e.rule.clone(),
            e.files.join(" "),
            e.symbol.clone().unwrap_or_default(),
            e.expires.map(|d| d.to_string()).unwrap_or_default(),
            r.days_left.map(|d| d.to_string()).unwrap_or_default(),
            e.added_by.clone().unwrap_or_default(),
            e.reason.clone().unwrap_or_default(),
            e.uses.to_string(),
            e.at.clone(),
            e.line.to_string(),
        ]
    }

    /// One line per exception; a leading `group` column when grouped.
    pub fn csv(&self) -> String {
        let q = |s: &str| if s.contains([',', '"', '\n', '\r']) { format!("\"{}\"", s.replace('"', "\"\"")) } else { s.to_owned() };
        let mut out = String::new();
        let head: Vec<&str> = self.group_by.iter().map(|_| "group").chain(Self::COLUMNS).collect();
        out.push_str(&head.join(","));
        out.push('\n');
        let mut emit = |group: Option<&str>, r: &Row<'_>| {
            let cells: Vec<String> = group.iter().map(|g| q(g)).chain(Self::fields(r).iter().map(|c| q(c))).collect();
            out.push_str(&cells.join(","));
            out.push('\n');
        };
        if self.group_by.is_some() {
            for g in &self.groups {
                for r in &g.rows {
                    emit(Some(&g.key), r);
                }
            }
        } else {
            for r in &self.rows {
                emit(None, r);
            }
        }
        out
    }

    /// GitHub-flavoured tables for a PR comment: one per group.
    pub fn markdown(&self) -> String {
        let mut out = format!("{}\n", self.as_of);
        if self.rows.is_empty() {
            out.push_str("\nno exceptions\n");
            return out;
        }
        let esc = |s: &str| s.replace('|', "\\|").replace('\n', " ");
        let table = |out: &mut String, rows: &[Row<'_>]| {
            out.push_str("\n| expires | when | rule | where | uses | added by | reason |\n|---|---|---|---|--:|---|---|\n");
            for r in rows {
                let e = r.e;
                out.push_str(&format!(
                    "| {} | {} | `{}` | `{}` | {} | {} | {} |\n",
                    e.expires.map(|d| d.to_string()).unwrap_or_default(),
                    when(r.days_left, e.expires.is_some()),
                    esc(&e.rule),
                    esc(&subject(e)),
                    e.uses,
                    esc(e.added_by.as_deref().unwrap_or("")),
                    esc(e.reason.as_deref().unwrap_or("")),
                ));
            }
        };
        if self.group_by.is_none() {
            table(&mut out, &self.rows);
        } else {
            for g in &self.groups {
                out.push_str(&format!("\n### {}: {}, {}\n", g.key, plural(g.rows.len(), "exception", "exceptions"), plural(g.uses(), "use", "uses")));
                table(&mut out, &g.rows);
            }
        }
        out
    }

    /// The JSON both the CLI and the MCP tool give (schema_version 1).
    pub fn json(&self) -> Value {
        let row = |r: &Row<'_>| {
            let e = r.e;
            json!({
                "kind": e.kind.name(),
                "rule": e.rule,
                "files": e.files,
                "symbol": e.symbol,
                "expires": e.expires.map(|d| d.to_string()),
                "days_left": r.days_left,
                "added_by": e.added_by,
                "reason": e.reason,
                "uses": e.uses,
                "at": e.at,
                "line": e.line,
            })
        };
        let groups: Value = match self.group_by {
            None => Value::Null,
            Some(_) => self
                .groups
                .iter()
                .map(|g| json!({ "key": g.key, "exceptions": g.rows.len(), "uses": g.uses(), "items": g.rows.iter().map(row).collect::<Vec<_>>() }))
                .collect::<Vec<_>>()
                .into(),
        };
        json!({
            "schema_version": SCHEMA_VERSION,
            "as_of": { "date": self.as_of_date.map(|d| d.to_string()), "line": self.as_of },
            "group_by": self.group_by.map(GroupBy::name),
            "exceptions": self.rows.iter().map(row).collect::<Vec<_>>(),
            "groups": groups,
            "total": { "exceptions": self.rows.len(), "uses": self.rows.iter().map(|r| r.e.uses).sum::<usize>() },
        })
    }
}
