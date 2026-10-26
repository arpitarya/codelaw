//! `.codelawignore` (SR-BASELINE decision 12, W-71): the exemptions from the
//! test rules of new code, one per line, at the project root.
//!
//! ```text
//! src/utils/http.ts#get        forever: thin wrapper over fetch
//! scripts/*.py                 on_touch=error expires=2027-01-31: one-off scripts
//! app/billing.py#Invoice.legacy_total  on_touch=error on_touch_scope=function expires=2027-03-01 by=lead@example.com: replaced in ORD-1500
//! ```
//!
//! A line is a target — a `.gitignore`-style glob, or `<path>#<qualified
//! name>` for one function — then keys, then `: <reason>`. `#` starts a
//! comment only at the start of a line. What a line must carry is checked
//! here against `[coverage.ignore]`; a line that fails covers nothing.
//! Nothing here reads the clock, git or the files (L5 decision 2).

use crate::date::Date;
use crate::exception::valid_email;

/// The file's name, at the project root.
pub const IGNORE_FILE: &str = ".codelawignore";
/// A line is missing a field or contradicts itself.
pub const INCOMPLETE_RULE: &str = "codelaw/incomplete-ignore";
/// A line matches no file or function.
pub const STALE_IGNORE_RULE: &str = "codelaw/stale-ignore";

/// What an edit does to an exemption.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IgnoreOnTouch {
    /// Revoked: the rule's findings come back at their own severity.
    Error,
    /// Revoked, the findings come back as warnings.
    Warn,
    /// The exemption holds.
    Keep,
}

impl IgnoreOnTouch {
    pub fn parse(s: &str) -> Result<IgnoreOnTouch, String> {
        match s {
            "error" => Ok(IgnoreOnTouch::Error),
            "warn" => Ok(IgnoreOnTouch::Warn),
            "keep" => Ok(IgnoreOnTouch::Keep),
            _ => Err(format!("`on_touch={s}` is not one of error, warn, keep")),
        }
    }
}

/// What counts as an edit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TouchScope {
    /// The file is in the change set at all.
    File,
    /// The function is edited by tokens.
    Function,
}

impl TouchScope {
    pub fn parse(s: &str) -> Result<TouchScope, String> {
        match s {
            "file" => Ok(TouchScope::File),
            "function" => Ok(TouchScope::Function),
            _ => Err(format!("`on_touch_scope={s}` is not one of file, function")),
        }
    }
}

/// `[coverage.ignore]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IgnorePolicy {
    /// Every line says `expires=` or `forever`.
    pub require_expiry: bool,
    /// Every line that is not `forever` says `on_touch=`.
    pub require_on_touch: bool,
    /// For a line that omits `on_touch=`.
    pub on_touch: IgnoreOnTouch,
    /// For a line that omits `on_touch_scope=`.
    pub on_touch_scope: TouchScope,
}

impl Default for IgnorePolicy {
    fn default() -> Self {
        IgnorePolicy { require_expiry: true, require_on_touch: true, on_touch: IgnoreOnTouch::Error, on_touch_scope: TouchScope::File }
    }
}

impl IgnorePolicy {
    /// The table's values; the loader has already refused an unknown key.
    pub fn from_table(require_expiry: Option<bool>, require_on_touch: Option<bool>, on_touch: Option<&str>, on_touch_scope: Option<&str>) -> Result<IgnorePolicy, String> {
        let d = IgnorePolicy::default();
        Ok(IgnorePolicy {
            require_expiry: require_expiry.unwrap_or(d.require_expiry),
            require_on_touch: require_on_touch.unwrap_or(d.require_on_touch),
            on_touch: on_touch.map(IgnoreOnTouch::parse).transpose().map_err(|e| format!("[coverage.ignore]: {e}"))?.unwrap_or(d.on_touch),
            on_touch_scope: on_touch_scope.map(TouchScope::parse).transpose().map_err(|e| format!("[coverage.ignore]: {e}"))?.unwrap_or(d.on_touch_scope),
        })
    }
}

