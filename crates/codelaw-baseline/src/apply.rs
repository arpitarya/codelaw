//! What each occurrence becomes under the baseline (SR-BASELINE decision
//! 2), the stale entries, and the two writers: `prune` lowers counts, `add`
//! records the current ones (decision 5).

use crate::date::Date;
use crate::exception::{EXPIRED_RULE, EXPIRING_RULE, Exceptions, Expiry, expiry, when};
use crate::file::{BaselineFile, Baselines, Entry, OnTouch};
use crate::registry::Policy;
use codelaw_core::{Diagnostic, Severity};
use std::collections::{BTreeMap, BTreeSet};

/// A finding the baseline may grandfather.
#[derive(Clone, Debug)]
pub struct Occurrence {
    pub diag: Diagnostic,
    /// The deprecated symbol, for `graph/no-deprecated-usage`.
    pub symbol: Option<String>,
    /// The deprecation's policy, which an entry inherits where it is silent.
    pub policy: Policy,
}

impl Occurrence {
    pub fn plain(diag: Diagnostic) -> Occurrence {
        Occurrence { diag, symbol: None, policy: Policy::default() }
    }
}

/// The inputs that are not files (L5 decision 2).
pub struct Context<'a> {
    /// The as-of date; `None` when there is none and deadlines are not
    /// evaluated (decision 4).
    pub as_of: Option<Date>,
    /// The change set, project-relative. Empty: nothing is touched.
    pub touched: &'a BTreeSet<String>,
    /// Whether the run saw every finding of `rule` in `file`, so a count
    /// below the entry's is real (a stale entry), not a partial run.
    pub evaluated: &'a dyn Fn(&str, &str) -> bool,
    /// `[exceptions]`: the default warning window.
    pub exceptions: Exceptions,
}

/// An entry whose file now has fewer occurrences than it allows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stale {
    pub rule: String,
    pub file: String,
    pub symbol: Option<String>,
    pub allowed: usize,
    pub found: usize,
}

pub struct Outcome {
    pub diagnostics: Vec<Diagnostic>,
    pub stale: Vec<Stale>,
    /// `codelaw/expiring-exception` and `codelaw/expired-exception`, at
    /// the entry's own line (decision 9).
    pub notes: Vec<Note>,
}

/// A diagnostic about a baseline entry itself, reported in its baseline
/// file at its `[[allow]]` line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note {
    pub rule: &'static str,
    /// The rule whose baseline file holds the entry.
    pub baseline: String,
    pub line: u32,
    pub message: String,
}

/// A rule, a file and a symbol.
pub type Key = (String, String, Option<String>);

/// Occurrences per rule, file and symbol.
pub type Counts = BTreeMap<Key, usize>;

fn key(o: &Occurrence) -> Key {
    (o.diag.rule.clone(), o.diag.file.clone(), o.symbol.clone())
}

/// Occurrences per rule, file and symbol.
pub fn counts(occurrences: &[Occurrence]) -> BTreeMap<Key, usize> {
    let mut out = BTreeMap::new();
    for o in occurrences {
        *out.entry(key(o)).or_default() += 1;
    }
    out
}

fn uses(n: usize) -> String {
    if n == 1 { "1 occurrence".into() } else { format!("{n} occurrences") }
}

