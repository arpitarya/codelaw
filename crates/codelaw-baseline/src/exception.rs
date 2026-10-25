//! Dated exceptions (SR-BASELINE decision 9): `expires`, the first day an
//! exception no longer holds; the warning window before it; the opt-in
//! `[exceptions] require_expiry`; and the `added_by` email. The same keys
//! on a baseline entry, a deprecations-registry entry and an inline
//! `codelaw-ignore`. Nothing here reads the clock (L5 decision 2): the day
//! is the as-of date the engine passes.

use crate::date::Date;

/// A warning: the exception expires within its warning window.
pub const EXPIRING_RULE: &str = "codelaw/expiring-exception";
/// An error: the exception expired and no longer covers anything.
pub const EXPIRED_RULE: &str = "codelaw/expired-exception";
/// An error under `require_expiry = true`: an exception without a date.
pub const WITHOUT_EXPIRY_RULE: &str = "codelaw/exception-without-expiry";
/// An error: a malformed `added_by`, or one that is not the git author.
pub const AUTHOR_RULE: &str = "codelaw/exception-author";

/// `[exceptions] warn_days` when the table does not set it.
pub const DEFAULT_WARN_DAYS: u32 = 3;

/// The `[exceptions]` table of `codelaw.toml`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Exceptions {
    /// Every exception must carry `expires`.
    pub require_expiry: bool,
    /// How many days before `expires` the warning starts.
    pub warn_days: u32,
}

impl Default for Exceptions {
    fn default() -> Self {
        Exceptions { require_expiry: false, warn_days: DEFAULT_WARN_DAYS }
    }
}

impl Exceptions {
    /// The table's values; the loader has already refused an unknown key.
    pub fn from_table(require_expiry: Option<bool>, warn_days: Option<i64>) -> Result<Exceptions, String> {
        let warn_days = match warn_days {
            None => DEFAULT_WARN_DAYS,
            Some(n) => u32::try_from(n).map_err(|_| format!("[exceptions]: `warn_days = {n}` must be a whole number of days, 0 or more"))?,
        };
        Ok(Exceptions { require_expiry: require_expiry.unwrap_or(false), warn_days })
    }
}

/// Where an exception stands on the as-of date.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expiry {
    /// No date, no as-of date, or before the warning window.
    Holds,
    /// In the warning window: `days` (1 or more) until `expires`.
    Expiring { days: i64, expires: Date },
    /// On or after `expires`: it no longer covers anything.
    Expired { expires: Date },
}

/// The timeline of decision 9: from `warn_from` (or `expires − warn_days`)
/// a warning, on and after `expires` an error. Without an as-of date
/// deadlines are not evaluated (decision 4).
pub fn expiry(as_of: Option<Date>, expires: Option<Date>, warn_from: Option<Date>, warn_days: u32) -> Expiry {
    let (Some(today), Some(expires)) = (as_of, expires) else { return Expiry::Holds };
    if today >= expires {
        return Expiry::Expired { expires };
    }
    let from = warn_from.unwrap_or_else(|| expires.add_days(-i64::from(warn_days)));
    if today >= from { Expiry::Expiring { days: today.days_until(expires), expires } } else { Expiry::Holds }
}

/// "in 2 days (2026-11-15)".
pub fn when(days: i64, expires: Date) -> String {
    if days == 1 { format!("in 1 day ({expires})") } else { format!("in {days} days ({expires})") }
}

/// `until` (the last allowed day, read for one minor release) and
/// `expires` (the first failing day) as one date: `until + 1`. Both on
/// one entry is an error.
pub fn expires_of(until: Option<Date>, expires: Option<Date>) -> Result<Option<Date>, String> {
    match (until, expires) {
        (Some(_), Some(_)) => Err("`until` and `expires` on one entry: keep `expires`, the first day it no longer holds".into()),
        (Some(u), None) => Ok(Some(u.add_days(1))),
        (None, e) => Ok(e),
    }
}

/// Whether `s` is an email address: one `@`, a local part, a dotted
/// domain, no spaces. A format check, not a lookup.
pub fn valid_email(s: &str) -> bool {
    let Some((local, domain)) = s.split_once('@') else { return false };
    let ok_chars = |p: &str| !p.is_empty() && p.chars().all(|c| !c.is_whitespace() && !c.is_control() && c != '@' && c != '<' && c != '>' && c != ',');
    ok_chars(local)
        && ok_chars(domain)
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.contains("..")
}

/// The 1-based line of each `[[<name>]]` header in a TOML text, in order:
/// where each entry of an array of tables starts.
pub fn header_lines(text: &str, name: &str) -> Vec<u32> {
    let want = format!("[[{name}]]");
    text.lines()
        .enumerate()
        .filter(|(_, l)| {
            let code = l.split('#').next().unwrap_or("");
            code.chars().filter(|c| !c.is_whitespace()).collect::<String>() == want
        })
        .map(|(i, _)| i as u32 + 1)
        .collect()
}