/// What a line exempts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IgnoreTarget {
    /// Every function in the files the glob matches.
    Glob(String),
    /// One function: the project-relative path and its qualified name.
    Function { path: String, name: String },
}

impl IgnoreTarget {
    pub fn text(&self) -> String {
        match self {
            IgnoreTarget::Glob(g) => g.clone(),
            IgnoreTarget::Function { path, name } => format!("{path}#{name}"),
        }
    }
}

/// One line of `.codelawignore`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IgnoreEntry {
    /// 1-based.
    pub line: u32,
    pub target: IgnoreTarget,
    pub forever: bool,
    pub expires: Option<Date>,
    pub warn_days: Option<u32>,
    pub by: Option<String>,
    pub on_touch: Option<IgnoreOnTouch>,
    pub on_touch_scope: Option<TouchScope>,
    pub reason: Option<String>,
    /// A key that cannot be read (`codelaw/unknown-suppression`).
    pub unreadable: Option<String>,
    /// What is missing or contradicts (`codelaw/incomplete-ignore`).
    pub incomplete: Vec<String>,
}

impl IgnoreEntry {
    /// The line covers something: it was read and is complete.
    pub fn valid(&self) -> bool {
        self.unreadable.is_none() && self.incomplete.is_empty()
    }

    /// `on_touch`, the line's or the policy's; `forever` never revokes.
    pub fn on_touch(&self, policy: &IgnorePolicy) -> IgnoreOnTouch {
        if self.forever { IgnoreOnTouch::Keep } else { self.on_touch.unwrap_or(policy.on_touch) }
    }

    /// `on_touch_scope`, the line's or the policy's; a glob is always `file`.
    pub fn scope(&self, policy: &IgnorePolicy) -> TouchScope {
        match self.target {
            IgnoreTarget::Glob(_) => TouchScope::File,
            IgnoreTarget::Function { .. } => self.on_touch_scope.unwrap_or(policy.on_touch_scope),
        }
    }
}

/// Every line of a `.codelawignore` text, checked against `policy`.
pub fn parse(text: &str, policy: &IgnorePolicy) -> Vec<IgnoreEntry> {
    let mut out = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        out.push(parse_line(line, u32::try_from(i + 1).unwrap_or(u32::MAX), policy));
    }
    out
}