/// Apply every rule's baseline to the occurrences.
pub fn apply(occurrences: Vec<Occurrence>, baselines: &Baselines, ctx: &Context<'_>) -> Outcome {
    let mut groups: BTreeMap<Key, Vec<Occurrence>> = BTreeMap::new();
    for o in occurrences {
        groups.entry(key(&o)).or_default().push(o);
    }
    let mut diagnostics = Vec::new();
    let mut notes = Vec::new();
    let mut found: BTreeMap<Key, usize> = BTreeMap::new();
    for ((rule, file, symbol), group) in groups {
        let n = group.len();
        found.insert((rule.clone(), file.clone(), symbol.clone()), n);
        let entry = baselines.rules.get(&rule).and_then(|b| b.find(&file, symbol.as_deref()));
        let Some(e) = entry else {
            diagnostics.extend(group.into_iter().map(|o| o.diag));
            continue;
        };
        if n > e.count {
            // Counts can't tell which one is new, so each is reported at
            // its rule's own severity (ESLint's bulk suppressions do the same).
            for o in group {
                let mut d = o.diag;
                d.message = format!("{} ({n} in this file; the baseline allows {})", d.message, e.count);
                diagnostics.push(d);
            }
            continue;
        }
        let policy = &group[0].policy;
        let expires = e.expires.or(policy.expires);
        let warn_from = e.warn_from.or(policy.warn_from);
        let warn_days = e.warn_days.or(policy.warn_days).unwrap_or(ctx.exceptions.warn_days);
        let on_touch = e.on_touch.or(policy.on_touch).unwrap_or_default();
        let whose = match &symbol {
            Some(s) => format!("The baseline entry of `{rule}` for `{file}` (`{s}`)"),
            None => format!("The baseline entry of `{rule}` for `{file}`"),
        };
        // Decision 9: on and after `expires` the entry covers nothing, and
        // the findings come back at the rule's own severity, touched or not.
        match expiry(ctx.as_of, expires, warn_from, warn_days) {
            Expiry::Expired { expires } => {
                notes.push(Note {
                    rule: EXPIRED_RULE,
                    baseline: rule.clone(),
                    line: e.line,
                    message: format!("{whose} expired on {expires}: its {} are reported again", uses(n)),
                });
                for o in group {
                    let mut d = o.diag;
                    d.message = format!("{}; its baseline entry expired on {expires}", d.message);
                    diagnostics.push(d);
                }
                continue;
            }
            Expiry::Expiring { days, expires } => notes.push(Note {
                rule: EXPIRING_RULE,
                baseline: rule.clone(),
                line: e.line,
                message: format!("{whose} expires {}: then its {} fail", when(days, expires), uses(n)),
            }),
            Expiry::Holds => {}
        }
        let touched = ctx.touched.contains(&file);
        let (severity, status) = if touched && on_touch != OnTouch::Keep {
            let sev = if on_touch == OnTouch::Error { Severity::Error } else { Severity::Warn };
            (sev, format!("you edited this file: migrate its {} now (on_touch = {})", uses(n), on_touch.name()))
        } else {
            match (ctx.as_of, expires, warn_from) {
                (Some(today), None, Some(w)) if today >= w => (Severity::Warn, format!("grandfathered, warning since {w}")),
                (_, Some(x), _) => (Severity::Info, format!("grandfathered, expires {x} ({n} left in this file)")),
                (_, None, _) => (Severity::Info, format!("grandfathered ({n} left in this file)")),
            }
        };
        for o in group {
            let mut d = o.diag;
            d.severity = severity;
            d.message = format!("{}; {status}", d.message);
            diagnostics.push(d);
        }
    }
    let mut stale = Vec::new();
    for b in baselines.rules.values() {
        for e in &b.entries {
            if !(ctx.evaluated)(&b.rule, &e.file) {
                continue;
            }
            let n = found.get(&(b.rule.clone(), e.file.clone(), e.symbol.clone())).copied().unwrap_or(0);
            if n < e.count {
                stale.push(Stale { rule: b.rule.clone(), file: e.file.clone(), symbol: e.symbol.clone(), allowed: e.count, found: n });
            }
        }
    }
    Outcome { diagnostics, stale, notes }
}

/// Lower every evaluated entry to what the run found and drop the ones at
/// zero; never raise one (decision 5). The baselines that changed.
pub fn prune(baselines: &Baselines, counts: &BTreeMap<Key, usize>, evaluated: &dyn Fn(&str, &str) -> bool) -> Vec<BaselineFile> {
    let mut changed = Vec::new();
    for b in baselines.rules.values() {
        let mut next = b.clone();
        next.entries.retain_mut(|e| {
            if !evaluated(&b.rule, &e.file) {
                return true;
            }
            let n = counts.get(&(b.rule.clone(), e.file.clone(), e.symbol.clone())).copied().unwrap_or(0);
            e.count = e.count.min(n);
            e.count > 0
        });
        if next != *b {
            changed.push(next);
        }
    }
    changed
}

/// What `baseline add` writes for a rule.
pub struct AddOptions<'a> {
    pub rule: &'a str,
    /// Only these files; every file when empty.
    pub files: &'a BTreeSet<String>,
    /// Only this symbol (`codelaw deprecate`).
    pub symbol: Option<&'a str>,
    pub expires: Option<Date>,
    pub warn_from: Option<Date>,
    pub warn_days: Option<u32>,
    pub on_touch: Option<OnTouch>,
    pub reason: Option<&'a str>,
    pub added_by: Option<&'a str>,
}

/// Record the current counts of a rule: a new entry per file (and symbol)
/// with occurrences, or an existing one set to the current count with the
/// given deadline and reason. The rule's baseline after the change.
pub fn add(existing: Option<&BaselineFile>, counts: &BTreeMap<Key, usize>, o: &AddOptions<'_>) -> BaselineFile {
    let mut b = existing.cloned().unwrap_or_else(|| BaselineFile { rule: o.rule.to_owned(), entries: Vec::new() });
    for ((rule, file, symbol), &n) in counts {
        if rule != o.rule || n == 0 || (!o.files.is_empty() && !o.files.contains(file)) || (o.symbol.is_some() && symbol.as_deref() != o.symbol) {
            continue;
        }
        match b.entries.iter_mut().find(|e| e.file == *file && e.symbol == *symbol) {
            Some(e) => {
                e.count = n;
                e.expires = o.expires.or(e.expires);
                e.warn_from = o.warn_from.or(e.warn_from);
                e.warn_days = o.warn_days.or(e.warn_days);
                e.on_touch = o.on_touch.or(e.on_touch);
                if let Some(r) = o.reason {
                    e.reason = Some(r.to_owned());
                }
                if let Some(a) = o.added_by {
                    e.added_by = Some(a.to_owned());
                }
            }
            None => b.entries.push(Entry {
                file: file.clone(),
                symbol: symbol.clone(),
                count: n,
                expires: o.expires,
                warn_from: o.warn_from,
                warn_days: o.warn_days,
                on_touch: Some(o.on_touch.unwrap_or_default()),
                reason: o.reason.map(str::to_owned),
                added_by: o.added_by.map(str::to_owned),
                line: 0,
            }),
        }
    }
    b.sort();
    b
}
