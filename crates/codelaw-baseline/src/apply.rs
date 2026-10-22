//! What each occurrence becomes under the baseline (SR-BASELINE decision
//! 2), the stale entries, and the two writers: `prune` lowers counts, `add`
//! records the current ones (decision 5).

use crate::date::Date;
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
        let until = e.until.or(policy.until);
        let warn_from = e.warn_from.or(policy.warn_from);
        let on_touch = e.on_touch.or(policy.on_touch).unwrap_or_default();
        let touched = ctx.touched.contains(&file);
        let (severity, status) = if touched && on_touch != OnTouch::Keep {
            let sev = if on_touch == OnTouch::Error { Severity::Error } else { Severity::Warn };
            (sev, format!("you edited this file: migrate its {} now (on_touch = {})", uses(n), on_touch.name()))
        } else {
            match (ctx.as_of, until, warn_from) {
                (Some(today), Some(u), _) if today > u => (Severity::Error, format!("its baseline deadline {u} has passed")),
                (Some(today), Some(u), Some(w)) if today >= w => {
                    let days = today.days_until(u);
                    let when = if days == 0 { "today".to_owned() } else if days == 1 { "in 1 day".to_owned() } else { format!("in {days} days") };
                    (Severity::Warn, format!("grandfathered until {u}: expires {when}"))
                }
                (Some(today), None, Some(w)) if today >= w => (Severity::Warn, format!("grandfathered, warning since {w}")),
                (_, Some(u), _) => (Severity::Info, format!("grandfathered until {u} ({} left in this file)", n)),
                (_, None, _) => (Severity::Info, format!("grandfathered ({} left in this file)", n)),
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
    Outcome { diagnostics, stale }
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
    pub until: Option<Date>,
    pub warn_from: Option<Date>,
    pub on_touch: Option<OnTouch>,
    pub reason: Option<&'a str>,
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
                e.until = o.until.or(e.until);
                e.warn_from = o.warn_from.or(e.warn_from);
                e.on_touch = o.on_touch.or(e.on_touch);
                if let Some(r) = o.reason {
                    e.reason = Some(r.to_owned());
                }
            }
            None => b.entries.push(Entry {
                file: file.clone(),
                symbol: symbol.clone(),
                count: n,
                until: o.until,
                warn_from: o.warn_from,
                on_touch: Some(o.on_touch.unwrap_or_default()),
                reason: o.reason.map(str::to_owned),
            }),
        }
    }
    b.sort();
    b
}