fn parse_line(line: &str, n: u32, policy: &IgnorePolicy) -> IgnoreEntry {
    let (head, reason) = match line.split_once(':') {
        Some((h, r)) => (h, Some(r.trim().to_owned()).filter(|r| !r.is_empty())),
        None => (line, None),
    };
    let mut words = head.split_whitespace();
    let target_text = words.next().unwrap_or_default();
    let target = match target_text.split_once('#') {
        Some((path, name)) => IgnoreTarget::Function { path: path.trim_start_matches("./").to_owned(), name: name.to_owned() },
        None => IgnoreTarget::Glob(target_text.trim_start_matches("./").to_owned()),
    };
    let mut e = IgnoreEntry {
        line: n,
        target,
        forever: false,
        expires: None,
        warn_days: None,
        by: None,
        on_touch: None,
        on_touch_scope: None,
        reason,
        unreadable: None,
        incomplete: Vec::new(),
    };
    for w in words {
        let r: Result<(), String> = match w.split_once('=') {
            None if w == "forever" => {
                e.forever = true;
                Ok(())
            }
            None => Err(format!("`{w}` is not a key (forever, expires=, warn_days=, by=, on_touch=, on_touch_scope=)")),
            Some(("expires", v)) => Date::parse(v).map(|d| e.expires = Some(d)).map_err(|err| format!("`expires=`: {err}")),
            Some(("warn_days", v)) => v.parse::<u32>().map(|d| e.warn_days = Some(d)).map_err(|_| format!("`warn_days={v}` is not a whole number of days")),
            Some(("by", v)) => {
                e.by = Some(v.to_owned());
                Ok(())
            }
            Some(("on_touch", v)) => IgnoreOnTouch::parse(v).map(|t| e.on_touch = Some(t)),
            Some(("on_touch_scope", v)) => TouchScope::parse(v).map(|t| e.on_touch_scope = Some(t)),
            Some((k, _)) => Err(format!("`{k}=` is not a key (expires, warn_days, by, on_touch, on_touch_scope)")),
        };
        if let Err(err) = r {
            e.unreadable.get_or_insert(err);
        }
    }
    if target_text.is_empty() {
        e.unreadable.get_or_insert("the line names no file, glob or function".into());
    }
    if let IgnoreTarget::Function { path, name } = &e.target
        && (path.is_empty() || name.is_empty())
    {
        e.unreadable.get_or_insert("a function target is `<path>#<qualified name>`".into());
    }
    if e.unreadable.is_some() {
        return e;
    }
    // What the line must carry (the four checks of decision 12).
    if (e.forever || e.expires.is_some()) && e.reason.is_none() {
        e.incomplete.push("a reason after a colon (`: <why>`), which a line with `expires=` or `forever` always needs".into());
    }
    if policy.require_expiry && !e.forever && e.expires.is_none() {
        e.incomplete.push("`expires=YYYY-MM-DD` or `forever` (`[coverage.ignore] require_expiry` is on)".into());
    }
    if policy.require_on_touch && !e.forever && e.on_touch.is_none() {
        e.incomplete.push("`on_touch=error|warn|keep` (`[coverage.ignore] require_on_touch` is on)".into());
    }
    if e.forever {
        for (set, key) in [(e.expires.is_some(), "expires="), (e.warn_days.is_some(), "warn_days="), (e.on_touch.is_some(), "on_touch="), (e.on_touch_scope.is_some(), "on_touch_scope=")] {
            if set {
                e.incomplete.push(format!("no `{key}` with `forever`, which never expires and is never revoked"));
            }
        }
    }
    if matches!(e.target, IgnoreTarget::Glob(_)) && e.on_touch_scope == Some(TouchScope::Function) {
        e.incomplete.push("no `on_touch_scope=function` on a glob, which has no single function to watch".into());
    }
    if let Some(by) = e.by.as_deref().filter(|b| !valid_email(b)) {
        e.incomplete.push(format!("`by={by}` to be an email address"));
    }
    e
}

/// The text without the lines `remove` names (1-based): `codelaw fix` on
/// `codelaw/stale-ignore`.
pub fn without_lines(text: &str, remove: &[u32]) -> String {
    let mut out = String::with_capacity(text.len());
    for (i, l) in text.split_inclusive('\n').enumerate() {
        if !remove.contains(&u32::try_from(i + 1).unwrap_or(u32::MAX)) {
            out.push_str(l);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "# .codelawignore — defaults from codelaw.toml: on_touch=error, on_touch_scope=file

# Never tested, never revoked
src/utils/http.ts#get        forever: thin wrapper over fetch, nothing to test
src/generated/**             forever: regenerated from the OpenAPI spec

scripts/*.py                 on_touch=error expires=2027-01-31: one-off migration scripts
src/orders/list.tsx#formatRow  on_touch=error expires=2026-12-31 by=a@example.com: covered by the e2e suite until ORD-1490
app/billing.py#Invoice.legacy_total  on_touch=error on_touch_scope=function expires=2027-03-01 by=lead@example.com: replaced in ORD-1500
app/legacy/report.py#build_report  on_touch=warn expires=2027-06-30: migration in progress
";

    #[test]
    fn the_sample_reads_whole() {
        let e = parse(SAMPLE, &IgnorePolicy::default());
        assert_eq!(e.len(), 6);
        assert!(e.iter().all(IgnoreEntry::valid), "{e:#?}");
        assert_eq!(e[0].target, IgnoreTarget::Function { path: "src/utils/http.ts".into(), name: "get".into() });
        assert!(e[0].forever);
        assert_eq!(e[0].line, 4);
        assert_eq!(e[1].target, IgnoreTarget::Glob("src/generated/**".into()));
        assert_eq!(e[2].expires, Some(Date::parse("2027-01-31").unwrap()));
        assert_eq!(e[2].reason.as_deref(), Some("one-off migration scripts"));
        assert_eq!(e[3].by.as_deref(), Some("a@example.com"));
        assert_eq!(e[4].target, IgnoreTarget::Function { path: "app/billing.py".into(), name: "Invoice.legacy_total".into() });
        assert_eq!(e[4].scope(&IgnorePolicy::default()), TouchScope::Function);
        assert_eq!(e[5].on_touch(&IgnorePolicy::default()), IgnoreOnTouch::Warn);
        assert_eq!(e[0].on_touch(&IgnorePolicy::default()), IgnoreOnTouch::Keep, "forever is never revoked");
        assert_eq!(e[2].scope(&IgnorePolicy::default()), TouchScope::File, "a glob is always file");
    }

    fn one(line: &str, policy: &IgnorePolicy) -> IgnoreEntry {
        parse(line, policy).remove(0)
    }

    #[test]
    fn every_missing_field_is_named() {
        let p = IgnorePolicy::default();
        assert!(one("a.ts#f forever", &p).incomplete[0].contains("reason"));
        assert!(one("a.ts#f expires=2027-01-01 on_touch=error", &p).incomplete[0].contains("reason"));
        let e = one("a.ts#f: why", &p);
        assert_eq!(e.incomplete.len(), 2, "{e:?}");
        assert!(e.incomplete[0].contains("require_expiry") && e.incomplete[1].contains("require_on_touch"));
        assert!(one("a.ts#f expires=2027-01-01: why", &p).incomplete[0].contains("on_touch="));
        assert!(!one("a.ts#f expires=2027-01-01: why", &p).valid());
    }

    #[test]
    fn the_switches_turn_their_checks_off_but_never_the_reason() {
        let p = IgnorePolicy { require_expiry: false, require_on_touch: false, ..IgnorePolicy::default() };
        assert!(one("a.ts#f: why", &p).valid());
        assert!(one("a.ts", &p).valid(), "no date, no forever: no reason needed");
        assert!(!one("a.ts expires=2027-01-01", &p).valid(), "a date always needs a reason");
    }

    #[test]
    fn contradictions_are_errors_at_the_line() {
        let p = IgnorePolicy::default();
        assert!(one("a.ts#f forever expires=2027-01-01: why", &p).incomplete[0].contains("expires="));
        assert!(one("a.ts#f forever on_touch=keep: why", &p).incomplete[0].contains("on_touch="));
        assert!(one("a.ts#f forever warn_days=3: why", &p).incomplete[0].contains("warn_days="));
        assert!(one("src/** on_touch=error on_touch_scope=function expires=2027-01-01: why", &p).incomplete[0].contains("glob"));
        assert!(one("a.ts#f on_touch=error expires=2027-01-01 by=nobody: why", &p).incomplete[0].contains("email"));
    }

    #[test]
    fn an_unreadable_key_is_not_incomplete() {
        let p = IgnorePolicy::default();
        assert!(one("a.ts#f forevr: why", &p).unreadable.as_deref().unwrap().contains("forevr"));
        assert!(one("a.ts#f on_touch=sometimes expires=2027-01-01: why", &p).unreadable.is_some());
        assert!(one("a.ts#f expires=2027-13-01: why", &p).unreadable.is_some());
        assert!(one("a.ts# forever: why", &p).unreadable.is_some());
    }

    #[test]
    fn the_policy_table_is_checked() {
        assert_eq!(IgnorePolicy::from_table(None, None, None, None).unwrap(), IgnorePolicy::default());
        let p = IgnorePolicy::from_table(Some(false), Some(false), Some("warn"), Some("function")).unwrap();
        assert_eq!((p.on_touch, p.on_touch_scope), (IgnoreOnTouch::Warn, TouchScope::Function));
        assert!(IgnorePolicy::from_table(None, None, Some("never"), None).unwrap_err().contains("[coverage.ignore]"));
    }

    #[test]
    fn removing_lines_keeps_the_rest_byte_for_byte() {
        assert_eq!(without_lines("a\nb\r\nc\n", &[2]), "a\nc\n");
        assert_eq!(without_lines("a\nb", &[2]), "a\n");
    }
}
